use super::{
    GitBlameHunk, GitBlameOptions, GitCommitInfo, GitDescribeOptions, GitDiffDelta, GitDiffOptions,
    GitDiffSummary, GitLogOptions, GitObjectInfo, GitRepository, GitSignatureInfo, oid_to_string,
    path_to_string, repo_error,
};
use crate::{Error, ErrorKind, Result};
use git2::{
    Commit, Delta, DescribeFormatOptions, DescribeOptions, DiffFindOptions, DiffFlags, DiffFormat,
    DiffOptions as RawDiffOptions, ErrorCode, Object, ObjectType, Repository, Sort, Tree,
};

impl GitRepository {
    pub fn resolve(&self, rev: &str) -> Result<GitObjectInfo> {
        let op = "ptool.git.Repo:resolve(rev)";
        validate_rev(rev, op)?;
        let (object, reference) = self
            .repo
            .revparse_ext(rev)
            .map_err(|err| repo_error(op, err))?;
        Ok(GitObjectInfo {
            oid: oid_to_string(object.id()),
            kind: object_kind_name(&object),
            shorthand: reference
                .as_ref()
                .and_then(|reference| reference.shorthand().map(str::to_string)),
        })
    }

    pub fn commit_info(&self, rev: Option<&str>) -> Result<GitCommitInfo> {
        let op = "ptool.git.Repo:commit_info(rev?)";
        let rev = rev.unwrap_or("HEAD");
        validate_rev(rev, op)?;
        let commit = self
            .repo
            .revparse_single(rev)
            .and_then(|object| object.peel_to_commit())
            .map_err(|err| repo_error(op, err))?;
        Ok(commit_to_info(&commit))
    }

    pub fn log(&self, options: GitLogOptions) -> Result<Vec<GitCommitInfo>> {
        let op = "ptool.git.Repo:log(options?)";
        validate_rev(&options.rev, op)?;
        let mut walk = self.repo.revwalk().map_err(|err| repo_error(op, err))?;
        if options.rev.contains("..") {
            walk.push_range(&options.rev)
                .map_err(|err| repo_error(op, err))?;
        } else {
            let oid = self
                .repo
                .revparse_single(&options.rev)
                .and_then(|object| object.peel_to_commit())
                .map(|commit| commit.id())
                .map_err(|err| repo_error(op, err))?;
            walk.push(oid).map_err(|err| repo_error(op, err))?;
        }
        if options.first_parent {
            walk.simplify_first_parent()
                .map_err(|err| repo_error(op, err))?;
        }
        let sort = if options.reverse {
            Sort::TOPOLOGICAL | Sort::REVERSE
        } else {
            Sort::TOPOLOGICAL | Sort::TIME
        };
        walk.set_sorting(sort).map_err(|err| repo_error(op, err))?;

        let mut result = Vec::new();
        let mut matched = 0usize;
        for oid in walk {
            let oid = oid.map_err(|err| repo_error(op, err))?;
            let commit = self
                .repo
                .find_commit(oid)
                .map_err(|err| repo_error(op, err))?;
            if !options.paths.is_empty()
                && !commit_touches_paths(&self.repo, &commit, &options.paths)?
            {
                continue;
            }
            if matched < options.skip {
                matched += 1;
                continue;
            }
            if result.len() >= options.max_count {
                break;
            }
            result.push(commit_to_info(&commit));
        }
        Ok(result)
    }

