use tree_sitter::Node;

use crate::analysis::{ContextFactoryCall, ImportSpec};

use super::general::{count_descendants, is_identifier_name, split_assignment};

pub(super) fn function_has_context_parameter(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> bool {
    let Some(parameters_node) = node.child_by_field_name("parameters") else {
        return false;
    };
    let Some(parameters_text) = source.get(parameters_node.byte_range()) else {
        return false;
    };

    imports
        .iter()
        .filter(|import| import.path == "context")
        .any(|import| parameters_text.contains(&format!("{}.Context", import.alias)))
}

pub(super) fn collect_sleep_in_loop_lines(
    body_node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_sleep_in_loop(body_node, source, imports, false, &mut lines);
    lines
}

fn visit_for_sleep_in_loop(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
    inside_loop: bool,
    lines: &mut Vec<usize>,
) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if next_inside_loop && node.kind() == "call_expression" {
        if let Some(function_node) = node.child_by_field_name("function") {
            let target = source.get(function_node.byte_range()).unwrap_or("").trim();
            if is_time_sleep_call(target, imports) {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_sleep_in_loop(child, source, imports, next_inside_loop, lines);
    }
}

fn is_time_sleep_call(target: &str, imports: &[ImportSpec]) -> bool {
    imports
        .iter()
        .filter(|import| import.path == "time")
        .any(|import| target == format!("{}.Sleep", import.alias))
}

pub(super) fn collect_busy_wait_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_busy_wait(body_node, source, false, &mut lines);
    lines
}

fn visit_for_busy_wait(node: Node<'_>, source: &str, inside_loop: bool, lines: &mut Vec<usize>) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if next_inside_loop
        && node.kind() == "select_statement"
        && source
            .get(node.byte_range())
            .is_some_and(|text| text.contains("default:"))
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_busy_wait(child, source, next_inside_loop, lines);
    }
}

pub(super) fn collect_context_factory_calls(
    body_node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Vec<ContextFactoryCall> {
    let mut calls = Vec::new();
    visit_for_context_factory_calls(body_node, source, imports, &mut calls);
    calls
}

fn visit_for_context_factory_calls(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
    calls: &mut Vec<ContextFactoryCall>,
) {
    if matches!(node.kind(), "assignment_statement" | "short_var_declaration" | "var_spec") {
        if let Some(call) = parse_context_factory_call(node, source, imports) {
            calls.push(call);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_context_factory_calls(child, source, imports, calls);
    }
}

fn parse_context_factory_call(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Option<ContextFactoryCall> {
    let text = source.get(node.byte_range())?;
    let (left, right) = split_assignment(text)?;
    let factory_name = context_factory_name(right, imports)?;
    let cancel_name = extract_cancel_name(left)?;

    Some(ContextFactoryCall {
        line: node.start_position().row + 1,
        cancel_name,
        factory_name,
    })
}

fn context_factory_name(text: &str, imports: &[ImportSpec]) -> Option<String> {
    let compact = text.split_whitespace().collect::<String>();

    for import in imports.iter().filter(|import| import.path == "context") {
        for factory_name in ["WithCancel", "WithTimeout", "WithDeadline"] {
            let prefix = format!("{}.{}(", import.alias, factory_name);
            if compact.starts_with(&prefix) {
                return Some(factory_name.to_string());
            }
        }
    }

    None
}

fn extract_cancel_name(left: &str) -> Option<String> {
    let candidate = left.rsplit(',').next()?.trim();
    let cancel_name = candidate.split_whitespace().last()?;
    if cancel_name == "_" || !is_identifier_name(cancel_name) {
        return None;
    }

    Some(cancel_name.to_string())
}

pub(super) fn collect_goroutine_launch_lines(body_node: Node<'_>) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_goroutine_launches(body_node, &mut lines);
    lines
}

pub(super) fn collect_goroutine_in_loop_lines(body_node: Node<'_>) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_goroutine_in_loop(body_node, false, &mut lines);
    lines
}

fn visit_for_goroutine_launches(node: Node<'_>, lines: &mut Vec<usize>) {
    if node.kind() == "go_statement" {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_goroutine_launches(child, lines);
    }
}

fn visit_for_goroutine_in_loop(node: Node<'_>, inside_loop: bool, lines: &mut Vec<usize>) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if next_inside_loop && node.kind() == "go_statement" {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_goroutine_in_loop(child, next_inside_loop, lines);
    }
}

pub(super) fn collect_goroutine_without_shutdown_lines(
    body_node: Node<'_>,
    source: &str,
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_goroutines_without_shutdown(body_node, source, &mut lines);
    lines
}

fn visit_for_goroutines_without_shutdown(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if node.kind() == "go_statement"
        && source.get(node.byte_range()).is_some_and(|text| {
            let compact = text.split_whitespace().collect::<String>();
            let has_func_literal = compact.contains("gofunc(") || compact.contains("gofunc()");
            let has_loop = count_descendants(node, "for_statement") > 0;
            let has_shutdown_signal = compact.contains("ctx.Done()")
                || compact.contains("<-done")
                || compact.contains("<-shutdown")
                || compact.contains("case<-done")
                || compact.contains("case<-shutdown")
                || compact.contains("case<-ctx.Done()");
            has_func_literal && has_loop && !has_shutdown_signal
        })
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_goroutines_without_shutdown(child, source, lines);
    }
}

pub(super) fn collect_mutex_lock_in_loop_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_mutex_lock_in_loop(body_node, source, false, &mut lines);
    lines
}

fn visit_for_mutex_lock_in_loop(
    node: Node<'_>,
    source: &str,
    inside_loop: bool,
    lines: &mut Vec<usize>,
) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if next_inside_loop && node.kind() == "call_expression" {
        if let Some(function_node) = node.child_by_field_name("function") {
            let target = source.get(function_node.byte_range()).unwrap_or("").trim();
            if is_mutex_lock_call(target) {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_mutex_lock_in_loop(child, source, next_inside_loop, lines);
    }
}

fn is_mutex_lock_call(target: &str) -> bool {
    target.ends_with(".Lock") || target.ends_with(".RLock")
}
