use tree_sitter::Node;

use crate::analysis::{
    CallSite, DeclaredSymbol, ImportSpec, NamedLiteral, StructTag, TestFunctionSummary,
};
use crate::model::SymbolKind;

pub(super) fn collect_calls(body_node: Node<'_>, source: &str) -> Vec<CallSite> {
    let mut calls = Vec::new();
    visit_for_calls(body_node, source, &mut calls);
    calls
}

fn visit_for_calls(node: Node<'_>, source: &str, calls: &mut Vec<CallSite>) {
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
        visit_for_calls(child, source, calls);
    }
}

pub(super) fn extract_call_target(
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

pub(super) fn collect_imports(root: Node<'_>, source: &str) -> Vec<ImportSpec> {
    let mut imports = Vec::new();
    visit_for_imports(root, source, &mut imports);
    imports.sort_by(|left, right| {
        left.alias
            .cmp(&right.alias)
            .then(left.path.cmp(&right.path))
    });
    imports
}

fn visit_for_imports(node: Node<'_>, source: &str, imports: &mut Vec<ImportSpec>) {
    if node.kind() == "import_spec"
        && let Some(import_spec) = parse_import_spec(node, source)
    {
        imports.push(import_spec);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_imports(child, source, imports);
    }
}

fn parse_import_spec(node: Node<'_>, source: &str) -> Option<ImportSpec> {
    let text = source.get(node.byte_range())?.trim();
    let mut parts = text.split_whitespace().collect::<Vec<_>>();
    let path_literal = parts.pop()?;
    let path = path_literal.trim_matches('"').to_string();
    let alias = parts
        .first()
        .map(|alias| alias.to_string())
        .unwrap_or_else(|| package_alias_from_import_path(&path));

    Some(ImportSpec {
        alias,
        path,
        namespace_path: None,
        imported_name: None,
    })
}

pub(super) fn collect_symbols(root: Node<'_>, source: &str) -> Vec<DeclaredSymbol> {
    let mut symbols = Vec::new();
    visit_for_symbols(root, source, &mut symbols);
    symbols.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    symbols
}

fn visit_for_symbols(node: Node<'_>, source: &str, symbols: &mut Vec<DeclaredSymbol>) {
    match node.kind() {
        "function_declaration" => {
            if let Some(symbol) = parse_function_symbol(node, source) {
                symbols.push(symbol);
            }
        }
        "method_declaration" => {
            if let Some(symbol) = parse_method_symbol(node, source) {
                symbols.push(symbol);
            }
        }
        "type_spec" => {
            if let Some(symbol) = parse_type_symbol(node, source) {
                symbols.push(symbol);
            }
        }
        "var_spec" => symbols.extend(parse_package_var_symbols(node, source)),
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_symbols(child, source, symbols);
    }
}

fn parse_package_var_symbols(node: Node<'_>, source: &str) -> Vec<DeclaredSymbol> {
    if !is_package_scope(node) {
        return Vec::new();
    }

    let Some(name_node) = find_var_name_node(node) else {
        return Vec::new();
    };
    let names = collect_identifiers(name_node, source);
    if names.is_empty() {
        return Vec::new();
    }

    let is_function_typed = node
        .child_by_field_name("type")
        .is_some_and(|type_node| type_node.kind() == "function_type");

    if is_function_typed {
        return names
            .into_iter()
            .map(|(name, line)| DeclaredSymbol {
                name,
                kind: SymbolKind::Function,
                receiver_type: None,
                receiver_is_pointer: None,
                line,
            })
            .collect();
    }

    let Some(value_node) = find_var_value_node(node) else {
        return Vec::new();
    };
    let values = collect_expression_nodes(value_node);

    names
        .into_iter()
        .enumerate()
        .filter_map(|(index, (name, line))| {
            let value = values.get(index)?;
            is_callable_var_value(*value).then_some(DeclaredSymbol {
                name,
                kind: SymbolKind::Function,
                receiver_type: None,
                receiver_is_pointer: None,
                line,
            })
        })
        .collect()
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

pub(super) fn find_var_name_node(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("name")
        .or_else(|| first_named_child_of_kind(node, "identifier_list"))
        .or_else(|| first_named_child_of_kind(node, "identifier"))
}

pub(super) fn find_var_value_node(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("value")
        .or_else(|| first_named_child_of_kind(node, "expression_list"))
        .or_else(|| {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find(|child| is_expression_node_kind(child.kind()))
        })
}

pub(super) fn first_named_child_of_kind<'tree>(
    node: Node<'tree>,
    kind: &str,
) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
}

