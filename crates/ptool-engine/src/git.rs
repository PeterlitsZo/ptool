use crate::{Error, ErrorKind, Result};
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{
    BranchType, Cred, CredentialType, FetchOptions, IndexAddOption, Oid, PushOptions,
    RemoteCallbacks, Repository, Signature, Status, StatusOptions,
};
use std::path::{Path, PathBuf};

pub struct GitRepository {
    repo: Repository,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitCloneOptions {
    pub branch: Option<String>,
    pub bare: bool,
    pub auth: GitRemoteAuth,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitFetchOptions {
    pub refspecs: Vec<String>,
    pub auth: GitRemoteAuth,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitPushOptions {
    pub auth: GitRemoteAuth,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitStatusOptions {
    pub include_untracked: bool,
    pub include_ignored: bool,
    pub recurse_untracked_dirs: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitAddOptions {
    pub update: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitCommitOptions {
    pub author: Option<GitSignature>,
    pub committer: Option<GitSignature>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitCheckoutOptions {
    pub force: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitSwitchOptions {
    pub create: bool,
    pub force: bool,
    pub start_point: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitSignature {
    pub name: String,
    pub email: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum GitRemoteAuth {
    #[default]
    Default,
    SshAgent {
        username: Option<String>,
    },
    UserPass {
        username: String,
        password: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitHeadInfo {
    pub oid: Option<String>,
    pub shorthand: Option<String>,
    pub detached: bool,
    pub unborn: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitStatusEntry {
    pub path: String,
    pub index_status: Option<String>,
    pub worktree_status: Option<String>,
    pub conflicted: bool,
    pub ignored: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitStatusSummary {
    pub root: Option<String>,
    pub branch: Option<String>,
    pub head: GitHeadInfo,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub clean: bool,
    pub entries: Vec<GitStatusEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitFetchStats {
    pub received_objects: usize,
    pub indexed_objects: usize,
    pub local_objects: usize,
    pub total_objects: usize,
    pub received_bytes: usize,
}

impl Default for GitStatusOptions {
    fn default() -> Self {
        Self {
            include_untracked: true,
            include_ignored: false,
            recurse_untracked_dirs: true,
        }
    }
}

pub fn open(path: Option<&str>, current_dir: &Path) -> Result<GitRepository> {
    let path = resolve_repo_path(current_dir, path);
    let repo =
        Repository::open(&path).map_err(|err| repo_path_error("ptool.git.open", &path, err))?;
    Ok(GitRepository { repo })
}

pub fn discover(path: Option<&str>, current_dir: &Path) -> Result<GitRepository> {
    let path = resolve_repo_path(current_dir, path);
    let repo = Repository::discover(&path)
        .map_err(|err| repo_path_error("ptool.git.discover", &path, err))?;
    Ok(GitRepository { repo })
}

pub fn clone_repo(
    url: &str,
    path: &str,
    current_dir: &Path,
    options: GitCloneOptions,
) -> Result<GitRepository> {
    if url.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.git.clone requires a non-empty url",
        )
        .with_op("ptool.git.clone"));
    }
    if path.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.git.clone requires a non-empty path",
        )
        .with_op("ptool.git.clone"));
    }

    let target_path = resolve_repo_path(current_dir, Some(path));
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(build_remote_callbacks(options.auth.clone()));

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);
    if let Some(branch) = options.branch.as_deref() {
        builder.branch(branch);
    }
    if options.bare {
        builder.bare(true);
    }

    let repo = builder
        .clone(url, &target_path)
        .map_err(|err| repo_error("ptool.git.clone", err).with_input(url.to_string()))?;
    Ok(GitRepository { repo })
}

impl GitRepository {
    pub fn path(&self) -> String {
        path_to_string(self.repo.path())
    }

    pub fn root(&self) -> Option<String> {
        self.repo.workdir().map(path_to_string)
    }

    pub fn is_bare(&self) -> bool {
        self.repo.is_bare()
    }

    pub fn head(&self) -> Result<GitHeadInfo> {
        build_head_info(&self.repo, "ptool.git.Repo:head()")
    }

    pub fn current_branch(&self) -> Result<Option<String>> {
        current_branch_name(&self.repo, "ptool.git.Repo:current_branch()")
    }

    pub fn status(&self, options: GitStatusOptions) -> Result<GitStatusSummary> {
        let mut status_options = StatusOptions::new();
        status_options.include_untracked(options.include_untracked);
        status_options.include_ignored(options.include_ignored);
        status_options.recurse_untracked_dirs(options.recurse_untracked_dirs);
        status_options.renames_head_to_index(true);
        status_options.renames_index_to_workdir(true);
        status_options.include_unmodified(false);

        let statuses = self
            .repo
            .statuses(Some(&mut status_options))
            .map_err(|err| repo_error("ptool.git.Repo:status()", err))?;

        let mut entries = Vec::new();
        for entry in statuses.iter() {
            let status = entry.status();
            let index_status = index_status_name(status).map(str::to_string);
            let worktree_status = worktree_status_name(status).map(str::to_string);
            let conflicted = status.is_conflicted();
            let ignored = status.is_ignored();

            if ignored && !options.include_ignored {
                continue;
            }

            entries.push(GitStatusEntry {
                path: entry.path().unwrap_or_default().to_string(),
                index_status,
                worktree_status,
                conflicted,
                ignored,
            });
        }

        let head = build_head_info(&self.repo, "ptool.git.Repo:status()")?;
        let branch = current_branch_name(&self.repo, "ptool.git.Repo:status()")?;
        let (upstream, ahead, behind) = branch_tracking(&self.repo, branch.as_deref())?;
        let clean = entries.iter().all(|entry| entry.ignored);

        Ok(GitStatusSummary {
            root: self.root(),
            branch,
            head,
            upstream,
            ahead,
            behind,
            clean,
            entries,
        })
    }

    pub fn is_clean(&self, options: GitStatusOptions) -> Result<bool> {
        Ok(self.status(options)?.clean)
    }

    pub fn add(&self, paths: &[String], options: GitAddOptions) -> Result<()> {
        validate_path_list(paths, "ptool.git.Repo:add(paths)")?;

        let mut index = self
            .repo
            .index()
            .map_err(|err| repo_error("ptool.git.Repo:add(paths)", err))?;

        let pathspecs: Vec<&str> = paths.iter().map(String::as_str).collect();
        if options.update {
            index
                .update_all(pathspecs, None)
                .map_err(|err| repo_error("ptool.git.Repo:add(paths)", err))?;
        } else {
            index
                .add_all(pathspecs, IndexAddOption::DEFAULT, None)
                .map_err(|err| repo_error("ptool.git.Repo:add(paths)", err))?;
        }
        index
            .write()
            .map_err(|err| repo_error("ptool.git.Repo:add(paths)", err))
    }

    pub fn commit(&self, message: &str, options: GitCommitOptions) -> Result<String> {
        if message.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:commit(message) requires a non-empty message",
            )
            .with_op("ptool.git.Repo:commit(message)"));
        }

        let mut index = self
            .repo
            .index()
            .map_err(|err| repo_error("ptool.git.Repo:commit(message)", err))?;
        let tree_oid = index
            .write_tree()
            .map_err(|err| repo_error("ptool.git.Repo:commit(message)", err))?;
        let tree = self
            .repo
            .find_tree(tree_oid)
            .map_err(|err| repo_error("ptool.git.Repo:commit(message)", err))?;

        let author = build_signature(
            options.author.as_ref(),
            "ptool.git.Repo:commit(message)",
            self.repo.signature().ok(),
        )?;
        let committer = build_signature(
            options.committer.as_ref().or(options.author.as_ref()),
            "ptool.git.Repo:commit(message)",
            self.repo.signature().ok(),
        )?;

        let parents = match self.repo.head() {
            Ok(head) => {
                if let Some(oid) = head.target() {
                    vec![
                        self.repo
                            .find_commit(oid)
                            .map_err(|err| repo_error("ptool.git.Repo:commit(message)", err))?,
                    ]
                } else {
                    Vec::new()
                }
            }
            Err(err) if err.code() == git2::ErrorCode::UnbornBranch => Vec::new(),
            Err(err) => return Err(repo_error("ptool.git.Repo:commit(message)", err)),
        };
        let parent_refs: Vec<_> = parents.iter().collect();

        let oid = self
            .repo
            .commit(
                Some("HEAD"),
                &author,
                &committer,
                message,
                &tree,
                &parent_refs,
            )
            .map_err(|err| repo_error("ptool.git.Repo:commit(message)", err))?;
        Ok(oid.to_string())
    }

    pub fn checkout(&self, rev: &str, options: GitCheckoutOptions) -> Result<()> {
        if rev.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:checkout(rev) requires a non-empty revision",
            )
            .with_op("ptool.git.Repo:checkout(rev)"));
        }

        let (object, reference) = self
            .repo
            .revparse_ext(rev)
            .map_err(|err| repo_error("ptool.git.Repo:checkout(rev)", err))?;

        let mut builder = CheckoutBuilder::new();
        if options.force {
            builder.force();
        }
        self.repo
            .checkout_tree(&object, Some(&mut builder))
            .map_err(|err| repo_error("ptool.git.Repo:checkout(rev)", err))?;

        if let Some(reference) = reference {
            if let Some(name) = reference.name() {
                self.repo
                    .set_head(name)
                    .map_err(|err| repo_error("ptool.git.Repo:checkout(rev)", err))?;
            } else {
                self.repo
                    .set_head_detached(object.id())
                    .map_err(|err| repo_error("ptool.git.Repo:checkout(rev)", err))?;
            }
        } else {
            self.repo
                .set_head_detached(object.id())
                .map_err(|err| repo_error("ptool.git.Repo:checkout(rev)", err))?;
        }

        Ok(())
    }

    pub fn switch(&self, branch: &str, options: GitSwitchOptions) -> Result<()> {
        if branch.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:switch(branch) requires a non-empty branch",
            )
            .with_op("ptool.git.Repo:switch(branch)"));
        }

