use mlua::{Lua, Table, Value};
use ptool_engine::{
    PromptConfirmOptions, PromptItem, PromptMultiSelectOptions, PromptSecretOptions,
    PromptSelectOptions, PromptTextOptions, PtoolEngine,
};

const ASK_SIGNATURE: &str = "ptool.ask(prompt[, options])";
const ASK_CONFIRM_SIGNATURE: &str = "ptool.ask.confirm(prompt[, options])";
const ASK_SELECT_SIGNATURE: &str = "ptool.ask.select(prompt, items[, options])";
const ASK_MULTISELECT_SIGNATURE: &str = "ptool.ask.multiselect(prompt, items[, options])";
const ASK_SECRET_SIGNATURE: &str = "ptool.ask.secret(prompt[, options])";

pub(crate) fn ask(
    engine: &PtoolEngine,
    prompt: String,
    options: Option<Table>,
) -> mlua::Result<String> {
    let options = parse_text_options(options, ASK_SIGNATURE)?;
    engine
        .prompt_text(ASK_SIGNATURE, &prompt, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, ASK_SIGNATURE))
}

pub(crate) fn ask_confirm(
    engine: &PtoolEngine,
    prompt: String,
    options: Option<Table>,
) -> mlua::Result<bool> {
    let options = parse_confirm_options(options, ASK_CONFIRM_SIGNATURE)?;
    engine
        .prompt_confirm(ASK_CONFIRM_SIGNATURE, &prompt, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, ASK_CONFIRM_SIGNATURE))
}

pub(crate) fn ask_select(
    engine: &PtoolEngine,
    prompt: String,
    items: Table,
    options: Option<Table>,
) -> mlua::Result<String> {
    let items = parse_items(items, ASK_SELECT_SIGNATURE)?;
    let options = parse_select_options(options, ASK_SELECT_SIGNATURE)?;
    engine
        .prompt_select(ASK_SELECT_SIGNATURE, &prompt, items, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, ASK_SELECT_SIGNATURE))
}

pub(crate) fn ask_multiselect(
    lua: &Lua,
    engine: &PtoolEngine,
    prompt: String,
    items: Table,
    options: Option<Table>,
) -> mlua::Result<Table> {
    let items = parse_items(items, ASK_MULTISELECT_SIGNATURE)?;
    let options = parse_multiselect_options(options, ASK_MULTISELECT_SIGNATURE)?;
    let selected = engine
        .prompt_multiselect(ASK_MULTISELECT_SIGNATURE, &prompt, items, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, ASK_MULTISELECT_SIGNATURE))?;
    let result = lua.create_table()?;
    for (index, value) in selected.into_iter().enumerate() {
        result.set(index + 1, value)?;
    }
    Ok(result)
}

pub(crate) fn ask_secret(
    engine: &PtoolEngine,
    prompt: String,
    options: Option<Table>,
) -> mlua::Result<String> {
    let options = parse_secret_options(options, ASK_SECRET_SIGNATURE)?;
    engine
        .prompt_secret(ASK_SECRET_SIGNATURE, &prompt, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, ASK_SECRET_SIGNATURE))
}

