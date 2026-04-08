use mlua::Variadic;
use ptool_engine::PtoolEngine;
use std::path::Path;

const JOIN_SIGNATURE: &str = "ptool.path.join(...)";
const NORMALIZE_SIGNATURE: &str = "ptool.path.normalize(path)";
const ABSPATH_SIGNATURE: &str = "ptool.path.abspath(path[, base])";
const RELPATH_SIGNATURE: &str = "ptool.path.relpath(path[, base])";
const ISABS_SIGNATURE: &str = "ptool.path.isabs(path)";
const DIRNAME_SIGNATURE: &str = "ptool.path.dirname(path)";
const BASENAME_SIGNATURE: &str = "ptool.path.basename(path)";
const EXTNAME_SIGNATURE: &str = "ptool.path.extname(path)";

pub(crate) fn join(engine: &PtoolEngine, segments: Variadic<String>) -> mlua::Result<String> {
    if segments.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{JOIN_SIGNATURE} requires at least one segment"
        )));
    }

    for segment in segments.iter() {
        ensure_non_empty(segment, JOIN_SIGNATURE)?;
    }

    engine
        .path_join(segments.iter().map(String::as_str))
        .map_err(|err| mlua::Error::runtime(format!("{JOIN_SIGNATURE} failed: {}", err.msg)))
}

pub(crate) fn normalize(engine: &PtoolEngine, path: String) -> mlua::Result<String> {
    ensure_non_empty(&path, NORMALIZE_SIGNATURE)?;
    engine
        .path_normalize(&path)
        .map_err(|err| mlua::Error::runtime(format!("{NORMALIZE_SIGNATURE} failed: {}", err.msg)))
}

pub(crate) fn abspath_from_args(
    engine: &PtoolEngine,
    args: Variadic<String>,
    current_dir: &Path,
) -> mlua::Result<String> {
    let (path, base) = parse_path_and_base_args(args, ABSPATH_SIGNATURE)?;
    abspath(engine, path, base, current_dir)
}

pub(crate) fn relpath_from_args(
    engine: &PtoolEngine,
    args: Variadic<String>,
    current_dir: &Path,
) -> mlua::Result<String> {
    let (path, base) = parse_path_and_base_args(args, RELPATH_SIGNATURE)?;
    engine
        .path_relpath(&path, base.as_deref(), current_dir)
        .map_err(|err| mlua::Error::runtime(format!("{RELPATH_SIGNATURE} failed: {}", err.msg)))
}

pub(crate) fn isabs(engine: &PtoolEngine, path: String) -> mlua::Result<bool> {
    ensure_non_empty(&path, ISABS_SIGNATURE)?;
    engine
        .path_isabs(&path)
        .map_err(|err| mlua::Error::runtime(format!("{ISABS_SIGNATURE} failed: {}", err.msg)))
}

pub(crate) fn dirname(engine: &PtoolEngine, path: String) -> mlua::Result<String> {
    ensure_non_empty(&path, DIRNAME_SIGNATURE)?;
    engine
        .path_dirname(&path)
        .map_err(|err| mlua::Error::runtime(format!("{DIRNAME_SIGNATURE} failed: {}", err.msg)))
}

pub(crate) fn basename(engine: &PtoolEngine, path: String) -> mlua::Result<String> {
    ensure_non_empty(&path, BASENAME_SIGNATURE)?;
    engine
        .path_basename(&path)
        .map_err(|err| mlua::Error::runtime(format!("{BASENAME_SIGNATURE} failed: {}", err.msg)))
}

pub(crate) fn extname(engine: &PtoolEngine, path: String) -> mlua::Result<String> {
    ensure_non_empty(&path, EXTNAME_SIGNATURE)?;
    engine
        .path_extname(&path)
        .map_err(|err| mlua::Error::runtime(format!("{EXTNAME_SIGNATURE} failed: {}", err.msg)))
}

fn abspath(
    engine: &PtoolEngine,
    path: String,
    base: Option<String>,
    current_dir: &Path,
) -> mlua::Result<String> {
    ensure_non_empty(&path, ABSPATH_SIGNATURE)?;
    engine
        .path_abspath(&path, base.as_deref(), current_dir)
        .map_err(|err| mlua::Error::runtime(format!("{ABSPATH_SIGNATURE} failed: {}", err.msg)))
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
