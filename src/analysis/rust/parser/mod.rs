mod imports;
mod items;
#[cfg(test)]
mod tests;

use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::analysis::{
    AnalysisResult, CallSite, Error, Language, MacroCall, NamedLiteral, ParsedFile, ParsedFunction,
    RuntimeCall, UnsafePattern, UnsafePatternKind,
};
use crate::model::FunctionFingerprint;

use self::imports::collect_imports;
use self::items::{
    collect_enum_summaries, collect_pkg_strings, collect_static_summaries,
    collect_struct_summaries, collect_symbols, collect_trait_impls, trait_impl_type,
};
use self::imports::combine_path_prefix;

pub(super) fn parse_file(path: &Path, source: &str) -> AnalysisResult<ParsedFile> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|error| Error::parser_configuration("Rust", error.to_string()))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| Error::missing_parse_tree("Rust"))?;

    let root = tree.root_node();
    let is_test_file = is_test_file(path);
    let imports = collect_imports(root, source);
    let package_string_literals = collect_pkg_strings(root, source);
    let default_impls = collect_trait_impls(root, source, "Default");
    let functions = collect_functions(root, source, is_test_file);
    let symbols = collect_symbols(root, source, &functions, &imports);
    let rust_statics = collect_static_summaries(root, source);
    let rust_enums = collect_enum_summaries(root, source);
    let structs = collect_struct_summaries(root, source, &default_impls);

    Ok(ParsedFile {
        language: Language::Rust,
        path: path.to_path_buf(),
        package_name: module_name_for_path(path),
        is_test_file,
        syntax_error: root.has_error(),
        line_count: source.lines().count(),
        byte_size: source.len(),
        pkg_strings: package_string_literals,
        comments: Vec::new(),
        struct_tags: Vec::new(),
        functions,
        imports,
        symbols,
        class_summaries: Vec::new(),
        package_vars: Vec::new(),
        interfaces: Vec::new(),
        go_structs: Vec::new(),
        module_scope_calls: Vec::new(),
        top_level_bindings: Vec::new(),
        python_models: Vec::new(),
        rust_statics,
        rust_enums,
        structs,
    })
}

fn collect_functions(root: Node<'_>, source: &str, is_test_file: bool) -> Vec<ParsedFunction> {
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
    node: Node<'_>,
    source: &str,
    is_test_file: bool,
    functions: &mut Vec<ParsedFunction>,
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

fn parse_function_node(node: Node<'_>, source: &str, is_test_file: bool) -> Option<ParsedFunction> {
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

    Some(ParsedFunction {
        fingerprint,
        signature_text: source
            .get(node.start_byte()..body_node.start_byte())
            .unwrap_or_default()
            .to_string(),
        body_start_line: body_node.start_position().row + 1,
        calls,
        exception_handlers: Vec::new(),
        has_context_parameter: false,
        is_test_function,
        local_binding_names,
        doc_comment,
        body_text,
        local_strings: local_string_literals,
        normalized_body: String::new(),
        validation_signature: None,
        exception_block_signatures: Vec::new(),
        test_summary: None,
        safety_comment_lines,
        unsafe_lines,
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
        concat_loops: Vec::new(),
        json_loops: Vec::new(),
        db_query_calls: Vec::new(),
        gorm_query_chains: Vec::new(),
        parse_input_calls: Vec::new(),
        gin_calls: Vec::new(),
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
    })
}

fn function_is_async(function_node: Node<'_>, body_node: Node<'_>, source: &str) -> bool {
    source
        .get(function_node.start_byte()..body_node.start_byte())
        .unwrap_or("")
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token == "async")
}

fn collect_await_points(node: Node<'_>) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_await_points(node, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_for_await_points(node: Node<'_>, lines: &mut Vec<usize>) {
    if node.kind() == "await_expression" {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_await_points(child, lines);
    }
}

fn collect_macro_calls(calls: &[CallSite]) -> Vec<MacroCall> {
    calls
        .iter()
        .filter(|call| call.name.ends_with('!'))
        .map(|call| MacroCall {
            line: call.line,
            name: call.name.clone(),
        })
        .collect()
}

