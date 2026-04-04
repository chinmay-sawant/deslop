#![allow(dead_code)]

#[path = "calls.rs"]
mod calls;
#[path = "evidence.rs"]
mod evidence;
#[path = "literals.rs"]
mod literals;
#[path = "metrics.rs"]
mod metrics;

use std::path::Path;

use tree_sitter::Parser;

use crate::analysis::{
    AnalysisResult, Error, Language, LanguageFileData, ParsedFile, RustFileData,
};

use super::imports::collect_imports;
use super::items::{
    collect_enum_summaries, collect_pkg_strings, collect_static_summaries,
    collect_struct_summaries, collect_symbols, collect_trait_impls,
};

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
        functions,
        imports,
        symbols,
        module_scope_calls: Vec::new(),
        top_level_bindings: Vec::new(),
        lang: LanguageFileData::Rust(RustFileData {
            rust_statics,
            rust_enums,
            structs,
        }),
    })
}

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

fn function_finding(
    file: &crate::analysis::ParsedFile,
    function: &crate::analysis::ParsedFunction,
    rule_id: &str,
    severity: crate::model::Severity,
    line: usize,
    message: String,
    evidence: Vec<String>,
) -> crate::model::Finding {
    let mut evidence = evidence;
    evidence.push(RUST_GUIDE_REFERENCE.to_string());

    crate::model::Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message,
        evidence,
    }
}

fn file_finding(
    file: &crate::analysis::ParsedFile,
    rule_id: &str,
    severity: crate::model::Severity,
    line: usize,
    message: String,
    evidence: Vec<String>,
) -> crate::model::Finding {
    let mut evidence = evidence;
    evidence.push(RUST_GUIDE_REFERENCE.to_string());

    crate::model::Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: None,
        start_line: line,
        end_line: line,
        message,
        evidence,
    }
}

fn struct_severity(summary: &crate::analysis::StructSummary) -> crate::model::Severity {
    if summary.visibility_pub || summary.fields.iter().any(|field| field.is_pub) {
        crate::model::Severity::Warning
    } else {
        crate::model::Severity::Info
    }
}

fn first_await_after(function: &crate::analysis::ParsedFunction, line: usize) -> Option<usize> {
    function
        .rust_evidence()
        .await_points
        .iter()
        .copied()
        .find(|await_line| *await_line > line)
}

fn secret_like(name: &str) -> bool {
    matches_token(
        name,
        &[
            "password",
            "secret",
            "token",
            "api_key",
            "apikey",
            "access_token",
            "private_key",
            "cert",
            "certificate",
            "auth",
            "key",
        ],
    )
}

fn credential_like(name: &str) -> bool {
    matches_token(
        name,
        &[
            "cert",
            "certificate",
            "key",
            "token",
            "auth",
            "password",
            "secret",
        ],
    )
}

fn enabled_like(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    normalized == "enabled"
        || normalized.ends_with("_enabled")
        || normalized.starts_with("use_")
        || normalized.starts_with("has_")
        || matches_token(name, &["ssl", "tls", "enabled"])
}

fn business_value_like(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    if normalized.ends_with("_ms") || normalized.starts_with("uses_") {
        return false;
    }

    matches_token(
        name,
        &[
            "amount", "price", "cost", "total", "balance", "rate", "money", "username", "email",
            "percent",
        ],
    )
}

fn sensitive_default_like(name: &str) -> bool {
    matches_token(
        name,
        &[
            "port", "token", "password", "api_key", "secret", "key", "enabled",
        ],
    )
}

fn matches_token(name: &str, tokens: &[&str]) -> bool {
    let normalized = name.to_ascii_lowercase();
    tokens
        .iter()
        .any(|token| normalized == *token || normalized.contains(token))
}

fn field_type_mentions(field: &crate::analysis::FieldSummary, text: &str) -> bool {
    field
        .type_text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .contains(text)
}

const RUST_GUIDE_REFERENCE: &str =
    "see guides/rust/heuristics-and-findings.md for remediation examples";
