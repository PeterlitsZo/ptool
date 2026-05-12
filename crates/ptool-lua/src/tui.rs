use mlua::{Function, Lua, RegistryKey, Table, UserData, UserDataMethods, Value};
use ptool_engine::{
    PtoolEngine, TuiColor, TuiContainerDirection, TuiEvent, TuiHighlight, TuiKeyEvent, TuiNode,
    TuiNodeBlock, TuiNodeKind, TuiSessionOptions, TuiStyle, TuiTextAlign,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const RUN_SIGNATURE: &str = "ptool.tui.run(options)";
#[derive(Clone, Default)]
struct LuaTuiApp {
    state: Rc<RefCell<LuaTuiAppState>>,
}

#[derive(Default)]
struct LuaTuiAppState {
    quit_requested: bool,
    result: Option<RegistryKey>,
}

pub(crate) fn run(lua: &Lua, engine: &PtoolEngine, options: Table) -> mlua::Result<Value> {
    validate_run_option_keys(&options)?;

    let init = options.get::<Option<Function>>("init")?;
    let Some(update) = options.get::<Option<Function>>("update")? else {
        return Err(crate::lua_error::invalid_argument(
            RUN_SIGNATURE,
            "requires function `update`",
        ));
    };
    let Some(view) = options.get::<Option<Function>>("view")? else {
        return Err(crate::lua_error::invalid_argument(
            RUN_SIGNATURE,
            "requires function `view`",
        ));
    };
    let tick_ms = options.get::<Option<i64>>("tick_ms")?.unwrap_or(100);
    let tick_ms = parse_positive_u64(tick_ms, RUN_SIGNATURE, "tick_ms")?;

    let mut session = engine
        .tui_session(TuiSessionOptions {
            tick_rate: Duration::from_millis(tick_ms),
        })
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, RUN_SIGNATURE))?;

    let mut state = match init {
        Some(init) => init.call::<Value>(())?,
        None => Value::Nil,
    };
    let app = LuaTuiApp::default();

    loop {
        let root_value = view.call::<Value>((state.clone(), app.clone()))?;
        let root = parse_node_value(root_value, RUN_SIGNATURE)?;
        session
            .draw(&root)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, RUN_SIGNATURE))?;

        if app.quit_requested() {
            break;
        }

        let event = session
            .next_event()
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, RUN_SIGNATURE))?;
        let event = event_to_lua(lua, event)?;
        let next_state = update.call::<Value>((state.clone(), event, app.clone()))?;
        if !matches!(next_state, Value::Nil) {
            state = next_state;
        }

        if app.quit_requested() {
            break;
        }
    }

    app.take_result(lua)
}

pub(crate) fn text_node(lua: &Lua, text: String, options: Option<Table>) -> mlua::Result<Table> {
    let node = lua.create_table()?;
    node.set("kind", "text")?;
    node.set("text", text)?;
    if let Some(options) = options {
        copy_table_fields(&node, &options)?;
    }
    Ok(node)
}

pub(crate) fn list_node(lua: &Lua, items: Table, options: Option<Table>) -> mlua::Result<Table> {
    let node = lua.create_table()?;
    node.set("kind", "list")?;
    node.set("items", items)?;
    if let Some(options) = options {
        copy_table_fields(&node, &options)?;
    }
    Ok(node)
}

pub(crate) fn row_node(lua: &Lua, options: Table) -> mlua::Result<Table> {
    let node = lua.create_table()?;
    node.set("kind", "row")?;
    copy_table_fields(&node, &options)?;
    Ok(node)
}

pub(crate) fn column_node(lua: &Lua, options: Table) -> mlua::Result<Table> {
    let node = lua.create_table()?;
    node.set("kind", "column")?;
    copy_table_fields(&node, &options)?;
    Ok(node)
}

impl UserData for LuaTuiApp {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("quit", |lua, this, value: Option<Value>| {
            this.quit(lua, value.unwrap_or(Value::Nil))
        });
    }
}

impl LuaTuiApp {
    fn quit(&self, lua: &Lua, value: Value) -> mlua::Result<()> {
        let mut state = self.state.borrow_mut();
        if let Some(previous) = state.result.take() {
            lua.remove_registry_value(previous)?;
        }
        if !matches!(value, Value::Nil) {
            state.result = Some(lua.create_registry_value(value)?);
        }
        state.quit_requested = true;
        Ok(())
    }

    fn quit_requested(&self) -> bool {
        self.state.borrow().quit_requested
    }

    fn take_result(&self, lua: &Lua) -> mlua::Result<Value> {
        let mut state = self.state.borrow_mut();
        let Some(result) = state.result.take() else {
            return Ok(Value::Nil);
        };
        let value = lua.registry_value(&result)?;
        lua.remove_registry_value(result)?;
        Ok(value)
    }
}

