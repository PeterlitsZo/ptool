use mlua::{Lua, Table, Value};
use ptool_engine::{
    ProcInfo, ProcKillOptions, ProcKillResult, ProcQuery, ProcSignal, ProcSortBy, ProcTarget,
    ProcWaitGoneOptions, ProcWaitGoneResult, PtoolEngine,
};

const SELF_SIGNATURE: &str = "ptool.proc.self()";
const GET_SIGNATURE: &str = "ptool.proc.get(pid)";
const EXISTS_SIGNATURE: &str = "ptool.proc.exists(pid)";
const FIND_SIGNATURE: &str = "ptool.proc.find(options?)";
const KILL_SIGNATURE: &str = "ptool.proc.kill(targets[, options])";
const WAIT_GONE_SIGNATURE: &str = "ptool.proc.wait_gone(targets[, options])";

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ProcLuaRunOptions {
    pub check: bool,
    pub confirm: bool,
}

pub(crate) fn self_process(lua: &Lua, engine: &PtoolEngine) -> mlua::Result<Table> {
    let info = engine
        .proc_self()
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, SELF_SIGNATURE))?;
    proc_info_to_lua(lua, info)
}

pub(crate) fn get_process(
    lua: &Lua,
    engine: &PtoolEngine,
    pid_value: Value,
) -> mlua::Result<Value> {
    let pid = parse_pid_value(pid_value, GET_SIGNATURE, "pid")?;
    let info = engine
        .proc_get(pid)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, GET_SIGNATURE))?;
    match info {
        Some(info) => Ok(Value::Table(proc_info_to_lua(lua, info)?)),
        None => Ok(Value::Nil),
    }
}

pub(crate) fn exists(engine: &PtoolEngine, pid_value: Value) -> mlua::Result<bool> {
    let pid = parse_pid_value(pid_value, EXISTS_SIGNATURE, "pid")?;
    engine
        .proc_exists(pid)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, EXISTS_SIGNATURE))
}

pub(crate) fn find_processes(
    lua: &Lua,
    engine: &PtoolEngine,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let query = parse_find_options(options)?;
    let results = engine
        .proc_find(&query)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, FIND_SIGNATURE))?;
    let list = lua.create_table()?;
    for (index, info) in results.into_iter().enumerate() {
        list.raw_set(index + 1, proc_info_to_lua(lua, info)?)?;
    }
    Ok(list)
}

pub(crate) fn kill_processes(
    lua: &Lua,
    engine: &PtoolEngine,
    targets_value: Value,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let targets = parse_targets_value(targets_value, KILL_SIGNATURE)?;
    let (kill_options, run_options) = parse_kill_options(options)?;

    if run_options.confirm {
        confirm_kill(engine, &targets, kill_options.signal)?;
    }

    let result = engine
        .proc_kill(&targets, &kill_options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, KILL_SIGNATURE))?;
    let table = build_kill_result(lua, result)?;
    if run_options.check {
        assert_kill_result(&table)?;
    }
    Ok(table)
}

pub(crate) fn wait_gone(
    lua: &Lua,
    engine: &PtoolEngine,
    targets_value: Value,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let targets = parse_targets_value(targets_value, WAIT_GONE_SIGNATURE)?;
    let (wait_options, run_options) = parse_wait_gone_options(options)?;
    let result = engine
        .proc_wait_gone(&targets, &wait_options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, WAIT_GONE_SIGNATURE))?;
    let table = build_wait_gone_result(lua, result)?;
    if run_options.check {
        assert_wait_gone_result(&table)?;
    }
    Ok(table)
}

