use crate::{Error, ErrorKind, JsonValue, Result, fs};
use minijinja::{Environment, UndefinedBehavior};

const TEMPLATE_NAME: &str = "__ptool_inline_template__";

pub(crate) fn render(template: &str, context: &JsonValue) -> Result<String> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Chainable);
    env.add_template(TEMPLATE_NAME, template)
        .map_err(template_error)?;
    let tpl = env.get_template(TEMPLATE_NAME).map_err(template_error)?;
    tpl.render(context).map_err(template_error)
}

pub(crate) fn write(path: &str, template: &str, context: &JsonValue) -> Result<()> {
    let rendered = render(template, context).map_err(|err| err.with_op("ptool.template.write"))?;
    fs::write(path, rendered.as_bytes()).map_err(|err| err.with_op("ptool.template.write"))
}

fn template_error(err: impl std::fmt::Display) -> Error {
    Error::new(
        ErrorKind::InvalidArgs,
        format!("template render failed: {err}"),
    )
    .with_op("ptool.template.render")
}
