use mlua::{Lua, Table, UserData, UserDataMethods, Value, Variadic};
use ptool_engine::{
    GitAddOptions, GitCheckoutOptions, GitCloneOptions, GitCommitOptions, GitFetchOptions,
    GitFetchStats, GitPushOptions, GitRemoteAuth, GitRepository, GitSignature, GitStatusEntry,
    GitStatusOptions, GitStatusSummary, GitSwitchOptions, PtoolEngine,
};
use std::path::Path;

const OPEN_SIGNATURE: &str = "ptool.git.open(path?)";
const DISCOVER_SIGNATURE: &str = "ptool.git.discover(path?)";
const CLONE_SIGNATURE: &str = "ptool.git.clone(url, path[, options])";
const HEAD_SIGNATURE: &str = "ptool.git.Repo:head()";
const CURRENT_BRANCH_SIGNATURE: &str = "ptool.git.Repo:current_branch()";
const STATUS_SIGNATURE: &str = "ptool.git.Repo:status(options?)";
const IS_CLEAN_SIGNATURE: &str = "ptool.git.Repo:is_clean(options?)";
const ADD_SIGNATURE: &str = "ptool.git.Repo:add(paths[, options])";
const COMMIT_SIGNATURE: &str = "ptool.git.Repo:commit(message[, options])";
const CHECKOUT_SIGNATURE: &str = "ptool.git.Repo:checkout(rev[, options])";
const SWITCH_SIGNATURE: &str = "ptool.git.Repo:switch(branch[, options])";
const FETCH_SIGNATURE: &str = "ptool.git.Repo:fetch(remote?, options?)";
const PUSH_SIGNATURE: &str = "ptool.git.Repo:push(remote?, refspecs?, options?)";

pub(crate) struct LuaGitRepo {
    repo: GitRepository,
}

pub(crate) fn open(
    path: Option<String>,
    current_dir: &Path,
    engine: &PtoolEngine,
) -> mlua::Result<LuaGitRepo> {
    let repo = engine
        .git_open(path.as_deref(), current_dir)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, OPEN_SIGNATURE))?;
    Ok(LuaGitRepo { repo })
}

pub(crate) fn discover(
    path: Option<String>,
    current_dir: &Path,
    engine: &PtoolEngine,
) -> mlua::Result<LuaGitRepo> {
    let repo = engine
        .git_discover(path.as_deref(), current_dir)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, DISCOVER_SIGNATURE))?;
    Ok(LuaGitRepo { repo })
}

pub(crate) fn clone_repo(
    url: String,
    path: String,
    options: Option<Table>,
    current_dir: &Path,
    engine: &PtoolEngine,
) -> mlua::Result<LuaGitRepo> {
    let options = parse_clone_options(options)?;
    let repo = engine
        .git_clone(&url, &path, current_dir, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, CLONE_SIGNATURE))?;
    Ok(LuaGitRepo { repo })
}

impl UserData for LuaGitRepo {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("path", |_, this, ()| Ok(this.repo.path()));
        methods.add_method("root", |_, this, ()| Ok(this.repo.root()));
        methods.add_method("is_bare", |_, this, ()| Ok(this.repo.is_bare()));
        methods.add_method("head", |lua, this, ()| this.head(lua));
        methods.add_method("current_branch", |_, this, ()| this.current_branch());
        methods.add_method("status", |lua, this, options: Option<Table>| {
            this.status(lua, options)
        });
        methods.add_method("is_clean", |_, this, options: Option<Table>| {
            this.is_clean(options)
        });
        methods.add_method(
            "add",
            |_, this, (paths, options): (Value, Option<Table>)| this.add(paths, options),
        );
        methods.add_method(
            "commit",
            |_, this, (message, options): (String, Option<Table>)| this.commit(message, options),
        );
        methods.add_method(
            "checkout",
            |_, this, (rev, options): (String, Option<Table>)| this.checkout(rev, options),
        );
        methods.add_method(
            "switch",
            |_, this, (branch, options): (String, Option<Table>)| this.switch(branch, options),
        );
        methods.add_method("fetch", |lua, this, args: Variadic<Value>| {
            this.fetch(lua, args)
        });
        methods.add_method("push", |_, this, args: Variadic<Value>| this.push(args));
    }
}

