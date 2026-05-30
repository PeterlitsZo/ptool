use mlua::{Function, Lua, Table, Value, Variadic};
use std::collections::BTreeMap;

const ANY_SIGNATURE: &str = "ptool.tbl.any(list, fn)";
const ALL_SIGNATURE: &str = "ptool.tbl.all(list, fn)";
const FIND_SIGNATURE: &str = "ptool.tbl.find(list, fn)";
const FIND_INDEX_SIGNATURE: &str = "ptool.tbl.find_index(list, fn)";
const COUNT_SIGNATURE: &str = "ptool.tbl.count(list, fn)";
const INCLUDES_SIGNATURE: &str = "ptool.tbl.includes(list, value)";
const REDUCE_SIGNATURE: &str = "ptool.tbl.reduce(list, initial, fn)";
const FLAT_MAP_SIGNATURE: &str = "ptool.tbl.flat_map(list, fn)";
const MAP_SIGNATURE: &str = "ptool.tbl.map(list, fn)";
const FILTER_SIGNATURE: &str = "ptool.tbl.filter(list, fn)";
const CONCAT_SIGNATURE: &str = "ptool.tbl.concat(...)";
const JOIN_SIGNATURE: &str = "ptool.tbl.join(...)";

pub(crate) fn any(_lua: &Lua, list: Table, callback: Function) -> mlua::Result<bool> {
    let values = collect_list_values(&list, ANY_SIGNATURE, "list")?;

    for (offset, value) in values.into_iter().enumerate() {
        let index = offset + 1;
        let matched: Value = callback.call((value, to_lua_index(index, ANY_SIGNATURE)?))?;
        if is_truthy(&matched) {
            return Ok(true);
        }
    }

    Ok(false)
}

pub(crate) fn all(_lua: &Lua, list: Table, callback: Function) -> mlua::Result<bool> {
    let values = collect_list_values(&list, ALL_SIGNATURE, "list")?;

    for (offset, value) in values.into_iter().enumerate() {
        let index = offset + 1;
        let matched: Value = callback.call((value, to_lua_index(index, ALL_SIGNATURE)?))?;
        if !is_truthy(&matched) {
            return Ok(false);
        }
    }

    Ok(true)
}

pub(crate) fn find(_lua: &Lua, list: Table, callback: Function) -> mlua::Result<Value> {
    let values = collect_list_values(&list, FIND_SIGNATURE, "list")?;

    for (offset, value) in values.into_iter().enumerate() {
        let index = offset + 1;
        let matched: Value =
            callback.call((value.clone(), to_lua_index(index, FIND_SIGNATURE)?))?;
        if is_truthy(&matched) {
            return Ok(value);
        }
    }

    Ok(Value::Nil)
}

pub(crate) fn find_index(_lua: &Lua, list: Table, callback: Function) -> mlua::Result<Value> {
    let values = collect_list_values(&list, FIND_INDEX_SIGNATURE, "list")?;

    for (offset, value) in values.into_iter().enumerate() {
        let index = offset + 1;
        let matched: Value = callback.call((value, to_lua_index(index, FIND_INDEX_SIGNATURE)?))?;
        if is_truthy(&matched) {
            return Ok(Value::Integer(to_lua_index(index, FIND_INDEX_SIGNATURE)?));
        }
    }

    Ok(Value::Nil)
}

pub(crate) fn count(_lua: &Lua, list: Table, callback: Function) -> mlua::Result<i64> {
    let values = collect_list_values(&list, COUNT_SIGNATURE, "list")?;
    let mut total = 0i64;

    for (offset, value) in values.into_iter().enumerate() {
        let index = offset + 1;
        let matched: Value = callback.call((value, to_lua_index(index, COUNT_SIGNATURE)?))?;
        if is_truthy(&matched) {
            total += 1;
        }
    }

    Ok(total)
}

pub(crate) fn includes(_lua: &Lua, list: Table, needle: Value) -> mlua::Result<bool> {
    let values = collect_list_values(&list, INCLUDES_SIGNATURE, "list")?;

    for value in values {
        if value.equals(&needle)? {
            return Ok(true);
        }
    }

    Ok(false)
}

pub(crate) fn reduce(
    _lua: &Lua,
    list: Table,
    initial: Value,
    callback: Function,
) -> mlua::Result<Value> {
    let values = collect_list_values(&list, REDUCE_SIGNATURE, "list")?;
    let mut acc = initial;

    for (offset, value) in values.into_iter().enumerate() {
        let index = offset + 1;
        let next: Value = callback.call((acc, value, to_lua_index(index, REDUCE_SIGNATURE)?))?;
        if matches!(next, Value::Nil) {
            return Err(mlua::Error::runtime(format!(
                "{REDUCE_SIGNATURE} `fn` must not return nil for `list[{index}]`"
            )));
        }
        acc = next;
    }

    Ok(acc)
}

