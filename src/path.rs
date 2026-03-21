use mlua::Variadic;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

pub(crate) fn join(segments: Variadic<String>) -> mlua::Result<String> {
    if segments.is_empty() {
        return Err(mlua::Error::runtime(
            "ptool.path.join(...) requires at least one segment",
        ));
    }

    let mut path = PathBuf::new();
    for segment in segments {
        ensure_non_empty(&segment, "ptool.path.join(...)")?;
        path.push(segment);
    }

    Ok(path_to_string(&normalize_path(&path)))
}

pub(crate) fn normalize(path: String) -> mlua::Result<String> {
    ensure_non_empty(&path, "ptool.path.normalize(path)")?;
    Ok(path_to_string(&normalize_path(Path::new(&path))))
}

pub(crate) fn abspath_from_args(args: Variadic<String>) -> mlua::Result<String> {
    let (path, base) = parse_path_and_base_args(args, "ptool.path.abspath(path[, base])")?;
    abspath(path, base)
}

pub(crate) fn relpath_from_args(args: Variadic<String>) -> mlua::Result<String> {
    let (path, base) = parse_path_and_base_args(args, "ptool.path.relpath(path[, base])")?;

    let base_dir = resolve_base_dir(base.as_deref(), "ptool.path.relpath(path[, base])")?;
    let target_path = resolve_path_with_base(&path, &base_dir);
    Ok(path_to_string(&make_relative_path(&base_dir, &target_path)))
}

pub(crate) fn isabs(path: String) -> mlua::Result<bool> {
    ensure_non_empty(&path, "ptool.path.isabs(path)")?;
    Ok(Path::new(&path).is_absolute())
}

pub(crate) fn dirname(path: String) -> mlua::Result<String> {
    ensure_non_empty(&path, "ptool.path.dirname(path)")?;

    let normalized = normalize_path(Path::new(&path));
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

pub(crate) fn basename(path: String) -> mlua::Result<String> {
    ensure_non_empty(&path, "ptool.path.basename(path)")?;

    let normalized = normalize_path(Path::new(&path));
    if is_root_path(&normalized) {
        return Ok(String::new());
    }

    let name = normalized
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    Ok(name)
}

pub(crate) fn extname(path: String) -> mlua::Result<String> {
    ensure_non_empty(&path, "ptool.path.extname(path)")?;

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

fn abspath(path: String, base: Option<String>) -> mlua::Result<String> {
    ensure_non_empty(&path, "ptool.path.abspath(path[, base])")?;
    let base_dir = resolve_base_dir(base.as_deref(), "ptool.path.abspath(path[, base])")?;
    let absolute = resolve_path_with_base(&path, &base_dir);
    Ok(path_to_string(&absolute))
}

fn parse_path_and_base_args(
    args: Variadic<String>,
    context: &str,
) -> mlua::Result<(String, Option<String>)> {
    match args.len() {
        1 => {
            let path = args[0].clone();
            ensure_non_empty(&path, context)?;
            Ok((path, None))
        }
        2 => {
            let path = args[0].clone();
            let base = args[1].clone();
            ensure_non_empty(&path, context)?;
            ensure_non_empty(&base, context)?;
            Ok((path, Some(base)))
        }
        _ => Err(mlua::Error::runtime(format!(
            "{context} accepts 1 or 2 string arguments"
        ))),
    }
}

fn ensure_non_empty(input: &str, context: &str) -> mlua::Result<()> {
    if input.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{context} does not accept empty string"
        )));
    }
    Ok(())
}

fn resolve_base_dir(base: Option<&str>, context: &str) -> mlua::Result<PathBuf> {
    let cwd = std::env::current_dir().map_err(|err| {
        mlua::Error::runtime(format!("{context} failed to get current dir: {err}"))
    })?;

    let base_dir = match base {
        Some(base) => {
            ensure_non_empty(base, context)?;
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
