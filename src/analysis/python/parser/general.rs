use std::path::Path;

use tree_sitter::Node;

use crate::analysis::{DeclaredSymbol, ImportSpec, ParsedFunction};
use crate::model::{FunctionFingerprint, SymbolKind};

use super::comments::parse_string_literal_text;

#[path = "general/function_evidence.rs"]
mod function_evidence;
#[path = "general/imports.rs"]
mod imports;
#[path = "general/module_scope.rs"]
mod module_scope;

pub(super) use function_evidence::{
    build_test_summary, collect_calls, collect_exception_handlers, collect_local_bindings,
    collect_local_strings,
};
pub(super) use imports::collect_imports;
pub(super) use module_scope::{
    collect_class_summaries, collect_module_scope_calls, collect_pkg_strings,
    collect_python_models, collect_top_level_bindings,
};

pub(super) fn is_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if file_name.starts_with("test_") || file_name.ends_with("_test.py") {
        return true;
    }

    path.components()
        .any(|component| component.as_os_str() == "tests")
}

pub(super) fn module_name_for_path(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    if stem == "__init__" {
        return path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .or_else(|| Some("__init__".to_string()));
    }

    Some(stem.to_string())
}

pub(super) fn collect_await_points(body_node: Node<'_>) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_await_points(body_node, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

pub(super) fn collect_symbols(
    root: Node<'_>,
    source: &str,
    functions: &[ParsedFunction],
    imports: &[ImportSpec],
) -> Vec<DeclaredSymbol> {
    let mut symbols = functions
        .iter()
        .map(|function| DeclaredSymbol {
            name: function.fingerprint.name.clone(),
            kind: if function.fingerprint.receiver_type.is_some() {
                SymbolKind::Method
            } else {
                SymbolKind::Function
            },
            receiver_type: function.fingerprint.receiver_type.clone(),
            receiver_is_pointer: None,
            line: function.fingerprint.start_line,
        })
        .collect::<Vec<_>>();

    for import in imports {
        if import.is_public && import.alias != "*" {
            symbols.push(DeclaredSymbol {
                name: import.alias.clone(),
                kind: SymbolKind::Type,
                receiver_type: None,
                receiver_is_pointer: None,
                line: 1,
            });
        }
    }

    visit_class_symbols(root, source, &mut symbols);
    symbols.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    symbols
}

pub(super) fn build_function_fingerprint(
    node: Node<'_>,
    body_node: Node<'_>,
    source: &str,
    kind: &str,
    receiver_type: Option<String>,
    call_count: usize,
    doc_comment: Option<&str>,
) -> Option<FunctionFingerprint> {
    let name_node = node.child_by_field_name("name")?;
    let function_text = source.get(node.byte_range())?;
    let name = source.get(name_node.byte_range())?.trim().to_string();
    let start_line = node.start_position().row + 1;
    let end_line = node.end_position().row + 1;
    let line_count = end_line.saturating_sub(start_line) + 1;
    let hash_comment_lines = function_text
        .lines()
        .filter(|line| line.trim_start().starts_with('#'))
        .count();
    let doc_comment_lines = doc_comment
        .map(|comment| comment.lines().count())
        .unwrap_or(0);
    let comment_lines = hash_comment_lines + doc_comment_lines;
    let code_lines = line_count.saturating_sub(comment_lines);
    let comment_to_code_ratio = if code_lines == 0 {
        comment_lines as f64
    } else {
        comment_lines as f64 / code_lines as f64
    };
    let complexity_score = count_descendants(
        body_node,
        &[
            "if_statement",
            "for_statement",
            "while_statement",
            "try_statement",
            "except_clause",
            "with_statement",
            "conditional_expression",
            "match_statement",
        ],
    );
    let contains_any_type = source
        .get(node.byte_range())
        .is_some_and(|text| text.contains("Any") || text.contains("typing.Any"));

    Some(FunctionFingerprint {
        name,
        kind: kind.to_string(),
        receiver_type,
        start_line,
        end_line,
        line_count,
        comment_lines,
        code_lines,
        comment_to_code_ratio,
        complexity_score,
        symmetry_score: 0.0,
        boilerplate_err_guards: 0,
        contains_any_type,
        contains_empty_interface: false,
        type_assertion_count: 0,
        call_count,
    })
}

pub(super) fn enclosing_class_name(node: Node<'_>, source: &str) -> Option<String> {
    let mut parent = node.parent();
    while let Some(current) = parent {
        if current.kind() == "function_definition" {
            return None;
        }
        if current.kind() == "class_definition" {
            return current
                .child_by_field_name("name")
                .and_then(|name_node| source.get(name_node.byte_range()))
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(ToOwned::to_owned);
        }
        parent = current.parent();
    }

    None
}

fn visit_await_points(node: Node<'_>, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "await" {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_await_points(child, lines);
    }
}

