use super::options::{parse_bool_option, parse_option_key};
use super::{LuaGitRepo, confirm_git_action};
use mlua::{Lua, Table, Value};
use ptool_console::GitAction;
use ptool_engine::{
    GitBranchCreateOptions, GitBranchDeleteOptions, GitBranchInfo, GitBranchKind,
    GitBranchListOptions,
};

pub(super) const BRANCHES_SIGNATURE: &str = "ptool.git.Repo:branches(options?)";
pub(super) const BRANCH_CREATE_SIGNATURE: &str = "ptool.git.Repo:branch_create(name, options?)";
pub(super) const BRANCH_DELETE_SIGNATURE: &str = "ptool.git.Repo:branch_delete(name, options?)";
pub(super) const BRANCH_RENAME_SIGNATURE: &str =
    "ptool.git.Repo:branch_rename(old_name, new_name, options?)";
pub(super) const BRANCH_UPSTREAM_SIGNATURE: &str =
    "ptool.git.Repo:branch_set_upstream(name, upstream_or_nil, options?)";

pub(super) fn branches(
    repo: &LuaGitRepo,
    lua: &Lua,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let options = parse_branch_list_options(options)?;
    let values = repo
        .repo
        .branches(options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, BRANCHES_SIGNATURE))?;
    branches_to_lua(lua, values)
}

pub(super) fn create(
    repo: &LuaGitRepo,
    lua: &Lua,
    name: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let (options, confirm) = parse_branch_create_options(options)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            BRANCH_CREATE_SIGNATURE,
            GitAction::CreateBranch {
                repository: &repository,
                branch: &name,
            },
        )?;
    }
    let value = repo
        .repo
        .branch_create(&name, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, BRANCH_CREATE_SIGNATURE))?;
    branch_to_lua(lua, value)
}

pub(super) fn delete(repo: &LuaGitRepo, name: String, options: Option<Table>) -> mlua::Result<()> {
    let (options, confirm) = parse_branch_delete_options(options)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            BRANCH_DELETE_SIGNATURE,
            GitAction::DeleteBranch {
                repository: &repository,
                branch: &name,
            },
        )?;
    }
    repo.repo
        .branch_delete(&name, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, BRANCH_DELETE_SIGNATURE))
}

pub(super) fn rename(
    repo: &LuaGitRepo,
    lua: &Lua,
    old: String,
    new: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let (force, confirm) = parse_force_confirm(options, BRANCH_RENAME_SIGNATURE)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            BRANCH_RENAME_SIGNATURE,
            GitAction::RenameBranch {
                repository: &repository,
                branch: &old,
                new_name: &new,
            },
        )?;
    }
    let value = repo
        .repo
        .branch_rename(&old, &new, force)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, BRANCH_RENAME_SIGNATURE))?;
    branch_to_lua(lua, value)
}

pub(super) fn set_upstream(
    repo: &LuaGitRepo,
    name: String,
    upstream: Option<String>,
    options: Option<Table>,
) -> mlua::Result<()> {
    let confirm = parse_confirm_only(options, BRANCH_UPSTREAM_SIGNATURE)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            BRANCH_UPSTREAM_SIGNATURE,
            GitAction::SetUpstream {
                repository: &repository,
                branch: &name,
                upstream: upstream.as_deref(),
            },
        )?;
    }
    repo.repo
        .branch_set_upstream(&name, upstream.as_deref())
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, BRANCH_UPSTREAM_SIGNATURE))
}

fn parse_branch_list_options(options: Option<Table>) -> mlua::Result<GitBranchListOptions> {
    let mut parsed = GitBranchListOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, BRANCHES_SIGNATURE)?;
        match key.as_str() {
            "kind" => match value {
                Value::String(value) => {
                    parsed.kind = match value.to_str()?.as_ref() {
                        "local" => GitBranchKind::Local,
                        "remote" => GitBranchKind::Remote,
                        "all" => GitBranchKind::All,
                        _ => {
                            return Err(crate::lua_error::invalid_option(
                                BRANCHES_SIGNATURE,
                                "`kind` must be `local`, `remote`, or `all`",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        BRANCHES_SIGNATURE,
                        "`kind` must be a string",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    BRANCHES_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_branch_create_options(
    options: Option<Table>,
) -> mlua::Result<(GitBranchCreateOptions, bool)> {
    let mut parsed = GitBranchCreateOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok((parsed, confirm));
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, BRANCH_CREATE_SIGNATURE)?;
        match key.as_str() {
            "start_point" => parsed.start_point = Some(parse_string(value, "start_point")?),
            "force" => parsed.force = parse_bool_option(value, BRANCH_CREATE_SIGNATURE, "force")?,
            "checkout" => {
                parsed.checkout = parse_bool_option(value, BRANCH_CREATE_SIGNATURE, "checkout")?
            }
            "upstream" => parsed.upstream = Some(parse_string(value, "upstream")?),
            "confirm" => confirm = parse_bool_option(value, BRANCH_CREATE_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    BRANCH_CREATE_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok((parsed, confirm))
}

fn parse_branch_delete_options(
    options: Option<Table>,
) -> mlua::Result<(GitBranchDeleteOptions, bool)> {
    let mut parsed = GitBranchDeleteOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok((parsed, confirm));
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, BRANCH_DELETE_SIGNATURE)?;
        match key.as_str() {
            "force" => parsed.force = parse_bool_option(value, BRANCH_DELETE_SIGNATURE, "force")?,
            "confirm" => confirm = parse_bool_option(value, BRANCH_DELETE_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    BRANCH_DELETE_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok((parsed, confirm))
}

fn parse_force_confirm(options: Option<Table>, op: &str) -> mlua::Result<(bool, bool)> {
    let mut force = false;
    let mut confirm = false;
    let Some(options) = options else {
        return Ok((force, confirm));
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, op)?;
        match key.as_str() {
            "force" => force = parse_bool_option(value, op, "force")?,
            "confirm" => confirm = parse_bool_option(value, op, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok((force, confirm))
}

fn parse_confirm_only(options: Option<Table>, op: &str) -> mlua::Result<bool> {
    let (_, confirm) = parse_force_confirm(options, op)?;
    Ok(confirm)
}

fn parse_string(value: Value, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_option(
            BRANCH_CREATE_SIGNATURE,
            format!("`{field}` must be a string"),
        )),
    }
}

fn branches_to_lua(lua: &Lua, values: Vec<GitBranchInfo>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, value) in values.into_iter().enumerate() {
        table.set(index + 1, branch_to_lua(lua, value)?)?;
    }
    Ok(table)
}

fn branch_to_lua(lua: &Lua, value: GitBranchInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("name", value.name)?;
    table.set("kind", value.kind)?;
    table.set("oid", value.oid)?;
    table.set("head", value.head)?;
    table.set("upstream", value.upstream)?;
    table.set("ahead", to_i64(value.ahead)?)?;
    table.set("behind", to_i64(value.behind)?)?;
    Ok(table)
}

fn to_i64(value: usize) -> mlua::Result<i64> {
    i64::try_from(value).map_err(|_| {
        crate::lua_error::invalid_argument(BRANCHES_SIGNATURE, "branch count is too large")
    })
}
