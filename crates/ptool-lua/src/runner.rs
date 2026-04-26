use std::cell::RefCell;
use std::io::{self, IsTerminal, Write};
use std::ops::Range;
use std::rc::Rc;

use mlua::{Error as LuaError, Lua, MultiValue, Table};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

const REPL_SCRIPT_NAME: &str = "=(ptool repl)";
const REPL_PROMPT: &str = ">>> ";
const REPL_CONTINUATION_PROMPT: &str = "... ";

type SharedLuaWorld = Rc<RefCell<crate::LuaWorld>>;
type LuaRuntime = (Lua, SharedLuaWorld);

pub fn run_script(
    filename: &str,
    script_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let script = std::fs::read_to_string(filename)?;
    let script = strip_shebang_preserve_lines(script);
    let (lua, _) = create_lua_runtime(filename, script_args)?;
    lua.load(&script).set_name(filename).exec()?;
    Ok(())
}

pub fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::other("ptool repl requires an interactive TTY").into());
    }

    let (lua, world) = create_lua_runtime(REPL_SCRIPT_NAME, &[])?;
    let repl_env = create_repl_environment(&lua)?;
    let mut editor = DefaultEditor::new()?;
    let mut stdout = io::stdout().lock();
    let mut chunk = String::new();
    let mut prompt = REPL_PROMPT;

    writeln!(stdout, "ptool repl ({})", env!("CARGO_PKG_VERSION"))?;
    writeln!(stdout, "Press Ctrl-D to exit.")?;

    loop {
        let line = match editor.readline(prompt) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                chunk.clear();
                prompt = REPL_PROMPT;
                continue;
            }
            Err(ReadlineError::Eof) => {
                writeln!(stdout)?;
                break;
            }
            Err(err) => return Err(err.into()),
        };

        if !line.is_empty() {
            editor.add_history_entry(line.as_str())?;
        }

        if !chunk.is_empty() {
            chunk.push('\n');
        }
        chunk.push_str(&line);

        let prepared_chunk = preprocess_repl_chunk(&chunk);

        match lua
            .load(&prepared_chunk)
            .set_name(REPL_SCRIPT_NAME)
            .set_environment(repl_env.clone())
            .eval::<MultiValue>()
        {
            Ok(values) => {
                print_repl_values(&mut stdout, &world, values)?;
                chunk.clear();
                prompt = REPL_PROMPT;
            }
            Err(LuaError::SyntaxError {
                incomplete_input: true,
                ..
            }) => {
                prompt = REPL_CONTINUATION_PROMPT;
            }
            Err(err) => {
                writeln!(
                    stdout,
                    "error: {}",
                    crate::lua_error::render_error_report(&err)
                )?;
                chunk.clear();
                prompt = REPL_PROMPT;
            }
        }
    }

    Ok(())
}

fn create_repl_environment(lua: &Lua) -> mlua::Result<Table> {
    let env = lua.create_table()?;
    let mt = lua.create_table()?;
    mt.set("__index", lua.globals())?;
    env.set_metatable(Some(mt))?;
    env.set("_G", env.clone())?;
    Ok(env)
}

fn preprocess_repl_chunk(chunk: &str) -> String {
    let local_ranges = find_top_level_local_ranges(chunk);
    if local_ranges.is_empty() {
        return chunk.to_string();
    }

    let mut rewritten = chunk.to_string();
    for range in local_ranges.into_iter().rev() {
        rewritten.replace_range(range, "     ");
    }
    rewritten
}