fn parse_find_options(options: Option<Table>) -> mlua::Result<ProcQuery> {
    let mut query = ProcQuery::default();
    let Some(options) = options else {
        return Ok(query);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, FIND_SIGNATURE)?;
        match key.as_str() {
            "pid" => query.pid = Some(parse_pid_value(value, FIND_SIGNATURE, "pid")?),
            "pids" => query.pids = parse_pid_list_value(value, FIND_SIGNATURE, "pids")?,
            "ppid" => query.ppid = Some(parse_pid_value(value, FIND_SIGNATURE, "ppid")?),
            "name" => {
                query.name = Some(parse_non_empty_string_value(value, FIND_SIGNATURE, "name")?)
            }
            "name_contains" => {
                query.name_contains = Some(parse_non_empty_string_value(
                    value,
                    FIND_SIGNATURE,
                    "name_contains",
                )?)
            }
            "exe" => query.exe = Some(parse_non_empty_string_value(value, FIND_SIGNATURE, "exe")?),
            "exe_contains" => {
                query.exe_contains = Some(parse_non_empty_string_value(
                    value,
                    FIND_SIGNATURE,
                    "exe_contains",
                )?)
            }
            "cmdline_contains" => {
                query.cmdline_contains = Some(parse_non_empty_string_value(
                    value,
                    FIND_SIGNATURE,
                    "cmdline_contains",
                )?)
            }
            "user" => {
                query.user = Some(parse_non_empty_string_value(value, FIND_SIGNATURE, "user")?)
            }
            "cwd" => query.cwd = Some(parse_non_empty_string_value(value, FIND_SIGNATURE, "cwd")?),
            "include_self" => {
                query.include_self = parse_bool_value(value, FIND_SIGNATURE, "include_self")?
            }
            "limit" => query.limit = Some(parse_usize_value(value, FIND_SIGNATURE, "limit")?),
            "sort_by" => {
                let value = parse_non_empty_string_value(value, FIND_SIGNATURE, "sort_by")?;
                query.sort_by = match value.as_str() {
                    "pid" => ProcSortBy::Pid,
                    "start_time" => ProcSortBy::StartTime,
                    _ => {
                        return Err(crate::lua_error::invalid_option(
                            FIND_SIGNATURE,
                            "`sort_by` must be one of `pid`, `start_time`",
                        ));
                    }
                };
            }
            "reverse" => query.reverse = parse_bool_value(value, FIND_SIGNATURE, "reverse")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    FIND_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(query)
}

fn parse_kill_options(
    options: Option<Table>,
) -> mlua::Result<(ProcKillOptions, ProcLuaRunOptions)> {
    let mut parsed = ProcKillOptions::default();
    let mut run_options = ProcLuaRunOptions::default();
    let Some(options) = options else {
        return Ok((parsed, run_options));
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, KILL_SIGNATURE)?;
        match key.as_str() {
            "signal" => {
                let value = parse_non_empty_string_value(value, KILL_SIGNATURE, "signal")?;
                parsed.signal = parse_signal(&value)?;
            }
            "missing_ok" => {
                parsed.missing_ok = parse_bool_value(value, KILL_SIGNATURE, "missing_ok")?
            }
            "allow_self" => {
                parsed.allow_self = parse_bool_value(value, KILL_SIGNATURE, "allow_self")?
            }
            "check" => run_options.check = parse_bool_value(value, KILL_SIGNATURE, "check")?,
            "confirm" => run_options.confirm = parse_bool_value(value, KILL_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    KILL_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok((parsed, run_options))
}

fn parse_wait_gone_options(
    options: Option<Table>,
) -> mlua::Result<(ProcWaitGoneOptions, ProcLuaRunOptions)> {
    let mut parsed = ProcWaitGoneOptions::default();
    let mut run_options = ProcLuaRunOptions::default();
    let Some(options) = options else {
        return Ok((parsed, run_options));
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, WAIT_GONE_SIGNATURE)?;
        match key.as_str() {
            "timeout_ms" => {
                parsed.timeout_ms = Some(parse_u64_value(value, WAIT_GONE_SIGNATURE, "timeout_ms")?)
            }
            "interval_ms" => {
                parsed.interval_ms = parse_u64_value(value, WAIT_GONE_SIGNATURE, "interval_ms")?
            }
            "check" => run_options.check = parse_bool_value(value, WAIT_GONE_SIGNATURE, "check")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    WAIT_GONE_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok((parsed, run_options))
}

fn parse_targets_value(value: Value, context: &str) -> mlua::Result<Vec<ProcTarget>> {
    let mut targets = Vec::new();
    collect_targets(value, context, &mut targets)?;
    Ok(targets)
}

fn collect_targets(value: Value, context: &str, out: &mut Vec<ProcTarget>) -> mlua::Result<()> {
    match value {
        Value::Integer(_) | Value::Number(_) => {
            let pid = parse_pid_value(value, context, "targets")?;
            out.push(ProcTarget {
                pid,
                start_time_unix_ms: None,
            });
            Ok(())
        }
        Value::Table(table) => {
            if is_sequence_table(&table)? {
                for value in table.sequence_values::<Value>() {
                    collect_targets(value?, context, out)?;
                }
                Ok(())
            } else {
                out.push(parse_proc_target_table(table, context)?);
                Ok(())
            }
        }
        _ => Err(crate::lua_error::invalid_argument(
            context,
            "expects a pid, a proc table, or an array of them",
        )),
    }
}

fn parse_proc_target_table(table: Table, context: &str) -> mlua::Result<ProcTarget> {
    let pid = parse_pid_value(table.get::<Value>("pid")?, context, "pid")?;
    let start_time_unix_ms = match table.get::<Value>("start_time_unix_ms")? {
        Value::Nil => None,
        value => Some(parse_u64_value(value, context, "start_time_unix_ms")?),
    };
    Ok(ProcTarget {
        pid,
        start_time_unix_ms,
    })
}

fn is_sequence_table(table: &Table) -> mlua::Result<bool> {
    if table.raw_len() == 0 {
        return Ok(false);
    }
    Ok(table.get::<Option<Value>>("pid")?.is_none())
}

fn parse_signal(value: &str) -> mlua::Result<ProcSignal> {
    match value {
        "hup" | "hangup" => Ok(ProcSignal::Hangup),
        "term" | "sigterm" => Ok(ProcSignal::Term),
        "kill" | "sigkill" => Ok(ProcSignal::Kill),
        "int" | "interrupt" | "sigint" => Ok(ProcSignal::Interrupt),
        "quit" | "sigquit" => Ok(ProcSignal::Quit),
        "stop" | "sigstop" => Ok(ProcSignal::Stop),
        "cont" | "continue" | "sigcont" => Ok(ProcSignal::Continue),
        "user1" | "sigusr1" => Ok(ProcSignal::User1),
        "user2" | "sigusr2" => Ok(ProcSignal::User2),
        _ => Err(crate::lua_error::invalid_option(
            KILL_SIGNATURE,
            "`signal` must be one of `hup`, `term`, `kill`, `int`, `quit`, `stop`, `cont`, `user1`, `user2`",
        )),
    }
}

fn proc_info_to_lua(lua: &Lua, info: ProcInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("pid", i64::from(info.pid))?;
    table.set("ppid", info.ppid.map(i64::from))?;
    table.set("name", info.name)?;
    table.set("exe", info.exe)?;
    table.set("cwd", info.cwd)?;
    table.set("user", info.user)?;
    table.set("cmdline", info.cmdline)?;
    table.set("argv", lua.create_sequence_from(info.argv)?)?;
    table.set("state", info.state)?;
    table.set(
        "start_time_unix_ms",
        i64::try_from(info.start_time_unix_ms).map_err(|_| {
            crate::lua_error::invalid_argument(
                SELF_SIGNATURE,
                "`start_time_unix_ms` is too large for Lua",
            )
        })?,
    )?;
    Ok(table)
}

fn build_kill_result(lua: &Lua, result: ProcKillResult) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("ok", result.ok)?;
    table.set("signal", result.signal.label())?;
    table.set("total", i64::try_from(result.total).unwrap_or(i64::MAX))?;
    table.set("sent", i64::try_from(result.sent).unwrap_or(i64::MAX))?;
    table.set("missing", i64::try_from(result.missing).unwrap_or(i64::MAX))?;
    table.set("failed", i64::try_from(result.failed).unwrap_or(i64::MAX))?;

    let entries = lua.create_table()?;
    for (index, entry) in result.entries.into_iter().enumerate() {
        let item = lua.create_table()?;
        item.set("pid", i64::from(entry.pid))?;
        item.set("ok", entry.ok)?;
        item.set("existed", entry.existed)?;
        item.set("signal", entry.signal.label())?;
        item.set("message", entry.message)?;
        entries.raw_set(index + 1, item)?;
    }
    table.set("entries", entries)?;

    let assert_ok_fn = lua.create_function(|_, this: Table| assert_kill_result(&this))?;
    table.set("assert_ok", assert_ok_fn)?;
    Ok(table)
}

fn build_wait_gone_result(lua: &Lua, result: ProcWaitGoneResult) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("ok", result.ok)?;
    table.set("timed_out", result.timed_out)?;
    table.set("total", i64::try_from(result.total).unwrap_or(i64::MAX))?;
    table.set(
        "gone",
        lua.create_sequence_from(result.gone.into_iter().map(i64::from).collect::<Vec<_>>())?,
    )?;
    table.set(
        "remaining",
        lua.create_sequence_from(
            result
                .remaining
                .into_iter()
                .map(i64::from)
                .collect::<Vec<_>>(),
        )?,
    )?;
    table.set(
        "elapsed_ms",
        i64::try_from(result.elapsed_ms).unwrap_or(i64::MAX),
    )?;

    let assert_ok_fn = lua.create_function(|_, this: Table| assert_wait_gone_result(&this))?;
    table.set("assert_ok", assert_ok_fn)?;
    Ok(table)
}

fn assert_kill_result(table: &Table) -> mlua::Result<()> {
    if table.get::<bool>("ok")? {
        return Ok(());
    }
    let signal = table.get::<String>("signal")?;
    let failed = table.get::<Option<i64>>("failed")?.unwrap_or_default();
    let missing = table.get::<Option<i64>>("missing")?.unwrap_or_default();
    let mut err = crate::lua_error::LuaError::new(
        "process_signal_failed",
        format!("ptool.proc.kill failed for {failed} target(s) with signal `{signal}`"),
    )
    .with_op("ptool.proc.kill")
    .with_detail(format!(
        "failed: {failed}, missing: {missing}, signal: {signal}"
    ));

    if let Some(message) = first_entry_message(table)? {
        err = err.with_stderr(message);
    }
    Err(crate::lua_error::to_mlua_error(err))
}

fn assert_wait_gone_result(table: &Table) -> mlua::Result<()> {
    if table.get::<bool>("ok")? {
        return Ok(());
    }
    let elapsed_ms = table.get::<Option<i64>>("elapsed_ms")?.unwrap_or_default();
    let remaining = table.get::<Table>("remaining")?;
    let remaining_pids = remaining
        .sequence_values::<i64>()
        .collect::<mlua::Result<Vec<_>>>()?
        .into_iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let err = crate::lua_error::LuaError::new(
        "timed_out",
        format!("ptool.proc.wait_gone timed out after {elapsed_ms}ms"),
    )
    .with_op("ptool.proc.wait_gone")
    .with_detail(format!("remaining: {remaining_pids}"));
    Err(crate::lua_error::to_mlua_error(err))
}

fn first_entry_message(table: &Table) -> mlua::Result<Option<String>> {
    let entries = table.get::<Table>("entries")?;
    for entry in entries.sequence_values::<Table>() {
        let entry = entry?;
        let ok = entry.get::<bool>("ok")?;
        if !ok {
            let pid = entry.get::<i64>("pid")?;
            let message = entry.get::<Option<String>>("message")?;
            return Ok(Some(match message {
                Some(message) => format!("pid {pid}: {message}"),
                None => format!("pid {pid}: signal failed"),
            }));
        }
    }
    Ok(None)
}

fn confirm_kill(
    engine: &PtoolEngine,
    targets: &[ProcTarget],
    signal: ProcSignal,
) -> mlua::Result<()> {
    let count = targets.len();
    let pid_preview = targets
        .iter()
        .take(5)
        .map(|target| target.pid.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let suffix = if count > 5 { ", ..." } else { "" };
    let prompt = format!(
        "Signal {} process(es) with `{}`? ({pid_preview}{suffix})",
        count,
        signal.label()
    );
    match engine.prompt_confirm(
        "ptool.proc.kill",
        &prompt,
        ptool_engine::PromptConfirmOptions {
            default: Some(true),
            help: None,
        },
    ) {
        Ok(true) => Ok(()),
        Ok(false) => Err(crate::lua_error::to_mlua_error(
            crate::lua_error::LuaError::cancelled("ptool.proc.kill", "cancelled by user"),
        )),
        Err(err) => Err(crate::lua_error::lua_error_from_engine(err, KILL_SIGNATURE)),
    }
}

fn parse_option_key(value: Value, context: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_option(
            context,
            "option keys must be strings",
        )),
    }
}

fn parse_non_empty_string_value(value: Value, context: &str, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => {
            let value = value.to_str()?.to_string();
            if value.is_empty() {
                return Err(crate::lua_error::invalid_option(
                    context,
                    format!("`{field}` must not be empty"),
                ));
            }
            Ok(value)
        }
        _ => Err(crate::lua_error::invalid_option(
            context,
            format!("`{field}` must be a string"),
        )),
    }
}

