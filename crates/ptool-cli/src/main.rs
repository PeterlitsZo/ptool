use clap_lex::RawArgs;
use indoc::formatdoc;
use shadow_rs::shadow;
use std::ffi::OsStr;
use std::process;

shadow!(build);

const APP_NAME: &str = "ptool";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_ABOUT: &str = "The PTOOL CLI";
const LUA_RUNTIME: &str = "Lua 5.4 (embedded, vendored via mlua)";
const WEBSITE_URL: &str = "https://ptool.peterlits.net";

#[derive(Debug)]
enum ParsedCli {
    Run {
        filename: String,
        script_args: Vec<String>,
    },
    Repl,
    ExitSuccess,
}

#[derive(Clone, Copy, Debug)]
enum UsageKind {
    Top,
    Run,
    Repl,
    Version,
}

#[derive(Debug)]
struct ParseError {
    message: String,
    usage: UsageKind,
}

impl ParseError {
    fn top(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            usage: UsageKind::Top,
        }
    }

    fn run(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            usage: UsageKind::Run,
        }
    }

    fn version(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            usage: UsageKind::Version,
        }
    }

    fn repl(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            usage: UsageKind::Repl,
        }
    }
}

fn main() {
    let raw = RawArgs::from_args();
    let mut cursor = raw.cursor();
    let bin = raw
        .next_os(&mut cursor)
        .unwrap_or_else(|| OsStr::new(APP_NAME));

    match parse_cli(&raw, &mut cursor, bin) {
        Ok(ParsedCli::Run {
            filename,
            script_args,
        }) => {
            if let Err(err) = ptool_lua::run_script(&filename, &script_args) {
                eprintln!(
                    "Failed to run Lua script `{filename}`:\n{}",
                    ptool_lua::format_error_report(err.as_ref())
                );
                process::exit(1);
            }
        }
        Ok(ParsedCli::Repl) => {
            if let Err(err) = ptool_lua::run_repl() {
                eprintln!(
                    "Failed to start REPL:\n{}",
                    ptool_lua::format_error_report(err.as_ref())
                );
                process::exit(1);
            }
        }
        Ok(ParsedCli::ExitSuccess) => {}
        Err(err) => {
            eprintln!("error: {}", err.message);
            eprintln!();
            match err.usage {
                UsageKind::Top => eprintln!("{}", top_usage(bin)),
                UsageKind::Run => eprintln!("{}", run_usage(bin)),
                UsageKind::Repl => eprintln!("{}", repl_usage(bin)),
                UsageKind::Version => eprintln!("{}", version_usage(bin)),
            }
            process::exit(2);
        }
    }
}

fn parse_cli(
    raw: &RawArgs,
    cursor: &mut clap_lex::ArgCursor,
    bin: &OsStr,
) -> Result<ParsedCli, ParseError> {
    let Some(command) = raw.next(cursor) else {
        return Err(ParseError::top("a subcommand is required"));
    };

    let command_value = parsed_arg_to_string(command.to_value_os(), "argument")?;
    match command_value.as_str() {
        "-h" | "--help" => {
            println!("{}", top_usage(bin));
            Ok(ParsedCli::ExitSuccess)
        }
        "-V" | "--version" => {
            print_version();
            Ok(ParsedCli::ExitSuccess)
        }
        "run" => parse_run(raw, cursor, bin),
        "repl" => parse_repl(raw, cursor, bin),
        "version" => parse_version(raw, cursor, bin),
        value if value.ends_with(".lua") => parse_run_with_filename(raw, cursor, value.to_string()),
        value if value.starts_with('-') => Err(ParseError::top(format!(
            "unexpected argument `{value}` found"
        ))),
        value => Err(ParseError::top(format!(
            "unrecognized subcommand `{value}`"
        ))),
    }
}

fn parse_run_with_filename(
    raw: &RawArgs,
    cursor: &mut clap_lex::ArgCursor,
    filename: String,
) -> Result<ParsedCli, ParseError> {
    let mut script_args = Vec::new();
    if let Some(next) = raw.next(cursor) {
        if next.is_escape() {
            for arg in raw.remaining(cursor) {
                script_args.push(parsed_arg_to_string(arg, "script argument")?);
            }
        } else {
            script_args.push(parsed_arg_to_string(next.to_value_os(), "script argument")?);
            for arg in raw.remaining(cursor) {
                script_args.push(parsed_arg_to_string(arg, "script argument")?);
            }
        }
    }

    Ok(ParsedCli::Run {
        filename,
        script_args,
    })
}

fn parse_run(
    raw: &RawArgs,
    cursor: &mut clap_lex::ArgCursor,
    bin: &OsStr,
) -> Result<ParsedCli, ParseError> {
    let Some(next) = raw.next(cursor) else {
        return Err(ParseError::run(
            "the following required argument was not provided: <filename>",
        ));
    };

    if next.to_value_os() == OsStr::new("-h") || next.to_value_os() == OsStr::new("--help") {
        println!("{}", run_usage(bin));
        return Ok(ParsedCli::ExitSuccess);
    }

    if next.is_escape() {
        return Err(ParseError::run(
            "the following required argument was not provided: <filename>",
        ));
    }

    let filename = parsed_arg_to_string(next.to_value_os(), "filename")?;
    parse_run_with_filename(raw, cursor, filename)
}

