use std::collections::BTreeSet;

use tree_sitter::Node;

use crate::analysis::ClassSummary;

pub(super) fn collect_none_comparison_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_none_comparisons(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_side_effect_comprehension_lines(body_node: Node<'_>) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_side_effect_comprehensions(body_node, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_redundant_return_none_lines(
    body_node: Node<'_>,
    source: &str,
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_return_none(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_list_materialization_lines(
    body_node: Node<'_>,
    source: &str,
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_list_materializations(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_deque_operation_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_deque_candidates(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn parameter_flags(function_node: Node<'_>, source: &str) -> (bool, bool) {
    let Some(parameters_node) = function_node.child_by_field_name("parameters") else {
        return (false, false);
    };
    let Some(parameters_text) = source.get(parameters_node.byte_range()) else {
        return (false, false);
    };

    let mut has_varargs = false;
    let mut has_kwargs = false;
    for entry in parameters_text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        if entry.starts_with("**") {
            has_kwargs = true;
        } else if entry.starts_with('*') && entry != "*" {
            has_varargs = true;
        }
    }

    (has_varargs, has_kwargs)
}

pub(super) fn collect_class_summaries(root: Node<'_>, source: &str) -> Vec<ClassSummary> {
    let mut summaries = Vec::new();
    visit_class_summaries(root, source, &mut summaries);
    summaries.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    summaries
}

fn visit_none_comparisons(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind().contains("comparison")
        && let Some(text) = source.get(node.byte_range())
        && is_none_comparison(text)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_none_comparisons(child, source, lines);
    }
}

fn visit_side_effect_comprehensions(node: Node<'_>, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "expression_statement"
        && let Some(expression) = node.named_child(0)
        && matches!(
            expression.kind(),
            "list_comprehension" | "dictionary_comprehension" | "set_comprehension"
        )
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_side_effect_comprehensions(child, lines);
    }
}

fn visit_return_none(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "return_statement"
        && let Some(text) = source.get(node.byte_range())
        && text.trim() == "return None"
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_return_none(child, source, lines);
    }
}

fn visit_list_materializations(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "subscript"
        && let Some(text) = source.get(node.byte_range())
        && is_list_materialization(text)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_list_materializations(child, source, lines);
    }
}

fn visit_deque_candidates(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "call"
        && let Some(text) = source.get(node.byte_range())
        && is_deque_candidate(text)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_deque_candidates(child, source, lines);
    }
}

fn visit_class_summaries(node: Node<'_>, source: &str, summaries: &mut Vec<ClassSummary>) {
    if node.kind() == "class_definition"
        && let Some(summary) = class_summary(node, source)
    {
        summaries.push(summary);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_class_summaries(child, source, summaries);
    }
}

fn class_summary(node: Node<'_>, source: &str) -> Option<ClassSummary> {
    let name = node
        .child_by_field_name("name")
        .and_then(|name_node| source.get(name_node.byte_range()))?
        .trim()
        .to_string();
    let body_node = node.child_by_field_name("body")?;
    let mut method_count = 0;
    let mut instance_attributes = BTreeSet::new();
    let mut cursor = body_node.walk();

    for child in body_node.named_children(&mut cursor) {
        if let Some(method_node) = direct_method_node(child) {
            method_count += 1;
            if let Some(method_body) = method_node.child_by_field_name("body") {
                collect_self_attribute_writes(method_body, source, &mut instance_attributes);
            }
        }
    }

    Some(ClassSummary {
        name,
        line: node.start_position().row + 1,
        method_count,
        instance_attribute_count: instance_attributes.len(),
    })
}

fn direct_method_node(node: Node<'_>) -> Option<Node<'_>> {
    match node.kind() {
        "function_definition" => Some(node),
        "decorated_definition" => {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find(|child| child.kind() == "function_definition")
        }
        _ => None,
    }
}

fn collect_self_attribute_writes(
    node: Node<'_>,
    source: &str,
    attributes: &mut BTreeSet<String>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "assignment" | "annotated_assignment" | "augmented_assignment")
        && let Some(text) = source.get(node.byte_range())
        && let Some(left) = assignment_left(text)
    {
        for target in left.split(',') {
            if let Some(attribute) = self_attribute_name(target) {
                attributes.insert(attribute);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_self_attribute_writes(child, source, attributes);
    }
}

fn assignment_left(text: &str) -> Option<&str> {
    for operator in ["+=", "-=", "*=", "/=", "%=", "="] {
        if let Some((left, _)) = text.split_once(operator) {
            return Some(left);
        }
    }
    None
}

fn self_attribute_name(target: &str) -> Option<String> {
    let trimmed = target
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(':')
        .next()
        .unwrap_or(target)
        .trim();
    let suffix = trimmed.strip_prefix("self.")?;
    let attribute = suffix
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    (!attribute.is_empty()).then_some(attribute)
}

fn is_none_comparison(text: &str) -> bool {
    let normalized = text.replace(['\n', '\t'], " ");
    normalized.contains("== None")
        || normalized.contains("!= None")
        || normalized.contains("None ==")
        || normalized.contains("None !=")
}

fn is_list_materialization(text: &str) -> bool {
    let normalized = text.replace(' ', "");
    normalized.starts_with("list(") && normalized.ends_with(")[0]")
}

fn is_deque_candidate(text: &str) -> bool {
    let normalized = text.replace(' ', "");
    normalized.contains(".pop(0)") || normalized.contains(".insert(0,")
}

fn should_skip_nested_scope(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_definition" | "class_definition")
}