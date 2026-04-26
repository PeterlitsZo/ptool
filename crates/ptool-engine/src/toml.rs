use crate::{Error, ErrorKind, Result};
use std::collections::BTreeMap;
use std::str::FromStr;
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value as EditValue};

const GET_OP: &str = "ptool.toml.get";
const PARSE_OP: &str = "ptool.toml.parse";
const REMOVE_OP: &str = "ptool.toml.remove";
const SET_OP: &str = "ptool.toml.set";
const STRINGIFY_OP: &str = "ptool.toml.stringify";

#[derive(Clone, Debug, PartialEq)]
pub enum TomlValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Datetime(String),
    Array(Vec<TomlValue>),
    Table(BTreeMap<String, TomlValue>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TomlPathSegment {
    Key(String),
    Index(usize),
}

pub(crate) fn parse(input: &str) -> Result<TomlValue> {
    let parsed = parse_root_value(input, PARSE_OP)?;
    Ok(from_plain_value(&parsed))
}

pub(crate) fn get(input: &str, path: &[TomlPathSegment]) -> Result<Option<TomlValue>> {
    ensure_non_empty_path(path, GET_OP)?;

    let parsed = parse_root_value(input, GET_OP)?;
    let Some(value) = get_plain_value_by_path(&parsed, path) else {
        return Ok(None);
    };
    Ok(Some(from_plain_value(value)))
}

pub(crate) fn set(input: &str, path: &[TomlPathSegment], value: &TomlValue) -> Result<String> {
    ensure_non_empty_path(path, SET_OP)?;

    let mut doc = parse_document(input, SET_OP)?;
    let (parents, last) = path.split_at(path.len() - 1);
    let mut container = ContainerMut::Table(doc.as_table_mut());

    for (index, segment) in parents.iter().enumerate() {
        let next = &path[index + 1];
        container = descend_container(container, segment, next, SET_OP)?;
    }

    apply_set(container, &last[0], value)?;
    Ok(doc.to_string())
}

pub(crate) fn remove(input: &str, path: &[TomlPathSegment]) -> Result<String> {
    ensure_non_empty_path(path, REMOVE_OP)?;

    let mut doc = parse_document(input, REMOVE_OP)?;
    let (parents, last) = path.split_at(path.len() - 1);
    let mut container = ContainerMut::Table(doc.as_table_mut());

    for (index, segment) in parents.iter().enumerate() {
        let next = &path[index + 1];
        container = match descend_container(container, segment, next, REMOVE_OP) {
            Ok(next_container) => next_container,
            Err(err) if is_missing_path_error(&err) => return Ok(doc.to_string()),
            Err(err) => return Err(err),
        };
    }

    apply_remove(container, &last[0])?;
    Ok(doc.to_string())
}

pub(crate) fn stringify(value: &TomlValue) -> Result<String> {
    let plain_value = to_plain_value(value, STRINGIFY_OP)?;
    let toml::Value::Table(_) = plain_value else {
        return Err(invalid_toml(STRINGIFY_OP, "root value must be a table"));
    };

    toml::to_string_pretty(&plain_value).map_err(|err| invalid_toml(STRINGIFY_OP, err.to_string()))
}

enum ContainerMut<'a> {
    Table(&'a mut Table),
    InlineTable(&'a mut InlineTable),
    Array(&'a mut Array),
    ArrayOfTables(&'a mut ArrayOfTables),
}

fn ensure_non_empty_path(path: &[TomlPathSegment], op: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "path must not be empty").with_op(op));
    }
    Ok(())
}

fn parse_root_value(input: &str, op: &str) -> Result<toml::Value> {
    let parsed: toml::Table = toml::from_str(input)
        .map_err(|err| invalid_toml(op, err.to_string()).with_input(input.to_string()))?;
    Ok(toml::Value::Table(parsed))
}

fn parse_document(input: &str, op: &str) -> Result<DocumentMut> {
    input
        .parse::<DocumentMut>()
        .map_err(|err| invalid_toml(op, err.to_string()).with_input(input.to_string()))
}

fn get_plain_value_by_path<'a>(
    root: &'a toml::Value,
    path: &[TomlPathSegment],
) -> Option<&'a toml::Value> {
    let mut current = root;
    for segment in path {
        current = match segment {
            TomlPathSegment::Key(key) => current.get(key.as_str())?,
            TomlPathSegment::Index(index) => current.get(*index)?,
        };
    }
    Some(current)
}

