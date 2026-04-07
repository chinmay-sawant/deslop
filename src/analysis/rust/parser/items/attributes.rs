use tree_sitter::Node;

use crate::analysis::RustAttributeSummary;

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
