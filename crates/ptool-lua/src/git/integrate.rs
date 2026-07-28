use super::{ConfirmableOptions, LuaGitRepo, confirm_git_action};
use mlua::{Lua, Table, Value, Variadic};
use ptool_console::GitAction;
use ptool_engine::{GitFastForwardMode, GitMergeOptions, GitPullOptions, GitPullStrategy};

pub(super) const CONFLICTS_SIGNATURE: &str = "ptool.git.Repo:conflicts()";
pub(super) const MERGE_ANALYSIS_SIGNATURE: &str = "ptool.git.Repo:merge_analysis(rev)";
pub(super) const MERGE_SIGNATURE: &str = "ptool.git.Repo:merge(rev, options?)";
pub(super) const MERGE_ABORT_SIGNATURE: &str = "ptool.git.Repo:merge_abort(options?)";
pub(super) const PULL_SIGNATURE: &str = "ptool.git.Repo:pull(remote?, branch?, options?)";

pub(super) fn state(repo: &LuaGitRepo) -> String {
    repo.repo.state()
}

pub(super) fn conflicts(repo: &LuaGitRepo, lua: &Lua) -> mlua::Result<Table> {
    let conflicts = repo
        .repo
        .conflicts()
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, CONFLICTS_SIGNATURE))?;
    super::render::git_conflicts_to_lua(lua, conflicts)
}

pub(super) fn merge_analysis(repo: &LuaGitRepo, rev: String) -> mlua::Result<String> {
    repo.repo
        .merge_analysis(&rev)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, MERGE_ANALYSIS_SIGNATURE))
}

pub(super) fn merge(
    repo: &LuaGitRepo,
    lua: &Lua,
    rev: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let options = parse_merge_options(options)?;
    if options.confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            MERGE_SIGNATURE,
            GitAction::Integrate {
                repository: &repository,
                operation: "Merge",
                revision: &rev,
            },
        )?;
    }
    let result = repo
        .repo
        .merge(&rev, options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, MERGE_SIGNATURE))?;
    super::render::git_integrate_result_to_lua(lua, result)
}

pub(super) fn merge_abort(repo: &LuaGitRepo, options: Option<Table>) -> mlua::Result<()> {
    let confirm = parse_confirm_only(options, MERGE_ABORT_SIGNATURE)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            MERGE_ABORT_SIGNATURE,
            GitAction::Integrate {
                repository: &repository,
                operation: "Abort merge",
                revision: "ORIG_HEAD",
            },
        )?;
    }
    repo.repo
        .merge_abort()
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, MERGE_ABORT_SIGNATURE))
}

pub(super) fn pull(repo: &LuaGitRepo, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
    let (remote, branch, options) = parse_pull_call(args, repo)?;
    if options.confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            PULL_SIGNATURE,
            GitAction::Pull {
                repository: &repository,
                remote: remote.as_deref().unwrap_or("origin"),
                branch: branch.as_deref().unwrap_or("current branch"),
            },
        )?;
    }
    let result = repo
        .repo
        .pull(remote.as_deref(), branch.as_deref(), options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, PULL_SIGNATURE))?;
    super::render::git_integrate_result_to_lua(lua, result)
}

