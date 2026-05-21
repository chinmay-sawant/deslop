use std::collections::BTreeMap;

use tree_sitter::Node;

pub(super) fn collect_concat_loops(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let string_like_names = collect_string_like_names(body_node, source);
    let mut lines = Vec::new();
    visit_concat_loops(body_node, source, &string_like_names, None, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn collect_string_like_names(node: Node<'_>, source: &str) -> BTreeMap<String, usize> {
    let mut names = BTreeMap::new();
    visit_string_like_names(node, source, &mut names);
    names
}

fn visit_string_like_names(node: Node<'_>, source: &str, names: &mut BTreeMap<String, usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        if let Some((left, right)) = trimmed.split_once('=')
            && (looks_like_string_literal(right.trim())
                || left.contains(": str")
                || left.contains(":str"))
        {
            for name in assignment_targets(left) {
                names.entry(name).or_insert(node.start_byte());
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_string_like_names(child, source, names);
    }
}

fn visit_concat_loops(
    node: Node<'_>,
    source: &str,
    string_like_names: &BTreeMap<String, usize>,
    current_loop: Option<Node<'_>>,
    lines: &mut Vec<usize>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    let next_loop = if matches!(node.kind(), "for_statement" | "while_statement") {
        Some(node)
    } else {
        current_loop
    };

    if next_loop.is_some()
        && let Some(var_name) = get_concat_assignment_target(node, source, string_like_names)
    {
        let mut flag = true;
        if let Some(decl_pos) = string_like_names.get(&var_name)
            && let Some(loop_node) = next_loop
            && *decl_pos >= loop_node.start_byte()
            && *decl_pos < loop_node.end_byte()
        {
            flag = false;
        }
        if flag {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_concat_loops(child, source, string_like_names, next_loop, lines);
    }
}

fn get_concat_assignment_target(
    node: Node<'_>,
    source: &str,
    string_like_names: &BTreeMap<String, usize>,
) -> Option<String> {
    let text = source.get(node.byte_range())?;
    let trimmed = text.trim();

    if node.kind() == "augmented_assignment"
        && trimmed.contains("+=")
        && let Some((left, _)) = trimmed.split_once("+=")
    {
        for name in assignment_targets(left) {
            if string_like_names.contains_key(&name) {
                return Some(name);
            }
        }
    }

    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some((left, right)) = trimmed.split_once('=')
    {
        let names = assignment_targets(left);
        if names.len() == 1
            && let Some(target_name) = names.first()
            && string_like_names.contains_key(target_name)
        {
            let normalized_right = right.replace(' ', "");
            if normalized_right.starts_with(&format!("{target_name}+")) {
                return Some(target_name.clone());
            }
        }
    }

    None
}

fn assignment_targets(text: &str) -> Vec<String> {
    text.trim()
        .split(':')
        .next()
        .unwrap_or(text)
        .split(',')
        .map(str::trim)
        .filter(|candidate| is_valid_identifier(candidate))
        .map(str::to_string)
        .collect()
}

fn looks_like_string_literal(text: &str) -> bool {
    let trimmed = text.trim();
    let quote_index = match trimmed.find(['\'', '"']) {
        Some(index) => index,
        None => return false,
    };
    trimmed
        .get(..quote_index)
        .unwrap_or("")
        .chars()
        .all(|character| matches!(character, 'r' | 'R' | 'u' | 'U' | 'b' | 'B' | 'f' | 'F'))
}

fn is_valid_identifier(candidate: &str) -> bool {
    let mut characters = candidate.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }

    characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn should_skip_nested_scope(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_definition" | "class_definition")
}
