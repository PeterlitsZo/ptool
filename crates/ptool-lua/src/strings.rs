use mlua::{Lua, Table, Value, Variadic};
use ptool_engine::PtoolEngine;

const SPLIT_SIGNATURE: &str = "ptool.str.split(s, sep[, options])";
const SPLIT_LINES_SIGNATURE: &str = "ptool.str.split_lines(s[, options])";
const JOIN_SIGNATURE: &str = "ptool.str.join(parts, sep)";
const REPLACE_SIGNATURE: &str = "ptool.str.replace(s, from, to[, n])";
const REPEAT_SIGNATURE: &str = "ptool.str.repeat(s, n)";
const INDENT_SIGNATURE: &str = "ptool.str.indent(s, prefix[, options])";

pub(crate) fn trim(engine: &PtoolEngine, input: String) -> String {
    engine.str_trim(&input)
}

pub(crate) fn trim_start(engine: &PtoolEngine, input: String) -> String {
    engine.str_trim_start(&input)
}

pub(crate) fn trim_end(engine: &PtoolEngine, input: String) -> String {
    engine.str_trim_end(&input)
}

pub(crate) fn is_blank(engine: &PtoolEngine, input: String) -> bool {
    engine.str_is_blank(&input)
}

pub(crate) fn starts_with(engine: &PtoolEngine, input: String, prefix: String) -> bool {
    engine.str_starts_with(&input, &prefix)
}

pub(crate) fn ends_with(engine: &PtoolEngine, input: String, suffix: String) -> bool {
    engine.str_ends_with(&input, &suffix)
}

pub(crate) fn contains(engine: &PtoolEngine, input: String, needle: String) -> bool {
    engine.str_contains(&input, &needle)
}

pub(crate) fn split(
    lua: &Lua,
    engine: &PtoolEngine,
    input: String,
    separator: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let options = LuaSplitOptions::parse(options)?;
    let parts = engine
        .str_split(
            &input,
            &separator,
            ptool_engine::SplitOptions {
                trim: options.trim,
                skip_empty: options.skip_empty,
            },
        )
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, SPLIT_SIGNATURE))?;
    lua.create_sequence_from(parts)
}

pub(crate) fn split_lines(
    lua: &Lua,
    engine: &PtoolEngine,
    input: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let options = LuaSplitLinesOptions::parse(options)?;
    let lines = engine.str_split_lines(
        &input,
        ptool_engine::SplitLinesOptions {
            keep_ending: options.keep_ending,
            skip_empty: options.skip_empty,
        },
    );
    lua.create_sequence_from(lines)
}

pub(crate) fn join(engine: &PtoolEngine, parts: Table, separator: String) -> mlua::Result<String> {
    let len = parts.raw_len();
    let mut values = Vec::with_capacity(len);
    for index in 1..=len {
        match parts.raw_get::<Value>(index)? {
            Value::String(value) => values.push(value.to_str()?.to_string()),
            _ => {
                return Err(mlua::Error::runtime(format!(
                    "{JOIN_SIGNATURE} `parts[{index}]` must be a string"
                )));
            }
        }
    }
    Ok(engine.str_join(&values, &separator))
}

pub(crate) fn replace(engine: &PtoolEngine, args: Variadic<Value>) -> mlua::Result<String> {
    if args.len() < 3 {
        return Err(mlua::Error::runtime(format!(
            "{REPLACE_SIGNATURE} requires `s`, `from`, and `to`"
        )));
    }
    if args.len() > 4 {
        return Err(mlua::Error::runtime(format!(
            "{REPLACE_SIGNATURE} accepts at most 4 arguments"
        )));
    }

    let input = parse_string_arg(&args[0], REPLACE_SIGNATURE, "s")?;
    let from = parse_string_arg(&args[1], REPLACE_SIGNATURE, "from")?;
    let to = parse_string_arg(&args[2], REPLACE_SIGNATURE, "to")?;

    let limit = match args.get(3) {
        None | Some(Value::Nil) => None,
        Some(Value::Integer(value)) if *value >= 0 => Some(*value as usize),
        Some(Value::Integer(_)) => {
            return Err(mlua::Error::runtime(format!(
                "{REPLACE_SIGNATURE} `n` must be >= 0"
            )));
        }
        Some(_) => {
            return Err(mlua::Error::runtime(format!(
                "{REPLACE_SIGNATURE} `n` must be an integer"
            )));
        }
    };

    engine
        .str_replace(&input, &from, &to, limit)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REPLACE_SIGNATURE))
}

