use tree_sitter::Node;

use crate::analysis::RustEnumSummary;

use super::{leading_attributes, named_child_by_kind, parse_attribute_texts, parse_derive_names};

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
