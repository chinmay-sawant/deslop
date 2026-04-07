use tree_sitter::Node;

use crate::analysis::ImportSpec;

pub(crate) fn collect_imports(root: Node<'_>, source: &str) -> Vec<ImportSpec> {
    let mut imports = Vec::new();
    visit_imports(root, source, &mut imports);
    imports
}

fn visit_imports(node: Node<'_>, source: &str, imports: &mut Vec<ImportSpec>) {
    if node.kind() == "import_spec"
        && let Some(import_spec) = parse_import_spec(node, source)
    {
        imports.push(import_spec);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_imports(child, source, imports);
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
        .unwrap_or_else(|| alias_from_path(&path));

    Some(ImportSpec {
        line: node.start_position().row + 1,
        group_line: node
            .parent()
            .map_or(node.start_position().row + 1, |parent| {
                parent.start_position().row + 1
            }),
        alias,
        path,
        namespace_path: None,
        imported_name: None,
        is_public: false,
    })
}

pub(crate) fn alias_from_path(path: &str) -> String {
    let mut segments = path.rsplit('/');
    let last = segments.next().unwrap_or(path);
    if is_go_module_major_version(last) {
        return segments.next().unwrap_or(last).to_string();
    }
    last.to_string()
}

fn is_go_module_major_version(segment: &str) -> bool {
    let mut characters = segment.chars();
    matches!(characters.next(), Some('v'))
        && characters.next().is_some()
        && characters.all(|character| character.is_ascii_digit())
}
