use mlua::{Lua, String as LuaString, Table, UserData, UserDataMethods, Value, Variadic};
use ptool_engine::{
    Error as EngineError, ErrorKind as EngineErrorKind, FsCopyOptions, FsGlobOptions,
    FsMkdirOptions, FsOpenOptions, FsRemoveOptions, FsSeekWhence, PtoolEngine,
};
use std::path::Path;

const OPEN_SIGNATURE: &str = "ptool.fs.open(path[, mode])";
const COPY_SIGNATURE: &str = "ptool.fs.copy(src, dst[, options])";
const GLOB_SIGNATURE: &str = "ptool.fs.glob(pattern[, options])";
const MKDIR_SIGNATURE: &str = "ptool.fs.mkdir(path[, options])";
const REMOVE_SIGNATURE: &str = "ptool.fs.remove(path[, options])";
const FILE_READ_SIGNATURE: &str = "ptool.fs.File:read([n])";
const FILE_WRITE_SIGNATURE: &str = "ptool.fs.File:write(content)";
const FILE_FLUSH_SIGNATURE: &str = "ptool.fs.File:flush()";
const FILE_SEEK_SIGNATURE: &str = "ptool.fs.File:seek([whence[, offset]])";
const FILE_CLOSE_SIGNATURE: &str = "ptool.fs.File:close()";

pub(crate) struct LuaFsFile {
    file: ptool_engine::FsFileHandle,
}

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

pub(crate) fn append(engine: &PtoolEngine, path: String, content: LuaString) -> mlua::Result<()> {
    engine
        .fs_append(&path, content.as_bytes().as_ref())
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.fs.append"))
}

pub(crate) fn open(
    engine: &PtoolEngine,
    path: String,
    mode: Option<String>,
) -> mlua::Result<LuaFsFile> {
    let mode = mode.unwrap_or_else(|| "r".to_string());
    let options = FsOpenOptions::parse(&mode)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, OPEN_SIGNATURE))?;
    let file = engine
        .fs_open(&path, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, OPEN_SIGNATURE))?;
    Ok(LuaFsFile { file })
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

pub(crate) fn is_file(engine: &PtoolEngine, path: String) -> bool {
    engine.fs_is_file(&path)
}

pub(crate) fn is_dir(engine: &PtoolEngine, path: String) -> bool {
    engine.fs_is_dir(&path)
}

pub(crate) fn remove(
    engine: &PtoolEngine,
    path: String,
    options: Option<Table>,
) -> mlua::Result<()> {
    let options = parse_remove_options(options)?;
    engine
        .fs_remove(&path, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REMOVE_SIGNATURE))
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

impl UserData for LuaFsFile {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("read", |lua, this, value: Option<Value>| {
            this.read(lua, value)
        });
        methods.add_method("write", |_, this, content: LuaString| this.write(content));
        methods.add_method("flush", |_, this, ()| this.flush());
        methods.add_method("seek", |_, this, args: Variadic<Value>| this.seek(args));
        methods.add_method("close", |_, this, ()| this.close());
    }
}

impl LuaFsFile {
    fn read(&self, lua: &Lua, value: Option<Value>) -> mlua::Result<LuaString> {
        let len = parse_read_len(value)?;
        let content = self
            .file
            .read(len)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, FILE_READ_SIGNATURE))?;
        lua.create_string(&content)
    }

    fn write(&self, content: LuaString) -> mlua::Result<()> {
        self.file
            .write(content.as_bytes().as_ref())
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, FILE_WRITE_SIGNATURE))
    }

    fn flush(&self) -> mlua::Result<()> {
        self.file
            .flush()
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, FILE_FLUSH_SIGNATURE))
    }

    fn seek(&self, args: Variadic<Value>) -> mlua::Result<i64> {
        let (whence, offset) = parse_seek_args(args)?;
        let position = self
            .file
            .seek(whence, offset)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, FILE_SEEK_SIGNATURE))?;
        i64::try_from(position).map_err(|_| {
            crate::lua_error::invalid_argument(
                FILE_SEEK_SIGNATURE,
                "result exceeds Lua integer range",
            )
        })
    }

    fn close(&self) -> mlua::Result<()> {
        self.file
            .close()
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, FILE_CLOSE_SIGNATURE))
    }
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

fn parse_remove_options(options: Option<Table>) -> mlua::Result<FsRemoveOptions> {
    let mut parsed = FsRemoveOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = match key {
            Value::String(key) => key.to_str()?.to_string(),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    REMOVE_SIGNATURE,
                    "option keys must be strings",
                ));
            }
        };

        match key.as_str() {
            "recursive" => match value {
                Value::Boolean(value) => parsed.recursive = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        REMOVE_SIGNATURE,
                        "`recursive` must be a boolean",
                    ));
                }
            },
            "missing_ok" => match value {
                Value::Boolean(value) => parsed.missing_ok = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        REMOVE_SIGNATURE,
                        "`missing_ok` must be a boolean",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    REMOVE_SIGNATURE,
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

fn parse_read_len(value: Option<Value>) -> mlua::Result<Option<usize>> {
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        Value::Nil => Ok(None),
        Value::Integer(value) => {
            let value = usize::try_from(value).map_err(|_| {
                crate::lua_error::invalid_argument(FILE_READ_SIGNATURE, "`n` must be >= 0")
            })?;
            Ok(Some(value))
        }
        _ => Err(crate::lua_error::invalid_argument(
            FILE_READ_SIGNATURE,
            "`n` must be an integer",
        )),
    }
}

fn parse_seek_args(args: Variadic<Value>) -> mlua::Result<(FsSeekWhence, i64)> {
    if args.len() > 2 {
        return Err(crate::lua_error::invalid_argument(
            FILE_SEEK_SIGNATURE,
            "expects at most 2 arguments",
        ));
    }

    let whence = match args.first() {
        None => FsSeekWhence::Cur,
        Some(Value::Nil) => FsSeekWhence::Cur,
        Some(Value::String(value)) => match value.to_str()?.as_ref() {
            "set" => FsSeekWhence::Set,
            "cur" => FsSeekWhence::Cur,
            "end" => FsSeekWhence::End,
            _ => {
                return Err(crate::lua_error::invalid_argument(
                    FILE_SEEK_SIGNATURE,
                    "`whence` must be `set`, `cur`, or `end`",
                ));
            }
        },
        Some(_) => {
            return Err(crate::lua_error::invalid_argument(
                FILE_SEEK_SIGNATURE,
                "`whence` must be a string",
            ));
        }
    };

    let offset = match args.get(1) {
        None | Some(Value::Nil) => 0,
        Some(Value::Integer(value)) => *value,
        Some(_) => {
            return Err(crate::lua_error::invalid_argument(
                FILE_SEEK_SIGNATURE,
                "`offset` must be an integer",
            ));
        }
    };

    Ok((whence, offset))
}