pub(super) fn collect_identifiers(node: Node<'_>, source: &str) -> Vec<(String, usize)> {
    if node.kind() == "identifier" {
        return source
            .get(node.byte_range())
            .map(|name| vec![(name.to_string(), node.start_position().row + 1)])
            .unwrap_or_default();
    }

    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .filter(|child| child.kind() == "identifier")
        .filter_map(|child| {
            source
                .get(child.byte_range())
                .map(|name| (name.to_string(), child.start_position().row + 1))
        })
        .collect()
}

pub(super) fn collect_expression_nodes(node: Node<'_>) -> Vec<Node<'_>> {
    if node.kind() != "expression_list" {
        return vec![node];
    }

    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

pub(super) fn is_callable_var_value(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "identifier"
            | "selector_expression"
            | "func_literal"
            | "parenthesized_expression"
            | "index_expression"
            | "slice_expression"
    )
}

pub(super) fn is_expression_node_kind(kind: &str) -> bool {
    matches!(
        kind,
        "identifier"
            | "selector_expression"
            | "func_literal"
            | "parenthesized_expression"
            | "call_expression"
            | "unary_expression"
            | "binary_expression"
            | "index_expression"
            | "slice_expression"
            | "type_assertion_expression"
            | "composite_literal"
            | "literal_value"
            | "int_literal"
            | "float_literal"
            | "imaginary_literal"
            | "rune_literal"
            | "raw_string_literal"
            | "interpreted_string_literal"
    )
}

fn parse_function_symbol(node: Node<'_>, source: &str) -> Option<DeclaredSymbol> {
    let name_node = node.child_by_field_name("name")?;
    Some(DeclaredSymbol {
        name: source.get(name_node.byte_range())?.to_string(),
        kind: SymbolKind::Function,
        receiver_type: None,
        receiver_is_pointer: None,
        line: node.start_position().row + 1,
    })
}

fn parse_method_symbol(node: Node<'_>, source: &str) -> Option<DeclaredSymbol> {
    let name_node = node.child_by_field_name("name")?;
    let (receiver_type, receiver_is_pointer) = node
        .child_by_field_name("receiver")
        .and_then(|receiver| extract_receiver_details(receiver, source))?;
    Some(DeclaredSymbol {
        name: source.get(name_node.byte_range())?.to_string(),
        kind: SymbolKind::Method,
        receiver_type: Some(receiver_type),
        receiver_is_pointer: Some(receiver_is_pointer),
        line: node.start_position().row + 1,
    })
}

fn parse_type_symbol(node: Node<'_>, source: &str) -> Option<DeclaredSymbol> {
    let name_node = node.child_by_field_name("name")?;
    let type_node = node.child_by_field_name("type")?;
    let kind = match type_node.kind() {
        "struct_type" => SymbolKind::Struct,
        "interface_type" => SymbolKind::Interface,
        _ => SymbolKind::Type,
    };

    Some(DeclaredSymbol {
        name: source.get(name_node.byte_range())?.to_string(),
        kind,
        receiver_type: None,
        receiver_is_pointer: None,
        line: node.start_position().row + 1,
    })
}

pub(super) fn find_package_name(root: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        if child.kind() != "package_clause" {
            continue;
        }

        let mut package_cursor = child.walk();
        for package_child in child.named_children(&mut package_cursor) {
            if package_child.kind() == "package_identifier" || package_child.kind() == "identifier"
            {
                return source
                    .get(package_child.byte_range())
                    .map(ToOwned::to_owned);
            }
        }
    }

    None
}

pub(super) fn extract_receiver_type(receiver_node: Node<'_>, source: &str) -> Option<String> {
    extract_receiver_details(receiver_node, source).map(|(receiver, _)| receiver)
}

pub(super) fn extract_receiver_details(
    receiver_node: Node<'_>,
    source: &str,
) -> Option<(String, bool)> {
    let text = source.get(receiver_node.byte_range())?;
    let receiver_is_pointer = text.contains('*');
    let sanitized = text
        .chars()
        .filter(|character| !matches!(character, '(' | ')' | '*' | ','))
        .collect::<String>();
    sanitized
        .split_whitespace()
        .last()
        .map(|receiver| (receiver.to_string(), receiver_is_pointer))
}

