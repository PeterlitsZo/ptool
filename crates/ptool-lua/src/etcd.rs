use mlua::{Lua, Table, UserData, UserDataMethods, Value};
use ptool_engine::{
    EtcdConnectOptions, EtcdConnection, EtcdDeleteOptions, EtcdDeleteResponse, EtcdGetOptions,
    EtcdKvEntry, EtcdListOptions, EtcdListResponse, EtcdPutOptions, EtcdPutResponse,
    EtcdResponseHeader, HttpRequestOptions, PtoolEngine,
};

const CONNECT_SIGNATURE: &str = "ptool.etcd.connect(options)";
const GET_SIGNATURE: &str = "ptool.etcd.Connection:get(key[, options])";
const PUT_SIGNATURE: &str = "ptool.etcd.Connection:put(key, value[, options])";
const DELETE_SIGNATURE: &str = "ptool.etcd.Connection:delete(key[, options])";
const LIST_SIGNATURE: &str = "ptool.etcd.Connection:list([prefix[, options]])";
const REQUEST_SIGNATURE: &str = "ptool.etcd.Connection:request(options)";

#[derive(Clone)]
pub(crate) struct LuaEtcdConnection {
    connection: EtcdConnection,
}

pub(crate) fn connect(options: Table, engine: &PtoolEngine) -> mlua::Result<LuaEtcdConnection> {
    let options = parse_connect_options(options)?;
    let connection = engine
        .etcd_connect(options)
        .map_err(|err| etcd_error(CONNECT_SIGNATURE, err))?;
    Ok(LuaEtcdConnection { connection })
}

impl UserData for LuaEtcdConnection {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "get",
            |lua, this, (key, options): (mlua::String, Option<Table>)| this.get(lua, key, options),
        );
        methods.add_method(
            "put",
            |lua, this, (key, value, options): (mlua::String, mlua::String, Option<Table>)| {
                this.put(lua, key, value, options)
            },
        );
        methods.add_method(
            "delete",
            |lua, this, (key, options): (mlua::String, Option<Table>)| {
                this.delete(lua, key, options)
            },
        );
        methods.add_method("list", |lua, this, args: mlua::MultiValue| {
            this.list(lua, args)
        });
        methods.add_method("request", |lua, this, options: Table| {
            this.request(lua, options)
        });
    }
}

impl LuaEtcdConnection {
    fn get(&self, lua: &Lua, key: mlua::String, options: Option<Table>) -> mlua::Result<Value> {
        let options = parse_get_options(options)?;
        let entry = self
            .connection
            .get(key.as_bytes().as_ref(), &options)
            .map_err(|err| etcd_error(GET_SIGNATURE, err))?;
        match entry {
            Some(entry) => Ok(Value::Table(kv_entry_to_lua(lua, entry)?)),
            None => Ok(Value::Nil),
        }
    }

    fn put(
        &self,
        lua: &Lua,
        key: mlua::String,
        value: mlua::String,
        options: Option<Table>,
    ) -> mlua::Result<Table> {
        let options = parse_put_options(options)?;
        let response = self
            .connection
            .put(key.as_bytes().as_ref(), value.as_bytes().as_ref(), &options)
            .map_err(|err| etcd_error(PUT_SIGNATURE, err))?;
        put_response_to_lua(lua, response)
    }

    fn delete(&self, lua: &Lua, key: mlua::String, options: Option<Table>) -> mlua::Result<Table> {
        let options = parse_delete_options(options)?;
        let response = self
            .connection
            .delete(key.as_bytes().as_ref(), &options)
            .map_err(|err| etcd_error(DELETE_SIGNATURE, err))?;
        delete_response_to_lua(lua, response)
    }

    fn list(&self, lua: &Lua, args: mlua::MultiValue) -> mlua::Result<Table> {
        let (prefix, options) = parse_list_args(args)?;
        let response = self
            .connection
            .list(prefix.as_deref().unwrap_or(&[]), &options)
            .map_err(|err| etcd_error(LIST_SIGNATURE, err))?;
        list_response_to_lua(lua, response)
    }

    fn request(&self, lua: &Lua, options: Table) -> mlua::Result<crate::http::HttpResponse> {
        let options = parse_request_options(lua, &self.connection, options)?;
        let response = self
            .connection
            .request(options)
            .map_err(|err| etcd_error(REQUEST_SIGNATURE, err))?;
        Ok(crate::http::HttpResponse::from_engine(response))
    }
}

