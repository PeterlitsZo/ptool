use super::{
    GitRepository, GitWorktreeAddOptions, GitWorktreeInfo, GitWorktreePruneOptions, path_to_string,
    repo_error,
};
use crate::{Error, ErrorKind, Result};
use git2::{WorktreeAddOptions, WorktreeLockStatus, WorktreePruneOptions};
use std::path::Path;

impl GitRepository {
    pub fn worktrees(&self) -> Result<Vec<GitWorktreeInfo>> {
        let op = "ptool.git.Repo:worktrees()";
        let names = self.repo.worktrees().map_err(|err| repo_error(op, err))?;
        let mut result = Vec::new();
        for name in names.iter().flatten() {
            result.push(self.worktree_info(name, op)?);
        }
        result.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(result)
    }

    pub fn worktree_add(
        &self,
        name: &str,
        path: &Path,
        options: GitWorktreeAddOptions,
    ) -> Result<GitWorktreeInfo> {
        let op = "ptool.git.Repo:worktree_add(name, path, options?)";
        validate_worktree_name(name, op)?;
        if path.as_os_str().is_empty() {
            return Err(
                Error::new(ErrorKind::InvalidArgs, "worktree path must not be empty").with_op(op),
            );
        }
        let reference = match options.reference.as_deref() {
            Some("") => {
                return Err(Error::new(
                    ErrorKind::InvalidArgs,
                    "worktree reference must not be empty",
                )
                .with_op(op));
            }
            Some(revision) => {
                let (_, reference) = self
                    .repo
                    .revparse_ext(revision)
                    .map_err(|err| repo_error(op, err))?;
                Some(reference.ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidArgs,
                        "worktree reference must resolve to a named Git reference",
                    )
                    .with_op(op)
                })?)
            }
            None => None,
        };
        let mut raw_options = WorktreeAddOptions::new();
        raw_options
            .lock(options.lock)
            .checkout_existing(options.checkout_existing)
            .reference(reference.as_ref());
        self.repo
            .worktree(name, path, Some(&raw_options))
            .map_err(|err| repo_error(op, err))?;
        self.worktree_info(name, op)
    }

    pub fn worktree_lock(&self, name: &str, reason: Option<&str>) -> Result<()> {
        let op = "ptool.git.Repo:worktree_lock(name, reason?, options?)";
        validate_worktree_name(name, op)?;
        if matches!(reason, Some("")) {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "worktree lock reason must not be empty",
            )
            .with_op(op));
        }
        self.repo
            .find_worktree(name)
            .and_then(|worktree| worktree.lock(reason))
            .map_err(|err| repo_error(op, err))
    }

    pub fn worktree_unlock(&self, name: &str) -> Result<()> {
        let op = "ptool.git.Repo:worktree_unlock(name, options?)";
        validate_worktree_name(name, op)?;
        self.repo
            .find_worktree(name)
            .and_then(|worktree| worktree.unlock())
            .map_err(|err| repo_error(op, err))
    }

    pub fn worktree_prune(&self, name: &str, options: GitWorktreePruneOptions) -> Result<()> {
        let op = "ptool.git.Repo:worktree_prune(name, options?)";
        validate_worktree_name(name, op)?;
        if (options.valid || options.locked || options.working_tree) && !options.force {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "pruning valid, locked, or on-disk worktrees requires force = true",
            )
            .with_op(op));
        }
        let worktree = self
            .repo
            .find_worktree(name)
            .map_err(|err| repo_error(op, err))?;
        let mut raw_options = WorktreePruneOptions::new();
        raw_options
            .valid(options.valid)
            .locked(options.locked)
            .working_tree(options.working_tree);
        worktree
            .prune(Some(&mut raw_options))
            .map_err(|err| repo_error(op, err))
    }

    fn worktree_info(&self, name: &str, op: &str) -> Result<GitWorktreeInfo> {
        let worktree = self
            .repo
            .find_worktree(name)
            .map_err(|err| repo_error(op, err))?;
        let (locked, lock_reason) = match worktree.is_locked().map_err(|err| repo_error(op, err))? {
            WorktreeLockStatus::Unlocked => (false, None),
            WorktreeLockStatus::Locked(reason) => (true, reason),
        };
        Ok(GitWorktreeInfo {
            name: worktree.name().unwrap_or(name).to_string(),
            path: path_to_string(worktree.path()),
            locked,
            lock_reason,
            valid: worktree.validate().is_ok(),
        })
    }
}

fn validate_worktree_name(name: &str, op: &str) -> Result<()> {
    if name.is_empty() || name.contains('/') || name.contains('\\') {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "worktree name must be non-empty and must not contain path separators",
        )
        .with_op(op));
    }
    Ok(())
}
