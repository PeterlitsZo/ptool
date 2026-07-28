use super::{
    GitIntegrateResult, GitRepository, GitStashApplyOptions, GitStashInfo, GitStashSaveOptions,
    oid_to_string, repo_error,
};
use crate::{Error, ErrorKind, Result};
use git2::{ErrorCode, StashApplyOptions, StashFlags};

use super::integrate::{collect_conflicts, integrate_result, integration_signature};

impl GitRepository {
    pub fn stash_save(
        &mut self,
        message: Option<&str>,
        options: GitStashSaveOptions,
    ) -> Result<String> {
        let op = "ptool.git.Repo:stash_save(message?, options?)";
        if matches!(message, Some("")) {
            return Err(
                Error::new(ErrorKind::InvalidArgs, "stash message must not be empty").with_op(op),
            );
        }
        let signature = integration_signature(&self.repo, options.signature.as_ref(), op)?;
        let mut flags = StashFlags::empty();
        if options.keep_index {
            flags.insert(StashFlags::KEEP_INDEX);
        }
        if options.include_untracked || options.include_ignored {
            flags.insert(StashFlags::INCLUDE_UNTRACKED);
        }
        if options.include_ignored {
            flags.insert(StashFlags::INCLUDE_IGNORED);
        }
        self.repo
            .stash_save2(&signature, message, Some(flags))
            .map(oid_to_string)
            .map_err(|err| repo_error(op, err))
    }

    pub fn stashes(&mut self) -> Result<Vec<GitStashInfo>> {
        let op = "ptool.git.Repo:stashes()";
        let mut result = Vec::new();
        self.repo
            .stash_foreach(|index, message, oid| {
                result.push(GitStashInfo {
                    index,
                    message: message.to_string(),
                    oid: oid_to_string(*oid),
                });
                true
            })
            .map_err(|err| repo_error(op, err))?;
        Ok(result)
    }

    pub fn stash_apply(
        &mut self,
        index: usize,
        options: GitStashApplyOptions,
    ) -> Result<GitIntegrateResult> {
        self.apply_stash(index, options, false)
    }

    pub fn stash_pop(
        &mut self,
        index: usize,
        options: GitStashApplyOptions,
    ) -> Result<GitIntegrateResult> {
        self.apply_stash(index, options, true)
    }

    pub fn stash_drop(&mut self, index: usize) -> Result<()> {
        let op = "ptool.git.Repo:stash_drop(index?, options?)";
        self.validate_stash_index(index, op)?;
        self.repo
            .stash_drop(index)
            .map_err(|err| repo_error(op, err))
    }

    fn apply_stash(
        &mut self,
        index: usize,
        options: GitStashApplyOptions,
        pop: bool,
    ) -> Result<GitIntegrateResult> {
        let op = if pop {
            "ptool.git.Repo:stash_pop(index?, options?)"
        } else {
            "ptool.git.Repo:stash_apply(index?, options?)"
        };
        let oid = self.validate_stash_index(index, op)?;
        let mut apply_options = StashApplyOptions::new();
        if options.reinstate_index {
            apply_options.reinstantiate_index();
        }
        let result = if pop {
            self.repo.stash_pop(index, Some(&mut apply_options))
        } else {
            self.repo.stash_apply(index, Some(&mut apply_options))
        };
        match result {
            Ok(()) => {
                let index = self.repo.index().map_err(|err| repo_error(op, err))?;
                let conflicts = collect_conflicts(&index, op)?;
                if conflicts.is_empty() {
                    Ok(integrate_result("merged", Some(oid), Vec::new()))
                } else {
                    Ok(integrate_result("conflicted", Some(oid), conflicts))
                }
            }
            Err(err) if err.code() == ErrorCode::Conflict => {
                let index = self.repo.index().map_err(|err| repo_error(op, err))?;
                let conflicts = collect_conflicts(&index, op)?;
                Ok(integrate_result("conflicted", Some(oid), conflicts))
            }
            Err(err) => Err(repo_error(op, err)),
        }
    }

    fn validate_stash_index(&mut self, index: usize, op: &str) -> Result<git2::Oid> {
        let mut found = None;
        self.repo
            .stash_foreach(|entry_index, _, oid| {
                if entry_index == index {
                    found = Some(*oid);
                    false
                } else {
                    true
                }
            })
            .map_err(|err| repo_error(op, err))?;
        found.ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidArgs,
                format!("stash index {index} is out of range"),
            )
            .with_op(op)
        })
    }
}
