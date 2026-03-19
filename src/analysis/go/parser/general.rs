use tree_sitter::Node;

use crate::analysis::{CallSite, DeclaredSymbol, ImportSpec};
use crate::model::SymbolKind;

pub(super) fn collect_calls(body_node: Node<'_>, source: &str) -> Vec<CallSite> {
    let mut calls = Vec::new();
    visit_for_calls(body_node, source, &mut calls);
    calls
}

fn visit_for_calls(node: Node<'_>, source: &str, calls: &mut Vec<CallSite>) {
    if node.kind() == "call_expression" {
        if let Some(function_node) = node.child_by_field_name("function") {
            if let Some((receiver, name)) = extract_call_target(function_node, source) {
                calls.push(CallSite {
                    receiver,
                    name,
                    line: node.start_position().row + 1,
                });
            }
        }
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
    imports.sort_by(|left, right| left.alias.cmp(&right.alias).then(left.path.cmp(&right.path)));
    imports
}

fn visit_for_imports(node: Node<'_>, source: &str, imports: &mut Vec<ImportSpec>) {
    if node.kind() == "import_spec" {
        if let Some(import_spec) = parse_import_spec(node, source) {
            imports.push(import_spec);
        }
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

    Some(ImportSpec { alias, path })
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

pub(super) fn first_named_child_of_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
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
        line: node.start_position().row + 1,
    })
}

fn parse_method_symbol(node: Node<'_>, source: &str) -> Option<DeclaredSymbol> {
    let name_node = node.child_by_field_name("name")?;
    let receiver_type = node
        .child_by_field_name("receiver")
        .and_then(|receiver| extract_receiver_type(receiver, source));
    Some(DeclaredSymbol {
        name: source.get(name_node.byte_range())?.to_string(),
        kind: SymbolKind::Method,
        receiver_type,
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
            if package_child.kind() == "package_identifier" || package_child.kind() == "identifier" {
                return source
                    .get(package_child.byte_range())
                    .map(ToOwned::to_owned);
            }
        }
    }

    None
}

pub(super) fn extract_receiver_type(receiver_node: Node<'_>, source: &str) -> Option<String> {
    let text = source.get(receiver_node.byte_range())?;
    let sanitized = text
        .chars()
        .filter(|character| !matches!(character, '(' | ')' | '*' | ','))
        .collect::<String>();
    sanitized
        .split_whitespace()
        .last()
        .map(|receiver| receiver.to_string())
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

pub(super) fn first_string_literal(node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if matches!(child.kind(), "interpreted_string_literal" | "raw_string_literal") {
            let literal = source.get(child.byte_range())?;
            return Some(literal.trim_matches('"').trim_matches('`').to_string());
        }
    }

    None
}
