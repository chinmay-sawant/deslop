use tree_sitter::Node;

use crate::model::FunctionFingerprint;

// Intentionally kept as a small scoring module so the parser facade stays lean.

pub(super) fn function_kind(node: Node<'_>, source: &str) -> &'static str {
    let Some(parameters) = node.child_by_field_name("parameters") else {
        return "function";
    };

    let Some(parameters_text) = source.get(parameters.byte_range()) else {
        return "function";
    };

    if parameters_text.contains("self") {
        "method"
    } else {
        "function"
    }
}

pub(super) fn enclosing_impl_type(node: Node<'_>, source: &str) -> Option<String> {
    let mut parent = node.parent();

    while let Some(current) = parent {
        if current.kind() == "impl_item" {
            return current
                .child_by_field_name("type")
                .and_then(|type_node| source.get(type_node.byte_range()))
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(ToOwned::to_owned);
        }

        parent = current.parent();
    }

    None
}

pub(super) fn build_function_fingerprint(
    node: Node<'_>,
    source: &str,
    kind: &str,
    receiver_type: Option<String>,
    call_count: usize,
) -> Option<FunctionFingerprint> {
    let name_node = node.child_by_field_name("name")?;
    let body_node = node.child_by_field_name("body")?;
    let function_text = source.get(node.byte_range())?;
    let name = source.get(name_node.byte_range())?.to_string();
    let comment_lines =
        count_comment_lines(function_text) + count_doc_lines(source, node.start_position().row);
    let code_lines = count_code_lines(function_text);
    let complexity_score = 1 + count_control_nodes(body_node);
    let symmetry_score = compute_symmetry_score(body_node);
    let line_count = node.end_position().row - node.start_position().row + 1;
    let comment_to_code_ratio = if code_lines == 0 {
        0.0
    } else {
        comment_lines as f64 / code_lines as f64
    };

    Some(FunctionFingerprint {
        name,
        kind: kind.to_string(),
        receiver_type,
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        line_count,
        comment_lines,
        code_lines,
        comment_to_code_ratio,
        complexity_score,
        symmetry_score,
        boilerplate_err_guards: 0,
        contains_any_type: false,
        contains_empty_interface: false,
        type_assertion_count: 0,
        call_count,
    })
}

fn count_control_nodes(node: Node<'_>) -> usize {
    let mut total = 0;
    let mut cursor = node.walk();

    for child in node.named_children(&mut cursor) {
        total += usize::from(is_control_node(child.kind()));
        total += count_control_nodes(child);
    }

    total
}

fn is_control_node(kind: &str) -> bool {
    matches!(
        kind,
        "if_expression"
            | "for_expression"
            | "while_expression"
            | "loop_expression"
            | "match_expression"
    )
}

fn compute_symmetry_score(body_node: Node<'_>) -> f64 {
    let mut cursor = body_node.walk();
    let mut statement_kinds = Vec::new();

    for child in body_node.named_children(&mut cursor) {
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

    for line in text.split('\n') {
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

    for line in text.split('\n') {
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

fn count_doc_lines(source: &str, function_start_row: usize) -> usize {
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

        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            count += 1;
            continue;
        }

        if trimmed.ends_with("*/") {
            count += 1;
            while index > 0 {
                index -= 1;
                count += 1;
                let block_line = lines[index].trim();
                if block_line.starts_with("/**") || block_line.starts_with("/*!") {
                    return count;
                }
            }
            return count;
        }

        break;
    }

    count
}
