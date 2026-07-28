use super::{
    GitCloneOptions, GitFetchOptions, GitFetchStats, GitPushOptions, GitPushRejection,
    GitPushResult, GitRemoteAddOptions, GitRemoteAuth, GitRemoteInfo, GitRepository,
    GitTagDownload, repo_error, resolve_repo_path,
};
use crate::{Error, ErrorKind, Result};
use git2::{
    AutotagOption, Config, Cred, CredentialType, FetchOptions, FetchPrune, PushOptions,
    RemoteCallbacks, Repository,
    build::{CheckoutBuilder, RepoBuilder},
};
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::repository::current_branch_name;

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
    validate_depth(options.depth, "ptool.git.clone")?;
    if matches!(options.remote.as_deref(), Some("")) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.git.clone remote must not be empty",
        )
        .with_op("ptool.git.clone"));
    }

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(build_remote_callbacks(options.auth.clone()));
    if let Some(depth) = options.depth {
        fetch_options.depth(depth);
    }
    fetch_options.download_tags(raw_tag_download(options.tags));

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);
    if let Some(branch) = options.branch.as_deref() {
        builder.branch(branch);
    }
    if options.bare {
        builder.bare(true);
    }
    if !options.checkout && !options.bare {
        let mut checkout = CheckoutBuilder::new();
        checkout.dry_run();
        builder.with_checkout(checkout);
    }
    if let Some(remote_name) = options.remote {
        builder.remote_create(move |repo, _name, remote_url| repo.remote(&remote_name, remote_url));
    }

    let repo = builder
        .clone(url, &target_path)
        .map_err(|err| repo_error("ptool.git.clone", err).with_input(url.to_string()))?;
    Ok(GitRepository { repo })
}

pub(super) fn build_remote_callbacks(auth: GitRemoteAuth) -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username_from_url, allowed_types| {
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

        if let GitRemoteAuth::SshKey {
            username,
            public_key,
            private_key,
            passphrase,
        } = &auth
            && allowed_types.contains(CredentialType::SSH_KEY)
        {
            return Cred::ssh_key(
                username,
                public_key.as_deref().map(Path::new),
                Path::new(private_key),
                passphrase.as_deref(),
            );
        }

        if let GitRemoteAuth::CredentialHelper { username } = &auth
            && allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT)
        {
            let config = Config::open_default()?;
            return Cred::credential_helper(
                &config,
                url,
                username.as_deref().or(username_from_url),
            );
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

impl GitRepository {
    pub fn remotes(&self) -> Result<Vec<GitRemoteInfo>> {
        let op = "ptool.git.Repo:remotes()";
        let names = self.repo.remotes().map_err(|err| repo_error(op, err))?;
        let mut result = Vec::new();
        for name in names.iter().flatten() {
            result.push(build_remote_info(&self.repo, name, op)?);
        }
        result.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(result)
    }

    pub fn remote_info(&self, name: &str) -> Result<GitRemoteInfo> {
        let op = "ptool.git.Repo:remote(name)";
        validate_remote_name(name, op)?;
        build_remote_info(&self.repo, name, op)
    }

    pub fn remote_add(
        &self,
        name: &str,
        url: &str,
        options: GitRemoteAddOptions,
    ) -> Result<GitRemoteInfo> {
        let op = "ptool.git.Repo:remote_add(name, url, options?)";
        validate_remote_name(name, op)?;
        validate_url(url, op)?;
        self.repo
            .remote(name, url)
            .map_err(|err| repo_error(op, err))?;
        if let Some(push_url) = options.push_url.as_deref() {
            validate_url(push_url, op)?;
            self.repo
                .remote_set_pushurl(name, Some(push_url))
                .map_err(|err| repo_error(op, err))?;
        }
        build_remote_info(&self.repo, name, op)
    }

    pub fn remote_remove(&self, name: &str) -> Result<()> {
        let op = "ptool.git.Repo:remote_remove(name, options?)";
        validate_remote_name(name, op)?;
        self.repo
            .remote_delete(name)
            .map_err(|err| repo_error(op, err))
    }

    pub fn remote_rename(&self, name: &str, new_name: &str) -> Result<GitRemoteInfo> {
        let op = "ptool.git.Repo:remote_rename(name, new_name, options?)";
        validate_remote_name(name, op)?;
        validate_remote_name(new_name, op)?;
        self.repo
            .remote_rename(name, new_name)
            .map_err(|err| repo_error(op, err))?;
        build_remote_info(&self.repo, new_name, op)
    }

    pub fn remote_set_url(&self, name: &str, url: &str, push: bool) -> Result<GitRemoteInfo> {
        let op = "ptool.git.Repo:remote_set_url(name, url, options?)";
        validate_remote_name(name, op)?;
        validate_url(url, op)?;
        if push {
            self.repo
                .remote_set_pushurl(name, Some(url))
                .map_err(|err| repo_error(op, err))?;
        } else {
            self.repo
                .remote_set_url(name, url)
                .map_err(|err| repo_error(op, err))?;
        }
        build_remote_info(&self.repo, name, op)
    }

    pub fn fetch(
        &self,
        remote_name: Option<&str>,
        options: GitFetchOptions,
    ) -> Result<GitFetchStats> {
        let op = "ptool.git.Repo:fetch(remote?, options?)";
        let remote_name = remote_name.unwrap_or("origin");
        validate_remote_name(remote_name, op)?;
        validate_depth(options.depth, op)?;
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .map_err(|err| repo_error(op, err))?;

        let updated_refs = Arc::new(Mutex::new(Vec::new()));
        let callback_refs = Arc::clone(&updated_refs);
        let mut callbacks = build_remote_callbacks(options.auth);
        callbacks.update_tips(move |name, old, new| {
            if old != new
                && let Ok(mut refs) = callback_refs.lock()
            {
                refs.push(name.to_string());
            }
            true
        });
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        fetch_options.prune(if options.prune {
            FetchPrune::On
        } else {
            FetchPrune::Off
        });
        fetch_options.download_tags(raw_tag_download(options.tags));
        fetch_options.update_fetchhead(options.update_fetchhead);
        if let Some(depth) = options.depth {
            fetch_options.depth(depth);
        }

        let refspecs: Vec<&str> = options.refspecs.iter().map(String::as_str).collect();
        remote
            .fetch(&refspecs, Some(&mut fetch_options), None)
            .map_err(|err| repo_error(op, err))?;
        let stats = remote.stats();
        let mut updated_refs = updated_refs
            .lock()
            .map_err(|_| Error::new(ErrorKind::Git, "git fetch callback state was poisoned"))?
            .clone();
        updated_refs.sort();
        updated_refs.dedup();

        Ok(GitFetchStats {
            received_objects: stats.received_objects(),
            indexed_objects: stats.indexed_objects(),
            local_objects: stats.local_objects(),
            total_objects: stats.total_objects(),
            received_bytes: stats.received_bytes(),
            updated_refs,
        })
    }

    pub fn push(
        &self,
        remote_name: Option<&str>,
        refspecs: &[String],
        options: GitPushOptions,
    ) -> Result<GitPushResult> {
        let op = "ptool.git.Repo:push(remote?, refspecs?, options?)";
        let remote_name = remote_name.unwrap_or("origin");
        validate_remote_name(remote_name, op)?;
        if options.force && refspecs.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:push force = true requires explicit refspecs",
            )
            .with_op(op));
        }
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .map_err(|err| repo_error(op, err))?;

        let rejected = Arc::new(Mutex::new(Vec::new()));
        let callback_rejected = Arc::clone(&rejected);
        let mut callbacks = build_remote_callbacks(options.auth);
        callbacks.push_update_reference(move |reference, status| {
            if let Some(message) = status
                && let Ok(mut rejected) = callback_rejected.lock()
            {
                rejected.push(GitPushRejection {
                    reference: reference.to_string(),
                    message: message.to_string(),
                });
            }
            Ok(())
        });
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let mut owned_refspecs = if refspecs.is_empty() {
            vec![default_push_refspec(&self.repo)?]
        } else {
            refspecs.to_vec()
        };
        if options.force {
            for refspec in &mut owned_refspecs {
                if !refspec.starts_with('+') && !refspec.starts_with(':') {
                    refspec.insert(0, '+');
                }
            }
        }
        let refspec_slices: Vec<&str> = owned_refspecs.iter().map(String::as_str).collect();
        remote
            .push(&refspec_slices, Some(&mut push_options))
            .map_err(|err| repo_error(op, err))?;

        if options.set_upstream {
            let branch_name = current_branch_name(&self.repo, op)?.ok_or_else(|| {
                Error::new(
                    ErrorKind::Git,
                    "ptool.git.Repo:push cannot set upstream from detached HEAD",
                )
                .with_op(op)
            })?;
            let mut branch = self
                .repo
                .find_branch(&branch_name, git2::BranchType::Local)
                .map_err(|err| repo_error(op, err))?;
            branch
                .set_upstream(Some(&format!("{remote_name}/{branch_name}")))
                .map_err(|err| repo_error(op, err))?;
        }

        let rejected = rejected
            .lock()
            .map_err(|_| Error::new(ErrorKind::Git, "git push callback state was poisoned"))?
            .clone();
        Ok(GitPushResult {
            refspecs: owned_refspecs,
            rejected,
        })
    }
}

