use crate::{Error, ErrorKind, Result};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::process::Stdio;
use std::thread;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunStdin {
    Inherit,
    Bytes(Vec<u8>),
    File { path: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunStreamMode {
    Inherit,
    Capture,
    Null,
    File { path: String, append: bool },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunOptions {
    pub cmd: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: Vec<(String, String)>,
    pub env_remove: Vec<String>,
    pub stdin: RunStdin,
    pub trim: bool,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecOptions {
    pub cmd: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: Vec<(String, String)>,
    pub env_remove: Vec<String>,
    pub stdin: RunStdin,
    pub stdout: RunStreamMode,
    pub stderr: RunStreamMode,
}

enum OutputHandle {
    Capture(thread::JoinHandle<io::Result<Vec<u8>>>),
    Inherit(thread::JoinHandle<io::Result<()>>),
}

pub fn run_command(options: &RunOptions, current_dir: &Path) -> Result<RunResult> {
    let resolved_cwd = resolve_run_cwd(current_dir, options.cwd.as_deref());

    let mut command = ProcessCommand::new(&options.cmd);
    configure_command(
        &mut command,
        &options.cmd,
        &options.args,
        &resolved_cwd,
        &options.env,
        &options.env_remove,
    );

    configure_stdio(
        &mut command,
        &options.stdin,
        &options.stdout,
        &options.stderr,
        &resolved_cwd,
        &options.cmd,
    )?;

    let mut child = command
        .spawn()
        .map_err(|err| build_run_io_error(&options.cmd, &resolved_cwd, err))?;

    let stdin_handle = match &options.stdin {
        RunStdin::Bytes(bytes) => child.stdin.take().map(|mut pipe| {
            let bytes = bytes.clone();
            thread::spawn(move || -> std::io::Result<()> {
                pipe.write_all(&bytes)?;
                pipe.flush()?;
                Ok(())
            })
        }),
        RunStdin::Inherit | RunStdin::File { .. } => None,
    };

    let stdout_handle = child
        .stdout
        .take()
        .map(|reader| spawn_output_thread(reader, &options.stdout, false));
    let stderr_handle = child
        .stderr
        .take()
        .map(|reader| spawn_output_thread(reader, &options.stderr, true));

    let status = child
        .wait()
        .map_err(|err| build_run_io_error(&options.cmd, &resolved_cwd, err))?;

    if let Some(handle) = stdin_handle {
        finish_io_thread(handle, "stdin", &options.cmd, &resolved_cwd)?;
    }

    let stdout = collect_process_output(
        stdout_handle,
        &options.stdout,
        "stdout",
        &options.cmd,
        &resolved_cwd,
    )?;
    let stderr = collect_process_output(
        stderr_handle,
        &options.stderr,
        "stderr",
        &options.cmd,
        &resolved_cwd,
    )?;

    Ok(RunResult {
        ok: status.success(),
        code: status.code(),
        stdout: bytes_to_captured_string(&stdout, &options.stdout, options.trim),
        stderr: bytes_to_captured_string(&stderr, &options.stderr, options.trim),
    })
}

pub fn exec_replace(options: &ExecOptions, current_dir: &Path) -> Result<()> {
    let resolved_cwd = resolve_run_cwd(current_dir, options.cwd.as_deref());
    validate_exec_stdio(options)?;
    let mut command = ProcessCommand::new(&options.cmd);
    configure_command(
        &mut command,
        &options.cmd,
        &options.args,
        &resolved_cwd,
        &options.env,
        &options.env_remove,
    );
    configure_stdio(
        &mut command,
        &options.stdin,
        &options.stdout,
        &options.stderr,
        &resolved_cwd,
        &options.cmd,
    )?;

    exec_current_process(command, &options.cmd, &resolved_cwd)
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
    stdin_mode: &RunStdin,
    stdout_mode: &RunStreamMode,
    stderr_mode: &RunStreamMode,
    cwd: &Path,
    cmd: &str,
) -> Result<()> {
    command.stdin(match stdin_mode {
        RunStdin::Inherit => Stdio::inherit(),
        RunStdin::Bytes(_) => Stdio::piped(),
        RunStdin::File { path } => Stdio::from(open_stdin_redirect_file(path, cwd, cmd)?),
    });
    command.stdout(configure_output_stdio(stdout_mode, "stdout", cwd, cmd)?);
    command.stderr(configure_output_stdio(stderr_mode, "stderr", cwd, cmd)?);
    Ok(())
}

fn configure_command(
    command: &mut ProcessCommand,
    _cmd: &str,
    args: &[String],
    cwd: &Path,
    env: &[(String, String)],
    env_remove: &[String],
) {
    command.args(args);
    command.current_dir(cwd);

    for key in env_remove {
        command.env_remove(key);
    }

    for (key, value) in env {
        command.env(key, value);
    }
}

fn bytes_to_captured_string(bytes: &[u8], mode: &RunStreamMode, trim: bool) -> Option<String> {
    match mode {
        RunStreamMode::Capture => {
            let text = String::from_utf8_lossy(bytes);
            Some(if trim {
                text.trim().to_string()
            } else {
                text.to_string()
            })
        }
        RunStreamMode::Inherit | RunStreamMode::Null | RunStreamMode::File { .. } => None,
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

fn spawn_output_thread<R>(reader: R, mode: &RunStreamMode, use_stderr: bool) -> OutputHandle
where
    R: Read + Send + 'static,
{
    match mode {
        RunStreamMode::Capture => OutputHandle::Capture(spawn_reader_thread(reader)),
        RunStreamMode::Inherit => {
            OutputHandle::Inherit(spawn_prefixed_output_thread(reader, use_stderr))
        }
        RunStreamMode::Null | RunStreamMode::File { .. } => {
            unreachable!("non-piped stream modes must not spawn output threads")
        }
    }
}

fn spawn_prefixed_output_thread<R>(
    mut reader: R,
    use_stderr: bool,
) -> thread::JoinHandle<io::Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        if use_stderr {
            let stderr = io::stderr();
            let mut writer = stderr.lock();
            forward_prefixed_stream(&mut reader, &mut writer)
        } else {
            let stdout = io::stdout();
            let mut writer = stdout.lock();
            forward_prefixed_stream(&mut reader, &mut writer)
        }
    })
}

fn forward_prefixed_stream<R, W>(reader: &mut R, writer: &mut W) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    let mut buffer = [0_u8; 8192];
    let mut at_line_start = true;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            writer.flush()?;
            return Ok(());
        }
        write_prefixed_chunk(writer, &buffer[..read], &mut at_line_start)?;
        writer.flush()?;
    }
}

