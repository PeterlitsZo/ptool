use super::{ConfirmableOptions, LuaGitRepo, confirm_git_action};
use mlua::{Lua, Table, Value};
use ptool_console::GitAction;
use ptool_engine::{GitConfigEntry, GitConfigScope, GitConfigValue};

pub(super) const CONFIG_GET_SIGNATURE: &str = "ptool.git.Repo:config_get(name, options?)";
pub(super) const CONFIG_LIST_SIGNATURE: &str = "ptool.git.Repo:config_list(options?)";
pub(super) const CONFIG_SET_SIGNATURE: &str = "ptool.git.Repo:config_set(name, value, options?)";
pub(super) const CONFIG_REMOVE_SIGNATURE: &str = "ptool.git.Repo:config_remove(name, options?)";

pub(super) fn get(
    repo: &LuaGitRepo,
    lua: &Lua,
    name: String,
    options: Option<Table>,
) -> mlua::Result<Value> {
    let scope = parse_read_options(options, CONFIG_GET_SIGNATURE)?;
    let value = repo
        .repo
        .config_get(&name, scope)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, CONFIG_GET_SIGNATURE))?;
    config_value_to_lua(lua, value)
}

pub(super) fn list(repo: &LuaGitRepo, lua: &Lua, options: Option<Table>) -> mlua::Result<Table> {
    let scope = parse_read_options(options, CONFIG_LIST_SIGNATURE)?;
    let entries = repo
        .repo
        .config_list(scope)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, CONFIG_LIST_SIGNATURE))?;
    config_entries_to_lua(lua, entries)
}

pub(super) fn set(
    repo: &LuaGitRepo,
    lua: &Lua,
    name: String,
    value: Value,
    options: Option<Table>,
) -> mlua::Result<()> {
    let options = parse_mutation_options(options, CONFIG_SET_SIGNATURE)?;
    require_global_confirmation(options.inner, options.confirm, CONFIG_SET_SIGNATURE)?;
    if options.confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            CONFIG_SET_SIGNATURE,
            GitAction::Config {
                repository: &repository,
                scope: options.inner.as_str(),
                name: &name,
            },
        )?;
    }
    let value = config_value_from_lua(lua, value, CONFIG_SET_SIGNATURE)?;
    repo.repo
        .config_set(&name, value, options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, CONFIG_SET_SIGNATURE))
}

pub(super) fn remove(repo: &LuaGitRepo, name: String, options: Option<Table>) -> mlua::Result<()> {
    let options = parse_mutation_options(options, CONFIG_REMOVE_SIGNATURE)?;
    require_global_confirmation(options.inner, options.confirm, CONFIG_REMOVE_SIGNATURE)?;
    if options.confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            CONFIG_REMOVE_SIGNATURE,
            GitAction::Config {
                repository: &repository,
                scope: options.inner.as_str(),
                name: &name,
            },
        )?;
    }
    repo.repo
        .config_remove(&name, options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, CONFIG_REMOVE_SIGNATURE))
}

fn parse_read_options(options: Option<Table>, op: &str) -> mlua::Result<Option<GitConfigScope>> {
    let Some(options) = options else {
        return Ok(None);
    };
    let mut scope = None;
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, op)?;
        match key.as_str() {
            "scope" => scope = Some(parse_scope(value, op)?),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(scope)
}

fn parse_mutation_options(
    options: Option<Table>,
    op: &str,
) -> mlua::Result<ConfirmableOptions<GitConfigScope>> {
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: GitConfigScope::Local,
            confirm: false,
        });
    };
    let mut scope = GitConfigScope::Local;
    let mut confirm = false;
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, op)?;
        match key.as_str() {
            "scope" => scope = parse_scope(value, op)?,
            "confirm" => {
                confirm = super::options::parse_bool_option(value, op, "confirm")?;
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(ConfirmableOptions {
        inner: scope,
        confirm,
    })
}

fn parse_scope(value: Value, op: &str) -> mlua::Result<GitConfigScope> {
    let Value::String(value) = value else {
        return Err(crate::lua_error::invalid_option(
            op,
            "`scope` must be `local`, `global`, or `system`",
        ));
    };
    match value.to_str()?.as_ref() {
        "local" => Ok(GitConfigScope::Local),
        "global" => Ok(GitConfigScope::Global),
        "system" => Ok(GitConfigScope::System),
        _ => Err(crate::lua_error::invalid_option(
            op,
            "`scope` must be `local`, `global`, or `system`",
        )),
    }
}

fn require_global_confirmation(scope: GitConfigScope, confirm: bool, op: &str) -> mlua::Result<()> {
    if scope == GitConfigScope::Global && !confirm {
        return Err(crate::lua_error::invalid_option(
            op,
            "global config mutations require `confirm = true`",
        ));
    }
    Ok(())
}

fn config_value_from_lua(_lua: &Lua, value: Value, op: &str) -> mlua::Result<GitConfigValue> {
    match value {
        Value::String(value) => Ok(GitConfigValue::String(value.to_str()?.to_string())),
        Value::Boolean(value) => Ok(GitConfigValue::Boolean(value)),
        Value::Integer(value) => Ok(GitConfigValue::Integer(value)),
        _ => Err(crate::lua_error::invalid_argument(
            op,
            "config value must be a string, boolean, or integer",
        )),
    }
}

fn config_value_to_lua(lua: &Lua, value: Option<GitConfigValue>) -> mlua::Result<Value> {
    match value {
        None => Ok(Value::Nil),
        Some(GitConfigValue::String(value)) => Ok(Value::String(lua.create_string(&value)?)),
        Some(GitConfigValue::Boolean(value)) => Ok(Value::Boolean(value)),
        Some(GitConfigValue::Integer(value)) => Ok(Value::Integer(value)),
    }
}

fn config_entries_to_lua(lua: &Lua, entries: Vec<GitConfigEntry>) -> mlua::Result<Table> {
    let result = lua.create_table()?;
    for (index, entry) in entries.into_iter().enumerate() {
        let item = lua.create_table()?;
        item.set("name", entry.name)?;
        item.set("value", config_value_to_lua(lua, Some(entry.value))?)?;
        item.set("scope", entry.scope)?;
        result.set(index + 1, item)?;
    }
    Ok(result)
}