pub(crate) fn flat_map(lua: &Lua, list: Table, callback: Function) -> mlua::Result<Table> {
    let values = collect_list_values(&list, FLAT_MAP_SIGNATURE, "list")?;
    let flattened = lua.create_table()?;
    let mut next_index = 1usize;

    for (offset, value) in values.into_iter().enumerate() {
        let index = offset + 1;
        let mapped: Value = callback.call((value, to_lua_index(index, FLAT_MAP_SIGNATURE)?))?;
        let Value::Table(mapped_list) = mapped else {
            return Err(mlua::Error::runtime(format!(
                "{FLAT_MAP_SIGNATURE} `fn` must return a list table for `list[{index}]`"
            )));
        };

        let mapped_values = collect_list_values(
            &mapped_list,
            FLAT_MAP_SIGNATURE,
            &format!("fn result for list[{index}]"),
        )?;

        for mapped_value in mapped_values {
            flattened.raw_set(next_index, mapped_value)?;
            next_index += 1;
        }
    }

    Ok(flattened)
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn any_short_circuits_and_empty_lists_are_false() -> mlua::Result<()> {
        let lua = Lua::new();
        let list = int_list(&lua, &[1, 2, 3])?;
        let calls = Rc::new(Cell::new(0usize));
        let calls_for_callback = Rc::clone(&calls);
        let callback = lua.create_function(move |_, (value, _index): (i64, i64)| {
            calls_for_callback.set(calls_for_callback.get() + 1);
            Ok(value == 2)
        })?;

        assert!(any(&lua, list, callback)?);
        assert_eq!(calls.get(), 2);

        let empty = lua.create_table()?;
        let empty_calls = Rc::new(Cell::new(0usize));
        let empty_calls_for_callback = Rc::clone(&empty_calls);
        let empty_callback = lua.create_function(move |_, (_value, _index): (Value, i64)| {
            empty_calls_for_callback.set(empty_calls_for_callback.get() + 1);
            Ok(true)
        })?;

        assert!(!any(&lua, empty, empty_callback)?);
        assert_eq!(empty_calls.get(), 0);
        Ok(())
    }

    #[test]
    fn all_short_circuits_and_empty_lists_are_true() -> mlua::Result<()> {
        let lua = Lua::new();
        let list = int_list(&lua, &[2, 4, 5, 6])?;
        let calls = Rc::new(Cell::new(0usize));
        let calls_for_callback = Rc::clone(&calls);
        let callback = lua.create_function(move |_, (value, _index): (i64, i64)| {
            calls_for_callback.set(calls_for_callback.get() + 1);
            Ok(value % 2 == 0)
        })?;

        assert!(!all(&lua, list, callback)?);
        assert_eq!(calls.get(), 3);

        let empty = lua.create_table()?;
        let empty_calls = Rc::new(Cell::new(0usize));
        let empty_calls_for_callback = Rc::clone(&empty_calls);
        let empty_callback = lua.create_function(move |_, (_value, _index): (Value, i64)| {
            empty_calls_for_callback.set(empty_calls_for_callback.get() + 1);
            Ok(false)
        })?;

        assert!(all(&lua, empty, empty_callback)?);
        assert_eq!(empty_calls.get(), 0);
        Ok(())
    }

    #[test]
    fn find_and_find_index_cover_matches_and_misses() -> mlua::Result<()> {
        let lua = Lua::new();
        let list = int_list(&lua, &[10, 20, 30])?;
        let callback = lua.create_function(|_, (value, _index): (i64, i64)| Ok(value >= 20))?;

        let found = find(&lua, list.clone(), callback)?;
        assert_eq!(found, Value::Integer(20));

        let index_callback =
            lua.create_function(|_, (value, _index): (i64, i64)| Ok(value >= 20))?;
        let found_index = find_index(&lua, list.clone(), index_callback)?;
        assert_eq!(found_index, Value::Integer(2));

        let missing_callback = lua.create_function(|_, (_value, _index): (i64, i64)| Ok(false))?;
        let missing = find(&lua, list.clone(), missing_callback)?;
        assert_eq!(missing, Value::Nil);

        let missing_index_callback =
            lua.create_function(|_, (_value, _index): (i64, i64)| Ok(false))?;
        let missing_index = find_index(&lua, list, missing_index_callback)?;
        assert_eq!(missing_index, Value::Nil);
        Ok(())
    }

    #[test]
    fn count_uses_lua_truthiness() -> mlua::Result<()> {
        let lua = Lua::new();
        let list = int_list(&lua, &[1, 2, 3, 4])?;
        let callback = lua.create_function(|lua, (value, _index): (i64, i64)| {
            let result = match value {
                1 => Value::Integer(0),
                2 => Value::Boolean(false),
                3 => Value::Nil,
                _ => Value::String(lua_string(lua, "keep")?),
            };
            Ok(result)
        })?;

        assert_eq!(count(&lua, list, callback)?, 2);
        Ok(())
    }

    #[test]
    fn includes_matches_and_rejects_non_matching_values() -> mlua::Result<()> {
        let lua = Lua::new();
        let shared = lua.create_table()?;
        shared.raw_set("name", "same")?;
        let list = lua.create_table()?;
        list.raw_set(1, 1)?;
        list.raw_set(2, "1")?;
        list.raw_set(3, shared.clone())?;

        assert!(includes(&lua, list.clone(), Value::Integer(1))?);
        assert!(includes(&lua, list.clone(), Value::Table(shared))?);
        assert!(!includes(&lua, list, Value::Number(2.0))?);
        Ok(())
    }

    #[test]
    fn reduce_accumulates_in_order_and_rejects_nil_returns() -> mlua::Result<()> {
        let lua = Lua::new();
        let list = int_list(&lua, &[1, 2, 3])?;
        let callback =
            lua.create_function(|_, (acc, value, _index): (i64, i64, i64)| Ok(acc * 10 + value))?;

        let reduced = reduce(&lua, list.clone(), Value::Integer(0), callback)?;
        assert_eq!(reduced, Value::Integer(123));

        let nil_callback =
            lua.create_function(|_, (_acc, _value, _index): (Value, i64, i64)| Ok(Value::Nil))?;
        let err = reduce(&lua, list, Value::Integer(0), nil_callback).unwrap_err();
        assert_runtime_error_contains(
            err,
            "ptool.tbl.reduce(list, initial, fn) `fn` must not return nil for `list[1]`",
        );
        Ok(())
    }

    #[test]
    fn flat_map_flattens_in_order_and_rejects_invalid_results() -> mlua::Result<()> {
        let lua = Lua::new();
        let list = int_list(&lua, &[1, 2])?;
        let callback = lua.create_function(|lua, (value, _index): (i64, i64)| {
            let mapped = lua.create_table()?;
            mapped.raw_set(1, value)?;
            mapped.raw_set(2, value * 10)?;
            Ok(mapped)
        })?;

        let flattened = flat_map(&lua, list.clone(), callback)?;
        assert_eq!(table_to_integers(&flattened)?, vec![1, 10, 2, 20]);

        let non_table_callback =
            lua.create_function(|_, (_value, _index): (i64, i64)| Ok(Value::Boolean(true)))?;
        let non_table_err = flat_map(&lua, list.clone(), non_table_callback).unwrap_err();
        assert_runtime_error_contains(
            non_table_err,
            "ptool.tbl.flat_map(list, fn) `fn` must return a list table for `list[1]`",
        );

        let sparse_callback = lua.create_function(|lua, (_value, _index): (i64, i64)| {
            let mapped = lua.create_table()?;
            mapped.raw_set(2, "gap")?;
            Ok(mapped)
        })?;
        let sparse_err = flat_map(&lua, list, sparse_callback).unwrap_err();
        assert_runtime_error_contains(
            sparse_err,
            "ptool.tbl.flat_map(list, fn) `fn result for list[1]` must be a dense list with contiguous integer keys starting at 1",
        );
        Ok(())
    }

    #[test]
    fn new_helpers_reuse_dense_list_validation() -> mlua::Result<()> {
        let lua = Lua::new();
        let sparse = lua.create_table()?;
        sparse.raw_set(1, "a")?;
        sparse.raw_set(3, "c")?;

        let any_callback = lua.create_function(|_, (_value, _index): (Value, i64)| Ok(true))?;
        let any_err = any(&lua, sparse.clone(), any_callback).unwrap_err();
        assert_runtime_error_contains(
            any_err,
            "ptool.tbl.any(list, fn) `list` must be a dense list with contiguous integer keys starting at 1",
        );

        let includes_err =
            includes(&lua, sparse, Value::String(lua_string(&lua, "a")?)).unwrap_err();
        assert_runtime_error_contains(
            includes_err,
            "ptool.tbl.includes(list, value) `list` must be a dense list with contiguous integer keys starting at 1",
        );
        Ok(())
    }

    fn int_list(lua: &Lua, values: &[i64]) -> mlua::Result<Table> {
        let list = lua.create_table()?;
        for (offset, value) in values.iter().copied().enumerate() {
            list.raw_set(offset + 1, value)?;
        }
        Ok(list)
    }

    fn table_to_integers(table: &Table) -> mlua::Result<Vec<i64>> {
        let mut values = Vec::new();
        for value in table.sequence_values::<i64>() {
            values.push(value?);
        }
        Ok(values)
    }

    fn lua_string(lua: &Lua, value: &str) -> mlua::Result<mlua::String> {
        lua.create_string(value)
    }

    fn assert_runtime_error_contains(err: mlua::Error, expected: &str) {
        let message = match err {
            mlua::Error::RuntimeError(message) => message,
            other => panic!("expected runtime error, got {other:?}"),
        };
        assert!(
            message.contains(expected),
            "expected error to contain `{expected}`, got `{message}`",
        );
    }
}
