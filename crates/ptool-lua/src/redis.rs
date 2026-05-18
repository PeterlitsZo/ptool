use mlua::{Lua, Table, UserData, UserDataMethods, Value, Variadic};
use ptool_engine::{PtoolEngine, RedisArg, RedisConnection, RedisReply};

const CONNECT_SIGNATURE: &str = "ptool.redis.connect(url_or_options)";
const CALL_SIGNATURE: &str = "ptool.redis.Connection:call(command, ...)";
const CLOSE_SIGNATURE: &str = "ptool.redis.Connection:close()";

#[derive(Clone)]
pub(crate) struct LuaRedisConnection {
    connection: RedisConnection,
}

pub(crate) fn connect(value: Value, engine: &PtoolEngine) -> mlua::Result<LuaRedisConnection> {
    let url = parse_connect_value(value)?;
    let connection = engine
        .redis_connect(&url)
        .map_err(|err| redis_error(CONNECT_SIGNATURE, err))?;
    Ok(LuaRedisConnection { connection })
}

impl UserData for LuaRedisConnection {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("call", |lua, this, args: Variadic<Value>| {
            this.call(lua, args)
        });
        methods.add_method("close", |_, this, ()| this.close());
    }
}

impl LuaRedisConnection {
    fn call(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Value> {
        let (command, args) = parse_call_args(args)?;
        let reply = self
            .connection
            .call(&command, &args)
            .map_err(|err| redis_error(CALL_SIGNATURE, err))?;
        redis_reply_to_lua(lua, reply)
    }

    fn close(&self) -> mlua::Result<()> {
        self.connection
            .close()
            .map_err(|err| redis_error(CLOSE_SIGNATURE, err))
    }
}

fn parse_connect_value(value: Value) -> mlua::Result<String> {
    match value {
        Value::String(text) => {
            let text = text.to_str()?.to_string();
            ensure_non_empty(&text, CONNECT_SIGNATURE, "url must not be empty")?;
            Ok(text)
        }
        Value::Table(options) => {
            let Some(url) = options.get::<Option<String>>("url")? else {
                return Err(crate::lua_error::invalid_argument(
                    CONNECT_SIGNATURE,
                    "requires string `url` when called with a table",
                ));
            };
            ensure_non_empty(&url, CONNECT_SIGNATURE, "url must not be empty")?;
            Ok(url)
        }
        _ => Err(crate::lua_error::invalid_argument(
            CONNECT_SIGNATURE,
            "expects a string or a table with `url`",
        )),
    }
}

fn parse_call_args(args: Variadic<Value>) -> mlua::Result<(String, Vec<RedisArg>)> {
    if args.is_empty() {
        return Err(crate::lua_error::invalid_argument(
            CALL_SIGNATURE,
            "requires a command string",
        ));
    }

    let mut values = args.into_iter();
    let command = match values.next().expect("checked above") {
        Value::String(value) => value.to_str()?.to_string(),
        _ => {
            return Err(crate::lua_error::invalid_argument(
                CALL_SIGNATURE,
                "command must be a string",
            ));
        }
    };
    ensure_non_empty(&command, CALL_SIGNATURE, "command must not be empty")?;

    let args = values
        .enumerate()
        .map(|(index, value)| parse_redis_arg(value, index + 1))
        .collect::<mlua::Result<Vec<_>>>()?;
    Ok((command, args))
}

fn parse_redis_arg(value: Value, index: usize) -> mlua::Result<RedisArg> {
    match value {
        Value::Boolean(value) => Ok(RedisArg::Boolean(value)),
        Value::Integer(value) => Ok(RedisArg::Integer(value)),
        Value::Number(value) => {
            if !value.is_finite() {
                return Err(crate::lua_error::invalid_argument(
                    CALL_SIGNATURE,
                    format!("argument #{index} must be a finite number"),
                ));
            }
            Ok(RedisArg::Number(value))
        }
        Value::String(value) => Ok(RedisArg::Bytes(value.as_bytes().to_vec())),
        Value::Nil => Err(crate::lua_error::invalid_argument(
            CALL_SIGNATURE,
            format!("argument #{index} does not support nil"),
        )),
        _ => Err(crate::lua_error::invalid_argument(
            CALL_SIGNATURE,
            format!("argument #{index} only supports boolean/integer/number/string"),
        )),
    }
}

fn redis_reply_to_lua(lua: &Lua, reply: RedisReply) -> mlua::Result<Value> {
    match reply {
        RedisReply::Nil => Ok(Value::Nil),
        RedisReply::Integer(value) => Ok(Value::Integer(value)),
        RedisReply::Bytes(value) => Ok(Value::String(lua.create_string(&value)?)),
        RedisReply::String(value) => Ok(Value::String(lua.create_string(&value)?)),
        RedisReply::Array(values) | RedisReply::Set(values) => {
            let table = lua.create_table()?;
            for (index, value) in values.into_iter().enumerate() {
                table.raw_set(index + 1, redis_reply_to_lua(lua, value)?)?;
            }
            Ok(Value::Table(table))
        }
        RedisReply::Map(entries) => Ok(Value::Table(redis_map_to_lua(lua, entries)?)),
        RedisReply::Number(value) => Ok(Value::Number(value)),
        RedisReply::Boolean(value) => Ok(Value::Boolean(value)),
        RedisReply::BigNumber(value) => Ok(Value::String(lua.create_string(&value)?)),
        RedisReply::Push { kind, data } => {
            let table = lua.create_table()?;
            table.set("kind", kind)?;
            let data_table = lua.create_table()?;
            for (index, value) in data.into_iter().enumerate() {
                data_table.raw_set(index + 1, redis_reply_to_lua(lua, value)?)?;
            }
            table.set("data", data_table)?;
            Ok(Value::Table(table))
        }
    }
}

fn redis_map_to_lua(lua: &Lua, entries: Vec<(RedisReply, RedisReply)>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in entries {
        let key = redis_map_key_to_lua(lua, key)?;
        table.set(key, redis_reply_to_lua(lua, value)?)?;
    }
    Ok(table)
}

fn redis_map_key_to_lua(lua: &Lua, key: RedisReply) -> mlua::Result<Value> {
    match key {
        RedisReply::Integer(value) => Ok(Value::Integer(value)),
        RedisReply::Bytes(value) => Ok(Value::String(lua.create_string(&value)?)),
        RedisReply::String(value) => Ok(Value::String(lua.create_string(&value)?)),
        RedisReply::Number(value) if value.is_finite() => Ok(Value::Number(value)),
        RedisReply::Boolean(value) => Ok(Value::Boolean(value)),
        RedisReply::BigNumber(value) => Ok(Value::String(lua.create_string(&value)?)),
        _ => Err(crate::lua_error::invalid_argument(
            CALL_SIGNATURE,
            "Redis map response contains a key that cannot be represented in Lua",
        )),
    }
}

fn ensure_non_empty(value: &str, signature: &str, detail: &str) -> mlua::Result<()> {
    if value.is_empty() {
        return Err(crate::lua_error::invalid_argument(signature, detail));
    }
    Ok(())
}

fn redis_error(signature: &str, err: ptool_engine::Error) -> mlua::Error {
    crate::lua_error::lua_error_from_engine(err, signature)
}
