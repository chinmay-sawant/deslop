use tree_sitter::Node;

use crate::analysis::{
    ClassSummary, NamedLiteral, PythonFieldSummary, PythonModelSummary, TopLevelBindingSummary,
    TopLevelCallSummary,
};

use super::super::phase4::collect_class_summaries as collect_phase4_class_summaries;
use super::{
    assignment_target_names, collect_base_classes, collect_class_decorators, is_module_level,
    named_literal_from_assignment, parse_call_target, split_assignment,
};

pub(crate) fn collect_pkg_strings(root: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_pkg_strings(root, source, &mut literals);
    literals.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    literals
}

pub(crate) fn collect_module_scope_calls(root: Node<'_>, source: &str) -> Vec<TopLevelCallSummary> {
    let mut calls = Vec::new();
    visit_module_scope_calls(root, source, &mut calls);
    calls.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    calls
}

pub(crate) fn collect_top_level_bindings(
    root: Node<'_>,
    source: &str,
) -> Vec<TopLevelBindingSummary> {
    let mut bindings = Vec::new();
    visit_top_level_bindings(root, source, &mut bindings);
    bindings.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    bindings
}

pub(crate) fn collect_python_models(root: Node<'_>, source: &str) -> Vec<PythonModelSummary> {
    let mut models = Vec::new();
    visit_python_models(root, source, &mut models);
    models.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    models
}

pub(crate) fn collect_class_summaries(root: Node<'_>, source: &str) -> Vec<ClassSummary> {
    collect_phase4_class_summaries(root, source)
}

fn visit_pkg_strings(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && is_module_level(node)
        && let Some(text) = source.get(node.byte_range())
        && let Some(literal) = named_literal_from_assignment(text, node.start_position().row + 1)
    {
        literals.push(literal);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_pkg_strings(child, source, literals);
    }
}

fn visit_module_scope_calls(node: Node<'_>, source: &str, calls: &mut Vec<TopLevelCallSummary>) {
    if node.kind() == "call"
        && is_module_level(node)
        && let Some(function_node) = node.child_by_field_name("function")
        && let Some(callee_text) = source.get(function_node.byte_range())
        && let Some((receiver, name)) = parse_call_target(callee_text)
    {
        calls.push(TopLevelCallSummary {
            line: node.start_position().row + 1,
            receiver,
            name,
            text: source
                .get(node.byte_range())
                .unwrap_or_default()
                .trim()
                .to_string(),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_module_scope_calls(child, source, calls);
    }
}

fn visit_top_level_bindings(
    node: Node<'_>,
    source: &str,
    bindings: &mut Vec<TopLevelBindingSummary>,
) {
    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && is_module_level(node)
        && let Some(text) = source.get(node.byte_range())
        && let Some((left, right)) = split_assignment(text)
    {
        for name in assignment_target_names(left) {
            bindings.push(TopLevelBindingSummary {
                name,
                line: node.start_position().row + 1,
                value_text: right.trim().to_string(),
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_top_level_bindings(child, source, bindings);
    }
}

fn visit_python_models(node: Node<'_>, source: &str, models: &mut Vec<PythonModelSummary>) {
    if node.kind() == "class_definition"
        && let Some(model) = python_model_summary(node, source)
    {
        models.push(model);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_python_models(child, source, models);
    }
}

fn python_model_summary(node: Node<'_>, source: &str) -> Option<PythonModelSummary> {
    let name_node = node.child_by_field_name("name")?;
    let body_node = node.child_by_field_name("body")?;
    let name = source.get(name_node.byte_range())?.trim().to_string();
    let base_classes = collect_base_classes(node, source);
    let decorators = collect_class_decorators(node, source);
    let is_dataclass = decorators
        .iter()
        .any(|decorator| decorator.ends_with("dataclass") || decorator.contains(".dataclass"));
    let is_typed_dict = base_classes
        .iter()
        .any(|base| base.ends_with("TypedDict") || base == "TypedDict");

    let mut fields = Vec::new();
    let mut method_names = Vec::new();
    let mut cursor = body_node.walk();
    for child in body_node.named_children(&mut cursor) {
        match child.kind() {
            "function_definition" => {
                if let Some(method_name) = child
                    .child_by_field_name("name")
                    .and_then(|method_name| source.get(method_name.byte_range()))
                {
                    method_names.push(method_name.trim().to_string());
                }
            }
            _ => visit_python_model_fields(child, source, &mut fields),
        }
    }

    Some(PythonModelSummary {
        name,
        line: node.start_position().row + 1,
        base_classes,
        decorators,
        is_dataclass,
        is_typed_dict,
        fields,
        method_names,
    })
}

fn python_field_summaries(node: Node<'_>, source: &str) -> Vec<PythonFieldSummary> {
    let Some(text) = source.get(node.byte_range()) else {
        return Vec::new();
    };
    let line = node.start_position().row + 1;

    if node.kind() == "expression_statement"
        && !text.contains('=')
        && let Some((left, right)) = text.split_once(':')
    {
        let target_name = assignment_target_names(left);
        if target_name.len() == 1 {
            return vec![PythonFieldSummary {
                name: target_name[0].clone(),
                line,
                annotation_text: Some(right.trim().to_string()),
                default_text: None,
            }];
        }
    }

    if node.kind() == "annotated_assignment" {
        let Some((left, right)) = split_assignment(text).or_else(|| text.split_once(':')) else {
            return Vec::new();
        };
        let target_name = assignment_target_names(left);
        if target_name.len() != 1 {
            return Vec::new();
        }
        let name = target_name[0].clone();
        let (annotation_text, default_text) = if text.contains('=') {
            let (annotated_left, value) = split_assignment(text).unwrap_or((left, right));
            (
                annotated_left
                    .split_once(':')
                    .map(|(_, annotation)| annotation.trim().to_string()),
                Some(value.trim().to_string()),
            )
        } else {
            (Some(right.trim().to_string()), None)
        };

        return vec![PythonFieldSummary {
            name,
            line,
            annotation_text,
            default_text,
        }];
    }

    if node.kind() == "assignment"
        && let Some((left, right)) = split_assignment(text)
    {
        return assignment_target_names(left)
            .into_iter()
            .map(|name| PythonFieldSummary {
                name,
                line,
                annotation_text: None,
                default_text: Some(right.trim().to_string()),
            })
            .collect();
    }

    Vec::new()
}

fn visit_python_model_fields(node: Node<'_>, source: &str, fields: &mut Vec<PythonFieldSummary>) {
    if matches!(node.kind(), "function_definition" | "class_definition") {
        return;
    }

    if matches!(
        node.kind(),
        "assignment" | "annotated_assignment" | "expression_statement"
    ) {
        fields.extend(python_field_summaries(node, source));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_python_model_fields(child, source, fields);
    }
}
