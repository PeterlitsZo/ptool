use super::{
    GitApplyCommitOptions, GitConflictEntry, GitFastForwardMode, GitFetchOptions,
    GitIntegrateResult, GitMergeOptions, GitPullOptions, GitPullStrategy, GitRepository,
    GitSignature, oid_to_string, repo_error,
};
use crate::{Error, ErrorKind, Result};
use git2::{
    AnnotatedCommit, CherrypickOptions, Commit, ErrorCode, Index, MergeAnalysis, Oid,
    RepositoryState, ResetType, RevertOptions, Signature, build::CheckoutBuilder,
};

impl GitRepository {
    pub fn state(&self) -> String {
        repository_state_name(self.repo.state()).to_string()
    }

    pub fn conflicts(&self) -> Result<Vec<GitConflictEntry>> {
        let op = "ptool.git.Repo:conflicts()";
        let index = self.repo.index().map_err(|err| repo_error(op, err))?;
        collect_conflicts(&index, op)
    }

    pub fn merge_analysis(&self, rev: &str) -> Result<String> {
        let op = "ptool.git.Repo:merge_analysis(rev)";
        let annotated = self.annotated_commit(rev, op)?;
        let (analysis, _) = self
            .repo
            .merge_analysis(&[&annotated])
            .map_err(|err| repo_error(op, err))?;
        Ok(analysis_name(analysis).to_string())
    }

    pub fn merge(&self, rev: &str, options: GitMergeOptions) -> Result<GitIntegrateResult> {
        let op = "ptool.git.Repo:merge(rev, options?)";
        if rev.is_empty() {
            return Err(
                Error::new(ErrorKind::InvalidArgs, "merge revision must not be empty").with_op(op),
            );
        }
        require_clean_repository(&self.repo, op)?;
        let annotated = self.annotated_commit(rev, op)?;
        self.merge_annotated(&annotated, rev, options, op)
    }

    pub fn merge_abort(&self) -> Result<()> {
        abort_integrate(&self.repo, "ptool.git.Repo:merge_abort(options?)")
    }

    pub fn cherry_pick(
        &self,
        rev: &str,
        options: GitApplyCommitOptions,
    ) -> Result<GitIntegrateResult> {
        self.apply_commit_operation(rev, options, ApplyOperation::CherryPick)
    }

    pub fn cherry_pick_abort(&self) -> Result<()> {
        abort_integrate(&self.repo, "ptool.git.Repo:cherry_pick_abort(options?)")
    }

    pub fn revert(&self, rev: &str, options: GitApplyCommitOptions) -> Result<GitIntegrateResult> {
        self.apply_commit_operation(rev, options, ApplyOperation::Revert)
    }

    pub fn revert_abort(&self) -> Result<()> {
        abort_integrate(&self.repo, "ptool.git.Repo:revert_abort(options?)")
    }

