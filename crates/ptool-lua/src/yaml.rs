use mlua::{Lua, Value};
use ptool_engine::{PtoolEngine, YamlPathSegment};

const GET_SIGNATURE: &str = "ptool.yaml.get(input, path)";
const PARSE_SIGNATURE: &str = "ptool.yaml.parse(input)";
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
    let path = parse_path(path, GET_SIGNATURE)?;
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

fn parse_path(path: Value, signature: &str) -> mlua::Result<Vec<YamlPathSegment>> {
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
                YamlPathSegment::Key(key)
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
                YamlPathSegment::Index(value - 1)
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
