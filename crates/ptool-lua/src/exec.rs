use crate::command_echo::print_local_command_echo;
use crate::lua_world::RunConfig;
use inquire::{Confirm, InquireError};
use mlua::{Lua, Table, Value, Variadic};
use ptool_engine::{
    PtoolEngine, RunResult, RunStreamMode, format_command_for_display, format_run_failed_message,
    resolve_run_cwd,
};
use std::path::Path;

type StreamMode = RunStreamMode;

#[derive(Clone, Copy)]
struct StreamDefaults {
    stdout: StreamMode,
    stderr: StreamMode,
}

const RUN_STREAM_DEFAULTS: StreamDefaults = StreamDefaults {
    stdout: StreamMode::Inherit,
    stderr: StreamMode::Inherit,
};

const RUN_CAPTURE_STREAM_DEFAULTS: StreamDefaults = StreamDefaults {
    stdout: StreamMode::Capture,
    stderr: StreamMode::Capture,
};

struct RunOptions {
    inner: ptool_engine::RunOptions,
    echo: bool,
    check: bool,
    confirm: bool,
    retry: bool,
}

struct RunCallOverrides {
    cwd: Option<String>,
    env: Option<Vec<(String, String)>>,
    echo: Option<bool>,
    stdout: Option<StreamMode>,
    stderr: Option<StreamMode>,
    check: Option<bool>,
    confirm: Option<bool>,
    retry: Option<bool>,
}

pub(crate) fn run_command(
    lua: &Lua,
    args: Variadic<Value>,
    current_dir: &Path,
    engine: &PtoolEngine,
    defaults: RunConfig,
) -> mlua::Result<Value> {
    run_command_with_stream_defaults(
        lua,
        args,
        current_dir,
        engine,
        defaults,
        RUN_STREAM_DEFAULTS,
    )
}

pub(crate) fn run_capture_command(
    lua: &Lua,
    args: Variadic<Value>,
    current_dir: &Path,
    engine: &PtoolEngine,
    defaults: RunConfig,
) -> mlua::Result<Value> {
    run_command_with_stream_defaults(
        lua,
        args,
        current_dir,
        engine,
        defaults,
        RUN_CAPTURE_STREAM_DEFAULTS,
    )
}

fn run_command_with_stream_defaults(
    lua: &Lua,
    args: Variadic<Value>,
    current_dir: &Path,
    engine: &PtoolEngine,
    defaults: RunConfig,
    stream_defaults: StreamDefaults,
) -> mlua::Result<Value> {
    let options = parse_run_options(args, defaults, stream_defaults)?;
    let cmd_for_error = options.inner.cmd.clone();
    let resolved_cwd = resolve_run_cwd(current_dir, options.inner.cwd.as_deref());
    let local_user_host = options.echo.then(|| engine.current_user_host());

    let display = if options.echo || options.confirm || (options.check && options.retry) {
        let command = format_command_for_display(&options.inner.cmd, &options.inner.args);
        Some((resolved_cwd.clone(), command))
    } else {
        None
    };

    if options.echo {
        let (display_cwd, display_command) = display.as_ref().ok_or_else(|| {
            mlua::Error::runtime("ptool.run internal error: missing display info")
        })?;
        let local_user_host = local_user_host.as_ref().ok_or_else(|| {
            mlua::Error::runtime("ptool.run internal error: missing local user/host info")
        })?;
        print_local_command_echo(
            &local_user_host.user,
            &local_user_host.host,
            display_cwd,
            display_command,
        );
    }

    if options.confirm {
        let (display_cwd, display_command) = display.as_ref().ok_or_else(|| {
            mlua::Error::runtime("ptool.run internal error: missing display info")
        })?;
        confirm_before_run(display_cwd, display_command, &cmd_for_error)?;
    }

    let mut is_retry = false;
    loop {
        if is_retry && options.echo {
            let (display_cwd, display_command) = display.as_ref().ok_or_else(|| {
                mlua::Error::runtime("ptool.run internal error: missing display info")
            })?;
            let local_user_host = local_user_host.as_ref().ok_or_else(|| {
                mlua::Error::runtime("ptool.run internal error: missing local user/host info")
            })?;
            print_local_command_echo(
                &local_user_host.user,
                &local_user_host.host,
                display_cwd,
                display_command,
            );
        }

        let result = engine
            .run_command(&options.inner, current_dir)
            .map_err(engine_error)?;
        if options.check && !result.ok {
            if options.retry {
                let (display_cwd, display_command) = display.as_ref().ok_or_else(|| {
                    mlua::Error::runtime("ptool.run internal error: missing display info")
                })?;
                if prompt_retry_after_failure(
                    display_cwd,
                    display_command,
                    result.code,
                    result.stderr.as_deref(),
                    &cmd_for_error,
                )? {
                    is_retry = true;
                    continue;
                }
            }

            return Err(build_run_failed_error(
                &cmd_for_error,
                result.code,
                result.stderr.as_deref(),
            ));
        }

        return build_run_result(lua, result, cmd_for_error);
    }
}

