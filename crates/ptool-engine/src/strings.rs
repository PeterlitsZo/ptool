use crate::{Error, ErrorKind, Result};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SplitOptions {
    pub trim: bool,
    pub skip_empty: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SplitLinesOptions {
    pub keep_ending: bool,
    pub skip_empty: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IndentOptions {
    pub skip_first: bool,
}

pub(crate) fn trim(input: &str) -> String {
    input.trim().to_string()
}

pub(crate) fn trim_start(input: &str) -> String {
    input.trim_start().to_string()
}

pub(crate) fn trim_end(input: &str) -> String {
    input.trim_end().to_string()
}

pub(crate) fn is_blank(input: &str) -> bool {
    input.trim().is_empty()
}

pub(crate) fn starts_with(input: &str, prefix: &str) -> bool {
    input.starts_with(prefix)
}

pub(crate) fn ends_with(input: &str, suffix: &str) -> bool {
    input.ends_with(suffix)
}

pub(crate) fn contains(input: &str, needle: &str) -> bool {
    input.contains(needle)
}

pub(crate) fn split(input: &str, separator: &str, options: SplitOptions) -> Result<Vec<String>> {
    ensure_non_empty(separator, "ptool.str.split", "sep")?;

    Ok(input
        .split(separator)
        .filter_map(|part| normalize_split_part(part, options.trim, options.skip_empty))
        .collect())
}

pub(crate) fn split_lines(input: &str, options: SplitLinesOptions) -> Vec<String> {
    collect_lines(input, options.keep_ending)
        .into_iter()
        .filter(|line| !options.skip_empty || !line_without_line_ending(line).is_empty())
        .collect()
}

pub(crate) fn join(parts: &[String], separator: &str) -> String {
    parts.join(separator)
}

pub(crate) fn replace(input: &str, from: &str, to: &str, limit: Option<usize>) -> Result<String> {
    ensure_non_empty(from, "ptool.str.replace", "from")?;

    Ok(match limit {
        Some(limit) => input.replacen(from, to, limit),
        None => input.replace(from, to),
    })
}

pub(crate) fn repeat(input: &str, count: i64) -> Result<String> {
    if count < 0 {
        return Err(
            Error::new(ErrorKind::InvalidArgs, "`n` must be >= 0").with_op("ptool.str.repeat")
        );
    }
    let count = usize::try_from(count).map_err(|_| {
        Error::new(ErrorKind::InvalidArgs, "`n` is too large").with_op("ptool.str.repeat")
    })?;
    Ok(input.repeat(count))
}

pub(crate) fn cut_prefix(input: &str, prefix: &str) -> String {
    match input.strip_prefix(prefix) {
        Some(stripped) => stripped.to_string(),
        None => input.to_string(),
    }
}

pub(crate) fn cut_suffix(input: &str, suffix: &str) -> String {
    match input.strip_suffix(suffix) {
        Some(stripped) => stripped.to_string(),
        None => input.to_string(),
    }
}

pub(crate) fn indent(input: &str, prefix: &str, options: IndentOptions) -> String {
    if input.is_empty() {
        return input.to_string();
    }

    let mut rendered = String::with_capacity(input.len() + prefix.len());
    for (index, line) in collect_lines(input, true).into_iter().enumerate() {
        if !(index == 0 && options.skip_first) {
            rendered.push_str(prefix);
        }
        rendered.push_str(&line);
    }
    rendered
}

fn ensure_non_empty(input: &str, op: &str, arg_name: &str) -> Result<()> {
    if input.is_empty() {
        return Err(Error::new(
            ErrorKind::EmptyInput,
            format!("`{arg_name}` does not accept empty string"),
        )
        .with_op(op));
    }
    Ok(())
}

fn normalize_split_part(part: &str, trim: bool, skip_empty: bool) -> Option<String> {
    let normalized = if trim { part.trim() } else { part };
    if skip_empty && normalized.is_empty() {
        return None;
    }
    Some(normalized.to_string())
}

fn collect_lines(input: &str, keep_ending: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let bytes = input.as_bytes();
    let mut start = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                let end = if keep_ending {
                    index + 1
                } else if index > start && bytes[index - 1] == b'\r' {
                    index - 1
                } else {
                    index
                };
                let slice_start = start.min(end);
                lines.push(input[slice_start..end].to_string());
                start = index + 1;
            }
            b'\r' if bytes.get(index + 1) != Some(&b'\n') => {
                let end = if keep_ending { index + 1 } else { index };
                let slice_start = start.min(end);
                lines.push(input[slice_start..end].to_string());
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }

    if start < input.len() {
        lines.push(input[start..].to_string());
    }

    lines
}

fn line_without_line_ending(line: &str) -> &str {
    if let Some(line) = line.strip_suffix("\r\n") {
        return line;
    }
    if let Some(line) = line.strip_suffix('\n') {
        return line;
    }
    if let Some(line) = line.strip_suffix('\r') {
        return line;
    }
    line
}