    pub fn diff(&self, options: GitDiffOptions) -> Result<GitDiffSummary> {
        let op = "ptool.git.Repo:diff(options?)";
        validate_diff_options(&options, op)?;
        let mut raw_options = RawDiffOptions::new();
        raw_options.context_lines(options.context_lines);
        for path in &options.paths {
            raw_options.pathspec(path);
        }

        let from_tree = resolve_optional_tree(
            &self.repo,
            options
                .from
                .as_deref()
                .or(if options.cached { Some("HEAD") } else { None }),
            op,
        )?;
        let to_tree = resolve_optional_tree(&self.repo, options.to.as_deref(), op)?;

        let mut diff = match (&from_tree, &to_tree, options.cached) {
            (from, Some(to), false) => {
                self.repo
                    .diff_tree_to_tree(from.as_ref(), Some(to), Some(&mut raw_options))
            }
            (from, None, true) => {
                self.repo
                    .diff_tree_to_index(from.as_ref(), None, Some(&mut raw_options))
            }
            (Some(from), None, false) => self
                .repo
                .diff_tree_to_workdir_with_index(Some(from), Some(&mut raw_options)),
            (None, None, false) => {
                let head_tree = resolve_head_tree(&self.repo, op)?;
                self.repo
                    .diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut raw_options))
            }
            (_, Some(_), true) => unreachable!("validated options"),
        }
        .map_err(|err| repo_error(op, err))?;

        if options.find_renames {
            let mut find = DiffFindOptions::new();
            find.renames(true);
            diff.find_similar(Some(&mut find))
                .map_err(|err| repo_error(op, err))?;
        }

        let stats = diff.stats().map_err(|err| repo_error(op, err))?;
        let deltas = diff
            .deltas()
            .map(|delta| GitDiffDelta {
                status: delta_status_name(delta.status()).to_string(),
                old_path: delta.old_file().path().map(path_to_string),
                new_path: delta.new_file().path().map(path_to_string),
                binary: delta.flags().contains(DiffFlags::BINARY),
            })
            .collect();
        let mut patch = Vec::new();
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            patch.extend_from_slice(line.content());
            true
        })
        .map_err(|err| repo_error(op, err))?;

        Ok(GitDiffSummary {
            patch: String::from_utf8_lossy(&patch).into_owned(),
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
            deltas,
        })
    }

    pub fn describe(&self, options: GitDescribeOptions) -> Result<Option<String>> {
        let op = "ptool.git.Repo:describe(options?)";
        if options.abbrev == 0 {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:describe abbrev must be greater than zero",
            )
            .with_op(op));
        }
        let mut describe_options = DescribeOptions::new();
        describe_options.describe_tags();
        describe_options.show_commit_oid_as_fallback(options.always);
        if let Some(pattern) = options.pattern.as_deref() {
            if pattern.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidArgs,
                    "ptool.git.Repo:describe pattern must not be empty",
                )
                .with_op(op));
            }
            describe_options.pattern(pattern);
        }

        let mut format = DescribeFormatOptions::new();
        format.abbreviated_size(options.abbrev as u32);
        if let Some(suffix) = options.dirty_suffix.as_deref() {
            format.dirty_suffix(suffix);
        }

        let format_result = |result: std::result::Result<git2::Describe<'_>, git2::Error>| {
            let described = match result {
                Ok(value) => value,
                Err(err) if err.code() == ErrorCode::NotFound => return Ok(None),
                Err(err) => return Err(repo_error(op, err)),
            };
            described
                .format(Some(&format))
                .map(Some)
                .map_err(|err| repo_error(op, err))
        };

        match options.rev.as_deref() {
            Some(rev) => {
                validate_rev(rev, op)?;
                let object = self
                    .repo
                    .revparse_single(rev)
                    .map_err(|err| repo_error(op, err))?;
                format_result(object.describe(&describe_options))
            }
            None => format_result(self.repo.describe(&describe_options)),
        }
    }
}

fn validate_rev(rev: &str, op: &str) -> Result<()> {
    if rev.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} requires a non-empty rev"),
        )
        .with_op(op));
    }
    Ok(())
}

fn validate_diff_options(options: &GitDiffOptions, op: &str) -> Result<()> {
    if options.cached && options.to.is_some() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.git.Repo:diff cannot combine `cached = true` with `to`",
        )
        .with_op(op));
    }
    if options.to.is_some() && options.from.is_none() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.git.Repo:diff requires `from` when `to` is provided",
        )
        .with_op(op));
    }
    if matches!(options.from.as_deref(), Some("")) || matches!(options.to.as_deref(), Some("")) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "ptool.git.Repo:diff revisions must not be empty",
        )
        .with_op(op));
    }
    Ok(())
}

fn resolve_optional_tree<'repo>(
    repo: &'repo Repository,
    rev: Option<&str>,
    op: &str,
) -> Result<Option<Tree<'repo>>> {
    rev.map(|rev| {
        repo.revparse_single(rev)
            .and_then(|object| object.peel_to_tree())
            .map_err(|err| repo_error(op, err))
    })
    .transpose()
}

fn resolve_head_tree<'repo>(repo: &'repo Repository, op: &str) -> Result<Option<Tree<'repo>>> {
    match repo.head() {
        Ok(head) => head
            .peel_to_tree()
            .map(Some)
            .map_err(|err| repo_error(op, err)),
        Err(err) if err.code() == ErrorCode::UnbornBranch => Ok(None),
        Err(err) => Err(repo_error(op, err)),
    }
}

fn commit_to_info(commit: &Commit<'_>) -> GitCommitInfo {
    GitCommitInfo {
        oid: oid_to_string(commit.id()),
        summary: commit.summary().map(str::to_string),
        message: commit.message().map(str::to_string),
        author: signature_to_info(commit.author()),
        committer: signature_to_info(commit.committer()),
        parent_oids: commit.parent_ids().map(oid_to_string).collect(),
    }
}