impl LuaGitRepo {
    fn head(&self, lua: &Lua) -> mlua::Result<Table> {
        let info = self
            .repo
            .head()
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, HEAD_SIGNATURE))?;
        git_head_to_lua(lua, info)
    }

    fn current_branch(&self) -> mlua::Result<Option<String>> {
        self.repo
            .current_branch()
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, CURRENT_BRANCH_SIGNATURE))
    }

    fn status(&self, lua: &Lua, options: Option<Table>) -> mlua::Result<Table> {
        let options = parse_status_options(options)?;
        let status = self
            .repo
            .status(options)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, STATUS_SIGNATURE))?;
        git_status_to_lua(lua, status)
    }

    fn is_clean(&self, options: Option<Table>) -> mlua::Result<bool> {
        let options = parse_status_options(options)?;
        self.repo
            .is_clean(options)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, IS_CLEAN_SIGNATURE))
    }

    fn add(&self, paths: Value, options: Option<Table>) -> mlua::Result<()> {
        let paths = parse_paths(paths, ADD_SIGNATURE)?;
        let options = parse_add_options(options)?;
        self.repo
            .add(&paths, options)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, ADD_SIGNATURE))
    }

    fn commit(&self, message: String, options: Option<Table>) -> mlua::Result<String> {
        let options = parse_commit_options(options)?;
        self.repo
            .commit(&message, options)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, COMMIT_SIGNATURE))
    }

    fn checkout(&self, rev: String, options: Option<Table>) -> mlua::Result<()> {
        let options = parse_checkout_options(options)?;
        self.repo
            .checkout(&rev, options)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, CHECKOUT_SIGNATURE))
    }

    fn switch(&self, branch: String, options: Option<Table>) -> mlua::Result<()> {
        let options = parse_switch_options(options)?;
        self.repo
            .switch(&branch, options)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, SWITCH_SIGNATURE))
    }

    fn fetch(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
        let (remote, options) = parse_fetch_call(args)?;
        let stats = self
            .repo
            .fetch(remote.as_deref(), options)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, FETCH_SIGNATURE))?;
        git_fetch_stats_to_lua(lua, stats)
    }

    fn push(&self, args: Variadic<Value>) -> mlua::Result<()> {
        let (remote, refspecs, options) = parse_push_call(args)?;
        self.repo
            .push(remote.as_deref(), &refspecs, options)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, PUSH_SIGNATURE))
    }
}

