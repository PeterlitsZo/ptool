use crate::lua_world::RunConfig;
use inquire::{Confirm, InquireError};
use jiff::Zoned;
use mlua::{Lua, Table, Value, Variadic};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::process::{ExitStatus, Stdio};

#[derive(Clone, Copy)]
enum StreamMode {
    Inherit,
    Capture,
    Null,
}

struct RunOptions {
    cmd: String,
    args: Vec<String>,
    cwd: Option<String>,
    env: Option<Table>,
    echo: bool,
    stdout: StreamMode,
    stderr: StreamMode,
    check: bool,
    confirm: bool,
}

struct RunCallOverrides {
    cwd: Option<String>,
    env: Option<Table>,
    echo: Option<bool>,
    stdout: Option<StreamMode>,
    stderr: Option<StreamMode>,
    check: Option<bool>,
    confirm: Option<bool>,
}

pub(crate) fn run_command(
    lua: &Lua,
    args: Variadic<Value>,
    current_dir: &Path,
    defaults: RunConfig,
) -> mlua::Result<Value> {
    let options = parse_run_options(args, defaults)?;
    let cmd_for_error = options.cmd.clone();
    let resolved_cwd = resolve_run_cwd(current_dir, options.cwd.as_deref());

    let display = if options.echo || options.confirm {
        let command = format_command_for_display(&options.cmd, &options.args);
        Some((resolved_cwd.clone(), command))
    } else {
        None
    };

    if options.echo {
        let (display_cwd, display_command) = display.as_ref().ok_or_else(|| {
            mlua::Error::runtime("ptool.run internal error: missing display info")
        })?;
        print_command_echo(display_cwd, display_command);
    }

    if options.confirm {
        let (display_cwd, display_command) = display.as_ref().ok_or_else(|| {
            mlua::Error::runtime("ptool.run internal error: missing display info")
        })?;
        confirm_before_run(display_cwd, display_command, &cmd_for_error)?;
    }

    let mut command = ProcessCommand::new(&options.cmd);
    command.args(options.args);
    command.current_dir(&resolved_cwd);

    if let Some(vars) = options.env {
        for pair in vars.pairs::<String, String>() {
            let (key, value) = pair?;
            command.env(key, value);
        }
    }

    apply_stream_mode_for_stdout(&mut command, options.stdout);
    apply_stream_mode_for_stderr(&mut command, options.stderr);

    let output = command.output().map_err(mlua::Error::external)?;
    let stdout = bytes_to_captured_string(&output.stdout, options.stdout);
    let stderr = bytes_to_captured_string(&output.stderr, options.stderr);
    if options.check && !output.status.success() {
        return Err(build_run_failed_error(
            &cmd_for_error,
            output.status.code(),
            stderr.as_deref(),
        ));
    }

    build_run_result(lua, output.status, stdout, stderr, cmd_for_error)
}

