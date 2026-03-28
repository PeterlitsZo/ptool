use mlua::{Table, Value};
use ptool_engine::{Color, StyleOptions};

const STYLE_SIGNATURE: &str = "ptool.ansi.style(text, options)";

pub(crate) fn style_options(
    options: Option<Table>,
    forced_fg: Option<&str>,
) -> mlua::Result<StyleOptions> {
    let mut parsed = StyleOptions::default();

    if let Some(options) = options {
        for pair in options.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let key = match key {
                Value::String(key) => key.to_str()?.to_string(),
                _ => {
                    return Err(mlua::Error::runtime(
                        "ptool.ansi.style(text, options) option keys must be strings",
                    ));
                }
            };

            match key.as_str() {
                "enabled" => parsed.enabled = parse_bool(value, "enabled")?,
                "fg" => {
                    parsed.fg = match value {
                        Value::Nil => None,
                        Value::String(value) => Some(parse_color(value.to_str()?.as_ref())?),
                        _ => {
                            return Err(mlua::Error::runtime(format!(
                                "{STYLE_SIGNATURE} `fg` must be a string"
                            )));
                        }
                    };
                }
                "bold" => parsed.bold = parse_bool(value, "bold")?,
                "dimmed" => parsed.dimmed = parse_bool(value, "dimmed")?,
                "italic" => parsed.italic = parse_bool(value, "italic")?,
                "underline" => parsed.underline = parse_bool(value, "underline")?,
                _ => {
                    return Err(mlua::Error::runtime(format!(
                        "{STYLE_SIGNATURE} unknown option `{key}`"
                    )));
                }
            }
        }
    }

    if let Some(forced_fg) = forced_fg {
        parsed.fg = Some(parse_color(forced_fg)?);
    }

    Ok(parsed)
}

fn parse_bool(value: Value, field: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(value) => Ok(value),
        _ => Err(mlua::Error::runtime(format!(
            "{STYLE_SIGNATURE} `{field}` must be a boolean"
        ))),
    }
}

fn parse_color(value: &str) -> mlua::Result<Color> {
    match value {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" | "purple" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "white" => Ok(Color::White),
        "bright_black" => Ok(Color::BrightBlack),
        "bright_red" => Ok(Color::BrightRed),
        "bright_green" => Ok(Color::BrightGreen),
        "bright_yellow" => Ok(Color::BrightYellow),
        "bright_blue" => Ok(Color::BrightBlue),
        "bright_magenta" | "bright_purple" => Ok(Color::BrightMagenta),
        "bright_cyan" => Ok(Color::BrightCyan),
        "bright_white" => Ok(Color::BrightWhite),
        _ => Err(mlua::Error::runtime(format!(
            "{STYLE_SIGNATURE} invalid `fg` color `{value}`"
        ))),
    }
}
