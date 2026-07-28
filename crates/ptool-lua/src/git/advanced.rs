use super::{ConfirmableOptions, LuaGitRepo, confirm_git_action};
use mlua::{Lua, Table, Value, Variadic};
use ptool_console::GitAction;
use ptool_engine::{
    GitBlameHunk, GitBlameOptions, GitSubmoduleInfo, GitSubmoduleInitOptions,
    GitSubmoduleSyncOptions, GitSubmoduleUpdateOptions, GitWorktreeAddOptions, GitWorktreeInfo,
    GitWorktreePruneOptions,
};

pub(super) const WORKTREES_SIGNATURE: &str = "ptool.git.Repo:worktrees()";
pub(super) const WORKTREE_ADD_SIGNATURE: &str = "ptool.git.Repo:worktree_add(name, path, options?)";
pub(super) const WORKTREE_LOCK_SIGNATURE: &str =
    "ptool.git.Repo:worktree_lock(name, reason?, options?)";
pub(super) const WORKTREE_UNLOCK_SIGNATURE: &str = "ptool.git.Repo:worktree_unlock(name, options?)";
pub(super) const WORKTREE_PRUNE_SIGNATURE: &str = "ptool.git.Repo:worktree_prune(name, options?)";
pub(super) const SUBMODULES_SIGNATURE: &str = "ptool.git.Repo:submodules()";
pub(super) const SUBMODULE_INIT_SIGNATURE: &str = "ptool.git.Repo:submodule_init(name?, options?)";
pub(super) const SUBMODULE_UPDATE_SIGNATURE: &str =
    "ptool.git.Repo:submodule_update(name?, options?)";
pub(super) const SUBMODULE_SYNC_SIGNATURE: &str = "ptool.git.Repo:submodule_sync(name?, options?)";
pub(super) const BLAME_SIGNATURE: &str = "ptool.git.Repo:blame(path, options?)";

pub(super) fn worktrees(repo: &LuaGitRepo, lua: &Lua) -> mlua::Result<Table> {
    let values = repo
        .repo
        .worktrees()
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, WORKTREES_SIGNATURE))?;
    worktrees_to_lua(lua, values)
}

pub(super) fn worktree_add(
    repo: &LuaGitRepo,
    lua: &Lua,
    name: String,
    path: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let options = parse_worktree_add_options(options)?;
    confirm_advanced(
        repo,
        WORKTREE_ADD_SIGNATURE,
        "Add worktree",
        &name,
        options.confirm,
    )?;
    let path = super::resolve_repo_path(&repo.current_dir, &path);
    let value = repo
        .repo
        .worktree_add(&name, &path, options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, WORKTREE_ADD_SIGNATURE))?;
    worktree_to_lua(lua, value)
}

pub(super) fn worktree_lock(repo: &LuaGitRepo, args: Variadic<Value>) -> mlua::Result<()> {
    let (name, reason, confirm) = parse_worktree_lock_call(args)?;
    confirm_advanced(
        repo,
        WORKTREE_LOCK_SIGNATURE,
        "Lock worktree",
        &name,
        confirm,
    )?;
    repo.repo
        .worktree_lock(&name, reason.as_deref())
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, WORKTREE_LOCK_SIGNATURE))
}

pub(super) fn worktree_unlock(
    repo: &LuaGitRepo,
    name: String,
    options: Option<Table>,
) -> mlua::Result<()> {
    let confirm = super::integrate::parse_confirm_only(options, WORKTREE_UNLOCK_SIGNATURE)?;
    confirm_advanced(
        repo,
        WORKTREE_UNLOCK_SIGNATURE,
        "Unlock worktree",
        &name,
        confirm,
    )?;
    repo.repo
        .worktree_unlock(&name)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, WORKTREE_UNLOCK_SIGNATURE))
}

pub(super) fn worktree_prune(
    repo: &LuaGitRepo,
    name: String,
    options: Option<Table>,
) -> mlua::Result<()> {
    let options = parse_worktree_prune_options(options)?;
    confirm_advanced(
        repo,
        WORKTREE_PRUNE_SIGNATURE,
        "Prune worktree",
        &name,
        options.confirm,
    )?;
    repo.repo
        .worktree_prune(&name, options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, WORKTREE_PRUNE_SIGNATURE))
}

pub(super) fn submodules(repo: &LuaGitRepo, lua: &Lua) -> mlua::Result<Table> {
    let values = repo
        .repo
        .submodules()
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, SUBMODULES_SIGNATURE))?;
    submodules_to_lua(lua, values)
}