fn parse_merge_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitMergeOptions>> {
    let mut parsed = GitMergeOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, MERGE_SIGNATURE)?;
        match key.as_str() {
            "ff" => parsed.ff = parse_ff(value, MERGE_SIGNATURE)?,
            "signature" => match value {
                Value::Table(value) => {
                    parsed.signature =
                        Some(super::options::parse_signature(value, MERGE_SIGNATURE)?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        MERGE_SIGNATURE,
                        "`signature` must be a table",
                    ));
                }
            },
            "message" => match value {
                Value::String(value) => parsed.message = Some(value.to_str()?.to_string()),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        MERGE_SIGNATURE,
                        "`message` must be a string",
                    ));
                }
            },
            "confirm" => {
                confirm = super::options::parse_bool_option(value, MERGE_SIGNATURE, "confirm")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    MERGE_SIGNATURE,
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

fn parse_pull_call(
    args: Variadic<Value>,
    repo: &LuaGitRepo,
) -> mlua::Result<(
    Option<String>,
    Option<String>,
    ConfirmableOptions<GitPullOptions>,
)> {
    let mut values = args.into_iter().collect::<Vec<_>>();
    let options = if matches!(values.last(), Some(Value::Table(_))) {
        let Value::Table(table) = values.pop().expect("table checked") else {
            unreachable!()
        };
        parse_pull_options(Some(table), repo)?
    } else {
        parse_pull_options(None, repo)?
    };
    if values.len() > 2 {
        return Err(crate::lua_error::invalid_argument(
            PULL_SIGNATURE,
            "accepts at most remote, branch, and options",
        ));
    }
    let remote = values
        .first()
        .map(|value| parse_string(value, PULL_SIGNATURE, "remote"))
        .transpose()?;
    let branch = values
        .get(1)
        .map(|value| parse_string(value, PULL_SIGNATURE, "branch"))
        .transpose()?;
    Ok((remote, branch, options))
}

fn parse_pull_options(
    options: Option<Table>,
    repo: &LuaGitRepo,
) -> mlua::Result<ConfirmableOptions<GitPullOptions>> {
    let mut parsed = GitPullOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, PULL_SIGNATURE)?;
        match key.as_str() {
            "auth" => match value {
                Value::Table(value) => {
                    parsed.auth = super::options::parse_auth_options(
                        value,
                        PULL_SIGNATURE,
                        &repo.current_dir,
                    )?
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        PULL_SIGNATURE,
                        "`auth` must be a table",
                    ));
                }
            },
            "depth" => match value {
                Value::Integer(value) if value > 0 && value <= i64::from(i32::MAX) => {
                    parsed.depth = Some(value as i32)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        PULL_SIGNATURE,
                        "`depth` must be a positive 32-bit integer",
                    ));
                }
            },
            "prune" => {
                parsed.prune = super::options::parse_bool_option(value, PULL_SIGNATURE, "prune")?
            }
            "tags" => parsed.tags = super::options::parse_tag_download(value, PULL_SIGNATURE)?,
            "update_fetchhead" => {
                parsed.update_fetchhead =
                    super::options::parse_bool_option(value, PULL_SIGNATURE, "update_fetchhead")?
            }
            "strategy" => parsed.strategy = parse_pull_strategy(value)?,
            "signature" => match value {
                Value::Table(value) => {
                    parsed.signature = Some(super::options::parse_signature(value, PULL_SIGNATURE)?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        PULL_SIGNATURE,
                        "`signature` must be a table",
                    ));
                }
            },
            "message" => match value {
                Value::String(value) => parsed.message = Some(value.to_str()?.to_string()),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        PULL_SIGNATURE,
                        "`message` must be a string",
                    ));
                }
            },
            "confirm" => {
                confirm = super::options::parse_bool_option(value, PULL_SIGNATURE, "confirm")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    PULL_SIGNATURE,
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

fn parse_ff(value: Value, op: &str) -> mlua::Result<GitFastForwardMode> {
    let value = parse_string(&value, op, "ff")?;
    match value.as_str() {
        "allow" => Ok(GitFastForwardMode::Allow),
        "only" => Ok(GitFastForwardMode::Only),
        "never" => Ok(GitFastForwardMode::Never),
        _ => Err(crate::lua_error::invalid_option(
            op,
            "`ff` must be `allow`, `only`, or `never`",
        )),
    }
}

fn parse_pull_strategy(value: Value) -> mlua::Result<GitPullStrategy> {
    let value = parse_string(&value, PULL_SIGNATURE, "strategy")?;
    match value.as_str() {
        "ff_only" => Ok(GitPullStrategy::FastForwardOnly),
        "merge" => Ok(GitPullStrategy::Merge),
        "rebase" => Ok(GitPullStrategy::Rebase),
        _ => Err(crate::lua_error::invalid_option(
            PULL_SIGNATURE,
            "`strategy` must be `ff_only`, `merge`, or `rebase`",
        )),
    }
}

pub(super) fn parse_confirm_only(options: Option<Table>, op: &str) -> mlua::Result<bool> {
    let Some(options) = options else {
        return Ok(false);
    };
    let mut confirm = false;
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, op)?;
        match key.as_str() {
            "confirm" => confirm = super::options::parse_bool_option(value, op, "confirm")?,
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

fn parse_string(value: &Value, op: &str, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_argument(
            op,
            format!("`{field}` must be a string"),
        )),
    }
}

