use std::collections::BTreeSet;
use std::path::Path;

use tree_sitter::Node;

use crate::analysis::{
    CallSite, ClassSummary, DeclaredSymbol, ExceptionHandler, ImportSpec, NamedLiteral,
    ParsedFunction, PythonFieldSummary, PythonModelSummary, TestFunctionSummary,
    TopLevelBindingSummary, TopLevelCallSummary,
};
use crate::model::{FunctionFingerprint, SymbolKind};

use super::comments::{extract_docstring, parse_string_literal_text};
use super::performance::collect_concat_loops;
use super::phase4::{
    collect_builtin_candidate_lines, collect_class_summaries as collect_phase4_class_summaries,
    collect_deque_operation_lines, collect_exception_block_signatures,
    collect_list_materialization_lines, collect_membership_loop_lines,
    collect_missing_manager_lines, collect_none_comparison_lines, collect_recursive_call_lines,
    collect_repeated_len_lines, collect_return_none_lines, collect_side_effect_lines,
    collect_temp_collection_lines, collect_validation_signature, has_complete_type_hints,
    normalize_body, parameter_flags,
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

pub(super) fn collect_imports(root: Node<'_>, source: &str) -> Vec<ImportSpec> {
    let mut imports = Vec::new();
    visit_imports(root, source, &mut imports);
    imports
}

pub(super) fn collect_pkg_strings(root: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_pkg_strings(root, source, &mut literals);
    literals.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    literals
}

pub(super) fn collect_functions(
    root: Node<'_>,
    source: &str,
    is_test_file: bool,
) -> Vec<ParsedFunction> {
    let mut functions = Vec::new();
    visit_functions(root, source, is_test_file, &mut functions);
    functions.sort_by(|left, right| {
        left.fingerprint
            .start_line
            .cmp(&right.fingerprint.start_line)
            .then(left.fingerprint.name.cmp(&right.fingerprint.name))
    });
    functions
}

pub(super) fn collect_module_scope_calls(
    root: Node<'_>,
    source: &str,
) -> Vec<TopLevelCallSummary> {
    let mut calls = Vec::new();
    visit_module_scope_calls(root, source, &mut calls);
    calls.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    calls
}

pub(super) fn collect_top_level_bindings(
    root: Node<'_>,
    source: &str,
) -> Vec<TopLevelBindingSummary> {
    let mut bindings = Vec::new();
    visit_top_level_bindings(root, source, &mut bindings);
    bindings.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    bindings
}

pub(super) fn collect_python_models(root: Node<'_>, source: &str) -> Vec<PythonModelSummary> {
    let mut models = Vec::new();
    visit_python_models(root, source, &mut models);
    models.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    models
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

pub(super) fn collect_class_summaries(root: Node<'_>, source: &str) -> Vec<ClassSummary> {
    collect_phase4_class_summaries(root, source)
}

pub(super) fn collect_calls(body_node: Node<'_>, source: &str) -> Vec<CallSite> {
    let mut calls = Vec::new();
    visit_calls(body_node, source, &mut calls);
    calls
}

pub(super) fn collect_local_strings(body_node: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_local_strings(body_node, source, &mut literals);
    literals.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    literals
}

pub(super) fn collect_exception_handlers(
    body_node: Node<'_>,
    source: &str,
) -> Vec<ExceptionHandler> {
    let mut handlers = Vec::new();
    visit_exception_handlers(body_node, source, &mut handlers);
    handlers
}

pub(super) fn collect_local_bindings(function_node: Node<'_>, source: &str) -> Vec<String> {
    let mut names = BTreeSet::new();

    if let Some(parameters_node) = function_node.child_by_field_name("parameters")
        && let Some(parameters_text) = source.get(parameters_node.byte_range())
    {
        for name in parameter_names(parameters_text) {
            names.insert(name);
        }
    }

    if let Some(body_node) = function_node.child_by_field_name("body") {
        visit_assignment_bindings(body_node, source, &mut names);
    }

    names.into_iter().collect()
}

pub(super) fn build_test_summary(
    function_name: &str,
    body_node: Node<'_>,
    source: &str,
    is_test_file: bool,
) -> Option<TestFunctionSummary> {
    if !is_test_file || !function_name.starts_with("test_") {
        return None;
    }

    let mut assertion_like_calls = 0;
    let mut error_assertion_calls = 0;
    let mut skip_calls = 0;
    let mut production_calls = 0;
    for call in collect_calls(body_node, source) {
        match (call.receiver.as_deref(), call.name.as_str()) {
            (_, "assert") | (Some("self"), _) if call.name.starts_with("assert") => {
                assertion_like_calls += 1;
                if matches!(call.name.as_str(), "assertRaises" | "assertRaisesRegex") {
                    error_assertion_calls += 1;
                }
            }
            (Some("pytest"), "raises") => {
                assertion_like_calls += 1;
                error_assertion_calls += 1;
            }
            (_, "skip") | (_, "skipTest") => {
                skip_calls += 1;
            }
            (_, name) if !matches!(name, "print" | "assert") => {
                production_calls += 1;
            }
            _ => {}
        }
    }

    // Python's bare `assert` keyword is a statement, not a call, so
    // collect_calls misses it. Count assert_statement AST nodes explicitly.
    assertion_like_calls += count_assert_statements(body_node);

    let body_text = source.get(body_node.byte_range()).unwrap_or_default();
    let has_todo_marker = body_text.to_ascii_uppercase().contains("TODO");

    Some(TestFunctionSummary {
        assertion_like_calls,
        error_assertion_calls,
        skip_calls,
        production_calls,
        has_todo_marker,
    })
}

fn visit_imports(node: Node<'_>, source: &str, imports: &mut Vec<ImportSpec>) {
    match node.kind() {
        "import_statement" => {
            if let Some(text) = source.get(node.byte_range()) {
                imports.extend(parse_import_statement_text(
                    text,
                    node.start_position().row + 1,
                ));
            }
        }
        "import_from_statement" => {
            if let Some(text) = source.get(node.byte_range()) {
                imports.extend(parse_import_from_stmt(text, node.start_position().row + 1));
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_imports(child, source, imports);
    }
}

fn parse_import_statement_text(text: &str, line: usize) -> Vec<ImportSpec> {
    let normalized = normalize_import_text(text);
    let Some(rest) = normalized.strip_prefix("import ") else {
        return Vec::new();
    };

    split_import_list(rest)
        .into_iter()
        .filter_map(|entry| {
            let (path, alias) = parse_alias(&entry);
            if path.is_empty() || alias.is_empty() {
                return None;
            }

            Some(ImportSpec {
                line,
                group_line: line,
                alias,
                path: path.clone(),
                namespace_path: namespace_path(&path),
                imported_name: imported_name(&path),
                is_public: false,
            })
        })
        .collect()
}

fn parse_import_from_stmt(text: &str, line: usize) -> Vec<ImportSpec> {
    let normalized = normalize_import_text(text);
    let Some(rest) = normalized.strip_prefix("from ") else {
        return Vec::new();
    };
    let Some((module_path, imports_text)) = rest.split_once(" import ") else {
        return Vec::new();
    };

    split_import_list(imports_text)
        .into_iter()
        .filter_map(|entry| {
            let (import_name, alias) = parse_alias(&entry);
            if import_name.is_empty() || alias.is_empty() {
                return None;
            }

            let full_path = if import_name == "*" {
                format!("{module_path}.*")
            } else {
                format!("{module_path}.{import_name}")
            };

            Some(ImportSpec {
                line,
                group_line: line,
                alias,
                path: full_path,
                namespace_path: Some(module_path.to_string()),
                imported_name: Some(import_name),
                is_public: false,
            })
        })
        .collect()
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

fn visit_functions(
    node: Node<'_>,
    source: &str,
    is_test_file: bool,
    functions: &mut Vec<ParsedFunction>,
) {
    if node.kind() == "function_definition"
        && !is_nested_function(node)
        && let Some(parsed_function) = parse_function_node(node, source, is_test_file)
    {
        functions.push(parsed_function);
        return;
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_functions(child, source, is_test_file, functions);
    }
}

fn visit_module_scope_calls(
    node: Node<'_>,
    source: &str,
    calls: &mut Vec<TopLevelCallSummary>,
) {
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

fn visit_python_models(
    node: Node<'_>,
    source: &str,
    models: &mut Vec<PythonModelSummary>,
) {
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

fn python_model_summary(node: Node<'_>, source: &str) -> Option<PythonModelSummary> {
    let name_node = node.child_by_field_name("name")?;
    let body_node = node.child_by_field_name("body")?;
    let name = source.get(name_node.byte_range())?.trim().to_string();
    let base_classes = collect_base_classes(node, source);
    let decorators = collect_class_decorators(node, source);
    let is_dataclass = decorators.iter().any(|decorator| {
        decorator.ends_with("dataclass") || decorator.contains(".dataclass")
    });
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

fn visit_python_model_fields(
    node: Node<'_>,
    source: &str,
    fields: &mut Vec<PythonFieldSummary>,
) {
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

fn parse_function_node(node: Node<'_>, source: &str, is_test_file: bool) -> Option<ParsedFunction> {
    let name_node = node.child_by_field_name("name")?;
    let body_node = node.child_by_field_name("body")?;
    let name = source.get(name_node.byte_range())?.trim().to_string();
    let receiver_type = enclosing_class_name(node, source);
    let is_async = source
        .get(node.byte_range())
        .is_some_and(|text| text.trim_start().starts_with("async def "));
    let kind = match (is_async, receiver_type.is_some()) {
        (true, true) => "async_method",
        (true, false) => "async_function",
        (false, true) => "method",
        (false, false) => "function",
    };
    let calls = collect_calls(body_node, source);
    let exception_handlers = collect_exception_handlers(body_node, source);
    let body_text = source.get(body_node.byte_range())?.to_string();
    let local_string_literals = collect_local_strings(body_node, source);
    let local_binding_names = collect_local_bindings(node, source);
    let doc_comment = extract_docstring(body_node, source);
    let normalized_body = normalize_body(body_node, source);
    let validation_signature = collect_validation_signature(body_node, source);
    let exception_block_signatures = collect_exception_block_signatures(body_node, source);
    let test_summary = build_test_summary(&name, body_node, source, is_test_file);
    let none_comparison_lines = collect_none_comparison_lines(body_node, source);
    let side_effect_lines = collect_side_effect_lines(body_node);
    let return_none_lines = collect_return_none_lines(body_node, source);
    let list_materialization_lines = collect_list_materialization_lines(body_node, source);
    let deque_operation_lines = collect_deque_operation_lines(body_node, source);
    let temp_collection_lines = collect_temp_collection_lines(body_node, source);
    let recursive_call_lines = collect_recursive_call_lines(&name, body_node, source);
    let membership_loop_lines = collect_membership_loop_lines(body_node, source);
    let repeated_len_lines = collect_repeated_len_lines(body_node, source);
    let builtin_candidate_lines = collect_builtin_candidate_lines(body_node, source);
    let missing_manager_lines = collect_missing_manager_lines(body_node, source);
    let has_complete_type_hints = has_complete_type_hints(node, source);
    let (has_varargs, has_kwargs) = parameter_flags(node, source);
    let await_points = collect_await_points(body_node);
    let is_test_function = test_summary.is_some()
        || (is_test_file
            && name.starts_with("test_")
            && receiver_type
                .as_deref()
                .is_none_or(|class_name| class_name.starts_with("Test")));
    let fingerprint = build_function_fingerprint(
        node,
        body_node,
        source,
        kind,
        receiver_type.clone(),
        calls.len(),
        doc_comment.as_deref(),
    )?;

    Some(ParsedFunction {
        fingerprint,
        signature_text: source
            .get(node.start_byte()..body_node.start_byte())
            .unwrap_or_default()
            .to_string(),
        body_start_line: body_node.start_position().row + 1,
        calls,
        exception_handlers,
        has_context_parameter: false,
        is_test_function,
        local_binding_names,
        doc_comment,
        body_text,
        local_strings: local_string_literals,
        normalized_body,
        validation_signature,
        exception_block_signatures,
        test_summary,
        safety_comment_lines: Vec::new(),
        unsafe_lines: Vec::new(),
        dropped_errors: Vec::new(),
        panic_errors: Vec::new(),
        errorf_calls: Vec::new(),
        context_factory_calls: Vec::new(),
        goroutines: Vec::new(),
        loop_goroutines: Vec::new(),
        unmanaged_goroutines: Vec::new(),
        sleep_loops: Vec::new(),
        busy_wait_lines: Vec::new(),
        mutex_loops: Vec::new(),
        alloc_loops: Vec::new(),
        fmt_loops: Vec::new(),
        reflect_loops: Vec::new(),
        concat_loops: collect_concat_loops(body_node, source),
        json_loops: Vec::new(),
        db_query_calls: Vec::new(),
        gorm_query_chains: Vec::new(),
        parse_input_calls: Vec::new(),
        gin_calls: Vec::new(),
        none_comparison_lines,
        side_effect_comprehension_lines: side_effect_lines,
        redundant_return_none_lines: return_none_lines,
        list_materialization_lines,
        deque_operation_lines,
        temp_collection_lines,
        recursive_call_lines,
        list_membership_loop_lines: membership_loop_lines,
        repeated_len_loop_lines: repeated_len_lines,
        builtin_candidate_lines,
        missing_context_manager_lines: missing_manager_lines,
        has_complete_type_hints,
        has_varargs,
        has_kwargs,
        is_async,
        await_points,
        macro_calls: Vec::new(),
        spawn_calls: Vec::new(),
        lock_calls: Vec::new(),
        permit_acquires: Vec::new(),
        futures_created: Vec::new(),
        blocking_calls: Vec::new(),
        select_macro_lines: Vec::new(),
        drop_impl: false,
        write_loops: Vec::new(),
        line_iteration_loops: Vec::new(),
        default_hasher_lines: Vec::new(),
        boxed_container_lines: Vec::new(),
        unsafe_soundness: Vec::new(),
    })
}

fn build_function_fingerprint(
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

fn count_assert_statements(node: Node<'_>) -> usize {
    let mut count = if node.kind() == "assert_statement" {
        1
    } else {
        0
    };
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        count += count_assert_statements(child);
    }
    count
}

fn visit_calls(node: Node<'_>, source: &str, calls: &mut Vec<CallSite>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "call"
        && let Some(function_node) = node.child_by_field_name("function")
        && let Some(callee_text) = source.get(function_node.byte_range())
        && let Some((receiver, name)) = parse_call_target(callee_text)
    {
        calls.push(CallSite {
            receiver,
            name,
            line: node.start_position().row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_calls(child, source, calls);
    }
}

fn visit_local_strings(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some(text) = source.get(node.byte_range())
        && let Some(literal) = named_literal_from_assignment(text, node.start_position().row + 1)
    {
        literals.push(literal);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_local_strings(child, source, literals);
    }
}

fn visit_assignment_bindings(node: Node<'_>, source: &str, names: &mut BTreeSet<String>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some(text) = source.get(node.byte_range())
    {
        for name in assignment_target_names(text) {
            names.insert(name);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_assignment_bindings(child, source, names);
    }
}

fn visit_exception_handlers(node: Node<'_>, source: &str, handlers: &mut Vec<ExceptionHandler>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "except_clause"
        && let Some(text) = source.get(node.byte_range())
        && let Some(handler) = exception_handler_from_text(text, node.start_position().row + 1)
    {
        handlers.push(handler);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_exception_handlers(child, source, handlers);
    }
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

fn exception_handler_from_text(text: &str, line: usize) -> Option<ExceptionHandler> {
    let trimmed_lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let clause_line = *trimmed_lines.first()?;
    if !clause_line.starts_with("except") {
        return None;
    }

    let action = handler_action(&trimmed_lines);
    let normalized_clause = clause_line.to_ascii_lowercase();
    let is_broad = normalized_clause == "except:"
        || normalized_clause.starts_with("except exception")
        || normalized_clause.starts_with("except baseexception")
        || normalized_clause.starts_with("except (exception")
        || normalized_clause.starts_with("except (baseexception");
    let suppresses = action.as_deref().is_some_and(is_suppression_action);

    Some(ExceptionHandler {
        line,
        clause: clause_line.to_string(),
        action,
        is_broad,
        suppresses,
    })
}

fn handler_action(trimmed_lines: &[&str]) -> Option<String> {
    if trimmed_lines.is_empty() {
        return None;
    }

    let clause_line = trimmed_lines[0];
    if let Some((_, inline_action)) = clause_line.split_once(':') {
        let inline_action = inline_action.trim();
        if !inline_action.is_empty() {
            return Some(inline_action.to_string());
        }
    }

    trimmed_lines.get(1).map(|line| (*line).to_string())
}

fn is_suppression_action(action: &str) -> bool {
    let normalized = action.trim().to_ascii_lowercase();
    normalized == "pass"
        || normalized == "continue"
        || normalized == "break"
        || normalized.starts_with("return")
}

fn named_literal_from_assignment(text: &str, line: usize) -> Option<NamedLiteral> {
    let (left, right) = split_assignment(text)?;
    let names = assignment_target_names(left);
    if names.len() != 1 {
        return None;
    }

    let value = parse_string_literal_text(right.trim())?;
    Some(NamedLiteral {
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

fn parameter_names(parameters_text: &str) -> Vec<String> {
    parameters_text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty() && *entry != "/")
        .filter_map(|entry| {
            let entry = entry.trim_start_matches('*').trim();
            let entry = entry.split('=').next().unwrap_or(entry).trim();
            let entry = entry.split(':').next().unwrap_or(entry).trim();
            is_valid_identifier(entry).then(|| entry.to_string())
        })
        .collect()
}

fn namespace_path(path: &str) -> Option<String> {
    path.rsplit_once('.')
        .map(|(namespace, _)| namespace.to_string())
}

fn imported_name(path: &str) -> Option<String> {
    path.rsplit('.').next().map(str::to_string)
}

fn parse_alias(entry: &str) -> (String, String) {
    let trimmed = entry
        .trim()
        .trim_matches(|character| character == '(' || character == ')');
    if let Some((path, alias)) = trimmed.rsplit_once(" as ") {
        return (path.trim().to_string(), alias.trim().to_string());
    }

    let alias = trimmed
        .rsplit('.')
        .next()
        .unwrap_or(trimmed)
        .trim()
        .to_string();
    (trimmed.to_string(), alias)
}

fn normalize_import_text(text: &str) -> String {
    text.lines()
        .map(strip_python_comment)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .replace("( ", "")
        .replace(" )", "")
}

fn strip_python_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut previous_was_escape = false;

    for (index, character) in line.char_indices() {
        match character {
            '\\' if in_single || in_double => {
                previous_was_escape = !previous_was_escape;
                continue;
            }
            '\'' if !in_double && !previous_was_escape => {
                in_single = !in_single;
            }
            '"' if !in_single && !previous_was_escape => {
                in_double = !in_double;
            }
            '#' if !in_single && !in_double => {
                return &line[..index];
            }
            _ => {}
        }

        if character != '\\' {
            previous_was_escape = false;
        }
    }

    line
}

fn split_import_list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
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

fn enclosing_class_name(node: Node<'_>, source: &str) -> Option<String> {
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