fn write_prefixed_chunk<W>(writer: &mut W, chunk: &[u8], at_line_start: &mut bool) -> io::Result<()>
where
    W: Write,
{
    for &byte in chunk {
        if *at_line_start {
            writer.write_all(crate::DISPLAY_STREAM_PREFIX.as_bytes())?;
            *at_line_start = false;
        }
        writer.write_all(&[byte])?;
        if byte == b'\n' {
            *at_line_start = true;
        }
    }
    Ok(())
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
    handle: Option<OutputHandle>,
    mode: &RunStreamMode,
    stream_name: &str,
    cmd: &str,
    cwd: &Path,
) -> Result<Vec<u8>> {
    match mode {
        RunStreamMode::Capture => match handle {
            Some(OutputHandle::Capture(handle)) => match handle.join() {
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
            Some(OutputHandle::Inherit(_)) => Err(build_run_io_error(
                cmd,
                cwd,
                std::io::Error::other(format!(
                    "child {stream_name} worker mode mismatch for `{cmd}`"
                )),
            )),
            None => Ok(Vec::new()),
        },
        RunStreamMode::Inherit => match handle {
            Some(OutputHandle::Inherit(handle)) => {
                finish_io_thread(handle, stream_name, cmd, cwd)?;
                Ok(Vec::new())
            }
            Some(OutputHandle::Capture(_)) => Err(build_run_io_error(
                cmd,
                cwd,
                std::io::Error::other(format!(
                    "child {stream_name} worker mode mismatch for `{cmd}`"
                )),
            )),
            None => Ok(Vec::new()),
        },
        RunStreamMode::Null | RunStreamMode::File { .. } => Ok(Vec::new()),
    }
}

fn configure_output_stdio(
    mode: &RunStreamMode,
    stream_name: &str,
    cwd: &Path,
    cmd: &str,
) -> Result<Stdio> {
    match mode {
        RunStreamMode::Inherit => Ok(Stdio::piped()),
        RunStreamMode::Capture => Ok(Stdio::piped()),
        RunStreamMode::Null => Ok(Stdio::null()),
        RunStreamMode::File { path, append } => Ok(Stdio::from(open_output_redirect_file(
            path,
            *append,
            stream_name,
            cwd,
            cmd,
        )?)),
    }
}

fn open_stdin_redirect_file(path: &str, cwd: &Path, cmd: &str) -> Result<File> {
    let resolved_path = resolve_redirect_path(cwd, path);
    File::open(&resolved_path).map_err(|err| {
        build_run_io_error(
            cmd,
            cwd,
            std::io::Error::new(
                err.kind(),
                format!(
                    "failed to open redirected stdin file `{}`: {err}",
                    resolved_path.display()
                ),
            ),
        )
    })
}

fn open_output_redirect_file(
    path: &str,
    append: bool,
    stream_name: &str,
    cwd: &Path,
    cmd: &str,
) -> Result<File> {
    let resolved_path = resolve_redirect_path(cwd, path);
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }
    options.open(&resolved_path).map_err(|err| {
        build_run_io_error(
            cmd,
            cwd,
            std::io::Error::new(
                err.kind(),
                format!(
                    "failed to open redirected {stream_name} file `{}`: {err}",
                    resolved_path.display()
                ),
            ),
        )
    })
}

