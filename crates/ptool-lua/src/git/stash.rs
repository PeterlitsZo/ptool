use super::{ConfirmableOptions, LuaGitRepo, confirm_git_action};
use mlua::{Lua, Table, Value, Variadic};
use ptool_console::GitAction;
use ptool_engine::{GitStashApplyOptions, GitStashSaveOptions};

pub(super) const STASH_SAVE_SIGNATURE: &str = "ptool.git.Repo:stash_save(message?, options?)";
pub(super) const STASHES_SIGNATURE: &str = "ptool.git.Repo:stashes()";
pub(super) const STASH_APPLY_SIGNATURE: &str = "ptool.git.Repo:stash_apply(index?, options?)";
pub(super) const STASH_POP_SIGNATURE: &str = "ptool.git.Repo:stash_pop(index?, options?)";
pub(super) const STASH_DROP_SIGNATURE: &str = "ptool.git.Repo:stash_drop(index?, options?)";

pub(super) fn save(repo: &mut LuaGitRepo, args: Variadic<Value>) -> mlua::Result<String> {
    let (message, options) = parse_save_call(args)?;
    if options.confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            STASH_SAVE_SIGNATURE,
            GitAction::Stash {
                repository: &repository,
                operation: "Save",
            },
        )?;
    }
    repo.repo
        .stash_save(message.as_deref(), options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, STASH_SAVE_SIGNATURE))
}

pub(super) fn list(repo: &mut LuaGitRepo, lua: &Lua) -> mlua::Result<Table> {
    let stashes = repo
        .repo
        .stashes()
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, STASHES_SIGNATURE))?;
    super::render::git_stashes_to_lua(lua, stashes)
}

pub(super) fn apply(
    repo: &mut LuaGitRepo,
    lua: &Lua,
    args: Variadic<Value>,
    pop: bool,
) -> mlua::Result<Table> {
    let op = if pop {
        STASH_POP_SIGNATURE
    } else {
        STASH_APPLY_SIGNATURE
    };
    let (index, options) = parse_apply_call(args, op)?;
    if options.confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            op,
            GitAction::Stash {
                repository: &repository,
                operation: if pop { "Pop" } else { "Apply" },
            },
        )?;
    }
    let result = if pop {
        repo.repo.stash_pop(index, options.inner)
    } else {
        repo.repo.stash_apply(index, options.inner)
    }
    .map_err(|err| crate::lua_error::lua_error_from_engine(err, op))?;
    super::render::git_integrate_result_to_lua(lua, result)
}

pub(super) fn drop(repo: &mut LuaGitRepo, args: Variadic<Value>) -> mlua::Result<()> {
    let (index, confirm) = parse_drop_call(args)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            STASH_DROP_SIGNATURE,
            GitAction::Stash {
                repository: &repository,
                operation: "Drop",
            },
        )?;
    }
    repo.repo
        .stash_drop(index)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, STASH_DROP_SIGNATURE))
}

fn parse_save_call(
    args: Variadic<Value>,
) -> mlua::Result<(Option<String>, ConfirmableOptions<GitStashSaveOptions>)> {
    match args.len() {
        0 => Ok((None, parse_save_options(None)?)),
        1 => match args.first() {
            Some(Value::String(value)) => {
                Ok((Some(value.to_str()?.to_string()), parse_save_options(None)?))
            }
            Some(Value::Table(value)) => Ok((None, parse_save_options(Some(value.clone()))?)),
            _ => Err(crate::lua_error::invalid_argument(
                STASH_SAVE_SIGNATURE,
                "expects a message string or options table",
            )),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::String(message)), Some(Value::Table(options))) => Ok((
                Some(message.to_str()?.to_string()),
                parse_save_options(Some(options.clone()))?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                STASH_SAVE_SIGNATURE,
                "expects (message, options)",
            )),
        },
        _ => Err(crate::lua_error::invalid_argument(
            STASH_SAVE_SIGNATURE,
            "accepts at most 2 arguments",
        )),
    }
}

