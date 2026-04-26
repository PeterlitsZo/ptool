use crate::{Error, ErrorKind, Result};
use glob::MatchOptions;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FsMkdirOptions {
    pub exist_ok: bool,
}

impl Default for FsMkdirOptions {
    fn default() -> Self {
        Self { exist_ok: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FsCopyOptions {
    pub parents: bool,
    pub overwrite: bool,
    pub echo: bool,
}

impl Default for FsCopyOptions {
    fn default() -> Self {
        Self {
            parents: false,
            overwrite: true,
            echo: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FsCopyResult {
    pub bytes: u64,
    pub from: String,
    pub to: String,
}

pub fn read(path: &str) -> Result<Vec<u8>> {
    ensure_non_empty_path(path)?;
    fs::read(path).map_err(|err| io_error(err).with_op("ptool.fs.read").with_path(path))
}

pub fn write(path: &str, content: &[u8]) -> Result<()> {
    ensure_non_empty_path(path)?;
    fs::write(path, content).map_err(|err| io_error(err).with_op("ptool.fs.write").with_path(path))
}

pub fn mkdir(path: &str, options: FsMkdirOptions) -> Result<()> {
    ensure_non_empty_path(path)?;

    if !options.exist_ok && Path::new(path).is_dir() {
        return Err(
            Error::new(ErrorKind::AlreadyExists, format!("`{path}` already exists"))
                .with_op("ptool.fs.mkdir")
                .with_path(path),
        );
    }

    fs::create_dir_all(path).map_err(|err| io_error(err).with_op("ptool.fs.mkdir").with_path(path))
}

pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn glob(pattern: &str, base_dir: &Path) -> Result<Vec<String>> {
    ensure_non_empty_path(pattern)?;

    let original_pattern = pattern;
    let pattern = resolve_glob_pattern(pattern, base_dir);
    let entries = glob::glob_with(
        &pattern,
        MatchOptions {
            case_sensitive: true,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        },
    )
    .map_err(|err| {
        Error::new(ErrorKind::InvalidGlob, err.to_string())
            .with_op("ptool.fs.glob")
            .with_input(original_pattern)
    })?;

    let mut matches = Vec::new();
    for entry in entries {
        let path = entry.map_err(|err| {
            Error::new(ErrorKind::Io, err.to_string())
                .with_op("ptool.fs.glob")
                .with_input(original_pattern)
        })?;
        if allows_hidden_match(original_pattern, base_dir, &path)? {
            matches.push(path_to_string(path));
        }
    }

    matches.sort();
    Ok(matches)
}

pub fn copy_local(src: &str, dst: &str, options: FsCopyOptions) -> Result<FsCopyResult> {
    ensure_non_empty_named_path(src, "src")?;
    ensure_non_empty_named_path(dst, "dst")?;

    let src_path = Path::new(src);
    let metadata = fs::metadata(src_path).map_err(|err| {
        Error::new(ErrorKind::Io, format!("failed to access `{src}`: {err}"))
            .with_op("ptool.fs.copy")
            .with_path(src)
    })?;
    if !metadata.is_file() {
        return Err(Error::new(
            ErrorKind::NotAFile,
            format!("`src` must be a file: `{src}`"),
        )
        .with_op("ptool.fs.copy")
        .with_path(src));
    }

    let dst_path = Path::new(dst);
    if options.parents
        && let Some(parent) = dst_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|err| {
            Error::new(
                ErrorKind::Io,
                format!(
                    "failed to create parent directory `{}`: {err}",
                    parent.display()
                ),
            )
            .with_op("ptool.fs.copy")
            .with_path(parent.display().to_string())
        })?;
    }

    if !options.overwrite && dst_path.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("`dst` already exists: `{dst}`"),
        )
        .with_op("ptool.fs.copy")
        .with_path(dst));
    }

    if options.echo {
        println!("[copy] {src} -> {dst}");
    }

    let bytes = fs::copy(src_path, dst_path).map_err(|err| {
        Error::new(
            ErrorKind::Io,
            format!("failed to copy `{src}` to `{dst}`: {err}"),
        )
        .with_op("ptool.fs.copy")
        .with_path(dst)
    })?;

    Ok(FsCopyResult {
        bytes,
        from: src.to_string(),
        to: dst.to_string(),
    })
}

fn ensure_non_empty_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "empty path"));
    }
    Ok(())
}

fn ensure_non_empty_named_path(path: &str, field: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::new(
            ErrorKind::EmptyPath,
            format!("`{field}` must not be empty"),
        ));
    }
    Ok(())
}

