use crate::interactive_output::{
    InteractiveOutputSink, InteractiveOutputViewport, spawn_interactive_output_thread,
};
use crate::{Console, Error, ErrorKind, Result};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Child;
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
pub struct PipeCommand {
    pub cmd: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipeOptions {
    pub commands: Vec<PipeCommand>,
    pub cwd: Option<String>,
    pub env: Vec<(String, String)>,
    pub env_remove: Vec<String>,
    pub stdin: RunStdin,
    pub trim: bool,
    pub stdout: RunStreamMode,
    pub stderr: RunStreamMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipeResult {
    pub ok: bool,
    pub code: Option<i32>,
    pub codes: Vec<Option<i32>>,
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

struct PipelineStageStdio<'a> {
    stdin_mode: &'a RunStdin,
    stdout_mode: &'a RunStreamMode,
    stderr_mode: &'a RunStreamMode,
    cwd: &'a Path,
    pipeline: &'a str,
    is_first: bool,
    is_last: bool,
    previous_stdout: Option<std::process::ChildStdout>,
}

pub fn run_command(
    options: &RunOptions,
    current_dir: &Path,
    console: &Console,
) -> Result<RunResult> {
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
    let interactive_output = InteractiveOutputViewport::maybe_new(
        *console,
        matches!(options.stdout, RunStreamMode::Inherit),
        matches!(options.stderr, RunStreamMode::Inherit),
    );
    let interactive_sink = interactive_output
        .as_ref()
        .map(InteractiveOutputViewport::sink);

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

    let stdout_handle = child.stdout.take().map(|reader| {
        spawn_output_thread(
            reader,
            &options.stdout,
            false,
            interactive_sink.clone(),
            *console,
        )
    });
    let stderr_handle = child.stderr.take().map(|reader| {
        spawn_output_thread(reader, &options.stderr, true, interactive_sink, *console)
    });

    let status = child
        .wait()
        .map_err(|err| build_run_io_error(&options.cmd, &resolved_cwd, err))?;

    if let Some(handle) = stdin_handle {
        finish_io_thread(handle, "stdin", &options.cmd, &resolved_cwd)?;
    }

    let stdout_result = collect_process_output(
        stdout_handle,
        &options.stdout,
        "stdout",
        &options.cmd,
        &resolved_cwd,
    );
    let stderr_result = collect_process_output(
        stderr_handle,
        &options.stderr,
        "stderr",
        &options.cmd,
        &resolved_cwd,
    );

    let interactive_result = if let Some(viewport) = interactive_output {
        finish_interactive_output(viewport, &options.cmd, &resolved_cwd)
    } else {
        Ok(())
    };
    let stdout = stdout_result?;
    let stderr = stderr_result?;
    interactive_result?;

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

pub fn run_pipeline(
    options: &PipeOptions,
    current_dir: &Path,
    console: &Console,
) -> Result<PipeResult> {
    validate_pipeline_options(options)?;

    let resolved_cwd = resolve_run_cwd(current_dir, options.cwd.as_deref());
    let pipeline_for_error = format_pipeline_for_display(&options.commands);
    let interactive_output = InteractiveOutputViewport::maybe_new(
        *console,
        matches!(options.stdout, RunStreamMode::Inherit),
        matches!(options.stderr, RunStreamMode::Inherit),
    );
    let interactive_sink = interactive_output
        .as_ref()
        .map(InteractiveOutputViewport::sink);

    let mut children = Vec::with_capacity(options.commands.len());
    let mut previous_stdout = None;
    let mut stdin_handle = None;
    let mut stdout_handle = None;
    let mut stderr_handles = Vec::with_capacity(options.commands.len());

    for (index, stage) in options.commands.iter().enumerate() {
        let is_first = index == 0;
        let is_last = index + 1 == options.commands.len();

        let mut command = ProcessCommand::new(&stage.cmd);
        configure_command(
            &mut command,
            &stage.cmd,
            &stage.args,
            &resolved_cwd,
            &options.env,
            &options.env_remove,
        );
        configure_pipeline_stdio(
            &mut command,
            PipelineStageStdio {
                stdin_mode: &options.stdin,
                stdout_mode: &options.stdout,
                stderr_mode: &options.stderr,
                cwd: &resolved_cwd,
                pipeline: &pipeline_for_error,
                is_first,
                is_last,
                previous_stdout: previous_stdout.take(),
            },
        )?;

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(err) => {
                cleanup_spawned_children(children);
                return Err(build_pipe_io_error(&pipeline_for_error, &resolved_cwd, err));
            }
        };

        if is_first {
            stdin_handle = match &options.stdin {
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
        }

        if is_last {
            stdout_handle = child.stdout.take().map(|reader| {
                spawn_output_thread(
                    reader,
                    &options.stdout,
                    false,
                    interactive_sink.clone(),
                    *console,
                )
            });
        } else {
            let next_stdin = child.stdout.take().ok_or_else(|| {
                build_pipe_io_error(
                    &pipeline_for_error,
                    &resolved_cwd,
                    io::Error::other(format!(
                        "failed to capture stdout for pipeline stage `{}`",
                        stage.cmd
                    )),
                )
            })?;
            previous_stdout = Some(next_stdin);
        }

        let stderr_mode = pipeline_stderr_mode(&options.stderr, index == 0);
        let stderr_handle = child.stderr.take().map(|reader| {
            spawn_output_thread(
                reader,
                &stderr_mode,
                true,
                interactive_sink.clone(),
                *console,
            )
        });
        stderr_handles.push(stderr_handle);
        children.push(child);
    }

    let mut ok = true;
    let mut codes = Vec::with_capacity(children.len());
    for mut child in children {
        let status = child
            .wait()
            .map_err(|err| build_pipe_io_error(&pipeline_for_error, &resolved_cwd, err))?;
        ok &= status.success();
        codes.push(status.code());
    }

    if let Some(handle) = stdin_handle {
        finish_io_thread(handle, "stdin", &pipeline_for_error, &resolved_cwd)?;
    }

    let stdout_result = collect_process_output(
        stdout_handle,
        &options.stdout,
        "stdout",
        &pipeline_for_error,
        &resolved_cwd,
    );
    let stderr_result = collect_pipeline_stderr(
        stderr_handles,
        &options.stderr,
        &pipeline_for_error,
        &resolved_cwd,
    );

    let interactive_result = if let Some(viewport) = interactive_output {
        finish_interactive_output(viewport, &pipeline_for_error, &resolved_cwd)
    } else {
        Ok(())
    };
    let stdout = stdout_result?;
    let stderr = stderr_result?;
    interactive_result?;

    Ok(PipeResult {
        ok,
        code: summarize_pipeline_code(ok, &codes),
        codes,
        stdout: bytes_to_captured_string(&stdout, &options.stdout, options.trim),
        stderr: bytes_to_captured_string(&stderr, &options.stderr, options.trim),
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

pub fn format_pipeline_for_display(commands: &[PipeCommand]) -> String {
    commands
        .iter()
        .map(|command| format_command_for_display(&command.cmd, &command.args))
        .collect::<Vec<_>>()
        .join(" | ")
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

fn configure_pipeline_stdio(
    command: &mut ProcessCommand,
    stage: PipelineStageStdio<'_>,
) -> Result<()> {
    if let Some(previous_stdout) = stage.previous_stdout {
        command.stdin(Stdio::from(previous_stdout));
    } else {
        command.stdin(match stage.stdin_mode {
            RunStdin::Inherit => Stdio::inherit(),
            RunStdin::Bytes(_) => Stdio::piped(),
            RunStdin::File { path } => {
                Stdio::from(open_stdin_redirect_file(path, stage.cwd, stage.pipeline)?)
            }
        });
    }

    command.stdout(if stage.is_last {
        configure_output_stdio(stage.stdout_mode, "stdout", stage.cwd, stage.pipeline)?
    } else {
        Stdio::piped()
    });

    let stderr_mode = pipeline_stderr_mode(stage.stderr_mode, stage.is_first);
    command.stderr(configure_output_stdio(
        &stderr_mode,
        "stderr",
        stage.cwd,
        stage.pipeline,
    )?);

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

fn pipeline_stderr_mode(mode: &RunStreamMode, is_first_stage: bool) -> RunStreamMode {
    match mode {
        RunStreamMode::File { path, append } => RunStreamMode::File {
            path: path.clone(),
            append: *append || !is_first_stage,
        },
        _ => mode.clone(),
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

fn spawn_output_thread<R>(
    reader: R,
    mode: &RunStreamMode,
    use_stderr: bool,
    interactive_sink: Option<InteractiveOutputSink>,
    console: Console,
) -> OutputHandle
where
    R: Read + Send + 'static,
{
    match mode {
        RunStreamMode::Capture => OutputHandle::Capture(spawn_reader_thread(reader)),
        RunStreamMode::Inherit => OutputHandle::Inherit(match interactive_sink {
            Some(sink) => spawn_interactive_output_thread(reader, sink),
            None => spawn_prefixed_output_thread(reader, use_stderr, console),
        }),
        RunStreamMode::Null | RunStreamMode::File { .. } => {
            unreachable!("non-piped stream modes must not spawn output threads")
        }
    }
}

fn spawn_prefixed_output_thread<R>(
    mut reader: R,
    use_stderr: bool,
    console: Console,
) -> thread::JoinHandle<io::Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        if use_stderr {
            let stderr = console.stderr();
            let mut writer = stderr.lock();
            forward_prefixed_stream(&mut reader, &mut writer)
        } else {
            let stdout = console.stdout();
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

fn finish_interactive_output(
    viewport: InteractiveOutputViewport,
    cmd: &str,
    cwd: &Path,
) -> Result<()> {
    viewport.finish().map_err(|err| {
        build_run_io_error(
            cmd,
            cwd,
            io::Error::new(
                err.kind(),
                format!("failed to render interactive command output: {err}"),
            ),
        )
    })
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

fn collect_pipeline_stderr(
    handles: Vec<Option<OutputHandle>>,
    mode: &RunStreamMode,
    cmd: &str,
    cwd: &Path,
) -> Result<Vec<u8>> {
    match mode {
        RunStreamMode::Capture => {
            let mut stderr = Vec::new();
            for handle in handles {
                let bytes = collect_process_output(handle, mode, "stderr", cmd, cwd)?;
                stderr.extend(bytes);
            }
            Ok(stderr)
        }
        RunStreamMode::Inherit => {
            for handle in handles {
                collect_process_output(handle, mode, "stderr", cmd, cwd)?;
            }
            Ok(Vec::new())
        }
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

fn cleanup_spawned_children(children: Vec<Child>) {
    for mut child in children {
        let _ = child.kill();
        let _ = child.wait();
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

fn validate_pipeline_options(options: &PipeOptions) -> Result<()> {
    if options.commands.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.pipe requires at least two pipeline stages",
        )
        .with_op("ptool.pipe"));
    }

    for command in &options.commands {
        if command.cmd.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.pipe pipeline stage command must not be empty",
            )
            .with_op("ptool.pipe"));
        }
    }

    Ok(())
}

fn summarize_pipeline_code(ok: bool, codes: &[Option<i32>]) -> Option<i32> {
    if ok {
        return codes.last().copied().flatten();
    }

    if codes
        .iter()
        .any(|code| code.is_none() || code.is_some_and(|value| value != 0))
    {
        for code in codes.iter().rev() {
            match code {
                Some(0) => continue,
                Some(code) => return Some(*code),
                None => return None,
            }
        }
    }

    None
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

fn build_pipe_io_error(cmd: &str, cwd: &Path, err: std::io::Error) -> Error {
    Error::new(
        ErrorKind::Io,
        format!(
            "failed to run pipeline `{cmd}` in `{}`: {err}",
            cwd.display()
        ),
    )
    .with_op("ptool.pipe")
    .with_cmd(cmd)
    .with_detail(format!("cwd: {}", cwd.display()))
}
