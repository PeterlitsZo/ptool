use crate::{Error, ErrorKind, JsonValue, Result};
use serde_norway::Value as RawYamlValue;

const GET_OP: &str = "ptool.yaml.get";
const PARSE_OP: &str = "ptool.yaml.parse";
const STRINGIFY_OP: &str = "ptool.yaml.stringify";

pub type YamlValue = JsonValue;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum YamlPathSegment {
    Key(String),
    Index(usize),
}

pub(crate) fn parse(input: &str) -> Result<YamlValue> {
    let parsed: RawYamlValue = serde_norway::from_str(input)
        .map_err(|err| invalid_yaml(PARSE_OP, err.to_string()).with_input(input.to_string()))?;
    raw_yaml_to_value(&parsed, PARSE_OP)
}

pub(crate) fn get(input: &str, path: &[YamlPathSegment]) -> Result<Option<YamlValue>> {
    ensure_non_empty_path(path, GET_OP)?;

    let parsed = parse_with_op(input, GET_OP)?;
    let Some(value) = get_value_by_path(&parsed, path) else {
        return Ok(None);
    };
    Ok(Some(value.clone()))
}

pub(crate) fn stringify(value: &YamlValue) -> Result<String> {
    serde_norway::to_string(value)
        .map_err(|err| invalid_yaml(STRINGIFY_OP, format!("yaml stringify failed: {err}")))
}

fn parse_with_op(input: &str, op: &str) -> Result<YamlValue> {
    let parsed: RawYamlValue = serde_norway::from_str(input)
        .map_err(|err| invalid_yaml(op, err.to_string()).with_input(input.to_string()))?;
    raw_yaml_to_value(&parsed, op)
}

fn raw_yaml_to_value(value: &RawYamlValue, op: &str) -> Result<YamlValue> {
    match value {
        RawYamlValue::Null => Ok(YamlValue::Null),
        RawYamlValue::Bool(value) => Ok(YamlValue::Bool(*value)),
        RawYamlValue::Number(value) => {
            if let Some(number) = value.as_i64() {
                return Ok(YamlValue::Number(number.into()));
            }
            if let Some(number) = value.as_u64() {
                return Ok(YamlValue::Number(number.into()));
            }
            if let Some(number) = value.as_f64() {
                let number = serde_json::Number::from_f64(number)
                    .ok_or_else(|| invalid_yaml(op, "yaml parse failed: floats must be finite"))?;
                return Ok(YamlValue::Number(number));
            }
            Err(invalid_yaml(op, "yaml parse failed: unsupported number"))
        }
        RawYamlValue::String(value) => Ok(YamlValue::String(value.clone())),
        RawYamlValue::Sequence(values) => {
            let mut array = Vec::with_capacity(values.len());
            for value in values {
                array.push(raw_yaml_to_value(value, op)?);
            }
            Ok(YamlValue::Array(array))
        }
        RawYamlValue::Mapping(values) => {
            let mut map = serde_json::Map::with_capacity(values.len());
            for (key, value) in values {
                let key = match key {
                    RawYamlValue::String(key) => key.clone(),
                    _ => {
                        return Err(invalid_yaml(
                            op,
                            "yaml parse failed: mapping keys must be strings",
                        ));
                    }
                };
                map.insert(key, raw_yaml_to_value(value, op)?);
            }
            Ok(YamlValue::Object(map))
        }
        RawYamlValue::Tagged(_) => Err(invalid_yaml(op, "yaml tags are not supported")),
    }
}

fn get_value_by_path<'a>(root: &'a YamlValue, path: &[YamlPathSegment]) -> Option<&'a YamlValue> {
    let mut current = root;
    for segment in path {
        current = match segment {
            YamlPathSegment::Key(key) => match current {
                YamlValue::Object(values) => values.get(key)?,
                _ => return None,
            },
            YamlPathSegment::Index(index) => match current {
                YamlValue::Array(values) => values.get(*index)?,
                _ => return None,
            },
        };
    }
    Some(current)
}

fn ensure_non_empty_path(path: &[YamlPathSegment], op: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "path must not be empty").with_op(op));
    }
    Ok(())
}

fn invalid_yaml(op: &str, msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidArgs, msg).with_op(op)
}