fn find_top_level_local_ranges(chunk: &str) -> Vec<Range<usize>> {
    #[derive(Clone, Copy)]
    enum ScanState {
        Normal,
        ShortString(u8),
        LongString(usize),
        LineComment,
        LongComment(usize),
    }

    let bytes = chunk.as_bytes();
    let mut ranges = Vec::new();
    let mut depth = 0usize;
    let mut idx = 0usize;
    let mut state = ScanState::Normal;

    while idx < bytes.len() {
        match state {
            ScanState::Normal => {
                if bytes[idx] == b'-' && bytes.get(idx + 1) == Some(&b'-') {
                    idx += 2;
                    if let Some(equals) = long_bracket_open_equals(bytes, idx) {
                        idx += equals + 2;
                        state = ScanState::LongComment(equals);
                    } else {
                        state = ScanState::LineComment;
                    }
                    continue;
                }

                if let Some(equals) = long_bracket_open_equals(bytes, idx) {
                    idx += equals + 2;
                    state = ScanState::LongString(equals);
                    continue;
                }

                if matches!(bytes[idx], b'\'' | b'"') {
                    state = ScanState::ShortString(bytes[idx]);
                    idx += 1;
                    continue;
                }

                if is_identifier_start(bytes[idx]) {
                    let start = idx;
                    idx += 1;
                    while idx < bytes.len() && is_identifier_continue(bytes[idx]) {
                        idx += 1;
                    }

                    let token = &chunk[start..idx];
                    match token {
                        "local" if depth == 0 => ranges.push(start..idx),
                        "function" | "do" | "then" | "repeat" => depth += 1,
                        "end" | "until" => depth = depth.saturating_sub(1),
                        _ => {}
                    }
                    continue;
                }

                idx += 1;
            }
            ScanState::ShortString(quote) => {
                if bytes[idx] == b'\\' {
                    idx += 1;
                    if idx < bytes.len() {
                        idx += 1;
                    }
                    continue;
                }

                if bytes[idx] == quote {
                    state = ScanState::Normal;
                }
                idx += 1;
            }
            ScanState::LongString(equals) => {
                if let Some(close_len) = long_bracket_close_len(bytes, idx, equals) {
                    idx += close_len;
                    state = ScanState::Normal;
                } else {
                    idx += 1;
                }
            }
            ScanState::LineComment => {
                if bytes[idx] == b'\n' {
                    state = ScanState::Normal;
                }
                idx += 1;
            }
            ScanState::LongComment(equals) => {
                if let Some(close_len) = long_bracket_close_len(bytes, idx, equals) {
                    idx += close_len;
                    state = ScanState::Normal;
                } else {
                    idx += 1;
                }
            }
        }
    }

    ranges
}

fn long_bracket_open_equals(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'[') {
        return None;
    }

    let mut idx = start + 1;
    while bytes.get(idx) == Some(&b'=') {
        idx += 1;
    }

    (bytes.get(idx) == Some(&b'[')).then_some(idx - start - 1)
}

fn long_bracket_close_len(bytes: &[u8], start: usize, equals: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b']') {
        return None;
    }

    let mut idx = start + 1;
    for _ in 0..equals {
        if bytes.get(idx) != Some(&b'=') {
            return None;
        }
        idx += 1;
    }

    (bytes.get(idx) == Some(&b']')).then_some(idx - start + 1)
}

fn is_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

fn create_lua_runtime(
    script_name: &str,
    script_args: &[String],
) -> Result<LuaRuntime, Box<dyn std::error::Error>> {
    let lua = Lua::new();
    let world = Rc::new(RefCell::new(crate::LuaWorld::new()?));
    crate::lua_api::install_ptool_module(&lua, Rc::clone(&world), script_name, script_args)?;
    Ok((lua, world))
}

fn print_repl_values(
    stdout: &mut impl Write,
    world: &SharedLuaWorld,
    values: MultiValue,
) -> Result<(), Box<dyn std::error::Error>> {
    if values.is_empty() {
        return Ok(());
    }

    let rendered = {
        let world = world.borrow();
        values
            .iter()
            .map(|value| world.inspect(value.clone(), None))
            .collect::<mlua::Result<Vec<_>>>()?
    };
    writeln!(stdout, "{}", rendered.join("\t"))?;
    Ok(())
}

fn strip_shebang_preserve_lines(script: String) -> String {
    if !script.starts_with("#!") {
        return script;
    }

    match script.find('\n') {
        Some(newline_idx) => format!("\n{}", &script[newline_idx + 1..]),
        None => "\n".to_string(),
    }
}