fn parse_clone_options(options: Option<Table>) -> mlua::Result<GitCloneOptions> {
    let mut parsed = GitCloneOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, CLONE_SIGNATURE)?;
        match key.as_str() {
            "branch" => match value {
                Value::String(value) => parsed.branch = Some(value.to_str()?.to_string()),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        CLONE_SIGNATURE,
                        "`branch` must be a string",
                    ));
                }
            },
            "bare" => match value {
                Value::Boolean(value) => parsed.bare = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        CLONE_SIGNATURE,
                        "`bare` must be a boolean",
                    ));
                }
            },
            "auth" => match value {
                Value::Table(value) => parsed.auth = parse_auth_options(value, CLONE_SIGNATURE)?,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        CLONE_SIGNATURE,
                        "`auth` must be a table",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    CLONE_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn parse_status_options(options: Option<Table>) -> mlua::Result<GitStatusOptions> {
    let mut parsed = GitStatusOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, STATUS_SIGNATURE)?;
        match key.as_str() {
            "include_untracked" => match value {
                Value::Boolean(value) => parsed.include_untracked = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        STATUS_SIGNATURE,
                        "`include_untracked` must be a boolean",
                    ));
                }
            },
            "include_ignored" => match value {
                Value::Boolean(value) => parsed.include_ignored = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        STATUS_SIGNATURE,
                        "`include_ignored` must be a boolean",
                    ));
                }
            },
            "recurse_untracked_dirs" => match value {
                Value::Boolean(value) => parsed.recurse_untracked_dirs = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        STATUS_SIGNATURE,
                        "`recurse_untracked_dirs` must be a boolean",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    STATUS_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn parse_add_options(options: Option<Table>) -> mlua::Result<GitAddOptions> {
    let mut parsed = GitAddOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, ADD_SIGNATURE)?;
        match key.as_str() {
            "update" => match value {
                Value::Boolean(value) => parsed.update = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        ADD_SIGNATURE,
                        "`update` must be a boolean",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    ADD_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn parse_commit_options(options: Option<Table>) -> mlua::Result<GitCommitOptions> {
    let mut parsed = GitCommitOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, COMMIT_SIGNATURE)?;
        match key.as_str() {
            "author" => match value {
                Value::Table(value) => {
                    parsed.author = Some(parse_signature(value, COMMIT_SIGNATURE)?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        COMMIT_SIGNATURE,
                        "`author` must be a table",
                    ));
                }
            },
            "committer" => match value {
                Value::Table(value) => {
                    parsed.committer = Some(parse_signature(value, COMMIT_SIGNATURE)?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        COMMIT_SIGNATURE,
                        "`committer` must be a table",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    COMMIT_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn parse_checkout_options(options: Option<Table>) -> mlua::Result<GitCheckoutOptions> {
    let mut parsed = GitCheckoutOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, CHECKOUT_SIGNATURE)?;
        match key.as_str() {
            "force" => match value {
                Value::Boolean(value) => parsed.force = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        CHECKOUT_SIGNATURE,
                        "`force` must be a boolean",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    CHECKOUT_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn parse_switch_options(options: Option<Table>) -> mlua::Result<GitSwitchOptions> {
    let mut parsed = GitSwitchOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, SWITCH_SIGNATURE)?;
        match key.as_str() {
            "create" => match value {
                Value::Boolean(value) => parsed.create = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        SWITCH_SIGNATURE,
                        "`create` must be a boolean",
                    ));
                }
            },
            "force" => match value {
                Value::Boolean(value) => parsed.force = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        SWITCH_SIGNATURE,
                        "`force` must be a boolean",
                    ));
                }
            },
            "start_point" => match value {
                Value::String(value) => parsed.start_point = Some(value.to_str()?.to_string()),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        SWITCH_SIGNATURE,
                        "`start_point` must be a string",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    SWITCH_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn parse_fetch_call(args: Variadic<Value>) -> mlua::Result<(Option<String>, GitFetchOptions)> {
    match args.len() {
        0 => Ok((None, GitFetchOptions::default())),
        1 => match args.first() {
            Some(Value::String(value)) => Ok((
                Some(value.to_str()?.to_string()),
                GitFetchOptions::default(),
            )),
            Some(Value::Table(value)) => Ok((
                None,
                parse_fetch_options(Some(value.clone()), FETCH_SIGNATURE)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                FETCH_SIGNATURE,
                "expects a string remote or an options table",
            )),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::String(remote)), Some(Value::Table(options))) => Ok((
                Some(remote.to_str()?.to_string()),
                parse_fetch_options(Some(options.clone()), FETCH_SIGNATURE)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                FETCH_SIGNATURE,
                "expects (remote, options)",
            )),
        },
        _ => Err(crate::lua_error::invalid_argument(
            FETCH_SIGNATURE,
            "accepts at most 2 arguments",
        )),
    }
}

fn parse_fetch_options(options: Option<Table>, op: &str) -> mlua::Result<GitFetchOptions> {
    let mut parsed = GitFetchOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, op)?;
        match key.as_str() {
            "refspecs" => parsed.refspecs = parse_string_list_from_value(value, op, "refspecs")?,
            "auth" => match value {
                Value::Table(value) => parsed.auth = parse_auth_options(value, op)?,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        op,
                        "`auth` must be a table",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn parse_push_call(
    args: Variadic<Value>,
) -> mlua::Result<(Option<String>, Vec<String>, GitPushOptions)> {
    match args.len() {
        0 => Ok((None, Vec::new(), GitPushOptions::default())),
        1 => match args.first() {
            Some(Value::String(value)) => Ok((
                Some(value.to_str()?.to_string()),
                Vec::new(),
                GitPushOptions::default(),
            )),
            Some(Value::Table(value)) => Ok((
                None,
                Vec::new(),
                parse_push_options(Some(value.clone()), PUSH_SIGNATURE)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                PUSH_SIGNATURE,
                "expects a string remote or an options table",
            )),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::String(remote)), Some(Value::Table(options))) => Ok((
                Some(remote.to_str()?.to_string()),
                Vec::new(),
                parse_push_options(Some(options.clone()), PUSH_SIGNATURE)?,
            )),
            (Some(Value::String(remote)), Some(value)) => Ok((
                Some(remote.to_str()?.to_string()),
                parse_string_list_from_value(value.clone(), PUSH_SIGNATURE, "refspecs")?,
                GitPushOptions::default(),
            )),
            _ => Err(crate::lua_error::invalid_argument(
                PUSH_SIGNATURE,
                "expects (remote, refspecs|options)",
            )),
        },
        3 => match (args.first(), args.get(1), args.get(2)) {
            (Some(Value::String(remote)), Some(value), Some(Value::Table(options))) => Ok((
                Some(remote.to_str()?.to_string()),
                parse_string_list_from_value(value.clone(), PUSH_SIGNATURE, "refspecs")?,
                parse_push_options(Some(options.clone()), PUSH_SIGNATURE)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                PUSH_SIGNATURE,
                "expects (remote, refspecs, options)",
            )),
        },
        _ => Err(crate::lua_error::invalid_argument(
            PUSH_SIGNATURE,
            "accepts at most 3 arguments",
        )),
    }
}

fn parse_push_options(options: Option<Table>, op: &str) -> mlua::Result<GitPushOptions> {
    let mut parsed = GitPushOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, op)?;
        match key.as_str() {
            "auth" => match value {
                Value::Table(value) => parsed.auth = parse_auth_options(value, op)?,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        op,
                        "`auth` must be a table",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn parse_auth_options(options: Table, op: &str) -> mlua::Result<GitRemoteAuth> {
    let Some(kind) = options.get::<Option<String>>("kind")? else {
        return Err(crate::lua_error::invalid_argument(
            op,
            "`auth.kind` is required",
        ));
    };

    match kind.as_str() {
        "default" => Ok(GitRemoteAuth::Default),
        "ssh_agent" => Ok(GitRemoteAuth::SshAgent {
            username: options.get::<Option<String>>("username")?,
        }),
        "userpass" => {
            let Some(username) = options.get::<Option<String>>("username")? else {
                return Err(crate::lua_error::invalid_argument(
                    op,
                    "`auth.username` is required for `userpass`",
                ));
            };
            let Some(password) = options.get::<Option<String>>("password")? else {
                return Err(crate::lua_error::invalid_argument(
                    op,
                    "`auth.password` is required for `userpass`",
                ));
            };
            Ok(GitRemoteAuth::UserPass { username, password })
        }
        _ => Err(crate::lua_error::invalid_argument(
            op,
            "`auth.kind` must be `default`, `ssh_agent`, or `userpass`",
        )),
    }
}

fn parse_signature(options: Table, op: &str) -> mlua::Result<GitSignature> {
    for pair in options.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = parse_option_key(key, op)?;
        match key.as_str() {
            "name" | "email" => {}
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
                    format!("unknown signature field `{key}`"),
                ));
            }
        }
    }

    let Some(name) = options.get::<Option<String>>("name")? else {
        return Err(crate::lua_error::invalid_argument(op, "`name` is required"));
    };
    let Some(email) = options.get::<Option<String>>("email")? else {
        return Err(crate::lua_error::invalid_argument(
            op,
            "`email` is required",
        ));
    };

    Ok(GitSignature { name, email })
}

