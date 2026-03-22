use tree_sitter::Node;

use crate::model::FunctionFingerprint;

pub(super) fn build_function_fingerprint(
    node: Node<'_>,
    source: &str,
    receiver_type: Option<String>,
    type_assertion_count: usize,
    call_count: usize,
) -> Option<FunctionFingerprint> {
    let name_node = node.child_by_field_name("name")?;
    let body_node = node.child_by_field_name("body")?;
    let function_text = source.get(node.byte_range())?;
    let signature_text = source.get(node.start_byte()..body_node.start_byte())?;

    let name = source.get(name_node.byte_range())?.to_string();
    let kind = match node.kind() {
        "method_declaration" => "method",
        _ => "function",
    }
    .to_string();

    let comment_lines = count_comment_lines(function_text)
        + count_leading_doc_comment_lines(source, node.start_position().row);
    let code_lines = count_code_lines(function_text);
    let boilerplate_err_guards = count_boilerplate_err_guards(function_text);
    let complexity_raw = count_control_nodes(body_node);
    let complexity_score = 1 + complexity_raw.saturating_sub(boilerplate_err_guards);
    let symmetry_score = compute_symmetry_score(body_node);
    let line_count = node.end_position().row - node.start_position().row + 1;
    let comment_to_code_ratio = if code_lines == 0 {
        0.0
    } else {
        comment_lines as f64 / code_lines as f64
    };

    Some(FunctionFingerprint {
        name,
        kind,
        receiver_type,
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        line_count,
        comment_lines,
        code_lines,
        comment_to_code_ratio,
        complexity_score,
        symmetry_score,
        boilerplate_err_guards,
        contains_any_type: contains_token(signature_text, "any"),
        contains_empty_interface: signature_text.contains("interface{}"),
        type_assertion_count,
        call_count,
    })
}

fn count_control_nodes(node: Node<'_>) -> usize {
    let mut total = 0;
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        total += usize::from(is_control_node(child.kind()));
        total += count_control_nodes(child);
    }

    total
}

fn is_control_node(kind: &str) -> bool {
    matches!(
        kind,
        "if_statement"
            | "for_statement"
            | "expression_switch_statement"
            | "type_switch_statement"
            | "select_statement"
            | "communication_case"
            | "fallthrough_statement"
            | "defer_statement"
            | "go_statement"
    )
}

fn compute_symmetry_score(body_node: Node<'_>) -> f64 {
    let mut cursor = body_node.walk();
    let mut statement_kinds = Vec::new();

    for child in body_node.named_children(&mut cursor) {
        if child.kind() == "{" || child.kind() == "}" {
            continue;
        }
        statement_kinds.push(child.kind().to_string());
    }

    if statement_kinds.len() < 2 {
        return 0.0;
    }

    statement_kinds.sort();

    let mut best_run = 1usize;
    let mut current_run = 1usize;

    for pair in statement_kinds.windows(2) {
        if pair[0] == pair[1] {
            current_run += 1;
            best_run = best_run.max(current_run);
        } else {
            current_run = 1;
        }
    }

    best_run as f64 / statement_kinds.len() as f64
}

fn count_comment_lines(text: &str) -> usize {
    let mut count = 0usize;
    let mut in_block_comment = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if in_block_comment {
            count += 1;
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("//") {
            count += 1;
            continue;
        }

        if trimmed.starts_with("/*") {
            count += 1;
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
        }
    }

    count
}

fn count_code_lines(text: &str) -> usize {
    let mut count = 0usize;
    let mut in_block_comment = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
                let suffix = trimmed
                    .split_once("*/")
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or("");
                if !suffix.is_empty() {
                    count += 1;
                }
            }
            continue;
        }

        if trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("/*") {
            if trimmed.contains("*/") {
                let suffix = trimmed
                    .split_once("*/")
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or("");
                if !suffix.is_empty() {
                    count += 1;
                }
            } else {
                in_block_comment = true;
            }
            continue;
        }

        count += 1;
    }

    count
}

fn count_boilerplate_err_guards(text: &str) -> usize {
    let relevant_lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect::<Vec<_>>();

    let mut count = 0usize;
    let mut index = 0usize;

    while index < relevant_lines.len() {
        let line = relevant_lines[index];

        if line.starts_with("if ") && line.contains(" != nil {") {
            if line.contains(" return ") {
                count += 1;
                index += 1;
                continue;
            }

            if let Some(next_line) = relevant_lines.get(index + 1)
                && next_line.starts_with("return ")
            {
                count += 1;
                index += 2;
                continue;
            }
        }

        index += 1;
    }

    count
}

fn count_leading_doc_comment_lines(source: &str, function_start_row: usize) -> usize {
    let lines = source.lines().collect::<Vec<_>>();
    if function_start_row == 0 || function_start_row > lines.len() {
        return 0;
    }

    let mut count = 0usize;
    let mut index = function_start_row;

    while index > 0 {
        index -= 1;
        let trimmed = lines[index].trim();

        if trimmed.is_empty() {
            break;
        }

        if trimmed.starts_with("//") {
            count += 1;
            continue;
        }

        if trimmed.ends_with("*/") {
            count += 1;
            while index > 0 {
                index -= 1;
                count += 1;
                if lines[index].trim().starts_with("/*") {
                    return count;
                }
            }
            return count;
        }

        break;
    }

    count
}

fn contains_token(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let needle_bytes = needle.as_bytes();

    if needle_bytes.is_empty() || needle_bytes.len() > bytes.len() {
        return false;
    }

    for start in 0..=bytes.len() - needle_bytes.len() {
        if &bytes[start..start + needle_bytes.len()] != needle_bytes {
            continue;
        }

        let left_ok = start == 0 || !is_identifier_byte(bytes[start - 1]);
        let right_index = start + needle_bytes.len();
        let right_ok = right_index == bytes.len() || !is_identifier_byte(bytes[right_index]);

        if left_ok && right_ok {
            return true;
        }
    }

    false
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
mod tests {
    use super::{
        contains_token, count_boilerplate_err_guards, count_code_lines, count_comment_lines,
        count_leading_doc_comment_lines,
    };

    #[test]
    fn counts_comments_and_code_lines() {
        let sample = "func demo() {\n// note\n/* block\nmore */\nvalue := 1\n}\n";
        assert_eq!(count_comment_lines(sample), 3);
        assert_eq!(count_code_lines(sample), 3);
    }

    #[test]
    fn counts_canonical_err_guards() {
        let sample = "func demo() error {\nif err != nil {\nreturn err\n}\nif another != nil { return another }\nreturn nil\n}\n";
        assert_eq!(count_boilerplate_err_guards(sample), 2);
    }

    #[test]
    fn counts_leading_doc_comments() {
        let sample = "// Add joins numbers\n// for reporting\nfunc Add(a int, b int) int {\nreturn a + b\n}\n";
        assert_eq!(count_leading_doc_comment_lines(sample, 2), 2);
    }

    #[test]
    fn matches_any_as_a_token() {
        assert!(contains_token("func Run(value any) any", "any"));
        assert!(!contains_token("func Run(many int)", "any"));
    }
}