fn parse_connect_options(options: Table) -> mlua::Result<EtcdConnectOptions> {
    validate_option_keys(
        &options,
        CONNECT_SIGNATURE,
        &[
            "address",
            "token",
            "username",
            "password",
            "api_prefix",
            "timeout_ms",
        ],
    )?;

    let address = options
        .get::<Option<String>>("address")?
        .unwrap_or_default();
    let token = optional_non_empty_string(&options, "token", CONNECT_SIGNATURE)?;
    let username = optional_non_empty_string(&options, "username", CONNECT_SIGNATURE)?;
    let password = optional_non_empty_string(&options, "password", CONNECT_SIGNATURE)?;
    let api_prefix = optional_non_empty_string(&options, "api_prefix", CONNECT_SIGNATURE)?;
    let timeout_ms = options.get::<Option<i64>>("timeout_ms")?;
    Ok(EtcdConnectOptions {
        address,
        token,
        username,
        password,
        api_prefix,
        timeout_ms,
    })
}

fn parse_get_options(options: Option<Table>) -> mlua::Result<EtcdGetOptions> {
    let Some(options) = options else {
        return Ok(EtcdGetOptions::default());
    };
    validate_option_keys(&options, GET_SIGNATURE, &["revision", "serializable"])?;
    Ok(EtcdGetOptions {
        revision: options.get::<Option<i64>>("revision")?,
        serializable: options
            .get::<Option<bool>>("serializable")?
            .unwrap_or(false),
    })
}

fn parse_put_options(options: Option<Table>) -> mlua::Result<EtcdPutOptions> {
    let Some(options) = options else {
        return Ok(EtcdPutOptions::default());
    };
    validate_option_keys(
        &options,
        PUT_SIGNATURE,
        &["lease", "prev_kv", "ignore_value", "ignore_lease"],
    )?;
    Ok(EtcdPutOptions {
        lease: options.get::<Option<i64>>("lease")?,
        prev_kv: options.get::<Option<bool>>("prev_kv")?.unwrap_or(false),
        ignore_value: options
            .get::<Option<bool>>("ignore_value")?
            .unwrap_or(false),
        ignore_lease: options
            .get::<Option<bool>>("ignore_lease")?
            .unwrap_or(false),
    })
}

fn parse_delete_options(options: Option<Table>) -> mlua::Result<EtcdDeleteOptions> {
    let Some(options) = options else {
        return Ok(EtcdDeleteOptions::default());
    };
    validate_option_keys(&options, DELETE_SIGNATURE, &["prefix", "prev_kv"])?;
    Ok(EtcdDeleteOptions {
        prefix: options.get::<Option<bool>>("prefix")?.unwrap_or(false),
        prev_kv: options.get::<Option<bool>>("prev_kv")?.unwrap_or(false),
    })
}

fn parse_list_args(args: mlua::MultiValue) -> mlua::Result<(Option<Vec<u8>>, EtcdListOptions)> {
    match args.len() {
        0 => Ok((None, EtcdListOptions::default())),
        1 => match args.front() {
            Some(Value::String(prefix)) => {
                Ok((Some(prefix.as_bytes().to_vec()), EtcdListOptions::default()))
            }
            Some(Value::Table(options)) => Ok((None, parse_list_options(Some(options.clone()))?)),
            Some(_) => Err(crate::lua_error::invalid_argument(
                LIST_SIGNATURE,
                "expected a prefix string or options table",
            )),
            None => unreachable!("validated length"),
        },
        2 => {
            let mut values = args.into_iter();
            let prefix = match values.next() {
                Some(Value::String(prefix)) => Some(prefix.as_bytes().to_vec()),
                Some(_) => {
                    return Err(crate::lua_error::invalid_argument(
                        LIST_SIGNATURE,
                        "first argument must be a prefix string",
                    ));
                }
                None => unreachable!("validated length"),
            };
            let options = match values.next() {
                Some(Value::Table(options)) => parse_list_options(Some(options))?,
                Some(_) => {
                    return Err(crate::lua_error::invalid_argument(
                        LIST_SIGNATURE,
                        "second argument must be an options table",
                    ));
                }
                None => unreachable!("validated length"),
            };
            Ok((prefix, options))
        }
        _ => Err(crate::lua_error::invalid_argument(
            LIST_SIGNATURE,
            "accepts at most two arguments",
        )),
    }
}

fn parse_list_options(options: Option<Table>) -> mlua::Result<EtcdListOptions> {
    let Some(options) = options else {
        return Ok(EtcdListOptions::default());
    };
    validate_option_keys(
        &options,
        LIST_SIGNATURE,
        &[
            "limit",
            "revision",
            "serializable",
            "keys_only",
            "count_only",
            "min_mod_revision",
            "max_mod_revision",
            "min_create_revision",
            "max_create_revision",
            "sort_order",
            "sort_target",
        ],
    )?;
    Ok(EtcdListOptions {
        limit: options.get::<Option<i64>>("limit")?,
        revision: options.get::<Option<i64>>("revision")?,
        serializable: options
            .get::<Option<bool>>("serializable")?
            .unwrap_or(false),
        keys_only: options.get::<Option<bool>>("keys_only")?.unwrap_or(false),
        count_only: options.get::<Option<bool>>("count_only")?.unwrap_or(false),
        min_mod_revision: options.get::<Option<i64>>("min_mod_revision")?,
        max_mod_revision: options.get::<Option<i64>>("max_mod_revision")?,
        min_create_revision: options.get::<Option<i64>>("min_create_revision")?,
        max_create_revision: options.get::<Option<i64>>("max_create_revision")?,
        sort_order: optional_non_empty_string(&options, "sort_order", LIST_SIGNATURE)?,
        sort_target: optional_non_empty_string(&options, "sort_target", LIST_SIGNATURE)?,
    })
}

