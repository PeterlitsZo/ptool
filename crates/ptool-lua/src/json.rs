use mlua::{Lua, LuaSerdeExt, Table, Value};
use serde_json::Value as JsonValue;

const PARSE_SIGNATURE: &str = "ptool.json.parse(input)";
const STRINGIFY_SIGNATURE: &str = "ptool.json.stringify(value[, options])";

pub(crate) fn parse(lua: &Lua, input: Value) -> mlua::Result<Value> {
    let input = parse_input_string(input, PARSE_SIGNATURE)?;
    let parsed: JsonValue = serde_json::from_str(&input)
        .map_err(|err| mlua::Error::runtime(format!("ptool.json.parse failed: {err}")))?;
    json_value_to_lua(lua, &parsed, "ptool.json.parse failed: unsupported number")
}

pub(crate) fn stringify(lua: &Lua, value: Value, options: Option<Table>) -> mlua::Result<String> {
    let options = parse_stringify_options(options)?;
    let value = lua_value_to_json(lua, value, &format!("{STRINGIFY_SIGNATURE} invalid value"))?;

    let result = if options.pretty {
        serde_json::to_string_pretty(&value)
    } else {
        serde_json::to_string(&value)
    };
    result.map_err(|err| mlua::Error::runtime(format!("ptool.json.stringify failed: {err}")))
}

pub(crate) fn lua_value_to_json(
    lua: &Lua,
    value: Value,
    error_prefix: &str,
) -> mlua::Result<JsonValue> {
    lua.from_value(value)
        .map_err(|err| mlua::Error::runtime(format!("{error_prefix}: {err}")))
}

pub(crate) fn json_value_to_lua(
    lua: &Lua,
    value: &JsonValue,
    unsupported_number_message: &str,
) -> mlua::Result<Value> {
    match value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(value) => Ok(Value::Boolean(*value)),
        JsonValue::Number(value) => json_number_to_lua(value, unsupported_number_message),
        JsonValue::String(value) => Ok(Value::String(lua.create_string(value)?)),
        JsonValue::Array(values) => {
            let table = lua.create_table()?;
            for (index, item) in values.iter().enumerate() {
                table.raw_set(
                    index + 1,
                    json_value_to_lua(lua, item, unsupported_number_message)?,
                )?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(values) => {
            let table = lua.create_table()?;
            for (key, item) in values {
                table.raw_set(
                    key.as_str(),
                    json_value_to_lua(lua, item, unsupported_number_message)?,
                )?;
            }
            Ok(Value::Table(table))
        }
    }
}

fn parse_input_string(input: Value, signature: &str) -> mlua::Result<String> {
    match input {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(mlua::Error::runtime(format!(
            "{signature} requires string input"
        ))),
    }
}

fn parse_stringify_options(options: Option<Table>) -> mlua::Result<StringifyOptions> {
    let mut parsed = StringifyOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = match key {
            Value::String(value) => value.to_str()?.to_string(),
            _ => {
                return Err(mlua::Error::runtime(format!(
                    "{STRINGIFY_SIGNATURE} option keys must be strings"
                )));
            }
        };

        match key.as_str() {
            "pretty" => match value {
                Value::Boolean(value) => parsed.pretty = value,
                _ => {
                    return Err(mlua::Error::runtime(format!(
                        "{STRINGIFY_SIGNATURE} `pretty` must be a boolean"
                    )));
                }
            },
            _ => {
                return Err(mlua::Error::runtime(format!(
                    "{STRINGIFY_SIGNATURE} unknown option `{key}`"
                )));
            }
        }
    }

    Ok(parsed)
}

fn json_number_to_lua(
    value: &serde_json::Number,
    unsupported_number_message: &str,
) -> mlua::Result<Value> {
    if let Some(number) = value.as_i64() {
        return Ok(Value::Integer(number));
    }
    if let Some(number) = value.as_u64() {
        if let Ok(number) = i64::try_from(number) {
            return Ok(Value::Integer(number));
        }
        return Ok(Value::Number(number as f64));
    }
    if let Some(number) = value.as_f64() {
        return Ok(Value::Number(number));
    }
    Err(mlua::Error::runtime(unsupported_number_message))
}

#[derive(Default)]
struct StringifyOptions {
    pretty: bool,
}
