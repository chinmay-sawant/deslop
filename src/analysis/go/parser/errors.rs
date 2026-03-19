use tree_sitter::Node;

use crate::analysis::FormattedErrorCall;

use super::general::first_string_literal;

pub(super) fn collect_dropped_error_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_dropped_errors(body_node, source, &mut lines);
    lines
}

fn visit_for_dropped_errors(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if matches!(node.kind(), "assignment_statement" | "short_var_declaration") {
        if let Some(text) = source.get(node.byte_range()) {
            let compact = text.split_whitespace().collect::<String>();
            let drops_named_err = compact.starts_with("_=err")
                || compact.starts_with("_=ctx.Err()")
                || compact.contains(",_=err")
                || compact.contains(",_=ctx.Err()");
            if drops_named_err {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_dropped_errors(child, source, lines);
    }
}

pub(super) fn collect_panic_on_error_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_panic_on_error(body_node, source, &mut lines);
    lines
}

fn visit_for_panic_on_error(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if node.kind() == "if_statement" {
        let condition = node
            .child_by_field_name("condition")
            .and_then(|condition| source.get(condition.byte_range()));
        let consequence = node
            .child_by_field_name("consequence")
            .and_then(|consequence| source.get(consequence.byte_range()));

        if let (Some(condition), Some(consequence)) = (condition, consequence) {
            let normalized_condition = condition.split_whitespace().collect::<String>();
            let panic_like = consequence.contains("panic(")
                || consequence.contains("log.Fatal(")
                || consequence.contains("log.Fatalf(")
                || consequence.contains("log.Fatalln(");
            if normalized_condition.contains("err!=nil") && panic_like {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_panic_on_error(child, source, lines);
    }
}

pub(super) fn collect_errorf_calls(body_node: Node<'_>, source: &str) -> Vec<FormattedErrorCall> {
    let mut calls = Vec::new();
    visit_for_errorf_calls(body_node, source, &mut calls);
    calls
}

fn visit_for_errorf_calls(node: Node<'_>, source: &str, calls: &mut Vec<FormattedErrorCall>) {
    if node.kind() == "call_expression" {
        let function_node = node.child_by_field_name("function");
        let arguments_node = node.child_by_field_name("arguments");

        if let (Some(function_node), Some(arguments_node)) = (function_node, arguments_node) {
            let target = source.get(function_node.byte_range()).unwrap_or("");
            if target.trim() == "fmt.Errorf" {
                let arguments = source.get(arguments_node.byte_range()).unwrap_or("");
                let format_string = first_string_literal(arguments_node, source);
                calls.push(FormattedErrorCall {
                    line: node.start_position().row + 1,
                    format_string,
                    mentions_err: arguments.contains("err"),
                    uses_percent_w: arguments.contains("%w"),
                });
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_errorf_calls(child, source, calls);
    }
}
