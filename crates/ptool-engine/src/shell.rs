use crate::{Error, ErrorKind, Result};

pub fn split(input: &str) -> Result<Vec<String>> {
    shlex::split(input).ok_or_else(|| {
        Error::new(ErrorKind::InvalidArgs, "failed to parse shell words")
            .with_op("ptool.sh.split")
            .with_input(input.to_string())
    })
}

pub fn quote(input: &str) -> Result<String> {
    shlex::try_quote(input)
        .map(|quoted| quoted.into_owned())
        .map_err(|err| {
            Error::new(
                ErrorKind::InvalidArgs,
                format!("failed to quote shell word: {err}"),
            )
            .with_op("ptool.sh.quote")
            .with_input(input.to_string())
        })
}

pub fn join(words: &[String]) -> Result<String> {
    shlex::try_join(words.iter().map(String::as_str)).map_err(|err| {
        Error::new(
            ErrorKind::InvalidArgs,
            format!("failed to join shell words: {err}"),
        )
        .with_op("ptool.sh.join")
    })
}
