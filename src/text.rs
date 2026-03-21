pub(crate) fn unindent_text(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut start = 0;
    let mut end = lines.len();

    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }
    while start < end && lines[end - 1].trim().is_empty() {
        end -= 1;
    }

    let mut normalized = Vec::with_capacity(end.saturating_sub(start));
    for line in &lines[start..end] {
        let trimmed = line.trim_start();
        let line = match trimmed.strip_prefix("| ") {
            Some(rest) => rest,
            None => line,
        };
        normalized.push(line);
    }

    normalized.join("\n")
}

#[cfg(test)]
mod tests {
    use super::unindent_text;

    #[test]
    fn unindent_pipe_prefixed_lines() {
        let input = "    | line 1\n    | line 2\n";
        let output = unindent_text(input);
        assert_eq!(output, "line 1\nline 2");
    }

    #[test]
    fn unindent_drops_surrounding_blank_lines() {
        let input = "\n    | a\n    | b\n\n";
        let output = unindent_text(input);
        assert_eq!(output, "a\nb");
    }
}
