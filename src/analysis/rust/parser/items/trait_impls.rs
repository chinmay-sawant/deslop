use std::collections::BTreeSet;

use tree_sitter::Node;

pub(super) fn collect_trait_impls(
    root: Node<'_>,
    source: &str,
    trait_name: &str,
) -> BTreeSet<String> {
    let mut impls = BTreeSet::new();
    visit_trait_impls(root, source, trait_name, &mut impls);
    impls
}

fn visit_trait_impls(node: Node<'_>, source: &str, trait_name: &str, impls: &mut BTreeSet<String>) {
    if node.kind() == "impl_item"
        && let Some(type_name) = trait_impl_type(node, source, trait_name)
    {
        impls.insert(type_name);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_trait_impls(child, source, trait_name, impls);
    }
}

pub(super) fn trait_impl_type(node: Node<'_>, source: &str, trait_name: &str) -> Option<String> {
    let normalized = source
        .get(node.byte_range())?
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let prefix = format!("impl{trait_name}for");
    let remainder = normalized.strip_prefix(&prefix)?;
    let type_name = remainder
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    if type_name.is_empty() {
        None
    } else {
        Some(type_name)
    }
}