        if options.create {
            let start_point = options.start_point.as_deref().unwrap_or("HEAD");
            let commit = self
                .repo
                .revparse_single(start_point)
                .and_then(|object| object.peel_to_commit())
                .map_err(|err| repo_error("ptool.git.Repo:switch(branch)", err))?;
            self.repo
                .branch(branch, &commit, false)
                .map_err(|err| repo_error("ptool.git.Repo:switch(branch)", err))?;
        } else {
            self.repo
                .find_branch(branch, BranchType::Local)
                .map_err(|err| repo_error("ptool.git.Repo:switch(branch)", err))?;
        }

        let head_name = format!("refs/heads/{branch}");
        self.repo
            .set_head(&head_name)
            .map_err(|err| repo_error("ptool.git.Repo:switch(branch)", err))?;

        let mut builder = CheckoutBuilder::new();
        if options.force {
            builder.force();
        }
        self.repo
            .checkout_head(Some(&mut builder))
            .map_err(|err| repo_error("ptool.git.Repo:switch(branch)", err))
    }

    pub fn fetch(
        &self,
        remote_name: Option<&str>,
        options: GitFetchOptions,
    ) -> Result<GitFetchStats> {
        let remote_name = remote_name.unwrap_or("origin");
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .map_err(|err| repo_error("ptool.git.Repo:fetch(remote)", err))?;

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(build_remote_callbacks(options.auth));

        let refspecs: Vec<&str> = options.refspecs.iter().map(String::as_str).collect();
        remote
            .fetch(&refspecs, Some(&mut fetch_options), None)
            .map_err(|err| repo_error("ptool.git.Repo:fetch(remote)", err))?;
        let stats = remote.stats();

        Ok(GitFetchStats {
            received_objects: stats.received_objects(),
            indexed_objects: stats.indexed_objects(),
            local_objects: stats.local_objects(),
            total_objects: stats.total_objects(),
            received_bytes: stats.received_bytes(),
        })
    }

    pub fn push(
        &self,
        remote_name: Option<&str>,
        refspecs: &[String],
        options: GitPushOptions,
    ) -> Result<()> {
        let remote_name = remote_name.unwrap_or("origin");
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .map_err(|err| repo_error("ptool.git.Repo:push(remote)", err))?;

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(build_remote_callbacks(options.auth));

        let owned_refspecs = if refspecs.is_empty() {
            vec![default_push_refspec(&self.repo)?]
        } else {
            refspecs.to_vec()
        };
        let refspec_slices: Vec<&str> = owned_refspecs.iter().map(String::as_str).collect();

        remote
            .push(&refspec_slices, Some(&mut push_options))
            .map_err(|err| repo_error("ptool.git.Repo:push(remote)", err))
    }
}

