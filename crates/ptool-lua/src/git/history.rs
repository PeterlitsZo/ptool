use super::options::{parse_bool_option, parse_option_key, parse_string_list_from_value};
use super::{DESCRIBE_SIGNATURE, DIFF_SIGNATURE, LOG_SIGNATURE, LuaGitRepo, RESOLVE_SIGNATURE};
use mlua::{Lua, Table, Value};
use ptool_engine::{
    GitCommitInfo, GitDescribeOptions, GitDiffDelta, GitDiffOptions, GitDiffSummary, GitLogOptions,
    GitObjectInfo, GitSignatureInfo,
};

pub(super) fn resolve(repo: &LuaGitRepo, lua: &Lua, rev: String) -> mlua::Result<Table> {
    let value = repo
        .repo
        .resolve(&rev)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, RESOLVE_SIGNATURE))?;
    object_to_lua(lua, value)
}

pub(super) fn commit_info(
    repo: &LuaGitRepo,
    lua: &Lua,
    rev: Option<String>,
) -> mlua::Result<Table> {
    let value = repo.repo.commit_info(rev.as_deref()).map_err(|err| {
        crate::lua_error::lua_error_from_engine(err, super::COMMIT_INFO_SIGNATURE)
    })?;
    commit_to_lua(lua, value)
}

pub(super) fn log(repo: &LuaGitRepo, lua: &Lua, options: Option<Table>) -> mlua::Result<Table> {
    let options = parse_log_options(options)?;
    let values = repo
        .repo
        .log(options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, LOG_SIGNATURE))?;
    let table = lua.create_table()?;
    for (index, value) in values.into_iter().enumerate() {
        table.set(index + 1, commit_to_lua(lua, value)?)?;
    }
    Ok(table)
}

pub(super) fn diff(repo: &LuaGitRepo, lua: &Lua, options: Option<Table>) -> mlua::Result<Table> {
    let options = parse_diff_options(options)?;
    let value = repo
        .repo
        .diff(options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, DIFF_SIGNATURE))?;
    diff_to_lua(lua, value)
}

pub(super) fn describe(repo: &LuaGitRepo, options: Option<Table>) -> mlua::Result<Option<String>> {
    let options = parse_describe_options(options)?;
    repo.repo
        .describe(options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, DESCRIBE_SIGNATURE))
}

