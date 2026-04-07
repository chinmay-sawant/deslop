use tree_sitter::Node;

use crate::analysis::NamedLiteral;

use super::{is_inside_function, string_literal_value};

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