fn validate_run_option_keys(options: &Table) -> mlua::Result<()> {
    for pair in options.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = parse_string_key(key, RUN_SIGNATURE, "option")?;
        match key.as_str() {
            "tick_ms" | "init" | "update" | "view" => {}
            _ => {
                return Err(crate::lua_error::invalid_option(
                    RUN_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(())
}

fn parse_node_value(value: Value, signature: &str) -> mlua::Result<TuiNode> {
    let Value::Table(node) = value else {
        return Err(crate::lua_error::invalid_argument(
            signature,
            "`view` must return a node table",
        ));
    };

    let Some(kind) = node.get::<Option<String>>("kind")? else {
        return Err(crate::lua_error::invalid_argument(
            signature,
            "node table requires string `kind`",
        ));
    };

    match kind.as_str() {
        "text" => parse_text_node(node, signature),
        "list" => parse_list_node(node, signature),
        "row" => parse_container_node(node, signature, TuiContainerDirection::Row),
        "column" => parse_container_node(node, signature, TuiContainerDirection::Column),
        _ => Err(crate::lua_error::invalid_argument(
            signature,
            format!("unknown node kind `{kind}`"),
        )),
    }
}

fn parse_text_node(node: Table, signature: &str) -> mlua::Result<TuiNode> {
    validate_node_keys(
        &node,
        signature,
        &[
            "kind", "text", "align", "title", "border", "padding", "grow", "style",
        ],
    )?;
    let Some(text) = node.get::<Option<String>>("text")? else {
        return Err(crate::lua_error::invalid_argument(
            signature,
            "text node requires `text`",
        ));
    };
    let align = parse_text_align(node.get::<Option<String>>("align")?, signature, "align")?;
    let (block, style, grow) = parse_common_node_options(&node, signature)?;
    Ok(TuiNode {
        block,
        style,
        grow,
        kind: TuiNodeKind::Text { text, align },
    })
}

fn parse_list_node(node: Table, signature: &str) -> mlua::Result<TuiNode> {
    validate_node_keys(
        &node,
        signature,
        &[
            "kind",
            "items",
            "selected",
            "highlight_style",
            "title",
            "border",
            "padding",
            "grow",
            "style",
        ],
    )?;
    let Some(items) = node.get::<Option<Table>>("items")? else {
        return Err(crate::lua_error::invalid_argument(
            signature,
            "list node requires `items`",
        ));
    };
    let items = parse_string_list(items, signature, "items")?;
    let selected = node
        .get::<Option<i64>>("selected")?
        .map(|value| parse_selected_index(value, signature))
        .transpose()?;
    let highlight = TuiHighlight {
        style: parse_style(
            node.get::<Option<Table>>("highlight_style")?,
            signature,
            "highlight_style",
        )?,
    };
    let (block, style, grow) = parse_common_node_options(&node, signature)?;
    Ok(TuiNode {
        block,
        style,
        grow,
        kind: TuiNodeKind::List {
            items,
            selected,
            highlight,
        },
    })
}

fn parse_container_node(
    node: Table,
    signature: &str,
    direction: TuiContainerDirection,
) -> mlua::Result<TuiNode> {
    validate_node_keys(
        &node,
        signature,
        &[
            "kind", "children", "title", "border", "padding", "grow", "style",
        ],
    )?;
    let Some(children) = node.get::<Option<Table>>("children")? else {
        return Err(crate::lua_error::invalid_argument(
            signature,
            "container node requires `children`",
        ));
    };
    let mut parsed_children = Vec::new();
    for value in children.sequence_values::<Value>() {
        parsed_children.push(parse_node_value(value?, signature)?);
    }
    let (block, style, grow) = parse_common_node_options(&node, signature)?;
    Ok(TuiNode {
        block,
        style,
        grow,
        kind: TuiNodeKind::Container {
            direction,
            children: parsed_children,
        },
    })
}

fn parse_common_node_options(
    node: &Table,
    signature: &str,
) -> mlua::Result<(TuiNodeBlock, TuiStyle, u16)> {
    let title = node.get::<Option<String>>("title")?;
    let border = node.get::<Option<bool>>("border")?.unwrap_or(false);
    let padding = node
        .get::<Option<i64>>("padding")?
        .map(|value| parse_u16(value, signature, "padding"))
        .transpose()?
        .unwrap_or(0);
    let grow = node
        .get::<Option<i64>>("grow")?
        .map(|value| parse_positive_u16(value, signature, "grow"))
        .transpose()?
        .unwrap_or(1);
    let style = parse_style(node.get::<Option<Table>>("style")?, signature, "style")?;
    Ok((
        TuiNodeBlock {
            title,
            border,
            padding,
        },
        style,
        grow,
    ))
}

fn parse_style(options: Option<Table>, signature: &str, field: &str) -> mlua::Result<TuiStyle> {
    let Some(options) = options else {
        return Ok(TuiStyle::default());
    };

    let mut style = TuiStyle::default();
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_string_key(key, signature, field)?;
        match key.as_str() {
            "fg" => style.fg = Some(parse_color(value, signature, field, "fg")?),
            "bg" => style.bg = Some(parse_color(value, signature, field, "bg")?),
            "bold" => style.bold = parse_bool(value, signature, field, "bold")?,
            "dim" => style.dim = parse_bool(value, signature, field, "dim")?,
            "italic" => style.italic = parse_bool(value, signature, field, "italic")?,
            "underlined" => style.underlined = parse_bool(value, signature, field, "underlined")?,
            "reversed" => style.reversed = parse_bool(value, signature, field, "reversed")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    signature,
                    format!("`{field}` unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(style)
}

fn parse_color(value: Value, signature: &str, field: &str, key: &str) -> mlua::Result<TuiColor> {
    let Value::String(value) = value else {
        return Err(crate::lua_error::invalid_argument(
            signature,
            format!("`{field}.{key}` must be a string"),
        ));
    };
    let value = value.to_str()?.to_string();
    TuiColor::parse(&value).ok_or_else(|| {
        crate::lua_error::invalid_argument(
            signature,
            format!("`{field}.{key}` has unsupported color `{value}`"),
        )
    })
}

fn parse_text_align(
    value: Option<String>,
    signature: &str,
    key: &str,
) -> mlua::Result<TuiTextAlign> {
    match value.as_deref().unwrap_or("left") {
        "left" => Ok(TuiTextAlign::Left),
        "center" => Ok(TuiTextAlign::Center),
        "right" => Ok(TuiTextAlign::Right),
        other => Err(crate::lua_error::invalid_argument(
            signature,
            format!("`{key}` has unsupported value `{other}`"),
        )),
    }
}

fn parse_selected_index(value: i64, signature: &str) -> mlua::Result<usize> {
    if value <= 0 {
        return Err(crate::lua_error::invalid_argument(
            signature,
            "`selected` must be >= 1",
        ));
    }
    usize::try_from(value - 1)
        .map_err(|_| crate::lua_error::invalid_argument(signature, "`selected` is too large"))
}

fn parse_string_list(items: Table, signature: &str, field: &str) -> mlua::Result<Vec<String>> {
    let mut parsed = Vec::new();
    for value in items.sequence_values::<Value>() {
        parsed.push(parse_string_like_value(value?, signature, field)?);
    }
    Ok(parsed)
}

fn parse_string_like_value(value: Value, signature: &str, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        Value::Integer(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Boolean(value) => Ok(value.to_string()),
        _ => Err(crate::lua_error::invalid_argument(
            signature,
            format!("`{field}` values must be strings, numbers, or booleans"),
        )),
    }
}

fn validate_node_keys(node: &Table, signature: &str, allowed: &[&str]) -> mlua::Result<()> {
    for pair in node.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = parse_string_key(key, signature, "node")?;
        if !allowed.contains(&key.as_str()) {
            return Err(crate::lua_error::invalid_option(
                signature,
                format!("unknown node field `{key}`"),
            ));
        }
    }
    Ok(())
}

fn event_to_lua(lua: &Lua, event: TuiEvent) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    match event {
        TuiEvent::Tick => {
            table.set("type", "tick")?;
        }
        TuiEvent::Resize { width, height } => {
            table.set("type", "resize")?;
            table.set("width", i64::from(width))?;
            table.set("height", i64::from(height))?;
        }
        TuiEvent::Key(TuiKeyEvent {
            key,
            ctrl,
            alt,
            shift,
        }) => {
            table.set("type", "key")?;
            table.set("key", key)?;
            table.set("ctrl", ctrl)?;
            table.set("alt", alt)?;
            table.set("shift", shift)?;
        }
    }
    Ok(table)
}

fn copy_table_fields(target: &Table, source: &Table) -> mlua::Result<()> {
    for pair in source.pairs::<Value, Value>() {
        let (key, value) = pair?;
        target.set(key, value)?;
    }
    Ok(())
}

fn parse_string_key(value: Value, signature: &str, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_argument(
            signature,
            format!("`{field}` keys must be strings"),
        )),
    }
}

fn parse_bool(value: Value, signature: &str, field: &str, key: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(value) => Ok(value),
        _ => Err(crate::lua_error::invalid_argument(
            signature,
            format!("`{field}.{key}` must be a boolean"),
        )),
    }
}

fn parse_u16(value: i64, signature: &str, key: &str) -> mlua::Result<u16> {
    u16::try_from(value).map_err(|_| {
        crate::lua_error::invalid_argument(
            signature,
            format!("`{key}` must be between 0 and 65535"),
        )
    })
}

fn parse_positive_u16(value: i64, signature: &str, key: &str) -> mlua::Result<u16> {
    if value <= 0 {
        return Err(crate::lua_error::invalid_argument(
            signature,
            format!("`{key}` must be > 0"),
        ));
    }
    u16::try_from(value)
        .map_err(|_| crate::lua_error::invalid_argument(signature, format!("`{key}` is too large")))
}

fn parse_positive_u64(value: i64, signature: &str, key: &str) -> mlua::Result<u64> {
    if value <= 0 {
        return Err(crate::lua_error::invalid_argument(
            signature,
            format!("`{key}` must be > 0"),
        ));
    }
    u64::try_from(value)
        .map_err(|_| crate::lua_error::invalid_argument(signature, format!("`{key}` is too large")))
}
