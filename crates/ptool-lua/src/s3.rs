use mlua::{Lua, Table, UserData, UserDataMethods, Value};
use ptool_engine::{
    PtoolEngine, S3AclGrants, S3BucketCannedAcl, S3ConnectOptions, S3Connection, S3Entry,
    S3ObjectCannedAcl, S3PutBucketAclOptions, S3PutObjectAclOptions, S3Range, S3ReadOptions,
    S3RequestPayer, S3WriteOptions,
};

const CONNECT_SIGNATURE: &str = "ptool.s3.connect(options)";
const READ_SIGNATURE: &str = "ptool.s3.Connection:read(path[, options])";
const WRITE_SIGNATURE: &str = "ptool.s3.Connection:write(path, content[, options])";
const DELETE_SIGNATURE: &str = "ptool.s3.Connection:delete(path)";
const EXISTS_SIGNATURE: &str = "ptool.s3.Connection:exists(path)";
const LIST_SIGNATURE: &str = "ptool.s3.Connection:list([prefix])";
const STAT_SIGNATURE: &str = "ptool.s3.Connection:stat(path)";
const PUT_BUCKET_ACL_SIGNATURE: &str = "ptool.s3.Connection:put_bucket_acl(options)";
const PUT_OBJECT_ACL_SIGNATURE: &str = "ptool.s3.Connection:put_object_acl(path, options)";

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
        methods.add_method(
            "read",
            |lua, this, (path, options): (String, Option<Table>)| this.read(lua, path, options),
        );
        methods.add_method(
            "write",
            |lua, this, (path, content, options): (String, mlua::String, Option<Table>)| {
                this.write(lua, path, content, options)
            },
        );
        methods.add_method("delete", |_, this, path: String| this.delete(path));
        methods.add_method("exists", |_, this, path: String| this.exists(path));
        methods.add_method("list", |lua, this, prefix: Option<String>| {
            this.list(lua, prefix)
        });
        methods.add_method("stat", |lua, this, path: String| this.stat(lua, path));
        methods.add_method("put_bucket_acl", |_, this, options: Table| {
            this.put_bucket_acl(options)
        });
        methods.add_method(
            "put_object_acl",
            |_, this, (path, options): (String, Table)| this.put_object_acl(path, options),
        );
    }
}

impl LuaS3Connection {
    fn read(&self, lua: &Lua, path: String, options: Option<Table>) -> mlua::Result<mlua::String> {
        let options = parse_read_options(options)?;
        let bytes = self
            .connection
            .read(&path, &options)
            .map_err(|err| s3_error(READ_SIGNATURE, err))?;
        lua.create_string(&bytes)
    }

