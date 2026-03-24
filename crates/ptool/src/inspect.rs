use mlua::{String as LuaString, Table, Value};
use std::collections::HashSet;

const DEFAULT_INDENT: &str = "  ";
const DEFAULT_INLINE_WIDTH: usize = 80;
const CYCLE_MARKER: &str = "<cycle>";
const MAX_DEPTH_MARKER: &str = "<max-depth>";

pub(crate) fn render(value: Value, options: Option<Table>) -> mlua::Result<String> {
    let options = InspectOptions::parse(options)?;
    let mut renderer = Renderer::new(options);
    renderer.render_value(&value, 0)
}

#[derive(Debug, Clone)]
struct InspectOptions {
    indent: String,
    multiline: bool,
    max_depth: Option<usize>,
}

impl Default for InspectOptions {
    fn default() -> Self {
        Self {
            indent: DEFAULT_INDENT.to_string(),
            multiline: true,
            max_depth: None,
        }
    }
}

impl InspectOptions {
    fn parse(options: Option<Table>) -> mlua::Result<Self> {
        let Some(options) = options else {
            return Ok(Self::default());
        };

        let mut parsed = Self::default();

        for pair in options.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let key = match key {
                Value::String(key) => key.to_str()?.to_string(),
                _ => {
                    return Err(mlua::Error::runtime(
                        "ptool.inspect(value, options) option keys must be strings",
                    ));
                }
            };

            match key.as_str() {
                "indent" => match value {
                    Value::String(value) => parsed.indent = value.to_str()?.to_string(),
                    _ => {
                        return Err(mlua::Error::runtime(
                            "ptool.inspect(value, options) `indent` must be a string",
                        ));
                    }
                },
                "multiline" => match value {
                    Value::Boolean(value) => parsed.multiline = value,
                    _ => {
                        return Err(mlua::Error::runtime(
                            "ptool.inspect(value, options) `multiline` must be a boolean",
                        ));
                    }
                },
                "max_depth" => match value {
                    Value::Nil => parsed.max_depth = None,
                    Value::Integer(value) if value >= 0 => parsed.max_depth = Some(value as usize),
                    Value::Integer(_) => {
                        return Err(mlua::Error::runtime(
                            "ptool.inspect(value, options) `max_depth` must be >= 0",
                        ));
                    }
                    _ => {
                        return Err(mlua::Error::runtime(
                            "ptool.inspect(value, options) `max_depth` must be an integer",
                        ));
                    }
                },
                _ => {
                    return Err(mlua::Error::runtime(format!(
                        "ptool.inspect(value, options) unknown option `{key}`"
                    )));
                }
            }
        }

        Ok(parsed)
    }
}

struct Renderer {
    options: InspectOptions,
    seen_tables: HashSet<usize>,
}

impl Renderer {
    fn new(options: InspectOptions) -> Self {
        Self {
            options,
            seen_tables: HashSet::new(),
        }
    }

    fn render_value(&mut self, value: &Value, depth: usize) -> mlua::Result<String> {
        if self
            .options
            .max_depth
            .is_some_and(|max_depth| depth >= max_depth)
        {
            return Ok(MAX_DEPTH_MARKER.to_string());
        }

        match value {
            Value::Nil => Ok("nil".to_string()),
            Value::Boolean(value) => Ok(value.to_string()),
            Value::LightUserData(_) => Ok("<lightuserdata>".to_string()),
            Value::Integer(value) => Ok(value.to_string()),
            Value::Number(value) => Ok(format_number(*value)),
            Value::String(value) => Ok(render_string(value)),
            Value::Table(value) => self.render_table(value, depth),
            Value::Function(_) => Ok("<function>".to_string()),
            Value::Thread(_) => Ok("<thread>".to_string()),
            Value::UserData(_) => Ok("<userdata>".to_string()),
            Value::Error(err) => Ok(format!("<error: {err}>")),
            _ => Ok("<value>".to_string()),
        }
    }

    fn render_table(&mut self, table: &Table, depth: usize) -> mlua::Result<String> {
        let table_id = table.to_pointer() as usize;
        if !self.seen_tables.insert(table_id) {
            return Ok(CYCLE_MARKER.to_string());
        }

        let rendered = self.render_table_inner(table, depth);
        self.seen_tables.remove(&table_id);
        rendered
    }