fn parse_text_options(options: Option<Table>, signature: &str) -> mlua::Result<PromptTextOptions> {
    let Some(options) = options else {
        return Ok(PromptTextOptions::default());
    };

    let mut parsed = PromptTextOptions::default();
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = option_key(key, signature)?;
        match key.as_str() {
            "default" => parsed.default = Some(parse_string_option(value, signature, "default")?),
            "help" => parsed.help = Some(parse_string_option(value, signature, "help")?),
            "placeholder" => {
                parsed.placeholder = Some(parse_string_option(value, signature, "placeholder")?)
            }
            "required" => parsed.required = parse_bool_option(value, signature, "required")?,
            "allow_empty" => {
                parsed.allow_empty = parse_bool_option(value, signature, "allow_empty")?
            }
            "trim" => parsed.trim = parse_bool_option(value, signature, "trim")?,
            "min_length" => {
                parsed.min_length = Some(parse_usize_option(value, signature, "min_length")?)
            }
            "max_length" => {
                parsed.max_length = Some(parse_usize_option(value, signature, "max_length")?)
            }
            "pattern" => parsed.pattern = Some(parse_string_option(value, signature, "pattern")?),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    signature,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_confirm_options(
    options: Option<Table>,
    signature: &str,
) -> mlua::Result<PromptConfirmOptions> {
    let Some(options) = options else {
        return Ok(PromptConfirmOptions::default());
    };

    let mut parsed = PromptConfirmOptions::default();
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = option_key(key, signature)?;
        match key.as_str() {
            "default" => parsed.default = Some(parse_bool_option(value, signature, "default")?),
            "help" => parsed.help = Some(parse_string_option(value, signature, "help")?),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    signature,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_select_options(
    options: Option<Table>,
    signature: &str,
) -> mlua::Result<PromptSelectOptions> {
    let Some(options) = options else {
        return Ok(PromptSelectOptions::default());
    };

    let mut parsed = PromptSelectOptions::default();
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = option_key(key, signature)?;
        match key.as_str() {
            "help" => parsed.help = Some(parse_string_option(value, signature, "help")?),
            "page_size" => {
                parsed.page_size = Some(parse_usize_option(value, signature, "page_size")?)
            }
            "default_index" => {
                let value = parse_usize_option(value, signature, "default_index")?;
                parsed.default_index = Some(value.saturating_sub(1));
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    signature,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_multiselect_options(
    options: Option<Table>,
    signature: &str,
) -> mlua::Result<PromptMultiSelectOptions> {
    let Some(options) = options else {
        return Ok(PromptMultiSelectOptions::default());
    };

    let mut parsed = PromptMultiSelectOptions::default();
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = option_key(key, signature)?;
        match key.as_str() {
            "help" => parsed.help = Some(parse_string_option(value, signature, "help")?),
            "page_size" => {
                parsed.page_size = Some(parse_usize_option(value, signature, "page_size")?)
            }
            "default_indexes" => {
                parsed.default_indexes =
                    parse_usize_array_option(value, signature, "default_indexes")?
                        .into_iter()
                        .map(|index| index.saturating_sub(1))
                        .collect();
            }
            "min_selected" => {
                parsed.min_selected = Some(parse_usize_option(value, signature, "min_selected")?)
            }
            "max_selected" => {
                parsed.max_selected = Some(parse_usize_option(value, signature, "max_selected")?)
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    signature,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_secret_options(
    options: Option<Table>,
    signature: &str,
) -> mlua::Result<PromptSecretOptions> {
    let Some(options) = options else {
        return Ok(PromptSecretOptions::default());
    };

    let mut parsed = PromptSecretOptions::default();
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = option_key(key, signature)?;
        match key.as_str() {
            "help" => parsed.help = Some(parse_string_option(value, signature, "help")?),
            "required" => parsed.required = parse_bool_option(value, signature, "required")?,
            "allow_empty" => {
                parsed.allow_empty = parse_bool_option(value, signature, "allow_empty")?
            }
            "confirm" => parsed.confirm = parse_bool_option(value, signature, "confirm")?,
            "confirm_prompt" => {
                parsed.confirm_prompt =
                    Some(parse_string_option(value, signature, "confirm_prompt")?)
            }
            "mismatch_message" => {
                parsed.mismatch_message =
                    Some(parse_string_option(value, signature, "mismatch_message")?)
            }
            "display_toggle" => {
                parsed.display_toggle = parse_bool_option(value, signature, "display_toggle")?
            }
            "min_length" => {
                parsed.min_length = Some(parse_usize_option(value, signature, "min_length")?)
            }
            "max_length" => {
                parsed.max_length = Some(parse_usize_option(value, signature, "max_length")?)
            }
            "pattern" => parsed.pattern = Some(parse_string_option(value, signature, "pattern")?),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    signature,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_items(items: Table, signature: &str) -> mlua::Result<Vec<PromptItem>> {
    let mut parsed = Vec::new();
    for value in items.sequence_values::<Value>() {
        let value = value?;
        match value {
            Value::String(value) => parsed.push(PromptItem {
                label: value.to_str()?.to_string(),
                value: value.to_str()?.to_string(),
            }),
            Value::Table(table) => {
                let label: String = table.get("label").map_err(|_| {
                    crate::lua_error::invalid_option(signature, "`items[].label` must be a string")
                })?;
                let value: String = table.get("value").map_err(|_| {
                    crate::lua_error::invalid_option(signature, "`items[].value` must be a string")
                })?;
                parsed.push(PromptItem { label, value });
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    signature,
                    "`items` entries must be strings or tables",
                ));
            }
        }
    }
    Ok(parsed)
}

fn option_key(key: Value, signature: &str) -> mlua::Result<String> {
    match key {
        Value::String(key) => Ok(key.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_option(
            signature,
            "option keys must be strings",
        )),
    }
}

fn parse_string_option(value: Value, signature: &str, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_option(
            signature,
            format!("`{field}` must be a string"),
        )),
    }
}

fn parse_bool_option(value: Value, signature: &str, field: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(value) => Ok(value),
        _ => Err(crate::lua_error::invalid_option(
            signature,
            format!("`{field}` must be a boolean"),
        )),
    }
}

fn parse_usize_option(value: Value, signature: &str, field: &str) -> mlua::Result<usize> {
    match value {
        Value::Integer(value) if value > 0 => Ok(value as usize),
        Value::Integer(_) => Err(crate::lua_error::invalid_option(
            signature,
            format!("`{field}` must be a positive integer"),
        )),
        _ => Err(crate::lua_error::invalid_option(
            signature,
            format!("`{field}` must be an integer"),
        )),
    }
}

fn parse_usize_array_option(
    value: Value,
    signature: &str,
    field: &str,
) -> mlua::Result<Vec<usize>> {
    let Value::Table(table) = value else {
        return Err(crate::lua_error::invalid_option(
            signature,
            format!("`{field}` must be an array of integers"),
        ));
    };

    let mut values = Vec::new();
    for value in table.sequence_values::<Value>() {
        values.push(parse_usize_option(value?, signature, field)?);
    }
    Ok(values)
}
