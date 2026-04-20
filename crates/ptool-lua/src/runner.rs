use std::cell::RefCell;
use std::io::{self, IsTerminal, Write};
use std::rc::Rc;

use mlua::{Error as LuaError, Lua, MultiValue};
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

        match lua
            .load(&chunk)
            .set_name(REPL_SCRIPT_NAME)
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
                writeln!(stdout, "error: {err}")?;
                chunk.clear();
                prompt = REPL_PROMPT;
            }
        }
    }

    Ok(())
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
