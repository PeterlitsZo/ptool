use std::cell::RefCell;
use std::rc::Rc;

use mlua::Lua;

pub fn run_script(
    filename: &str,
    script_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let script = std::fs::read_to_string(filename)?;
    let script = strip_shebang_preserve_lines(script);
    let lua = Lua::new();
    let world = Rc::new(RefCell::new(crate::LuaWorld::new()?));
    crate::lua_api::install_ptool_module(&lua, world, filename, script_args)?;
    lua.load(&script).set_name(filename).exec()?;
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
