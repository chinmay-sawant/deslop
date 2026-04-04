#[path = "calls.rs"]
mod calls;
#[path = "evidence.rs"]
mod evidence;
#[path = "literals.rs"]
mod literals;
#[path = "metrics.rs"]
mod metrics;

use std::path::Path;

use self::calls::{
    collect_blocking_calls, collect_calls, collect_macro_calls, collect_named_runtime_calls,
};
use self::evidence::{
    collect_await_points, collect_boxed_container_lines, collect_future_creations,
    collect_loop_operation_lines, collect_unsafe_lines, collect_unsafe_patterns, function_is_async,
    inside_trait_impl,
};
use self::literals::{
    collect_local_bindings, collect_local_strings, collect_safety_comments, extract_doc_comment,
    function_is_test_only,
};
use self::metrics::{build_function_fingerprint, enclosing_impl_type, function_kind};

pub(crate) use evidence::is_inside_function;
pub(crate) use literals::{leading_attributes, string_literal_value};

pub(super) fn collect_functions(
    root: tree_sitter::Node<'_>,
    source: &str,
    is_test_file: bool,
) -> Vec<crate::analysis::ParsedFunction> {
    let mut functions = Vec::new();
    visit_for_functions(root, source, is_test_file, &mut functions);
    functions.sort_by(|left, right| {
        left.fingerprint
            .start_line
            .cmp(&right.fingerprint.start_line)
            .then(left.fingerprint.name.cmp(&right.fingerprint.name))
    });
    functions
}

fn visit_for_functions(
    node: tree_sitter::Node<'_>,
    source: &str,
    is_test_file: bool,
    functions: &mut Vec<crate::analysis::ParsedFunction>,
) {
    if node.kind() == "function_item"
        && let Some(parsed_function) = parse_function_node(node, source, is_test_file)
    {
        functions.push(parsed_function);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_functions(child, source, is_test_file, functions);
    }
}

fn parse_function_node(
    node: tree_sitter::Node<'_>,
    source: &str,
    is_test_file: bool,
) -> Option<crate::analysis::ParsedFunction> {
    let body_node = node.child_by_field_name("body")?;
    let calls = collect_calls(body_node, source);
    let local_string_literals = collect_local_strings(body_node, source);
    let local_binding_names = collect_local_bindings(node, source);
    let doc_comment = extract_doc_comment(source, node.start_position().row);
    let body_text = source
        .get(body_node.byte_range())
        .unwrap_or_default()
        .to_string();
    let kind = function_kind(node, source);
    let receiver_type = if kind == "method" {
        enclosing_impl_type(node, source)
    } else {
        None
    };
    let is_test_function = function_is_test_only(node, source, is_test_file);
    let safety_comment_lines = collect_safety_comments(source, node);
    let unsafe_lines = collect_unsafe_lines(node, body_node, source);
    let is_async = function_is_async(node, body_node, source);
    let await_points = collect_await_points(body_node);
    let macro_calls = collect_macro_calls(&calls);
    let spawn_calls =
        collect_named_runtime_calls(&calls, &["spawn", "spawn_local", "spawn_blocking"]);
    let lock_calls = collect_named_runtime_calls(&calls, &["lock", "lock_owned", "read", "write"]);
    let permit_acquires =
        collect_named_runtime_calls(&calls, &["acquire", "acquire_owned", "reserve", "get"]);
    let futures_created = collect_future_creations(body_node, source);
    let blocking_calls = collect_blocking_calls(body_node, source, &calls);
    let select_macro_lines = macro_calls
        .iter()
        .filter(|call| call.name == "select!")
        .map(|call| call.line)
        .collect::<Vec<_>>();
    let drop_impl = inside_trait_impl(node, source, "Drop");
    let (write_loops, line_iteration_loops, default_hasher_lines) =
        collect_loop_operation_lines(body_node, source);
    let boxed_container_lines = collect_boxed_container_lines(body_node, source);
    let unsafe_soundness = collect_unsafe_patterns(body_node, source);
    let fingerprint = build_function_fingerprint(node, source, kind, receiver_type, calls.len())?;

    Some(crate::analysis::ParsedFunction {
        fingerprint,
        signature_text: source
            .get(node.start_byte()..body_node.start_byte())
            .unwrap_or_default()
            .to_string(),
        body_start_line: body_node.start_position().row + 1,
        calls,
        is_test_function,
        local_binding_names,
        doc_comment,
        body_text,
        local_strings: local_string_literals,
        test_summary: None,
        go: None,
        python: None,
        rust: Some(crate::analysis::RustFunctionEvidence {
            safety_comment_lines,
            unsafe_lines,
            is_async,
            await_points,
            macro_calls,
            spawn_calls,
            lock_calls,
            permit_acquires,
            futures_created,
            blocking_calls,
            select_macro_lines,
            drop_impl,
            write_loops,
            line_iteration_loops,
            default_hasher_lines,
            boxed_container_lines,
            unsafe_soundness,
        }),
    })
}

pub(crate) fn module_name_for_path(path: &Path) -> Option<String> {
    if path.file_stem().and_then(|stem| stem.to_str()) == Some("mod") {
        return path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned);
    }

    path.file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn is_test_file(path: &Path) -> bool {
    let in_tests_directory = path
        .components()
        .any(|component| component.as_os_str() == "tests");
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    in_tests_directory || file_name == "tests.rs" || file_name.ends_with("_test.rs")
}
