use crate::{Color, Error, ErrorKind, Result, StyleOptions};
use jiff::Zoned;
use std::io::{self, IsTerminal, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn label(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Trace => Color::BrightBlack,
            Self::Debug => Color::Blue,
            Self::Info => Color::Green,
            Self::Warn => Color::Yellow,
            Self::Error => Color::Red,
        }
    }

    fn use_stderr(self) -> bool {
        matches!(self, Self::Error)
    }
}

pub(crate) fn format_line(level: LogLevel, message: &str, color_enabled: bool) -> String {
    let timestamp = Zoned::now().strftime("%Y-%m-%d %H:%M:%S").to_string();
    let timestamp = crate::ansi::style(
        format!("[{timestamp}]"),
        StyleOptions {
            enabled: color_enabled,
            fg: Some(Color::BrightBlack),
            ..StyleOptions::default()
        },
    );
    let level = crate::ansi::style(
        level.label().to_string(),
        StyleOptions {
            enabled: color_enabled,
            fg: Some(level.color()),
            bold: true,
            ..StyleOptions::default()
        },
    );

    if message.is_empty() {
        format!("{timestamp} {level}")
    } else {
        format!("{timestamp} {level} {message}")
    }
}

pub(crate) fn write_line(level: LogLevel, message: &str) -> Result<()> {
    if level.use_stderr() {
        let color_enabled = io::stderr().is_terminal();
        let rendered = format_line(level, message, color_enabled);
        let mut stderr = io::stderr().lock();
        writeln!(stderr, "{rendered}").map_err(|err| io_error(err, level))?;
    } else {
        let color_enabled = io::stdout().is_terminal();
        let rendered = format_line(level, message, color_enabled);
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{rendered}").map_err(|err| io_error(err, level))?;
    }

    Ok(())
}

fn io_error(err: io::Error, level: LogLevel) -> Error {
    Error::new(
        ErrorKind::Io,
        format!("ptool.log.{} failed: {err}", level.label().to_lowercase()),
    )
    .with_op(format!("ptool.log.{}", level.label().to_lowercase()))
}