    fn render_table_inner(&mut self, table: &Table, depth: usize) -> mlua::Result<String> {
        let array_len = table.raw_len();
        let mut array_items = Vec::with_capacity(array_len);
        for index in 1..=array_len {
            let value: Value = table.raw_get(index)?;
            array_items.push(self.render_value(&value, depth + 1)?);
        }

        let mut keyed_items = Vec::new();
        for pair in table.pairs::<Value, Value>() {
            let (key, value) = pair?;
            if is_array_index_key(&key, array_len) {
                continue;
            }

            let rendered_key = self.render_key(&key, depth + 1)?;
            let rendered_value = self.render_value(&value, depth + 1)?;
            let sort_key = key_sort_key(&key)?;
            keyed_items.push((sort_key, rendered_key, rendered_value));
        }

        keyed_items.sort_by(|a, b| a.0.cmp(&b.0));

        let mut entries = Vec::with_capacity(array_items.len() + keyed_items.len());
        entries.extend(array_items);
        entries.extend(
            keyed_items
                .into_iter()
                .map(|(_, key, value)| format!("{key} = {value}")),
        );

        self.join_table_entries(entries, depth)
    }

    fn render_key(&mut self, key: &Value, depth: usize) -> mlua::Result<String> {
        if let Value::String(key) = key
            && let Some(key) = identifier_name(key)
        {
            return Ok(key);
        }

        Ok(format!("[{}]", self.render_value(key, depth)?))
    }

    fn join_table_entries(&self, entries: Vec<String>, depth: usize) -> mlua::Result<String> {
        if entries.is_empty() {
            return Ok("{}".to_string());
        }

        let inline = render_inline_table(&entries);
        if !self.options.multiline {
            return Ok(inline);
        }

        if self.should_render_table_inline(&inline, &entries, depth) {
            return Ok(inline);
        }

        let current_indent = self.options.indent.repeat(depth);
        let next_indent = self.options.indent.repeat(depth + 1);
        let mut rendered = String::from("{");
        for entry in entries {
            rendered.push('\n');
            rendered.push_str(&next_indent);
            rendered.push_str(&entry);
            rendered.push(',');
        }
        rendered.push('\n');
        rendered.push_str(&current_indent);
        rendered.push('}');
        Ok(rendered)
    }

    fn should_render_table_inline(&self, inline: &str, entries: &[String], depth: usize) -> bool {
        let current_indent_width = self.options.indent.repeat(depth).chars().count();

        !entries
            .iter()
            .any(|entry| entry.contains('\n') || entry.contains('\r'))
            && current_indent_width + inline.chars().count() <= DEFAULT_INLINE_WIDTH
    }
}

fn render_inline_table(entries: &[String]) -> String {
    format!("{{ {} }}", entries.join(", "))
}

fn is_array_index_key(key: &Value, array_len: usize) -> bool {
    match key {
        Value::Integer(index) => *index >= 1 && (*index as usize) <= array_len,
        _ => false,
    }
}

fn key_sort_key(key: &Value) -> mlua::Result<String> {
    let sort_key = match key {
        Value::Nil => "0:nil".to_string(),
        Value::Boolean(value) => format!("1:boolean:{value}"),
        Value::LightUserData(_) => "2:lightuserdata".to_string(),
        Value::Integer(value) => format!("3:integer:{value}"),
        Value::Number(value) => format!("4:number:{}", format_number(*value)),
        Value::String(value) => format!("5:string:{}", render_string(value)),
        Value::Table(value) => format!("6:table:{:p}", value.to_pointer()),
        Value::Function(_) => "7:function".to_string(),
        Value::Thread(_) => "8:thread".to_string(),
        Value::UserData(_) => "9:userdata".to_string(),
        Value::Error(err) => format!("a:error:{err}"),
        _ => "z:value".to_string(),
    };
    Ok(sort_key)
}

fn identifier_name(value: &LuaString) -> Option<String> {
    let text = value.to_str().ok()?;
    if is_lua_identifier(text.as_ref()) {
        Some(text.to_string())
    } else {
        None
    }
}

fn is_lua_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn render_string(value: &LuaString) -> String {
    let mut rendered = String::new();
    rendered.push('"');
    for &byte in value.as_bytes().as_ref() {
        match byte {
            b'\\' => rendered.push_str("\\\\"),
            b'\"' => rendered.push_str("\\\""),
            b'\n' => rendered.push_str("\\n"),
            b'\r' => rendered.push_str("\\r"),
            b'\t' => rendered.push_str("\\t"),
            0 => rendered.push_str("\\0"),
            0x20..=0x7e => rendered.push(byte as char),
            _ => rendered.push_str(&format!("\\x{byte:02X}")),
        }
    }
    rendered.push('"');
    rendered
}

fn format_number(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }
    value.to_string()
}