fn resolve_repo_path(current_dir: &Path, path: Option<&str>) -> PathBuf {
    match path {
        Some(path) if Path::new(path).is_absolute() => PathBuf::from(path),
        Some(path) => current_dir.join(path),
        None => current_dir.to_path_buf(),
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn repo_error(op: &str, err: git2::Error) -> Error {
    Error::new(ErrorKind::Git, format!("{op} failed: {err}"))
        .with_op(op)
        .with_detail(format!(
            "git2 code: {:?}, class: {:?}",
            err.code(),
            err.class()
        ))
}

fn repo_path_error(op: &str, path: &Path, err: git2::Error) -> Error {
    repo_error(op, err).with_path(path_to_string(path))
}

fn build_remote_callbacks(auth: GitRemoteAuth) -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, allowed_types| {
        if let GitRemoteAuth::UserPass { username, password } = &auth
            && allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT)
        {
            return Cred::userpass_plaintext(username, password);
        }

        if let GitRemoteAuth::SshAgent { username } = &auth {
            let username = username.as_deref().or(username_from_url);
            if let Some(username) = username {
                if allowed_types.contains(CredentialType::SSH_KEY) {
                    return Cred::ssh_key_from_agent(username);
                }
                if allowed_types.contains(CredentialType::USERNAME) {
                    return Cred::username(username);
                }
            }
        }

        if allowed_types.contains(CredentialType::DEFAULT) {
            return Cred::default();
        }

        if allowed_types.contains(CredentialType::USERNAME)
            && let Some(username) = username_from_url
        {
            return Cred::username(username);
        }

        Err(git2::Error::from_str("unsupported git credential request"))
    });
    callbacks
}

