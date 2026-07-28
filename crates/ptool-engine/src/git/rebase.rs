use super::{
    GitConflictEntry, GitFastForwardMode, GitIntegrateResult, GitMergeOptions,
    GitRebaseContinueOptions, GitRebaseOptions, GitRebaseResult, GitRepository, GitSignature,
    oid_to_string, repo_error,
};
use crate::{Error, ErrorKind, Result};
use git2::{AnnotatedCommit, Rebase, RepositoryState};

use super::integrate::{
    collect_conflicts, integrate_result, integration_signature, record_orig_head_if_present,
    require_clean_repository,
};

impl GitRepository {
    pub fn rebase(&self, options: GitRebaseOptions) -> Result<GitRebaseResult> {
        let op = "ptool.git.Repo:rebase(options)";
        require_clean_repository(&self.repo, op)?;
        if options.upstream.is_empty() || options.branch.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "rebase requires non-empty upstream and branch revisions",
            )
            .with_op(op));
        }
        let branch = self.rebase_annotated(&options.branch, op)?;
        let upstream = self.rebase_annotated(&options.upstream, op)?;
        let onto_rev = options.onto.as_deref().unwrap_or(&options.upstream);
        if onto_rev.is_empty() {
            return Err(
                Error::new(ErrorKind::InvalidArgs, "rebase onto must not be empty").with_op(op),
            );
        }
        let onto = self.rebase_annotated(onto_rev, op)?;
        record_orig_head_if_present(&self.repo, op)?;
        let signature = integration_signature(&self.repo, options.signature.as_ref(), op)?;
        let mut rebase = self
            .repo
            .rebase(Some(&branch), Some(&upstream), Some(&onto), None)
            .map_err(|err| repo_error(op, err))?;
        process_rebase(&self.repo, &mut rebase, &signature, false, op)
    }

    pub fn rebase_continue(&self, options: GitRebaseContinueOptions) -> Result<GitRebaseResult> {
        let op = "ptool.git.Repo:rebase_continue(options?)";
        if !matches!(
            self.repo.state(),
            RepositoryState::Rebase
                | RepositoryState::RebaseMerge
                | RepositoryState::RebaseInteractive
        ) {
            return Err(Error::new(ErrorKind::InvalidArgs, "no rebase is in progress").with_op(op));
        }
        let index = self.repo.index().map_err(|err| repo_error(op, err))?;
        let conflicts = collect_conflicts(&index, op)?;
        if !conflicts.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "rebase continue requires all index conflicts to be resolved",
            )
            .with_op(op));
        }
        let signature = integration_signature(&self.repo, options.signature.as_ref(), op)?;
        let mut rebase = self
            .repo
            .open_rebase(None)
            .map_err(|err| repo_error(op, err))?;
        if rebase.operation_current().is_none() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "rebase has no current operation to continue",
            )
            .with_op(op));
        }
        process_rebase(&self.repo, &mut rebase, &signature, true, op)
    }

    pub fn rebase_abort(&self) -> Result<()> {
        let op = "ptool.git.Repo:rebase_abort(options?)";
        if !matches!(
            self.repo.state(),
            RepositoryState::Rebase
                | RepositoryState::RebaseMerge
                | RepositoryState::RebaseInteractive
        ) {
            return Err(Error::new(ErrorKind::InvalidArgs, "no rebase is in progress").with_op(op));
        }
        let mut rebase = self
            .repo
            .open_rebase(None)
            .map_err(|err| repo_error(op, err))?;
        rebase.abort().map_err(|err| repo_error(op, err))
    }

    pub(super) fn pull_rebase(
        &self,
        remote_rev: &str,
        signature: Option<GitSignature>,
    ) -> Result<GitIntegrateResult> {
        let op = "ptool.git.Repo:pull(remote?, branch?, options?)";
        let annotated = self.rebase_annotated(remote_rev, op)?;
        let (analysis, _) = self
            .repo
            .merge_analysis(&[&annotated])
            .map_err(|err| repo_error(op, err))?;
        if analysis.is_up_to_date() {
            return Ok(integrate_result(
                "up_to_date",
                Some(annotated.id()),
                Vec::new(),
            ));
        }
        if analysis.is_fast_forward() || analysis.is_unborn() {
            return self.merge(
                remote_rev,
                GitMergeOptions {
                    ff: GitFastForwardMode::Only,
                    signature,
                    message: None,
                },
            );
        }
        let result = self.rebase(GitRebaseOptions {
            upstream: remote_rev.to_string(),
            onto: Some(remote_rev.to_string()),
            branch: "HEAD".to_string(),
            signature,
        })?;
        Ok(GitIntegrateResult {
            outcome: if result.outcome == "conflicted" {
                "conflicted".to_string()
            } else {
                "merged".to_string()
            },
            oid: result.oid,
            conflicts: result.conflicts,
        })
    }

    fn rebase_annotated<'repo>(&'repo self, rev: &str, op: &str) -> Result<AnnotatedCommit<'repo>> {
        let object = self
            .repo
            .revparse_single(rev)
            .map_err(|err| repo_error(op, err))?;
        self.repo
            .find_annotated_commit(object.id())
            .map_err(|err| repo_error(op, err))
    }
}

fn process_rebase(
    repo: &git2::Repository,
    rebase: &mut Rebase<'_>,
    signature: &git2::Signature<'_>,
    commit_current: bool,
    op: &str,
) -> Result<GitRebaseResult> {
    let total = rebase.len();
    let mut last_oid = None;
    if commit_current {
        last_oid = Some(
            rebase
                .commit(None, signature, None)
                .map_err(|err| repo_error(op, err))?,
        );
    }
    loop {
        let Some(operation) = rebase.next() else {
            rebase
                .finish(Some(signature))
                .map_err(|err| repo_error(op, err))?;
            let oid = match repo.head() {
                Ok(head) => head.target().or(last_oid),
                Err(_) => last_oid,
            };
            return Ok(GitRebaseResult {
                outcome: "rebased".to_string(),
                oid: oid.map(oid_to_string),
                conflicts: Vec::new(),
                current: None,
                total,
            });
        };
        operation.map_err(|err| repo_error(op, err))?;
        let index = repo.index().map_err(|err| repo_error(op, err))?;
        let conflicts = collect_conflicts(&index, op)?;
        if !conflicts.is_empty() {
            return Ok(conflicted_result(rebase, conflicts, total));
        }
        last_oid = Some(
            rebase
                .commit(None, signature, None)
                .map_err(|err| repo_error(op, err))?,
        );
    }
}

fn conflicted_result(
    rebase: &mut Rebase<'_>,
    conflicts: Vec<GitConflictEntry>,
    total: usize,
) -> GitRebaseResult {
    GitRebaseResult {
        outcome: "conflicted".to_string(),
        oid: None,
        conflicts,
        current: rebase.operation_current(),
        total,
    }
}
