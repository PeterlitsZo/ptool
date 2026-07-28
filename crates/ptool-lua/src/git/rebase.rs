use super::{ConfirmableOptions, LuaGitRepo, confirm_git_action};
use mlua::{Lua, Table, Value};
use ptool_console::GitAction;
use ptool_engine::{GitRebaseContinueOptions, GitRebaseOptions};

pub(super) const REBASE_SIGNATURE: &str = "ptool.git.Repo:rebase(options)";
pub(super) const REBASE_CONTINUE_SIGNATURE: &str = "ptool.git.Repo:rebase_continue(options?)";
pub(super) const REBASE_ABORT_SIGNATURE: &str = "ptool.git.Repo:rebase_abort(options?)";

pub(super) fn start(repo: &LuaGitRepo, lua: &Lua, options: Table) -> mlua::Result<Table> {
    let options = parse_start_options(options)?;
    if options.confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            REBASE_SIGNATURE,
            GitAction::Integrate {
                repository: &repository,
                operation: "Rebase",
                revision: &options.inner.upstream,
            },
        )?;
    }
    let result = repo
        .repo
        .rebase(options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REBASE_SIGNATURE))?;
    super::render::git_rebase_result_to_lua(lua, result)
}

pub(super) fn continue_rebase(
    repo: &LuaGitRepo,
    lua: &Lua,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let options = parse_continue_options(options)?;
    if options.confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            REBASE_CONTINUE_SIGNATURE,
            GitAction::Integrate {
                repository: &repository,
                operation: "Continue rebase",
                revision: "current operation",
            },
        )?;
    }
    let result = repo
        .repo
        .rebase_continue(options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REBASE_CONTINUE_SIGNATURE))?;
    super::render::git_rebase_result_to_lua(lua, result)
}

pub(super) fn abort(repo: &LuaGitRepo, options: Option<Table>) -> mlua::Result<()> {
    let confirm = super::integrate::parse_confirm_only(options, REBASE_ABORT_SIGNATURE)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            REBASE_ABORT_SIGNATURE,
            GitAction::Integrate {
                repository: &repository,
                operation: "Abort rebase",
                revision: "original HEAD",
            },
        )?;
    }
    repo.repo
        .rebase_abort()
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REBASE_ABORT_SIGNATURE))
}

fn parse_start_options(options: Table) -> mlua::Result<ConfirmableOptions<GitRebaseOptions>> {
    let mut upstream = None;
    let mut onto = None;
    let mut branch = "HEAD".to_string();
    let mut signature = None;
    let mut confirm = false;
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, REBASE_SIGNATURE)?;
        match key.as_str() {
            "upstream" => upstream = Some(parse_string(value, "upstream", REBASE_SIGNATURE)?),
            "onto" => onto = Some(parse_string(value, "onto", REBASE_SIGNATURE)?),
            "branch" => branch = parse_string(value, "branch", REBASE_SIGNATURE)?,
            "signature" => match value {
                Value::Table(value) => {
                    signature = Some(super::options::parse_signature(value, REBASE_SIGNATURE)?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        REBASE_SIGNATURE,
                        "`signature` must be a table",
                    ));
                }
            },
            "confirm" => {
                confirm = super::options::parse_bool_option(value, REBASE_SIGNATURE, "confirm")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    REBASE_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    let upstream = upstream.ok_or_else(|| {
        crate::lua_error::invalid_argument(REBASE_SIGNATURE, "`upstream` is required")
    })?;
    Ok(ConfirmableOptions {
        inner: GitRebaseOptions {
            upstream,
            onto,
            branch,
            signature,
        },
        confirm,
    })
}

fn parse_continue_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitRebaseContinueOptions>> {
    let mut parsed = GitRebaseContinueOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, REBASE_CONTINUE_SIGNATURE)?;
        match key.as_str() {
            "signature" => match value {
                Value::Table(value) => {
                    parsed.signature = Some(super::options::parse_signature(
                        value,
                        REBASE_CONTINUE_SIGNATURE,
                    )?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        REBASE_CONTINUE_SIGNATURE,
                        "`signature` must be a table",
                    ));
                }
            },
            "confirm" => {
                confirm =
                    super::options::parse_bool_option(value, REBASE_CONTINUE_SIGNATURE, "confirm")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    REBASE_CONTINUE_SIGNATURE,
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

fn parse_string(value: Value, field: &str, op: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_option(
            op,
            format!("`{field}` must be a string"),
        )),
    }
}