fn build_head_info(repo: &Repository, op: &str) -> Result<GitHeadInfo> {
    let detached = repo.head_detached().map_err(|err| repo_error(op, err))?;
    let unborn = repo.is_empty().map_err(|err| repo_error(op, err))?;
    let head = repo.head().ok();

    let oid = head
        .as_ref()
        .and_then(|head| head.target())
        .or_else(|| {
            head.as_ref()
                .and_then(|head| head.peel_to_commit().ok().map(|commit| commit.id()))
        })
        .map(oid_to_string);
    let shorthand = head
        .as_ref()
        .and_then(|head| head.shorthand().map(str::to_string));

    Ok(GitHeadInfo {
        oid,
        shorthand,
        detached,
        unborn,
    })
}

fn current_branch_name(repo: &Repository, op: &str) -> Result<Option<String>> {
    if repo.head_detached().map_err(|err| repo_error(op, err))? {
        return Ok(None);
    }

    let head = match repo.head() {
        Ok(head) => head,
        Err(err) if err.code() == git2::ErrorCode::UnbornBranch => return Ok(None),
        Err(err) => return Err(repo_error(op, err)),
    };

    if !head.is_branch() {
        return Ok(None);
    }

    Ok(head.shorthand().map(str::to_string))
}

fn branch_tracking(
    repo: &Repository,
    branch_name: Option<&str>,
) -> Result<(Option<String>, usize, usize)> {
    let Some(branch_name) = branch_name else {
        return Ok((None, 0, 0));
    };

    let branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|err| repo_error("ptool.git.Repo:status()", err))?;
    let upstream = match branch.upstream() {
        Ok(upstream) => upstream,
        Err(err) if err.code() == git2::ErrorCode::NotFound => return Ok((None, 0, 0)),
        Err(err) => return Err(repo_error("ptool.git.Repo:status()", err)),
    };

    let upstream_name = upstream
        .name()
        .map_err(|err| repo_error("ptool.git.Repo:status()", err))?
        .map(str::to_string);

    let local_oid = branch.get().target();
    let upstream_oid = upstream.get().target();
    let (ahead, behind) = match (local_oid, upstream_oid) {
        (Some(local_oid), Some(upstream_oid)) => {
            repo.graph_ahead_behind(local_oid, upstream_oid)
                .map_err(|err| repo_error("ptool.git.Repo:status()", err))?
        }
        _ => (0, 0),
    };

    Ok((upstream_name, ahead, behind))
}

