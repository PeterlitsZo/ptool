use mlua::{Lua, Table, Value, Variadic};
use std::fs as stdfs;
use std::path::Path;

const COPY_SIGNATURE: &str = "ptool.fs.copy(src, dst[, options])";
const MKDIR_SIGNATURE: &str = "ptool.fs.mkdir(path[, options])";

pub(crate) fn read(path: String) -> mlua::Result<String> {
    stdfs::read_to_string(&path)
        .map_err(|err| mlua::Error::runtime(format!("ptool.fs.read `{path}` failed: {err}")))
}

pub(crate) fn write(path: String, content: String) -> mlua::Result<()> {
    stdfs::write(&path, content)
        .map_err(|err| mlua::Error::runtime(format!("ptool.fs.write `{path}` failed: {err}")))
}

pub(crate) fn mkdir(path: String, options: Option<Table>) -> mlua::Result<()> {
    let options = MkdirOptions::parse(options)?;
    if !options.exist_ok && Path::new(&path).is_dir() {
        return Err(mlua::Error::runtime(format!(
            "{MKDIR_SIGNATURE} `{path}` already exists"
        )));
    }

    stdfs::create_dir_all(&path)
        .map_err(|err| mlua::Error::runtime(format!("{MKDIR_SIGNATURE} `{path}` failed: {err}")))
}

pub(crate) fn exists(path: String) -> bool {
    Path::new(&path).exists()
}

#[derive(Debug, Clone, Copy)]
struct MkdirOptions {
    exist_ok: bool,
}

impl Default for MkdirOptions {
    fn default() -> Self {
        Self { exist_ok: true }
    }
}

impl MkdirOptions {
    fn parse(options: Option<Table>) -> mlua::Result<Self> {
        let mut parsed = Self::default();
        let Some(options) = options else {
            return Ok(parsed);
        };

        for pair in options.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let key = match key {
                Value::String(key) => key.to_str()?.to_string(),
                _ => {
                    return Err(mlua::Error::runtime(format!(
                        "{MKDIR_SIGNATURE} option keys must be strings"
                    )));
                }
            };

            match key.as_str() {
                "exist_ok" => match value {
                    Value::Boolean(value) => parsed.exist_ok = value,
                    _ => {
                        return Err(mlua::Error::runtime(format!(
                            "{MKDIR_SIGNATURE} `exist_ok` must be a boolean"
                        )));
                    }
                },
                _ => {
                    return Err(mlua::Error::runtime(format!(
                        "{MKDIR_SIGNATURE} unknown option `{key}`"
                    )));
                }
            }
        }

        Ok(parsed)
    }
}

pub(crate) fn copy(lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
    if !(2..=3).contains(&args.len()) {
        return Err(mlua::Error::runtime(format!(
            "{COPY_SIGNATURE} expects 2 or 3 arguments"
        )));
    }

    let src = parse_copy_endpoint(args[0].clone(), "src")?;
    let dst = parse_copy_endpoint(args[1].clone(), "dst")?;
    let options = crate::ssh::parse_transfer_options(args.get(2).cloned(), COPY_SIGNATURE)?;

    let result = match (src, dst) {
        (CopyEndpoint::Local(src), CopyEndpoint::Local(dst)) => {
            copy_local_to_local(&src, &dst, options)?
        }
        (CopyEndpoint::Local(src), CopyEndpoint::Remote(dst)) => {
            dst.connection()
                .upload_file(Path::new(&src), dst.path(), options)?
        }
        (CopyEndpoint::Remote(src), CopyEndpoint::Local(dst)) => {
            src.connection()
                .download_file(src.path(), Path::new(&dst), options)?
        }
        (CopyEndpoint::Remote(_), CopyEndpoint::Remote(_)) => {
            return Err(mlua::Error::runtime(format!(
                "{COPY_SIGNATURE} does not support remote-to-remote copies"
            )));
        }
    };

    crate::ssh::build_transfer_result(lua, result)
}

enum CopyEndpoint {
    Local(String),
    Remote(crate::ssh::LuaSshPath),
}

fn parse_copy_endpoint(value: Value, field: &str) -> mlua::Result<CopyEndpoint> {
    match value {
        Value::String(path) => {
            let path = path.to_str()?.to_string();
            ensure_non_empty_string(&path, field)?;
            Ok(CopyEndpoint::Local(path))
        }
        Value::UserData(_) => Ok(CopyEndpoint::Remote(crate::ssh::parse_remote_path_value(
            value,
            COPY_SIGNATURE,
            field,
        )?)),
        _ => Err(mlua::Error::runtime(format!(
            "{COPY_SIGNATURE} `{field}` must be a string path or a remote path from `conn:path(...)`"
        ))),
    }
}

fn copy_local_to_local(
    src: &str,
    dst: &str,
    options: crate::ssh::TransferOptions,
) -> mlua::Result<crate::ssh::TransferResult> {
    let src_path = Path::new(src);
    let metadata = stdfs::metadata(src_path).map_err(|err| {
        mlua::Error::runtime(format!("{COPY_SIGNATURE} failed to access `{src}`: {err}"))
    })?;
    if !metadata.is_file() {
        return Err(mlua::Error::runtime(format!(
            "{COPY_SIGNATURE} `src` must be a file: `{src}`"
        )));
    }

    let dst_path = Path::new(dst);
    if options.parents
        && let Some(parent) = dst_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
    {
        stdfs::create_dir_all(parent).map_err(|err| {
            mlua::Error::runtime(format!(
                "{COPY_SIGNATURE} failed to create parent directory `{}`: {err}",
                parent.display()
            ))
        })?;
    }

    if !options.overwrite && dst_path.exists() {
        return Err(mlua::Error::runtime(format!(
            "{COPY_SIGNATURE} `dst` already exists: `{dst}`"
        )));
    }

    if options.echo {
        println!("[copy] {src} -> {dst}");
    }

    let bytes = stdfs::copy(src_path, dst_path).map_err(|err| {
        mlua::Error::runtime(format!(
            "{COPY_SIGNATURE} failed to copy `{src}` to `{dst}`: {err}"
        ))
    })?;

    Ok(crate::ssh::TransferResult {
        bytes,
        from: src.to_string(),
        to: dst.to_string(),
    })
}

fn ensure_non_empty_string(value: &str, field: &str) -> mlua::Result<()> {
    if value.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{COPY_SIGNATURE} `{field}` must not be empty"
        )));
    }
    Ok(())
}