fn parse_run_options(args: Variadic<Value>, defaults: RunConfig) -> mlua::Result<RunOptions> {
    match args.len() {
        0 => Err(mlua::Error::runtime("ptool.run requires arguments")),
        1 => match args.first() {
            Some(Value::String(cmdline)) => {
                let (cmd, args) = parse_cmdline_to_cmd_and_args(&cmdline.to_str()?)?;
                Ok(RunOptions {
                    cmd,
                    args,
                    cwd: None,
                    env: None,
                    echo: defaults.echo,
                    stdout: StreamMode::Inherit,
                    stderr: StreamMode::Inherit,
                    check: defaults.check,
                    confirm: defaults.confirm,
                })
            }
            Some(Value::Table(options)) => parse_full_options_table(options.clone(), defaults),
            _ => Err(mlua::Error::runtime(
                "ptool.run expects a command string or an options table",
            )),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::String(cmd)), Some(Value::String(argsline))) => Ok(RunOptions {
                cmd: cmd.to_str()?.to_owned(),
                args: parse_argsline(&argsline.to_str()?)?,
                cwd: None,
                env: None,
                echo: defaults.echo,
                stdout: StreamMode::Inherit,
                stderr: StreamMode::Inherit,
                check: defaults.check,
                confirm: defaults.confirm,
            }),
            (Some(Value::String(cmd_or_cmdline)), Some(Value::Table(second_table))) => {
                if looks_like_options_table(second_table)? {
                    let (cmd, args) = parse_cmdline_to_cmd_and_args(&cmd_or_cmdline.to_str()?)?;
                    let overrides =
                        parse_overrides_table(second_table.clone(), "ptool.run(cmdline, options)")?;
                    Ok(apply_overrides(cmd, args, overrides, defaults))
                } else {
                    Ok(RunOptions {
                        cmd: cmd_or_cmdline.to_str()?.to_owned(),
                        args: parse_string_list(second_table)?,
                        cwd: None,
                        env: None,
                        echo: defaults.echo,
                        stdout: StreamMode::Inherit,
                        stderr: StreamMode::Inherit,
                        check: defaults.check,
                        confirm: defaults.confirm,
                    })
                }
            }
            _ => Err(mlua::Error::runtime(
                "ptool.run(cmd, args) expects (string, table|string)",
            )),
        },
        3 => match (args.first(), args.get(1), args.get(2)) {
            (
                Some(Value::String(cmd)),
                Some(Value::String(argsline)),
                Some(Value::Table(options)),
            ) => {
                let overrides =
                    parse_overrides_table(options.clone(), "ptool.run(cmd, argsline, options)")?;
                Ok(apply_overrides(
                    cmd.to_str()?.to_owned(),
                    parse_argsline(&argsline.to_str()?)?,
                    overrides,
                    defaults,
                ))
            }
            (
                Some(Value::String(cmd)),
                Some(Value::Table(args_table)),
                Some(Value::Table(options)),
            ) => {
                let overrides =
                    parse_overrides_table(options.clone(), "ptool.run(cmd, args, options)")?;
                Ok(apply_overrides(
                    cmd.to_str()?.to_owned(),
                    parse_string_list(args_table)?,
                    overrides,
                    defaults,
                ))
            }
            _ => Err(mlua::Error::runtime(
                "ptool.run(cmd, args, options) expects (string, table|string, table)",
            )),
        },
        _ => Err(mlua::Error::runtime(
            "ptool.run accepts at most 3 arguments",
        )),
    }
}

fn parse_full_options_table(options: Table, defaults: RunConfig) -> mlua::Result<RunOptions> {
    let Some(cmd) = options.get::<Option<String>>("cmd")? else {
        return Err(mlua::Error::runtime(
            "ptool.run options mode requires `cmd`",
        ));
    };

    let args = parse_named_args(&options)?;
    let cwd: Option<String> = options.get("cwd")?;
    let env: Option<Table> = options.get("env")?;
    let echo = options
        .get::<Option<bool>>("echo")?
        .unwrap_or(defaults.echo);
    let stdout = parse_stream_mode(
        options.get::<Option<String>>("stdout")?,
        "stdout",
        "ptool.run(options)",
    )?
    .unwrap_or(StreamMode::Inherit);
    let stderr = parse_stream_mode(
        options.get::<Option<String>>("stderr")?,
        "stderr",
        "ptool.run(options)",
    )?
    .unwrap_or(StreamMode::Inherit);
    let check = options
        .get::<Option<bool>>("check")?
        .unwrap_or(defaults.check);
    let confirm = options
        .get::<Option<bool>>("confirm")?
        .unwrap_or(defaults.confirm);

    Ok(RunOptions {
        cmd,
        args,
        cwd,
        env,
        echo,
        stdout,
        stderr,
        check,
        confirm,
    })
}

fn parse_named_args(options: &Table) -> mlua::Result<Vec<String>> {
    let Some(args_table) = options.get::<Option<Table>>("args")? else {
        return Ok(Vec::new());
    };

    parse_string_list(&args_table)
}

fn parse_string_list(args_table: &Table) -> mlua::Result<Vec<String>> {
    let mut args = Vec::new();
    for value in args_table.sequence_values::<String>() {
        args.push(value?);
    }
    Ok(args)
}