fn signature_to_info(signature: git2::Signature<'_>) -> GitSignatureInfo {
    GitSignatureInfo {
        name: signature.name().map(str::to_string),
        email: signature.email().map(str::to_string),
        time_seconds: signature.when().seconds(),
        offset_minutes: signature.when().offset_minutes(),
    }
}

fn commit_touches_paths(repo: &Repository, commit: &Commit<'_>, paths: &[String]) -> Result<bool> {
    let op = "ptool.git.Repo:log(options?)";
    let tree = commit.tree().map_err(|err| repo_error(op, err))?;
    let parent_tree = if commit.parent_count() == 0 {
        None
    } else {
        Some(
            commit
                .parent(0)
                .and_then(|parent| parent.tree())
                .map_err(|err| repo_error(op, err))?,
        )
    };
    let mut options = RawDiffOptions::new();
    for path in paths {
        options.pathspec(path);
    }
    let diff = repo
        .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut options))
        .map_err(|err| repo_error(op, err))?;
    Ok(diff.deltas().next().is_some())
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

fn delta_status_name(status: Delta) -> &'static str {
    match status {
        Delta::Unmodified => "unmodified",
        Delta::Added => "added",
        Delta::Deleted => "deleted",
        Delta::Modified => "modified",
        Delta::Renamed => "renamed",
        Delta::Copied => "copied",
        Delta::Ignored => "ignored",
        Delta::Untracked => "untracked",
        Delta::Typechange => "typechange",
        Delta::Unreadable => "unreadable",
        Delta::Conflicted => "conflicted",
    }
}

impl GitRepository {
    pub fn blame(&self, path: &str, options: GitBlameOptions) -> Result<Vec<GitBlameHunk>> {
        let op = "ptool.git.Repo:blame(path, options?)";
        let path_value = std::path::Path::new(path);
        if path.is_empty()
            || path_value.is_absolute()
            || path_value.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::ParentDir
                        | std::path::Component::RootDir
                        | std::path::Component::Prefix(_)
                )
            })
        {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "blame requires a safe repository-relative path",
            )
            .with_op(op)
            .with_path(path.to_string()));
        }
        if matches!(options.min_line, Some(0)) || matches!(options.max_line, Some(0)) {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "blame line numbers are one-based and must be positive",
            )
            .with_op(op));
        }
        if let (Some(min), Some(max)) = (options.min_line, options.max_line)
            && min > max
        {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "blame min_line must not exceed max_line",
            )
            .with_op(op));
        }
        let mut raw_options = git2::BlameOptions::new();
        raw_options
            .first_parent(options.first_parent)
            .track_copies_same_file(options.track_copies_same_file)
            .track_copies_same_commit_moves(options.track_copies_same_commit_moves)
            .track_copies_same_commit_copies(options.track_copies_same_commit_copies)
            .track_copies_any_commit_copies(options.track_copies_any_commit_copies)
            .ignore_whitespace(options.ignore_whitespace)
            .use_mailmap(options.use_mailmap);
        if let Some(revision) = options.newest.as_deref() {
            let oid = self
                .repo
                .revparse_single(revision)
                .map_err(|err| repo_error(op, err))?
                .id();
            raw_options.newest_commit(oid);
        }
        if let Some(revision) = options.oldest.as_deref() {
            let oid = self
                .repo
                .revparse_single(revision)
                .map_err(|err| repo_error(op, err))?
                .id();
            raw_options.oldest_commit(oid);
        }
        if let Some(line) = options.min_line {
            raw_options.min_line(line);
        }
        if let Some(line) = options.max_line {
            raw_options.max_line(line);
        }
        let blame = self
            .repo
            .blame_file(path_value, Some(&mut raw_options))
            .map_err(|err| repo_error(op, err))?;
        Ok(blame
            .iter()
            .map(|hunk| {
                let signature = hunk.final_signature();
                GitBlameHunk {
                    final_start_line: hunk.final_start_line(),
                    original_start_line: hunk.orig_start_line(),
                    line_count: hunk.lines_in_hunk(),
                    commit_oid: oid_to_string(hunk.final_commit_id()),
                    author: GitSignatureInfo {
                        name: signature.name().map(str::to_string),
                        email: signature.email().map(str::to_string),
                        time_seconds: signature.when().seconds(),
                        offset_minutes: signature.when().offset_minutes(),
                    },
                    origin_path: hunk.path().map(path_to_string),
                    boundary: hunk.is_boundary(),
                }
            })
            .collect())
    }
}
