use tree_sitter::Node;

use crate::analysis::RustStaticSummary;

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