fn looks_like_options_table(options: &Table) -> mlua::Result<bool> {
    Ok(!is_array_table(options)?)
}

fn is_array_table(table: &Table) -> mlua::Result<bool> {
    let mut entry_count = 0usize;
    let mut max_index = 0usize;

    for pair in table.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let Value::Integer(index) = key else {
            return Ok(false);
        };
        if index <= 0 {
            return Ok(false);
        }
        let Ok(index) = usize::try_from(index) else {
            return Ok(false);
        };
        entry_count += 1;
        max_index = max_index.max(index);
    }

    Ok(entry_count == max_index)
}

fn has_key(options: &Table, key: &str) -> mlua::Result<bool> {
    Ok(options.get::<Option<Value>>(key)?.is_some())
}

fn parse_overrides_table(options: Table, context: &str) -> mlua::Result<RunCallOverrides> {
    if has_key(&options, "cmd")? || has_key(&options, "args")? {
        return Err(mlua::Error::runtime(format!(
            "{context} options table does not allow `cmd` or `args`"
        )));
    }

    Ok(RunCallOverrides {
        cwd: options.get("cwd")?,
        env: options.get("env")?,
        echo: options.get::<Option<bool>>("echo")?,
        stdout: parse_stream_mode(options.get::<Option<String>>("stdout")?, "stdout", context)?,
        stderr: parse_stream_mode(options.get::<Option<String>>("stderr")?, "stderr", context)?,
        check: options.get::<Option<bool>>("check")?,
        confirm: options.get::<Option<bool>>("confirm")?,
    })
}

fn apply_overrides(
    cmd: String,
    args: Vec<String>,
    overrides: RunCallOverrides,
    defaults: RunConfig,
) -> RunOptions {
    RunOptions {
        cmd,
        args,
        cwd: overrides.cwd,
        env: overrides.env,
        echo: overrides.echo.unwrap_or(defaults.echo),
        stdout: overrides.stdout.unwrap_or(StreamMode::Inherit),
        stderr: overrides.stderr.unwrap_or(StreamMode::Inherit),
        check: overrides.check.unwrap_or(defaults.check),
        confirm: overrides.confirm.unwrap_or(defaults.confirm),
    }
}

fn build_run_result(
    lua: &Lua,
    status: ExitStatus,
    stdout: Option<String>,
    stderr: Option<String>,
    cmd_for_error: String,
) -> mlua::Result<Value> {
    let result = lua.create_table()?;
    result.set("ok", status.success())?;
    result.set("code", status.code().map(i64::from))?;
    if let Some(stdout) = stdout {
        result.set("stdout", stdout)?;
    }
    if let Some(stderr) = stderr {
        result.set("stderr", stderr)?;
    }

    let assert_cmd_for_error = cmd_for_error.clone();
    let assert_ok_fn = lua.create_function(move |_, this: Table| {
        let ok = this.get::<bool>("ok")?;
        if ok {
            return Ok(());
        }
        let code = this
            .get::<Option<i64>>("code")?
            .and_then(|value| i32::try_from(value).ok());
        let stderr = this.get::<Option<String>>("stderr")?;
        Err(build_run_failed_error(
            &assert_cmd_for_error,
            code,
            stderr.as_deref(),
        ))
    })?;
    result.set("assert_ok", assert_ok_fn)?;

    Ok(Value::Table(result))
}

fn parse_stream_mode(
    mode: Option<String>,
    field_name: &str,
    context: &str,
) -> mlua::Result<Option<StreamMode>> {
    let Some(mode) = mode else {
        return Ok(None);
    };

    let mode = match mode.as_str() {
        "inherit" => StreamMode::Inherit,
        "capture" => StreamMode::Capture,
        "null" => StreamMode::Null,
        _ => {
            return Err(mlua::Error::runtime(format!(
                "{context} `{field_name}` must be one of `inherit`, `capture`, `null`"
            )));
        }
    };
    Ok(Some(mode))
}