fn parse_log_options(options: Option<Table>) -> mlua::Result<GitLogOptions> {
    let mut parsed = GitLogOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, LOG_SIGNATURE)?;
        match key.as_str() {
            "rev" => parsed.rev = parse_string(value, LOG_SIGNATURE, "rev")?,
            "max_count" => parsed.max_count = parse_usize(value, LOG_SIGNATURE, "max_count")?,
            "skip" => parsed.skip = parse_usize(value, LOG_SIGNATURE, "skip")?,
            "first_parent" => {
                parsed.first_parent = parse_bool_option(value, LOG_SIGNATURE, "first_parent")?
            }
            "reverse" => parsed.reverse = parse_bool_option(value, LOG_SIGNATURE, "reverse")?,
            "paths" => parsed.paths = parse_string_list_from_value(value, LOG_SIGNATURE, "paths")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    LOG_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_diff_options(options: Option<Table>) -> mlua::Result<GitDiffOptions> {
    let mut parsed = GitDiffOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, DIFF_SIGNATURE)?;
        match key.as_str() {
            "from" => parsed.from = Some(parse_string(value, DIFF_SIGNATURE, "from")?),
            "to" => parsed.to = Some(parse_string(value, DIFF_SIGNATURE, "to")?),
            "cached" => parsed.cached = parse_bool_option(value, DIFF_SIGNATURE, "cached")?,
            "paths" => parsed.paths = parse_string_list_from_value(value, DIFF_SIGNATURE, "paths")?,
            "context_lines" => {
                let value = parse_usize(value, DIFF_SIGNATURE, "context_lines")?;
                parsed.context_lines = u32::try_from(value).map_err(|_| {
                    crate::lua_error::invalid_option(DIFF_SIGNATURE, "`context_lines` is too large")
                })?;
            }
            "find_renames" => {
                parsed.find_renames = parse_bool_option(value, DIFF_SIGNATURE, "find_renames")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    DIFF_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_describe_options(options: Option<Table>) -> mlua::Result<GitDescribeOptions> {
    let mut parsed = GitDescribeOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, DESCRIBE_SIGNATURE)?;
        match key.as_str() {
            "rev" => parsed.rev = Some(parse_string(value, DESCRIBE_SIGNATURE, "rev")?),
            "pattern" => parsed.pattern = Some(parse_string(value, DESCRIBE_SIGNATURE, "pattern")?),
            "always" => parsed.always = parse_bool_option(value, DESCRIBE_SIGNATURE, "always")?,
            "abbrev" => parsed.abbrev = parse_usize(value, DESCRIBE_SIGNATURE, "abbrev")?,
            "dirty_suffix" => {
                parsed.dirty_suffix = Some(parse_string(value, DESCRIBE_SIGNATURE, "dirty_suffix")?)
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    DESCRIBE_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_string(value: Value, op: &str, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_option(
            op,
            format!("`{field}` must be a string"),
        )),
    }
}

fn parse_usize(value: Value, op: &str, field: &str) -> mlua::Result<usize> {
    match value {
        Value::Integer(value) if value >= 0 => usize::try_from(value)
            .map_err(|_| crate::lua_error::invalid_option(op, format!("`{field}` is too large"))),
        _ => Err(crate::lua_error::invalid_option(
            op,
            format!("`{field}` must be a non-negative integer"),
        )),
    }
}

fn object_to_lua(lua: &Lua, value: GitObjectInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("oid", value.oid)?;
    table.set("kind", value.kind)?;
    table.set("shorthand", value.shorthand)?;
    Ok(table)
}

pub(super) fn commit_to_lua(lua: &Lua, value: GitCommitInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("oid", value.oid)?;
    table.set("summary", value.summary)?;
    table.set("message", value.message)?;
    table.set("author", signature_to_lua(lua, value.author)?)?;
    table.set("committer", signature_to_lua(lua, value.committer)?)?;
    let parents = lua.create_table()?;
    for (index, oid) in value.parent_oids.into_iter().enumerate() {
        parents.set(index + 1, oid)?;
    }
    table.set("parent_oids", parents)?;
    Ok(table)
}

fn signature_to_lua(lua: &Lua, value: GitSignatureInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("name", value.name)?;
    table.set("email", value.email)?;
    table.set("time_seconds", value.time_seconds)?;
    table.set("offset_minutes", value.offset_minutes)?;
    Ok(table)
}

fn diff_to_lua(lua: &Lua, value: GitDiffSummary) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("patch", value.patch)?;
    table.set(
        "files_changed",
        to_i64(value.files_changed, DIFF_SIGNATURE)?,
    )?;
    table.set("insertions", to_i64(value.insertions, DIFF_SIGNATURE)?)?;
    table.set("deletions", to_i64(value.deletions, DIFF_SIGNATURE)?)?;
    let deltas = lua.create_table()?;
    for (index, delta) in value.deltas.into_iter().enumerate() {
        deltas.set(index + 1, delta_to_lua(lua, delta)?)?;
    }
    table.set("deltas", deltas)?;
    Ok(table)
}

fn delta_to_lua(lua: &Lua, value: GitDiffDelta) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("status", value.status)?;
    table.set("old_path", value.old_path)?;
    table.set("new_path", value.new_path)?;
    table.set("binary", value.binary)?;
    Ok(table)
}

fn to_i64(value: usize, op: &str) -> mlua::Result<i64> {
    i64::try_from(value).map_err(|_| crate::lua_error::invalid_argument(op, "value is too large"))
}
