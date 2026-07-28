mod advanced;
mod branch;
mod config;
mod history;
mod integrate;
mod options;
mod rebase;
mod remote;
mod render;
mod stash;
mod tag;
mod worktree;

use mlua::{Lua, Table, UserData, UserDataMethods, Value, Variadic};
use ptool_console::GitAction;
use ptool_engine::{GitRepository, PromptConfirmOptions, PtoolEngine};
use std::path::{Path, PathBuf};

use options::{
    parse_add_options, parse_checkout_options, parse_clone_options, parse_commit_options,
    parse_fetch_call, parse_init_options, parse_paths, parse_push_call, parse_status_options,
    parse_switch_options,
};
use render::{git_fetch_stats_to_lua, git_head_to_lua, git_push_result_to_lua, git_status_to_lua};

pub(super) const INIT_SIGNATURE: &str = "ptool.git.init(path?, options?)";
pub(super) const OPEN_SIGNATURE: &str = "ptool.git.open(path?)";
pub(super) const DISCOVER_SIGNATURE: &str = "ptool.git.discover(path?)";
pub(super) const CLONE_SIGNATURE: &str = "ptool.git.clone(url, path[, options])";
pub(super) const HEAD_SIGNATURE: &str = "ptool.git.Repo:head()";
pub(super) const RESOLVE_SIGNATURE: &str = "ptool.git.Repo:resolve(rev)";
pub(super) const COMMIT_INFO_SIGNATURE: &str = "ptool.git.Repo:commit_info(rev?)";
pub(super) const LOG_SIGNATURE: &str = "ptool.git.Repo:log(options?)";
pub(super) const DIFF_SIGNATURE: &str = "ptool.git.Repo:diff(options?)";
pub(super) const DESCRIBE_SIGNATURE: &str = "ptool.git.Repo:describe(options?)";
pub(super) const CURRENT_BRANCH_SIGNATURE: &str = "ptool.git.Repo:current_branch()";
pub(super) const STATUS_SIGNATURE: &str = "ptool.git.Repo:status(options?)";
pub(super) const IS_CLEAN_SIGNATURE: &str = "ptool.git.Repo:is_clean(options?)";
pub(super) const ADD_SIGNATURE: &str = "ptool.git.Repo:add(paths[, options])";
pub(super) const COMMIT_SIGNATURE: &str = "ptool.git.Repo:commit(message[, options])";
pub(super) const CHECKOUT_SIGNATURE: &str = "ptool.git.Repo:checkout(rev[, options])";
pub(super) const SWITCH_SIGNATURE: &str = "ptool.git.Repo:switch(branch[, options])";
pub(super) const FETCH_SIGNATURE: &str = "ptool.git.Repo:fetch(remote?, options?)";
pub(super) const TAGS_SIGNATURE: &str = "ptool.git.Repo:tags(pattern?)";
pub(super) const TAG_CREATE_SIGNATURE: &str = "ptool.git.Repo:tag_create(name, target?, options?)";
pub(super) const TAG_DELETE_SIGNATURE: &str = "ptool.git.Repo:tag_delete(name, options?)";
pub(super) const PUSH_SIGNATURE: &str = "ptool.git.Repo:push(remote?, refspecs?, options?)";

pub(crate) struct LuaGitRepo {
    engine: PtoolEngine,
    repo: GitRepository,
    current_dir: PathBuf,
}

pub(super) struct ConfirmableOptions<T> {
    pub(super) inner: T,
    pub(super) confirm: bool,
}

pub(crate) fn init(
    path: Option<String>,
    options: Option<Table>,
    current_dir: &Path,
    engine: &PtoolEngine,
) -> mlua::Result<LuaGitRepo> {
    let options = parse_init_options(options)?;
    let target_path = match path.as_deref() {
        Some(path) => resolve_repo_path(current_dir, path),
        None => current_dir.to_path_buf(),
    };
    if options.confirm {
        let target = target_path.to_string_lossy();
        confirm_git_action(engine, INIT_SIGNATURE, GitAction::Init { path: &target })?;
    }
    let repo = engine
        .git_init(path.as_deref(), current_dir, options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, INIT_SIGNATURE))?;
    Ok(LuaGitRepo {
        engine: engine.clone(),
        repo,
        current_dir: current_dir.to_path_buf(),
    })
}

