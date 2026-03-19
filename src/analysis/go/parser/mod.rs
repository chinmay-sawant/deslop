mod comments;
mod context;
mod errors;
mod general;
mod performance;
#[cfg(test)]
mod tests;

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use tree_sitter::{Node, Parser};

use crate::analysis::go::fingerprint::build_function_fingerprint;
use crate::analysis::{ParsedFile, ParsedFunction};

use self::comments::extract_doc_comment;
use self::context::{
    collect_busy_wait_lines, collect_context_factory_calls, collect_goroutine_launch_lines,
    collect_goroutine_without_shutdown_lines, collect_mutex_lock_in_loop_lines,
    collect_sleep_in_loop_lines, function_has_context_parameter,
};
use self::errors::{
    collect_dropped_error_lines, collect_errorf_calls, collect_panic_on_error_lines,
};
use self::general::{
    collect_calls, collect_imports, collect_symbols, count_descendants, extract_receiver_type,
    find_package_name,
};
use self::performance::{
    collect_allocation_in_loop_lines, collect_db_query_calls, collect_fmt_in_loop_lines,
    collect_json_marshal_in_loop_lines, collect_reflection_in_loop_lines,
    collect_string_concat_in_loop_lines,
};

pub(super) fn parse_file(path: &Path, source: &str) -> Result<ParsedFile> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .map_err(|error| anyhow!(error.to_string()))
        .context("failed to configure Go parser")?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("tree-sitter returned no parse tree"))?;

    let root = tree.root_node();
    let package_name = find_package_name(root, source);
    let imports = collect_imports(root, source);
    let symbols = collect_symbols(root, source);
    let functions = collect_functions(root, source, &imports);

    Ok(ParsedFile {
        path: path.to_path_buf(),
        package_name,
        syntax_error: root.has_error(),
        byte_size: source.len(),
        functions,
        imports,
        symbols,
    })
}

fn collect_functions(root: Node<'_>, source: &str, imports: &[crate::analysis::ImportSpec]) -> Vec<ParsedFunction> {
    let mut functions = Vec::new();
    visit_for_functions(root, source, imports, &mut functions);
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
    functions: &mut Vec<ParsedFunction>,
) {
    if matches!(node.kind(), "function_declaration" | "method_declaration") {
        if let Some(parsed_function) = parse_function_node(node, source, imports) {
            functions.push(parsed_function);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_functions(child, source, imports, functions);
    }
}

fn parse_function_node(
    node: Node<'_>,
    source: &str,
    imports: &[crate::analysis::ImportSpec],
) -> Option<ParsedFunction> {
    let body_node = node.child_by_field_name("body")?;
    let calls = collect_calls(body_node, source);
    let type_assertion_count = count_descendants(body_node, "type_assertion_expression");
    let has_context_parameter = function_has_context_parameter(node, source, imports);
    let doc_comment = extract_doc_comment(source, node.start_position().row);
    let dropped_error_lines = collect_dropped_error_lines(body_node, source);
    let panic_on_error_lines = collect_panic_on_error_lines(body_node, source);
    let errorf_calls = collect_errorf_calls(body_node, source);
    let context_factory_calls = collect_context_factory_calls(body_node, source, imports);
    let goroutine_launch_lines = collect_goroutine_launch_lines(body_node);
    let goroutine_without_shutdown_lines =
        collect_goroutine_without_shutdown_lines(body_node, source);
    let sleep_in_loop_lines = collect_sleep_in_loop_lines(body_node, source, imports);
    let busy_wait_lines = collect_busy_wait_lines(body_node, source);
    let mutex_lock_in_loop_lines = collect_mutex_lock_in_loop_lines(body_node, source);
    let allocation_in_loop_lines = collect_allocation_in_loop_lines(body_node, source, imports);
    let fmt_in_loop_lines = collect_fmt_in_loop_lines(body_node, source, imports);
    let reflection_in_loop_lines = collect_reflection_in_loop_lines(body_node, source, imports);
    let string_concat_in_loop_lines = collect_string_concat_in_loop_lines(body_node, source);
    let json_marshal_in_loop_lines =
        collect_json_marshal_in_loop_lines(body_node, source, imports);
    let db_query_calls = collect_db_query_calls(body_node, source);
    let receiver_type = node
        .child_by_field_name("receiver")
        .and_then(|receiver| extract_receiver_type(receiver, source));
    let fingerprint = build_function_fingerprint(
        node,
        source,
        receiver_type,
        type_assertion_count,
        calls.len(),
    )?;

    Some(ParsedFunction {
        fingerprint,
        calls,
        has_context_parameter,
        doc_comment,
        dropped_error_lines,
        panic_on_error_lines,
        errorf_calls,
        context_factory_calls,
        goroutine_launch_lines,
        goroutine_without_shutdown_lines,
        sleep_in_loop_lines,
        busy_wait_lines,
        mutex_lock_in_loop_lines,
        allocation_in_loop_lines,
        fmt_in_loop_lines,
        reflection_in_loop_lines,
        string_concat_in_loop_lines,
        json_marshal_in_loop_lines,
        db_query_calls,
    })
}