fn parse_bool_value(value: Value, context: &str, field: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(value) => Ok(value),
        _ => Err(crate::lua_error::invalid_option(
            context,
            format!("`{field}` must be a boolean"),
        )),
    }
}

fn parse_pid_value(value: Value, context: &str, field: &str) -> mlua::Result<u32> {
    let value = match value {
        Value::Integer(value) => value,
        Value::Number(value) if value.fract() == 0.0 => value as i64,
        _ => {
            return Err(crate::lua_error::invalid_argument(
                context,
                format!("`{field}` must be an integer pid"),
            ));
        }
    };
    if value <= 0 {
        return Err(crate::lua_error::invalid_argument(
            context,
            format!("`{field}` must be > 0"),
        ));
    }
    u32::try_from(value)
        .map_err(|_| crate::lua_error::invalid_argument(context, format!("`{field}` is too large")))
}

fn parse_pid_list_value(value: Value, context: &str, field: &str) -> mlua::Result<Vec<u32>> {
    let Value::Table(table) = value else {
        return Err(crate::lua_error::invalid_option(
            context,
            format!("`{field}` must be an array of pids"),
        ));
    };
    let mut pids = Vec::new();
    for value in table.sequence_values::<Value>() {
        pids.push(parse_pid_value(value?, context, field)?);
    }
    Ok(pids)
}

fn parse_usize_value(value: Value, context: &str, field: &str) -> mlua::Result<usize> {
    let value = parse_u64_value(value, context, field)?;
    usize::try_from(value)
        .map_err(|_| crate::lua_error::invalid_option(context, format!("`{field}` is too large")))
}

fn parse_u64_value(value: Value, context: &str, field: &str) -> mlua::Result<u64> {
    let value = match value {
        Value::Integer(value) => value,
        Value::Number(value) if value.fract() == 0.0 => value as i64,
        _ => {
            return Err(crate::lua_error::invalid_option(
                context,
                format!("`{field}` must be an integer"),
            ));
        }
    };
    if value < 0 {
        return Err(crate::lua_error::invalid_option(
            context,
            format!("`{field}` must be >= 0"),
        ));
    }
    u64::try_from(value)
        .map_err(|_| crate::lua_error::invalid_option(context, format!("`{field}` is too large")))
}
