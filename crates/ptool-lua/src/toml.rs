use mlua::{Lua, Table, Value};
use ptool_engine::{PtoolEngine, TomlPathSegment, TomlValue};
use std::collections::BTreeMap;

const GET_SIGNATURE: &str = "ptool.toml.get(input, path)";
const PARSE_SIGNATURE: &str = "ptool.toml.parse(input)";
const REMOVE_SIGNATURE: &str = "ptool.toml.remove(input, path)";
const SET_SIGNATURE: &str = "ptool.toml.set(input, path, value)";
const STRINGIFY_SIGNATURE: &str = "ptool.toml.stringify(value)";

pub(crate) fn parse(lua: &Lua, engine: &PtoolEngine, input: Value) -> mlua::Result<Table> {
    let input = parse_input_string(input, PARSE_SIGNATURE)?;
    let parsed = engine
        .toml_parse(&input)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, PARSE_SIGNATURE))?;

    match toml_value_to_lua(lua, &parsed)? {
        Value::Table(table) => Ok(table),
        _ => Err(crate::lua_error::invalid_argument(
            PARSE_SIGNATURE,
            "root value must be a table",
        )),
    }
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
        .toml_get(&input, &path)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, GET_SIGNATURE))?;

    match value {
        Some(value) => toml_value_to_lua(lua, &value),
        None => Ok(Value::Nil),
    }
}

pub(crate) fn set(
    engine: &PtoolEngine,
    input: Value,
    path: Value,
    value: Value,
) -> mlua::Result<String> {
    let input = parse_input_string(input, SET_SIGNATURE)?;
    let path = parse_path(path, SET_SIGNATURE)?;
    let value = lua_value_to_toml(value, SET_SIGNATURE)?;

    engine
        .toml_set(&input, &path, &value)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, SET_SIGNATURE))
}

pub(crate) fn remove(engine: &PtoolEngine, input: Value, path: Value) -> mlua::Result<String> {
    let input = parse_input_string(input, REMOVE_SIGNATURE)?;
    let path = parse_path(path, REMOVE_SIGNATURE)?;

    engine
        .toml_remove(&input, &path)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REMOVE_SIGNATURE))
}

pub(crate) fn stringify(engine: &PtoolEngine, value: Value) -> mlua::Result<String> {
    let value = lua_value_to_toml(value, STRINGIFY_SIGNATURE)?;
    engine
        .toml_stringify(&value)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, STRINGIFY_SIGNATURE))
}

fn toml_value_to_lua(lua: &Lua, value: &TomlValue) -> mlua::Result<Value> {
    match value {
        TomlValue::String(value) => Ok(Value::String(lua.create_string(value)?)),
        TomlValue::Integer(value) => Ok(Value::Integer(*value)),
        TomlValue::Float(value) => Ok(Value::Number(*value)),
        TomlValue::Boolean(value) => Ok(Value::Boolean(*value)),
        TomlValue::Datetime(value) => Ok(Value::String(lua.create_string(value)?)),
        TomlValue::Array(values) => toml_array_to_lua(lua, values),
        TomlValue::Table(values) => toml_table_to_lua(lua, values),
    }
}

fn toml_array_to_lua(lua: &Lua, values: &[TomlValue]) -> mlua::Result<Value> {
    let table = lua.create_table()?;
    for (index, value) in values.iter().enumerate() {
        table.raw_set(index + 1, toml_value_to_lua(lua, value)?)?;
    }
    Ok(Value::Table(table))
}

fn toml_table_to_lua(lua: &Lua, values: &BTreeMap<String, TomlValue>) -> mlua::Result<Value> {
    let table = lua.create_table()?;
    for (key, value) in values {
        table.raw_set(key.as_str(), toml_value_to_lua(lua, value)?)?;
    }
    Ok(Value::Table(table))
}

fn parse_input_string(input: Value, signature: &str) -> mlua::Result<String> {
    match input {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_argument(
            signature,
            "requires string input",
        )),
    }
}

fn parse_path(path: Value, signature: &str) -> mlua::Result<Vec<TomlPathSegment>> {
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
                TomlPathSegment::Key(key)
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
                TomlPathSegment::Index(value - 1)
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

fn lua_value_to_toml(value: Value, signature: &str) -> mlua::Result<TomlValue> {
    match value {
        Value::String(value) => Ok(TomlValue::String(value.to_str()?.to_string())),
        Value::Integer(value) => Ok(TomlValue::Integer(value)),
        Value::Number(value) => {
            if !value.is_finite() {
                return Err(crate::lua_error::invalid_argument(
                    signature,
                    "`value` must be a finite number",
                ));
            }
            Ok(TomlValue::Float(value))
        }
        Value::Boolean(value) => Ok(TomlValue::Boolean(value)),
        Value::Table(value) => lua_table_to_toml(value, signature),
        _ => Err(crate::lua_error::invalid_argument(
            signature,
            "`value` only supports string/integer/number/boolean/table",
        )),
    }
}

fn lua_table_to_toml(table: Table, signature: &str) -> mlua::Result<TomlValue> {
    let raw_len = table.raw_len();
    let mut pair_count = 0usize;
    let mut has_string_keys = false;
    let mut has_non_string_keys = false;

    for pair in table.pairs::<Value, Value>() {
        let (key, _) = pair?;
        pair_count += 1;
        match key {
            Value::String(_) => has_string_keys = true,
            _ => has_non_string_keys = true,
        }
    }

    if has_string_keys {
        if has_non_string_keys {
            return Err(crate::lua_error::invalid_argument(
                signature,
                "Lua tables used as TOML tables must only have string keys",
            ));
        }

        let mut values = BTreeMap::new();
        for pair in table.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let Value::String(key) = key else {
                return Err(crate::lua_error::invalid_argument(
                    signature,
                    "Lua tables used as TOML tables must only have string keys",
                ));
            };
            values.insert(
                key.to_str()?.to_string(),
                lua_value_to_toml(value, signature)?,
            );
        }
        return Ok(TomlValue::Table(values));
    }

    if raw_len == 0 {
        return Ok(TomlValue::Table(BTreeMap::new()));
    }

    if pair_count != raw_len {
        return Err(crate::lua_error::invalid_argument(
            signature,
            "Lua arrays used as TOML arrays must be dense sequences starting at 1",
        ));
    }

    let mut values = Vec::with_capacity(raw_len);
    for index in 1..=raw_len {
        let value = table.raw_get::<Value>(index)?;
        if matches!(value, Value::Nil) {
            return Err(crate::lua_error::invalid_argument(
                signature,
                "Lua arrays used as TOML arrays must be dense sequences starting at 1",
            ));
        }
        values.push(lua_value_to_toml(value, signature)?);
    }
    Ok(TomlValue::Array(values))
}