fn parse_repl(
    raw: &RawArgs,
    cursor: &mut clap_lex::ArgCursor,
    bin: &OsStr,
) -> Result<ParsedCli, ParseError> {
    if let Some(next) = raw.next(cursor) {
        let value = parsed_arg_to_string(next.to_value_os(), "argument")?;
        if value == "-h" || value == "--help" {
            println!("{}", repl_usage(bin));
            return Ok(ParsedCli::ExitSuccess);
        }

        return Err(ParseError::repl(format!(
            "unexpected argument `{value}` found"
        )));
    }

    Ok(ParsedCli::Repl)
}

fn parse_version(
    raw: &RawArgs,
    cursor: &mut clap_lex::ArgCursor,
    bin: &OsStr,
) -> Result<ParsedCli, ParseError> {
    if let Some(next) = raw.next(cursor) {
        let value = parsed_arg_to_string(next.to_value_os(), "argument")?;
        if value == "-h" || value == "--help" {
            println!("{}", version_usage(bin));
            return Ok(ParsedCli::ExitSuccess);
        }

        return Err(ParseError::version(format!(
            "unexpected argument `{value}` found"
        )));
    }

    print_version();
    Ok(ParsedCli::ExitSuccess)
}

fn print_version() {
    let mut lines = vec![
        ("name", APP_NAME.to_string()),
        ("version", APP_VERSION.to_string()),
    ];

    if let Some(tag) = non_empty(build::TAG) {
        lines.push(("tag", tag.to_string()));
    }

    lines.extend([
        ("branch", git_branch()),
        ("commit", git_commit()),
        ("status", git_status().to_string()),
        ("commit-at", commit_at().to_string()),
        ("build", build_at().to_string()),
        ("target", build::BUILD_TARGET.to_string()),
        ("profile", build_profile().to_string()),
        ("rustc", build::RUST_VERSION.to_string()),
        ("cargo", build::CARGO_VERSION.to_string()),
        ("lua", LUA_RUNTIME.to_string()),
    ]);

    let width = lines
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    for (label, value) in lines {
        println!("{label:>width$}  {value}");
    }
}

fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

fn git_branch() -> String {
    non_empty(build::BRANCH).unwrap_or("<unknown>").to_string()
}

fn git_commit() -> String {
    non_empty(build::SHORT_COMMIT)
        .or_else(|| non_empty(build::COMMIT_HASH))
        .unwrap_or("<unknown>")
        .to_string()
}

fn git_status() -> &'static str {
    if build::GIT_CLEAN { "clean" } else { "dirty" }
}

fn commit_at() -> &'static str {
    non_empty(build::COMMIT_DATE_3339)
        .or_else(|| non_empty(build::COMMIT_DATE))
        .unwrap_or("unknown")
}

fn build_at() -> &'static str {
    non_empty(build::BUILD_TIME_3339)
        .or_else(|| non_empty(build::BUILD_TIME))
        .unwrap_or("unknown")
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "unknown" {
        None
    } else {
        Some(trimmed)
    }
}

fn parsed_arg_to_string(value: &OsStr, field: &str) -> Result<String, ParseError> {
    value
        .to_str()
        .map(std::string::ToString::to_string)
        .ok_or_else(|| ParseError::top(format!("{field} contains invalid UTF-8")))
}

fn top_usage(bin: &OsStr) -> String {
    let bin = bin.to_string_lossy();
    formatdoc! { r#"
        {APP_ABOUT}

        Website: {WEBSITE_URL}

        Usage: {bin} <COMMAND>
               {bin} <FILE.lua> [--] [<script_args>...]

        Commands:
          run      Run ptool script
          repl     Start interactive Lua REPL
          version  Print version

        Options:
          -h, --help     Print help
          -V, --version  Print version
    "# }
}

fn run_usage(bin: &OsStr) -> String {
    let bin = bin.to_string_lossy();
    formatdoc! { r#"
        Run ptool script

        Usage: {bin} run <filename> [--] [<script_args>...]

        Arguments:
          <filename>      Lua script filename
          <script_args>   Arguments passed to Lua script (supports optional `--` separator)

        Options:
          -h, --help      Print help
    "# }
}

fn repl_usage(bin: &OsStr) -> String {
    let bin = bin.to_string_lossy();
    formatdoc! { r#"
        Start interactive Lua REPL

        Usage: {bin} repl

        Options:
          -h, --help  Print help
    "# }
}

fn version_usage(bin: &OsStr) -> String {
    let bin = bin.to_string_lossy();
    formatdoc! { r#"
        Print ptool version

        Usage: {bin} version

        Options:
          -h, --help  Print help
    "# }
}
