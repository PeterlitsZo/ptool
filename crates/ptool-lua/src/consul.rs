use mlua::{Lua, Table, UserData, UserDataMethods, Value};
use ptool_engine::{
    ConsulConnectOptions, ConsulConnection, ConsulDeleteOptions, ConsulGetOptions, ConsulKvEntry,
    ConsulListOptions, ConsulPutOptions, HttpRequestOptions, PtoolEngine,
};

const CONNECT_SIGNATURE: &str = "ptool.consul.connect(options)";
const GET_SIGNATURE: &str = "ptool.consul.Connection:get(key[, options])";
const PUT_SIGNATURE: &str = "ptool.consul.Connection:put(key, value[, options])";
const DELETE_SIGNATURE: &str = "ptool.consul.Connection:delete(key[, options])";
const LIST_SIGNATURE: &str = "ptool.consul.Connection:list(prefix[, options])";
const REQUEST_SIGNATURE: &str = "ptool.consul.Connection:request(options)";

#[derive(Clone)]
pub(crate) struct LuaConsulConnection {
    connection: ConsulConnection,
}

pub(crate) fn connect(options: Table, engine: &PtoolEngine) -> mlua::Result<LuaConsulConnection> {
    let options = parse_connect_options(options)?;
    let connection = engine
        .consul_connect(options)
        .map_err(|err| consul_error(CONNECT_SIGNATURE, err))?;
    Ok(LuaConsulConnection { connection })
}

impl UserData for LuaConsulConnection {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "get",
            |lua, this, (key, options): (String, Option<Table>)| this.get(lua, key, options),
        );
        methods.add_method(
            "put",
            |_, this, (key, value, options): (String, mlua::String, Option<Table>)| {
                this.put(key, value, options)
            },
        );
        methods.add_method(
            "delete",
            |_, this, (key, options): (String, Option<Table>)| this.delete(key, options),
        );
        methods.add_method(
            "list",
            |lua, this, (prefix, options): (String, Option<Table>)| this.list(lua, prefix, options),
        );
        methods.add_method("request", |lua, this, options: Table| {
            this.request(lua, options)
        });
    }
}

impl LuaConsulConnection {
    fn get(&self, lua: &Lua, key: String, options: Option<Table>) -> mlua::Result<Value> {
        let options = parse_get_options(options, GET_SIGNATURE)?;
        let entry = self
            .connection
            .get(&key, &options)
            .map_err(|err| consul_error(GET_SIGNATURE, err))?;
        match entry {
            Some(entry) => Ok(Value::Table(kv_entry_to_lua(lua, entry)?)),
            None => Ok(Value::Nil),
        }
    }

    fn put(&self, key: String, value: mlua::String, options: Option<Table>) -> mlua::Result<bool> {
        let options = parse_put_options(options)?;
        self.connection
            .put(&key, value.as_bytes().as_ref(), &options)
            .map_err(|err| consul_error(PUT_SIGNATURE, err))
    }

    fn delete(&self, key: String, options: Option<Table>) -> mlua::Result<bool> {
        let options = parse_delete_options(options)?;
        self.connection
            .delete(&key, &options)
            .map_err(|err| consul_error(DELETE_SIGNATURE, err))
    }

    fn list(&self, lua: &Lua, prefix: String, options: Option<Table>) -> mlua::Result<Table> {
        let options = parse_list_options(options, LIST_SIGNATURE)?;
        let items = self
            .connection
            .list(&prefix, &options)
            .map_err(|err| consul_error(LIST_SIGNATURE, err))?;
        let table = lua.create_table()?;
        for (index, entry) in items.into_iter().enumerate() {
            table.raw_set(index + 1, kv_entry_to_lua(lua, entry)?)?;
        }
        Ok(table)
    }

    fn request(&self, lua: &Lua, options: Table) -> mlua::Result<crate::http::HttpResponse> {
        let options = parse_request_options(lua, &self.connection, options)?;
        let response = self
            .connection
            .request(options)
            .map_err(|err| consul_error(REQUEST_SIGNATURE, err))?;
        Ok(crate::http::HttpResponse::from_engine(response))
    }
}