fn index_status_name(status: Status) -> Option<&'static str> {
    if status.is_index_new() {
        Some("new")
    } else if status.is_index_modified() {
        Some("modified")
    } else if status.is_index_deleted() {
        Some("deleted")
    } else if status.is_index_renamed() {
        Some("renamed")
    } else if status.is_index_typechange() {
        Some("typechange")
    } else {
        None
    }
}

fn worktree_status_name(status: Status) -> Option<&'static str> {
    if status.is_wt_new() {
        Some("new")
    } else if status.is_wt_modified() {
        Some("modified")
    } else if status.is_wt_deleted() {
        Some("deleted")
    } else if status.is_wt_renamed() {
        Some("renamed")
    } else if status.is_wt_typechange() {
        Some("typechange")
    } else if status.is_ignored() {
        Some("ignored")
    } else {
        None
    }
}

fn validate_path_list(paths: &[String], op: &str) -> Result<()> {
    if paths.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} requires at least one path"),
        )
        .with_op(op));
    }
    if paths.iter().any(|path| path.is_empty()) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} paths must not be empty"),
        )
        .with_op(op));
    }
    Ok(())
}

fn build_signature(
    input: Option<&GitSignature>,
    op: &str,
    fallback: Option<Signature<'static>>,
) -> Result<Signature<'static>> {
    if let Some(input) = input {
        if input.name.is_empty() || input.email.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                format!("{op} signature requires non-empty name and email"),
            )
            .with_op(op));
        }
        return Signature::now(&input.name, &input.email).map_err(|err| repo_error(op, err));
    }

    if let Some(fallback) = fallback {
        return Ok(fallback);
    }

    Err(Error::new(
        ErrorKind::Git,
        format!("{op} failed: git user identity is not configured"),
    )
    .with_op(op))
}

fn default_push_refspec(repo: &Repository) -> Result<String> {
    let branch = current_branch_name(repo, "ptool.git.Repo:push(remote)")?.ok_or_else(|| {
        Error::new(
            ErrorKind::Git,
            "ptool.git.Repo:push(remote) failed: cannot infer refspec from detached HEAD",
        )
        .with_op("ptool.git.Repo:push(remote)")
    })?;
    Ok(format!("refs/heads/{branch}:refs/heads/{branch}"))
}

fn oid_to_string(oid: Oid) -> String {
    oid.to_string()
}
