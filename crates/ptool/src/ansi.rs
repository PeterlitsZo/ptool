use mlua::{Table, Value};
use owo_colors::Style;
use std::io::IsTerminal;

const STYLE_SIGNATURE: &str = "ptool.ansi.style(text, options)";

pub(crate) fn style(
    text: String,
    options: Option<Table>,
    forced_fg: Option<&str>,
) -> mlua::Result<String> {
    let options = AnsiOptions::parse(options, forced_fg)?;
    if !options.enabled {
        return Ok(text);
    }

    let mut style = Style::new();
    if let Some(fg) = options.fg {
        style = fg.apply(style);
    }
    if options.bold {
        style = style.bold();
    }
    if options.dimmed {
        style = style.dimmed();
    }
    if options.italic {
        style = style.italic();
    }
    if options.underline {
        style = style.underline();
    }

    Ok(style.style(text).to_string())
}

#[derive(Debug, Clone, Copy)]
struct AnsiOptions {
    enabled: bool,
    fg: Option<FgColor>,
    bold: bool,
    dimmed: bool,
    italic: bool,
    underline: bool,
}

impl Default for AnsiOptions {
    fn default() -> Self {
        Self {
            enabled: std::io::stdout().is_terminal(),
            fg: None,
            bold: false,
            dimmed: false,
            italic: false,
            underline: false,
        }
    }
}

impl AnsiOptions {
    fn parse(options: Option<Table>, forced_fg: Option<&str>) -> mlua::Result<Self> {
        let mut parsed = Self::default();

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
                            Value::String(value) => Some(FgColor::parse(value.to_str()?.as_ref())?),
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
            parsed.fg = Some(FgColor::parse(forced_fg)?);
        }

        Ok(parsed)
    }
}

fn parse_bool(value: Value, field: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(value) => Ok(value),
        _ => Err(mlua::Error::runtime(format!(
            "{STYLE_SIGNATURE} `{field}` must be a boolean"
        ))),
    }
}

#[derive(Debug, Clone, Copy)]
enum FgColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl FgColor {
    fn parse(value: &str) -> mlua::Result<Self> {
        match value {
            "black" => Ok(Self::Black),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "yellow" => Ok(Self::Yellow),
            "blue" => Ok(Self::Blue),
            "magenta" | "purple" => Ok(Self::Magenta),
            "cyan" => Ok(Self::Cyan),
            "white" => Ok(Self::White),
            "bright_black" => Ok(Self::BrightBlack),
            "bright_red" => Ok(Self::BrightRed),
            "bright_green" => Ok(Self::BrightGreen),
            "bright_yellow" => Ok(Self::BrightYellow),
            "bright_blue" => Ok(Self::BrightBlue),
            "bright_magenta" | "bright_purple" => Ok(Self::BrightMagenta),
            "bright_cyan" => Ok(Self::BrightCyan),
            "bright_white" => Ok(Self::BrightWhite),
            _ => Err(mlua::Error::runtime(format!(
                "{STYLE_SIGNATURE} invalid `fg` color `{value}`"
            ))),
        }
    }

    fn apply(self, style: Style) -> Style {
        match self {
            Self::Black => style.black(),
            Self::Red => style.red(),
            Self::Green => style.green(),
            Self::Yellow => style.yellow(),
            Self::Blue => style.blue(),
            Self::Magenta => style.magenta(),
            Self::Cyan => style.cyan(),
            Self::White => style.white(),
            Self::BrightBlack => style.bright_black(),
            Self::BrightRed => style.bright_red(),
            Self::BrightGreen => style.bright_green(),
            Self::BrightYellow => style.bright_yellow(),
            Self::BrightBlue => style.bright_blue(),
            Self::BrightMagenta => style.bright_magenta(),
            Self::BrightCyan => style.bright_cyan(),
            Self::BrightWhite => style.bright_white(),
        }
    }
}
