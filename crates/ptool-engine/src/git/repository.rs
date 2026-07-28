use super::{
    GitAddOptions, GitCheckoutOptions, GitCommitOptions, GitHeadInfo, GitRepository, GitSignature,
    GitStatusEntry, GitStatusOptions, GitStatusSummary, GitSwitchOptions, oid_to_string,
    path_to_string, repo_error,
};
use crate::{Error, ErrorKind, Result};
use git2::build::CheckoutBuilder;
use git2::{BranchType, IndexAddOption, Repository, Signature, Status, StatusOptions};

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
        for path in &options.paths {
            status_options.pathspec(path);
        }
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
        let op = "ptool.git.Repo:commit(message)";
        if message.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:commit(message) requires a non-empty message",
            )
            .with_op(op));
        }

        let mut index = self.repo.index().map_err(|err| repo_error(op, err))?;
        let tree_oid = index.write_tree().map_err(|err| repo_error(op, err))?;
        let tree = self
            .repo
            .find_tree(tree_oid)
            .map_err(|err| repo_error(op, err))?;
        let previous = match self.repo.head() {
            Ok(head) => Some(head.peel_to_commit().map_err(|err| repo_error(op, err))?),
            Err(err) if err.code() == git2::ErrorCode::UnbornBranch => None,
            Err(err) => return Err(repo_error(op, err)),
        };

        if !options.allow_empty
            && previous
                .as_ref()
                .is_some_and(|commit| commit.tree_id() == tree_oid)
        {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:commit has no changes; pass allow_empty = true to proceed",
            )
            .with_op(op));
        }
        if options.amend && previous.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:commit amend requires an existing HEAD commit",
            )
            .with_op(op));
        }

        let fallback_identity = self.repo.signature().ok();
        let amend_author = previous
            .as_ref()
            .filter(|_| options.amend)
            .map(|commit| owned_signature(&commit.author(), op))
            .transpose()?;
        let author = build_signature(
            options.author.as_ref(),
            op,
            amend_author.or_else(|| {
                fallback_identity
                    .as_ref()
                    .map(|value| owned_signature_unchecked(value))
            }),
        )?;
        let committer = build_signature(
            options.committer.as_ref(),
            op,
            fallback_identity.map(|value| owned_signature_unchecked(&value)),
        )?;

        let oid = if options.amend {
            previous
                .as_ref()
                .expect("amend previous commit validated")
                .amend(
                    Some("HEAD"),
                    Some(&author),
                    Some(&committer),
                    None,
                    Some(message),
                    Some(&tree),
                )
                .map_err(|err| repo_error(op, err))?
        } else {
            let parents = previous.into_iter().collect::<Vec<_>>();
            let parent_refs = parents.iter().collect::<Vec<_>>();
            self.repo
                .commit(
                    Some("HEAD"),
                    &author,
                    &committer,
                    message,
                    &tree,
                    &parent_refs,
                )
                .map_err(|err| repo_error(op, err))?
        };
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
        let op = "ptool.git.Repo:switch(branch)";
        if branch.is_empty() || !git2::Reference::is_valid_name(&format!("refs/heads/{branch}")) {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:switch(branch) received an invalid branch name",
            )
            .with_op(op));
        }
        if options.orphan && !options.create {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:switch orphan = true requires create = true",
            )
            .with_op(op));
        }
        if options.orphan && options.start_point.is_some() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:switch orphan cannot be combined with start_point",
            )
            .with_op(op));
        }

        let head_name = format!("refs/heads/{branch}");
        if options.orphan {
            if self.repo.find_branch(branch, BranchType::Local).is_ok() && !options.force {
                return Err(Error::new(
                    ErrorKind::InvalidArgs,
                    "ptool.git.Repo:switch refuses to overwrite an existing branch without force = true",
                )
                .with_op(op));
            }
            if options.force {
                let _ = self
                    .repo
                    .find_reference(&head_name)
                    .and_then(|mut reference| reference.delete());
            }
            self.repo
                .set_head(&head_name)
                .map_err(|err| repo_error(op, err))?;
            let mut index = self.repo.index().map_err(|err| repo_error(op, err))?;
            index.clear().map_err(|err| repo_error(op, err))?;
            index.write().map_err(|err| repo_error(op, err))?;
            let mut builder = CheckoutBuilder::new();
            builder.force().remove_untracked(true);
            self.repo
                .checkout_index(Some(&mut index), Some(&mut builder))
                .map_err(|err| repo_error(op, err))?;
        } else {
            if options.create {
                self.branch_create(
                    branch,
                    super::GitBranchCreateOptions {
                        start_point: options.start_point.clone(),
                        force: options.force,
                        checkout: false,
                        upstream: options.track.clone(),
                    },
                )?;
            } else {
                self.repo
                    .find_branch(branch, BranchType::Local)
                    .map_err(|err| repo_error(op, err))?;
                if let Some(upstream) = options.track.as_deref() {
                    self.branch_set_upstream(branch, Some(upstream))?;
                }
            }

            let target = self
                .repo
                .revparse_single(&head_name)
                .map_err(|err| repo_error(op, err))?;
            let mut builder = CheckoutBuilder::new();
            if options.force {
                builder.force();
            }
            self.repo
                .checkout_tree(&target, Some(&mut builder))
                .map_err(|err| repo_error(op, err))?;
            self.repo
                .set_head(&head_name)
                .map_err(|err| repo_error(op, err))?;
        }
        Ok(())
    }
}