fn parse_request_options(
    lua: &Lua,
    connection: &EtcdConnection,
    options: Table,
) -> mlua::Result<HttpRequestOptions> {
    validate_option_keys(
        &options,
        REQUEST_SIGNATURE,
        &[
            "path",
            "url",
            "method",
            "headers",
            "body",
            "query",
            "json",
            "form",
            "timeout_ms",
            "connect_timeout_ms",
            "follow_redirects",
            "max_redirects",
            "user_agent",
            "basic_auth",
            "bearer_token",
            "fail_on_http_error",
        ],
    )?;

    let path = options.get::<Option<String>>("path")?;
    let url = options.get::<Option<String>>("url")?;
    if path.is_some() == url.is_some() {
        return Err(crate::lua_error::invalid_argument(
            REQUEST_SIGNATURE,
            "requires exactly one of `path` or `url`",
        ));
    }

    let url = match (path, url) {
        (Some(path), None) => connection
            .build_url(&path)
            .map_err(|err| etcd_error(REQUEST_SIGNATURE, err))?,
        (None, Some(url)) => url,
        _ => unreachable!("validated above"),
    };

    let normalized = lua.create_table()?;
    normalized.set("url", url)?;
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let Value::String(key) = key else {
            return Err(crate::lua_error::invalid_option(
                REQUEST_SIGNATURE,
                "option keys must be strings",
            ));
        };
        let key = key.to_str()?.to_string();
        if key == "path" || key == "url" {
            continue;
        }
        normalized.set(key, value)?;
    }
    crate::http::parse_request_options(lua, normalized, REQUEST_SIGNATURE)
}

fn kv_entry_to_lua(lua: &Lua, entry: EtcdKvEntry) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("key", lua.create_string(&entry.key)?)?;
    table.set("value", lua.create_string(&entry.value)?)?;
    table.set("create_revision", entry.create_revision)?;
    table.set("mod_revision", entry.mod_revision)?;
    table.set("version", entry.version)?;
    table.set("lease", entry.lease)?;
    Ok(table)
}

fn response_header_to_lua(lua: &Lua, header: EtcdResponseHeader) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("cluster_id", header.cluster_id)?;
    table.set("member_id", header.member_id)?;
    table.set("revision", header.revision)?;
    table.set("raft_term", header.raft_term)?;
    Ok(table)
}

fn put_response_to_lua(lua: &Lua, response: EtcdPutResponse) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    set_optional_header(lua, &table, response.header)?;
    match response.prev_kv {
        Some(prev_kv) => table.set("prev_kv", kv_entry_to_lua(lua, prev_kv)?)?,
        None => table.set("prev_kv", Value::Nil)?,
    }
    Ok(table)
}

fn delete_response_to_lua(lua: &Lua, response: EtcdDeleteResponse) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    set_optional_header(lua, &table, response.header)?;
    table.set("deleted", response.deleted)?;
    let prev_kvs = lua.create_table()?;
    for (index, kv) in response.prev_kvs.into_iter().enumerate() {
        prev_kvs.raw_set(index + 1, kv_entry_to_lua(lua, kv)?)?;
    }
    table.set("prev_kvs", prev_kvs)?;
    Ok(table)
}

fn list_response_to_lua(lua: &Lua, response: EtcdListResponse) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    set_optional_header(lua, &table, response.header)?;
    let kvs = lua.create_table()?;
    for (index, kv) in response.kvs.into_iter().enumerate() {
        kvs.raw_set(index + 1, kv_entry_to_lua(lua, kv)?)?;
    }
    table.set("kvs", kvs)?;
    table.set("count", response.count)?;
    table.set("more", response.more)?;
    Ok(table)
}

fn set_optional_header(
    lua: &Lua,
    table: &Table,
    header: Option<EtcdResponseHeader>,
) -> mlua::Result<()> {
    match header {
        Some(header) => table.set("header", response_header_to_lua(lua, header)?)?,
        None => table.set("header", Value::Nil)?,
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

fn etcd_error(signature: &str, err: ptool_engine::Error) -> mlua::Error {
    crate::lua_error::lua_error_from_engine(err, signature)
}
