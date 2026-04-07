#[path = "phase4/classes.rs"]
mod classes;
#[path = "phase4/signatures.rs"]
mod signatures;
#[path = "phase4/typing.rs"]
mod typing;

use std::collections::BTreeSet;

use tree_sitter::Node;

pub(super) use classes::collect_class_summaries;
pub(super) use signatures::{
    collect_exception_block_signatures, collect_validation_signature, normalize_body,
};
pub(super) use typing::{has_complete_type_hints, parameter_flags};

pub(super) fn collect_none_comparison_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_none_comparisons(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_side_effect_lines(body_node: Node<'_>) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_side_effect_comprehensions(body_node, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_return_none_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_return_none(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_list_materialization_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
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

pub(super) fn collect_temp_collection_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_temp_collections(body_node, source, false, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_recursive_call_lines(
    function_name: &str,
    body_node: Node<'_>,
    source: &str,
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_recursive_calls(function_name, body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_membership_loop_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let list_like_names = collect_list_like_names(body_node, source);
    let mut lines = Vec::new();
    visit_list_membership(body_node, source, &list_like_names, false, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_repeated_len_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_repeated_len_loops(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_builtin_candidate_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_builtin_candidates(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_missing_manager_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_missing_manager_calls(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
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

fn visit_temp_collections(node: Node<'_>, source: &str, inside_loop: bool, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    let next_inside_loop =
        inside_loop || matches!(node.kind(), "for_statement" | "while_statement");
    if next_inside_loop
        && matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some(text) = source.get(node.byte_range())
        && assignment_creates_temp_collection(text)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_temp_collections(child, source, next_inside_loop, lines);
    }
}

fn visit_recursive_calls(
    function_name: &str,
    node: Node<'_>,
    source: &str,
    lines: &mut Vec<usize>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "call"
        && let Some(function_node) = node.child_by_field_name("function")
        && let Some(target_text) = source.get(function_node.byte_range())
    {
        let normalized = target_text.trim();
        if is_recursive_call_target(normalized, function_name) {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_recursive_calls(function_name, child, source, lines);
    }
}

fn is_recursive_call_target(normalized: &str, function_name: &str) -> bool {
    normalized == function_name
        || normalized.strip_prefix("self.") == Some(function_name)
        || normalized.strip_prefix("cls.") == Some(function_name)
        || normalized.strip_prefix("super().") == Some(function_name)
}

fn visit_list_membership(
    node: Node<'_>,
    source: &str,
    list_like_names: &BTreeSet<String>,
    inside_loop: bool,
    lines: &mut Vec<usize>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    let next_inside_loop =
        inside_loop || matches!(node.kind(), "for_statement" | "while_statement");
    if next_inside_loop
        && node.kind().contains("comparison")
        && let Some(text) = source.get(node.byte_range())
        && is_list_membership_comparison(text, list_like_names)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_list_membership(child, source, list_like_names, next_inside_loop, lines);
    }
}

fn visit_repeated_len_loops(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "for_statement" | "while_statement")
        && let Some(text) = source.get(node.byte_range())
        && has_repeated_len_target(text)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_repeated_len_loops(child, source, lines);
    }
}

fn visit_builtin_candidates(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "for_statement" | "while_statement")
        && let Some(text) = source.get(node.byte_range())
        && looks_like_builtin_reduction(text)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_builtin_candidates(child, source, lines);
    }
}

fn visit_missing_manager_calls(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "call"
        && !is_inside_with(node)
        && let Some(function_node) = node.child_by_field_name("function")
        && let Some(target_text) = source.get(function_node.byte_range())
        && looks_like_manager_candidate(target_text)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_missing_manager_calls(child, source, lines);
    }
}

fn is_none_comparison(text: &str) -> bool {
    let normalized = text.replace(['\n', '\t'], " ");
    normalized.contains("== None")
        || normalized.contains("!= None")
        || normalized.contains("None ==")
        || normalized.contains("None !=")
}

fn assignment_creates_temp_collection(text: &str) -> bool {
    let Some((_, right)) = text.split_once('=') else {
        return false;
    };
    let normalized = right.trim().replace(' ', "");
    normalized.starts_with('[')
        || normalized.starts_with('{')
        || normalized.starts_with("list(")
        || normalized.starts_with("dict(")
        || normalized.starts_with("set(")
}

fn collect_list_like_names(node: Node<'_>, source: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    visit_list_like_names(node, source, &mut names);
    names
}

fn visit_list_like_names(node: Node<'_>, source: &str, names: &mut BTreeSet<String>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some(text) = source.get(node.byte_range())
        && let Some((left, right)) = text.split_once('=')
    {
        let normalized = right.trim().replace(' ', "");
        if normalized.starts_with('[') || normalized.starts_with("list(") {
            for name in left.split(',').filter_map(|target| {
                let trimmed = target.trim();
                (!trimmed.is_empty()).then_some(trimmed.to_string())
            }) {
                names.insert(name);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_list_like_names(child, source, names);
    }
}

fn is_list_membership_comparison(text: &str, list_like_names: &BTreeSet<String>) -> bool {
    let normalized = text.replace(['\n', '\t'], " ");
    if normalized.contains(" in [") || normalized.contains(" not in [") {
        return true;
    }

    list_like_names.iter().any(|name| {
        normalized.contains(&format!(" in {name}"))
            || normalized.contains(&format!(" not in {name}"))
    })
}

fn has_repeated_len_target(text: &str) -> bool {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for target in len_targets(text) {
        *counts.entry(target).or_insert(0) += 1;
    }

    let assigned_targets = local_assignment_targets(text)
        .into_iter()
        .collect::<BTreeSet<_>>();
    counts
        .into_iter()
        .any(|(target, count)| count >= 2 && !assigned_targets.contains(&target))
}

fn len_targets(text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index + 4 <= bytes.len() {
        if bytes.get(index..index + 4) == Some(b"len(") {
            let start = index + 4;
            let mut end = start;
            while end < bytes.len()
                && ((bytes[end] as char).is_ascii_alphanumeric() || bytes[end] == b'_')
            {
                end += 1;
            }
            if end > start
                && let Some(target) = text.get(start..end)
            {
                targets.push(target.to_string());
            }
            index = end;
            continue;
        }
        index += 1;
    }
    targets
}

fn local_assignment_targets(text: &str) -> Vec<String> {
    let Some((left, _)) = text.split_once('=') else {
        return Vec::new();
    };

    left.split(',')
        .map(str::trim)
        .map(|target| target.split(':').next().unwrap_or(target).trim())
        .filter(|target| {
            let mut characters = target.chars();
            let Some(first) = characters.next() else {
                return false;
            };
            (first == '_' || first.is_ascii_alphabetic())
                && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
        })
        .map(str::to_string)
        .collect()
}

fn looks_like_builtin_reduction(text: &str) -> bool {
    let normalized = text.replace(' ', "");
    (normalized.contains("+=") && normalized.contains("for"))
        || (normalized.contains("returnTrue") && normalized.contains("returnFalse"))
        || (normalized.contains("returnFalse") && normalized.contains("returnTrue"))
}

fn looks_like_manager_candidate(target_text: &str) -> bool {
    let normalized = target_text.trim();
    normalized == "open"
        || normalized.ends_with(".open")
        || normalized.ends_with(".acquire")
        || normalized.ends_with(".connect")
}

fn is_inside_with(node: Node<'_>) -> bool {
    let mut parent = node.parent();
    while let Some(current) = parent {
        if current.kind() == "with_statement" {
            return true;
        }
        if matches!(current.kind(), "function_definition" | "class_definition") {
            return false;
        }
        parent = current.parent();
    }
    false
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