fn parse_connect_options(options: Table) -> mlua::Result<ConsulConnectOptions> {
    validate_option_keys(
        &options,
        CONNECT_SIGNATURE,
        &["address", "token", "datacenter", "timeout_ms"],
    )?;

    let address = options
        .get::<Option<String>>("address")?
        .unwrap_or_default();
    let token = optional_non_empty_string(&options, "token", CONNECT_SIGNATURE)?;
    let datacenter = optional_non_empty_string(&options, "datacenter", CONNECT_SIGNATURE)?;
    let timeout_ms = options.get::<Option<i64>>("timeout_ms")?;
    Ok(ConsulConnectOptions {
        address,
        token,
        datacenter,
        timeout_ms,
    })
}

fn parse_get_options(options: Option<Table>, signature: &str) -> mlua::Result<ConsulGetOptions> {
    let Some(options) = options else {
        return Ok(ConsulGetOptions::default());
    };
    validate_option_keys(
        &options,
        signature,
        &["consistent", "stale", "wait_index", "wait_time"],
    )?;
    Ok(ConsulGetOptions {
        consistent: options.get::<Option<bool>>("consistent")?.unwrap_or(false),
        stale: options.get::<Option<bool>>("stale")?.unwrap_or(false),
        wait_index: optional_u64(&options, "wait_index", signature)?,
        wait_time: optional_non_empty_string(&options, "wait_time", signature)?,
    })
}

fn parse_list_options(options: Option<Table>, signature: &str) -> mlua::Result<ConsulListOptions> {
    let options = parse_get_options(options, signature)?;
    Ok(ConsulListOptions {
        consistent: options.consistent,
        stale: options.stale,
        wait_index: options.wait_index,
        wait_time: options.wait_time,
    })
}

fn parse_put_options(options: Option<Table>) -> mlua::Result<ConsulPutOptions> {
    let Some(options) = options else {
        return Ok(ConsulPutOptions::default());
    };
    validate_option_keys(&options, PUT_SIGNATURE, &["flags", "cas"])?;
    Ok(ConsulPutOptions {
        flags: optional_u64(&options, "flags", PUT_SIGNATURE)?,
        cas: optional_u64(&options, "cas", PUT_SIGNATURE)?,
    })
}

fn parse_delete_options(options: Option<Table>) -> mlua::Result<ConsulDeleteOptions> {
    let Some(options) = options else {
        return Ok(ConsulDeleteOptions::default());
    };
    validate_option_keys(&options, DELETE_SIGNATURE, &["cas", "recurse"])?;
    Ok(ConsulDeleteOptions {
        cas: optional_u64(&options, "cas", DELETE_SIGNATURE)?,
        recurse: options.get::<Option<bool>>("recurse")?.unwrap_or(false),
    })
}

fn parse_request_options(
    lua: &Lua,
    connection: &ConsulConnection,
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
            .map_err(|err| consul_error(REQUEST_SIGNATURE, err))?,
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

fn kv_entry_to_lua(lua: &Lua, entry: ConsulKvEntry) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("key", entry.key)?;
    match entry.value {
        Some(value) => table.set("value", lua.create_string(&value)?)?,
        None => table.set("value", Value::Nil)?,
    }
    table.set("flags", lua_integer(entry.flags, GET_SIGNATURE, "flags")?)?;
    table.set(
        "create_index",
        lua_integer(entry.create_index, GET_SIGNATURE, "create_index")?,
    )?;
    table.set(
        "modify_index",
        lua_integer(entry.modify_index, GET_SIGNATURE, "modify_index")?,
    )?;
    table.set(
        "lock_index",
        lua_integer(entry.lock_index, GET_SIGNATURE, "lock_index")?,
    )?;
    table.set("session", entry.session)?;
    table.set("namespace", entry.namespace)?;
    table.set("partition", entry.partition)?;
    Ok(table)
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

fn lua_integer(value: u64, signature: &str, field: &str) -> mlua::Result<i64> {
    i64::try_from(value).map_err(|_| {
        crate::lua_error::invalid_argument(
            signature,
            format!("`{field}` exceeds Lua integer range"),
        )
    })
}

fn consul_error(signature: &str, err: ptool_engine::Error) -> mlua::Error {
    crate::lua_error::lua_error_from_engine(err, signature)
}
