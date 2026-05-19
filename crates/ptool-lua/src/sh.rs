use mlua::{Lua, Table, Value};
use ptool_engine::PtoolEngine;

const SPLIT_SIGNATURE: &str = "ptool.sh.split(command)";
const QUOTE_SIGNATURE: &str = "ptool.sh.quote(word)";
const JOIN_SIGNATURE: &str = "ptool.sh.join(words)";

pub(crate) fn split(lua: &Lua, engine: &PtoolEngine, input: String) -> mlua::Result<Table> {
    let parts = engine
        .shell_split(&input)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, SPLIT_SIGNATURE))?;
    lua.create_sequence_from(parts)
}

pub(crate) fn quote(engine: &PtoolEngine, input: String) -> mlua::Result<String> {
    engine
        .shell_quote(&input)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, QUOTE_SIGNATURE))
}

pub(crate) fn join(engine: &PtoolEngine, words: Table) -> mlua::Result<String> {
    let len = words.raw_len();
    let mut values = Vec::with_capacity(len);
    for index in 1..=len {
        match words.raw_get::<Value>(index)? {
            Value::String(value) => values.push(value.to_str()?.to_string()),
            _ => {
                return Err(crate::lua_error::invalid_argument(
                    JOIN_SIGNATURE,
                    format!("`words[{index}]` must be a string"),
                ));
            }
        }
    }

    engine
        .shell_join(&values)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, JOIN_SIGNATURE))
}