fn from_plain_value(value: &toml::Value) -> TomlValue {
    match value {
        toml::Value::String(value) => TomlValue::String(value.clone()),
        toml::Value::Integer(value) => TomlValue::Integer(*value),
        toml::Value::Float(value) => TomlValue::Float(*value),
        toml::Value::Boolean(value) => TomlValue::Boolean(*value),
        toml::Value::Datetime(value) => TomlValue::Datetime(value.to_string()),
        toml::Value::Array(values) => {
            TomlValue::Array(values.iter().map(from_plain_value).collect())
        }
        toml::Value::Table(values) => TomlValue::Table(
            values
                .iter()
                .map(|(key, value)| (key.clone(), from_plain_value(value)))
                .collect(),
        ),
    }
}

fn to_plain_value(value: &TomlValue, op: &str) -> Result<toml::Value> {
    match value {
        TomlValue::String(value) => Ok(toml::Value::String(value.clone())),
        TomlValue::Integer(value) => Ok(toml::Value::Integer(*value)),
        TomlValue::Float(value) => {
            if !value.is_finite() {
                return Err(invalid_toml(op, "TOML floats must be finite"));
            }
            Ok(toml::Value::Float(*value))
        }
        TomlValue::Boolean(value) => Ok(toml::Value::Boolean(*value)),
        TomlValue::Datetime(value) => {
            let parsed = toml::value::Datetime::from_str(value).map_err(|err| {
                invalid_toml(op, format!("invalid TOML datetime `{value}`: {err}"))
            })?;
            Ok(toml::Value::Datetime(parsed))
        }
        TomlValue::Array(values) => {
            let mut array = Vec::with_capacity(values.len());
            for value in values {
                array.push(to_plain_value(value, op)?);
            }
            Ok(toml::Value::Array(array))
        }
        TomlValue::Table(values) => {
            let mut table = toml::Table::new();
            for (key, value) in values {
                table.insert(key.clone(), to_plain_value(value, op)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}

fn descend_container<'a>(
    container: ContainerMut<'a>,
    segment: &TomlPathSegment,
    next: &TomlPathSegment,
    op: &str,
) -> Result<ContainerMut<'a>> {
    match (container, segment) {
        (ContainerMut::Table(table), TomlPathSegment::Key(key)) => {
            descend_table(table, key, Some(next), op)
        }
        (ContainerMut::InlineTable(table), TomlPathSegment::Key(key)) => {
            descend_inline_table(table, key, Some(next), op)
        }
        (ContainerMut::Array(array), TomlPathSegment::Index(index)) => {
            descend_array(array, *index, op)
        }
        (ContainerMut::ArrayOfTables(array), TomlPathSegment::Index(index)) => {
            descend_array_of_tables(array, *index, op)
        }
        (ContainerMut::Table(_), TomlPathSegment::Index(index))
        | (ContainerMut::InlineTable(_), TomlPathSegment::Index(index)) => Err(invalid_toml(
            op,
            format!("path index `{index}` requires an array value"),
        )),
        (ContainerMut::Array(_), TomlPathSegment::Key(key))
        | (ContainerMut::ArrayOfTables(_), TomlPathSegment::Key(key)) => Err(invalid_toml(
            op,
            format!("path key `{key}` requires a table value"),
        )),
    }
}

fn descend_table<'a>(
    table: &'a mut Table,
    key: &str,
    next: Option<&TomlPathSegment>,
    op: &str,
) -> Result<ContainerMut<'a>> {
    if !table.contains_key(key) {
        match next {
            Some(TomlPathSegment::Key(_)) => {
                table.insert(key, toml_edit::table());
            }
            Some(TomlPathSegment::Index(_)) => {
                return Err(missing_path(op, format!("path key `{key}` does not exist")));
            }
            None => {}
        }
    }

    let Some(item) = table.get_mut(key) else {
        return Err(missing_path(op, format!("path key `{key}` does not exist")));
    };
    item_to_container(item, op, key)
}

fn descend_inline_table<'a>(
    table: &'a mut InlineTable,
    key: &str,
    next: Option<&TomlPathSegment>,
    op: &str,
) -> Result<ContainerMut<'a>> {
    if !table.contains_key(key) {
        match next {
            Some(TomlPathSegment::Key(_)) => {
                table.insert(
                    key.to_string(),
                    EditValue::InlineTable(InlineTable::default()),
                );
            }
            Some(TomlPathSegment::Index(_)) => {
                return Err(missing_path(op, format!("path key `{key}` does not exist")));
            }
            None => {}
        }
    }

    let Some(value) = table.get_mut(key) else {
        return Err(missing_path(op, format!("path key `{key}` does not exist")));
    };
    value_to_container(value, op, key)
}