pub(super) fn submodule_init(repo: &LuaGitRepo, args: Variadic<Value>) -> mlua::Result<()> {
    let (name, options) = parse_named_options(args, SUBMODULE_INIT_SIGNATURE, |table| {
        parse_submodule_init_options(table)
    })?;
    confirm_advanced(
        repo,
        SUBMODULE_INIT_SIGNATURE,
        "Initialize submodule",
        name.as_deref().unwrap_or("all"),
        options.confirm,
    )?;
    repo.repo
        .submodule_init(name.as_deref(), options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, SUBMODULE_INIT_SIGNATURE))
}

pub(super) fn submodule_update(repo: &LuaGitRepo, args: Variadic<Value>) -> mlua::Result<()> {
    let (name, options) = parse_named_options(args, SUBMODULE_UPDATE_SIGNATURE, |table| {
        parse_submodule_update_options(table, repo)
    })?;
    confirm_advanced(
        repo,
        SUBMODULE_UPDATE_SIGNATURE,
        "Update submodule",
        name.as_deref().unwrap_or("all"),
        options.confirm,
    )?;
    repo.repo
        .submodule_update(name.as_deref(), options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, SUBMODULE_UPDATE_SIGNATURE))
}

pub(super) fn submodule_sync(repo: &LuaGitRepo, args: Variadic<Value>) -> mlua::Result<()> {
    let (name, options) = parse_named_options(args, SUBMODULE_SYNC_SIGNATURE, |table| {
        parse_submodule_sync_options(table)
    })?;
    confirm_advanced(
        repo,
        SUBMODULE_SYNC_SIGNATURE,
        "Sync submodule",
        name.as_deref().unwrap_or("all"),
        options.confirm,
    )?;
    repo.repo
        .submodule_sync(name.as_deref(), options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, SUBMODULE_SYNC_SIGNATURE))
}

pub(super) fn blame(
    repo: &LuaGitRepo,
    lua: &Lua,
    path: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let options = parse_blame_options(options)?;
    let hunks = repo
        .repo
        .blame(&path, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, BLAME_SIGNATURE))?;
    blame_to_lua(lua, hunks)
}

fn confirm_advanced(
    repo: &LuaGitRepo,
    op: &'static str,
    operation: &str,
    target: &str,
    confirm: bool,
) -> mlua::Result<()> {
    if !confirm {
        return Ok(());
    }
    let repository = repo.repo_label();
    confirm_git_action(
        &repo.engine,
        op,
        GitAction::Advanced {
            repository: &repository,
            operation,
            target,
        },
    )
}

