pub(super) fn extract_doc_comment(source: &str, function_start_row: usize) -> Option<String> {
    let lines = source.lines().collect::<Vec<_>>();
    if function_start_row == 0 || function_start_row > lines.len() {
        return None;
    }

    let mut index = function_start_row;
    let mut comment_lines = Vec::new();

    while index > 0 {
        index -= 1;
        let trimmed = lines[index].trim();

        if trimmed.is_empty() {
            break;
        }

        if trimmed.starts_with("//") {
            comment_lines.push(trimmed.trim_start_matches("//").trim().to_string());
            continue;
        }

        if trimmed.ends_with("*/") {
            let mut block_lines = vec![trimmed.to_string()];
            while index > 0 {
                index -= 1;
                block_lines.push(lines[index].trim().to_string());
                if lines[index].trim().starts_with("/*") {
                    break;
                }
            }
            block_lines.reverse();
            let normalized = block_lines
                .into_iter()
                .map(|line| {
                    line.trim_start_matches("/*")
                        .trim_end_matches("*/")
                        .trim_start_matches('*')
                        .trim()
                        .to_string()
                })
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>();
            if normalized.is_empty() {
                return None;
            }
            return Some(normalized.join("\n"));
        }

        break;
    }

    if comment_lines.is_empty() {
        None
    } else {
        comment_lines.reverse();
        Some(comment_lines.join("\n"))
    }
}
