use minijinja::{Environment, UndefinedBehavior};
use mlua::{Lua, LuaSerdeExt, Value};
use serde_json::Value as JsonValue;

const TEMPLATE_NAME: &str = "__ptool_inline_template__";

pub(crate) fn render(lua: &Lua, template: String, context: Value) -> mlua::Result<String> {
    let data = context_to_json(lua, context)?;

    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Chainable);
    env.add_template(TEMPLATE_NAME, &template)
        .map_err(|err| mlua::Error::runtime(format!("ptool.template.render failed: {err}")))?;
    let tpl = env
        .get_template(TEMPLATE_NAME)
        .map_err(|err| mlua::Error::runtime(format!("ptool.template.render failed: {err}")))?;
    tpl.render(&data)
        .map_err(|err| mlua::Error::runtime(format!("ptool.template.render failed: {err}")))
}

fn context_to_json(lua: &Lua, context: Value) -> mlua::Result<JsonValue> {
    lua.from_value(context).map_err(|err| {
        mlua::Error::runtime(format!("ptool.template.render invalid context: {err}"))
    })
}
