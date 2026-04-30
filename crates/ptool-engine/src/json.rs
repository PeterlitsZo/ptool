use crate::{Error, ErrorKind, Result};

pub type JsonValue = serde_json::Value;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JsonStringifyOptions {
    pub pretty: bool,
}

pub(crate) fn parse(input: &str) -> Result<JsonValue> {
    serde_json::from_str(input).map_err(|err| {
        Error::new(ErrorKind::InvalidArgs, format!("json parse failed: {err}"))
            .with_op("ptool.json.parse")
            .with_input(input.to_string())
    })
}

pub(crate) fn stringify(value: &JsonValue, options: JsonStringifyOptions) -> Result<String> {
    let result = if options.pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    };

    result.map_err(|err| {
        Error::new(
            ErrorKind::InvalidArgs,
            format!("json stringify failed: {err}"),
        )
        .with_op("ptool.json.stringify")
    })
}
