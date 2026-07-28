use super::options::{
    parse_bool_option, parse_option_key, parse_paths, parse_string_list_from_value,
};
use super::{LuaGitRepo, confirm_git_action};
use mlua::{Lua, Table, Value};
use ptool_console::GitAction;
use ptool_engine::{
    GitCleanOptions, GitRemoveOptions, GitResetMode, GitResetOptions, GitRestoreOptions,
};

pub(super) const RESTORE_SIGNATURE: &str = "ptool.git.Repo:restore(paths, options?)";
pub(super) const RESET_SIGNATURE: &str = "ptool.git.Repo:reset(rev?, options?)";
pub(super) const REMOVE_SIGNATURE: &str = "ptool.git.Repo:remove(paths, options?)";
pub(super) const CLEAN_SIGNATURE: &str = "ptool.git.Repo:clean(options?)";

pub(super) fn restore(repo: &LuaGitRepo, paths: Value, options: Option<Table>) -> mlua::Result<()> {
    let paths = parse_paths(paths, RESTORE_SIGNATURE)?;
    let (options, confirm) = parse_restore_options(options)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            RESTORE_SIGNATURE,
            GitAction::Restore {
                repository: &repository,
                paths: &paths,
            },
        )?;
    }
    repo.repo
        .restore(&paths, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, RESTORE_SIGNATURE))
}

pub(super) fn reset(
    repo: &LuaGitRepo,
    rev: Option<String>,
    options: Option<Table>,
) -> mlua::Result<()> {
    let (options, confirm) = parse_reset_options(options)?;
    let revision = rev.as_deref().unwrap_or("HEAD");
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            RESET_SIGNATURE,
            GitAction::Reset {
                repository: &repository,
                revision,
                mode: options.mode.as_str(),
            },
        )?;
    }
    repo.repo
        .reset(rev.as_deref(), options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, RESET_SIGNATURE))
}

pub(super) fn remove(repo: &LuaGitRepo, paths: Value, options: Option<Table>) -> mlua::Result<()> {
    let paths = parse_paths(paths, REMOVE_SIGNATURE)?;
    let (options, confirm) = parse_remove_options(options)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            REMOVE_SIGNATURE,
            GitAction::Remove {
                repository: &repository,
                paths: &paths,
            },
        )?;
    }
    repo.repo
        .remove(&paths, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REMOVE_SIGNATURE))
}

pub(super) fn clean(repo: &LuaGitRepo, lua: &Lua, options: Option<Table>) -> mlua::Result<Table> {
    let (options, confirm) = parse_clean_options(options)?;
    let preview = repo
        .repo
        .clean(GitCleanOptions {
            dry_run: true,
            force: false,
            ..options.clone()
        })
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, CLEAN_SIGNATURE))?;
    if confirm && !options.dry_run {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            CLEAN_SIGNATURE,
            GitAction::Clean {
                repository: &repository,
                paths: &preview,
            },
        )?;
    }
    let paths = if options.dry_run {
        preview
    } else {
        repo.repo
            .clean(options)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, CLEAN_SIGNATURE))?
    };
    strings_to_lua(lua, paths)
}

fn parse_restore_options(options: Option<Table>) -> mlua::Result<(GitRestoreOptions, bool)> {
    let mut parsed = GitRestoreOptions::default();
    let mut confirm = false;
    let mut worktree_explicit = false;
    let Some(options) = options else {
        return Ok((parsed, confirm));
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, RESTORE_SIGNATURE)?;
        match key.as_str() {
            "source" => match value {
                Value::String(value) => parsed.source = value.to_str()?.to_string(),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        RESTORE_SIGNATURE,
                        "`source` must be a string",
                    ));
                }
            },
            "staged" => parsed.staged = parse_bool_option(value, RESTORE_SIGNATURE, "staged")?,
            "worktree" => {
                parsed.worktree = parse_bool_option(value, RESTORE_SIGNATURE, "worktree")?;
                worktree_explicit = true;
            }
            "confirm" => confirm = parse_bool_option(value, RESTORE_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    RESTORE_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    if parsed.staged && !worktree_explicit {
        parsed.worktree = false;
    }
    Ok((parsed, confirm))
}

fn parse_reset_options(options: Option<Table>) -> mlua::Result<(GitResetOptions, bool)> {
    let mut parsed = GitResetOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok((parsed, confirm));
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, RESET_SIGNATURE)?;
        match key.as_str() {
            "mode" => match value {
                Value::String(value) => {
                    parsed.mode = match value.to_str()?.as_ref() {
                        "soft" => GitResetMode::Soft,
                        "mixed" => GitResetMode::Mixed,
                        "hard" => GitResetMode::Hard,
                        _ => {
                            return Err(crate::lua_error::invalid_option(
                                RESET_SIGNATURE,
                                "`mode` must be `soft`, `mixed`, or `hard`",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        RESET_SIGNATURE,
                        "`mode` must be a string",
                    ));
                }
            },
            "force" => parsed.force = parse_bool_option(value, RESET_SIGNATURE, "force")?,
            "confirm" => confirm = parse_bool_option(value, RESET_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    RESET_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok((parsed, confirm))
}

fn parse_remove_options(options: Option<Table>) -> mlua::Result<(GitRemoveOptions, bool)> {
    let mut parsed = GitRemoveOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok((parsed, confirm));
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, REMOVE_SIGNATURE)?;
        match key.as_str() {
            "cached" => parsed.cached = parse_bool_option(value, REMOVE_SIGNATURE, "cached")?,
            "force" => parsed.force = parse_bool_option(value, REMOVE_SIGNATURE, "force")?,
            "confirm" => confirm = parse_bool_option(value, REMOVE_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    REMOVE_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok((parsed, confirm))
}

fn parse_clean_options(options: Option<Table>) -> mlua::Result<(GitCleanOptions, bool)> {
    let mut parsed = GitCleanOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok((parsed, confirm));
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, CLEAN_SIGNATURE)?;
        match key.as_str() {
            "dry_run" => parsed.dry_run = parse_bool_option(value, CLEAN_SIGNATURE, "dry_run")?,
            "force" => parsed.force = parse_bool_option(value, CLEAN_SIGNATURE, "force")?,
            "dirs" => parsed.dirs = parse_bool_option(value, CLEAN_SIGNATURE, "dirs")?,
            "ignored" => parsed.ignored = parse_bool_option(value, CLEAN_SIGNATURE, "ignored")?,
            "paths" => {
                parsed.paths = parse_string_list_from_value(value, CLEAN_SIGNATURE, "paths")?
            }
            "confirm" => confirm = parse_bool_option(value, CLEAN_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    CLEAN_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok((parsed, confirm))
}

fn strings_to_lua(lua: &Lua, values: Vec<String>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, value) in values.into_iter().enumerate() {
        table.set(index + 1, value)?;
    }
    Ok(table)
}
