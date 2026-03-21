use mlua::{Lua, Table, Value};
use toml_edit::{DocumentMut, Item};

const PARSE_SIGNATURE: &str = "ptool.toml.parse(input)";
const GET_SIGNATURE: &str = "ptool.toml.get(input, path)";
const SET_SIGNATURE: &str = "ptool.toml.set(input, path, value)";
const REMOVE_SIGNATURE: &str = "ptool.toml.remove(input, path)";

pub(crate) fn parse(lua: &Lua, input: Value) -> mlua::Result<Table> {
    let input = parse_input_string(input, PARSE_SIGNATURE)?;

    let parsed: ::toml::Table = ::toml::from_str(&input)
        .map_err(|err| mlua::Error::runtime(format!("ptool.toml.parse failed: {err}")))?;

    match toml_table_to_lua(lua, &parsed)? {
        Value::Table(table) => Ok(table),
        _ => Err(mlua::Error::runtime(
            "ptool.toml.parse failed: root value must be a table",
        )),
    }
}

pub(crate) fn get(lua: &Lua, input: Value, path: Value) -> mlua::Result<Value> {
    let input = parse_input_string(input, GET_SIGNATURE)?;
    let path = parse_path(path, GET_SIGNATURE)?;
    let parsed: ::toml::Table = ::toml::from_str(&input)
        .map_err(|err| mlua::Error::runtime(format!("ptool.toml.get failed: {err}")))?;

    let Some(value) = get_toml_value_by_path(&parsed, &path) else {
        return Ok(Value::Nil);
    };
    toml_value_to_lua(lua, value)
}

pub(crate) fn set(input: Value, path: Value, value: Value) -> mlua::Result<String> {
    let input = parse_input_string(input, SET_SIGNATURE)?;
    let path = parse_path(path, SET_SIGNATURE)?;
    let value = lua_to_toml_edit_value(value, SET_SIGNATURE)?;
    let mut doc = parse_toml_edit_document(&input, SET_SIGNATURE)?;
    set_toml_edit_value(&mut doc, &path, Item::Value(value), SET_SIGNATURE)?;
    Ok(doc.to_string())
}

pub(crate) fn remove(input: Value, path: Value) -> mlua::Result<String> {
    let input = parse_input_string(input, REMOVE_SIGNATURE)?;
    let path = parse_path(path, REMOVE_SIGNATURE)?;
    let mut doc = parse_toml_edit_document(&input, REMOVE_SIGNATURE)?;
    remove_toml_edit_value(&mut doc, &path, REMOVE_SIGNATURE)?;
    Ok(doc.to_string())
}

fn toml_value_to_lua(lua: &Lua, value: &::toml::Value) -> mlua::Result<Value> {
    match value {
        ::toml::Value::String(value) => Ok(Value::String(lua.create_string(value)?)),
        ::toml::Value::Integer(value) => Ok(Value::Integer(*value)),
        ::toml::Value::Float(value) => Ok(Value::Number(*value)),
        ::toml::Value::Boolean(value) => Ok(Value::Boolean(*value)),
        ::toml::Value::Datetime(value) => Ok(Value::String(lua.create_string(value.to_string())?)),
        ::toml::Value::Array(values) => toml_array_to_lua(lua, values),
        ::toml::Value::Table(values) => toml_table_to_lua(lua, values),
    }
}

fn toml_array_to_lua(lua: &Lua, values: &[::toml::Value]) -> mlua::Result<Value> {
    let table = lua.create_table()?;
    for (index, value) in values.iter().enumerate() {
        table.raw_set(index + 1, toml_value_to_lua(lua, value)?)?;
    }
    Ok(Value::Table(table))
}

fn toml_table_to_lua(
    lua: &Lua,
    values: &::toml::map::Map<String, ::toml::Value>,
) -> mlua::Result<Value> {
    let table = lua.create_table()?;
    for (key, value) in values {
        table.raw_set(key.as_str(), toml_value_to_lua(lua, value)?)?;
    }
    Ok(Value::Table(table))
}

fn parse_input_string(input: Value, signature: &str) -> mlua::Result<String> {
    match input {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(mlua::Error::runtime(format!(
            "{signature} requires string input"
        ))),
    }
}

