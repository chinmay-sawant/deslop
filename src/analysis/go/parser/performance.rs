use std::collections::BTreeSet;

use tree_sitter::Node;

use crate::analysis::{DbQueryCall, ImportSpec};

use super::general::{
    collect_identifiers, extract_call_target, find_var_name_node, first_string_literal,
    is_identifier_name, split_assignment,
};

pub(super) fn collect_string_concat_in_loop_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let string_variables = collect_explicit_string_variables(body_node, source);
    let mut lines = Vec::new();
    visit_for_string_concat_in_loop(body_node, source, &string_variables, false, &mut lines);
    lines
}

fn collect_explicit_string_variables(body_node: Node<'_>, source: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    visit_for_string_variables(body_node, source, &mut names);
    names
}

fn visit_for_string_variables(node: Node<'_>, source: &str, names: &mut BTreeSet<String>) {
    match node.kind() {
        "var_spec" => {
            let Some(type_node) = node.child_by_field_name("type") else {
                return;
            };
            if source
                .get(type_node.byte_range())
                .is_some_and(|text| text.trim() == "string")
            {
                if let Some(name_node) = find_var_name_node(node) {
                    for (name, _) in collect_identifiers(name_node, source) {
                        names.insert(name);
                    }
                }
            }
        }
        "short_var_declaration" | "assignment_statement" => {
            if let Some(text) = source.get(node.byte_range()) {
                if let Some((left, right)) = split_assignment(text) {
                    let left = left.trim();
                    if is_identifier_name(left) && contains_string_literal(right) {
                        names.insert(left.to_string());
                    }
                }
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_string_variables(child, source, names);
    }
}

fn visit_for_string_concat_in_loop(
    node: Node<'_>,
    source: &str,
    string_variables: &BTreeSet<String>,
    inside_loop: bool,
    lines: &mut Vec<usize>,
) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if next_inside_loop && node.kind() == "assignment_statement" {
        if let Some(text) = source.get(node.byte_range()) {
            if is_string_concat_assignment(text, string_variables) {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_string_concat_in_loop(child, source, string_variables, next_inside_loop, lines);
    }
}

fn is_string_concat_assignment(text: &str, string_variables: &BTreeSet<String>) -> bool {
    let compact = text.split_whitespace().collect::<String>();

    if let Some((left, right)) = compact.split_once("+=") {
        return is_identifier_name(left)
            && (string_variables.contains(left) || contains_string_literal(right));
    }

    let Some((left, right)) = compact.split_once('=') else {
        return false;
    };
    if !is_identifier_name(left) || !string_variables.contains(left) {
        return false;
    }

    right.starts_with(&format!("{left}+")) || right.contains(&format!("+\"")) || right.contains("+`")
}

fn contains_string_literal(text: &str) -> bool {
    text.contains('"') || text.contains('`')
}

pub(super) fn collect_allocation_in_loop_lines(
    body_node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_allocation_in_loop(body_node, source, imports, false, &mut lines);
    lines
}

fn visit_for_allocation_in_loop(
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
            if is_allocation_call(target, imports) {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_allocation_in_loop(child, source, imports, next_inside_loop, lines);
    }
}

fn is_allocation_call(target: &str, imports: &[ImportSpec]) -> bool {
    if matches!(target, "make" | "new") {
        return true;
    }

    imports.iter().any(|import| {
        import.path == "bytes"
            && (target == format!("{}.NewBuffer", import.alias)
                || target == format!("{}.NewBufferString", import.alias))
    })
}

pub(super) fn collect_fmt_in_loop_lines(
    body_node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_fmt_in_loop(body_node, source, imports, false, &mut lines);
    lines
}

fn visit_for_fmt_in_loop(
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
            if is_fmt_hot_path_call(target, imports) {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_fmt_in_loop(child, source, imports, next_inside_loop, lines);
    }
}

fn is_fmt_hot_path_call(target: &str, imports: &[ImportSpec]) -> bool {
    imports.iter().any(|import| {
        import.path == "fmt"
            && ["Sprintf", "Sprint", "Sprintln", "Fprintf", "Fprint", "Fprintln"]
                .iter()
                .any(|name| target == format!("{}.{}", import.alias, name))
    })
}

pub(super) fn collect_reflection_in_loop_lines(
    body_node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_reflection_in_loop(body_node, source, imports, false, &mut lines);
    lines
}

fn visit_for_reflection_in_loop(
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
            if is_reflection_call(target, imports) {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_reflection_in_loop(child, source, imports, next_inside_loop, lines);
    }
}

fn is_reflection_call(target: &str, imports: &[ImportSpec]) -> bool {
    imports
        .iter()
        .filter(|import| import.path == "reflect")
        .any(|import| target.starts_with(&format!("{}.", import.alias)))
}

pub(super) fn collect_json_marshal_in_loop_lines(
    body_node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_json_marshal_in_loop(body_node, source, imports, false, &mut lines);
    lines
}

fn visit_for_json_marshal_in_loop(
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
            if is_json_marshal_call(target, imports) {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_json_marshal_in_loop(child, source, imports, next_inside_loop, lines);
    }
}

fn is_json_marshal_call(target: &str, imports: &[ImportSpec]) -> bool {
    imports
        .iter()
        .filter(|import| import.path == "encoding/json")
        .any(|import| {
            target == format!("{}.Marshal", import.alias)
                || target == format!("{}.MarshalIndent", import.alias)
        })
}

pub(super) fn collect_db_query_calls(body_node: Node<'_>, source: &str) -> Vec<DbQueryCall> {
    let mut calls = Vec::new();
    visit_for_db_query_calls(body_node, source, false, &mut calls);
    calls
}

fn visit_for_db_query_calls(
    node: Node<'_>,
    source: &str,
    inside_loop: bool,
    calls: &mut Vec<DbQueryCall>,
) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if node.kind() == "call_expression" {
        let function_node = node.child_by_field_name("function");
        let arguments_node = node.child_by_field_name("arguments");

        if let Some(function_node) = function_node {
            if let Some((receiver, name)) = extract_call_target(function_node, source) {
                if is_database_query_method(&name) {
                    let query_text =
                        arguments_node.and_then(|arguments| first_string_literal(arguments, source));
                    calls.push(DbQueryCall {
                        line: node.start_position().row + 1,
                        receiver,
                        method_name: name,
                        query_text,
                        in_loop: next_inside_loop,
                    });
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_db_query_calls(child, source, next_inside_loop, calls);
    }
}

fn is_database_query_method(name: &str) -> bool {
    matches!(
        name,
        "Query"
            | "QueryContext"
            | "QueryRow"
            | "QueryRowContext"
            | "Exec"
            | "ExecContext"
            | "Get"
            | "Select"
            | "Raw"
            | "First"
            | "Find"
            | "Take"
            | "Preload"
    )
}