fn apply_stream_mode_for_stdout(command: &mut ProcessCommand, mode: StreamMode) {
    match mode {
        StreamMode::Inherit => {
            command.stdout(Stdio::inherit());
        }
        StreamMode::Capture => {
            command.stdout(Stdio::piped());
        }
        StreamMode::Null => {
            command.stdout(Stdio::null());
        }
    }
}

fn apply_stream_mode_for_stderr(command: &mut ProcessCommand, mode: StreamMode) {
    match mode {
        StreamMode::Inherit => {
            command.stderr(Stdio::inherit());
        }
        StreamMode::Capture => {
            command.stderr(Stdio::piped());
        }
        StreamMode::Null => {
            command.stderr(Stdio::null());
        }
    }
}

fn bytes_to_captured_string(bytes: &[u8], mode: StreamMode) -> Option<String> {
    match mode {
        StreamMode::Capture => Some(String::from_utf8_lossy(bytes).to_string()),
        StreamMode::Inherit | StreamMode::Null => None,
    }
}

fn build_run_failed_error(
    cmd_for_error: &str,
    code: Option<i32>,
    stderr: Option<&str>,
) -> mlua::Error {
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
    mlua::Error::runtime(message)
}

fn confirm_before_run(cwd: &Path, command: &str, cmd_for_error: &str) -> mlua::Result<()> {
    let prompt = format!("Run command -- {}?", command);
    let help_msg = format!("The cwd is {}", cwd.display());
    match Confirm::new(&prompt)
        .with_default(true)
        .with_help_message(&help_msg)
        .prompt()
    {
        Ok(true) => Ok(()),
        Ok(false) => Err(mlua::Error::runtime(format!(
            "ptool.run command `{cmd_for_error}` cancelled by user"
        ))),
        Err(InquireError::NotTTY | InquireError::IO(_)) => Err(mlua::Error::runtime(format!(
            "ptool.run command `{cmd_for_error}` requires confirmation, but no interactive TTY is available"
        ))),
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
            Err(mlua::Error::runtime(format!(
                "ptool.run command `{cmd_for_error}` cancelled by user"
            )))
        }
        Err(err) => Err(mlua::Error::runtime(format!(
            "ptool.run command `{cmd_for_error}` confirmation failed: {err}"
        ))),
    }
}

fn parse_cmdline_to_cmd_and_args(input: &str) -> mlua::Result<(String, Vec<String>)> {
    let parts = parse_shell_words(input, "ptool.run command string")?;
    let mut iter = parts.into_iter();
    let Some(cmd) = iter.next() else {
        return Err(mlua::Error::runtime(
            "ptool.run command string must not be empty",
        ));
    };
    Ok((cmd, iter.collect()))
}

fn parse_argsline(input: &str) -> mlua::Result<Vec<String>> {
    parse_shell_words(input, "ptool.run args string")
}

fn parse_shell_words(input: &str, context: &str) -> mlua::Result<Vec<String>> {
    shlex::split(input)
        .ok_or_else(|| mlua::Error::runtime(format!("{context} failed to parse as shell words")))
}

fn resolve_run_cwd(current_dir: &Path, cwd: Option<&str>) -> PathBuf {
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

fn format_command_for_display(cmd: &str, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(shell_quote(cmd));
    for arg in args {
        parts.push(shell_quote(arg));
    }
    parts.join(" ")
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

fn print_command_echo(cwd: &Path, command: &str) {
    let time = Zoned::now().strftime("%Y-%m-%d %H:%M:%S").to_string();
    let time_segment = format!("[{time}]");
    let cwd_segment = cwd.display().to_string();

    print!(
        "{} {} {}",
        time_segment.bright_black().bold(),
        cwd_segment.cyan().bold(),
        "$".green().bold(),
    );
    print_command_self(command);
}

fn print_command_self(command: &str) {
    let mut lines = command.split('\n');
    let Some(first) = lines.next() else {
        println!();
        return;
    };

    print!(" {}", first);
    for line in lines {
        print!("\n{} {}", "...".dimmed(), line.to_string().bold());
    }
    println!();
}
