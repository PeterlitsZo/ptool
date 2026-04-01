use inquire::{InquireError, Text};
use mlua::{Table, Value};

const ASK_SIGNATURE: &str = "ptool.ask(prompt[, options])";

#[derive(Debug, Default)]
struct AskOptions {
    default: Option<String>,
    help: Option<String>,
    placeholder: Option<String>,
}

pub(crate) fn ask(prompt: String, options: Option<Table>) -> mlua::Result<String> {
    ensure_non_empty(&prompt, ASK_SIGNATURE, "prompt")?;
    let options = AskOptions::parse(options)?;

    let mut text = Text::new(&prompt);
    if let Some(default) = options.default.as_deref() {
        text = text.with_default(default);
    }
    if let Some(help) = options.help.as_deref() {
        text = text.with_help_message(help);
    }
    if let Some(placeholder) = options.placeholder.as_deref() {
        text = text.with_placeholder(placeholder);
    }

    match text.prompt() {
        Ok(value) => Ok(value),
        Err(InquireError::NotTTY | InquireError::IO(_)) => Err(mlua::Error::runtime(format!(
            "{ASK_SIGNATURE} requires an interactive TTY"
        ))),
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => Err(
            mlua::Error::runtime(format!("{ASK_SIGNATURE} cancelled by user")),
        ),
        Err(err) => Err(mlua::Error::runtime(format!(
            "{ASK_SIGNATURE} failed: {err}"
        ))),
    }
}

impl AskOptions {
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
                    return Err(mlua::Error::runtime(format!(
                        "{ASK_SIGNATURE} option keys must be strings"
                    )));
                }
            };

            match key.as_str() {
                "default" => {
                    parsed.default = Some(parse_string_option(value, "default")?);
                }
                "help" => {
                    parsed.help = Some(parse_string_option(value, "help")?);
                }
                "placeholder" => {
                    parsed.placeholder = Some(parse_string_option(value, "placeholder")?);
                }
                _ => {
                    return Err(mlua::Error::runtime(format!(
                        "{ASK_SIGNATURE} unknown option `{key}`"
                    )));
                }
            }
        }

        Ok(parsed)
    }
}

fn parse_string_option(value: Value, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(mlua::Error::runtime(format!(
            "{ASK_SIGNATURE} `{field}` must be a string"
        ))),
    }
}

fn ensure_non_empty(input: &str, signature: &str, field: &str) -> mlua::Result<()> {
    if input.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{signature} `{field}` must not be empty"
        )));
    }
    Ok(())
}
