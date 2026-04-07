use tree_sitter::Node;

use crate::analysis::{
    DeclaredSymbol, GoFieldSummary, GoStructSummary, InterfaceSummary, PackageVarSummary,
};
use crate::model::SymbolKind;

pub(crate) fn collect_symbols(root: Node<'_>, source: &str) -> Vec<DeclaredSymbol> {
    let mut symbols = Vec::new();
    visit_symbols(root, source, &mut symbols);
    symbols.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    symbols
}

pub(crate) fn collect_package_vars(root: Node<'_>, source: &str) -> Vec<PackageVarSummary> {
    let mut vars = Vec::new();
    visit_package_vars(root, source, &mut vars);
    vars.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    vars
}

fn visit_package_vars(node: Node<'_>, source: &str, vars: &mut Vec<PackageVarSummary>) {
    if node.kind() == "var_spec" && is_package_scope(node) {
        vars.extend(extract_package_vars(node, source));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_package_vars(child, source, vars);
    }
}

fn extract_package_vars(node: Node<'_>, source: &str) -> Vec<PackageVarSummary> {
    let Some(name_node) = find_var_name_node(node) else {
        return Vec::new();
    };
    let names = collect_identifiers(name_node, source);
    if names.is_empty() {
        return Vec::new();
    }

    let type_text = node
        .child_by_field_name("type")
        .and_then(|type_node| source.get(type_node.byte_range()))
        .map(|text| text.trim().to_string());
    let values = find_var_value_node(node)
        .map(collect_expression_nodes)
        .unwrap_or_default();

    names
        .into_iter()
        .enumerate()
        .map(|(index, (name, line))| PackageVarSummary {
            is_pub: is_exported_name(&name),
            line,
            name,
            type_text: type_text.clone(),
            value_text: values
                .get(index)
                .and_then(|value_node| source.get(value_node.byte_range()))
                .map(|text| text.trim().to_string()),
        })
        .collect()
}

pub(crate) fn collect_interface_summaries(root: Node<'_>, source: &str) -> Vec<InterfaceSummary> {
    let mut summaries = Vec::new();
    visit_interface_summaries(root, source, &mut summaries);
    summaries.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    summaries
}

fn visit_interface_summaries(node: Node<'_>, source: &str, summaries: &mut Vec<InterfaceSummary>) {
    if node.kind() == "type_spec"
        && let Some(summary) = extract_interface_summary(node, source)
    {
        summaries.push(summary);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_interface_summaries(child, source, summaries);
    }
}

fn extract_interface_summary(node: Node<'_>, source: &str) -> Option<InterfaceSummary> {
    let name_node = node.child_by_field_name("name")?;
    let type_node = node.child_by_field_name("type")?;
    if type_node.kind() != "interface_type" {
        return None;
    }

    let name = source.get(name_node.byte_range())?.to_string();
    let text = source.get(type_node.byte_range())?;
    let methods = text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed == "interface{"
                || trimmed == "interface {"
                || trimmed == "}"
            {
                return None;
            }
            let name = trimmed.split('(').next()?.trim();
            is_identifier_name(name).then(|| name.to_string())
        })
        .collect::<Vec<_>>();

    Some(InterfaceSummary {
        name: name.clone(),
        line: node.start_position().row + 1,
        is_pub: is_exported_name(&name),
        methods,
    })
}

pub(crate) fn collect_go_structs(root: Node<'_>, source: &str) -> Vec<GoStructSummary> {
    let mut structs = Vec::new();
    visit_go_structs(root, source, &mut structs);
    structs.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    structs
}

fn visit_go_structs(node: Node<'_>, source: &str, structs: &mut Vec<GoStructSummary>) {
    if node.kind() == "type_spec"
        && let Some(summary) = extract_go_struct_summary(node, source)
    {
        structs.push(summary);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_go_structs(child, source, structs);
    }
}

fn extract_go_struct_summary(node: Node<'_>, source: &str) -> Option<GoStructSummary> {
    let name_node = node.child_by_field_name("name")?;
    let type_node = node.child_by_field_name("type")?;
    if type_node.kind() != "struct_type" {
        return None;
    }

    let name = source.get(name_node.byte_range())?.to_string();
    let struct_text = source.get(type_node.byte_range())?;
    let base_line = type_node.start_position().row + 1;
    let mut fields = Vec::new();

    for (offset, line) in struct_text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "struct{" || trimmed == "struct {" || trimmed == "}" {
            continue;
        }

        let without_tag = trimmed.split('`').next().unwrap_or(trimmed).trim();
        let mut parts = without_tag.split_whitespace();
        let Some(first) = parts.next() else {
            continue;
        };
        let Some(second) = parts.next() else {
            continue;
        };
        let remaining_parts = parts.collect::<Vec<_>>();

        if first == "//" {
            continue;
        }

        for field_name in first
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            fields.push(GoFieldSummary {
                name: field_name.to_string(),
                line: base_line + offset,
                type_text: std::iter::once(second)
                    .chain(remaining_parts.iter().copied())
                    .collect::<Vec<_>>()
                    .join(" "),
                is_pub: is_exported_name(field_name),
            });
        }
    }

    Some(GoStructSummary {
        name: name.clone(),
        line: node.start_position().row + 1,
        is_pub: is_exported_name(&name),
        fields,
    })
}

fn visit_symbols(node: Node<'_>, source: &str, symbols: &mut Vec<DeclaredSymbol>) {
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
        "var_spec" => symbols.extend(parse_pkg_var_symbols(node, source)),
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_symbols(child, source, symbols);
    }
}

fn parse_pkg_var_symbols(node: Node<'_>, source: &str) -> Vec<DeclaredSymbol> {
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

pub(crate) fn find_var_name_node(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("name")
        .or_else(|| first_named_child(node, "identifier_list"))
        .or_else(|| first_named_child(node, "identifier"))
}

pub(crate) fn find_var_value_node(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("value")
        .or_else(|| first_named_child(node, "expression_list"))
        .or_else(|| {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find(|child| is_expression_kind(child.kind()))
        })
}

pub(crate) fn first_named_child<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
}

pub(crate) fn collect_identifiers(node: Node<'_>, source: &str) -> Vec<(String, usize)> {
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

pub(crate) fn collect_expression_nodes(node: Node<'_>) -> Vec<Node<'_>> {
    if node.kind() != "expression_list" {
        return vec![node];
    }

    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

pub(crate) fn is_callable_var_value(node: Node<'_>) -> bool {
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

pub(crate) fn is_expression_kind(kind: &str) -> bool {
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
        .and_then(|receiver| extract_receiver(receiver, source))?;
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

pub(crate) fn find_package_name(root: Node<'_>, source: &str) -> Option<String> {
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

pub(crate) fn extract_receiver(receiver_node: Node<'_>, source: &str) -> Option<(String, bool)> {
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

pub(crate) fn count_descendants(node: Node<'_>, kind: &str) -> usize {
    let mut total = usize::from(node.kind() == kind);
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        total += count_descendants(child, kind);
    }
    total
}

pub(crate) fn split_assignment(text: &str) -> Option<(&str, &str)> {
    text.split_once(":=").or_else(|| text.split_once('='))
}

pub(crate) fn is_identifier_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && text
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
}

fn is_exported_name(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
}
