use std::collections::BTreeSet;

use tree_sitter::Node;

use crate::analysis::{BlockFingerprint, ClassSummary};

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

pub(super) fn collect_validation_signature(
    body_node: Node<'_>,
    source: &str,
) -> Option<BlockFingerprint> {
    let mut signature_parts = Vec::new();
    let mut cursor = body_node.walk();

    for child in body_node.named_children(&mut cursor) {
        if child.kind() != "if_statement" {
            if signature_parts.is_empty() {
                continue;
            }
            break;
        }

        let text = source.get(child.byte_range())?.trim();
        if !(text.contains("return ") || text.contains("raise ") || text.ends_with("return")) {
            break;
        }

        signature_parts.push(normalize_shape(text));
        if signature_parts.len() >= 4 {
            break;
        }
    }

    (!signature_parts.is_empty()).then_some(BlockFingerprint {
        line: body_node.start_position().row + 1,
        signature: signature_parts.join(" | "),
    })
}

pub(super) fn collect_exception_block_signatures(
    body_node: Node<'_>,
    source: &str,
) -> Vec<BlockFingerprint> {
    let mut signatures = Vec::new();
    visit_exception_signatures(body_node, source, &mut signatures);
    signatures.sort_by(|left, right| left.line.cmp(&right.line));
    signatures.dedup_by(|left, right| left.line == right.line && left.signature == right.signature);
    signatures
}

pub(super) fn normalize_body(body_node: Node<'_>, source: &str) -> String {
    source
        .get(body_node.byte_range())
        .map(normalize_shape)
        .unwrap_or_default()
}

pub(super) fn has_complete_type_hints(function_node: Node<'_>, source: &str) -> bool {
    let Some(parameters_node) = function_node.child_by_field_name("parameters") else {
        return false;
    };
    let Some(function_text) = source.get(function_node.byte_range()) else {
        return false;
    };
    let Some(parameters_text) = source.get(parameters_node.byte_range()) else {
        return false;
    };

    let has_return_annotation = function_text.contains(") ->");
    let all_parameters_annotated = parameters_text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty() && *entry != "/" && *entry != "*")
        .filter(|entry| {
            let trimmed = entry.trim_start_matches('*').trim();
            !matches!(trimmed, "self" | "cls")
        })
        .all(|entry| {
            let trimmed = entry.trim_start_matches('*').trim();
            let subject = trimmed.split('=').next().unwrap_or(trimmed).trim();
            subject.contains(':')
        });

    has_return_annotation && all_parameters_annotated
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
        if normalized == function_name || normalized.ends_with(&format!(".{function_name}")) {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_recursive_calls(function_name, child, source, lines);
    }
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

fn visit_exception_signatures(
    node: Node<'_>,
    source: &str,
    signatures: &mut Vec<BlockFingerprint>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "except_clause"
        && let Some(text) = source.get(node.byte_range())
    {
        let normalized_lines = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .take(3)
            .collect::<Vec<_>>();
        if !normalized_lines.is_empty() {
            signatures.push(BlockFingerprint {
                line: node.start_position().row + 1,
                signature: normalize_shape(&normalized_lines.join(" ")),
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_exception_signatures(child, source, signatures);
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
    let class_text = source.get(node.byte_range())?;
    let mut method_count = 0;
    let mut public_method_count = 0;
    let mut instance_attributes = BTreeSet::new();
    let mut constructor_collaborator_count = 0;
    let mut cursor = body_node.walk();

    for child in body_node.named_children(&mut cursor) {
        if let Some(method_node) = direct_method_node(child) {
            method_count += 1;
            let method_name = method_node
                .child_by_field_name("name")
                .and_then(|name_node| source.get(name_node.byte_range()))
                .map(str::trim)
                .unwrap_or_default();
            if !method_name.starts_with('_') {
                public_method_count += 1;
            }
            if let Some(method_body) = method_node.child_by_field_name("body") {
                collect_self_attribute_writes(method_body, source, &mut instance_attributes);
                if method_name == "__init__" {
                    constructor_collaborator_count =
                        collect_constructor_collaborators(method_body, source);
                }
            }
        }
    }

    Some(ClassSummary {
        name,
        line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        method_count,
        public_method_count,
        instance_attribute_count: instance_attributes.len(),
        base_classes: class_base_names(class_text),
        constructor_collaborator_count,
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

fn collect_self_attribute_writes(node: Node<'_>, source: &str, attributes: &mut BTreeSet<String>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(
        node.kind(),
        "assignment" | "annotated_assignment" | "augmented_assignment"
    ) && let Some(text) = source.get(node.byte_range())
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

fn collect_constructor_collaborators(body_node: Node<'_>, source: &str) -> usize {
    let mut collaborators = BTreeSet::new();
    visit_constructor_collaborators(body_node, source, &mut collaborators);
    collaborators.len()
}

fn visit_constructor_collaborators(
    node: Node<'_>,
    source: &str,
    collaborators: &mut BTreeSet<String>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some(text) = source.get(node.byte_range())
        && let Some((left, right)) = text.split_once('=')
        && let Some(attribute_name) = self_attribute_name(left)
        && looks_like_constructor_call(right)
    {
        collaborators.insert(attribute_name);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_constructor_collaborators(child, source, collaborators);
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

fn class_base_names(text: &str) -> Vec<String> {
    let header = text.lines().next().unwrap_or_default().trim();
    let Some(open_index) = header.find('(') else {
        return Vec::new();
    };
    let Some(close_index) = header
        .get(open_index + 1..)
        .and_then(|suffix| suffix.find(')'))
    else {
        return Vec::new();
    };
    header
        .get(open_index + 1..open_index + 1 + close_index)
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|base| !base.is_empty())
        .map(str::to_string)
        .collect()
}

fn looks_like_constructor_call(text: &str) -> bool {
    let normalized = text.trim().replace(' ', "");
    if !normalized.contains('(') {
        return false;
    }

    let Some(target) = normalized.split('(').next() else {
        return false;
    };
    !matches!(
        target,
        "list" | "dict" | "set" | "tuple" | "str" | "int" | "float" | "bool"
    ) && !target.starts_with("self.")
}

fn normalize_shape(text: &str) -> String {
    let mut output = String::new();
    let mut token = String::new();
    let mut in_string = false;
    let mut quote_char = '\0';

    for character in text.chars() {
        if in_string {
            if character == quote_char {
                output.push_str("STR");
                in_string = false;
            }
            continue;
        }

        if matches!(character, '\'' | '"') {
            flush_shape_token(&mut output, &mut token);
            in_string = true;
            quote_char = character;
            continue;
        }

        if character.is_ascii_alphanumeric() || character == '_' {
            token.push(character);
            continue;
        }

        flush_shape_token(&mut output, &mut token);
        if !character.is_whitespace() {
            output.push(character);
        }
    }

    flush_shape_token(&mut output, &mut token);
    output
}

fn flush_shape_token(output: &mut String, token: &mut String) {
    if token.is_empty() {
        return;
    }

    let replacement = if token.chars().all(|character| character.is_ascii_digit()) {
        "NUM"
    } else if is_python_keyword(token) {
        token.as_str()
    } else {
        "ID"
    };
    output.push_str(replacement);
    token.clear();
}

fn is_python_keyword(token: &str) -> bool {
    matches!(
        token,
        "and"
            | "as"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "elif"
            | "else"
            | "except"
            | "False"
            | "finally"
            | "for"
            | "from"
            | "if"
            | "import"
            | "in"
            | "is"
            | "None"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "True"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
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