fn parse_paths(value: Value, op: &str) -> mlua::Result<Vec<String>> {
    match value {
        Value::String(value) => Ok(vec![value.to_str()?.to_string()]),
        Value::Table(value) => parse_string_list(value, op, "paths"),
        _ => Err(crate::lua_error::invalid_argument(
            op,
            "expects a string path or an array of paths",
        )),
    }
}

fn parse_string_list_from_value(value: Value, op: &str, field: &str) -> mlua::Result<Vec<String>> {
    match value {
        Value::String(value) => Ok(vec![value.to_str()?.to_string()]),
        Value::Table(value) => parse_string_list(value, op, field),
        _ => Err(crate::lua_error::invalid_argument(
            op,
            format!("`{field}` must be a string or an array of strings"),
        )),
    }
}

fn parse_string_list(table: Table, op: &str, field: &str) -> mlua::Result<Vec<String>> {
    let mut values = Vec::new();
    for value in table.sequence_values::<String>() {
        values.push(value.map_err(|_| {
            crate::lua_error::invalid_argument(op, format!("`{field}` must be an array of strings"))
        })?);
    }
    Ok(values)
}

fn parse_option_key(key: Value, op: &str) -> mlua::Result<String> {
    match key {
        Value::String(key) => Ok(key.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_option(
            op,
            "option keys must be strings",
        )),
    }
}