pub(super) fn package_alias_from_import_path(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

pub(super) fn count_descendants(node: Node<'_>, kind: &str) -> usize {
    let mut total = usize::from(node.kind() == kind);
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        total += count_descendants(child, kind);
    }
    total
}

pub(super) fn split_assignment(text: &str) -> Option<(&str, &str)> {
    text.split_once(":=").or_else(|| text.split_once('='))
}

pub(super) fn is_identifier_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && text
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
}

pub(super) fn collect_package_string_literals(root: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_for_package_string_literals(root, source, &mut literals);
    literals.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    literals
}

fn visit_for_package_string_literals(
    node: Node<'_>,
    source: &str,
    literals: &mut Vec<NamedLiteral>,
) {
    if matches!(node.kind(), "var_spec" | "const_spec") && is_package_scope(node) {
        literals.extend(extract_named_string_literals(node, source));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_package_string_literals(child, source, literals);
    }
}

pub(super) fn collect_local_string_literals(
    body_node: Node<'_>,
    source: &str,
) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_for_local_string_literals(body_node, source, &mut literals);
    literals.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    literals
}

fn visit_for_local_string_literals(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if matches!(
        node.kind(),
        "var_spec" | "const_spec" | "assignment_statement" | "short_var_declaration"
    ) {
        literals.extend(extract_named_string_literals(node, source));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_local_string_literals(child, source, literals);
    }
}

fn extract_named_string_literals(node: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let Some(name_node) = find_var_name_node(node) else {
        return fallback_named_string_literal(node, source)
            .into_iter()
            .collect();
    };
    let Some(value_node) = find_var_value_node(node) else {
        return fallback_named_string_literal(node, source)
            .into_iter()
            .collect();
    };

    let names = collect_identifiers(name_node, source);
    let values = collect_expression_nodes(value_node);

    let literals = names
        .into_iter()
        .enumerate()
        .filter_map(|(index, (name, line))| {
            let value_node = values.get(index)?;
            let value = extract_string_literal(*value_node, source)?;
            Some(NamedLiteral { line, name, value })
        })
        .collect::<Vec<_>>();

    if literals.is_empty() {
        fallback_named_string_literal(node, source)
            .into_iter()
            .collect()
    } else {
        literals
    }
}

fn extract_string_literal(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "interpreted_string_literal" | "raw_string_literal" => source
            .get(node.byte_range())
            .map(|literal| literal.trim_matches('"').trim_matches('`').to_string()),
        _ => None,
    }
}

fn fallback_named_string_literal(node: Node<'_>, source: &str) -> Option<NamedLiteral> {
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

pub(super) fn collect_struct_tags(root: Node<'_>, source: &str) -> Vec<StructTag> {
    let mut tags = Vec::new();
    visit_for_struct_tags(root, source, &mut tags);
    tags.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(left.field_name.cmp(&right.field_name))
    });
    tags
}

fn visit_for_struct_tags(node: Node<'_>, source: &str, tags: &mut Vec<StructTag>) {
    if node.kind() == "type_spec" {
        tags.extend(extract_struct_tags(node, source));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_struct_tags(child, source, tags);
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

    for (offset, line) in struct_text.lines().enumerate() {
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

pub(super) fn build_test_function_summary(
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
    let assertion_like_calls = calls
        .iter()
        .filter(|call| is_assertion_like_call(call))
        .count();
    let error_assertion_calls = calls
        .iter()
        .filter(|call| is_error_assertion_call(call))
        .count()
        + usize::from(body_text.contains("err == nil") || body_text.contains("err==nil"));
    let skip_calls = calls.iter().filter(|call| is_skip_call(call)).count();
    let production_calls = calls
        .iter()
        .filter(|call| !is_assertion_like_call(call) && !is_test_infra_call(call))
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

fn is_assertion_like_call(call: &CallSite) -> bool {
    matches!(
        (call.receiver.as_deref(), call.name.as_str()),
        (
            Some("t"),
            "Error" | "Errorf" | "Fatal" | "Fatalf" | "Fail" | "FailNow"
        ) | (Some("assert" | "require"), _)
    )
}

fn is_error_assertion_call(call: &CallSite) -> bool {
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

pub(super) fn first_string_literal(node: Node<'_>, source: &str) -> Option<String> {
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
