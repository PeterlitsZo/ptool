use mlua::{Lua, Value};
use ptool_engine::PtoolEngine;

pub(crate) fn render(
    lua: &Lua,
    engine: &PtoolEngine,
    template: String,
    context: Value,
) -> mlua::Result<String> {
    let data = context_to_json(lua, context)?;
    engine
        .template_render(&template, &data)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.template.render"))
}

fn context_to_json(lua: &Lua, context: Value) -> mlua::Result<serde_json::Value> {
    crate::json::lua_value_to_json(lua, context, "ptool.template.render invalid context")
}
