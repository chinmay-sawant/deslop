mod attributes;
mod enums;
mod includes;
mod modules;
mod pkg_strings;
mod statics;
mod structs;
mod symbols;
mod trait_impls;

use tree_sitter::Node;

use super::{is_inside_function, leading_attributes, string_literal_value};

pub(super) fn collect_attribute_summaries(
    root: Node<'_>,
    source: &str,
) -> Vec<crate::analysis::RustAttributeSummary> {
    attributes::collect_attribute_summaries(root, source)
}

pub(super) fn collect_enum_summaries(
    root: Node<'_>,
    source: &str,
) -> Vec<crate::analysis::RustEnumSummary> {
    enums::collect_enum_summaries(root, source)
}

pub(super) fn collect_module_declarations(
    root: Node<'_>,
    source: &str,
) -> Vec<crate::analysis::RustModuleDeclaration> {
    modules::collect_module_declarations(root, source)
}

pub(super) fn collect_include_declarations(
    root: Node<'_>,
    source: &str,
) -> Vec<crate::analysis::RustIncludeDeclaration> {
    includes::collect_include_declarations(root, source)
}

pub(super) fn collect_pkg_strings(
    root: Node<'_>,
    source: &str,
) -> Vec<crate::analysis::NamedLiteral> {
    pkg_strings::collect_pkg_strings(root, source)
}

pub(super) fn collect_static_summaries(
    root: Node<'_>,
    source: &str,
) -> Vec<crate::analysis::RustStaticSummary> {
    statics::collect_static_summaries(root, source)
}

pub(super) fn collect_struct_summaries(
    root: Node<'_>,
    source: &str,
    default_impls: &std::collections::BTreeSet<String>,
) -> Vec<crate::analysis::StructSummary> {
    structs::collect_struct_summaries(root, source, default_impls)
}

pub(super) fn collect_symbols(
    root: Node<'_>,
    source: &str,
    functions: &[crate::analysis::ParsedFunction],
    imports: &[crate::analysis::ImportSpec],
) -> Vec<crate::analysis::DeclaredSymbol> {
    symbols::collect_symbols(root, source, functions, imports)
}

pub(super) fn collect_trait_impls(
    root: Node<'_>,
    source: &str,
    trait_name: &str,
) -> std::collections::BTreeSet<String> {
    trait_impls::collect_trait_impls(root, source, trait_name)
}

pub(super) fn trait_impl_type(node: Node<'_>, source: &str, trait_name: &str) -> Option<String> {
    trait_impls::trait_impl_type(node, source, trait_name)
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

fn named_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
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
