use crate::{Error, ErrorKind, Result};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::process::Stdio;
use std::thread;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStreamMode {
    Inherit,
    Capture,
    Null,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunOptions {
    pub cmd: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: Vec<(String, String)>,
    pub env_remove: Vec<String>,
    pub stdin: Option<Vec<u8>>,
    pub stdout: RunStreamMode,
    pub stderr: RunStreamMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunResult {
    pub ok: bool,
    pub code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

pub fn run_command(options: &RunOptions, current_dir: &Path) -> Result<RunResult> {
    let resolved_cwd = resolve_run_cwd(current_dir, options.cwd.as_deref());

    let mut command = ProcessCommand::new(&options.cmd);
    command.args(&options.args);
    command.current_dir(&resolved_cwd);

    for key in &options.env_remove {
        command.env_remove(key);
    }

    for (key, value) in &options.env {
        command.env(key, value);
    }

    configure_stdio(
        &mut command,
        options.stdin.is_some(),
        options.stdout,
        options.stderr,
    );

    let mut child = command
        .spawn()
        .map_err(|err| build_run_io_error(&options.cmd, &resolved_cwd, err))?;

    let stdin_handle = options.stdin.as_ref().and_then(|bytes| {
        child.stdin.take().map(|mut pipe| {
            let bytes = bytes.clone();
            thread::spawn(move || -> std::io::Result<()> {
                pipe.write_all(&bytes)?;
                pipe.flush()?;
                Ok(())
            })
        })
    });

    let stdout_handle = child.stdout.take().map(spawn_reader_thread);
    let stderr_handle = child.stderr.take().map(spawn_reader_thread);

    let status = child
        .wait()
        .map_err(|err| build_run_io_error(&options.cmd, &resolved_cwd, err))?;

    if let Some(handle) = stdin_handle {
        finish_io_thread(handle, "stdin", &options.cmd, &resolved_cwd)?;
    }

    let stdout = collect_process_output(
        stdout_handle,
        options.stdout,
        "stdout",
        &options.cmd,
        &resolved_cwd,
    )?;
    let stderr = collect_process_output(
        stderr_handle,
        options.stderr,
        "stderr",
        &options.cmd,
        &resolved_cwd,
    )?;

    Ok(RunResult {
        ok: status.success(),
        code: status.code(),
        stdout: bytes_to_captured_string(&stdout, options.stdout),
        stderr: bytes_to_captured_string(&stderr, options.stderr),
    })
}

pub fn resolve_run_cwd(current_dir: &Path, cwd: Option<&str>) -> PathBuf {
    let base = current_dir.to_path_buf();
    match cwd {
        Some(dir) => {
            let path = PathBuf::from(dir);
            if path.is_absolute() {
                path
            } else {
                base.join(path)
            }
        }
        None => base,
    }
}

pub fn format_command_for_display(cmd: &str, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(shell_quote(cmd));
    for arg in args {
        parts.push(shell_quote(arg));
    }
    parts.join(" ")
}

pub fn format_run_failed_message(
    cmd_for_error: &str,
    code: Option<i32>,
    stderr: Option<&str>,
) -> String {
    let code = code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "terminated by signal".to_string());
    let mut message = format!("ptool.run command `{cmd_for_error}` failed with status {code}");
    if let Some(stderr) = stderr {
        let stderr = stderr.trim();
        if !stderr.is_empty() {
            message.push_str(": ");
            message.push_str(stderr);
        }
    }
    message
}

fn configure_stdio(
    command: &mut ProcessCommand,
    has_stdin: bool,
    stdout_mode: RunStreamMode,
    stderr_mode: RunStreamMode,
) {
    command.stdin(if has_stdin {
        Stdio::piped()
    } else {
        Stdio::inherit()
    });
    command.stdout(match stdout_mode {
        RunStreamMode::Inherit => Stdio::inherit(),
        RunStreamMode::Capture => Stdio::piped(),
        RunStreamMode::Null => Stdio::null(),
    });
    command.stderr(match stderr_mode {
        RunStreamMode::Inherit => Stdio::inherit(),
        RunStreamMode::Capture => Stdio::piped(),
        RunStreamMode::Null => Stdio::null(),
    });
}

fn bytes_to_captured_string(bytes: &[u8], mode: RunStreamMode) -> Option<String> {
    match mode {
        RunStreamMode::Capture => Some(String::from_utf8_lossy(bytes).to_string()),
        RunStreamMode::Inherit | RunStreamMode::Null => None,
    }
}

fn spawn_reader_thread<R>(mut reader: R) -> thread::JoinHandle<std::io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    })
}

fn finish_io_thread(
    handle: thread::JoinHandle<std::io::Result<()>>,
    stream_name: &str,
    cmd: &str,
    cwd: &Path,
) -> Result<()> {
    match handle.join() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(build_run_io_error(
            cmd,
            cwd,
            std::io::Error::new(
                err.kind(),
                format!("failed to write child {stream_name}: {err}"),
            ),
        )),
        Err(_) => Err(build_run_io_error(
            cmd,
            cwd,
            std::io::Error::other(format!("child {stream_name} worker panicked")),
        )),
    }
}

fn collect_process_output(
    handle: Option<thread::JoinHandle<std::io::Result<Vec<u8>>>>,
    mode: RunStreamMode,
    stream_name: &str,
    cmd: &str,
    cwd: &Path,
) -> Result<Vec<u8>> {
    match mode {
        RunStreamMode::Capture => match handle {
            Some(handle) => match handle.join() {
                Ok(Ok(bytes)) => Ok(bytes),
                Ok(Err(err)) => Err(build_run_io_error(
                    cmd,
                    cwd,
                    std::io::Error::new(
                        err.kind(),
                        format!("failed to read child {stream_name}: {err}"),
                    ),
                )),
                Err(_) => Err(build_run_io_error(
                    cmd,
                    cwd,
                    std::io::Error::other(format!("child {stream_name} worker panicked")),
                )),
            },
            None => Ok(Vec::new()),
        },
        RunStreamMode::Inherit | RunStreamMode::Null => Ok(Vec::new()),
    }
}

fn shell_quote(value: &str) -> String {
    const SAFE_CHARS: &str =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._/:@%+=";
    if !value.is_empty() && value.chars().all(|ch| SAFE_CHARS.contains(ch)) {
        return value.to_string();
    }

    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

fn build_run_io_error(cmd: &str, cwd: &Path, err: std::io::Error) -> Error {
    Error::new(
        ErrorKind::Io,
        format!(
            "failed to run command `{cmd}` in `{}`: {err}",
            cwd.display()
        ),
    )
    .with_op("ptool.run")
    .with_cmd(cmd)
    .with_detail(format!("cwd: {}", cwd.display()))
}
