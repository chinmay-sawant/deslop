use tree_sitter::Node;

use crate::analysis::{CallSite, NamedLiteral, StructTag, TestFunctionSummary};

use super::symbols::{
    collect_expression_nodes, collect_identifiers, find_var_name_node, find_var_value_node,
    is_identifier_name, split_assignment,
};

pub(crate) fn collect_pkg_strings(root: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_pkg_strings(root, source, &mut literals);
    literals.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    literals
}

fn visit_pkg_strings(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if matches!(node.kind(), "var_spec" | "const_spec") && is_package_scope(node) {
        literals.extend(extract_named_strings(node, source));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_pkg_strings(child, source, literals);
    }
}

pub(crate) fn collect_local_strings(body_node: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_local_strings(body_node, source, &mut literals);
    literals.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    literals
}

fn visit_local_strings(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if matches!(
        node.kind(),
        "var_spec" | "const_spec" | "assignment_statement" | "short_var_declaration"
    ) {
        literals.extend(extract_named_strings(node, source));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_local_strings(child, source, literals);
    }
}

fn extract_named_strings(node: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let Some(name_node) = find_var_name_node(node) else {
        return fallback_named_string(node, source).into_iter().collect();
    };
    let Some(value_node) = find_var_value_node(node) else {
        return fallback_named_string(node, source).into_iter().collect();
    };

    let names = collect_identifiers(name_node, source);
    let values = collect_expression_nodes(value_node);

    let literals = names
        .into_iter()
        .enumerate()
        .filter_map(|(index, (name, line))| {
            let value_node = values.get(index)?;
            let value = extract_string(*value_node, source)?;
            Some(NamedLiteral { line, name, value })
        })
        .collect::<Vec<_>>();

    if literals.is_empty() {
        fallback_named_string(node, source).into_iter().collect()
    } else {
        literals
    }
}

fn extract_string(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "interpreted_string_literal" | "raw_string_literal" => source
            .get(node.byte_range())
            .map(|literal| literal.trim_matches('"').trim_matches('`').to_string()),
        _ => None,
    }
}

fn fallback_named_string(node: Node<'_>, source: &str) -> Option<NamedLiteral> {
    let text = source.get(node.byte_range())?;
    let (left, right) = split_assignment(text)?;
    let name = left.trim().split(',').next()?.trim();
    if !is_identifier_name(name) {
        return None;
    }

    let value = right.trim();
    let value = value
        .strip_prefix('"')
        .and_then(|trimmed| trimmed.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('`')
                .and_then(|trimmed| trimmed.strip_suffix('`'))
        })?;

    Some(NamedLiteral {
        line: node.start_position().row + 1,
        name: name.to_string(),
        value: value.to_string(),
    })
}

pub(crate) fn collect_struct_tags(root: Node<'_>, source: &str) -> Vec<StructTag> {
    let mut tags = Vec::new();
    visit_struct_tags(root, source, &mut tags);
    tags.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(left.field_name.cmp(&right.field_name))
    });
    tags
}

fn visit_struct_tags(node: Node<'_>, source: &str, tags: &mut Vec<StructTag>) {
    if node.kind() == "type_spec" {
        tags.extend(extract_struct_tags(node, source));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_struct_tags(child, source, tags);
    }
}

fn extract_struct_tags(node: Node<'_>, source: &str) -> Vec<StructTag> {
    let Some(name_node) = node.child_by_field_name("name") else {
        return Vec::new();
    };
    let Some(type_node) = node.child_by_field_name("type") else {
        return Vec::new();
    };
    if type_node.kind() != "struct_type" {
        return Vec::new();
    }

    let struct_name = source.get(name_node.byte_range()).unwrap_or("").to_string();
    let Some(struct_text) = source.get(type_node.byte_range()) else {
        return Vec::new();
    };

    let mut tags = Vec::new();
    let base_line = type_node.start_position().row + 1;

    for (offset, line) in struct_text.split('\n').enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "struct{" || trimmed == "struct {" || trimmed == "}" {
            continue;
        }
        let Some((_, rest)) = trimmed.split_once('`') else {
            continue;
        };
        let Some((raw_tag, _)) = rest.split_once('`') else {
            continue;
        };

        let field_name = trimmed
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches(',')
            .to_string();
        if field_name.is_empty() {
            continue;
        }

        tags.push(StructTag {
            line: base_line + offset,
            struct_name: struct_name.to_string(),
            field_name,
            raw_tag: raw_tag.to_string(),
        });
    }

    tags
}

