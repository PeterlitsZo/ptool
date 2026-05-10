use mlua::{Lua, String as LuaString, Table, Value, Variadic};
use ptool_engine::{
    Error as EngineError, ErrorKind as EngineErrorKind, FsCopyOptions, FsGlobOptions,
    FsMkdirOptions, PtoolEngine,
};
use std::path::Path;

const COPY_SIGNATURE: &str = "ptool.fs.copy(src, dst[, options])";
const GLOB_SIGNATURE: &str = "ptool.fs.glob(pattern[, options])";
const MKDIR_SIGNATURE: &str = "ptool.fs.mkdir(path[, options])";

pub(crate) fn read(lua: &Lua, engine: &PtoolEngine, path: String) -> mlua::Result<LuaString> {
    let content = engine
        .fs_read(&path)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.fs.read"))?;
    lua.create_string(&content)
}

pub(crate) fn write(engine: &PtoolEngine, path: String, content: LuaString) -> mlua::Result<()> {
    engine
        .fs_write(&path, content.as_bytes().as_ref())
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.fs.write"))
}

pub(crate) fn mkdir(
    engine: &PtoolEngine,
    path: String,
    options: Option<Table>,
) -> mlua::Result<()> {
    let options = parse_mkdir_options(options)?;
    engine
        .fs_mkdir(&path, options)
        .map_err(|err| fs_mkdir_error(&path, err))
}

pub(crate) fn exists(engine: &PtoolEngine, path: String) -> bool {
    engine.fs_exists(&path)
}

pub(crate) fn glob(
    engine: &PtoolEngine,
    current_dir: &Path,
    lua: &Lua,
    pattern: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let options = parse_glob_options(options)?;
    let matches = engine
        .fs_glob(&pattern, current_dir, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, GLOB_SIGNATURE))?;
    lua.create_sequence_from(matches)
}

pub(crate) fn copy(engine: &PtoolEngine, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
    if !(2..=3).contains(&args.len()) {
        return Err(crate::lua_error::invalid_argument(
            COPY_SIGNATURE,
            "expects 2 or 3 arguments",
        ));
    }

    let src = parse_copy_endpoint(args[0].clone(), "src")?;
    let dst = parse_copy_endpoint(args[1].clone(), "dst")?;
    let options = crate::ssh::parse_transfer_options(args.get(2).cloned(), COPY_SIGNATURE)?;

    let result = match (src, dst) {
        (CopyEndpoint::Local(src), CopyEndpoint::Local(dst)) => {
            let options = FsCopyOptions {
                parents: options.parents,
                overwrite: options.overwrite,
                echo: options.echo,
            };
            let result = engine
                .fs_copy_local(&src, &dst, options)
                .map_err(fs_copy_error)?;
            crate::ssh::TransferResult {
                bytes: result.bytes,
                from: result.from,
                to: result.to,
            }
        }
        (CopyEndpoint::Local(src), CopyEndpoint::Remote(dst)) => {
            dst.connection()
                .upload_path(Path::new(&src), dst.path(), options)?
        }
        (CopyEndpoint::Remote(src), CopyEndpoint::Local(dst)) => {
            src.connection()
                .download_path(src.path(), Path::new(&dst), options)?
        }
        (CopyEndpoint::Remote(_), CopyEndpoint::Remote(_)) => {
            return Err(crate::lua_error::invalid_argument(
                COPY_SIGNATURE,
                "does not support remote-to-remote copies",
            ));
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
        _ => Err(crate::lua_error::invalid_argument(
            COPY_SIGNATURE,
            format!("`{field}` must be a string path or a remote path from `conn:path(...)`"),
        )),
    }
}

fn ensure_non_empty_string(value: &str, field: &str) -> mlua::Result<()> {
    if value.is_empty() {
        return Err(crate::lua_error::invalid_argument(
            COPY_SIGNATURE,
            format!("`{field}` must not be empty"),
        ));
    }
    Ok(())
}

fn parse_mkdir_options(options: Option<Table>) -> mlua::Result<FsMkdirOptions> {
    let mut parsed = FsMkdirOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = match key {
            Value::String(key) => key.to_str()?.to_string(),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    MKDIR_SIGNATURE,
                    "option keys must be strings",
                ));
            }
        };

        match key.as_str() {
            "exist_ok" => match value {
                Value::Boolean(value) => parsed.exist_ok = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        MKDIR_SIGNATURE,
                        "`exist_ok` must be a boolean",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    MKDIR_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn parse_glob_options(options: Option<Table>) -> mlua::Result<FsGlobOptions> {
    let mut parsed = FsGlobOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = match key {
            Value::String(key) => key.to_str()?.to_string(),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    GLOB_SIGNATURE,
                    "option keys must be strings",
                ));
            }
        };

        match key.as_str() {
            "working_dir" => match value {
                Value::String(value) => {
                    let value = value.to_str()?.to_string();
                    if value.is_empty() {
                        return Err(crate::lua_error::invalid_option(
                            GLOB_SIGNATURE,
                            "`working_dir` must not be empty",
                        ));
                    }
                    parsed.working_dir = Some(value);
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        GLOB_SIGNATURE,
                        "`working_dir` must be a string",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    GLOB_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn fs_mkdir_error(path: &str, err: EngineError) -> mlua::Error {
    match err.kind {
        EngineErrorKind::AlreadyExists => crate::lua_error::to_mlua_error(
            crate::lua_error::LuaError::from_engine(err, MKDIR_SIGNATURE).with_path(path),
        ),
        _ => crate::lua_error::lua_error_from_engine(err, MKDIR_SIGNATURE),
    }
}

fn fs_copy_error(err: EngineError) -> mlua::Error {
    crate::lua_error::lua_error_from_engine(err, COPY_SIGNATURE)
}
