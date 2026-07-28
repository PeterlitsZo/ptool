use crate::{Error, ErrorKind, Result};

const GET_OP: &str = "ptool.json.get";
const PARSE_OP: &str = "ptool.json.parse";
const SET_OP: &str = "ptool.json.set";
const STRINGIFY_OP: &str = "ptool.json.stringify";

pub type JsonValue = serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonPathSegment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JsonStringifyOptions {
    pub pretty: bool,
}

pub(crate) fn parse(input: &str) -> Result<JsonValue> {
    parse_with_op(input, PARSE_OP)
}

pub(crate) fn get(input: &str, path: &[JsonPathSegment]) -> Result<Option<JsonValue>> {
    ensure_non_empty_path(path, GET_OP)?;
    let parsed = parse_with_op(input, GET_OP)?;
    Ok(get_value_by_path(&parsed, path).cloned())
}

pub(crate) fn set(input: &str, path: &[JsonPathSegment], value: &JsonValue) -> Result<String> {
    ensure_non_empty_path(path, SET_OP)?;
    let mut parsed = parse_with_op(input, SET_OP)?;
    set_value_by_path(&mut parsed, path, value.clone(), SET_OP)?;
    stringify_with_op(&parsed, JsonStringifyOptions::default(), SET_OP)
}

pub(crate) fn stringify(value: &JsonValue, options: JsonStringifyOptions) -> Result<String> {
    stringify_with_op(value, options, STRINGIFY_OP)
}

fn parse_with_op(input: &str, op: &str) -> Result<JsonValue> {
    serde_json::from_str(input).map_err(|err| {
        Error::new(ErrorKind::InvalidArgs, format!("{op} failed: {err}"))
            .with_op(op)
            .with_input(input.to_string())
    })
}

fn stringify_with_op(value: &JsonValue, options: JsonStringifyOptions, op: &str) -> Result<String> {
    let result = if options.pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    };

    result.map_err(|err| {
        Error::new(ErrorKind::InvalidArgs, format!("{op} failed: {err}")).with_op(op)
    })
}

pub(crate) fn ensure_non_empty_path(path: &[JsonPathSegment], op: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "path must not be empty").with_op(op));
    }
    Ok(())
}

pub(crate) fn get_value_by_path<'a>(
    root: &'a JsonValue,
    path: &[JsonPathSegment],
) -> Option<&'a JsonValue> {
    let mut current = root;
    for segment in path {
        current = match segment {
            JsonPathSegment::Key(key) => match current {
                JsonValue::Object(values) => values.get(key)?,
                _ => return None,
            },
            JsonPathSegment::Index(index) => match current {
                JsonValue::Array(values) => values.get(*index)?,
                _ => return None,
            },
        };
    }
    Some(current)
}

pub(crate) fn set_value_by_path(
    root: &mut JsonValue,
    path: &[JsonPathSegment],
    value: JsonValue,
    op: &str,
) -> Result<()> {
    ensure_non_empty_path(path, op)?;

    let (parents, last) = path.split_at(path.len() - 1);
    let mut current = root;
    for (index, segment) in parents.iter().enumerate() {
        current = descend_value(current, segment, &path[index + 1], op)?;
    }

    apply_set(current, &last[0], value, op)
}

fn descend_value<'a>(
    current: &'a mut JsonValue,
    segment: &JsonPathSegment,
    next: &JsonPathSegment,
    op: &str,
) -> Result<&'a mut JsonValue> {
    match segment {
        JsonPathSegment::Key(key) => {
            let values = current.as_object_mut().ok_or_else(|| {
                invalid_path(op, format!("path key `{key}` requires an object value"))
            })?;

            if !values.contains_key(key) {
                match next {
                    JsonPathSegment::Key(_) => {
                        values.insert(key.clone(), JsonValue::Object(serde_json::Map::new()));
                    }
                    JsonPathSegment::Index(_) => {
                        return Err(invalid_path(op, format!("path key `{key}` does not exist")));
                    }
                }
            }

            values
                .get_mut(key)
                .ok_or_else(|| invalid_path(op, format!("path key `{key}` does not exist")))
        }
        JsonPathSegment::Index(index) => {
            let values = current.as_array_mut().ok_or_else(|| {
                invalid_path(op, format!("path index `{index}` requires an array value"))
            })?;
            values
                .get_mut(*index)
                .ok_or_else(|| invalid_path(op, format!("path index `{index}` is out of bounds")))
        }
    }
}

fn apply_set(
    current: &mut JsonValue,
    segment: &JsonPathSegment,
    value: JsonValue,
    op: &str,
) -> Result<()> {
    match segment {
        JsonPathSegment::Key(key) => {
            let values = current.as_object_mut().ok_or_else(|| {
                invalid_path(op, format!("path key `{key}` requires an object value"))
            })?;
            values.insert(key.clone(), value);
            Ok(())
        }
        JsonPathSegment::Index(index) => {
            let values = current.as_array_mut().ok_or_else(|| {
                invalid_path(op, format!("path index `{index}` requires an array value"))
            })?;
            let slot = values.get_mut(*index).ok_or_else(|| {
                invalid_path(op, format!("path index `{index}` is out of bounds"))
            })?;
            *slot = value;
            Ok(())
        }
    }
}

fn invalid_path(op: &str, detail: impl Into<String>) -> Error {
    let detail = detail.into();
    Error::new(ErrorKind::InvalidArgs, format!("{op} failed: {detail}"))
        .with_op(op)
        .with_detail(detail)
}
