use tree_sitter::Node;

use crate::analysis::{ParsedFunction, PythonFunctionEvidence};

use super::comments::extract_docstring;
use super::general::{
    build_function_fingerprint, build_test_summary, collect_await_points, collect_calls,
    collect_exception_handlers, collect_local_bindings, collect_local_strings,
    enclosing_class_name,
};
use super::performance::collect_concat_loops;
use super::phase4::{
    collect_builtin_candidate_lines, collect_deque_operation_lines,
    collect_exception_block_signatures, collect_list_materialization_lines,
    collect_membership_loop_lines, collect_missing_manager_lines, collect_none_comparison_lines,
    collect_recursive_call_lines, collect_repeated_len_lines, collect_return_none_lines,
    collect_side_effect_lines, collect_temp_collection_lines, collect_validation_signature,
    has_complete_type_hints, normalize_body, parameter_flags,
};

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

#[derive(Debug)]
struct FunctionShape {
    fingerprint: crate::model::FunctionFingerprint,
    signature_text: String,
    body_start_line: usize,
    calls: Vec<crate::analysis::CallSite>,
    exception_handlers: Vec<crate::analysis::ExceptionHandler>,
    local_binding_names: Vec<String>,
    doc_comment: Option<String>,
    body_text: String,
    normalized_body: String,
    validation_signature: Option<crate::analysis::BlockFingerprint>,
    exception_block_signatures: Vec<crate::analysis::BlockFingerprint>,
    test_summary: Option<crate::analysis::TestFunctionSummary>,
    is_test_function: bool,
    is_async: bool,
}

#[derive(Debug)]
struct FunctionEvidence {
    local_strings: Vec<crate::analysis::NamedLiteral>,
    concat_loops: Vec<usize>,
    none_comparison_lines: Vec<usize>,
    side_effect_comprehension_lines: Vec<usize>,
    redundant_return_none_lines: Vec<usize>,
    list_materialization_lines: Vec<usize>,
    deque_operation_lines: Vec<usize>,
    temp_collection_lines: Vec<usize>,
    recursive_call_lines: Vec<usize>,
    list_membership_loop_lines: Vec<usize>,
    repeated_len_loop_lines: Vec<usize>,
    builtin_candidate_lines: Vec<usize>,
    missing_context_manager_lines: Vec<usize>,
    has_complete_type_hints: bool,
    has_varargs: bool,
    has_kwargs: bool,
    await_points: Vec<usize>,
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

fn parse_function_node(node: Node<'_>, source: &str, is_test_file: bool) -> Option<ParsedFunction> {
    let shape = collect_function_shape(node, source, is_test_file)?;
    let evidence = collect_function_evidence(node, source, &shape.fingerprint.name);

    Some(ParsedFunction {
        fingerprint: shape.fingerprint,
        signature_text: shape.signature_text,
        body_start_line: shape.body_start_line,
        calls: shape.calls,
        is_test_function: shape.is_test_function,
        local_binding_names: shape.local_binding_names,
        doc_comment: shape.doc_comment,
        body_text: shape.body_text,
        local_strings: evidence.local_strings,
        test_summary: shape.test_summary,
        go: None,
        python: Some(PythonFunctionEvidence {
            exception_handlers: shape.exception_handlers,
            validation_signature: shape.validation_signature,
            exception_block_signatures: shape.exception_block_signatures,
            normalized_body: shape.normalized_body,
            concat_loops: evidence.concat_loops,
            none_comparison_lines: evidence.none_comparison_lines,
            side_effect_comprehension_lines: evidence.side_effect_comprehension_lines,
            redundant_return_none_lines: evidence.redundant_return_none_lines,
            list_materialization_lines: evidence.list_materialization_lines,
            deque_operation_lines: evidence.deque_operation_lines,
            temp_collection_lines: evidence.temp_collection_lines,
            recursive_call_lines: evidence.recursive_call_lines,
            list_membership_loop_lines: evidence.list_membership_loop_lines,
            repeated_len_loop_lines: evidence.repeated_len_loop_lines,
            builtin_candidate_lines: evidence.builtin_candidate_lines,
            missing_context_manager_lines: evidence.missing_context_manager_lines,
            has_complete_type_hints: evidence.has_complete_type_hints,
            has_varargs: evidence.has_varargs,
            has_kwargs: evidence.has_kwargs,
            is_async: shape.is_async,
            await_points: evidence.await_points,
        }),
        rust: None,
    })
}

fn collect_function_shape(
    node: Node<'_>,
    source: &str,
    is_test_file: bool,
) -> Option<FunctionShape> {
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
    let local_binding_names = collect_local_bindings(node, source);
    let doc_comment = extract_docstring(body_node, source);
    let normalized_body = normalize_body(body_node, source);
    let validation_signature = collect_validation_signature(body_node, source);
    let exception_block_signatures = collect_exception_block_signatures(body_node, source);
    let test_summary = build_test_summary(&name, body_node, source, is_test_file);
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
        receiver_type,
        calls.len(),
        doc_comment.as_deref(),
    )?;

    Some(FunctionShape {
        fingerprint,
        signature_text: source
            .get(node.start_byte()..body_node.start_byte())
            .unwrap_or_default()
            .to_string(),
        body_start_line: body_node.start_position().row + 1,
        calls,
        exception_handlers,
        local_binding_names,
        doc_comment,
        body_text,
        normalized_body,
        validation_signature,
        exception_block_signatures,
        test_summary,
        is_test_function,
        is_async,
    })
}

fn collect_function_evidence(
    node: Node<'_>,
    source: &str,
    function_name: &str,
) -> FunctionEvidence {
    let Some(body_node) = node.child_by_field_name("body") else {
        return FunctionEvidence {
            local_strings: Vec::new(),
            concat_loops: Vec::new(),
            none_comparison_lines: Vec::new(),
            side_effect_comprehension_lines: Vec::new(),
            redundant_return_none_lines: Vec::new(),
            list_materialization_lines: Vec::new(),
            deque_operation_lines: Vec::new(),
            temp_collection_lines: Vec::new(),
            recursive_call_lines: Vec::new(),
            list_membership_loop_lines: Vec::new(),
            repeated_len_loop_lines: Vec::new(),
            builtin_candidate_lines: Vec::new(),
            missing_context_manager_lines: Vec::new(),
            has_complete_type_hints: false,
            has_varargs: false,
            has_kwargs: false,
            await_points: Vec::new(),
        };
    };
    let (has_varargs, has_kwargs) = parameter_flags(node, source);

    FunctionEvidence {
        local_strings: collect_local_strings(body_node, source),
        concat_loops: collect_concat_loops(body_node, source),
        none_comparison_lines: collect_none_comparison_lines(body_node, source),
        side_effect_comprehension_lines: collect_side_effect_lines(body_node),
        redundant_return_none_lines: collect_return_none_lines(body_node, source),
        list_materialization_lines: collect_list_materialization_lines(body_node, source),
        deque_operation_lines: collect_deque_operation_lines(body_node, source),
        temp_collection_lines: collect_temp_collection_lines(body_node, source),
        recursive_call_lines: collect_recursive_call_lines(function_name, body_node, source),
        list_membership_loop_lines: collect_membership_loop_lines(body_node, source),
        repeated_len_loop_lines: collect_repeated_len_lines(body_node, source),
        builtin_candidate_lines: collect_builtin_candidate_lines(body_node, source),
        missing_context_manager_lines: collect_missing_manager_lines(body_node, source),
        has_complete_type_hints: has_complete_type_hints(node, source),
        has_varargs,
        has_kwargs,
        await_points: collect_await_points(body_node),
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