    fn write(
        &self,
        lua: &Lua,
        path: String,
        content: mlua::String,
        options: Option<Table>,
    ) -> mlua::Result<Table> {
        let options = parse_write_options(options)?;
        let entry = self
            .connection
            .write(&path, content.as_bytes().as_ref(), &options)
            .map_err(|err| s3_error(WRITE_SIGNATURE, err))?;
        s3_entry_to_lua(lua, entry, WRITE_SIGNATURE)
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
            table.raw_set(index + 1, s3_entry_to_lua(lua, entry, LIST_SIGNATURE)?)?;
        }
        Ok(table)
    }

    fn stat(&self, lua: &Lua, path: String) -> mlua::Result<Table> {
        let entry = self
            .connection
            .stat(&path)
            .map_err(|err| s3_error(STAT_SIGNATURE, err))?;
        s3_entry_to_lua(lua, entry, STAT_SIGNATURE)
    }

    fn put_bucket_acl(&self, options: Table) -> mlua::Result<()> {
        let options = parse_put_bucket_acl_options(options)?;
        self.connection
            .put_bucket_acl(&options)
            .map_err(|err| s3_error(PUT_BUCKET_ACL_SIGNATURE, err))
    }

    fn put_object_acl(&self, path: String, options: Table) -> mlua::Result<()> {
        let options = parse_put_object_acl_options(options)?;
        self.connection
            .put_object_acl(&path, &options)
            .map_err(|err| s3_error(PUT_OBJECT_ACL_SIGNATURE, err))
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

fn parse_write_options(options: Option<Table>) -> mlua::Result<S3WriteOptions> {
    let Some(options) = options else {
        return Ok(S3WriteOptions::default());
    };

    validate_option_keys(
        &options,
        WRITE_SIGNATURE,
        &[
            "content_type",
            "cache_control",
            "content_disposition",
            "content_encoding",
            "metadata",
            "if_not_exists",
            "if_match",
            "if_none_match",
        ],
    )?;

    Ok(S3WriteOptions {
        content_type: optional_non_empty_string(&options, "content_type", WRITE_SIGNATURE)?,
        cache_control: optional_non_empty_string(&options, "cache_control", WRITE_SIGNATURE)?,
        content_disposition: optional_non_empty_string(
            &options,
            "content_disposition",
            WRITE_SIGNATURE,
        )?,
        content_encoding: optional_non_empty_string(&options, "content_encoding", WRITE_SIGNATURE)?,
        metadata: optional_string_map(&options, "metadata", WRITE_SIGNATURE)?,
        if_not_exists: options
            .get::<Option<bool>>("if_not_exists")?
            .unwrap_or(false),
        if_match: optional_non_empty_string(&options, "if_match", WRITE_SIGNATURE)?,
        if_none_match: optional_non_empty_string(&options, "if_none_match", WRITE_SIGNATURE)?,
    })
}

fn parse_read_options(options: Option<Table>) -> mlua::Result<S3ReadOptions> {
    let Some(options) = options else {
        return Ok(S3ReadOptions::default());
    };

    validate_option_keys(&options, READ_SIGNATURE, &["range"])?;

    let range = match options.get::<Option<Table>>("range")? {
        Some(range) => {
            validate_option_keys(&range, READ_SIGNATURE, &["start", "end"])?;
            Some(S3Range {
                start: optional_u64(&range, "start", READ_SIGNATURE)?,
                end: optional_u64(&range, "end", READ_SIGNATURE)?,
            })
        }
        None => None,
    };

    Ok(S3ReadOptions { range })
}

fn parse_put_bucket_acl_options(options: Table) -> mlua::Result<S3PutBucketAclOptions> {
    validate_option_keys(
        &options,
        PUT_BUCKET_ACL_SIGNATURE,
        &[
            "acl",
            "expected_bucket_owner",
            "grant_full_control",
            "grant_read",
            "grant_read_acp",
            "grant_write",
            "grant_write_acp",
        ],
    )?;

    let acl = optional_non_empty_string(&options, "acl", PUT_BUCKET_ACL_SIGNATURE)?
        .map(|value| {
            S3BucketCannedAcl::try_from(value.as_str())
                .map_err(|err| s3_error(PUT_BUCKET_ACL_SIGNATURE, err))
        })
        .transpose()?;

    Ok(S3PutBucketAclOptions {
        acl,
        grants: parse_acl_grants(&options, PUT_BUCKET_ACL_SIGNATURE)?,
        expected_bucket_owner: optional_non_empty_string(
            &options,
            "expected_bucket_owner",
            PUT_BUCKET_ACL_SIGNATURE,
        )?,
    })
}

fn parse_put_object_acl_options(options: Table) -> mlua::Result<S3PutObjectAclOptions> {
    validate_option_keys(
        &options,
        PUT_OBJECT_ACL_SIGNATURE,
        &[
            "acl",
            "expected_bucket_owner",
            "grant_full_control",
            "grant_read",
            "grant_read_acp",
            "grant_write",
            "grant_write_acp",
            "version_id",
            "request_payer",
        ],
    )?;

    let acl = optional_non_empty_string(&options, "acl", PUT_OBJECT_ACL_SIGNATURE)?
        .map(|value| {
            S3ObjectCannedAcl::try_from(value.as_str())
                .map_err(|err| s3_error(PUT_OBJECT_ACL_SIGNATURE, err))
        })
        .transpose()?;
    let request_payer =
        optional_non_empty_string(&options, "request_payer", PUT_OBJECT_ACL_SIGNATURE)?
            .map(|value| {
                S3RequestPayer::try_from(value.as_str())
                    .map_err(|err| s3_error(PUT_OBJECT_ACL_SIGNATURE, err))
            })
            .transpose()?;

    Ok(S3PutObjectAclOptions {
        acl,
        grants: parse_acl_grants(&options, PUT_OBJECT_ACL_SIGNATURE)?,
        expected_bucket_owner: optional_non_empty_string(
            &options,
            "expected_bucket_owner",
            PUT_OBJECT_ACL_SIGNATURE,
        )?,
        version_id: optional_non_empty_string(&options, "version_id", PUT_OBJECT_ACL_SIGNATURE)?,
        request_payer,
    })
}

fn parse_acl_grants(options: &Table, signature: &str) -> mlua::Result<S3AclGrants> {
    Ok(S3AclGrants {
        full_control: optional_non_empty_string(options, "grant_full_control", signature)?,
        read: optional_non_empty_string(options, "grant_read", signature)?,
        read_acp: optional_non_empty_string(options, "grant_read_acp", signature)?,
        write: optional_non_empty_string(options, "grant_write", signature)?,
        write_acp: optional_non_empty_string(options, "grant_write_acp", signature)?,
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

fn validate_option_keys(options: &Table, signature: &str, allowed: &[&str]) -> mlua::Result<()> {
    for pair in options.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = match key {
            Value::String(value) => value.to_str()?.to_string(),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    signature,
                    "option keys must be strings",
                ));
            }
        };
        if !allowed.iter().any(|allowed| *allowed == key) {
            return Err(crate::lua_error::invalid_option(
                signature,
                format!("unknown option `{key}`"),
            ));
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

fn optional_string_map(
    options: &Table,
    field: &str,
    signature: &str,
) -> mlua::Result<Option<Vec<(String, String)>>> {
    let Some(value) = options.get::<Option<Table>>(field)? else {
        return Ok(None);
    };

    let mut entries = Vec::new();
    for pair in value.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = match key {
            Value::String(value) => value.to_str()?.to_string(),
            _ => {
                return Err(crate::lua_error::invalid_argument(
                    signature,
                    format!("`{field}` keys must be strings"),
                ));
            }
        };
        let value = match value {
            Value::String(value) => value.to_str()?.to_string(),
            _ => {
                return Err(crate::lua_error::invalid_argument(
                    signature,
                    format!("`{field}` values must be strings"),
                ));
            }
        };
        entries.push((key, value));
    }

    Ok(Some(entries))
}

fn optional_u64(options: &Table, field: &str, signature: &str) -> mlua::Result<Option<u64>> {
    let value = options.get::<Option<i64>>(field)?;
    value
        .map(|value| {
            u64::try_from(value).map_err(|_| {
                crate::lua_error::invalid_argument(
                    signature,
                    format!("`{field}` must be a non-negative integer"),
                )
            })
        })
        .transpose()
}

fn s3_entry_to_lua(lua: &Lua, entry: S3Entry, signature: &str) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("path", entry.path)?;
    table.set("size", lua_size(entry.size, signature)?)?;
    table.set("etag", entry.etag)?;
    table.set("last_modified", entry.last_modified)?;
    table.set("content_type", entry.content_type)?;
    table.set("version", entry.version)?;
    match entry.metadata {
        Some(metadata) => {
            let metadata_table = lua.create_table()?;
            for (key, value) in metadata {
                metadata_table.set(key, value)?;
            }
            table.set("metadata", metadata_table)?;
        }
        None => table.set("metadata", Value::Nil)?,
    }
    table.set("is_file", entry.is_file)?;
    table.set("is_dir", entry.is_dir)?;
    table.set("mode", entry.mode)?;
    Ok(table)
}

fn lua_size(size: u64, op: &str) -> mlua::Result<i64> {
    i64::try_from(size)
        .map_err(|_| crate::lua_error::invalid_argument(op, "`size` exceeds Lua integer range"))
}

fn s3_error(signature: &str, err: ptool_engine::Error) -> mlua::Error {
    crate::lua_error::lua_error_from_engine(err, signature)
}