fn git_head_to_lua(lua: &Lua, head: ptool_engine::GitHeadInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("oid", head.oid)?;
    table.set("shorthand", head.shorthand)?;
    table.set("detached", head.detached)?;
    table.set("unborn", head.unborn)?;
    Ok(table)
}

fn git_status_to_lua(lua: &Lua, status: GitStatusSummary) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("root", status.root)?;
    table.set("branch", status.branch)?;
    table.set("head", git_head_to_lua(lua, status.head)?)?;
    table.set("upstream", status.upstream)?;
    table.set(
        "ahead",
        i64::try_from(status.ahead).map_err(|_| {
            crate::lua_error::invalid_argument(STATUS_SIGNATURE, "`ahead` is too large")
        })?,
    )?;
    table.set(
        "behind",
        i64::try_from(status.behind).map_err(|_| {
            crate::lua_error::invalid_argument(STATUS_SIGNATURE, "`behind` is too large")
        })?,
    )?;
    table.set("clean", status.clean)?;

    let entries = lua.create_table()?;
    for (index, entry) in status.entries.into_iter().enumerate() {
        entries.set(index + 1, git_status_entry_to_lua(lua, entry)?)?;
    }
    table.set("entries", entries)?;
    Ok(table)
}

fn git_status_entry_to_lua(lua: &Lua, entry: GitStatusEntry) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("path", entry.path)?;
    table.set("index_status", entry.index_status)?;
    table.set("worktree_status", entry.worktree_status)?;
    table.set("conflicted", entry.conflicted)?;
    table.set("ignored", entry.ignored)?;
    Ok(table)
}

fn git_fetch_stats_to_lua(lua: &Lua, stats: GitFetchStats) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set(
        "received_objects",
        i64::try_from(stats.received_objects).map_err(|_| {
            crate::lua_error::invalid_argument(FETCH_SIGNATURE, "`received_objects` is too large")
        })?,
    )?;
    table.set(
        "indexed_objects",
        i64::try_from(stats.indexed_objects).map_err(|_| {
            crate::lua_error::invalid_argument(FETCH_SIGNATURE, "`indexed_objects` is too large")
        })?,
    )?;
    table.set(
        "local_objects",
        i64::try_from(stats.local_objects).map_err(|_| {
            crate::lua_error::invalid_argument(FETCH_SIGNATURE, "`local_objects` is too large")
        })?,
    )?;
    table.set(
        "total_objects",
        i64::try_from(stats.total_objects).map_err(|_| {
            crate::lua_error::invalid_argument(FETCH_SIGNATURE, "`total_objects` is too large")
        })?,
    )?;
    table.set(
        "received_bytes",
        i64::try_from(stats.received_bytes).map_err(|_| {
            crate::lua_error::invalid_argument(FETCH_SIGNATURE, "`received_bytes` is too large")
        })?,
    )?;
    Ok(table)
}