fn descend_array<'a>(array: &'a mut Array, index: usize, op: &str) -> Result<ContainerMut<'a>> {
    let Some(value) = array.get_mut(index) else {
        return Err(missing_path(
            op,
            format!("path index `{index}` is out of bounds"),
        ));
    };
    value_to_container(value, op, &index.to_string())
}

fn descend_array_of_tables<'a>(
    array: &'a mut ArrayOfTables,
    index: usize,
    op: &str,
) -> Result<ContainerMut<'a>> {
    let Some(table) = array.get_mut(index) else {
        return Err(missing_path(
            op,
            format!("path index `{index}` is out of bounds"),
        ));
    };
    Ok(ContainerMut::Table(table))
}

fn item_to_container<'a>(item: &'a mut Item, op: &str, segment: &str) -> Result<ContainerMut<'a>> {
    match item {
        Item::Table(table) => Ok(ContainerMut::Table(table)),
        Item::ArrayOfTables(array) => Ok(ContainerMut::ArrayOfTables(array)),
        Item::Value(value) => value_to_container(value, op, segment),
        Item::None => Err(invalid_toml(
            op,
            format!("path segment `{segment}` is not a table or array"),
        )),
    }
}

fn value_to_container<'a>(
    value: &'a mut EditValue,
    op: &str,
    segment: &str,
) -> Result<ContainerMut<'a>> {
    match value {
        EditValue::Array(array) => Ok(ContainerMut::Array(array)),
        EditValue::InlineTable(table) => Ok(ContainerMut::InlineTable(table)),
        _ => Err(invalid_toml(
            op,
            format!("path segment `{segment}` is not a table or array"),
        )),
    }
}

fn apply_set(
    container: ContainerMut<'_>,
    segment: &TomlPathSegment,
    value: &TomlValue,
) -> Result<()> {
    match (container, segment) {
        (ContainerMut::Table(table), TomlPathSegment::Key(key)) => {
            table.insert(key, to_edit_item_for_key(value, SET_OP)?);
            Ok(())
        }
        (ContainerMut::InlineTable(table), TomlPathSegment::Key(key)) => {
            table.insert(key.clone(), to_edit_value_nested(value, SET_OP)?);
            Ok(())
        }
        (ContainerMut::Array(array), TomlPathSegment::Index(index)) => {
            if *index >= array.len() {
                return Err(invalid_toml(
                    SET_OP,
                    format!("path index `{index}` is out of bounds"),
                ));
            }
            array.replace_formatted(*index, to_edit_value_nested(value, SET_OP)?);
            Ok(())
        }
        (ContainerMut::ArrayOfTables(array), TomlPathSegment::Index(index)) => {
            let Some(slot) = array.get_mut(*index) else {
                return Err(invalid_toml(
                    SET_OP,
                    format!("path index `{index}` is out of bounds"),
                ));
            };
            let replacement = to_edit_table(value, SET_OP)?;
            *slot = replacement;
            Ok(())
        }
        (ContainerMut::Table(_), TomlPathSegment::Index(index))
        | (ContainerMut::InlineTable(_), TomlPathSegment::Index(index)) => Err(invalid_toml(
            SET_OP,
            format!("path index `{index}` requires an array value"),
        )),
        (ContainerMut::Array(_), TomlPathSegment::Key(key))
        | (ContainerMut::ArrayOfTables(_), TomlPathSegment::Key(key)) => Err(invalid_toml(
            SET_OP,
            format!("path key `{key}` requires a table value"),
        )),
    }
}

fn apply_remove(container: ContainerMut<'_>, segment: &TomlPathSegment) -> Result<()> {
    match (container, segment) {
        (ContainerMut::Table(table), TomlPathSegment::Key(key)) => {
            table.remove(key);
            Ok(())
        }
        (ContainerMut::InlineTable(table), TomlPathSegment::Key(key)) => {
            table.remove(key);
            Ok(())
        }
        (ContainerMut::Array(array), TomlPathSegment::Index(index)) => {
            if *index < array.len() {
                array.remove(*index);
            }
            Ok(())
        }
        (ContainerMut::ArrayOfTables(array), TomlPathSegment::Index(index)) => {
            if *index < array.len() {
                array.remove(*index);
            }
            Ok(())
        }
        (ContainerMut::Table(_), TomlPathSegment::Index(index))
        | (ContainerMut::InlineTable(_), TomlPathSegment::Index(index)) => Err(invalid_toml(
            REMOVE_OP,
            format!("path index `{index}` requires an array value"),
        )),
        (ContainerMut::Array(_), TomlPathSegment::Key(key))
        | (ContainerMut::ArrayOfTables(_), TomlPathSegment::Key(key)) => Err(invalid_toml(
            REMOVE_OP,
            format!("path key `{key}` requires a table value"),
        )),
    }
}

