use tree_sitter::Node;

use crate::analysis::ImportSpec;

pub(super) fn collect_imports(root: Node<'_>, source: &str) -> Vec<ImportSpec> {
    let mut imports = Vec::new();
    visit_for_imports(root, source, &mut imports);
    imports
}

fn visit_for_imports(node: Node<'_>, source: &str, imports: &mut Vec<ImportSpec>) {
    if node.kind() == "use_declaration"
        && let Some(argument) = node.child_by_field_name("argument")
    {
        let is_public = source
            .get(node.byte_range())
            .is_some_and(|text| text.starts_with("pub"));
        flatten_use_tree(argument, source, None, imports, is_public);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_imports(child, source, imports);
    }
}

fn flatten_use_tree(
    node: Node<'_>,
    source: &str,
    prefix: Option<String>,
    imports: &mut Vec<ImportSpec>,
    is_public: bool,
) {
    match node.kind() {
        "use_as_clause" => {
            let Some(path_node) = node.child_by_field_name("path") else {
                return;
            };
            let alias = node
                .child_by_field_name("alias")
                .and_then(|alias_node| source.get(alias_node.byte_range()))
                .map(str::trim)
                .filter(|alias| !alias.is_empty())
                .unwrap_or("*")
                .to_string();
            let path = combine_path_prefix(prefix.as_deref(), &render_use_path(path_node, source));

            imports.push(build_rust_import_spec(
                alias,
                path,
                is_public,
                node.start_position().row + 1,
            ));
        }
        "scoped_use_list" => {
            let next_prefix = node
                .child_by_field_name("path")
                .map(|path_node| {
                    combine_path_prefix(prefix.as_deref(), &render_use_path(path_node, source))
                })
                .or(prefix);

            if let Some(list_node) = node.child_by_field_name("list") {
                flatten_use_tree(list_node, source, next_prefix, imports, is_public);
            }
        }
        "use_list" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                flatten_use_tree(child, source, prefix.clone(), imports, is_public);
            }
        }
        "use_wildcard" => {
            let wildcard_path = node
                .named_child(0)
                .map(|child| {
                    combine_path_prefix(prefix.as_deref(), &render_use_path(child, source))
                })
                .unwrap_or_else(|| combine_path_prefix(prefix.as_deref(), "*"));

            imports.push(ImportSpec {
                line: node.start_position().row + 1,
                group_line: node.start_position().row + 1,
                alias: "*".to_string(),
                path: wildcard_path,
                namespace_path: prefix,
                imported_name: Some("*".to_string()),
                is_public,
            });
        }
        _ => {
            let (alias, path) = if node.kind() == "self" {
                let path = prefix.unwrap_or_else(|| render_use_path(node, source));
                (import_alias(&path), path)
            } else {
                let path = combine_path_prefix(prefix.as_deref(), &render_use_path(node, source));
                (import_alias(&path), path)
            };
            imports.push(build_rust_import_spec(
                alias,
                path,
                is_public,
                node.start_position().row + 1,
            ));
        }
    }
}

fn build_rust_import_spec(alias: String, path: String, is_public: bool, line: usize) -> ImportSpec {
    let (namespace_path, imported_name) = rust_import_segments(&path);

    ImportSpec {
        line,
        group_line: line,
        alias,
        path,
        namespace_path,
        imported_name,
        is_public,
    }
}

fn rust_import_segments(path: &str) -> (Option<String>, Option<String>) {
    if let Some((namespace_path, imported_name)) = path.rsplit_once("::") {
        return (
            Some(namespace_path.to_string()),
            Some(imported_name.to_string()),
        );
    }

    (None, None)
}

fn render_use_path(node: Node<'_>, source: &str) -> String {
    match node.kind() {
        "scoped_identifier" => {
            let path = node
                .child_by_field_name("path")
                .map(|path_node| render_use_path(path_node, source));
            let name = node
                .child_by_field_name("name")
                .and_then(|name_node| source.get(name_node.byte_range()))
                .map(str::trim)
                .unwrap_or("");
            combine_path_prefix(path.as_deref(), name)
        }
        _ => source
            .get(node.byte_range())
            .map(str::trim)
            .unwrap_or("")
            .to_string(),
    }
}

pub(super) fn combine_path_prefix(prefix: Option<&str>, suffix: &str) -> String {
    match prefix.map(str::trim).filter(|prefix| !prefix.is_empty()) {
        Some(prefix) if !suffix.is_empty() => format!("{prefix}::{suffix}"),
        Some(prefix) => prefix.to_string(),
        None => suffix.to_string(),
    }
}

fn import_alias(path: &str) -> String {
    path.rsplit("::")
        .next()
        .filter(|segment| !segment.is_empty())
        .unwrap_or(path)
        .to_string()
}