fn parse_worktree_add_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitWorktreeAddOptions>> {
    let mut parsed = GitWorktreeAddOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, WORKTREE_ADD_SIGNATURE)?;
        match key.as_str() {
            "reference" => {
                parsed.reference = Some(parse_string(value, WORKTREE_ADD_SIGNATURE, "reference")?)
            }
            "lock" => {
                parsed.lock =
                    super::options::parse_bool_option(value, WORKTREE_ADD_SIGNATURE, "lock")?
            }
            "checkout_existing" => {
                parsed.checkout_existing = super::options::parse_bool_option(
                    value,
                    WORKTREE_ADD_SIGNATURE,
                    "checkout_existing",
                )?
            }
            "confirm" => {
                confirm =
                    super::options::parse_bool_option(value, WORKTREE_ADD_SIGNATURE, "confirm")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    WORKTREE_ADD_SIGNATURE,
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

fn parse_worktree_lock_call(args: Variadic<Value>) -> mlua::Result<(String, Option<String>, bool)> {
    if args.is_empty() || args.len() > 3 {
        return Err(crate::lua_error::invalid_argument(
            WORKTREE_LOCK_SIGNATURE,
            "expects name, optional reason, and optional options",
        ));
    }
    let name = parse_string(args[0].clone(), WORKTREE_LOCK_SIGNATURE, "name")?;
    let mut reason = None;
    let mut confirm = false;
    match args.get(1) {
        Some(Value::String(value)) => reason = Some(value.to_str()?.to_string()),
        Some(Value::Table(options)) => {
            confirm = super::integrate::parse_confirm_only(
                Some(options.clone()),
                WORKTREE_LOCK_SIGNATURE,
            )?
        }
        Some(_) => {
            return Err(crate::lua_error::invalid_argument(
                WORKTREE_LOCK_SIGNATURE,
                "reason must be a string",
            ));
        }
        None => {}
    }
    if let Some(value) = args.get(2) {
        let Value::Table(options) = value else {
            return Err(crate::lua_error::invalid_argument(
                WORKTREE_LOCK_SIGNATURE,
                "options must be a table",
            ));
        };
        confirm =
            super::integrate::parse_confirm_only(Some(options.clone()), WORKTREE_LOCK_SIGNATURE)?;
    }
    Ok((name, reason, confirm))
}

fn parse_worktree_prune_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitWorktreePruneOptions>> {
    let mut parsed = GitWorktreePruneOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, WORKTREE_PRUNE_SIGNATURE)?;
        match key.as_str() {
            "valid" => {
                parsed.valid =
                    super::options::parse_bool_option(value, WORKTREE_PRUNE_SIGNATURE, "valid")?
            }
            "locked" => {
                parsed.locked =
                    super::options::parse_bool_option(value, WORKTREE_PRUNE_SIGNATURE, "locked")?
            }
            "working_tree" => {
                parsed.working_tree = super::options::parse_bool_option(
                    value,
                    WORKTREE_PRUNE_SIGNATURE,
                    "working_tree",
                )?
            }
            "force" => {
                parsed.force =
                    super::options::parse_bool_option(value, WORKTREE_PRUNE_SIGNATURE, "force")?
            }
            "confirm" => {
                confirm =
                    super::options::parse_bool_option(value, WORKTREE_PRUNE_SIGNATURE, "confirm")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    WORKTREE_PRUNE_SIGNATURE,
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

fn parse_named_options<T, F>(
    args: Variadic<Value>,
    op: &str,
    parser: F,
) -> mlua::Result<(Option<String>, ConfirmableOptions<T>)>
where
    F: Fn(Option<Table>) -> mlua::Result<ConfirmableOptions<T>>,
{
    match args.len() {
        0 => Ok((None, parser(None)?)),
        1 => match args.first() {
            Some(Value::String(value)) => Ok((Some(value.to_str()?.to_string()), parser(None)?)),
            Some(Value::Table(value)) => Ok((None, parser(Some(value.clone()))?)),
            _ => Err(crate::lua_error::invalid_argument(
                op,
                "expects a name string or options table",
            )),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::String(name)), Some(Value::Table(options))) => Ok((
                Some(name.to_str()?.to_string()),
                parser(Some(options.clone()))?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                op,
                "expects (name, options)",
            )),
        },
        _ => Err(crate::lua_error::invalid_argument(
            op,
            "accepts at most 2 arguments",
        )),
    }
}

fn parse_submodule_init_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitSubmoduleInitOptions>> {
    let mut parsed = GitSubmoduleInitOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, SUBMODULE_INIT_SIGNATURE)?;
        match key.as_str() {
            "overwrite" => {
                parsed.overwrite =
                    super::options::parse_bool_option(value, SUBMODULE_INIT_SIGNATURE, "overwrite")?
            }
            "recursive" => {
                parsed.recursive =
                    super::options::parse_bool_option(value, SUBMODULE_INIT_SIGNATURE, "recursive")?
            }
            "confirm" => {
                confirm =
                    super::options::parse_bool_option(value, SUBMODULE_INIT_SIGNATURE, "confirm")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    SUBMODULE_INIT_SIGNATURE,
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

fn parse_submodule_update_options(
    options: Option<Table>,
    repo: &LuaGitRepo,
) -> mlua::Result<ConfirmableOptions<GitSubmoduleUpdateOptions>> {
    let mut parsed = GitSubmoduleUpdateOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, SUBMODULE_UPDATE_SIGNATURE)?;
        match key.as_str() {
            "init" => {
                parsed.init =
                    super::options::parse_bool_option(value, SUBMODULE_UPDATE_SIGNATURE, "init")?
            }
            "recursive" => {
                parsed.recursive = super::options::parse_bool_option(
                    value,
                    SUBMODULE_UPDATE_SIGNATURE,
                    "recursive",
                )?
            }
            "allow_fetch" => {
                parsed.allow_fetch = super::options::parse_bool_option(
                    value,
                    SUBMODULE_UPDATE_SIGNATURE,
                    "allow_fetch",
                )?
            }
            "auth" => match value {
                Value::Table(value) => {
                    parsed.auth = super::options::parse_auth_options(
                        value,
                        SUBMODULE_UPDATE_SIGNATURE,
                        &repo.current_dir,
                    )?
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        SUBMODULE_UPDATE_SIGNATURE,
                        "`auth` must be a table",
                    ));
                }
            },
            "confirm" => {
                confirm =
                    super::options::parse_bool_option(value, SUBMODULE_UPDATE_SIGNATURE, "confirm")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    SUBMODULE_UPDATE_SIGNATURE,
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

fn parse_submodule_sync_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitSubmoduleSyncOptions>> {
    let mut parsed = GitSubmoduleSyncOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, SUBMODULE_SYNC_SIGNATURE)?;
        match key.as_str() {
            "recursive" => {
                parsed.recursive =
                    super::options::parse_bool_option(value, SUBMODULE_SYNC_SIGNATURE, "recursive")?
            }
            "confirm" => {
                confirm =
                    super::options::parse_bool_option(value, SUBMODULE_SYNC_SIGNATURE, "confirm")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    SUBMODULE_SYNC_SIGNATURE,
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

fn parse_blame_options(options: Option<Table>) -> mlua::Result<GitBlameOptions> {
    let mut parsed = GitBlameOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = super::options::parse_option_key(key, BLAME_SIGNATURE)?;
        match key.as_str() {
            "newest" => parsed.newest = Some(parse_string(value, BLAME_SIGNATURE, "newest")?),
            "oldest" => parsed.oldest = Some(parse_string(value, BLAME_SIGNATURE, "oldest")?),
            "min_line" => {
                parsed.min_line = Some(parse_positive_usize(value, BLAME_SIGNATURE, "min_line")?)
            }
            "max_line" => {
                parsed.max_line = Some(parse_positive_usize(value, BLAME_SIGNATURE, "max_line")?)
            }
            "first_parent" => {
                parsed.first_parent =
                    super::options::parse_bool_option(value, BLAME_SIGNATURE, "first_parent")?
            }
            "track_copies_same_file" => {
                parsed.track_copies_same_file = super::options::parse_bool_option(
                    value,
                    BLAME_SIGNATURE,
                    "track_copies_same_file",
                )?
            }
            "track_copies_same_commit_moves" => {
                parsed.track_copies_same_commit_moves = super::options::parse_bool_option(
                    value,
                    BLAME_SIGNATURE,
                    "track_copies_same_commit_moves",
                )?
            }
            "track_copies_same_commit_copies" => {
                parsed.track_copies_same_commit_copies = super::options::parse_bool_option(
                    value,
                    BLAME_SIGNATURE,
                    "track_copies_same_commit_copies",
                )?
            }
            "track_copies_any_commit_copies" => {
                parsed.track_copies_any_commit_copies = super::options::parse_bool_option(
                    value,
                    BLAME_SIGNATURE,
                    "track_copies_any_commit_copies",
                )?
            }
            "ignore_whitespace" => {
                parsed.ignore_whitespace =
                    super::options::parse_bool_option(value, BLAME_SIGNATURE, "ignore_whitespace")?
            }
            "use_mailmap" => {
                parsed.use_mailmap =
                    super::options::parse_bool_option(value, BLAME_SIGNATURE, "use_mailmap")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    BLAME_SIGNATURE,
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

fn parse_positive_usize(value: Value, op: &str, field: &str) -> mlua::Result<usize> {
    match value {
        Value::Integer(value) if value > 0 => usize::try_from(value)
            .map_err(|_| crate::lua_error::invalid_option(op, format!("`{field}` is too large"))),
        _ => Err(crate::lua_error::invalid_option(
            op,
            format!("`{field}` must be a positive integer"),
        )),
    }
}

fn worktrees_to_lua(lua: &Lua, values: Vec<GitWorktreeInfo>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, value) in values.into_iter().enumerate() {
        table.set(index + 1, worktree_to_lua(lua, value)?)?;
    }
    Ok(table)
}

fn worktree_to_lua(lua: &Lua, value: GitWorktreeInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("name", value.name)?;
    table.set("path", value.path)?;
    table.set("locked", value.locked)?;
    table.set("lock_reason", value.lock_reason)?;
    table.set("valid", value.valid)?;
    Ok(table)
}

fn submodules_to_lua(lua: &Lua, values: Vec<GitSubmoduleInfo>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, value) in values.into_iter().enumerate() {
        let item = lua.create_table()?;
        item.set("name", value.name)?;
        item.set("path", value.path)?;
        item.set("url", value.url)?;
        item.set("branch", value.branch)?;
        item.set("head_oid", value.head_oid)?;
        item.set("index_oid", value.index_oid)?;
        item.set("workdir_oid", value.workdir_oid)?;
        table.set(index + 1, item)?;
    }
    Ok(table)
}

fn blame_to_lua(lua: &Lua, values: Vec<GitBlameHunk>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, value) in values.into_iter().enumerate() {
        let item = lua.create_table()?;
        item.set("final_start_line", usize_to_lua(value.final_start_line)?)?;
        item.set(
            "original_start_line",
            usize_to_lua(value.original_start_line)?,
        )?;
        item.set("line_count", usize_to_lua(value.line_count)?)?;
        item.set("commit_oid", value.commit_oid)?;
        let author = lua.create_table()?;
        author.set("name", value.author.name)?;
        author.set("email", value.author.email)?;
        author.set("time_seconds", value.author.time_seconds)?;
        author.set("offset_minutes", value.author.offset_minutes)?;
        item.set("author", author)?;
        item.set("origin_path", value.origin_path)?;
        item.set("boundary", value.boundary)?;
        table.set(index + 1, item)?;
    }
    Ok(table)
}

fn usize_to_lua(value: usize) -> mlua::Result<i64> {
    i64::try_from(value).map_err(|_| {
        crate::lua_error::invalid_argument(BLAME_SIGNATURE, "numeric value is too large")
    })
}