fn build_remote_info(repo: &Repository, name: &str, op: &str) -> Result<GitRemoteInfo> {
    let remote = repo.find_remote(name).map_err(|err| repo_error(op, err))?;
    let fetch_refspecs = remote
        .fetch_refspecs()
        .map_err(|err| repo_error(op, err))?
        .iter()
        .flatten()
        .map(str::to_string)
        .collect();
    let push_refspecs = remote
        .push_refspecs()
        .map_err(|err| repo_error(op, err))?
        .iter()
        .flatten()
        .map(str::to_string)
        .collect();
    Ok(GitRemoteInfo {
        name: name.to_string(),
        url: remote.url().map(str::to_string),
        push_url: remote.pushurl().map(str::to_string),
        fetch_refspecs,
        push_refspecs,
    })
}

fn default_push_refspec(repo: &Repository) -> Result<String> {
    let op = "ptool.git.Repo:push(remote?, refspecs?, options?)";
    let branch = current_branch_name(repo, op)?.ok_or_else(|| {
        Error::new(
            ErrorKind::Git,
            "ptool.git.Repo:push failed: cannot infer refspec from detached HEAD",
        )
        .with_op(op)
    })?;
    Ok(format!("refs/heads/{branch}:refs/heads/{branch}"))
}

fn validate_remote_name(name: &str, op: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} requires a non-empty remote name"),
        )
        .with_op(op));
    }
    Ok(())
}

fn validate_url(url: &str, op: &str) -> Result<()> {
    if url.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} requires a non-empty url"),
        )
        .with_op(op));
    }
    Ok(())
}

fn validate_depth(depth: Option<i32>, op: &str) -> Result<()> {
    if matches!(depth, Some(depth) if depth <= 0) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} depth must be greater than zero"),
        )
        .with_op(op));
    }
    Ok(())
}

fn raw_tag_download(tags: GitTagDownload) -> AutotagOption {
    match tags {
        GitTagDownload::Auto => AutotagOption::Auto,
        GitTagDownload::All => AutotagOption::All,
        GitTagDownload::None => AutotagOption::None,
    }
}
