use crate::{Error, ErrorKind, Result};
use regex::{Regex, RegexBuilder};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegexOptions {
    pub case_insensitive: bool,
}

#[derive(Clone, Debug)]
pub struct RegexPattern {
    regex: Regex,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegexMatch {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegexCaptures {
    pub full: String,
    pub groups: Vec<Option<String>>,
    pub named: Vec<(String, String)>,
}

pub(crate) fn compile(pattern: &str, options: RegexOptions) -> Result<RegexPattern> {
    RegexBuilder::new(pattern)
        .case_insensitive(options.case_insensitive)
        .build()
        .map(|regex| RegexPattern { regex })
        .map_err(|err| {
            Error::new(
                ErrorKind::InvalidArgs,
                format!("regex compile failed: {err}"),
            )
            .with_op("ptool.re.compile")
        })
}

pub(crate) fn escape(text: &str) -> String {
    regex::escape(text)
}

impl RegexPattern {
    pub fn is_match(&self, input: &str) -> bool {
        self.regex.is_match(input)
    }

    pub fn find(&self, input: &str, start: usize) -> Result<Option<RegexMatch>> {
        if start > input.len() {
            return Err(
                Error::new(ErrorKind::InvalidArgs, "start exceeds input length")
                    .with_op("ptool.re.Regex:find"),
            );
        }

        Ok(self
            .regex
            .find_at(input, start)
            .map(|matched| RegexMatch::from_match(input, matched)))
    }

    pub fn find_all(&self, input: &str) -> Vec<RegexMatch> {
        self.regex
            .find_iter(input)
            .map(|matched| RegexMatch::from_match(input, matched))
            .collect()
    }

    pub fn captures(&self, input: &str) -> Option<RegexCaptures> {
        self.regex
            .captures(input)
            .map(|captures| RegexCaptures::from_captures(&self.regex, input, captures))
    }

    pub fn captures_all(&self, input: &str) -> Vec<RegexCaptures> {
        self.regex
            .captures_iter(input)
            .map(|captures| RegexCaptures::from_captures(&self.regex, input, captures))
            .collect()
    }

    pub fn replace(&self, input: &str, replacement: &str) -> String {
        self.regex.replacen(input, 1, replacement).into_owned()
    }

    pub fn replace_all(&self, input: &str, replacement: &str) -> String {
        self.regex.replace_all(input, replacement).into_owned()
    }

    pub fn split(&self, input: &str, limit: Option<usize>) -> Vec<String> {
        match limit {
            None => self
                .regex
                .split(input)
                .map(std::string::ToString::to_string)
                .collect(),
            Some(limit) => self
                .regex
                .splitn(input, limit)
                .map(std::string::ToString::to_string)
                .collect(),
        }
    }
}

impl RegexMatch {
    fn from_match(input: &str, matched: regex::Match<'_>) -> Self {
        Self {
            start: matched.start(),
            end: matched.end(),
            text: input[matched.start()..matched.end()].to_string(),
        }
    }
}

impl RegexCaptures {
    fn from_captures(regex: &Regex, input: &str, captures: regex::Captures<'_>) -> Self {
        let full = captures
            .get(0)
            .map(|matched| input[matched.start()..matched.end()].to_string())
            .unwrap_or_default();

        let mut groups = Vec::with_capacity(captures.len().saturating_sub(1));
        for group_index in 1..captures.len() {
            groups.push(
                captures
                    .get(group_index)
                    .map(|group| input[group.start()..group.end()].to_string()),
            );
        }

        let mut named = Vec::new();
        for (group_index, name) in regex.capture_names().enumerate().skip(1) {
            let Some(name) = name else {
                continue;
            };
            if let Some(group) = captures.get(group_index) {
                named.push((
                    name.to_string(),
                    input[group.start()..group.end()].to_string(),
                ));
            }
        }

        Self {
            full,
            groups,
            named,
        }
    }
}