    pub fn pull(
        &self,
        remote: Option<&str>,
        branch: Option<&str>,
        options: GitPullOptions,
    ) -> Result<GitIntegrateResult> {
        let op = "ptool.git.Repo:pull(remote?, branch?, options?)";
        require_clean_repository(&self.repo, op)?;
        let remote = remote.unwrap_or("origin");
        if remote.is_empty() {
            return Err(
                Error::new(ErrorKind::InvalidArgs, "pull remote must not be empty").with_op(op),
            );
        }
        let branch = match branch {
            Some(branch) if !branch.is_empty() => branch.to_string(),
            Some(_) => {
                return Err(
                    Error::new(ErrorKind::InvalidArgs, "pull branch must not be empty").with_op(op),
                );
            }
            None => super::repository::current_branch_name(&self.repo, op)?.ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidArgs,
                    "pull requires an explicit branch when HEAD is detached or unborn",
                )
                .with_op(op)
            })?,
        };
        self.fetch(
            Some(remote),
            GitFetchOptions {
                refspecs: Vec::new(),
                auth: options.auth,
                depth: options.depth,
                prune: options.prune,
                tags: options.tags,
                update_fetchhead: options.update_fetchhead,
            },
        )?;
        let remote_rev = format!("refs/remotes/{remote}/{branch}");
        let annotated = self.annotated_commit(&remote_rev, op)?;
        match options.strategy {
            GitPullStrategy::FastForwardOnly => self.merge_annotated(
                &annotated,
                &remote_rev,
                GitMergeOptions {
                    ff: GitFastForwardMode::Only,
                    signature: options.signature,
                    message: options.message,
                },
                op,
            ),
            GitPullStrategy::Merge => self.merge_annotated(
                &annotated,
                &remote_rev,
                GitMergeOptions {
                    ff: GitFastForwardMode::Allow,
                    signature: options.signature,
                    message: options.message,
                },
                op,
            ),
            GitPullStrategy::Rebase => self.pull_rebase(&remote_rev, options.signature),
        }
    }

    fn apply_commit_operation(
        &self,
        rev: &str,
        options: GitApplyCommitOptions,
        operation: ApplyOperation,
    ) -> Result<GitIntegrateResult> {
        let op = operation.op();
        if rev.is_empty() {
            return Err(
                Error::new(ErrorKind::InvalidArgs, "revision must not be empty").with_op(op),
            );
        }
        require_clean_repository(&self.repo, op)?;
        let object = self
            .repo
            .revparse_single(rev)
            .map_err(|err| repo_error(op, err))?;
        let commit = object.peel_to_commit().map_err(|err| repo_error(op, err))?;
        if commit.parent_count() > 1 && options.mainline.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "applying a merge commit requires `mainline`",
            )
            .with_op(op));
        }
        let original = head_commit(&self.repo, op)?;
        record_orig_head(&self.repo, original.id(), op)?;
        let mut checkout = CheckoutBuilder::new();
        checkout
            .safe()
            .allow_conflicts(true)
            .conflict_style_merge(true);
        match operation {
            ApplyOperation::CherryPick => {
                let mut raw_options = CherrypickOptions::new();
                raw_options.checkout_builder(checkout);
                if let Some(mainline) = options.mainline {
                    raw_options.mainline(mainline);
                }
                self.repo
                    .cherrypick(&commit, Some(&mut raw_options))
                    .map_err(|err| repo_error(op, err))?;
            }
            ApplyOperation::Revert => {
                let mut raw_options = RevertOptions::new();
                raw_options.checkout_builder(checkout);
                if let Some(mainline) = options.mainline {
                    raw_options.mainline(mainline);
                }
                self.repo
                    .revert(&commit, Some(&mut raw_options))
                    .map_err(|err| repo_error(op, err))?;
            }
        }
        let mut index = self.repo.index().map_err(|err| repo_error(op, err))?;
        let conflicts = collect_conflicts(&index, op)?;
        if !conflicts.is_empty() {
            return Ok(integrate_result("conflicted", None, conflicts));
        }
        if !options.commit {
            self.repo
                .cleanup_state()
                .map_err(|err| repo_error(op, err))?;
            return Ok(integrate_result("merged", None, Vec::new()));
        }
        let tree_oid = index.write_tree().map_err(|err| repo_error(op, err))?;
        let tree = self
            .repo
            .find_tree(tree_oid)
            .map_err(|err| repo_error(op, err))?;
        let signature = integration_signature(&self.repo, options.signature.as_ref(), op)?;
        let default_message = match operation {
            ApplyOperation::CherryPick => commit
                .message()
                .map(str::to_string)
                .unwrap_or_else(|| format!("Cherry-pick {rev}")),
            ApplyOperation::Revert => {
                format!("Revert \"{}\"", commit.summary().unwrap_or(rev))
            }
        };
        let message = options.message.unwrap_or(default_message);
        let oid = self
            .repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                &message,
                &tree,
                &[&original],
            )
            .map_err(|err| repo_error(op, err))?;
        self.repo
            .cleanup_state()
            .map_err(|err| repo_error(op, err))?;
        Ok(integrate_result("merged", Some(oid), Vec::new()))
    }

    fn annotated_commit<'repo>(&'repo self, rev: &str, op: &str) -> Result<AnnotatedCommit<'repo>> {
        let object = self
            .repo
            .revparse_single(rev)
            .map_err(|err| repo_error(op, err))?;
        self.repo
            .find_annotated_commit(object.id())
            .map_err(|err| repo_error(op, err))
    }

    fn merge_annotated(
        &self,
        annotated: &AnnotatedCommit<'_>,
        revision_label: &str,
        options: GitMergeOptions,
        op: &str,
    ) -> Result<GitIntegrateResult> {
        let (analysis, _) = self
            .repo
            .merge_analysis(&[annotated])
            .map_err(|err| repo_error(op, err))?;
        let target_oid = annotated.id();
        if analysis.is_up_to_date() {
            return Ok(integrate_result("up_to_date", Some(target_oid), Vec::new()));
        }
        if analysis.is_unborn() {
            record_orig_head_if_present(&self.repo, op)?;
            fast_forward(&self.repo, target_oid, op)?;
            return Ok(integrate_result(
                "fast_forward",
                Some(target_oid),
                Vec::new(),
            ));
        }
        if analysis.is_fast_forward() && options.ff != GitFastForwardMode::Never {
            record_orig_head_if_present(&self.repo, op)?;
            fast_forward(&self.repo, target_oid, op)?;
            return Ok(integrate_result(
                "fast_forward",
                Some(target_oid),
                Vec::new(),
            ));
        }
        if options.ff == GitFastForwardMode::Only {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "merge requires a non-fast-forward integration but ff = `only`",
            )
            .with_op(op));
        }
        if !analysis.is_normal() && !analysis.is_fast_forward() {
            return Err(
                Error::new(ErrorKind::Git, "libgit2 reported no merge strategy").with_op(op),
            );
        }

        let original = head_commit(&self.repo, op)?;
        record_orig_head(&self.repo, original.id(), op)?;
        let mut checkout = CheckoutBuilder::new();
        checkout
            .safe()
            .allow_conflicts(true)
            .conflict_style_merge(true);
        self.repo
            .merge(&[annotated], None, Some(&mut checkout))
            .map_err(|err| repo_error(op, err))?;
        let mut index = self.repo.index().map_err(|err| repo_error(op, err))?;
        let conflicts = collect_conflicts(&index, op)?;
        if !conflicts.is_empty() {
            return Ok(integrate_result("conflicted", None, conflicts));
        }

        let tree_oid = index.write_tree().map_err(|err| repo_error(op, err))?;
        let tree = self
            .repo
            .find_tree(tree_oid)
            .map_err(|err| repo_error(op, err))?;
        let target = self
            .repo
            .find_commit(target_oid)
            .map_err(|err| repo_error(op, err))?;
        let signature = integration_signature(&self.repo, options.signature.as_ref(), op)?;
        let message = options
            .message
            .unwrap_or_else(|| format!("Merge {revision_label}"));
        let oid = self
            .repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                &message,
                &tree,
                &[&original, &target],
            )
            .map_err(|err| repo_error(op, err))?;
        self.repo
            .cleanup_state()
            .map_err(|err| repo_error(op, err))?;
        Ok(integrate_result("merged", Some(oid), Vec::new()))
    }
}

