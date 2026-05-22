use crate::{Console, Error, ErrorKind, Result};

pub use ptool_console::LogLevel;

pub(crate) fn write_line(console: &Console, level: LogLevel, message: &str) -> Result<()> {
    console
        .log(level, message)
        .map_err(|err| io_error(err, level))
}

fn io_error(err: std::io::Error, level: LogLevel) -> Error {
    Error::new(
        ErrorKind::Io,
        format!("ptool.log.{} failed: {err}", op_suffix(level)),
    )
    .with_op(format!("ptool.log.{}", op_suffix(level)))
}

const fn op_suffix(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug",
        LogLevel::Info => "info ",
        LogLevel::Warn => "warn ",
        LogLevel::Error => "error",
        LogLevel::Fatal => "fatal",
    }
}