fn collect_named_runtime_calls(calls: &[CallSite], names: &[&str]) -> Vec<RuntimeCall> {
    calls
        .iter()
        .filter(|call| names.contains(&call.name.as_str()))
        .map(|call| RuntimeCall {
            line: call.line,
            name: call.name.clone(),
            receiver: call.receiver.clone(),
        })
        .collect()
}

fn collect_future_creations(node: Node<'_>, source: &str) -> Vec<RuntimeCall> {
    let mut futures = Vec::new();
    visit_for_future_creations(node, source, &mut futures);
    futures
}

fn visit_for_future_creations(node: Node<'_>, source: &str, futures: &mut Vec<RuntimeCall>) {
    if node.kind() == "let_declaration"
        && let Some(value_node) = node.child_by_field_name("value")
        && let Some(value_text) = source.get(value_node.byte_range())
    {
        let trimmed = value_text.trim();
        if trimmed.starts_with("async ")
            || trimmed.contains(".fuse()")
            || trimmed.contains("Future")
        {
            futures.push(RuntimeCall {
                line: node.start_position().row + 1,
                name: "future".to_string(),
                receiver: None,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_future_creations(child, source, futures);
    }
}

fn collect_blocking_calls(node: Node<'_>, source: &str, calls: &[CallSite]) -> Vec<RuntimeCall> {
    let mut blocking_calls = calls
        .iter()
        .filter(|call| is_blocking_call(call))
        .map(|call| RuntimeCall {
            line: call.line,
            name: call.name.clone(),
            receiver: call.receiver.clone(),
        })
        .collect::<Vec<_>>();

    visit_textual_blocking_calls(node, source, &mut blocking_calls);
    blocking_calls
        .sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    blocking_calls.dedup_by(|left, right| {
        left.line == right.line && left.name == right.name && left.receiver == right.receiver
    });
    blocking_calls
}

fn is_blocking_call(call: &CallSite) -> bool {
    let receiver = call.receiver.as_deref().unwrap_or_default();
    matches!(
        call.name.as_str(),
        "read_to_string"
            | "read"
            | "read_to_end"
            | "write"
            | "write_all"
            | "open"
            | "create"
            | "metadata"
            | "sleep"
            | "join"
            | "block_on"
    ) || receiver.contains("std::fs")
        || receiver.contains("fs")
        || receiver.contains("std::thread")
        || receiver.contains("File")
}

fn visit_textual_blocking_calls(
    node: Node<'_>,
    source: &str,
    blocking_calls: &mut Vec<RuntimeCall>,
) {
    if let Some(text) = source.get(node.byte_range()) {
        let blocking_name = if text.contains("std::thread::sleep") {
            Some("sleep")
        } else if text.contains("std::fs::") || text.contains("fs::read_to_string") {
            Some("fs")
        } else if text.contains("block_on(") {
            Some("block_on")
        } else {
            None
        };

        if let Some(name) = blocking_name {
            blocking_calls.push(RuntimeCall {
                line: node.start_position().row + 1,
                name: name.to_string(),
                receiver: None,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_textual_blocking_calls(child, source, blocking_calls);
    }
}

fn collect_loop_operation_lines(
    node: Node<'_>,
    source: &str,
) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let mut write_loops = Vec::new();
    let mut line_iteration_loops = Vec::new();
    let mut default_hasher_lines = Vec::new();
    visit_loop_operation_lines(
        node,
        source,
        false,
        &mut write_loops,
        &mut line_iteration_loops,
        &mut default_hasher_lines,
    );
    (write_loops, line_iteration_loops, default_hasher_lines)
}

fn visit_loop_operation_lines(
    node: Node<'_>,
    source: &str,
    in_loop: bool,
    write_loops: &mut Vec<usize>,
    line_iteration_loops: &mut Vec<usize>,
    default_hasher_lines: &mut Vec<usize>,
) {
    let child_in_loop = in_loop || is_loop_node(node.kind());

    if child_in_loop
        && node.kind() == "call_expression"
        && let Some(function_node) = node.child_by_field_name("function")
    {
        let target = render_call_target(function_node, source);
        let (_, name) = split_call_target(&target);
        let line = node.start_position().row + 1;
        if matches!(name.as_str(), "write" | "write_all") || target.contains("File::write") {
            write_loops.push(line);
        }
        if name == "lines" {
            line_iteration_loops.push(line);
        }
        if is_default_hashmap_target(&target) {
            default_hasher_lines.push(line);
        }
    }

    if child_in_loop
        && node.kind() == "macro_invocation"
        && let Some(macro_node) = node.child_by_field_name("macro")
        && let Some(macro_text) = source.get(macro_node.byte_range())
        && matches!(macro_text.trim(), "write" | "writeln")
    {
        write_loops.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_loop_operation_lines(
            child,
            source,
            child_in_loop,
            write_loops,
            line_iteration_loops,
            default_hasher_lines,
        );
    }
}

fn is_loop_node(kind: &str) -> bool {
    matches!(
        kind,
        "for_expression" | "while_expression" | "loop_expression"
    )
}

fn collect_boxed_container_lines(node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_boxed_vec_lines(node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_boxed_vec_lines(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if node.kind() == "let_declaration"
        && let Some(text) = source.get(node.byte_range())
        && contains_boxed_vec_type(text)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_boxed_vec_lines(child, source, lines);
    }
}

fn contains_boxed_vec_type(text: &str) -> bool {
    let Some(vec_start) = text.find("Vec<") else {
        return false;
    };
    text[vec_start + 4..].contains("Box<")
}

fn is_default_hashmap_target(target: &str) -> bool {
    target.contains("HashMap::") && (target.ends_with("::new") || target.ends_with("::default"))
}

fn collect_unsafe_patterns(node: Node<'_>, source: &str) -> Vec<UnsafePattern> {
    let mut patterns = Vec::new();
    visit_for_unsafe_patterns(node, source, &mut patterns);
    patterns.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(left.detail.cmp(&right.detail))
    });
    patterns.dedup_by(|left, right| {
        left.line == right.line && left.kind == right.kind && left.detail == right.detail
    });
    patterns
}

fn visit_for_unsafe_patterns(node: Node<'_>, source: &str, patterns: &mut Vec<UnsafePattern>) {
    if node.kind() == "call_expression"
        && let Some(function_node) = node.child_by_field_name("function")
    {
        let target = render_call_target(function_node, source);
        let (_, name) = split_call_target(&target);
        let kind = match name.as_str() {
            "get_unchecked" | "get_unchecked_mut" => Some(UnsafePatternKind::GetUnchecked),
            "from_raw_parts" | "from_raw_parts_mut" => Some(UnsafePatternKind::RawParts),
            "set_len" => Some(UnsafePatternKind::SetLen),
            "assume_init" => Some(UnsafePatternKind::AssumeInit),
            "transmute" => Some(UnsafePatternKind::Transmute),
            _ => None,
        };

        if let Some(kind) = kind {
            patterns.push(UnsafePattern {
                line: node.start_position().row + 1,
                kind,
                detail: target,
            });
        }
    }

    if matches!(node.kind(), "cast_expression" | "type_cast_expression")
        && let Some(text) = source.get(node.byte_range())
        && (text.contains(" as *const ")
            || text.contains(" as *mut ")
            || text.ends_with(" as *const")
            || text.ends_with(" as *mut"))
    {
        patterns.push(UnsafePattern {
            line: node.start_position().row + 1,
            kind: UnsafePatternKind::RawPointerCast,
            detail: text.trim().to_string(),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_unsafe_patterns(child, source, patterns);
    }
}

fn inside_trait_impl(node: Node<'_>, source: &str, trait_name: &str) -> bool {
    let mut current = node.parent();

    while let Some(parent) = current {
        if parent.kind() == "impl_item" && trait_impl_type(parent, source, trait_name).is_some() {
            return true;
        }
        current = parent.parent();
    }

    false
}

fn function_kind(node: Node<'_>, source: &str) -> &'static str {
    let Some(parameters) = node.child_by_field_name("parameters") else {
        return "function";
    };

    let Some(parameters_text) = source.get(parameters.byte_range()) else {
        return "function";
    };

    if parameters_text.contains("self") {
        "method"
    } else {
        "function"
    }
}

fn enclosing_impl_type(node: Node<'_>, source: &str) -> Option<String> {
    let mut parent = node.parent();

    while let Some(current) = parent {
        if current.kind() == "impl_item" {
            return current
                .child_by_field_name("type")
                .and_then(|type_node| source.get(type_node.byte_range()))
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(ToOwned::to_owned);
        }

        parent = current.parent();
    }

    None
}

fn build_function_fingerprint(
    node: Node<'_>,
    source: &str,
    kind: &str,
    receiver_type: Option<String>,
    call_count: usize,
) -> Option<FunctionFingerprint> {
    let name_node = node.child_by_field_name("name")?;
    let body_node = node.child_by_field_name("body")?;
    let function_text = source.get(node.byte_range())?;
    let name = source.get(name_node.byte_range())?.to_string();
    let comment_lines =
        count_comment_lines(function_text) + count_doc_lines(source, node.start_position().row);
    let code_lines = count_code_lines(function_text);
    let complexity_score = 1 + count_control_nodes(body_node);
    let symmetry_score = compute_symmetry_score(body_node);
    let line_count = node.end_position().row - node.start_position().row + 1;
    let comment_to_code_ratio = if code_lines == 0 {
        0.0
    } else {
        comment_lines as f64 / code_lines as f64
    };

    Some(FunctionFingerprint {
        name,
        kind: kind.to_string(),
        receiver_type,
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        line_count,
        comment_lines,
        code_lines,
        comment_to_code_ratio,
        complexity_score,
        symmetry_score,
        boilerplate_err_guards: 0,
        contains_any_type: false,
        contains_empty_interface: false,
        type_assertion_count: 0,
        call_count,
    })
}

fn collect_calls(node: Node<'_>, source: &str) -> Vec<CallSite> {
    let mut calls = Vec::new();
    visit_for_calls(node, source, &mut calls);
    calls
}

fn visit_for_calls(node: Node<'_>, source: &str, calls: &mut Vec<CallSite>) {
    if node.kind() == "call_expression"
        && let Some(function_node) = node.child_by_field_name("function")
    {
        let function_text = render_call_target(function_node, source);
        let (receiver, name) = split_call_target(&function_text);
        calls.push(CallSite {
            receiver,
            name,
            line: node.start_position().row + 1,
        });
    }

    if node.kind() == "macro_invocation"
        && let Some(macro_node) = node.child_by_field_name("macro")
        && let Some(macro_text) = source.get(macro_node.byte_range())
    {
        let (receiver, name) = split_call_target(macro_text);
        calls.push(CallSite {
            receiver,
            name: format!("{name}!"),
            line: node.start_position().row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_calls(child, source, calls);
    }
}

fn render_call_target(node: Node<'_>, source: &str) -> String {
    if node.kind() == "field_expression" {
        let value = node
            .child_by_field_name("value")
            .map(|value_node| render_call_target(value_node, source));
        let field = node
            .child_by_field_name("field")
            .and_then(|field_node| source.get(field_node.byte_range()))
            .map(str::trim)
            .unwrap_or("");

        return combine_path_prefix(value.as_deref(), field);
    }

    source
        .get(node.byte_range())
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

fn split_call_target(function_text: &str) -> (Option<String>, String) {
    let normalized = function_text.trim();

    if let Some((receiver, name)) = normalized.rsplit_once('.') {
        return (Some(receiver.trim().to_string()), name.trim().to_string());
    }

    if let Some((receiver, name)) = normalized.rsplit_once("::") {
        return (Some(receiver.trim().to_string()), name.trim().to_string());
    }

    (None, normalized.to_string())
}

fn module_name_for_path(path: &Path) -> Option<String> {
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

fn is_test_file(path: &Path) -> bool {
    let in_tests_directory = path
        .components()
        .any(|component| component.as_os_str() == "tests");
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    in_tests_directory || file_name == "tests.rs" || file_name.ends_with("_test.rs")
}

fn extract_doc_comment(source: &str, function_start_row: usize) -> Option<String> {
    let lines = source.lines().collect::<Vec<_>>();
    if function_start_row == 0 || function_start_row > lines.len() {
        return None;
    }

    let mut collected = Vec::new();
    let mut index = function_start_row;

    while index > 0 {
        index -= 1;
        let trimmed = lines[index].trim();

        if trimmed.is_empty() {
            break;
        }

        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            collected.push(
                trimmed
                    .trim_start_matches('/')
                    .trim_start_matches('!')
                    .trim()
                    .to_string(),
            );
            continue;
        }

        if trimmed.ends_with("*/") {
            let mut block_lines = vec![trimmed.trim_end_matches("*/").trim().to_string()];
            while index > 0 {
                index -= 1;
                let block_line = lines[index].trim();
                block_lines.push(block_line.trim_start_matches("/*").trim().to_string());
                if block_line.starts_with("/**") || block_line.starts_with("/*!") {
                    block_lines.reverse();
                    collected.extend(block_lines);
                    return Some(collected.join("\n").trim().to_string());
                }
            }
            break;
        }

        break;
    }

    if collected.is_empty() {
        None
    } else {
        collected.reverse();
        Some(collected.join("\n").trim().to_string())
    }
}

fn function_is_test_only(node: Node<'_>, source: &str, is_test_file: bool) -> bool {
    if is_test_file {
        return true;
    }

    let mut current = Some(node);
    while let Some(candidate) = current {
        if leading_attributes(candidate)
            .into_iter()
            .any(|attribute| attribute_marks_test(attribute, source))
        {
            return true;
        }

        current = candidate.parent();
    }

    false
}

pub(super) fn leading_attributes(node: Node<'_>) -> Vec<Node<'_>> {
    let mut attributes = Vec::new();
    let mut current = node.prev_named_sibling();

    while let Some(sibling) = current {
        if sibling.kind() != "attribute_item" {
            break;
        }

        attributes.push(sibling);
        current = sibling.prev_named_sibling();
    }

    attributes
}

fn attribute_marks_test(node: Node<'_>, source: &str) -> bool {
    let normalized = source
        .get(node.byte_range())
        .unwrap_or("")
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    normalized.contains("cfg(test)")
        || normalized.starts_with("#[test]")
        || normalized.ends_with("::test]")
        || normalized.contains("::test(")
}

fn collect_unsafe_lines(function_node: Node<'_>, body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    let signature_text = source
        .get(function_node.start_byte()..body_node.start_byte())
        .unwrap_or("");

    if signature_text
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token == "unsafe")
    {
        lines.push(function_node.start_position().row + 1);
    }

    visit_for_unsafe_lines(body_node, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn collect_safety_comments(source: &str, function_node: Node<'_>) -> Vec<usize> {
    let lines = source.lines().collect::<Vec<_>>();
    let start = function_node.start_position().row.saturating_sub(2);
    let end = function_node
        .end_position()
        .row
        .min(lines.len().saturating_sub(1));
    let mut safety_lines = Vec::new();

    for (index, line) in lines.iter().enumerate().take(end + 1).skip(start) {
        if line.contains("SAFETY:") {
            safety_lines.push(index + 1);
        }
    }

    safety_lines
}

fn visit_for_unsafe_lines(node: Node<'_>, lines: &mut Vec<usize>) {
    if node.kind() == "unsafe_block" {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_unsafe_lines(child, lines);
    }
}

fn collect_local_strings(node: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_local_strings(node, source, &mut literals);
    literals
}

fn collect_local_bindings(function_node: Node<'_>, source: &str) -> Vec<String> {
    let mut names = Vec::new();

    if let Some(parameters) = function_node.child_by_field_name("parameters") {
        collect_param_bindings(parameters, source, &mut names);
    }

    if let Some(body_node) = function_node.child_by_field_name("body") {
        visit_local_bindings(body_node, source, &mut names);
    }

    names.sort();
    names.dedup();
    names
}

fn collect_param_bindings(node: Node<'_>, source: &str, names: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "parameter" => {
                if let Some(pattern) = child.child_by_field_name("pattern") {
                    collect_ident_patterns(pattern, source, names);
                }
            }
            "self_parameter" => names.push("self".to_string()),
            _ => {}
        }
    }
}

fn visit_local_bindings(node: Node<'_>, source: &str, names: &mut Vec<String>) {
    if node.kind() == "let_declaration"
        && let Some(pattern_node) = node.child_by_field_name("pattern")
    {
        collect_ident_patterns(pattern_node, source, names);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_local_bindings(child, source, names);
    }
}

fn collect_ident_patterns(node: Node<'_>, source: &str, names: &mut Vec<String>) {
    if matches!(node.kind(), "identifier" | "self")
        && let Some(name) = source.get(node.byte_range())
    {
        names.push(name.trim().to_string());
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_ident_patterns(child, source, names);
    }
}

fn visit_local_strings(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if node.kind() == "let_declaration"
        && let Some(pattern_node) = node.child_by_field_name("pattern")
        && pattern_node.kind() == "identifier"
        && let Some(name) = source.get(pattern_node.byte_range())
        && let Some(value_node) = node.child_by_field_name("value")
        && let Some(value) = string_literal_value(value_node, source)
    {
        literals.push(NamedLiteral {
            line: node.start_position().row + 1,
            name: name.trim().to_string(),
            value,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_local_strings(child, source, literals);
    }
}

pub(super) fn string_literal_value(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "string_literal" | "raw_string_literal" => {
            let text = source.get(node.byte_range())?.trim();
            unquote_rust_string(text)
        }
        _ => None,
    }
}

fn unquote_rust_string(text: &str) -> Option<String> {
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return text.get(1..text.len() - 1).map(ToOwned::to_owned);
    }

    if text.starts_with('r') {
        let start = text.find('"')?;
        let end = text.rfind('"')?;
        if end > start {
            return text.get(start + 1..end).map(ToOwned::to_owned);
        }
    }

    None
}

pub(super) fn is_inside_function(node: Node<'_>) -> bool {
    let mut current = node.parent();

    while let Some(parent) = current {
        if parent.kind() == "function_item" {
            return true;
        }

        current = parent.parent();
    }

    false
}

fn count_control_nodes(node: Node<'_>) -> usize {
    let mut total = 0;
    let mut cursor = node.walk();

    for child in node.named_children(&mut cursor) {
        total += usize::from(is_control_node(child.kind()));
        total += count_control_nodes(child);
    }

    total
}

fn is_control_node(kind: &str) -> bool {
    matches!(
        kind,
        "if_expression"
            | "for_expression"
            | "while_expression"
            | "loop_expression"
            | "match_expression"
    )
}

fn compute_symmetry_score(body_node: Node<'_>) -> f64 {
    let mut cursor = body_node.walk();
    let mut statement_kinds = Vec::new();

    for child in body_node.named_children(&mut cursor) {
        statement_kinds.push(child.kind().to_string());
    }

    if statement_kinds.len() < 2 {
        return 0.0;
    }

    statement_kinds.sort();

    let mut best_run = 1usize;
    let mut current_run = 1usize;

    for pair in statement_kinds.windows(2) {
        if pair[0] == pair[1] {
            current_run += 1;
            best_run = best_run.max(current_run);
        } else {
            current_run = 1;
        }
    }

    best_run as f64 / statement_kinds.len() as f64
}

fn count_comment_lines(text: &str) -> usize {
    let mut count = 0usize;
    let mut in_block_comment = false;

    for line in text.split('\n') {
        let trimmed = line.trim();

        if in_block_comment {
            count += 1;
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("//") {
            count += 1;
            continue;
        }

        if trimmed.starts_with("/*") {
            count += 1;
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
        }
    }

    count
}

fn count_code_lines(text: &str) -> usize {
    let mut count = 0usize;
    let mut in_block_comment = false;

    for line in text.split('\n') {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
                let suffix = trimmed
                    .split_once("*/")
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or("");
                if !suffix.is_empty() {
                    count += 1;
                }
            }
            continue;
        }

        if trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("/*") {
            if trimmed.contains("*/") {
                let suffix = trimmed
                    .split_once("*/")
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or("");
                if !suffix.is_empty() {
                    count += 1;
                }
            } else {
                in_block_comment = true;
            }
            continue;
        }

        count += 1;
    }

    count
}

fn count_doc_lines(source: &str, function_start_row: usize) -> usize {
    let lines = source.lines().collect::<Vec<_>>();
    if function_start_row == 0 || function_start_row > lines.len() {
        return 0;
    }

    let mut count = 0usize;
    let mut index = function_start_row;

    while index > 0 {
        index -= 1;
        let trimmed = lines[index].trim();

        if trimmed.is_empty() {
            break;
        }

        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            count += 1;
            continue;
        }

        if trimmed.ends_with("*/") {
            count += 1;
            while index > 0 {
                index -= 1;
                count += 1;
                let block_line = lines[index].trim();
                if block_line.starts_with("/**") || block_line.starts_with("/*!") {
                    return count;
                }
            }
            return count;
        }

        break;
    }

    count
}
