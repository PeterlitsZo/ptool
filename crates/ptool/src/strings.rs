use mlua::{Lua, Table, Value, Variadic};

const SPLIT_SIGNATURE: &str = "ptool.str.split(s, sep[, options])";
const SPLIT_LINES_SIGNATURE: &str = "ptool.str.split_lines(s[, options])";
const JOIN_SIGNATURE: &str = "ptool.str.join(parts, sep)";
const REPLACE_SIGNATURE: &str = "ptool.str.replace(s, from, to[, n])";
const REPEAT_SIGNATURE: &str = "ptool.str.repeat(s, n)";
const INDENT_SIGNATURE: &str = "ptool.str.indent(s, prefix[, options])";

pub(crate) fn trim(input: String) -> String {
    input.trim().to_string()
}

pub(crate) fn trim_start(input: String) -> String {
    input.trim_start().to_string()
}

pub(crate) fn trim_end(input: String) -> String {
    input.trim_end().to_string()
}

pub(crate) fn is_blank(input: String) -> bool {
    input.trim().is_empty()
}

pub(crate) fn starts_with(input: String, prefix: String) -> bool {
    input.starts_with(&prefix)
}

pub(crate) fn ends_with(input: String, suffix: String) -> bool {
    input.ends_with(&suffix)
}

pub(crate) fn contains(input: String, needle: String) -> bool {
    input.contains(&needle)
}

pub(crate) fn split(
    lua: &Lua,
    input: String,
    separator: String,
    options: Option<Table>,
) -> mlua::Result<Table> {
    ensure_non_empty(&separator, SPLIT_SIGNATURE, "sep")?;
    let options = SplitOptions::parse(options)?;

    let parts: Vec<String> = input
        .split(&separator)
        .filter_map(|part| normalize_split_part(part, options.trim, options.skip_empty))
        .collect();
    lua.create_sequence_from(parts)
}

pub(crate) fn split_lines(lua: &Lua, input: String, options: Option<Table>) -> mlua::Result<Table> {
    let options = SplitLinesOptions::parse(options)?;

    let lines: Vec<String> = collect_lines(&input, options.keep_ending)
        .into_iter()
        .filter(|line| !options.skip_empty || !line_without_line_ending(line).is_empty())
        .collect();
    lua.create_sequence_from(lines)
}

pub(crate) fn join(parts: Table, separator: String) -> mlua::Result<String> {
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
    Ok(values.join(&separator))
}

pub(crate) fn replace(args: Variadic<Value>) -> mlua::Result<String> {
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
    ensure_non_empty(&from, REPLACE_SIGNATURE, "from")?;

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

    Ok(match limit {
        Some(limit) => input.replacen(&from, &to, limit),
        None => input.replace(&from, &to),
    })
}

pub(crate) fn repeat(input: String, count: i64) -> mlua::Result<String> {
    if count < 0 {
        return Err(mlua::Error::runtime(format!(
            "{REPEAT_SIGNATURE} `n` must be >= 0"
        )));
    }
    let count = usize::try_from(count)
        .map_err(|_| mlua::Error::runtime(format!("{REPEAT_SIGNATURE} `n` is too large")))?;
    Ok(input.repeat(count))
}

pub(crate) fn cut_prefix(input: String, prefix: String) -> String {
    match input.strip_prefix(&prefix) {
        Some(stripped) => stripped.to_string(),
        None => input,
    }
}

pub(crate) fn cut_suffix(input: String, suffix: String) -> String {
    match input.strip_suffix(&suffix) {
        Some(stripped) => stripped.to_string(),
        None => input,
    }
}

pub(crate) fn indent(
    input: String,
    prefix: String,
    options: Option<Table>,
) -> mlua::Result<String> {
    let options = IndentOptions::parse(options)?;
    if input.is_empty() {
        return Ok(input);
    }

    let mut rendered = String::with_capacity(input.len() + prefix.len());
    for (index, line) in collect_lines(&input, true).into_iter().enumerate() {
        if !(index == 0 && options.skip_first) {
            rendered.push_str(&prefix);
        }
        rendered.push_str(&line);
    }
    Ok(rendered)
}

#[derive(Clone, Copy, Debug, Default)]
struct SplitOptions {
    trim: bool,
    skip_empty: bool,
}

impl SplitOptions {
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
struct SplitLinesOptions {
    keep_ending: bool,
    skip_empty: bool,
}

impl SplitLinesOptions {
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
struct IndentOptions {
    skip_first: bool,
}

impl IndentOptions {
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

fn ensure_non_empty(input: &str, context: &str, arg_name: &str) -> mlua::Result<()> {
    if input.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{context} `{arg_name}` does not accept empty string"
        )));
    }
    Ok(())
}

fn normalize_split_part(part: &str, trim: bool, skip_empty: bool) -> Option<String> {
    let normalized = if trim { part.trim() } else { part };
    if skip_empty && normalized.is_empty() {
        return None;
    }
    Some(normalized.to_string())
}

fn collect_lines(input: &str, keep_ending: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let bytes = input.as_bytes();
    let mut start = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                let end = if keep_ending {
                    index + 1
                } else if index > start && bytes[index - 1] == b'\r' {
                    index - 1
                } else {
                    index
                };
                let slice_start = start.min(end);
                lines.push(input[slice_start..end].to_string());
                start = index + 1;
            }
            b'\r' if bytes.get(index + 1) != Some(&b'\n') => {
                let end = if keep_ending { index + 1 } else { index };
                let slice_start = start.min(end);
                lines.push(input[slice_start..end].to_string());
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }

    if start < input.len() {
        lines.push(input[start..].to_string());
    }

    lines
}

fn line_without_line_ending(line: &str) -> &str {
    if let Some(line) = line.strip_suffix("\r\n") {
        return line;
    }
    if let Some(line) = line.strip_suffix('\n') {
        return line;
    }
    if let Some(line) = line.strip_suffix('\r') {
        return line;
    }
    line
}
