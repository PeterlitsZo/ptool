pub(crate) fn unindent(input: &str) -> String {
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
