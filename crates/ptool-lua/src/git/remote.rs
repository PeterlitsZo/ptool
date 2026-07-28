use super::options::{parse_bool_option, parse_option_key};
use super::{LuaGitRepo, confirm_git_action};
use mlua::{Lua, Table, Value};
use ptool_console::GitAction;
use ptool_engine::{GitRemoteAddOptions, GitRemoteInfo};

pub(super) const REMOTES_SIGNATURE: &str = "ptool.git.Repo:remotes()";
pub(super) const REMOTE_SIGNATURE: &str = "ptool.git.Repo:remote(name)";
pub(super) const REMOTE_ADD_SIGNATURE: &str = "ptool.git.Repo:remote_add(name, url, options?)";
pub(super) const REMOTE_REMOVE_SIGNATURE: &str = "ptool.git.Repo:remote_remove(name, options?)";
pub(super) const REMOTE_RENAME_SIGNATURE: &str =
    "ptool.git.Repo:remote_rename(name, new_name, options?)";
pub(super) const REMOTE_SET_URL_SIGNATURE: &str =
    "ptool.git.Repo:remote_set_url(name, url, options?)";

pub(super) fn remotes(repo: &LuaGitRepo, lua: &Lua) -> mlua::Result<Table> {
    let values = repo
        .repo
        .remotes()
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REMOTES_SIGNATURE))?;
    remotes_to_lua(lua, values)
}

pub(super) fn remote(repo: &LuaGitRepo, lua: &Lua, name: String) -> mlua::Result<Table> {
    let value = repo
        .repo
        .remote_info(&name)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REMOTE_SIGNATURE))?;
    remote_to_lua(lua, value)
}

pub(super) fn add(
    repo: &LuaGitRepo,
    lua: &Lua,
    name: String,
    url: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let (options, confirm) = parse_add_options(options)?;
    confirm_change(repo, &name, "Add", confirm, REMOTE_ADD_SIGNATURE)?;
    let value = repo
        .repo
        .remote_add(&name, &url, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REMOTE_ADD_SIGNATURE))?;
    remote_to_lua(lua, value)
}

pub(super) fn remove(repo: &LuaGitRepo, name: String, options: Option<Table>) -> mlua::Result<()> {
    let confirm = parse_confirm_only(options, REMOTE_REMOVE_SIGNATURE)?;
    confirm_change(repo, &name, "Remove", confirm, REMOTE_REMOVE_SIGNATURE)?;
    repo.repo
        .remote_remove(&name)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REMOTE_REMOVE_SIGNATURE))
}

pub(super) fn rename(
    repo: &LuaGitRepo,
    lua: &Lua,
    name: String,
    new_name: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let confirm = parse_confirm_only(options, REMOTE_RENAME_SIGNATURE)?;
    confirm_change(repo, &name, "Rename", confirm, REMOTE_RENAME_SIGNATURE)?;
    let value = repo
        .repo
        .remote_rename(&name, &new_name)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REMOTE_RENAME_SIGNATURE))?;
    remote_to_lua(lua, value)
}

pub(super) fn set_url(
    repo: &LuaGitRepo,
    lua: &Lua,
    name: String,
    url: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let (push, confirm) = parse_set_url_options(options)?;
    confirm_change(
        repo,
        &name,
        "Set URL for",
        confirm,
        REMOTE_SET_URL_SIGNATURE,
    )?;
    let value = repo
        .repo
        .remote_set_url(&name, &url, push)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REMOTE_SET_URL_SIGNATURE))?;
    remote_to_lua(lua, value)
}

fn confirm_change(
    repo: &LuaGitRepo,
    name: &str,
    operation: &str,
    confirm: bool,
    signature: &'static str,
) -> mlua::Result<()> {
    if !confirm {
        return Ok(());
    }
    let repository = repo.repo_label();
    confirm_git_action(
        &repo.engine,
        signature,
        GitAction::ChangeRemote {
            repository: &repository,
            operation,
            remote: name,
        },
    )
}

fn parse_add_options(options: Option<Table>) -> mlua::Result<(GitRemoteAddOptions, bool)> {
    let mut parsed = GitRemoteAddOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok((parsed, confirm));
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, REMOTE_ADD_SIGNATURE)?;
        match key.as_str() {
            "push_url" => match value {
                Value::String(value) => parsed.push_url = Some(value.to_str()?.to_string()),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        REMOTE_ADD_SIGNATURE,
                        "`push_url` must be a string",
                    ));
                }
            },
            "confirm" => confirm = parse_bool_option(value, REMOTE_ADD_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    REMOTE_ADD_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok((parsed, confirm))
}

fn parse_set_url_options(options: Option<Table>) -> mlua::Result<(bool, bool)> {
    let mut push = false;
    let mut confirm = false;
    let Some(options) = options else {
        return Ok((push, confirm));
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, REMOTE_SET_URL_SIGNATURE)?;
        match key.as_str() {
            "push" => push = parse_bool_option(value, REMOTE_SET_URL_SIGNATURE, "push")?,
            "confirm" => confirm = parse_bool_option(value, REMOTE_SET_URL_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    REMOTE_SET_URL_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok((push, confirm))
}

fn parse_confirm_only(options: Option<Table>, op: &str) -> mlua::Result<bool> {
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(confirm);
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, op)?;
        match key.as_str() {
            "confirm" => confirm = parse_bool_option(value, op, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(confirm)
}

fn remotes_to_lua(lua: &Lua, values: Vec<GitRemoteInfo>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, value) in values.into_iter().enumerate() {
        table.set(index + 1, remote_to_lua(lua, value)?)?;
    }
    Ok(table)
}

fn remote_to_lua(lua: &Lua, value: GitRemoteInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("name", value.name)?;
    table.set("url", value.url)?;
    table.set("push_url", value.push_url)?;
    table.set("fetch_refspecs", strings_to_lua(lua, value.fetch_refspecs)?)?;
    table.set("push_refspecs", strings_to_lua(lua, value.push_refspecs)?)?;
    Ok(table)
}

fn strings_to_lua(lua: &Lua, values: Vec<String>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, value) in values.into_iter().enumerate() {
        table.set(index + 1, value)?;
    }
    Ok(table)
}