fn to_edit_item_for_key(value: &TomlValue, op: &str) -> Result<Item> {
    match value {
        TomlValue::String(value) => Ok(toml_edit::value(value.clone())),
        TomlValue::Integer(value) => Ok(toml_edit::value(*value)),
        TomlValue::Float(value) => {
            if !value.is_finite() {
                return Err(invalid_toml(op, "TOML floats must be finite"));
            }
            Ok(toml_edit::value(*value))
        }
        TomlValue::Boolean(value) => Ok(toml_edit::value(*value)),
        TomlValue::Datetime(value) => Ok(toml_edit::value(parse_edit_datetime(value, op)?)),
        TomlValue::Array(values) => {
            if is_array_of_tables(values) {
                Ok(Item::ArrayOfTables(to_edit_array_of_tables(values, op)?))
            } else {
                Ok(toml_edit::value(to_edit_array(values, op)?))
            }
        }
        TomlValue::Table(_) => Ok(Item::Table(to_edit_table(value, op)?)),
    }
}

fn to_edit_value_nested(value: &TomlValue, op: &str) -> Result<EditValue> {
    match value {
        TomlValue::String(value) => Ok(EditValue::from(value.clone())),
        TomlValue::Integer(value) => Ok(EditValue::from(*value)),
        TomlValue::Float(value) => {
            if !value.is_finite() {
                return Err(invalid_toml(op, "TOML floats must be finite"));
            }
            Ok(EditValue::from(*value))
        }
        TomlValue::Boolean(value) => Ok(EditValue::from(*value)),
        TomlValue::Datetime(value) => Ok(EditValue::from(parse_edit_datetime(value, op)?)),
        TomlValue::Array(values) => Ok(EditValue::from(to_edit_array(values, op)?)),
        TomlValue::Table(values) => {
            let mut table = InlineTable::default();
            for (key, value) in values {
                table.insert(key.clone(), to_edit_value_nested(value, op)?);
            }
            table.fmt();
            Ok(EditValue::from(table))
        }
    }
}

fn to_edit_array(values: &[TomlValue], op: &str) -> Result<Array> {
    let mut array = Array::new();
    for value in values {
        array.push_formatted(to_edit_value_nested(value, op)?);
    }
    array.fmt();
    Ok(array)
}

fn to_edit_array_of_tables(values: &[TomlValue], op: &str) -> Result<ArrayOfTables> {
    let mut array = ArrayOfTables::new();
    for value in values {
        array.push(to_edit_table(value, op)?);
    }
    Ok(array)
}

fn to_edit_table(value: &TomlValue, op: &str) -> Result<Table> {
    let TomlValue::Table(values) = value else {
        return Err(invalid_toml(op, "expected a TOML table value"));
    };

    let mut table = Table::new();
    for (key, value) in values {
        table.insert(key, to_edit_item_for_key(value, op)?);
    }
    table.fmt();
    Ok(table)
}

fn parse_edit_datetime(value: &str, op: &str) -> Result<toml_edit::Datetime> {
    toml_edit::Datetime::from_str(value)
        .map_err(|err| invalid_toml(op, format!("invalid TOML datetime `{value}`: {err}")))
}

fn is_array_of_tables(values: &[TomlValue]) -> bool {
    !values.is_empty()
        && values
            .iter()
            .all(|value| matches!(value, TomlValue::Table(_)))
}

fn invalid_toml(op: &str, detail: impl Into<String>) -> Error {
    let detail = detail.into();
    Error::new(ErrorKind::InvalidToml, format!("{op} failed: {detail}"))
        .with_op(op)
        .with_detail(detail)
}

fn missing_path(op: &str, detail: impl Into<String>) -> Error {
    let detail = detail.into();
    Error::new(ErrorKind::InvalidToml, format!("{op} failed: {detail}"))
        .with_op(op)
        .with_detail(format!("missing_path: {detail}"))
}

fn is_missing_path_error(err: &Error) -> bool {
    err.kind == ErrorKind::InvalidToml
        && err
            .detail()
            .map(|detail| detail.starts_with("missing_path: "))
            .unwrap_or(false)
}