fn parse_save_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitStashSaveOptions>> {
    let mut parsed = GitStashSaveOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, STASH_SAVE_SIGNATURE)?;
        match key.as_str() {
            "include_untracked" => {
                parsed.include_untracked = super::options::parse_bool_option(
                    value,
                    STASH_SAVE_SIGNATURE,
                    "include_untracked",
                )?
            }
            "include_ignored" => {
                parsed.include_ignored = super::options::parse_bool_option(
                    value,
                    STASH_SAVE_SIGNATURE,
                    "include_ignored",
                )?
            }
            "keep_index" => {
                parsed.keep_index =
                    super::options::parse_bool_option(value, STASH_SAVE_SIGNATURE, "keep_index")?
            }
            "signature" => match value {
                Value::Table(value) => {
                    parsed.signature = Some(super::options::parse_signature(
                        value,
                        STASH_SAVE_SIGNATURE,
                    )?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        STASH_SAVE_SIGNATURE,
                        "`signature` must be a table",
                    ));
                }
            },
            "confirm" => {
                confirm = super::options::parse_bool_option(value, STASH_SAVE_SIGNATURE, "confirm")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    STASH_SAVE_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(ConfirmableOptions {
        inner: parsed,
        confirm,
    })
}

fn parse_apply_call(
    args: Variadic<Value>,
    op: &str,
) -> mlua::Result<(usize, ConfirmableOptions<GitStashApplyOptions>)> {
    match args.len() {
        0 => Ok((0, parse_apply_options(None, op)?)),
        1 => match args.first() {
            Some(Value::Integer(index)) => {
                Ok((parse_index(*index, op)?, parse_apply_options(None, op)?))
            }
            Some(Value::Table(options)) => Ok((0, parse_apply_options(Some(options.clone()), op)?)),
            _ => Err(crate::lua_error::invalid_argument(
                op,
                "expects an index or options table",
            )),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::Integer(index)), Some(Value::Table(options))) => Ok((
                parse_index(*index, op)?,
                parse_apply_options(Some(options.clone()), op)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                op,
                "expects (index, options)",
            )),
        },
        _ => Err(crate::lua_error::invalid_argument(
            op,
            "accepts at most 2 arguments",
        )),
    }
}

fn parse_apply_options(
    options: Option<Table>,
    op: &str,
) -> mlua::Result<ConfirmableOptions<GitStashApplyOptions>> {
    let mut parsed = GitStashApplyOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, op)?;
        match key.as_str() {
            "reinstate_index" => {
                parsed.reinstate_index =
                    super::options::parse_bool_option(value, op, "reinstate_index")?
            }
            "confirm" => confirm = super::options::parse_bool_option(value, op, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(ConfirmableOptions {
        inner: parsed,
        confirm,
    })
}

fn parse_drop_call(args: Variadic<Value>) -> mlua::Result<(usize, bool)> {
    match args.len() {
        0 => Ok((0, false)),
        1 => match args.first() {
            Some(Value::Integer(index)) => Ok((parse_index(*index, STASH_DROP_SIGNATURE)?, false)),
            Some(Value::Table(options)) => Ok((
                0,
                super::integrate::parse_confirm_only(Some(options.clone()), STASH_DROP_SIGNATURE)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                STASH_DROP_SIGNATURE,
                "expects an index or options table",
            )),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::Integer(index)), Some(Value::Table(options))) => Ok((
                parse_index(*index, STASH_DROP_SIGNATURE)?,
                super::integrate::parse_confirm_only(Some(options.clone()), STASH_DROP_SIGNATURE)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                STASH_DROP_SIGNATURE,
                "expects (index, options)",
            )),
        },
        _ => Err(crate::lua_error::invalid_argument(
            STASH_DROP_SIGNATURE,
            "accepts at most 2 arguments",
        )),
    }
}

fn parse_index(index: i64, op: &str) -> mlua::Result<usize> {
    usize::try_from(index).map_err(|_| {
        crate::lua_error::invalid_argument(op, "stash index must be a non-negative integer")
    })
}
