use mlua::{Lua, Value};
use ptool_engine::PtoolEngine;

const RENDER_SIGNATURE: &str = "ptool.template.render(template, context)";
const WRITE_SIGNATURE: &str = "ptool.template.write(path, template, context)";

pub(crate) fn render(
    lua: &Lua,
    engine: &PtoolEngine,
    template: String,
    context: Value,
) -> mlua::Result<String> {
    let data = context_to_json(lua, context, RENDER_SIGNATURE)?;
    engine
        .template_render(&template, &data)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, RENDER_SIGNATURE))
}

pub(crate) fn write(
    lua: &Lua,
    engine: &PtoolEngine,
    path: String,
    template: String,
    context: Value,
) -> mlua::Result<()> {
    let data = context_to_json(lua, context, WRITE_SIGNATURE)?;
    engine
        .template_write(&path, &template, &data)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, WRITE_SIGNATURE))
}

fn context_to_json(lua: &Lua, context: Value, signature: &str) -> mlua::Result<serde_json::Value> {
    crate::json::lua_value_to_json(lua, context, &format!("{signature} invalid context"))
}
