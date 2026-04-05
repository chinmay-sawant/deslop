use std::collections::BTreeSet;

use tree_sitter::Node;

use crate::analysis::{
    DeclaredSymbol, FieldSummary, ImportSpec, NamedLiteral, ParsedFunction, RustAttributeSummary,
    RustEnumSummary, RustModuleDeclaration, RustStaticSummary, StructSummary,
};
use crate::model::SymbolKind;

use super::{is_inside_function, leading_attributes, string_literal_value};

pub(super) fn collect_symbols(
    root: Node<'_>,
    source: &str,
    functions: &[ParsedFunction],
    imports: &[ImportSpec],
) -> Vec<DeclaredSymbol> {
    let mut symbols = functions
        .iter()
        .map(|function| DeclaredSymbol {
            name: function.fingerprint.name.clone(),
            kind: if function.fingerprint.kind == "method" {
                SymbolKind::Method
            } else {
                SymbolKind::Function
            },
            receiver_type: function.fingerprint.receiver_type.clone(),
            receiver_is_pointer: None,
            line: function.fingerprint.start_line,
        })
        .collect::<Vec<_>>();

    // Add re-exports as symbols to allow resolution of public imports
    for import in imports {
        if import.is_public && import.alias != "*" {
            symbols.push(DeclaredSymbol {
                name: import.alias.clone(),
                kind: SymbolKind::Function, // Treat re-exports as functions for resolution
                receiver_type: None,
                receiver_is_pointer: None,
                line: 1, // Metadata only
            });
        }
    }

    visit_for_symbols(root, source, &mut symbols);
    symbols.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    symbols
}

pub(super) fn collect_module_declarations(
    root: Node<'_>,
    source: &str,
) -> Vec<RustModuleDeclaration> {
    let mut declarations = Vec::new();
    visit_for_module_declarations(root, source, &mut declarations);
    declarations.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    declarations.dedup_by(|left, right| left.line == right.line && left.name == right.name);
    declarations
}

