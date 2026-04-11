use tree_sitter::Node;

use crate::analysis::RustIncludeDeclaration;

use super::{is_inside_function, string_literal_value};

pub(super) fn collect_include_declarations(
    root: Node<'_>,
    source: &str,
) -> Vec<RustIncludeDeclaration> {
    let mut declarations = Vec::new();
    visit_for_include_declarations(root, source, &mut declarations);
    declarations.sort_by(|left, right| left.line.cmp(&right.line).then(left.path.cmp(&right.path)));
    declarations.dedup_by(|left, right| left.line == right.line && left.path == right.path);
    declarations
}

fn visit_for_include_declarations(
    node: Node<'_>,
    source: &str,
    declarations: &mut Vec<RustIncludeDeclaration>,
) {
    if node.kind() == "macro_invocation"
        && !is_inside_function(node)
        && macro_name(node, source).as_deref() == Some("include")
        && let Some(path) = include_path(node, source)
    {
        declarations.push(RustIncludeDeclaration {
            line: node.start_position().row + 1,
            path,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_include_declarations(child, source, declarations);
    }
}

fn macro_name(node: Node<'_>, source: &str) -> Option<String> {
    let macro_node = node.child_by_field_name("macro")?;
    let raw = source.get(macro_node.byte_range())?.trim();
    raw.rsplit("::").next().map(ToOwned::to_owned)
}

fn include_path(node: Node<'_>, source: &str) -> Option<String> {
    let token_tree = node
        .child_by_field_name("token_tree")
        .or_else(|| named_child_by_kind(node, "token_tree"))?;
    find_first_string_literal(token_tree, source)
}

fn find_first_string_literal(node: Node<'_>, source: &str) -> Option<String> {
    if let Some(value) = string_literal_value(node, source) {
        return Some(value);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Some(value) = find_first_string_literal(child, source) {
            return Some(value);
        }
    }

    None
}

fn named_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
}
