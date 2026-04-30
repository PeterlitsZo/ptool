use crate::{Error, ErrorKind, Result};
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

pub fn join<I, S>(segments: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut path = PathBuf::new();
    let mut saw_segment = false;

    for segment in segments {
        saw_segment = true;
        let segment = segment.as_ref();
        ensure_non_empty_path_segment(segment)?;
        path.push(segment);
    }

    if !saw_segment {
        return Err(Error::new(
            ErrorKind::EmptyPath,
            "path.join requires at least one segment",
        ));
    }

    Ok(path_to_string(&normalize_path(&path)))
}

pub fn normalize(path: &str) -> Result<String> {
    ensure_non_empty_path(path)?;
    Ok(path_to_string(&normalize_path(Path::new(path))))
}

pub fn abspath(path: &str, base: Option<&str>, current_dir: &Path) -> Result<String> {
    ensure_non_empty_path(path)?;
    let base_dir = resolve_base_dir(base, current_dir)?;
    let absolute = resolve_path_with_base(path, &base_dir);
    Ok(path_to_string(&absolute))
}

pub fn runtime_abspath(path: &str) -> Result<String> {
    ensure_non_empty_path(path)?;
    let current_dir = std::env::current_dir().map_err(|err| {
        Error::new(
            ErrorKind::Io,
            format!("failed to determine current directory: {err}"),
        )
        .with_op("ptool.script_path")
        .with_path(path.to_string())
    })?;
    abspath(path, None, &current_dir)
}

pub fn relpath(path: &str, base: Option<&str>, current_dir: &Path) -> Result<String> {
    ensure_non_empty_path(path)?;
    let base_dir = resolve_base_dir(base, current_dir)?;
    let target_path = resolve_path_with_base(path, &base_dir);
    Ok(path_to_string(&make_relative_path(&base_dir, &target_path)))
}

pub fn isabs(path: &str) -> Result<bool> {
    ensure_non_empty_path(path)?;
    Ok(Path::new(path).is_absolute())
}

pub fn dirname(path: &str) -> Result<String> {
    ensure_non_empty_path(path)?;

    let normalized = normalize_path(Path::new(path));
    if is_root_path(&normalized) {
        return Ok(path_to_string(&normalized));
    }

    let dirname = match normalized.parent() {
        Some(parent) if parent.as_os_str().is_empty() => PathBuf::from("."),
        Some(parent) => normalize_path(parent),
        None => PathBuf::from("."),
    };
    Ok(path_to_string(&dirname))
}

pub fn basename(path: &str) -> Result<String> {
    ensure_non_empty_path(path)?;

    let normalized = normalize_path(Path::new(path));
    if is_root_path(&normalized) {
        return Ok(String::new());
    }

    let name = normalized
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    Ok(name)
}

pub fn extname(path: &str) -> Result<String> {
    ensure_non_empty_path(path)?;

    let base = basename(path)?;
    if base.is_empty() || base == "." || base == ".." {
        return Ok(String::new());
    }

    let Some(dot_index) = base.rfind('.') else {
        return Ok(String::new());
    };
    if dot_index == 0 {
        return Ok(String::new());
    }

    Ok(base[dot_index..].to_string())
}

fn ensure_non_empty_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "empty path"));
    }
    Ok(())
}

fn ensure_non_empty_path_segment(segment: &str) -> Result<()> {
    if segment.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "empty path segment"));
    }
    Ok(())
}

fn resolve_base_dir(base: Option<&str>, current_dir: &Path) -> Result<PathBuf> {
    let cwd = normalize_path(current_dir);
    let base_dir = match base {
        Some(base) => {
            ensure_non_empty_path(base)?;
            let base = Path::new(base);
            if base.is_absolute() {
                base.to_path_buf()
            } else {
                cwd.join(base)
            }
        }
        None => cwd,
    };

    Ok(normalize_path(&base_dir))
}

fn resolve_path_with_base(path: &str, base_dir: &Path) -> PathBuf {
    if Path::new(path).is_absolute() {
        normalize_path(Path::new(path))
    } else {
        normalize_path(&base_dir.join(path))
    }
}

fn make_relative_path(base: &Path, target: &Path) -> PathBuf {
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();

    let mut common = 0usize;
    let max_common = base_components.len().min(target_components.len());
    while common < max_common && base_components[common] == target_components[common] {
        common += 1;
    }

    if common == 0 {
        return target.to_path_buf();
    }

    let mut relative = PathBuf::new();
    for component in &base_components[common..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }

    for component in &target_components[common..] {
        if let Component::Normal(segment) = component {
            relative.push(segment);
        }
    }

    if relative.as_os_str().is_empty() {
        relative.push(".");
    }
    relative
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut prefix: Option<OsString> = None;
    let mut has_root = false;
    let mut segments: Vec<OsString> = Vec::new();

    for component in path.components() {
        match component {
            Component::Prefix(value) => {
                prefix = Some(value.as_os_str().to_os_string());
            }
            Component::RootDir => {
                has_root = true;
                segments.clear();
            }
            Component::CurDir => {}
            Component::ParentDir => {
                let can_pop = matches!(segments.last(), Some(last) if last != "..");
                if can_pop {
                    segments.pop();
                } else if !has_root {
                    segments.push(OsString::from(".."));
                }
            }
            Component::Normal(segment) => {
                segments.push(segment.to_os_string());
            }
        }
    }

    let mut normalized = PathBuf::new();
    if let Some(prefix) = prefix {
        normalized.push(prefix);
    }
    if has_root {
        normalized.push(std::path::MAIN_SEPARATOR.to_string());
    }
    for segment in segments {
        normalized.push(segment);
    }

    if normalized.as_os_str().is_empty() {
        if has_root {
            normalized.push(std::path::MAIN_SEPARATOR.to_string());
        } else {
            normalized.push(".");
        }
    }

    normalized
}

fn is_root_path(path: &Path) -> bool {
    path.is_absolute() && path.parent().is_none()
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
