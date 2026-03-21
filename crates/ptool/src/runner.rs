use mlua::Lua;

pub fn run_script(
    filename: &str,
    script_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let script = std::fs::read_to_string(filename)?;
    let script = strip_shebang(&script);
    let lua = Lua::new();
    crate::lua_api::install_ptool_module(&lua, filename, script_args)?;
    lua.load(script).set_name(filename).exec()?;
    Ok(())
}

fn strip_shebang(script: &str) -> &str {
    if !script.starts_with("#!") {
        return script;
    }

    match script.find('\n') {
        Some(newline_idx) => &script[newline_idx + 1..],
        None => "",
    }
}