fn parse_run_options(
    args: Variadic<Value>,
    defaults: RunConfig,
    stream_defaults: StreamDefaults,
) -> mlua::Result<RunOptions> {
    match args.len() {
        0 => Err(mlua::Error::runtime("ptool.run requires arguments")),
        1 => match args.first() {
            Some(Value::String(cmdline)) => {
                let (cmd, args) = parse_cmdline_to_cmd_and_args(&cmdline.to_str()?)?;
                Ok(RunOptions {
                    inner: ptool_engine::RunOptions {
                        cmd,
                        args,
                        cwd: None,
                        env: Vec::new(),
                        stdout: stream_defaults.stdout,
                        stderr: stream_defaults.stderr,
                    },
                    echo: defaults.echo,
                    check: defaults.check,
                    confirm: defaults.confirm,
                    retry: defaults.retry,
                })
            }
            Some(Value::Table(options)) => {
                parse_full_options_table(options.clone(), defaults, stream_defaults)
            }
            _ => Err(mlua::Error::runtime(
                "ptool.run expects a command string or an options table",
            )),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::String(cmd)), Some(Value::String(argsline))) => Ok(RunOptions {
                inner: ptool_engine::RunOptions {
                    cmd: cmd.to_str()?.to_owned(),
                    args: parse_argsline(&argsline.to_str()?)?,
                    cwd: None,
                    env: Vec::new(),
                    stdout: stream_defaults.stdout,
                    stderr: stream_defaults.stderr,
                },
                echo: defaults.echo,
                check: defaults.check,
                confirm: defaults.confirm,
                retry: defaults.retry,
            }),
            (Some(Value::String(cmd_or_cmdline)), Some(Value::Table(second_table))) => {
                if looks_like_options_table(second_table)? {
                    let (cmd, args) = parse_cmdline_to_cmd_and_args(&cmd_or_cmdline.to_str()?)?;
                    let overrides =
                        parse_overrides_table(second_table.clone(), "ptool.run(cmdline, options)")?;
                    Ok(apply_overrides(
                        cmd,
                        args,
                        overrides,
                        defaults,
                        stream_defaults,
                    ))
                } else {
                    Ok(RunOptions {
                        inner: ptool_engine::RunOptions {
                            cmd: cmd_or_cmdline.to_str()?.to_owned(),
                            args: parse_string_list(second_table)?,
                            cwd: None,
                            env: Vec::new(),
                            stdout: stream_defaults.stdout,
                            stderr: stream_defaults.stderr,
                        },
                        echo: defaults.echo,
                        check: defaults.check,
                        confirm: defaults.confirm,
                        retry: defaults.retry,
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
                    stream_defaults,
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
                    stream_defaults,
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

fn parse_full_options_table(
    options: Table,
    defaults: RunConfig,
    stream_defaults: StreamDefaults,
) -> mlua::Result<RunOptions> {
    let Some(cmd) = options.get::<Option<String>>("cmd")? else {
        return Err(mlua::Error::runtime(
            "ptool.run options mode requires `cmd`",
        ));
    };

    let args = parse_named_args(&options)?;
    let cwd: Option<String> = options.get("cwd")?;
    let env = parse_env_table(options.get::<Option<Table>>("env")?)?;
    let echo = options
        .get::<Option<bool>>("echo")?
        .unwrap_or(defaults.echo);
    let stdout = parse_stream_mode(
        options.get::<Option<String>>("stdout")?,
        "stdout",
        "ptool.run(options)",
    )?
    .unwrap_or(stream_defaults.stdout);
    let stderr = parse_stream_mode(
        options.get::<Option<String>>("stderr")?,
        "stderr",
        "ptool.run(options)",
    )?
    .unwrap_or(stream_defaults.stderr);
    let check = options
        .get::<Option<bool>>("check")?
        .unwrap_or(defaults.check);
    let confirm = options
        .get::<Option<bool>>("confirm")?
        .unwrap_or(defaults.confirm);
    let retry = options
        .get::<Option<bool>>("retry")?
        .unwrap_or(defaults.retry);

    Ok(RunOptions {
        inner: ptool_engine::RunOptions {
            cmd,
            args,
            cwd,
            env,
            stdout,
            stderr,
        },
        echo,
        check,
        confirm,
        retry,
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
        env: parse_optional_env_table(options.get::<Option<Table>>("env")?)?,
        echo: options.get::<Option<bool>>("echo")?,
        stdout: parse_stream_mode(options.get::<Option<String>>("stdout")?, "stdout", context)?,
        stderr: parse_stream_mode(options.get::<Option<String>>("stderr")?, "stderr", context)?,
        check: options.get::<Option<bool>>("check")?,
        confirm: options.get::<Option<bool>>("confirm")?,
        retry: options.get::<Option<bool>>("retry")?,
    })
}

fn apply_overrides(
    cmd: String,
    args: Vec<String>,
    overrides: RunCallOverrides,
    defaults: RunConfig,
    stream_defaults: StreamDefaults,
) -> RunOptions {
    RunOptions {
        inner: ptool_engine::RunOptions {
            cmd,
            args,
            cwd: overrides.cwd,
            env: overrides.env.unwrap_or_default(),
            stdout: overrides.stdout.unwrap_or(stream_defaults.stdout),
            stderr: overrides.stderr.unwrap_or(stream_defaults.stderr),
        },
        echo: overrides.echo.unwrap_or(defaults.echo),
        check: overrides.check.unwrap_or(defaults.check),
        confirm: overrides.confirm.unwrap_or(defaults.confirm),
        retry: overrides.retry.unwrap_or(defaults.retry),
    }
}

fn build_run_result(
    lua: &Lua,
    run_result: RunResult,
    cmd_for_error: String,
) -> mlua::Result<Value> {
    let result = lua.create_table()?;
    result.set("ok", run_result.ok)?;
    result.set("code", run_result.code.map(i64::from))?;
    if let Some(stdout) = run_result.stdout {
        result.set("stdout", stdout)?;
    }
    if let Some(stderr) = run_result.stderr {
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

fn build_run_failed_error(
    cmd_for_error: &str,
    code: Option<i32>,
    stderr: Option<&str>,
) -> mlua::Error {
    mlua::Error::runtime(format_run_failed_message(cmd_for_error, code, stderr))
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

fn prompt_retry_after_failure(
    cwd: &Path,
    command: &str,
    code: Option<i32>,
    stderr: Option<&str>,
    cmd_for_error: &str,
) -> mlua::Result<bool> {
    let code = code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "terminated by signal".to_string());
    let prompt = format!("Command failed with status {code}. Retry -- {command}?");
    let help_msg = build_retry_help_message(cwd, stderr);
    match Confirm::new(&prompt)
        .with_default(true)
        .with_help_message(&help_msg)
        .prompt()
    {
        Ok(answer) => Ok(answer),
        Err(InquireError::NotTTY | InquireError::IO(_)) => Err(mlua::Error::runtime(format!(
            "ptool.run command `{cmd_for_error}` failed and retry requires an interactive TTY"
        ))),
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
            Err(mlua::Error::runtime(format!(
                "ptool.run retry for command `{cmd_for_error}` cancelled by user"
            )))
        }
        Err(err) => Err(mlua::Error::runtime(format!(
            "ptool.run retry prompt for command `{cmd_for_error}` failed: {err}"
        ))),
    }
}

fn build_retry_help_message(cwd: &Path, stderr: Option<&str>) -> String {
    let mut help_msg = format!("The cwd is {}", cwd.display());
    if let Some(stderr_summary) = summarize_stderr_for_prompt(stderr) {
        help_msg.push_str("\nStderr: ");
        help_msg.push_str(&stderr_summary);
    }
    help_msg
}

fn summarize_stderr_for_prompt(stderr: Option<&str>) -> Option<String> {
    let stderr = stderr?.trim();
    if stderr.is_empty() {
        return None;
    }

    let summary = stderr.replace('\n', " | ");
    let mut truncated = String::new();
    for (index, ch) in summary.chars().enumerate() {
        if index >= 160 {
            truncated.push('…');
            break;
        }
        truncated.push(ch);
    }
    Some(truncated)
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

fn parse_env_table(env: Option<Table>) -> mlua::Result<Vec<(String, String)>> {
    let Some(env) = env else {
        return Ok(Vec::new());
    };

    let mut vars = Vec::new();
    for pair in env.pairs::<String, String>() {
        vars.push(pair?);
    }
    Ok(vars)
}

fn parse_optional_env_table(env: Option<Table>) -> mlua::Result<Option<Vec<(String, String)>>> {
    match env {
        Some(env) => Ok(Some(parse_env_table(Some(env))?)),
        None => Ok(None),
    }
}

fn engine_error(err: ptool_engine::Error) -> mlua::Error {
    mlua::Error::runtime(err.to_string())
}
