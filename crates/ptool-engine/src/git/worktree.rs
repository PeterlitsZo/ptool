use super::{
    GitCleanOptions, GitRemoveOptions, GitRepository, GitResetMode, GitResetOptions,
    GitRestoreOptions, path_to_string, repo_error,
};
use crate::{Error, ErrorKind, Result};
use git2::{ResetType, Status, StatusOptions, build::CheckoutBuilder};
use std::path::{Component, Path};

impl GitRepository {
    pub fn restore(&self, paths: &[String], options: GitRestoreOptions) -> Result<()> {
        let op = "ptool.git.Repo:restore(paths, options?)";
        validate_exact_paths(paths, op)?;
        if options.source.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:restore source must not be empty",
            )
            .with_op(op));
        }
        if !options.staged && !options.worktree {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:restore must target staged, worktree, or both",
            )
            .with_op(op));
        }
        let object = self
            .repo
            .revparse_single(&options.source)
            .map_err(|err| repo_error(op, err))?;
        if options.staged {
            self.repo
                .reset_default(Some(&object), paths.iter().map(String::as_str))
                .map_err(|err| repo_error(op, err))?;
        }
        if options.worktree {
            let mut checkout = CheckoutBuilder::new();
            checkout.force().recreate_missing(true);
            for path in paths {
                checkout.path(path);
            }
            self.repo
                .checkout_tree(&object, Some(&mut checkout))
                .map_err(|err| repo_error(op, err))?;
        }
        Ok(())
    }

    pub fn reset(&self, rev: Option<&str>, options: GitResetOptions) -> Result<()> {
        let op = "ptool.git.Repo:reset(rev?, options?)";
        let rev = rev.unwrap_or("HEAD");
        if rev.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:reset revision must not be empty",
            )
            .with_op(op));
        }
        if options.mode == GitResetMode::Hard && !options.force {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:reset hard mode requires force = true",
            )
            .with_op(op));
        }
        let object = self
            .repo
            .revparse_single(rev)
            .map_err(|err| repo_error(op, err))?;
        let kind = match options.mode {
            GitResetMode::Soft => ResetType::Soft,
            GitResetMode::Mixed => ResetType::Mixed,
            GitResetMode::Hard => ResetType::Hard,
        };
        let mut checkout = CheckoutBuilder::new();
        if options.mode == GitResetMode::Hard {
            checkout.force().recreate_missing(true);
        }
        self.repo
            .reset(&object, kind, Some(&mut checkout))
            .map_err(|err| repo_error(op, err))
    }

    pub fn remove(&self, paths: &[String], options: GitRemoveOptions) -> Result<()> {
        let op = "ptool.git.Repo:remove(paths, options?)";
        validate_exact_paths(paths, op)?;
        if !options.force {
            for path in paths {
                let status = self
                    .repo
                    .status_file(Path::new(path))
                    .map_err(|err| repo_error(op, err))?;
                if has_local_changes(status) {
                    return Err(Error::new(
                        ErrorKind::InvalidArgs,
                        "ptool.git.Repo:remove refuses to remove paths with local changes without force = true",
                    )
                    .with_op(op)
                    .with_path(path.clone()));
                }
            }
        }

        if !options.cached {
            let workdir = self.repo.workdir().ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidArgs,
                    "cannot remove worktree paths from a bare repository",
                )
                .with_op(op)
            })?;
            for path in paths {
                remove_path(&workdir.join(path), op)?;
            }
        }

        let mut index = self.repo.index().map_err(|err| repo_error(op, err))?;
        index
            .remove_all(paths.iter().map(String::as_str), None)
            .map_err(|err| repo_error(op, err))?;
        index.write().map_err(|err| repo_error(op, err))
    }

    pub fn clean(&self, options: GitCleanOptions) -> Result<Vec<String>> {
        let op = "ptool.git.Repo:clean(options?)";
        if !options.dry_run && !options.force {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:clean execution requires dry_run = false and force = true",
            )
            .with_op(op));
        }
        for path in &options.paths {
            validate_relative_path(path, op)?;
        }
        let workdir = self.repo.workdir().ok_or_else(|| {
            Error::new(ErrorKind::InvalidArgs, "cannot clean a bare repository").with_op(op)
        })?;
        let mut status_options = StatusOptions::new();
        status_options
            .include_untracked(true)
            .recurse_untracked_dirs(false)
            .include_ignored(options.ignored);
        for path in &options.paths {
            status_options.pathspec(path);
        }
        let statuses = self
            .repo
            .statuses(Some(&mut status_options))
            .map_err(|err| repo_error(op, err))?;
        let mut paths = Vec::new();
        for entry in statuses.iter() {
            let status = entry.status();
            if !(status.is_wt_new() || options.ignored && status.is_ignored()) {
                continue;
            }
            let Some(path) = entry.path() else {
                continue;
            };
            validate_relative_path(path, op)?;
            let full_path = workdir.join(path);
            if full_path.is_dir() && !options.dirs {
                continue;
            }
            paths.push(path.to_string());
        }
        paths.sort();
        paths.dedup();

        if !options.dry_run {
            // Remove deepest paths first so explicitly listed directories can be removed last.
            paths.sort_by_key(|path| std::cmp::Reverse(path.matches('/').count()));
            for path in &paths {
                remove_path(&workdir.join(path), op)?;
            }
            paths.sort();
        }
        Ok(paths)
    }
}

fn validate_exact_paths(paths: &[String], op: &str) -> Result<()> {
    if paths.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} requires at least one path"),
        )
        .with_op(op));
    }
    for path in paths {
        validate_relative_path(path, op)?;
        if path.contains('*') || path.contains('?') || path.contains('[') {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                format!("{op} requires exact paths, not pathspec patterns"),
            )
            .with_op(op)
            .with_path(path.clone()));
        }
    }
    Ok(())
}

fn validate_relative_path(path: &str, op: &str) -> Result<()> {
    let path_value = Path::new(path);
    if path.is_empty()
        || path_value.is_absolute()
        || path_value.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} requires a safe repository-relative path"),
        )
        .with_op(op)
        .with_path(path.to_string()));
    }
    Ok(())
}

fn has_local_changes(status: Status) -> bool {
    status.intersects(
        Status::INDEX_NEW
            | Status::INDEX_MODIFIED
            | Status::INDEX_DELETED
            | Status::INDEX_RENAMED
            | Status::INDEX_TYPECHANGE
            | Status::WT_MODIFIED
            | Status::WT_DELETED
            | Status::WT_RENAMED
            | Status::WT_TYPECHANGE
            | Status::CONFLICTED,
    )
}

fn remove_path(path: &Path, op: &str) -> Result<()> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(Error::new(ErrorKind::Io, format!("{op} failed: {err}"))
                .with_op(op)
                .with_path(path_to_string(path)));
        }
    };
    let result = if metadata.file_type().is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    result.map_err(|err| {
        Error::new(ErrorKind::Io, format!("{op} failed: {err}"))
            .with_op(op)
            .with_path(path_to_string(path))
    })
}
