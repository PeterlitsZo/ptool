use super::{
    GitRepository, GitSignature, GitTagCreateOptions, GitTagInfo, oid_to_string, repo_error,
};
use crate::{Error, ErrorKind, Result};
use git2::{Object, ObjectType, Repository, Signature};

impl GitRepository {
    pub fn tags(&self, pattern: Option<&str>) -> Result<Vec<GitTagInfo>> {
        let names = self
            .repo
            .tag_names(pattern)
            .map_err(|err| repo_error("ptool.git.Repo:tags(pattern?)", err))?;
        let mut tags = Vec::new();
        for name in names.iter().flatten() {
            tags.push(build_tag_info(&self.repo, name)?);
        }
        tags.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(tags)
    }

    pub fn tag_create(
        &self,
        name: &str,
        target: Option<&str>,
        options: GitTagCreateOptions,
    ) -> Result<GitTagInfo> {
        validate_tag_name(name, "ptool.git.Repo:tag_create(name, target?, options?)")?;
        let target = target.unwrap_or("HEAD");
        if target.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:tag_create target must not be empty",
            )
            .with_op("ptool.git.Repo:tag_create(name, target?, options?)"));
        }
        let object = self
            .repo
            .revparse_single(target)
            .map_err(|err| repo_error("ptool.git.Repo:tag_create(name, target?, options?)", err))?;

        match options.message.as_deref() {
            Some(message) => {
                if message.is_empty() {
                    return Err(Error::new(
                        ErrorKind::InvalidArgs,
                        "ptool.git.Repo:tag_create message must not be empty",
                    )
                    .with_op("ptool.git.Repo:tag_create(name, target?, options?)"));
                }
                let tagger = build_tag_signature(&self.repo, options.tagger.as_ref())?;
                self.repo
                    .tag(name, &object, &tagger, message, options.force)
                    .map_err(|err| {
                        repo_error("ptool.git.Repo:tag_create(name, target?, options?)", err)
                    })?;
            }
            None => {
                self.repo
                    .tag_lightweight(name, &object, options.force)
                    .map_err(|err| {
                        repo_error("ptool.git.Repo:tag_create(name, target?, options?)", err)
                    })?;
            }
        }

        build_tag_info(&self.repo, name)
    }

    pub fn tag_delete(&self, name: &str) -> Result<()> {
        validate_tag_name(name, "ptool.git.Repo:tag_delete(name)")?;
        self.repo
            .tag_delete(name)
            .map_err(|err| repo_error("ptool.git.Repo:tag_delete(name)", err))
    }
}

fn validate_tag_name(name: &str, op: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} requires a non-empty tag"),
        )
        .with_op(op));
    }
    if !git2::Reference::is_valid_name(&format!("refs/tags/{name}")) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} received an invalid tag name"),
        )
        .with_op(op)
        .with_input(name.to_string()));
    }
    Ok(())
}

fn build_tag_signature(
    repo: &Repository,
    input: Option<&GitSignature>,
) -> Result<Signature<'static>> {
    let op = "ptool.git.Repo:tag_create(name, target?, options?)";
    if let Some(input) = input {
        if input.name.is_empty() || input.email.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:tag_create tagger requires non-empty name and email",
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
                "ptool.git.Repo:tag_create tagger time fields must be provided together",
            )
            .with_op(op)),
        };
    }
    repo.signature().map_err(|_| {
        Error::new(
            ErrorKind::Git,
            "ptool.git.Repo:tag_create failed: git user identity is not configured",
        )
        .with_op(op)
    })
}

fn build_tag_info(repo: &Repository, name: &str) -> Result<GitTagInfo> {
    let op = "ptool.git.Repo:tags(pattern?)";
    let reference = repo
        .find_reference(&format!("refs/tags/{name}"))
        .map_err(|err| repo_error(op, err))?;
    let oid = reference
        .target()
        .ok_or_else(|| Error::new(ErrorKind::Git, format!("{op} failed: tag has no target")))?;
    let object = repo
        .find_object(oid, None)
        .map_err(|err| repo_error(op, err))?;

    if object.kind() == Some(ObjectType::Tag) {
        let tag = repo.find_tag(oid).map_err(|err| repo_error(op, err))?;
        let target = tag.target().map_err(|err| repo_error(op, err))?;
        Ok(GitTagInfo {
            name: name.to_string(),
            oid: oid_to_string(oid),
            target_oid: oid_to_string(target.id()),
            target_kind: object_kind_name(&target),
            annotated: true,
            message: tag.message().map(str::to_string),
            tagger: tag.tagger().map(|signature| GitSignature {
                name: signature.name().unwrap_or_default().to_string(),
                email: signature.email().unwrap_or_default().to_string(),
                time_seconds: Some(signature.when().seconds()),
                offset_minutes: Some(signature.when().offset_minutes()),
            }),
        })
    } else {
        Ok(GitTagInfo {
            name: name.to_string(),
            oid: oid_to_string(oid),
            target_oid: oid_to_string(oid),
            target_kind: object_kind_name(&object),
            annotated: false,
            message: None,
            tagger: None,
        })
    }
}

fn object_kind_name(object: &Object<'_>) -> String {
    match object.kind() {
        Some(ObjectType::Any) => "any",
        Some(ObjectType::Commit) => "commit",
        Some(ObjectType::Tree) => "tree",
        Some(ObjectType::Blob) => "blob",
        Some(ObjectType::Tag) => "tag",
        None => "unknown",
    }
    .to_string()
}
