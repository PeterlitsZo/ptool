use mlua::{Lua, Table, UserData, UserDataMethods, Value};
use ptool_engine::{PtoolEngine, S3ConnectOptions, S3Connection, S3Entry};

const CONNECT_SIGNATURE: &str = "ptool.s3.connect(options)";
const READ_SIGNATURE: &str = "ptool.s3.Connection:read(path)";
const WRITE_SIGNATURE: &str = "ptool.s3.Connection:write(path, content)";
const DELETE_SIGNATURE: &str = "ptool.s3.Connection:delete(path)";
const EXISTS_SIGNATURE: &str = "ptool.s3.Connection:exists(path)";
const LIST_SIGNATURE: &str = "ptool.s3.Connection:list([prefix])";
const STAT_SIGNATURE: &str = "ptool.s3.Connection:stat(path)";

#[derive(Clone)]
pub(crate) struct LuaS3Connection {
    connection: S3Connection,
}

pub(crate) fn connect(options: Table, engine: &PtoolEngine) -> mlua::Result<LuaS3Connection> {
    let options = parse_connect_options(options)?;
    let connection = engine
        .s3_connect(options)
        .map_err(|err| s3_error(CONNECT_SIGNATURE, err))?;
    Ok(LuaS3Connection { connection })
}

impl UserData for LuaS3Connection {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("read", |lua, this, path: String| this.read(lua, path));
        methods.add_method(
            "write",
            |_, this, (path, content): (String, mlua::String)| this.write(path, content),
        );
        methods.add_method("delete", |_, this, path: String| this.delete(path));
        methods.add_method("exists", |_, this, path: String| this.exists(path));
        methods.add_method("list", |lua, this, prefix: Option<String>| {
            this.list(lua, prefix)
        });
        methods.add_method("stat", |lua, this, path: String| this.stat(lua, path));
    }
}

impl LuaS3Connection {
    fn read(&self, lua: &Lua, path: String) -> mlua::Result<mlua::String> {
        let bytes = self
            .connection
            .read(&path)
            .map_err(|err| s3_error(READ_SIGNATURE, err))?;
        lua.create_string(&bytes)
    }

    fn write(&self, path: String, content: mlua::String) -> mlua::Result<()> {
        self.connection
            .write(&path, content.as_bytes().as_ref())
            .map_err(|err| s3_error(WRITE_SIGNATURE, err))
    }

    fn delete(&self, path: String) -> mlua::Result<()> {
        self.connection
            .delete(&path)
            .map_err(|err| s3_error(DELETE_SIGNATURE, err))
    }

    fn exists(&self, path: String) -> mlua::Result<bool> {
        self.connection
            .exists(&path)
            .map_err(|err| s3_error(EXISTS_SIGNATURE, err))
    }

    fn list(&self, lua: &Lua, prefix: Option<String>) -> mlua::Result<Table> {
        let entries = self
            .connection
            .list(prefix.as_deref().unwrap_or_default())
            .map_err(|err| s3_error(LIST_SIGNATURE, err))?;
        let table = lua.create_table()?;
        for (index, entry) in entries.into_iter().enumerate() {
            table.raw_set(index + 1, s3_entry_to_lua(lua, entry)?)?;
        }
        Ok(table)
    }

    fn stat(&self, lua: &Lua, path: String) -> mlua::Result<Table> {
        let entry = self
            .connection
            .stat(&path)
            .map_err(|err| s3_error(STAT_SIGNATURE, err))?;
        s3_entry_to_lua(lua, entry)
    }
}

fn parse_connect_options(options: Table) -> mlua::Result<S3ConnectOptions> {
    validate_connect_option_keys(&options)?;

    let bucket = required_non_empty_string(&options, "bucket", CONNECT_SIGNATURE)?;
    let region = optional_non_empty_string(&options, "region", CONNECT_SIGNATURE)?;
    let endpoint = optional_non_empty_string(&options, "endpoint", CONNECT_SIGNATURE)?;
    let access_key_id = optional_non_empty_string(&options, "access_key_id", CONNECT_SIGNATURE)?;
    let secret_access_key =
        optional_non_empty_string(&options, "secret_access_key", CONNECT_SIGNATURE)?;
    let session_token = optional_non_empty_string(&options, "session_token", CONNECT_SIGNATURE)?;
    let root = options.get::<Option<String>>("root")?;
    let allow_anonymous = options
        .get::<Option<bool>>("allow_anonymous")?
        .unwrap_or(false);

    Ok(S3ConnectOptions {
        bucket,
        region,
        endpoint,
        access_key_id,
        secret_access_key,
        session_token,
        root,
        allow_anonymous,
    })
}

fn validate_connect_option_keys(options: &Table) -> mlua::Result<()> {
    for pair in options.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = match key {
            Value::String(value) => value.to_str()?.to_string(),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    CONNECT_SIGNATURE,
                    "option keys must be strings",
                ));
            }
        };

        match key.as_str() {
            "bucket" | "region" | "endpoint" | "access_key_id" | "secret_access_key"
            | "session_token" | "root" | "allow_anonymous" => {}
            _ => {
                return Err(crate::lua_error::invalid_option(
                    CONNECT_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(())
}

fn required_non_empty_string(
    options: &Table,
    field: &str,
    signature: &str,
) -> mlua::Result<String> {
    let Some(value) = options.get::<Option<String>>(field)? else {
        return Err(crate::lua_error::invalid_argument(
            signature,
            format!("requires `{field}`"),
        ));
    };
    if value.is_empty() {
        return Err(crate::lua_error::invalid_argument(
            signature,
            format!("`{field}` must not be empty"),
        ));
    }
    Ok(value)
}

fn optional_non_empty_string(
    options: &Table,
    field: &str,
    signature: &str,
) -> mlua::Result<Option<String>> {
    let value = options.get::<Option<String>>(field)?;
    if matches!(value.as_deref(), Some("")) {
        return Err(crate::lua_error::invalid_argument(
            signature,
            format!("`{field}` must not be empty"),
        ));
    }
    Ok(value)
}

fn s3_entry_to_lua(lua: &Lua, entry: S3Entry) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("path", entry.path)?;
    table.set("size", lua_size(entry.size)?)?;
    table.set("etag", entry.etag)?;
    table.set("last_modified", entry.last_modified)?;
    table.set("content_type", entry.content_type)?;
    table.set("is_file", entry.is_file)?;
    table.set("is_dir", entry.is_dir)?;
    table.set("mode", entry.mode)?;
    Ok(table)
}

fn lua_size(size: u64) -> mlua::Result<i64> {
    i64::try_from(size).map_err(|_| {
        crate::lua_error::invalid_argument(STAT_SIGNATURE, "`size` exceeds Lua integer range")
    })
}

fn s3_error(signature: &str, err: ptool_engine::Error) -> mlua::Error {
    crate::lua_error::lua_error_from_engine(err, signature)
}