pub(super) const CHERRY_PICK_SIGNATURE: &str = "ptool.git.Repo:cherry_pick(rev, options?)";
pub(super) const CHERRY_PICK_ABORT_SIGNATURE: &str = "ptool.git.Repo:cherry_pick_abort(options?)";
pub(super) const REVERT_SIGNATURE: &str = "ptool.git.Repo:revert(rev, options?)";
pub(super) const REVERT_ABORT_SIGNATURE: &str = "ptool.git.Repo:revert_abort(options?)";

pub(super) fn cherry_pick(
    repo: &LuaGitRepo,
    lua: &Lua,
    rev: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    apply_commit(repo, lua, rev, options, true)
}

pub(super) fn revert(
    repo: &LuaGitRepo,
    lua: &Lua,
    rev: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    apply_commit(repo, lua, rev, options, false)
}

pub(super) fn cherry_pick_abort(repo: &LuaGitRepo, options: Option<Table>) -> mlua::Result<()> {
    apply_abort(repo, options, true)
}

pub(super) fn revert_abort(repo: &LuaGitRepo, options: Option<Table>) -> mlua::Result<()> {
    apply_abort(repo, options, false)
}

fn apply_commit(
    repo: &LuaGitRepo,
    lua: &Lua,
    rev: String,
    options: Option<Table>,
    cherry_pick: bool,
) -> mlua::Result<Table> {
    let op = if cherry_pick {
        CHERRY_PICK_SIGNATURE
    } else {
        REVERT_SIGNATURE
    };
    let options = parse_apply_commit_options(options, op)?;
    if options.confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            op,
            GitAction::Integrate {
                repository: &repository,
                operation: if cherry_pick { "Cherry-pick" } else { "Revert" },
                revision: &rev,
            },
        )?;
    }
    let result = if cherry_pick {
        repo.repo.cherry_pick(&rev, options.inner)
    } else {
        repo.repo.revert(&rev, options.inner)
    }
    .map_err(|err| crate::lua_error::lua_error_from_engine(err, op))?;
    super::render::git_integrate_result_to_lua(lua, result)
}

fn apply_abort(repo: &LuaGitRepo, options: Option<Table>, cherry_pick: bool) -> mlua::Result<()> {
    let op = if cherry_pick {
        CHERRY_PICK_ABORT_SIGNATURE
    } else {
        REVERT_ABORT_SIGNATURE
    };
    let confirm = parse_confirm_only(options, op)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            op,
            GitAction::Integrate {
                repository: &repository,
                operation: if cherry_pick {
                    "Abort cherry-pick"
                } else {
                    "Abort revert"
                },
                revision: "ORIG_HEAD",
            },
        )?;
    }
    let result = if cherry_pick {
        repo.repo.cherry_pick_abort()
    } else {
        repo.repo.revert_abort()
    };
    result.map_err(|err| crate::lua_error::lua_error_from_engine(err, op))
}

fn parse_apply_commit_options(
    options: Option<Table>,
    op: &str,
) -> mlua::Result<ConfirmableOptions<ptool_engine::GitApplyCommitOptions>> {
    let mut parsed = ptool_engine::GitApplyCommitOptions::default();
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
            "commit" => parsed.commit = super::options::parse_bool_option(value, op, "commit")?,
            "signature" => match value {
                Value::Table(value) => {
                    parsed.signature = Some(super::options::parse_signature(value, op)?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        op,
                        "`signature` must be a table",
                    ));
                }
            },
            "message" => match value {
                Value::String(value) => parsed.message = Some(value.to_str()?.to_string()),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        op,
                        "`message` must be a string",
                    ));
                }
            },
            "mainline" => match value {
                Value::Integer(value) if value > 0 && value <= i64::from(u32::MAX) => {
                    parsed.mainline = Some(value as u32)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        op,
                        "`mainline` must be a positive 32-bit integer",
                    ));
                }
            },
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