pub(crate) fn repeat(engine: &PtoolEngine, input: String, count: i64) -> mlua::Result<String> {
    engine
        .str_repeat(&input, count)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, REPEAT_SIGNATURE))
}

pub(crate) fn cut_prefix(engine: &PtoolEngine, input: String, prefix: String) -> String {
    engine.str_cut_prefix(&input, &prefix)
}

pub(crate) fn cut_suffix(engine: &PtoolEngine, input: String, suffix: String) -> String {
    engine.str_cut_suffix(&input, &suffix)
}

pub(crate) fn indent(
    engine: &PtoolEngine,
    input: String,
    prefix: String,
    options: Option<Table>,
) -> mlua::Result<String> {
    let options = LuaIndentOptions::parse(options)?;
    Ok(engine.str_indent(
        &input,
        &prefix,
        ptool_engine::IndentOptions {
            skip_first: options.skip_first,
        },
    ))
}

#[derive(Clone, Copy, Debug, Default)]
struct LuaSplitOptions {
    trim: bool,
    skip_empty: bool,
}

impl LuaSplitOptions {
    fn parse(options: Option<Table>) -> mlua::Result<Self> {
        let mut parsed = Self::default();
        let Some(options) = options else {
            return Ok(parsed);
        };

        for pair in options.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let key = table_key_to_string(key, SPLIT_SIGNATURE)?;
            match key.as_str() {
                "trim" => parsed.trim = parse_bool_option(value, SPLIT_SIGNATURE, "trim")?,
                "skip_empty" => {
                    parsed.skip_empty = parse_bool_option(value, SPLIT_SIGNATURE, "skip_empty")?
                }
                _ => {
                    return Err(mlua::Error::runtime(format!(
                        "{SPLIT_SIGNATURE} unknown option `{key}`"
                    )));
                }
            }
        }

        Ok(parsed)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct LuaSplitLinesOptions {
    keep_ending: bool,
    skip_empty: bool,
}

impl LuaSplitLinesOptions {
    fn parse(options: Option<Table>) -> mlua::Result<Self> {
        let mut parsed = Self::default();
        let Some(options) = options else {
            return Ok(parsed);
        };

        for pair in options.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let key = table_key_to_string(key, SPLIT_LINES_SIGNATURE)?;
            match key.as_str() {
                "keep_ending" => {
                    parsed.keep_ending =
                        parse_bool_option(value, SPLIT_LINES_SIGNATURE, "keep_ending")?
                }
                "skip_empty" => {
                    parsed.skip_empty =
                        parse_bool_option(value, SPLIT_LINES_SIGNATURE, "skip_empty")?
                }
                _ => {
                    return Err(mlua::Error::runtime(format!(
                        "{SPLIT_LINES_SIGNATURE} unknown option `{key}`"
                    )));
                }
            }
        }

        Ok(parsed)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct LuaIndentOptions {
    skip_first: bool,
}

impl LuaIndentOptions {
    fn parse(options: Option<Table>) -> mlua::Result<Self> {
        let mut parsed = Self::default();
        let Some(options) = options else {
            return Ok(parsed);
        };

        for pair in options.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let key = table_key_to_string(key, INDENT_SIGNATURE)?;
            match key.as_str() {
                "skip_first" => {
                    parsed.skip_first = parse_bool_option(value, INDENT_SIGNATURE, "skip_first")?
                }
                _ => {
                    return Err(mlua::Error::runtime(format!(
                        "{INDENT_SIGNATURE} unknown option `{key}`"
                    )));
                }
            }
        }

        Ok(parsed)
    }
}

fn parse_string_arg(value: &Value, context: &str, arg_name: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(mlua::Error::runtime(format!(
            "{context} `{arg_name}` must be a string"
        ))),
    }
}

fn table_key_to_string(value: Value, context: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(mlua::Error::runtime(format!(
            "{context} option keys must be strings"
        ))),
    }
}

fn parse_bool_option(value: Value, context: &str, key: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(value) => Ok(value),
        _ => Err(mlua::Error::runtime(format!(
            "{context} `{key}` must be a boolean"
        ))),
    }
}