fn visit_class_symbols(node: Node<'_>, source: &str, symbols: &mut Vec<DeclaredSymbol>) {
    if node.kind() == "class_definition"
        && !is_nested_function(node)
        && let Some(name_node) = node.child_by_field_name("name")
        && let Some(name) = source.get(name_node.byte_range())
    {
        symbols.push(DeclaredSymbol {
            name: name.trim().to_string(),
            kind: SymbolKind::Struct,
            receiver_type: None,
            receiver_is_pointer: None,
            line: node.start_position().row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_class_symbols(child, source, symbols);
    }
}

fn is_nested_function(node: Node<'_>) -> bool {
    let mut parent = node.parent();
    while let Some(current) = parent {
        if current.kind() == "function_definition" {
            return true;
        }
        parent = current.parent();
    }

    false
}

fn collect_base_classes(node: Node<'_>, source: &str) -> Vec<String> {
    let Some(arguments_node) = node.child_by_field_name("superclasses") else {
        return Vec::new();
    };
    let Some(arguments_text) = source.get(arguments_node.byte_range()) else {
        return Vec::new();
    };

    arguments_text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
        .filter(|base| !base.is_empty())
        .map(str::to_string)
        .collect()
}

fn collect_class_decorators(node: Node<'_>, source: &str) -> Vec<String> {
    let Some(parent) = node.parent() else {
        return Vec::new();
    };
    if parent.kind() != "decorated_definition" {
        return Vec::new();
    }

    let mut decorators = Vec::new();
    let mut cursor = parent.walk();
    for child in parent.named_children(&mut cursor) {
        if child.kind() == "decorator"
            && let Some(text) = source.get(child.byte_range())
        {
            decorators.push(text.trim().trim_start_matches('@').to_string());
        }
    }
    decorators
}

fn parse_call_target(callee_text: &str) -> Option<(Option<String>, String)> {
    let normalized = callee_text.trim();
    if normalized.is_empty() {
        return None;
    }

    if let Some((receiver, name)) = normalized.rsplit_once('.') {
        return Some((Some(receiver.trim().to_string()), name.trim().to_string()));
    }

    Some((None, normalized.to_string()))
}

fn named_literal_from_assignment(text: &str, line: usize) -> Option<crate::analysis::NamedLiteral> {
    let (left, right) = split_assignment(text)?;
    let names = assignment_target_names(left);
    if names.len() != 1 {
        return None;
    }

    let value = parse_string_literal_text(right.trim())?;
    Some(crate::analysis::NamedLiteral {
        line,
        name: names.into_iter().next()?,
        value,
    })
}

fn split_assignment(text: &str) -> Option<(&str, &str)> {
    let normalized = text.trim();
    if normalized.contains("==") {
        return None;
    }

    normalized.split_once('=')
}

fn assignment_target_names(text: &str) -> Vec<String> {
    let target_text = text.trim().trim_start_matches('(').trim_end_matches(')');
    let target_text = target_text.split(':').next().unwrap_or(target_text).trim();

    target_text
        .split(',')
        .map(str::trim)
        .filter(|candidate| is_valid_identifier(candidate))
        .map(str::to_string)
        .collect()
}

fn is_module_level(node: Node<'_>) -> bool {
    let mut parent = node.parent();
    while let Some(current) = parent {
        if matches!(current.kind(), "function_definition" | "class_definition") {
            return false;
        }
        parent = current.parent();
    }

    true
}

fn should_skip_nested_scope(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_definition" | "class_definition")
}

fn count_descendants(node: Node<'_>, target_kinds: &[&str]) -> usize {
    let mut count = 0;
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if target_kinds.contains(&child.kind()) {
            count += 1;
        }
        count += count_descendants(child, target_kinds);
    }
    count
}

fn is_valid_identifier(candidate: &str) -> bool {
    let mut characters = candidate.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }

    characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
