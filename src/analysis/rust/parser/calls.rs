use tree_sitter::Node;

use crate::analysis::{CallSite, RuntimeCall};

use super::super::imports::combine_path_prefix;

pub(super) fn collect_calls(node: Node<'_>, source: &str) -> Vec<CallSite> {
    let mut calls = Vec::new();
    visit_for_calls(node, source, &mut calls);
    calls
}

fn visit_for_calls(node: Node<'_>, source: &str, calls: &mut Vec<CallSite>) {
    if node.kind() == "call_expression"
        && let Some(function_node) = node.child_by_field_name("function")
    {
        let function_text = render_call_target(function_node, source);
        let (receiver, name) = split_call_target(&function_text);
        calls.push(CallSite {
            receiver,
            name,
            line: node.start_position().row + 1,
        });
    }

    if node.kind() == "macro_invocation"
        && let Some(macro_node) = node.child_by_field_name("macro")
        && let Some(macro_text) = source.get(macro_node.byte_range())
    {
        let (receiver, name) = split_call_target(macro_text);
        calls.push(CallSite {
            receiver,
            name: format!("{name}!"),
            line: node.start_position().row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_calls(child, source, calls);
    }
}

pub(super) fn render_call_target(node: Node<'_>, source: &str) -> String {
    if node.kind() == "field_expression" {
        let value = node
            .child_by_field_name("value")
            .map(|value_node| render_call_target(value_node, source));
        let field = node
            .child_by_field_name("field")
            .and_then(|field_node| source.get(field_node.byte_range()))
            .map(str::trim)
            .unwrap_or("");

        return combine_path_prefix(value.as_deref(), field);
    }

    source
        .get(node.byte_range())
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

pub(super) fn split_call_target(function_text: &str) -> (Option<String>, String) {
    let normalized = function_text.trim();

    if let Some((receiver, name)) = normalized.rsplit_once('.') {
        return (Some(receiver.trim().to_string()), name.trim().to_string());
    }

    if let Some((receiver, name)) = normalized.rsplit_once("::") {
        return (Some(receiver.trim().to_string()), name.trim().to_string());
    }

    (None, normalized.to_string())
}

pub(super) fn collect_macro_calls(calls: &[CallSite]) -> Vec<crate::analysis::MacroCall> {
    calls
        .iter()
        .filter(|call| call.name.ends_with('!'))
        .map(|call| crate::analysis::MacroCall {
            line: call.line,
            name: call.name.clone(),
        })
        .collect()
}

pub(super) fn collect_named_runtime_calls(calls: &[CallSite], names: &[&str]) -> Vec<RuntimeCall> {
    calls
        .iter()
        .filter(|call| names.contains(&call.name.as_str()))
        .map(|call| RuntimeCall {
            line: call.line,
            name: call.name.clone(),
            receiver: call.receiver.clone(),
        })
        .collect()
}

pub(super) fn collect_blocking_calls(
    node: Node<'_>,
    source: &str,
    calls: &[CallSite],
) -> Vec<RuntimeCall> {
    let mut blocking_calls = calls
        .iter()
        .filter(|call| is_blocking_call(call))
        .map(|call| RuntimeCall {
            line: call.line,
            name: call.name.clone(),
            receiver: call.receiver.clone(),
        })
        .collect::<Vec<_>>();

    visit_textual_blocking_calls(node, source, &mut blocking_calls);
    blocking_calls
        .sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    blocking_calls.dedup_by(|left, right| {
        left.line == right.line && left.name == right.name && left.receiver == right.receiver
    });
    blocking_calls
}

fn is_blocking_call(call: &CallSite) -> bool {
    let receiver = call.receiver.as_deref().unwrap_or_default();
    matches!(
        call.name.as_str(),
        "read_to_string"
            | "read"
            | "read_to_end"
            | "write"
            | "write_all"
            | "open"
            | "create"
            | "metadata"
            | "sleep"
            | "join"
            | "block_on"
    ) || receiver.contains("std::fs")
        || receiver.contains("fs")
        || receiver.contains("std::thread")
        || receiver.contains("File")
}

fn visit_textual_blocking_calls(
    node: Node<'_>,
    source: &str,
    blocking_calls: &mut Vec<RuntimeCall>,
) {
    if let Some(text) = source.get(node.byte_range()) {
        let blocking_name = if text.contains("std::thread::sleep") {
            Some("sleep")
        } else if text.contains("std::fs::") || text.contains("fs::read_to_string") {
            Some("fs")
        } else if text.contains("block_on(") {
            Some("block_on")
        } else {
            None
        };

        if let Some(name) = blocking_name {
            blocking_calls.push(RuntimeCall {
                line: node.start_position().row + 1,
                name: name.to_string(),
                receiver: None,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_textual_blocking_calls(child, source, blocking_calls);
    }
}
