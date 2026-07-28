use mlua::{Lua, Value};
use ptool_engine::PtoolEngine;

const GET_SIGNATURE: &str = "ptool.yaml.get(input, path)";
const PARSE_SIGNATURE: &str = "ptool.yaml.parse(input)";
const SET_SIGNATURE: &str = "ptool.yaml.set(input, path, value)";
const STRINGIFY_SIGNATURE: &str = "ptool.yaml.stringify(value)";

pub(crate) fn parse(lua: &Lua, engine: &PtoolEngine, input: Value) -> mlua::Result<Value> {
    let input = parse_input_string(input, PARSE_SIGNATURE)?;
    let parsed = engine
        .yaml_parse(&input)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, PARSE_SIGNATURE))?;
    crate::json::json_value_to_lua(lua, &parsed, "ptool.yaml.parse failed: unsupported number")
}

pub(crate) fn get(
    lua: &Lua,
    engine: &PtoolEngine,
    input: Value,
    path: Value,
) -> mlua::Result<Value> {
    let input = parse_input_string(input, GET_SIGNATURE)?;
    let path = crate::json::parse_path(path, GET_SIGNATURE)?;
    let value = engine
        .yaml_get(&input, &path)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, GET_SIGNATURE))?;

    match value {
        Some(value) => {
            crate::json::json_value_to_lua(lua, &value, "ptool.yaml.get failed: unsupported number")
        }
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
    let path = crate::json::parse_path(path, SET_SIGNATURE)?;
    let value =
        crate::json::lua_value_to_json(lua, value, &format!("{SET_SIGNATURE} invalid value"))?;

    engine
        .yaml_set(&input, &path, &value)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, SET_SIGNATURE))
}

pub(crate) fn stringify(lua: &Lua, engine: &PtoolEngine, value: Value) -> mlua::Result<String> {
    let value = crate::json::lua_value_to_json(
        lua,
        value,
        &format!("{STRINGIFY_SIGNATURE} invalid value"),
    )?;
    engine
        .yaml_stringify(&value)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, STRINGIFY_SIGNATURE))
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
