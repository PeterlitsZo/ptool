use mlua::{Lua, LuaSerdeExt, Table, Value};
use ptool_engine::{JsonPathSegment, JsonStringifyOptions, JsonValue, PtoolEngine};

const GET_SIGNATURE: &str = "ptool.json.get(input, path)";
const PARSE_SIGNATURE: &str = "ptool.json.parse(input)";
const SET_SIGNATURE: &str = "ptool.json.set(input, path, value)";
const STRINGIFY_SIGNATURE: &str = "ptool.json.stringify(value[, options])";

pub(crate) fn parse(lua: &Lua, engine: &PtoolEngine, input: Value) -> mlua::Result<Value> {
    let input = parse_input_string(input, PARSE_SIGNATURE)?;
    let parsed = engine
        .json_parse(&input)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, PARSE_SIGNATURE))?;
    json_value_to_lua(lua, &parsed, "ptool.json.parse failed: unsupported number")
}

pub(crate) fn get(
    lua: &Lua,
    engine: &PtoolEngine,
    input: Value,
    path: Value,
) -> mlua::Result<Value> {
    let input = parse_input_string(input, GET_SIGNATURE)?;
    let path = parse_path(path, GET_SIGNATURE)?;
    let value = engine
        .json_get(&input, &path)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, GET_SIGNATURE))?;

    match value {
        Some(value) => json_value_to_lua(lua, &value, "ptool.json.get failed: unsupported number"),
        None => Ok(Value::Nil),
    }
}

pub(crate) fn set(
    lua: &Lua,
    engine: &PtoolEngine,
    input: Value,
    path: Value,
    value: Value,
) -> mlua::Result<String> {
    let input = parse_input_string(input, SET_SIGNATURE)?;
    let path = parse_path(path, SET_SIGNATURE)?;
    let value = lua_value_to_json(lua, value, &format!("{SET_SIGNATURE} invalid value"))?;

    engine
        .json_set(&input, &path, &value)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, SET_SIGNATURE))
}

pub(crate) fn stringify(
    lua: &Lua,
    engine: &PtoolEngine,
    value: Value,
    options: Option<Table>,
) -> mlua::Result<String> {
    let options = parse_stringify_options(options)?;
    let value = lua_value_to_json(lua, value, &format!("{STRINGIFY_SIGNATURE} invalid value"))?;
    engine
        .json_stringify(&value, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, STRINGIFY_SIGNATURE))
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

pub(crate) fn parse_path(path: Value, signature: &str) -> mlua::Result<Vec<JsonPathSegment>> {
    let path = match path {
        Value::Table(path) => path,
        _ => {
            return Err(crate::lua_error::invalid_argument(
                signature,
                "requires path as an array of strings and positive integer indexes",
            ));
        }
    };

    let len = path.raw_len();
    if len == 0 {
        return Err(crate::lua_error::invalid_argument(
            signature,
            "path must not be empty",
        ));
    }

    let mut segments = Vec::with_capacity(len);
    for index in 1..=len {
        let segment = path.raw_get::<Value>(index)?;
        let segment = match segment {
            Value::String(key) => {
                let key = key.to_str()?.to_string();
                if key.is_empty() {
                    return Err(crate::lua_error::invalid_argument(
                        signature,
                        format!("path[{index}] must not be empty"),
                    ));
                }
                JsonPathSegment::Key(key)
            }
            Value::Integer(value) => {
                let value = usize::try_from(value).map_err(|_| {
                    crate::lua_error::invalid_argument(
                        signature,
                        format!("path[{index}] must be a positive integer"),
                    )
                })?;
                if value == 0 {
                    return Err(crate::lua_error::invalid_argument(
                        signature,
                        format!("path[{index}] must be a positive integer"),
                    ));
                }
                JsonPathSegment::Index(value - 1)
            }
            _ => {
                return Err(crate::lua_error::invalid_argument(
                    signature,
                    format!("path[{index}] must be a string or positive integer"),
                ));
            }
        };
        segments.push(segment);
    }

    Ok(segments)
}

fn parse_input_string(input: Value, signature: &str) -> mlua::Result<String> {
    match input {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(mlua::Error::runtime(format!(
            "{signature} requires string input"
        ))),
    }
}

fn parse_stringify_options(options: Option<Table>) -> mlua::Result<JsonStringifyOptions> {
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

type StringifyOptions = JsonStringifyOptions;
