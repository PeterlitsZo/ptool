use mlua::{Function, Lua, Table, Value, Variadic};
use std::collections::BTreeMap;

const MAP_SIGNATURE: &str = "ptool.tbl.map(list, fn)";
const FILTER_SIGNATURE: &str = "ptool.tbl.filter(list, fn)";
const CONCAT_SIGNATURE: &str = "ptool.tbl.concat(...)";
const JOIN_SIGNATURE: &str = "ptool.tbl.join(...)";

pub(crate) fn map(lua: &Lua, list: Table, callback: Function) -> mlua::Result<Table> {
    let values = collect_list_values(&list, MAP_SIGNATURE, "list")?;
    let mapped = lua.create_table()?;

    for (offset, value) in values.into_iter().enumerate() {
        let index = offset + 1;
        let mapped_value: Value = callback.call((value, to_lua_index(index, MAP_SIGNATURE)?))?;
        if matches!(mapped_value, Value::Nil) {
            return Err(mlua::Error::runtime(format!(
                "{MAP_SIGNATURE} `fn` must not return nil for `list[{index}]`"
            )));
        }
        mapped.raw_set(index, mapped_value)?;
    }

    Ok(mapped)
}

pub(crate) fn filter(lua: &Lua, list: Table, callback: Function) -> mlua::Result<Table> {
    let values = collect_list_values(&list, FILTER_SIGNATURE, "list")?;
    let filtered = lua.create_table()?;
    let mut next_index = 1usize;

    for (offset, value) in values.into_iter().enumerate() {
        let index = offset + 1;
        let keep: Value = callback.call((value.clone(), to_lua_index(index, FILTER_SIGNATURE)?))?;
        if is_truthy(&keep) {
            filtered.raw_set(next_index, value)?;
            next_index += 1;
        }
    }

    Ok(filtered)
}

pub(crate) fn concat(lua: &Lua, lists: Variadic<Value>) -> mlua::Result<Table> {
    concat_impl(lua, lists, CONCAT_SIGNATURE)
}

pub(crate) fn join(lua: &Lua, lists: Variadic<Value>) -> mlua::Result<Table> {
    concat_impl(lua, lists, JOIN_SIGNATURE)
}

fn concat_impl(lua: &Lua, lists: Variadic<Value>, signature: &str) -> mlua::Result<Table> {
    let joined = lua.create_table()?;
    let mut next_index = 1usize;

    for (offset, value) in lists.into_iter().enumerate() {
        let Value::Table(list) = value else {
            let arg_index = offset + 1;
            return Err(mlua::Error::runtime(format!(
                "{signature} argument #{arg_index} must be a list table"
            )));
        };

        let values = collect_list_values(&list, signature, &format!("argument #{}", offset + 1))?;
        for value in values {
            joined.raw_set(next_index, value)?;
            next_index += 1;
        }
    }

    Ok(joined)
}

fn collect_list_values(table: &Table, signature: &str, arg_name: &str) -> mlua::Result<Vec<Value>> {
    let mut entries = BTreeMap::new();

    for pair in table.clone().pairs::<Value, Value>() {
        let (key, value) = pair?;
        let index = parse_list_key(key, signature, arg_name)?;
        entries.insert(index, value);
    }

    if entries.is_empty() {
        return Ok(Vec::new());
    }

    let max_index = *entries.keys().next_back().expect("entries is not empty");
    if max_index != entries.len() {
        return Err(mlua::Error::runtime(format!(
            "{signature} `{arg_name}` must be a dense list with contiguous integer keys starting at 1"
        )));
    }

    Ok(entries.into_values().collect())
}

fn parse_list_key(key: Value, signature: &str, arg_name: &str) -> mlua::Result<usize> {
    let index = match key {
        Value::Integer(index) => index,
        _ => {
            return Err(mlua::Error::runtime(format!(
                "{signature} `{arg_name}` must be a list table with only integer keys"
            )));
        }
    };

    if index < 1 {
        return Err(mlua::Error::runtime(format!(
            "{signature} `{arg_name}` must be a list table with keys starting at 1"
        )));
    }

    usize::try_from(index).map_err(|_| {
        mlua::Error::runtime(format!(
            "{signature} `{arg_name}` contains an index that is too large"
        ))
    })
}

fn to_lua_index(index: usize, signature: &str) -> mlua::Result<i64> {
    i64::try_from(index)
        .map_err(|_| mlua::Error::runtime(format!("{signature} list index is too large")))
}

fn is_truthy(value: &Value) -> bool {
    !matches!(value, Value::Nil | Value::Boolean(false))
}
