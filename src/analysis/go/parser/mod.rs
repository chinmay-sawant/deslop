mod comments;
mod context;
mod errors;
mod frameworks;
mod general;
mod performance;
#[cfg(test)]
mod tests;

use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::analysis::go::fingerprint::build_function_fingerprint;
use crate::analysis::{AnalysisResult, Error, Language, ParsedFile, ParsedFunction};

use self::comments::extract_doc_comment;
use self::context::{
    collect_busy_wait_lines, collect_ctx_factories, collect_goroutines, collect_loop_goroutines,
    collect_mutex_loops, collect_sleep_loops, collect_unmanaged_goroutines, has_ctx_param,
};
use self::errors::{collect_dropped_errors, collect_errorf_calls, collect_panic_errors};
use self::frameworks::{collect_gin_calls, collect_gorm_query_chains, collect_parse_input_calls};
use self::general::{
    build_test_summary, collect_calls, collect_go_structs, collect_imports,
    collect_interface_summaries, collect_local_strings, collect_package_vars, collect_pkg_strings,
    collect_struct_tags, collect_symbols, count_descendants, extract_receiver, find_package_name,
};
use self::performance::{
    collect_alloc_loops, collect_concat_loops, collect_db_query_calls, collect_fmt_loops,
    collect_json_loops, collect_reflect_loops,
};

pub(super) fn parse_file(path: &Path, source: &str) -> AnalysisResult<ParsedFile> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .map_err(|error| Error::parser_configuration("Go", error.to_string()))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| Error::missing_parse_tree("Go"))?;

    let root = tree.root_node();
    let package_name = find_package_name(root, source);
    let is_test_file = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with("_test.go"));
    let imports = collect_imports(root, source);
    let package_string_literals = collect_pkg_strings(root, source);
    let struct_tags = collect_struct_tags(root, source);
    let symbols = collect_symbols(root, source);
    let package_vars = collect_package_vars(root, source);
    let interfaces = collect_interface_summaries(root, source);
    let go_structs = collect_go_structs(root, source);
    let functions = collect_functions(root, source, &imports, is_test_file);

    Ok(ParsedFile {
        language: Language::Go,
        path: path.to_path_buf(),
        package_name,
        is_test_file,
        syntax_error: root.has_error(),
        line_count: source.lines().count(),
        byte_size: source.len(),
        pkg_strings: package_string_literals,
        comments: Vec::new(),
        struct_tags,
        functions,
        imports,
        symbols,
        class_summaries: Vec::new(),
        package_vars,
        interfaces,
        go_structs,
        module_scope_calls: Vec::new(),
        top_level_bindings: Vec::new(),
        python_models: Vec::new(),
        rust_statics: Vec::new(),
        rust_enums: Vec::new(),
        structs: Vec::new(),
    })
}

fn collect_functions(
    root: Node<'_>,
    source: &str,
    imports: &[crate::analysis::ImportSpec],
    is_test_file: bool,
) -> Vec<ParsedFunction> {
    let mut functions = Vec::new();
    visit_for_functions(root, source, imports, is_test_file, &mut functions);
    functions.sort_by(|left, right| {
        left.fingerprint
            .start_line
            .cmp(&right.fingerprint.start_line)
            .then(left.fingerprint.name.cmp(&right.fingerprint.name))
    });
    functions
}

fn visit_for_functions(
    node: Node<'_>,
    source: &str,
    imports: &[crate::analysis::ImportSpec],
    is_test_file: bool,
    functions: &mut Vec<ParsedFunction>,
) {
    if matches!(node.kind(), "function_declaration" | "method_declaration")
        && let Some(parsed_function) = parse_function_node(node, source, imports, is_test_file)
    {
        functions.push(parsed_function);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_functions(child, source, imports, is_test_file, functions);
    }
}

