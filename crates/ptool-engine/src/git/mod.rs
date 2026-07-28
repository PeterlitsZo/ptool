mod branch;
mod config;
mod history;
mod integrate;
mod rebase;
mod remote;
mod repository;
mod stash;
mod submodule;
mod tag;
mod types;
mod worktree;
mod worktrees;

use crate::{Error, ErrorKind, Result};
use git2::{Oid, Repository, RepositoryInitOptions};
use std::path::{Path, PathBuf};

pub use remote::clone_repo;
pub use types::*;

pub struct GitRepository {
    pub(super) repo: Repository,
}

pub fn init(
    path: Option<&str>,
    current_dir: &Path,
    options: GitInitOptions,
) -> Result<GitRepository> {
    let path = resolve_repo_path(current_dir, path);
    if matches!(options.initial_head.as_deref(), Some("")) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.git.init initial_head must not be empty",
        )
        .with_op("ptool.git.init"));
    }

    let mut init_options = RepositoryInitOptions::new();
    init_options.bare(options.bare);
    if let Some(initial_head) = options.initial_head.as_deref() {
        init_options.initial_head(initial_head);
    }
    let repo = Repository::init_opts(&path, &init_options)
        .map_err(|err| repo_path_error("ptool.git.init", &path, err))?;
    Ok(GitRepository { repo })
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

pub(super) fn resolve_repo_path(current_dir: &Path, path: Option<&str>) -> PathBuf {
    match path {
        Some(path) if Path::new(path).is_absolute() => PathBuf::from(path),
        Some(path) => current_dir.join(path),
        None => current_dir.to_path_buf(),
    }
}

pub(super) fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub(super) fn repo_error(op: &str, err: git2::Error) -> Error {
    Error::new(ErrorKind::Git, format!("{op} failed: {err}"))
        .with_op(op)
        .with_detail(format!(
            "git2 code: {:?}, class: {:?}",
            err.code(),
            err.class()
        ))
}

pub(super) fn repo_path_error(op: &str, path: &Path, err: git2::Error) -> Error {
    repo_error(op, err).with_path(path_to_string(path))
}

pub(super) fn oid_to_string(oid: Oid) -> String {
    oid.to_string()
}