fn build_head_info(repo: &Repository, op: &str) -> Result<GitHeadInfo> {
    let detached = repo.head_detached().map_err(|err| repo_error(op, err))?;
    let head_result = repo.head();
    let unborn = matches!(
        head_result.as_ref(),
        Err(err) if err.code() == git2::ErrorCode::UnbornBranch
    );
    let head = match head_result {
        Ok(head) => Some(head),
        Err(err) if err.code() == git2::ErrorCode::UnbornBranch => None,
        Err(err) => return Err(repo_error(op, err)),
    };

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

pub(super) fn current_branch_name(repo: &Repository, op: &str) -> Result<Option<String>> {
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
        return match (input.time_seconds, input.offset_minutes) {
            (Some(seconds), Some(offset)) => {
                Signature::new(&input.name, &input.email, &git2::Time::new(seconds, offset))
                    .map_err(|err| repo_error(op, err))
            }
            (None, None) => {
                Signature::now(&input.name, &input.email).map_err(|err| repo_error(op, err))
            }
            _ => Err(Error::new(
                ErrorKind::InvalidArgs,
                format!("{op} signature time_seconds and offset_minutes must be provided together"),
            )
            .with_op(op)),
        };
    }

    fallback.ok_or_else(|| {
        Error::new(
            ErrorKind::Git,
            format!("{op} failed: git user identity is not configured"),
        )
        .with_op(op)
    })
}

fn owned_signature(signature: &Signature<'_>, op: &str) -> Result<Signature<'static>> {
    let name = signature.name().ok_or_else(|| {
        Error::new(
            ErrorKind::Git,
            format!("{op} failed: signature name is not UTF-8"),
        )
        .with_op(op)
    })?;
    let email = signature.email().ok_or_else(|| {
        Error::new(
            ErrorKind::Git,
            format!("{op} failed: signature email is not UTF-8"),
        )
        .with_op(op)
    })?;
    Signature::new(name, email, &signature.when()).map_err(|err| repo_error(op, err))
}

fn owned_signature_unchecked(signature: &Signature<'_>) -> Signature<'static> {
    Signature::new(
        signature.name().unwrap_or_default(),
        signature.email().unwrap_or_default(),
        &signature.when(),
    )
    .expect("repository signature was already validated by git2")
}