fn parse_function_node(
    node: Node<'_>,
    source: &str,
    imports: &[crate::analysis::ImportSpec],
    is_test_file: bool,
) -> Option<ParsedFunction> {
    let body_node = node.child_by_field_name("body")?;
    let signature_text = source
        .get(node.start_byte()..body_node.start_byte())
        .unwrap_or_default()
        .to_string();
    let calls = collect_calls(body_node, source);
    let local_string_literals = collect_local_strings(body_node, source);
    let type_assertion_count = count_descendants(body_node, "type_assertion_expression");
    let has_context_parameter = has_ctx_param(node, source, imports);
    let doc_comment = extract_doc_comment(source, node.start_position().row);
    let function_name = source
        .get(node.child_by_field_name("name")?.byte_range())?
        .to_string();
    let test_summary = build_test_summary(&function_name, body_node, source, &calls, is_test_file);
    let is_test_function = test_summary.is_some();
    let dropped_error_lines = collect_dropped_errors(body_node, source);
    let panic_on_error_lines = collect_panic_errors(body_node, source);
    let errorf_calls = collect_errorf_calls(body_node, source);
    let context_factory_calls = collect_ctx_factories(body_node, source, imports);
    let goroutine_launch_lines = collect_goroutines(body_node);
    let goroutine_in_loop_lines = collect_loop_goroutines(body_node);
    let goroutine_without_shutdown_lines = collect_unmanaged_goroutines(body_node, source);
    let sleep_in_loop_lines = collect_sleep_loops(body_node, source, imports);
    let busy_wait_lines = collect_busy_wait_lines(body_node, source);
    let mutex_lock_in_loop_lines = collect_mutex_loops(body_node, source);
    let allocation_in_loop_lines = collect_alloc_loops(body_node, source, imports);
    let fmt_in_loop_lines = collect_fmt_loops(body_node, source, imports);
    let reflection_in_loop_lines = collect_reflect_loops(body_node, source, imports);
    let string_concat_in_loop_lines = collect_concat_loops(body_node, source);
    let json_marshal_in_loop_lines = collect_json_loops(body_node, source, imports);
    let db_query_calls = collect_db_query_calls(body_node, source);
    let gorm_query_chains = collect_gorm_query_chains(body_node, source, imports);
    let parse_input_calls = collect_parse_input_calls(body_node, source, imports);
    let gin_calls = collect_gin_calls(body_node, source, imports);
    let body_text = source
        .get(body_node.byte_range())
        .unwrap_or_default()
        .to_string();
    let receiver_type = node
        .child_by_field_name("receiver")
        .and_then(|receiver| extract_receiver(receiver, source))
        .map(|(name, _)| name);
    let fingerprint = build_function_fingerprint(
        node,
        source,
        receiver_type,
        type_assertion_count,
        calls.len(),
    )?;

    Some(ParsedFunction {
        fingerprint,
        signature_text,
        body_start_line: body_node.start_position().row + 1,
        calls,
        exception_handlers: Vec::new(),
        has_context_parameter,
        is_test_function,
        local_binding_names: Vec::new(),
        doc_comment,
        body_text,
        local_strings: local_string_literals,
        normalized_body: String::new(),
        validation_signature: None,
        exception_block_signatures: Vec::new(),
        test_summary,
        safety_comment_lines: Vec::new(),
        unsafe_lines: Vec::new(),
        dropped_errors: dropped_error_lines,
        panic_errors: panic_on_error_lines,
        errorf_calls,
        context_factory_calls,
        goroutines: goroutine_launch_lines,
        loop_goroutines: goroutine_in_loop_lines,
        unmanaged_goroutines: goroutine_without_shutdown_lines,
        sleep_loops: sleep_in_loop_lines,
        busy_wait_lines,
        mutex_loops: mutex_lock_in_loop_lines,
        alloc_loops: allocation_in_loop_lines,
        fmt_loops: fmt_in_loop_lines,
        reflect_loops: reflection_in_loop_lines,
        concat_loops: string_concat_in_loop_lines,
        json_loops: json_marshal_in_loop_lines,
        db_query_calls,
        gorm_query_chains,
        parse_input_calls,
        gin_calls,
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
        is_async: false,
        await_points: Vec::new(),
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