fn resolve_redirect_path(cwd: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
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

fn validate_exec_stdio(options: &ExecOptions) -> Result<()> {
    if matches!(options.stdin, RunStdin::Bytes(_)) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.exec does not support string stdin; use `stdin = { file = ... }` instead",
        )
        .with_op("ptool.exec")
        .with_cmd(&options.cmd));
    }

    if matches!(options.stdout, RunStreamMode::Capture) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.exec does not support `stdout = \"capture\"`; redirect to a file or use inherit/null instead",
        )
        .with_op("ptool.exec")
        .with_cmd(&options.cmd));
    }

    if matches!(options.stderr, RunStreamMode::Capture) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.exec does not support `stderr = \"capture\"`; redirect to a file or use inherit/null instead",
        )
        .with_op("ptool.exec")
        .with_cmd(&options.cmd));
    }

    Ok(())
}

#[cfg(unix)]
fn exec_current_process(mut command: ProcessCommand, cmd: &str, cwd: &Path) -> Result<()> {
    let err = command.exec();
    Err(build_exec_io_error(cmd, cwd, err))
}

#[cfg(not(unix))]
fn exec_current_process(command: ProcessCommand, cmd: &str, cwd: &Path) -> Result<()> {
    let _ = (command, cmd, cwd);
    Err(Error::new(
        ErrorKind::Unsupported,
        "ptool.exec is not supported on this platform",
    )
    .with_op("ptool.exec"))
}

#[cfg(unix)]
fn build_exec_io_error(cmd: &str, cwd: &Path, err: std::io::Error) -> Error {
    let path = lookup_executable_path(cmd, cwd)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| cmd.to_string());
    Error::new(
        ErrorKind::Io,
        format!(
            "failed to exec command `{cmd}` in `{}`: {err}",
            cwd.display()
        ),
    )
    .with_op("ptool.exec")
    .with_cmd(cmd)
    .with_path(path)
    .with_detail(format!("cwd: {}", cwd.display()))
}

#[cfg(unix)]
fn lookup_executable_path(cmd: &str, cwd: &Path) -> Option<PathBuf> {
    let candidate = PathBuf::from(cmd);
    if candidate.components().count() > 1 || candidate.is_absolute() {
        return Some(if candidate.is_absolute() {
            candidate
        } else {
            cwd.join(candidate)
        });
    }

    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(cmd);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }

    None
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
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
