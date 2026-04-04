use tree_sitter::Node;

use crate::analysis::{RuntimeCall, UnsafePattern, UnsafePatternKind};

use super::super::items::trait_impl_type;
use super::calls::{render_call_target, split_call_target};

pub(super) fn function_is_async(
    function_node: Node<'_>,
    body_node: Node<'_>,
    source: &str,
) -> bool {
    source
        .get(function_node.start_byte()..body_node.start_byte())
        .unwrap_or("")
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token == "async")
}

pub(super) fn collect_await_points(node: Node<'_>) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_await_points(node, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_future_creations(node: Node<'_>, source: &str) -> Vec<RuntimeCall> {
    let mut futures = Vec::new();
    visit_for_future_creations(node, source, &mut futures);
    futures
}

fn visit_for_await_points(node: Node<'_>, lines: &mut Vec<usize>) {
    if node.kind() == "await_expression" {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_await_points(child, lines);
    }
}

fn visit_for_future_creations(node: Node<'_>, source: &str, futures: &mut Vec<RuntimeCall>) {
    if node.kind() == "let_declaration"
        && let Some(value_node) = node.child_by_field_name("value")
        && let Some(value_text) = source.get(value_node.byte_range())
    {
        let trimmed = value_text.trim();
        if trimmed.starts_with("async ")
            || trimmed.contains(".fuse()")
            || trimmed.contains("Future")
        {
            futures.push(RuntimeCall {
                line: node.start_position().row + 1,
                name: "future".to_string(),
                receiver: None,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_future_creations(child, source, futures);
    }
}

pub(super) fn collect_unsafe_lines(
    function_node: Node<'_>,
    body_node: Node<'_>,
    source: &str,
) -> Vec<usize> {
    let mut lines = Vec::new();
    let signature_text = source
        .get(function_node.start_byte()..body_node.start_byte())
        .unwrap_or("");

    if signature_text
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token == "unsafe")
    {
        lines.push(function_node.start_position().row + 1);
    }

    visit_for_unsafe_lines(body_node, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_for_unsafe_lines(node: Node<'_>, lines: &mut Vec<usize>) {
    if node.kind() == "unsafe_block" {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_unsafe_lines(child, lines);
    }
}

pub(super) fn collect_loop_operation_lines(
    node: Node<'_>,
    source: &str,
) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let mut write_loops = Vec::new();
    let mut line_iteration_loops = Vec::new();
    let mut default_hasher_lines = Vec::new();
    visit_loop_operation_lines(
        node,
        source,
        false,
        &mut write_loops,
        &mut line_iteration_loops,
        &mut default_hasher_lines,
    );
    (write_loops, line_iteration_loops, default_hasher_lines)
}

fn visit_loop_operation_lines(
    node: Node<'_>,
    source: &str,
    in_loop: bool,
    write_loops: &mut Vec<usize>,
    line_iteration_loops: &mut Vec<usize>,
    default_hasher_lines: &mut Vec<usize>,
) {
    let child_in_loop = in_loop || is_loop_node(node.kind());

    if child_in_loop
        && node.kind() == "call_expression"
        && let Some(function_node) = node.child_by_field_name("function")
    {
        let target = render_call_target(function_node, source);
        let (_, name) = split_call_target(&target);
        let line = node.start_position().row + 1;
        if matches!(name.as_str(), "write" | "write_all") || target.contains("File::write") {
            write_loops.push(line);
        }
        if name == "lines" {
            line_iteration_loops.push(line);
        }
        if is_default_hashmap_target(&target) {
            default_hasher_lines.push(line);
        }
    }

    if child_in_loop
        && node.kind() == "macro_invocation"
        && let Some(macro_node) = node.child_by_field_name("macro")
        && let Some(macro_text) = source.get(macro_node.byte_range())
        && matches!(macro_text.trim(), "write" | "writeln")
    {
        write_loops.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_loop_operation_lines(
            child,
            source,
            child_in_loop,
            write_loops,
            line_iteration_loops,
            default_hasher_lines,
        );
    }
}

fn is_loop_node(kind: &str) -> bool {
    matches!(
        kind,
        "for_expression" | "while_expression" | "loop_expression"
    )
}

pub(super) fn collect_boxed_container_lines(node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_boxed_vec_lines(node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_boxed_vec_lines(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if node.kind() == "let_declaration"
        && let Some(text) = source.get(node.byte_range())
        && contains_boxed_vec_type(text)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_boxed_vec_lines(child, source, lines);
    }
}

fn contains_boxed_vec_type(text: &str) -> bool {
    let Some(vec_start) = text.find("Vec<") else {
        return false;
    };
    text[vec_start + 4..].contains("Box<")
}

fn is_default_hashmap_target(target: &str) -> bool {
    target.contains("HashMap::") && (target.ends_with("::new") || target.ends_with("::default"))
}

pub(super) fn collect_unsafe_patterns(node: Node<'_>, source: &str) -> Vec<UnsafePattern> {
    let mut patterns = Vec::new();
    visit_for_unsafe_patterns(node, source, &mut patterns);
    patterns.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(left.detail.cmp(&right.detail))
    });
    patterns.dedup_by(|left, right| {
        left.line == right.line && left.kind == right.kind && left.detail == right.detail
    });
    patterns
}

fn visit_for_unsafe_patterns(node: Node<'_>, source: &str, patterns: &mut Vec<UnsafePattern>) {
    if node.kind() == "call_expression"
        && let Some(function_node) = node.child_by_field_name("function")
    {
        let target = render_call_target(function_node, source);
        let (_, name) = split_call_target(&target);
        let kind = match name.as_str() {
            "get_unchecked" | "get_unchecked_mut" => Some(UnsafePatternKind::GetUnchecked),
            "from_raw_parts" | "from_raw_parts_mut" => Some(UnsafePatternKind::RawParts),
            "set_len" => Some(UnsafePatternKind::SetLen),
            "assume_init" => Some(UnsafePatternKind::AssumeInit),
            "transmute" => Some(UnsafePatternKind::Transmute),
            _ => None,
        };

        if let Some(kind) = kind {
            patterns.push(UnsafePattern {
                line: node.start_position().row + 1,
                kind,
                detail: target,
            });
        }
    }

    if matches!(node.kind(), "cast_expression" | "type_cast_expression")
        && let Some(text) = source.get(node.byte_range())
        && (text.contains(" as *const ")
            || text.contains(" as *mut ")
            || text.ends_with(" as *const")
            || text.ends_with(" as *mut"))
    {
        patterns.push(UnsafePattern {
            line: node.start_position().row + 1,
            kind: UnsafePatternKind::RawPointerCast,
            detail: text.trim().to_string(),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_unsafe_patterns(child, source, patterns);
    }
}

pub(super) fn inside_trait_impl(node: Node<'_>, source: &str, trait_name: &str) -> bool {
    let mut current = node.parent();

    while let Some(parent) = current {
        if parent.kind() == "impl_item" && trait_impl_type(parent, source, trait_name).is_some() {
            return true;
        }
        current = parent.parent();
    }

    false
}

pub(crate) fn is_inside_function(node: Node<'_>) -> bool {
    let mut current = node.parent();

    while let Some(parent) = current {
        if parent.kind() == "function_item" {
            return true;
        }

        current = parent.parent();
    }

    false
}