fn resolve_glob_pattern(pattern: &str, base_dir: &Path) -> String {
    let pattern_path = Path::new(pattern);
    if pattern_path.is_absolute() {
        pattern.to_string()
    } else {
        path_to_string(base_dir.join(pattern_path))
    }
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn allows_hidden_match(pattern: &str, base_dir: &Path, path: &Path) -> Result<bool> {
    let candidate = if Path::new(pattern).is_absolute() {
        path.to_path_buf()
    } else {
        path.strip_prefix(base_dir)
            .map(Path::to_path_buf)
            .map_err(|err| {
                Error::new(
                    ErrorKind::Io,
                    format!(
                        "failed to resolve glob match `{}` relative to `{}`: {err}",
                        path.display(),
                        base_dir.display()
                    ),
                )
            })?
    };

    let pattern_components = collect_components(Path::new(pattern));
    let path_components = collect_components(&candidate);
    Ok(match_components_allowing_hidden(
        &pattern_components,
        &path_components,
    ))
}

fn collect_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            std::path::Component::CurDir => Some(".".to_string()),
            std::path::Component::ParentDir => Some("..".to_string()),
            std::path::Component::Prefix(_) | std::path::Component::RootDir => None,
        })
        .collect()
}

fn match_components_allowing_hidden(
    pattern_components: &[String],
    path_components: &[String],
) -> bool {
    let mut memo = vec![vec![None; path_components.len() + 1]; pattern_components.len() + 1];
    match_components_allowing_hidden_impl(pattern_components, path_components, 0, 0, &mut memo)
}

fn match_components_allowing_hidden_impl(
    pattern_components: &[String],
    path_components: &[String],
    pattern_index: usize,
    path_index: usize,
    memo: &mut [Vec<Option<bool>>],
) -> bool {
    if let Some(result) = memo[pattern_index][path_index] {
        return result;
    }

    let result = if pattern_index == pattern_components.len() {
        path_index == path_components.len()
    } else if pattern_components[pattern_index] == "**" {
        match_components_allowing_hidden_impl(
            pattern_components,
            path_components,
            pattern_index + 1,
            path_index,
            memo,
        ) || (path_index < path_components.len()
            && !is_hidden_component(&path_components[path_index])
            && match_components_allowing_hidden_impl(
                pattern_components,
                path_components,
                pattern_index,
                path_index + 1,
                memo,
            ))
    } else if path_index == path_components.len() {
        false
    } else {
        component_matches(
            &pattern_components[pattern_index],
            &path_components[path_index],
        ) && (!is_hidden_component(&path_components[path_index])
            || pattern_explicitly_matches_hidden(&pattern_components[pattern_index]))
            && match_components_allowing_hidden_impl(
                pattern_components,
                path_components,
                pattern_index + 1,
                path_index + 1,
                memo,
            )
    };

    memo[pattern_index][path_index] = Some(result);
    result
}

fn component_matches(pattern_component: &str, path_component: &str) -> bool {
    glob::Pattern::new(pattern_component)
        .map(|pattern| {
            pattern.matches_with(
                path_component,
                MatchOptions {
                    case_sensitive: true,
                    require_literal_separator: false,
                    require_literal_leading_dot: false,
                },
            )
        })
        .unwrap_or(false)
}

fn is_hidden_component(component: &str) -> bool {
    component.starts_with('.') && component != "." && component != ".."
}

fn pattern_explicitly_matches_hidden(pattern_component: &str) -> bool {
    let mut chars = pattern_component.chars();
    match chars.next() {
        Some('.') => true,
        Some('\\') => matches!(chars.next(), Some('.')),
        Some('[') => class_explicitly_matches_dot(chars),
        _ => false,
    }
}

fn class_explicitly_matches_dot(mut chars: std::str::Chars<'_>) -> bool {
    let mut negate = false;
    let mut first = true;
    let mut matches_dot = false;

    while let Some(ch) = chars.next() {
        if first {
            first = false;
            if ch == '!' || ch == '^' {
                negate = true;
                continue;
            }
        }

        if ch == ']' {
            break;
        }

        if ch == '\\' {
            if let Some(escaped) = chars.next() {
                matches_dot |= escaped == '.';
            }
            continue;
        }

        matches_dot |= ch == '.';
    }

    if negate { !matches_dot } else { matches_dot }
}

fn io_error(err: std::io::Error) -> Error {
    Error::new(ErrorKind::Io, err.to_string())
}
