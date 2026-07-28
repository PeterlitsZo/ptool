use super::{
    GitRepository, GitSubmoduleInfo, GitSubmoduleInitOptions, GitSubmoduleSyncOptions,
    GitSubmoduleUpdateOptions, oid_to_string, path_to_string, repo_error,
};
use crate::{Error, ErrorKind, Result};
use git2::{FetchOptions, Submodule, SubmoduleUpdateOptions};

use super::remote::build_remote_callbacks;

impl GitRepository {
    pub fn submodules(&self) -> Result<Vec<GitSubmoduleInfo>> {
        let op = "ptool.git.Repo:submodules()";
        let mut result = self
            .repo
            .submodules()
            .map_err(|err| repo_error(op, err))?
            .iter()
            .map(submodule_info)
            .collect::<Vec<_>>();
        result.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(result)
    }

    pub fn submodule_init(
        &self,
        name: Option<&str>,
        options: GitSubmoduleInitOptions,
    ) -> Result<()> {
        let op = "ptool.git.Repo:submodule_init(name?, options?)";
        for mut submodule in selected_submodules(&self.repo, name, op)? {
            submodule
                .init(options.overwrite)
                .map_err(|err| repo_error(op, err))?;
            if options.recursive
                && let Ok(repo) = submodule.open()
            {
                init_submodules_recursive(&repo, options.overwrite, op)?;
            }
        }
        Ok(())
    }

    pub fn submodule_update(
        &self,
        name: Option<&str>,
        options: GitSubmoduleUpdateOptions,
    ) -> Result<()> {
        let op = "ptool.git.Repo:submodule_update(name?, options?)";
        for mut submodule in selected_submodules(&self.repo, name, op)? {
            update_one(&mut submodule, &options, op)?;
            if options.recursive {
                let repo = submodule.open().map_err(|err| repo_error(op, err))?;
                update_submodules_recursive(&repo, &options, op)?;
            }
        }
        Ok(())
    }

    pub fn submodule_sync(
        &self,
        name: Option<&str>,
        options: GitSubmoduleSyncOptions,
    ) -> Result<()> {
        let op = "ptool.git.Repo:submodule_sync(name?, options?)";
        for mut submodule in selected_submodules(&self.repo, name, op)? {
            submodule.sync().map_err(|err| repo_error(op, err))?;
            if options.recursive
                && let Ok(repo) = submodule.open()
            {
                sync_submodules_recursive(&repo, op)?;
            }
        }
        Ok(())
    }
}

fn selected_submodules<'repo>(
    repo: &'repo git2::Repository,
    name: Option<&str>,
    op: &str,
) -> Result<Vec<Submodule<'repo>>> {
    if matches!(name, Some("")) {
        return Err(
            Error::new(ErrorKind::InvalidArgs, "submodule name must not be empty").with_op(op),
        );
    }
    let submodules = repo.submodules().map_err(|err| repo_error(op, err))?;
    if let Some(name) = name {
        let selected = submodules
            .into_iter()
            .find(|submodule| submodule.name() == Some(name))
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidArgs,
                    format!("submodule `{name}` was not found"),
                )
                .with_op(op)
            })?;
        Ok(vec![selected])
    } else {
        Ok(submodules)
    }
}

fn update_one(
    submodule: &mut Submodule<'_>,
    options: &GitSubmoduleUpdateOptions,
    op: &str,
) -> Result<()> {
    let mut fetch = FetchOptions::new();
    fetch.remote_callbacks(build_remote_callbacks(options.auth.clone()));
    let mut raw_options = SubmoduleUpdateOptions::new();
    raw_options.allow_fetch(options.allow_fetch).fetch(fetch);
    submodule
        .update(options.init, Some(&mut raw_options))
        .map_err(|err| repo_error(op, err))
}

fn init_submodules_recursive(repo: &git2::Repository, overwrite: bool, op: &str) -> Result<()> {
    for mut submodule in repo.submodules().map_err(|err| repo_error(op, err))? {
        submodule
            .init(overwrite)
            .map_err(|err| repo_error(op, err))?;
        if let Ok(child) = submodule.open() {
            init_submodules_recursive(&child, overwrite, op)?;
        }
    }
    Ok(())
}

fn update_submodules_recursive(
    repo: &git2::Repository,
    options: &GitSubmoduleUpdateOptions,
    op: &str,
) -> Result<()> {
    for mut submodule in repo.submodules().map_err(|err| repo_error(op, err))? {
        update_one(&mut submodule, options, op)?;
        let child = submodule.open().map_err(|err| repo_error(op, err))?;
        update_submodules_recursive(&child, options, op)?;
    }
    Ok(())
}

fn sync_submodules_recursive(repo: &git2::Repository, op: &str) -> Result<()> {
    for mut submodule in repo.submodules().map_err(|err| repo_error(op, err))? {
        submodule.sync().map_err(|err| repo_error(op, err))?;
        if let Ok(child) = submodule.open() {
            sync_submodules_recursive(&child, op)?;
        }
    }
    Ok(())
}

fn submodule_info(submodule: &Submodule<'_>) -> GitSubmoduleInfo {
    GitSubmoduleInfo {
        name: submodule
            .name()
            .map(str::to_string)
            .unwrap_or_else(|| String::from_utf8_lossy(submodule.name_bytes()).to_string()),
        path: path_to_string(submodule.path()),
        url: submodule.url().map(str::to_string),
        branch: submodule.branch().map(str::to_string),
        head_oid: submodule.head_id().map(oid_to_string),
        index_oid: submodule.index_id().map(oid_to_string),
        workdir_oid: submodule.workdir_id().map(oid_to_string),
    }
}