fn visit_for_module_declarations(
    node: Node<'_>,
    source: &str,
    declarations: &mut Vec<RustModuleDeclaration>,
) {
    if node.kind() == "mod_item"
        && !is_inside_function(node)
        && let Some(text) = source.get(node.byte_range())
        && text.trim_end().ends_with(';')
        && let Some(name_node) = node.child_by_field_name("name")
        && let Some(name) = source.get(name_node.byte_range())
    {
        declarations.push(RustModuleDeclaration {
            line: node.start_position().row + 1,
            name: name.trim().to_string(),
            path_override: module_path_override(node, source),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_module_declarations(child, source, declarations);
    }
}

fn module_path_override(node: Node<'_>, source: &str) -> Option<String> {
    for attribute in leading_attributes(node) {
        let normalized = source
            .get(attribute.byte_range())?
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let Some(path_text) = normalized.strip_prefix("#[path=\"") else {
            continue;
        };
        let Some((path, _)) = path_text.split_once('"') else {
            continue;
        };
        if !path.is_empty() {
            return Some(path.to_string());
        }
    }

    None
}

fn visit_for_symbols(node: Node<'_>, source: &str, symbols: &mut Vec<DeclaredSymbol>) {
    let symbol_kind = match node.kind() {
        "struct_item" => Some(SymbolKind::Struct),
        "enum_item" | "type_item" => Some(SymbolKind::Type),
        "trait_item" => Some(SymbolKind::Interface),
        _ => None,
    };

    if let Some(kind) = symbol_kind
        && let Some(name_node) = node.child_by_field_name("name")
        && let Some(name) = source.get(name_node.byte_range())
    {
        symbols.push(DeclaredSymbol {
            name: name.trim().to_string(),
            kind,
            receiver_type: None,
            receiver_is_pointer: None,
            line: node.start_position().row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_symbols(child, source, symbols);
    }
}

pub(super) fn collect_pkg_strings(root: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_pkg_strings(root, source, &mut literals);
    literals
}

fn visit_pkg_strings(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if matches!(node.kind(), "const_item" | "static_item")
        && let Some(literal) = named_string_from_item(node, source)
        && !is_inside_function(node)
    {
        literals.push(literal);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_pkg_strings(child, source, literals);
    }
}

fn named_string_from_item(node: Node<'_>, source: &str) -> Option<NamedLiteral> {
    let name = node
        .child_by_field_name("name")
        .and_then(|name_node| source.get(name_node.byte_range()))?
        .trim()
        .to_string();
    let value_node = node.child_by_field_name("value")?;
    let value = string_literal_value(value_node, source)?;

    Some(NamedLiteral {
        line: node.start_position().row + 1,
        name,
        value,
    })
}

pub(super) fn collect_trait_impls(
    root: Node<'_>,
    source: &str,
    trait_name: &str,
) -> BTreeSet<String> {
    let mut impls = BTreeSet::new();
    visit_trait_impls(root, source, trait_name, &mut impls);
    impls
}

fn visit_trait_impls(node: Node<'_>, source: &str, trait_name: &str, impls: &mut BTreeSet<String>) {
    if node.kind() == "impl_item"
        && let Some(type_name) = trait_impl_type(node, source, trait_name)
    {
        impls.insert(type_name);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_trait_impls(child, source, trait_name, impls);
    }
}

pub(super) fn trait_impl_type(node: Node<'_>, source: &str, trait_name: &str) -> Option<String> {
    let normalized = source
        .get(node.byte_range())?
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let prefix = format!("impl{trait_name}for");
    let remainder = normalized.strip_prefix(&prefix)?;
    let type_name = remainder
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    if type_name.is_empty() {
        None
    } else {
        Some(type_name)
    }
}

pub(super) fn collect_struct_summaries(
    root: Node<'_>,
    source: &str,
    default_impls: &BTreeSet<String>,
) -> Vec<StructSummary> {
    let mut structs = Vec::new();
    visit_for_struct_summaries(root, source, default_impls, &mut structs);
    structs.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    structs
}

fn visit_for_struct_summaries(
    node: Node<'_>,
    source: &str,
    default_impls: &BTreeSet<String>,
    structs: &mut Vec<StructSummary>,
) {
    if node.kind() == "struct_item"
        && let Some(summary) = build_struct_summary(node, source, default_impls)
    {
        structs.push(summary);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_struct_summaries(child, source, default_impls, structs);
    }
}

fn build_struct_summary(
    node: Node<'_>,
    source: &str,
    default_impls: &BTreeSet<String>,
) -> Option<StructSummary> {
    let name_node = node.child_by_field_name("name")?;
    let name = source.get(name_node.byte_range())?.trim().to_string();
    let fields = node
        .child_by_field_name("body")
        .or_else(|| named_child_by_kind(node, "field_declaration_list"))
        .map(|body| collect_struct_fields(body, source))
        .unwrap_or_default();
    let derives = parse_derive_names(&leading_attributes(node), source);
    let visibility_pub = source.get(node.byte_range()).is_some_and(|text| {
        text.trim_start().starts_with("pub ") || text.trim_start().starts_with("pub(")
    });

    Some(StructSummary {
        line: node.start_position().row + 1,
        name: name.clone(),
        fields,
        has_debug_derive: derives.iter().any(|derive| derive == "Debug"),
        has_default_derive: derives.iter().any(|derive| derive == "Default"),
        has_serialize_derive: derives.iter().any(|derive| derive == "Serialize"),
        has_deserialize_derive: derives.iter().any(|derive| derive == "Deserialize"),
        visibility_pub,
        derives,
        attributes: parse_attribute_texts(&leading_attributes(node), source),
        impl_default: default_impls.contains(&name),
    })
}

fn named_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
}

fn collect_struct_fields(node: Node<'_>, source: &str) -> Vec<FieldSummary> {
    let mut fields = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() != "field_declaration" {
            continue;
        }

        let Some(name_node) = child.child_by_field_name("name") else {
            continue;
        };
        let Some(type_node) = child.child_by_field_name("type") else {
            continue;
        };
        let name = source
            .get(name_node.byte_range())
            .unwrap_or("")
            .trim()
            .to_string();
        let type_text = source
            .get(type_node.byte_range())
            .unwrap_or("")
            .trim()
            .to_string();
        let normalized_type = type_text
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let primitive_name = normalized_type
            .trim_start_matches('&')
            .trim_start_matches("mut")
            .trim_start_matches('&');
        let is_primitive = matches!(
            primitive_name,
            "bool"
                | "str"
                | "String"
                | "usize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "isize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "f32"
                | "f64"
        );

        fields.push(FieldSummary {
            line: child.start_position().row + 1,
            name,
            attributes: parse_attribute_texts(&leading_attributes(child), source),
            is_pub: source.get(child.byte_range()).is_some_and(|text| {
                text.trim_start().starts_with("pub ") || text.trim_start().starts_with("pub(")
            }),
            is_option: normalized_type.starts_with("Option<")
                || normalized_type.contains("::Option<")
                || normalized_type.starts_with("std::option::Option<"),
            is_bool: primitive_name == "bool",
            is_primitive,
            type_text,
        });
    }
    fields
}

fn parse_derive_names(attributes: &[Node<'_>], source: &str) -> Vec<String> {
    let mut derives = Vec::new();

    for attribute in attributes {
        let Some(text) = source.get(attribute.byte_range()) else {
            continue;
        };
        let Some(start) = text.find("derive(") else {
            continue;
        };
        let Some(derive_text) = text.get(start + "derive(".len()..) else {
            continue;
        };
        let Some(end) = derive_text.find(')') else {
            continue;
        };
        for derive in derive_text.get(..end).unwrap_or("").split(',') {
            let cleaned = derive.trim().trim_matches(']');
            if cleaned.is_empty() {
                continue;
            }
            let simple = cleaned.rsplit("::").next().unwrap_or(cleaned).to_string();
            derives.push(simple);
        }
    }

    derives.sort();
    derives.dedup();
    derives
}

fn parse_attribute_texts(attributes: &[Node<'_>], source: &str) -> Vec<String> {
    let mut parsed = Vec::new();

    for attribute in attributes {
        let Some(text) = source.get(attribute.byte_range()) else {
            continue;
        };
        let normalized = text
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        if !normalized.is_empty() {
            parsed.push(normalized);
        }
    }

    parsed.sort();
    parsed.dedup();
    parsed
}

pub(super) fn collect_static_summaries(root: Node<'_>, source: &str) -> Vec<RustStaticSummary> {
    let mut statics = Vec::new();
    visit_for_static_summaries(root, source, &mut statics);
    statics.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    statics
}

fn visit_for_static_summaries(node: Node<'_>, source: &str, statics: &mut Vec<RustStaticSummary>) {
    if node.kind() == "static_item"
        && let Some(summary) = build_static_summary(node, source)
    {
        statics.push(summary);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_static_summaries(child, source, statics);
    }
}

fn build_static_summary(node: Node<'_>, source: &str) -> Option<RustStaticSummary> {
    let name = node
        .child_by_field_name("name")
        .and_then(|name_node| source.get(name_node.byte_range()))?
        .trim()
        .to_string();
    let type_text = node
        .child_by_field_name("type")
        .and_then(|type_node| source.get(type_node.byte_range()))?
        .trim()
        .to_string();
    let value_text = node
        .child_by_field_name("value")
        .and_then(|value_node| source.get(value_node.byte_range()))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    Some(RustStaticSummary {
        line: node.start_position().row + 1,
        name,
        type_text,
        value_text,
        visibility_pub: source.get(node.byte_range()).is_some_and(|text| {
            text.trim_start().starts_with("pub ") || text.trim_start().starts_with("pub(")
        }),
        is_mut: source
            .get(node.byte_range())
            .is_some_and(|text| text.contains("static mut ")),
    })
}

pub(super) fn collect_attribute_summaries(
    root: Node<'_>,
    source: &str,
) -> Vec<RustAttributeSummary> {
    let mut attributes = Vec::new();
    visit_for_attribute_summaries(root, source, &mut attributes);
    attributes.sort_by(|left, right| left.line.cmp(&right.line).then(left.text.cmp(&right.text)));
    attributes.dedup_by(|left, right| left.line == right.line && left.text == right.text);
    attributes
}

fn visit_for_attribute_summaries(
    node: Node<'_>,
    source: &str,
    attributes: &mut Vec<RustAttributeSummary>,
) {
    if matches!(node.kind(), "attribute_item" | "inner_attribute_item")
        && let Some(text) = source.get(node.byte_range())
    {
        let normalized = text
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        if !normalized.is_empty() {
            attributes.push(RustAttributeSummary {
                line: node.start_position().row + 1,
                text: normalized,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_attribute_summaries(child, source, attributes);
    }
}

pub(super) fn collect_enum_summaries(root: Node<'_>, source: &str) -> Vec<RustEnumSummary> {
    let mut enums = Vec::new();
    visit_for_enum_summaries(root, source, &mut enums);
    enums.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    enums
}

fn visit_for_enum_summaries(node: Node<'_>, source: &str, enums: &mut Vec<RustEnumSummary>) {
    if node.kind() == "enum_item"
        && let Some(summary) = build_enum_summary(node, source)
    {
        enums.push(summary);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_enum_summaries(child, source, enums);
    }
}

fn build_enum_summary(node: Node<'_>, source: &str) -> Option<RustEnumSummary> {
    let name = node
        .child_by_field_name("name")
        .and_then(|name_node| source.get(name_node.byte_range()))?
        .trim()
        .to_string();
    let derives = parse_derive_names(&leading_attributes(node), source);
    let attributes = parse_attribute_texts(&leading_attributes(node), source);
    let variant_count = node
        .child_by_field_name("body")
        .or_else(|| named_child_by_kind(node, "enum_variant_list"))
        .map(|body| {
            let mut cursor = body.walk();
            body.named_children(&mut cursor)
                .filter(|child| child.kind() == "enum_variant")
                .count()
        })
        .unwrap_or(0);

    Some(RustEnumSummary {
        line: node.start_position().row + 1,
        name,
        variant_count,
        has_serialize_derive: derives.iter().any(|derive| derive == "Serialize"),
        has_deserialize_derive: derives.iter().any(|derive| derive == "Deserialize"),
        derives,
        attributes,
        visibility_pub: source.get(node.byte_range()).is_some_and(|text| {
            text.trim_start().starts_with("pub ") || text.trim_start().starts_with("pub(")
        }),
    })
}
