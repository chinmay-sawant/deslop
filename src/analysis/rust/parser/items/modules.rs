use tree_sitter::Node;

use crate::analysis::RustModuleDeclaration;

use super::{is_inside_function, module_path_override};

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
