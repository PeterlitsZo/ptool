use std::io::IsTerminal;

use owo_colors::Style as OwoStyle;

/// ANSI styling options for terminal output.
pub struct StyleOptions {
    pub enabled: bool,
    pub fg: Option<Color>,
    pub bold: bool,
    pub dimmed: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for StyleOptions {
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

#[derive(Clone, Copy)]
pub enum Color {
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

impl Color {
    fn apply_fg(self, style: OwoStyle) -> OwoStyle {
        match self {
            Color::Black => style.black(),
            Color::Red => style.red(),
            Color::Green => style.green(),
            Color::Yellow => style.yellow(),
            Color::Blue => style.blue(),
            Color::Magenta => style.magenta(),
            Color::Cyan => style.cyan(),
            Color::White => style.white(),
            Color::BrightBlack => style.bright_black(),
            Color::BrightRed => style.bright_red(),
            Color::BrightGreen => style.bright_green(),
            Color::BrightYellow => style.bright_yellow(),
            Color::BrightBlue => style.bright_blue(),
            Color::BrightMagenta => style.bright_magenta(),
            Color::BrightCyan => style.bright_cyan(),
            Color::BrightWhite => style.bright_white(),
        }
    }
}

pub(crate) fn style(text: String, options: StyleOptions) -> String {
    if !options.enabled {
        return text;
    }

    let mut style = OwoStyle::new();
    if let Some(fg) = options.fg {
        style = fg.apply_fg(style);
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

    style.style(text).to_string()
}
