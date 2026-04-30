use crate::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::process::Stdio;

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

    apply_stream_mode_for_stdout(&mut command, options.stdout);
    apply_stream_mode_for_stderr(&mut command, options.stderr);

    let output = command
        .output()
        .map_err(|err| build_run_io_error(&options.cmd, &resolved_cwd, err))?;

    Ok(RunResult {
        ok: output.status.success(),
        code: output.status.code(),
        stdout: bytes_to_captured_string(&output.stdout, options.stdout),
        stderr: bytes_to_captured_string(&output.stderr, options.stderr),
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

fn apply_stream_mode_for_stdout(command: &mut ProcessCommand, mode: RunStreamMode) {
    match mode {
        RunStreamMode::Inherit => {
            command.stdout(Stdio::inherit());
        }
        RunStreamMode::Capture => {
            command.stdout(Stdio::piped());
        }
        RunStreamMode::Null => {
            command.stdout(Stdio::null());
        }
    }
}

fn apply_stream_mode_for_stderr(command: &mut ProcessCommand, mode: RunStreamMode) {
    match mode {
        RunStreamMode::Inherit => {
            command.stderr(Stdio::inherit());
        }
        RunStreamMode::Capture => {
            command.stderr(Stdio::piped());
        }
        RunStreamMode::Null => {
            command.stderr(Stdio::null());
        }
    }
}

fn bytes_to_captured_string(bytes: &[u8], mode: RunStreamMode) -> Option<String> {
    match mode {
        RunStreamMode::Capture => Some(String::from_utf8_lossy(bytes).to_string()),
        RunStreamMode::Inherit | RunStreamMode::Null => None,
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