pub(super) fn repository_state_name(state: RepositoryState) -> &'static str {
    match state {
        RepositoryState::Clean => "clean",
        RepositoryState::Merge => "merge",
        RepositoryState::Revert => "revert",
        RepositoryState::RevertSequence => "revert_sequence",
        RepositoryState::CherryPick => "cherry_pick",
        RepositoryState::CherryPickSequence => "cherry_pick_sequence",
        RepositoryState::Bisect => "bisect",
        RepositoryState::Rebase => "rebase",
        RepositoryState::RebaseInteractive => "rebase_interactive",
        RepositoryState::RebaseMerge => "rebase_merge",
        RepositoryState::ApplyMailbox => "apply_mailbox",
        RepositoryState::ApplyMailboxOrRebase => "apply_mailbox_or_rebase",
    }
}

pub(super) fn require_clean_repository(repo: &git2::Repository, op: &str) -> Result<()> {
    if repo.state() != RepositoryState::Clean {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!(
                "repository is in `{}` state",
                repository_state_name(repo.state())
            ),
        )
        .with_op(op));
    }
    let mut options = git2::StatusOptions::new();
    options
        .include_untracked(true)
        .include_ignored(false)
        .recurse_untracked_dirs(true);
    let statuses = repo
        .statuses(Some(&mut options))
        .map_err(|err| repo_error(op, err))?;
    if !statuses.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "integration requires a clean index and worktree",
        )
        .with_op(op));
    }
    Ok(())
}

pub(super) fn collect_conflicts(index: &Index, op: &str) -> Result<Vec<GitConflictEntry>> {
    if !index.has_conflicts() {
        return Ok(Vec::new());
    }
    let conflicts = index.conflicts().map_err(|err| repo_error(op, err))?;
    let mut result = Vec::new();
    for conflict in conflicts {
        let conflict = conflict.map_err(|err| repo_error(op, err))?;
        let path = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
            .map(|entry| String::from_utf8_lossy(&entry.path).to_string())
            .unwrap_or_default();
        result.push(GitConflictEntry {
            path,
            ancestor: conflict.ancestor.is_some(),
            ours: conflict.our.is_some(),
            theirs: conflict.their.is_some(),
        });
    }
    result.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(result)
}

pub(super) fn integrate_result(
    outcome: &str,
    oid: Option<Oid>,
    conflicts: Vec<GitConflictEntry>,
) -> GitIntegrateResult {
    GitIntegrateResult {
        outcome: outcome.to_string(),
        oid: oid.map(oid_to_string),
        conflicts,
    }
}