pub(crate) fn open(
    path: Option<String>,
    current_dir: &Path,
    engine: &PtoolEngine,
) -> mlua::Result<LuaGitRepo> {
    let repo = engine
        .git_open(path.as_deref(), current_dir)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, OPEN_SIGNATURE))?;
    Ok(LuaGitRepo {
        engine: engine.clone(),
        repo,
        current_dir: current_dir.to_path_buf(),
    })
}

pub(crate) fn discover(
    path: Option<String>,
    current_dir: &Path,
    engine: &PtoolEngine,
) -> mlua::Result<LuaGitRepo> {
    let repo = engine
        .git_discover(path.as_deref(), current_dir)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, DISCOVER_SIGNATURE))?;
    Ok(LuaGitRepo {
        engine: engine.clone(),
        repo,
        current_dir: current_dir.to_path_buf(),
    })
}

pub(crate) fn clone_repo(
    url: String,
    path: String,
    options: Option<Table>,
    current_dir: &Path,
    engine: &PtoolEngine,
) -> mlua::Result<LuaGitRepo> {
    let options = parse_clone_options(options, current_dir)?;
    let target_path = resolve_repo_path(current_dir, &path);
    if options.confirm {
        let destination = target_path.to_string_lossy();
        confirm_git_action(
            engine,
            CLONE_SIGNATURE,
            GitAction::Clone {
                url: &url,
                destination: &destination,
            },
        )?;
    }
    let repo = engine
        .git_clone(&url, &path, current_dir, options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, CLONE_SIGNATURE))?;
    Ok(LuaGitRepo {
        engine: engine.clone(),
        repo,
        current_dir: current_dir.to_path_buf(),
    })
}

