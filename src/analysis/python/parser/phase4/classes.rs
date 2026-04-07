use std::collections::BTreeSet;

use tree_sitter::Node;

use crate::analysis::ClassSummary;

pub(crate) fn collect_class_summaries(root: Node<'_>, source: &str) -> Vec<ClassSummary> {
    let mut summaries = Vec::new();
    visit_class_summaries(root, source, &mut summaries);
    summaries.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    summaries
}

fn visit_class_summaries(node: Node<'_>, source: &str, summaries: &mut Vec<ClassSummary>) {
    if node.kind() == "class_definition"
        && let Some(summary) = class_summary(node, source)
    {
        summaries.push(summary);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_class_summaries(child, source, summaries);
    }
}

fn class_summary(node: Node<'_>, source: &str) -> Option<ClassSummary> {
    let name = node
        .child_by_field_name("name")
        .and_then(|name_node| source.get(name_node.byte_range()))?
        .trim()
        .to_string();
    let body_node = node.child_by_field_name("body")?;
    let class_text = source.get(node.byte_range())?;
    let mut method_count = 0;
    let mut public_method_count = 0;
    let mut instance_attributes = BTreeSet::new();
    let mut constructor_collaborator_count = 0;
    let mut cursor = body_node.walk();

    for child in body_node.named_children(&mut cursor) {
        if let Some(method_node) = direct_method_node(child) {
            method_count += 1;
            let method_name = method_node
                .child_by_field_name("name")
                .and_then(|name_node| source.get(name_node.byte_range()))
                .map(str::trim)
                .unwrap_or_default();
            if !method_name.starts_with('_') {
                public_method_count += 1;
            }
            if let Some(method_body) = method_node.child_by_field_name("body") {
                collect_self_attribute_writes(method_body, source, &mut instance_attributes);
                if method_name == "__init__" {
                    constructor_collaborator_count =
                        collect_constructor_collaborators(method_body, source);
                }
            }
        }
    }

    Some(ClassSummary {
        name,
        line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        method_count,
        public_method_count,
        instance_attribute_count: instance_attributes.len(),
        base_classes: class_base_names(class_text),
        constructor_collaborator_count,
    })
}

fn direct_method_node(node: Node<'_>) -> Option<Node<'_>> {
    match node.kind() {
        "function_definition" => Some(node),
        "decorated_definition" => {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find(|child| child.kind() == "function_definition")
        }
        _ => None,
    }
}

fn collect_self_attribute_writes(node: Node<'_>, source: &str, attributes: &mut BTreeSet<String>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(
        node.kind(),
        "assignment" | "annotated_assignment" | "augmented_assignment"
    ) && let Some(text) = source.get(node.byte_range())
        && let Some(left) = assignment_left(text)
    {
        for target in left.split(',') {
            if let Some(attribute) = self_attribute_name(target) {
                attributes.insert(attribute);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_self_attribute_writes(child, source, attributes);
    }
}

fn assignment_left(text: &str) -> Option<&str> {
    for operator in ["+=", "-=", "*=", "/=", "%=", "="] {
        if let Some((left, _)) = text.split_once(operator) {
            return Some(left);
        }
    }
    None
}

fn self_attribute_name(target: &str) -> Option<String> {
    let trimmed = target
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(':')
        .next()
        .unwrap_or(target)
        .trim();
    let suffix = trimmed.strip_prefix("self.")?;
    let attribute = suffix
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    (!attribute.is_empty()).then_some(attribute)
}

fn collect_constructor_collaborators(body_node: Node<'_>, source: &str) -> usize {
    let mut collaborators = BTreeSet::new();
    visit_constructor_collaborators(body_node, source, &mut collaborators);
    collaborators.len()
}

fn visit_constructor_collaborators(
    node: Node<'_>,
    source: &str,
    collaborators: &mut BTreeSet<String>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some(text) = source.get(node.byte_range())
        && let Some((left, right)) = text.split_once('=')
        && let Some(attribute_name) = self_attribute_name(left)
        && looks_like_constructor_call(right)
    {
        collaborators.insert(attribute_name);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_constructor_collaborators(child, source, collaborators);
    }
}

fn class_base_names(text: &str) -> Vec<String> {
    let header = text.lines().next().unwrap_or_default().trim();
    let Some(open_index) = header.find('(') else {
        return Vec::new();
    };
    let Some(close_index) = header
        .get(open_index + 1..)
        .and_then(|suffix| suffix.find(')'))
    else {
        return Vec::new();
    };
    header
        .get(open_index + 1..open_index + 1 + close_index)
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|base| !base.is_empty())
        .map(str::to_string)
        .collect()
}

fn looks_like_constructor_call(text: &str) -> bool {
    let normalized = text.trim().replace(' ', "");
    if !normalized.contains('(') {
        return false;
    }

    let Some(target) = normalized.split('(').next() else {
        return false;
    };
    !matches!(
        target,
        "list" | "dict" | "set" | "tuple" | "str" | "int" | "float" | "bool"
    ) && !target.starts_with("self.")
}

fn should_skip_nested_scope(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_definition" | "class_definition")
}
