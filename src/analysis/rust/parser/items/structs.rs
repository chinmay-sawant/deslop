use std::collections::BTreeSet;

use tree_sitter::Node;

use crate::analysis::{FieldSummary, StructSummary};

use super::{leading_attributes, named_child_by_kind, parse_attribute_texts, parse_derive_names};

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