impl UserData for LuaGitRepo {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("path", |_, this, ()| Ok(this.repo.path()));
        methods.add_method("root", |_, this, ()| Ok(this.repo.root()));
        methods.add_method("is_bare", |_, this, ()| Ok(this.repo.is_bare()));
        methods.add_method("head", |lua, this, ()| this.head(lua));
        methods.add_method("current_branch", |_, this, ()| this.current_branch());
        methods.add_method("resolve", |lua, this, rev: String| {
            history::resolve(this, lua, rev)
        });
        methods.add_method("commit_info", |lua, this, rev: Option<String>| {
            history::commit_info(this, lua, rev)
        });
        methods.add_method("log", |lua, this, options: Option<Table>| {
            history::log(this, lua, options)
        });
        methods.add_method("diff", |lua, this, options: Option<Table>| {
            history::diff(this, lua, options)
        });
        methods.add_method("describe", |_, this, options: Option<Table>| {
            history::describe(this, options)
        });
        methods.add_method("branches", |lua, this, options: Option<Table>| {
            branch::branches(this, lua, options)
        });
        methods.add_method(
            "branch_create",
            |lua, this, (name, options): (String, Option<Table>)| {
                branch::create(this, lua, name, options)
            },
        );
        methods.add_method(
            "branch_delete",
            |_, this, (name, options): (String, Option<Table>)| branch::delete(this, name, options),
        );
        methods.add_method(
            "branch_rename",
            |lua, this, (old, new, options): (String, String, Option<Table>)| {
                branch::rename(this, lua, old, new, options)
            },
        );
        methods.add_method(
            "branch_set_upstream",
            |_, this, (name, upstream, options): (String, Option<String>, Option<Table>)| {
                branch::set_upstream(this, name, upstream, options)
            },
        );
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
        methods.add_method("push", |lua, this, args: Variadic<Value>| {
            this.push(lua, args)
        });
        methods.add_method("remotes", |lua, this, ()| remote::remotes(this, lua));
        methods.add_method("remote", |lua, this, name: String| {
            remote::remote(this, lua, name)
        });
        methods.add_method(
            "remote_add",
            |lua, this, (name, url, options): (String, String, Option<Table>)| {
                remote::add(this, lua, name, url, options)
            },
        );
        methods.add_method(
            "remote_remove",
            |_, this, (name, options): (String, Option<Table>)| remote::remove(this, name, options),
        );
        methods.add_method(
            "remote_rename",
            |lua, this, (name, new_name, options): (String, String, Option<Table>)| {
                remote::rename(this, lua, name, new_name, options)
            },
        );
        methods.add_method(
            "remote_set_url",
            |lua, this, (name, url, options): (String, String, Option<Table>)| {
                remote::set_url(this, lua, name, url, options)
            },
        );
        methods.add_method("tags", |lua, this, pattern: Option<String>| {
            tag::tags(this, lua, pattern)
        });
        methods.add_method("tag_create", |lua, this, args: Variadic<Value>| {
            tag::tag_create(this, lua, args)
        });
        methods.add_method(
            "tag_delete",
            |_, this, (name, options): (String, Option<Table>)| {
                tag::tag_delete(this, name, options)
            },
        );
        methods.add_method(
            "restore",
            |_, this, (paths, options): (Value, Option<Table>)| {
                worktree::restore(this, paths, options)
            },
        );
        methods.add_method(
            "reset",
            |_, this, (rev, options): (Option<String>, Option<Table>)| {
                worktree::reset(this, rev, options)
            },
        );
        methods.add_method(
            "remove",
            |_, this, (paths, options): (Value, Option<Table>)| {
                worktree::remove(this, paths, options)
            },
        );
        methods.add_method("clean", |lua, this, options: Option<Table>| {
            worktree::clean(this, lua, options)
        });
        methods.add_method(
            "config_get",
            |lua, this, (name, options): (String, Option<Table>)| {
                config::get(this, lua, name, options)
            },
        );
        methods.add_method("config_list", |lua, this, options: Option<Table>| {
            config::list(this, lua, options)
        });
        methods.add_method(
            "config_set",
            |lua, this, (name, value, options): (String, Value, Option<Table>)| {
                config::set(this, lua, name, value, options)
            },
        );
        methods.add_method(
            "config_remove",
            |_, this, (name, options): (String, Option<Table>)| config::remove(this, name, options),
        );
        methods.add_method("state", |_, this, ()| Ok(integrate::state(this)));
        methods.add_method("conflicts", |lua, this, ()| integrate::conflicts(this, lua));
        methods.add_method("merge_analysis", |_, this, rev: String| {
            integrate::merge_analysis(this, rev)
        });
        methods.add_method(
            "merge",
            |lua, this, (rev, options): (String, Option<Table>)| {
                integrate::merge(this, lua, rev, options)
            },
        );
        methods.add_method("merge_abort", |_, this, options: Option<Table>| {
            integrate::merge_abort(this, options)
        });
        methods.add_method("pull", |lua, this, args: Variadic<Value>| {
            integrate::pull(this, lua, args)
        });
        methods.add_method(
            "cherry_pick",
            |lua, this, (rev, options): (String, Option<Table>)| {
                integrate::cherry_pick(this, lua, rev, options)
            },
        );
        methods.add_method("cherry_pick_abort", |_, this, options: Option<Table>| {
            integrate::cherry_pick_abort(this, options)
        });
        methods.add_method(
            "revert",
            |lua, this, (rev, options): (String, Option<Table>)| {
                integrate::revert(this, lua, rev, options)
            },
        );
        methods.add_method("revert_abort", |_, this, options: Option<Table>| {
            integrate::revert_abort(this, options)
        });
        methods.add_method_mut("stash_save", |_, this, args: Variadic<Value>| {
            stash::save(this, args)
        });
        methods.add_method_mut("stashes", |lua, this, ()| stash::list(this, lua));
        methods.add_method_mut("stash_apply", |lua, this, args: Variadic<Value>| {
            stash::apply(this, lua, args, false)
        });
        methods.add_method_mut("stash_pop", |lua, this, args: Variadic<Value>| {
            stash::apply(this, lua, args, true)
        });
        methods.add_method_mut("stash_drop", |_, this, args: Variadic<Value>| {
            stash::drop(this, args)
        });
        methods.add_method("rebase", |lua, this, options: Table| {
            rebase::start(this, lua, options)
        });
        methods.add_method("rebase_continue", |lua, this, options: Option<Table>| {
            rebase::continue_rebase(this, lua, options)
        });
        methods.add_method("rebase_abort", |_, this, options: Option<Table>| {
            rebase::abort(this, options)
        });
        methods.add_method("worktrees", |lua, this, ()| advanced::worktrees(this, lua));
        methods.add_method(
            "worktree_add",
            |lua, this, (name, path, options): (String, String, Option<Table>)| {
                advanced::worktree_add(this, lua, name, path, options)
            },
        );
        methods.add_method("worktree_lock", |_, this, args: Variadic<Value>| {
            advanced::worktree_lock(this, args)
        });
        methods.add_method(
            "worktree_unlock",
            |_, this, (name, options): (String, Option<Table>)| {
                advanced::worktree_unlock(this, name, options)
            },
        );
        methods.add_method(
            "worktree_prune",
            |_, this, (name, options): (String, Option<Table>)| {
                advanced::worktree_prune(this, name, options)
            },
        );
        methods.add_method("submodules", |lua, this, ()| {
            advanced::submodules(this, lua)
        });
        methods.add_method("submodule_init", |_, this, args: Variadic<Value>| {
            advanced::submodule_init(this, args)
        });
        methods.add_method("submodule_update", |_, this, args: Variadic<Value>| {
            advanced::submodule_update(this, args)
        });
        methods.add_method("submodule_sync", |_, this, args: Variadic<Value>| {
            advanced::submodule_sync(this, args)
        });
        methods.add_method(
            "blame",
            |lua, this, (path, options): (String, Option<Table>)| {
                advanced::blame(this, lua, path, options)
            },
        );
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
        if options.confirm {
            let repository = self.repo_label();
            confirm_git_action(
                &self.engine,
                ADD_SIGNATURE,
                GitAction::Add {
                    repository: &repository,
                    paths: &paths,
                },
            )?;
        }
        self.repo
            .add(&paths, options.inner)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, ADD_SIGNATURE))
    }

    fn commit(&self, message: String, options: Option<Table>) -> mlua::Result<String> {
        let options = parse_commit_options(options)?;
        if options.confirm {
            let repository = self.repo_label();
            confirm_git_action(
                &self.engine,
                COMMIT_SIGNATURE,
                GitAction::Commit {
                    repository: &repository,
                    message: &message,
                },
            )?;
        }
        self.repo
            .commit(&message, options.inner)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, COMMIT_SIGNATURE))
    }

    fn checkout(&self, rev: String, options: Option<Table>) -> mlua::Result<()> {
        let options = parse_checkout_options(options)?;
        if options.confirm {
            let repository = self.repo_label();
            confirm_git_action(
                &self.engine,
                CHECKOUT_SIGNATURE,
                GitAction::Checkout {
                    repository: &repository,
                    revision: &rev,
                },
            )?;
        }
        self.repo
            .checkout(&rev, options.inner)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, CHECKOUT_SIGNATURE))
    }

    fn switch(&self, branch: String, options: Option<Table>) -> mlua::Result<()> {
        let options = parse_switch_options(options)?;
        if options.confirm {
            let repository = self.repo_label();
            confirm_git_action(
                &self.engine,
                SWITCH_SIGNATURE,
                GitAction::Switch {
                    repository: &repository,
                    branch: &branch,
                },
            )?;
        }
        self.repo
            .switch(&branch, options.inner)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, SWITCH_SIGNATURE))
    }

    fn fetch(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
        let (remote, options) = parse_fetch_call(args, &self.current_dir)?;
        let remote_name = remote.as_deref().unwrap_or("origin");
        if options.confirm {
            let repository = self.repo_label();
            confirm_git_action(
                &self.engine,
                FETCH_SIGNATURE,
                GitAction::Fetch {
                    repository: &repository,
                    remote: remote_name,
                },
            )?;
        }
        let stats = self
            .repo
            .fetch(remote.as_deref(), options.inner)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, FETCH_SIGNATURE))?;
        git_fetch_stats_to_lua(lua, stats)
    }

    fn push(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
        let (remote, refspecs, options) = parse_push_call(args, &self.current_dir)?;
        let remote_name = remote.as_deref().unwrap_or("origin");
        if options.confirm {
            let repository = self.repo_label();
            confirm_git_action(
                &self.engine,
                PUSH_SIGNATURE,
                GitAction::Push {
                    repository: &repository,
                    remote: remote_name,
                    refspecs: &refspecs,
                },
            )?;
        }
        let result = self
            .repo
            .push(remote.as_deref(), &refspecs, options.inner)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, PUSH_SIGNATURE))?;
        git_push_result_to_lua(lua, result)
    }

    pub(super) fn repo_label(&self) -> String {
        self.repo.root().unwrap_or_else(|| self.repo.path())
    }
}

pub(super) fn confirm_git_action(
    engine: &PtoolEngine,
    op: &'static str,
    action: GitAction<'_>,
) -> mlua::Result<()> {
    let confirmation = engine.console().git_confirmation(action);
    match engine.prompt_confirm(
        op,
        &confirmation.prompt,
        PromptConfirmOptions {
            default: Some(true),
            help: confirmation.help,
        },
    ) {
        Ok(true) => Ok(()),
        Ok(false) => Err(
            crate::lua_error::LuaError::cancelled(op, confirmation.cancelled_detail)
                .into_mlua_error(),
        ),
        Err(err) => Err(crate::lua_error::lua_error_from_engine(err, op)),
    }
}

fn resolve_repo_path(current_dir: &Path, path: &str) -> std::path::PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    }
}
