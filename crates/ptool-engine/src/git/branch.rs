use super::{
    GitBranchCreateOptions, GitBranchDeleteOptions, GitBranchInfo, GitBranchKind,
    GitBranchListOptions, GitRepository, oid_to_string, repo_error,
};
use crate::{Error, ErrorKind, Result};
use git2::{Branch, BranchType, ErrorCode, Repository, build::CheckoutBuilder};

impl GitRepository {
    pub fn branches(&self, options: GitBranchListOptions) -> Result<Vec<GitBranchInfo>> {
        let op = "ptool.git.Repo:branches(options?)";
        let branch_type = match options.kind {
            GitBranchKind::Local => Some(BranchType::Local),
            GitBranchKind::Remote => Some(BranchType::Remote),
            GitBranchKind::All => None,
        };
        let iter = self
            .repo
            .branches(branch_type)
            .map_err(|err| repo_error(op, err))?;
        let mut result = Vec::new();
        for item in iter {
            let (branch, kind) = item.map_err(|err| repo_error(op, err))?;
            result.push(branch_to_info(&self.repo, &branch, kind, op)?);
        }
        result.sort_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.name.cmp(&right.name))
        });
        Ok(result)
    }

    pub fn branch_create(
        &self,
        name: &str,
        options: GitBranchCreateOptions,
    ) -> Result<GitBranchInfo> {
        let op = "ptool.git.Repo:branch_create(name, options?)";
        validate_branch_name(name, op)?;
        let start_point = options.start_point.as_deref().unwrap_or("HEAD");
        if start_point.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:branch_create start_point must not be empty",
            )
            .with_op(op));
        }
        let commit = self
            .repo
            .revparse_single(start_point)
            .and_then(|object| object.peel_to_commit())
            .map_err(|err| repo_error(op, err))?;
        let mut branch = self
            .repo
            .branch(name, &commit, options.force)
            .map_err(|err| repo_error(op, err))?;
        if let Some(upstream) = options.upstream.as_deref() {
            if upstream.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidArgs,
                    "ptool.git.Repo:branch_create upstream must not be empty",
                )
                .with_op(op));
            }
            branch
                .set_upstream(Some(upstream))
                .map_err(|err| repo_error(op, err))?;
        }
        if options.checkout {
            let reference_name = branch
                .get()
                .name()
                .ok_or_else(|| {
                    Error::new(ErrorKind::Git, "created branch has no valid reference name")
                        .with_op(op)
                })?
                .to_string();
            let mut checkout = CheckoutBuilder::new();
            self.repo
                .checkout_tree(commit.as_object(), Some(&mut checkout))
                .map_err(|err| repo_error(op, err))?;
            self.repo
                .set_head(&reference_name)
                .map_err(|err| repo_error(op, err))?;
        }
        branch_to_info(&self.repo, &branch, BranchType::Local, op)
    }

    pub fn branch_delete(&self, name: &str, options: GitBranchDeleteOptions) -> Result<()> {
        let op = "ptool.git.Repo:branch_delete(name, options?)";
        validate_branch_name(name, op)?;
        let mut branch = self
            .repo
            .find_branch(name, BranchType::Local)
            .map_err(|err| repo_error(op, err))?;
        if branch.is_head() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:branch_delete cannot delete the current branch",
            )
            .with_op(op));
        }
        if !options.force && !branch_is_merged_into_head(&self.repo, &branch, op)? {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:branch_delete refuses to delete an unmerged branch without force = true",
            )
            .with_op(op)
            .with_input(name.to_string()));
        }
        branch.delete().map_err(|err| repo_error(op, err))
    }

    pub fn branch_rename(&self, old: &str, new: &str, force: bool) -> Result<GitBranchInfo> {
        let op = "ptool.git.Repo:branch_rename(old_name, new_name, options?)";
        validate_branch_name(old, op)?;
        validate_branch_name(new, op)?;
        let mut branch = self
            .repo
            .find_branch(old, BranchType::Local)
            .map_err(|err| repo_error(op, err))?;
        let branch = branch
            .rename(new, force)
            .map_err(|err| repo_error(op, err))?;
        branch_to_info(&self.repo, &branch, BranchType::Local, op)
    }

    pub fn branch_set_upstream(&self, name: &str, upstream: Option<&str>) -> Result<()> {
        let op = "ptool.git.Repo:branch_set_upstream(name, upstream_or_nil, options?)";
        validate_branch_name(name, op)?;
        if matches!(upstream, Some("")) {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:branch_set_upstream upstream must not be empty",
            )
            .with_op(op));
        }
        let mut branch = self
            .repo
            .find_branch(name, BranchType::Local)
            .map_err(|err| repo_error(op, err))?;
        branch
            .set_upstream(upstream)
            .map_err(|err| repo_error(op, err))
    }
}

fn validate_branch_name(name: &str, op: &str) -> Result<()> {
    if name.is_empty() || !git2::Reference::is_valid_name(&format!("refs/heads/{name}")) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} received an invalid branch name"),
        )
        .with_op(op)
        .with_input(name.to_string()));
    }
    Ok(())
}

fn branch_to_info(
    repo: &Repository,
    branch: &Branch<'_>,
    kind: BranchType,
    op: &str,
) -> Result<GitBranchInfo> {
    let name = branch
        .name()
        .map_err(|err| repo_error(op, err))?
        .ok_or_else(|| Error::new(ErrorKind::Git, "branch name is not valid UTF-8").with_op(op))?
        .to_string();
    let oid = branch
        .get()
        .target()
        .or_else(|| branch.get().peel_to_commit().ok().map(|commit| commit.id()))
        .ok_or_else(|| Error::new(ErrorKind::Git, "branch has no commit target").with_op(op))?;
    let (upstream, ahead, behind) = if kind == BranchType::Local {
        match branch.upstream() {
            Ok(upstream) => {
                let upstream_name = upstream
                    .name()
                    .map_err(|err| repo_error(op, err))?
                    .map(str::to_string);
                let upstream_oid = upstream.get().target();
                let (ahead, behind) = match upstream_oid {
                    Some(upstream_oid) => repo
                        .graph_ahead_behind(oid, upstream_oid)
                        .map_err(|err| repo_error(op, err))?,
                    None => (0, 0),
                };
                (upstream_name, ahead, behind)
            }
            Err(err) if err.code() == ErrorCode::NotFound => (None, 0, 0),
            Err(err) => return Err(repo_error(op, err)),
        }
    } else {
        (None, 0, 0)
    };
    Ok(GitBranchInfo {
        name,
        kind: match kind {
            BranchType::Local => "local",
            BranchType::Remote => "remote",
        }
        .to_string(),
        oid: oid_to_string(oid),
        head: branch.is_head(),
        upstream,
        ahead,
        behind,
    })
}

fn branch_is_merged_into_head(repo: &Repository, branch: &Branch<'_>, op: &str) -> Result<bool> {
    let branch_oid = branch
        .get()
        .target()
        .ok_or_else(|| Error::new(ErrorKind::Git, "branch has no target").with_op(op))?;
    let head_oid = match repo.head() {
        Ok(head) => head
            .target()
            .or_else(|| head.peel_to_commit().ok().map(|commit| commit.id())),
        Err(err) if err.code() == ErrorCode::UnbornBranch => None,
        Err(err) => return Err(repo_error(op, err)),
    };
    let Some(head_oid) = head_oid else {
        return Ok(false);
    };
    if head_oid == branch_oid {
        return Ok(true);
    }
    repo.graph_descendant_of(head_oid, branch_oid)
        .map_err(|err| repo_error(op, err))
}
