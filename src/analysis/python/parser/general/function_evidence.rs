use std::collections::BTreeSet;

use tree_sitter::Node;

use crate::analysis::{CallSite, ExceptionHandler, NamedLiteral, TestFunctionSummary};

use super::{named_literal_from_assignment, parse_call_target, should_skip_nested_scope};

pub(crate) fn collect_calls(body_node: Node<'_>, source: &str) -> Vec<CallSite> {
    let mut calls = Vec::new();
    visit_calls(body_node, source, &mut calls);
    calls
}

pub(crate) fn collect_local_strings(body_node: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_local_strings(body_node, source, &mut literals);
    literals.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    literals
}

pub(crate) fn collect_exception_handlers(
    body_node: Node<'_>,
    source: &str,
) -> Vec<ExceptionHandler> {
    let mut handlers = Vec::new();
    visit_exception_handlers(body_node, source, &mut handlers);
    handlers
}

pub(crate) fn collect_local_bindings(function_node: Node<'_>, source: &str) -> Vec<String> {
    let mut names = BTreeSet::new();

    if let Some(parameters_node) = function_node.child_by_field_name("parameters")
        && let Some(parameters_text) = source.get(parameters_node.byte_range())
    {
        for name in parameter_names(parameters_text) {
            names.insert(name);
        }
    }

    if let Some(body_node) = function_node.child_by_field_name("body") {
        visit_assignment_bindings(body_node, source, &mut names);
    }

    names.into_iter().collect()
}

pub(crate) fn build_test_summary(
    function_name: &str,
    body_node: Node<'_>,
    source: &str,
    is_test_file: bool,
) -> Option<TestFunctionSummary> {
    if !is_test_file || !function_name.starts_with("test_") {
        return None;
    }

    let mut assertion_like_calls = 0;
    let mut error_assertion_calls = 0;
    let mut skip_calls = 0;
    let mut production_calls = 0;
    for call in collect_calls(body_node, source) {
        match (call.receiver.as_deref(), call.name.as_str()) {
            (_, "assert") | (Some("self"), _) if call.name.starts_with("assert") => {
                assertion_like_calls += 1;
                if matches!(call.name.as_str(), "assertRaises" | "assertRaisesRegex") {
                    error_assertion_calls += 1;
                }
            }
            (Some("pytest"), "raises") => {
                assertion_like_calls += 1;
                error_assertion_calls += 1;
            }
            (_, "skip") | (_, "skipTest") => {
                skip_calls += 1;
            }
            (_, name) if !matches!(name, "print" | "assert") => {
                production_calls += 1;
            }
            _ => {}
        }
    }

    assertion_like_calls += count_assert_statements(body_node);

    let body_text = source.get(body_node.byte_range()).unwrap_or_default();
    let has_todo_marker = body_text.to_ascii_uppercase().contains("TODO");

    Some(TestFunctionSummary {
        assertion_like_calls,
        error_assertion_calls,
        skip_calls,
        production_calls,
        has_todo_marker,
    })
}

fn count_assert_statements(node: Node<'_>) -> usize {
    let mut count = if node.kind() == "assert_statement" {
        1
    } else {
        0
    };
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        count += count_assert_statements(child);
    }
    count
}

fn visit_calls(node: Node<'_>, source: &str, calls: &mut Vec<CallSite>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "call"
        && let Some(function_node) = node.child_by_field_name("function")
        && let Some(callee_text) = source.get(function_node.byte_range())
        && let Some((receiver, name)) = parse_call_target(callee_text)
    {
        calls.push(CallSite {
            receiver,
            name,
            line: node.start_position().row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_calls(child, source, calls);
    }
}

fn visit_local_strings(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some(text) = source.get(node.byte_range())
        && let Some(literal) = named_literal_from_assignment(text, node.start_position().row + 1)
    {
        literals.push(literal);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_local_strings(child, source, literals);
    }
}

fn visit_assignment_bindings(node: Node<'_>, source: &str, names: &mut BTreeSet<String>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some(text) = source.get(node.byte_range())
    {
        for name in assignment_target_names(text) {
            names.insert(name);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_assignment_bindings(child, source, names);
    }
}

fn visit_exception_handlers(node: Node<'_>, source: &str, handlers: &mut Vec<ExceptionHandler>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "except_clause"
        && let Some(text) = source.get(node.byte_range())
        && let Some(handler) = exception_handler_from_text(text, node.start_position().row + 1)
    {
        handlers.push(handler);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_exception_handlers(child, source, handlers);
    }
}

fn exception_handler_from_text(text: &str, line: usize) -> Option<ExceptionHandler> {
    let trimmed_lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let clause_line = *trimmed_lines.first()?;
    if !clause_line.starts_with("except") {
        return None;
    }

    let action = handler_action(&trimmed_lines);
    let normalized_clause = clause_line.to_ascii_lowercase();
    let is_broad = normalized_clause == "except:"
        || normalized_clause.starts_with("except exception")
        || normalized_clause.starts_with("except baseexception")
        || normalized_clause.starts_with("except (exception")
        || normalized_clause.starts_with("except (baseexception");
    let suppresses = action.as_deref().is_some_and(is_suppression_action);

    Some(ExceptionHandler {
        line,
        clause: clause_line.to_string(),
        action,
        is_broad,
        suppresses,
    })
}

fn handler_action(trimmed_lines: &[&str]) -> Option<String> {
    if trimmed_lines.is_empty() {
        return None;
    }

    let clause_line = trimmed_lines[0];
    if let Some((_, inline_action)) = clause_line.split_once(':') {
        let inline_action = inline_action.trim();
        if !inline_action.is_empty() {
            return Some(inline_action.to_string());
        }
    }

    trimmed_lines.get(1).map(|line| (*line).to_string())
}

fn is_suppression_action(action: &str) -> bool {
    let normalized = action.trim().to_ascii_lowercase();
    normalized == "pass"
        || normalized == "continue"
        || normalized == "break"
        || normalized.starts_with("return")
}

fn assignment_target_names(text: &str) -> Vec<String> {
    let target_text = text.trim().trim_start_matches('(').trim_end_matches(')');
    let target_text = target_text.split(':').next().unwrap_or(target_text).trim();

    target_text
        .split(',')
        .map(str::trim)
        .filter(|candidate| is_valid_identifier(candidate))
        .map(str::to_string)
        .collect()
}

fn parameter_names(parameters_text: &str) -> Vec<String> {
    parameters_text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty() && *entry != "/")
        .filter_map(|entry| {
            let entry = entry.trim_start_matches('*').trim();
            let entry = entry.split('=').next().unwrap_or(entry).trim();
            let entry = entry.split(':').next().unwrap_or(entry).trim();
            is_valid_identifier(entry).then(|| entry.to_string())
        })
        .collect()
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
