use tree_sitter::Node;

use crate::analysis::CallSite;

pub(crate) fn collect_calls(body_node: Node<'_>, source: &str) -> Vec<CallSite> {
    let mut calls = Vec::new();
    visit_calls(body_node, source, &mut calls);
    calls
}

fn visit_calls(node: Node<'_>, source: &str, calls: &mut Vec<CallSite>) {
    if node.kind() == "call_expression"
        && let Some(function_node) = node.child_by_field_name("function")
        && let Some((receiver, name)) = extract_call_target(function_node, source)
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

pub(crate) fn extract_call_target(
    function_node: Node<'_>,
    source: &str,
) -> Option<(Option<String>, String)> {
    let text = source.get(function_node.byte_range())?.trim();
    if text.is_empty() {
        return None;
    }

    if let Some((receiver, name)) = text.rsplit_once('.') {
        return Some((Some(receiver.trim().to_string()), name.trim().to_string()));
    }

    Some((None, text.to_string()))
}