pub(super) fn integration_signature(
    repo: &git2::Repository,
    input: Option<&GitSignature>,
    op: &str,
) -> Result<Signature<'static>> {
    if let Some(input) = input {
        if input.name.is_empty() || input.email.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "signature requires non-empty name and email",
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
                "signature time_seconds and offset_minutes must be provided together",
            )
            .with_op(op)),
        };
    }
    let signature = repo.signature().map_err(|err| repo_error(op, err))?;
    let name = signature.name().ok_or_else(|| {
        Error::new(ErrorKind::Git, "configured signature name is not UTF-8").with_op(op)
    })?;
    let email = signature.email().ok_or_else(|| {
        Error::new(ErrorKind::Git, "configured signature email is not UTF-8").with_op(op)
    })?;
    Signature::new(name, email, &signature.when()).map_err(|err| repo_error(op, err))
}

pub(super) fn record_orig_head_if_present(repo: &git2::Repository, op: &str) -> Result<()> {
    match repo.head() {
        Ok(head) => {
            let commit = head.peel_to_commit().map_err(|err| repo_error(op, err))?;
            record_orig_head(repo, commit.id(), op)
        }
        Err(err) if err.code() == ErrorCode::UnbornBranch => Ok(()),
        Err(err) => Err(repo_error(op, err)),
    }
}

pub(super) fn record_orig_head(repo: &git2::Repository, oid: Oid, op: &str) -> Result<()> {
    repo.reference("ORIG_HEAD", oid, true, "ptool integration start")
        .map(|_| ())
        .map_err(|err| repo_error(op, err))
}

pub(super) fn abort_integrate(repo: &git2::Repository, op: &str) -> Result<()> {
    let reference = repo
        .find_reference("ORIG_HEAD")
        .map_err(|err| repo_error(op, err))?;
    let oid = reference.target().ok_or_else(|| {
        Error::new(
            ErrorKind::Git,
            "ORIG_HEAD does not point to a direct object",
        )
        .with_op(op)
    })?;
    let object = repo
        .find_object(oid, None)
        .map_err(|err| repo_error(op, err))?;
    let mut checkout = CheckoutBuilder::new();
    checkout
        .force()
        .recreate_missing(true)
        .remove_untracked(true);
    repo.reset(&object, ResetType::Hard, Some(&mut checkout))
        .map_err(|err| repo_error(op, err))?;
    repo.cleanup_state().map_err(|err| repo_error(op, err))
}

fn fast_forward(repo: &git2::Repository, target_oid: Oid, op: &str) -> Result<()> {
    let target = repo
        .find_object(target_oid, None)
        .map_err(|err| repo_error(op, err))?;
    match repo.head() {
        Ok(head) if head.is_branch() => {
            let name = head.name().ok_or_else(|| {
                Error::new(ErrorKind::Git, "HEAD branch name is not UTF-8").with_op(op)
            })?;
            repo.reference(name, target_oid, true, "ptool fast-forward")
                .map_err(|err| repo_error(op, err))?;
            repo.set_head(name).map_err(|err| repo_error(op, err))?;
        }
        Ok(_) => repo
            .set_head_detached(target_oid)
            .map_err(|err| repo_error(op, err))?,
        Err(err) if err.code() == ErrorCode::UnbornBranch => {
            let head = repo
                .find_reference("HEAD")
                .map_err(|err| repo_error(op, err))?;
            let name = head.symbolic_target().ok_or_else(|| {
                Error::new(ErrorKind::Git, "unborn HEAD is not symbolic").with_op(op)
            })?;
            repo.reference(name, target_oid, true, "ptool initialize branch")
                .map_err(|err| repo_error(op, err))?;
            repo.set_head(name).map_err(|err| repo_error(op, err))?;
        }
        Err(err) => return Err(repo_error(op, err)),
    }
    let mut checkout = CheckoutBuilder::new();
    checkout.force().recreate_missing(true);
    repo.checkout_tree(&target, Some(&mut checkout))
        .map_err(|err| repo_error(op, err))
}

fn head_commit<'repo>(repo: &'repo git2::Repository, op: &str) -> Result<Commit<'repo>> {
    repo.head()
        .and_then(|head| head.peel_to_commit())
        .map_err(|err| repo_error(op, err))
}

fn analysis_name(analysis: MergeAnalysis) -> &'static str {
    if analysis.is_up_to_date() {
        "up_to_date"
    } else if analysis.is_fast_forward() {
        "fast_forward"
    } else if analysis.is_normal() {
        "normal"
    } else if analysis.is_unborn() {
        "unborn"
    } else {
        "none"
    }
}

#[derive(Clone, Copy)]
enum ApplyOperation {
    CherryPick,
    Revert,
}

impl ApplyOperation {
    fn op(self) -> &'static str {
        match self {
            Self::CherryPick => "ptool.git.Repo:cherry_pick(rev, options?)",
            Self::Revert => "ptool.git.Repo:revert(rev, options?)",
        }
    }
}