fn parse_path(path: Value, signature: &str) -> mlua::Result<Vec<String>> {
    let path = match path {
        Value::Table(path) => path,
        _ => {
            return Err(mlua::Error::runtime(format!(
                "{signature} requires path as an array of strings"
            )));
        }
    };

    let len = path.raw_len();
    if len == 0 {
        return Err(mlua::Error::runtime(format!(
            "{signature} path must not be empty"
        )));
    }

    let mut keys = Vec::with_capacity(len);
    for index in 1..=len {
        let key = match path.raw_get::<Value>(index)? {
            Value::String(key) => key.to_str()?.to_string(),
            _ => {
                return Err(mlua::Error::runtime(format!(
                    "{signature} path[{index}] must be a string"
                )));
            }
        };
        if key.is_empty() {
            return Err(mlua::Error::runtime(format!(
                "{signature} path[{index}] must not be empty"
            )));
        }
        keys.push(key);
    }
    Ok(keys)
}

fn get_toml_value_by_path<'a>(
    table: &'a ::toml::Table,
    path: &[String],
) -> Option<&'a ::toml::Value> {
    let mut value = table.get(path.first()?.as_str())?;
    for key in &path[1..] {
        let next_table = value.as_table()?;
        value = next_table.get(key.as_str())?;
    }
    Some(value)
}

fn lua_to_toml_edit_value(value: Value, signature: &str) -> mlua::Result<toml_edit::Value> {
    match value {
        Value::String(value) => Ok(toml_edit::Value::from(value.to_str()?.to_string())),
        Value::Integer(value) => Ok(toml_edit::Value::from(value)),
        Value::Number(value) => {
            if !value.is_finite() {
                return Err(mlua::Error::runtime(format!(
                    "{signature} `value` must be a finite number"
                )));
            }
            Ok(toml_edit::Value::from(value))
        }
        Value::Boolean(value) => Ok(toml_edit::Value::from(value)),
        _ => Err(mlua::Error::runtime(format!(
            "{signature} `value` only supports string/integer/number/boolean"
        ))),
    }
}

fn parse_toml_edit_document(input: &str, signature: &str) -> mlua::Result<DocumentMut> {
    input
        .parse::<DocumentMut>()
        .map_err(|err| mlua::Error::runtime(format!("{signature} failed: {err}")))
}

fn set_toml_edit_value(
    doc: &mut DocumentMut,
    path: &[String],
    value: Item,
    signature: &str,
) -> mlua::Result<()> {
    let Some(last_key) = path.last() else {
        return Err(mlua::Error::runtime(format!(
            "{signature} path must not be empty"
        )));
    };

    let mut table = doc.as_table_mut();
    for key in &path[..path.len() - 1] {
        if !table.contains_key(key.as_str()) {
            table.insert(key.as_str(), toml_edit::table());
        }

        let Some(next_item) = table.get_mut(key.as_str()) else {
            return Err(mlua::Error::runtime(format!(
                "{signature} failed: path segment `{key}` does not exist"
            )));
        };
        let Some(next_table) = next_item.as_table_mut() else {
            return Err(mlua::Error::runtime(format!(
                "{signature} failed: path segment `{key}` is not a table"
            )));
        };
        table = next_table;
    }

    table.insert(last_key.as_str(), value);
    Ok(())
}

fn remove_toml_edit_value(
    doc: &mut DocumentMut,
    path: &[String],
    signature: &str,
) -> mlua::Result<()> {
    let Some(last_key) = path.last() else {
        return Err(mlua::Error::runtime(format!(
            "{signature} path must not be empty"
        )));
    };

    if path.len() == 1 {
        doc.as_table_mut().remove(last_key.as_str());
        return Ok(());
    }

    let mut table = doc.as_table_mut();
    for key in &path[..path.len() - 1] {
        let Some(next_item) = table.get_mut(key.as_str()) else {
            return Ok(());
        };
        let Some(next_table) = next_item.as_table_mut() else {
            return Err(mlua::Error::runtime(format!(
                "{signature} failed: path segment `{key}` is not a table"
            )));
        };
        table = next_table;
    }

    table.remove(last_key.as_str());
    Ok(())
}