pub(crate) fn build_test_summary(
    function_name: &str,
    body_node: Node<'_>,
    source: &str,
    calls: &[CallSite],
    is_test_file: bool,
) -> Option<TestFunctionSummary> {
    if !is_test_file || !function_name.starts_with("Test") {
        return None;
    }

    let body_text = source.get(body_node.byte_range()).unwrap_or("");
    let assertion_like_calls = calls.iter().filter(|call| is_assert_call(call)).count();
    let error_assertion_calls = calls.iter().filter(|call| is_err_assert_call(call)).count()
        + usize::from(body_text.contains("err == nil") || body_text.contains("err==nil"));
    let skip_calls = calls.iter().filter(|call| is_skip_call(call)).count();
    let production_calls = calls
        .iter()
        .filter(|call| !is_assert_call(call) && !is_test_infra_call(call))
        .count();
    let normalized = body_text.to_ascii_lowercase();

    Some(TestFunctionSummary {
        assertion_like_calls,
        error_assertion_calls,
        skip_calls,
        production_calls,
        has_todo_marker: normalized.contains("todo") || normalized.contains("fixme"),
    })
}

fn is_assert_call(call: &CallSite) -> bool {
    matches!(
        (call.receiver.as_deref(), call.name.as_str()),
        (
            Some("t"),
            "Error" | "Errorf" | "Fatal" | "Fatalf" | "Fail" | "FailNow"
        ) | (Some("assert" | "require"), _)
    )
}

fn is_err_assert_call(call: &CallSite) -> bool {
    matches!(
        (call.receiver.as_deref(), call.name.as_str()),
        (
            Some("assert" | "require"),
            "Error" | "Errorf" | "ErrorIs" | "ErrorContains" | "Panics" | "PanicsWithValue"
        ) | (Some("t"), "Fatal" | "Fatalf" | "Error" | "Errorf")
    )
}

fn is_skip_call(call: &CallSite) -> bool {
    matches!(
        (call.receiver.as_deref(), call.name.as_str()),
        (Some("t"), "Skip" | "SkipNow" | "Skipf")
    )
}

fn is_test_infra_call(call: &CallSite) -> bool {
    matches!(
        (call.receiver.as_deref(), call.name.as_str()),
        (
            Some("t"),
            "Helper"
                | "Run"
                | "Parallel"
                | "Cleanup"
                | "TempDir"
                | "Setenv"
                | "Skip"
                | "SkipNow"
                | "Skipf"
                | "Log"
                | "Logf"
        ) | (Some("assert" | "require"), _)
    )
}

pub(crate) fn first_string_literal(node: Node<'_>, source: &str) -> Option<String> {
    if matches!(
        node.kind(),
        "interpreted_string_literal" | "raw_string_literal"
    ) {
        let literal = source.get(node.byte_range())?;
        return Some(literal.trim_matches('"').trim_matches('`').to_string());
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if matches!(
            child.kind(),
            "interpreted_string_literal" | "raw_string_literal"
        ) {
            let literal = source.get(child.byte_range())?;
            return Some(literal.trim_matches('"').trim_matches('`').to_string());
        }
    }

    None
}

fn is_package_scope(node: Node<'_>) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "function_declaration" | "method_declaration" | "func_literal" => return false,
            "source_file" => return true,
            _ => current = parent.parent(),
        }
    }

    false
}
