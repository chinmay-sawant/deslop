use tree_sitter::Node;

use crate::analysis::NamedLiteral;

pub(super) fn extract_doc_comment(source: &str, function_start_row: usize) -> Option<String> {
    let lines = source.lines().collect::<Vec<_>>();
    if function_start_row == 0 || function_start_row > lines.len() {
        return None;
    }

    let mut collected = Vec::new();
    let mut index = function_start_row;

    while index > 0 {
        index -= 1;
        let trimmed = lines[index].trim();

        if trimmed.is_empty() {
            break;
        }

        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            collected.push(
                trimmed
                    .trim_start_matches('/')
                    .trim_start_matches('!')
                    .trim()
                    .to_string(),
            );
            continue;
        }

        if trimmed.ends_with("*/") {
            let mut block_lines = vec![trimmed.trim_end_matches("*/").trim().to_string()];
            while index > 0 {
                index -= 1;
                let block_line = lines[index].trim();
                block_lines.push(block_line.trim_start_matches("/*").trim().to_string());
                if block_line.starts_with("/**") || block_line.starts_with("/*!") {
                    block_lines.reverse();
                    collected.extend(block_lines);
                    return Some(collected.join("\n").trim().to_string());
                }
            }
            break;
        }

        break;
    }

    if collected.is_empty() {
        None
    } else {
        collected.reverse();
        Some(collected.join("\n").trim().to_string())
    }
}

pub(super) fn collect_local_strings(node: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_local_strings(node, source, &mut literals);
    literals
}

fn visit_local_strings(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if node.kind() == "let_declaration"
        && let Some(pattern_node) = node.child_by_field_name("pattern")
        && pattern_node.kind() == "identifier"
        && let Some(name) = source.get(pattern_node.byte_range())
        && let Some(value_node) = node.child_by_field_name("value")
        && let Some(value) = string_literal_value(value_node, source)
    {
        literals.push(NamedLiteral {
            line: node.start_position().row + 1,
            name: name.trim().to_string(),
            value,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_local_strings(child, source, literals);
    }
}

pub(super) fn collect_local_bindings(function_node: Node<'_>, source: &str) -> Vec<String> {
    let mut names = Vec::new();

    if let Some(parameters) = function_node.child_by_field_name("parameters") {
        collect_param_bindings(parameters, source, &mut names);
    }

    if let Some(body_node) = function_node.child_by_field_name("body") {
        visit_local_bindings(body_node, source, &mut names);
    }

    names.sort();
    names.dedup();
    names
}

fn collect_param_bindings(node: Node<'_>, source: &str, names: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "parameter" => {
                if let Some(pattern) = child.child_by_field_name("pattern") {
                    collect_ident_patterns(pattern, source, names);
                }
            }
            "self_parameter" => names.push("self".to_string()),
            _ => {}
        }
    }
}

fn visit_local_bindings(node: Node<'_>, source: &str, names: &mut Vec<String>) {
    if node.kind() == "let_declaration"
        && let Some(pattern_node) = node.child_by_field_name("pattern")
    {
        collect_ident_patterns(pattern_node, source, names);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_local_bindings(child, source, names);
    }
}

fn collect_ident_patterns(node: Node<'_>, source: &str, names: &mut Vec<String>) {
    if matches!(node.kind(), "identifier" | "self")
        && let Some(name) = source.get(node.byte_range())
    {
        names.push(name.trim().to_string());
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_ident_patterns(child, source, names);
    }
}

pub(super) fn collect_safety_comments(source: &str, function_node: Node<'_>) -> Vec<usize> {
    let lines = source.lines().collect::<Vec<_>>();
    let start = function_node.start_position().row.saturating_sub(2);
    let end = function_node
        .end_position()
        .row
        .min(lines.len().saturating_sub(1));
    let mut safety_lines = Vec::new();

    for (index, line) in lines.iter().enumerate().take(end + 1).skip(start) {
        if line.contains("SAFETY:") {
            safety_lines.push(index + 1);
        }
    }

    safety_lines
}

pub(super) fn function_is_test_only(node: Node<'_>, source: &str, is_test_file: bool) -> bool {
    if is_test_file {
        return true;
    }

    let mut current = Some(node);
    while let Some(candidate) = current {
        if leading_attributes(candidate)
            .into_iter()
            .any(|attribute| attribute_marks_test(attribute, source))
        {
            return true;
        }

        current = candidate.parent();
    }

    false
}

pub(crate) fn leading_attributes(node: Node<'_>) -> Vec<Node<'_>> {
    let mut attributes = Vec::new();
    let mut current = node.prev_named_sibling();

    while let Some(sibling) = current {
        if sibling.kind() != "attribute_item" {
            break;
        }

        attributes.push(sibling);
        current = sibling.prev_named_sibling();
    }

    attributes
}

fn attribute_marks_test(node: Node<'_>, source: &str) -> bool {
    let normalized = source
        .get(node.byte_range())
        .unwrap_or("")
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    normalized.contains("cfg(test)")
        || normalized.starts_with("#[test]")
        || normalized.ends_with("::test]")
        || normalized.contains("::test(")
}

pub(crate) fn string_literal_value(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "string_literal" | "raw_string_literal" => {
            let text = source.get(node.byte_range())?.trim();
            unquote_rust_string(text)
        }
        _ => None,
    }
}

fn unquote_rust_string(text: &str) -> Option<String> {
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return text.get(1..text.len() - 1).map(ToOwned::to_owned);
    }

    if text.starts_with('r') {
        let start = text.find('"')?;
        let end = text.rfind('"')?;
        if end > start {
            return text.get(start + 1..end).map(ToOwned::to_owned);
        }
    }

    None
}
