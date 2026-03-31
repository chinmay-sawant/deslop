use std::path::Path;

use crate::model::SymbolKind;

use super::{comments::extract_doc_comment, general::alias_from_path, parse_file};

macro_rules! go_parser_fixture {
    ($name:literal) => {
        include_str!(concat!("../../../../tests/fixtures/go/parser/", $name, ".txt"))
    };
}

#[test]
fn test_import_alias() {
    assert_eq!(alias_from_path("github.com/acme/utils"), "utils");
}

#[test]
fn test_alias_symbols() {
    let source = go_parser_fixture!("alias_symbols");

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    assert!(parsed.symbols.iter().any(|symbol| {
        symbol.name == "IsCustomFont" && matches!(symbol.kind, SymbolKind::Function)
    }));
    assert!(
        !parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "PlainValue")
    );
}

#[test]
fn test_doc_comment() {
    let source = go_parser_fixture!("doc_comment");

    let comment = extract_doc_comment(source, 2).expect("doc comment should exist");
    assert_eq!(
        comment,
        "Run Processes The Input\nThis function does X by doing Y because Z"
    );
}

#[test]
fn test_error_handling() {
    let source = go_parser_fixture!("error_handling");

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let run = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Run")
        .expect("Run should be parsed");
    let log_only = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "LogOnly")
        .expect("LogOnly should be parsed");

    assert_eq!(run.dropped_errors, vec![9]);
    assert_eq!(run.panic_errors, vec![10]);
    assert_eq!(run.errorf_calls.len(), 1);
    assert!(run.errorf_calls[0].mentions_err);
    assert!(!run.errorf_calls[0].uses_percent_w);
    assert_eq!(log_only.panic_errors, vec![17]);
}

#[test]
fn test_ctx_sleep() {
    let source = go_parser_fixture!("ctx_sleep");

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let poll = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Poll")
        .expect("Poll should be parsed");

    assert!(poll.has_context_parameter);
    assert_eq!(poll.sleep_loops, vec![10]);
}

#[test]
fn test_ctx_busy_json() {
    let source = go_parser_fixture!("ctx_busy_json");

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let run = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Run")
        .expect("Run should be parsed");

    assert_eq!(run.context_factory_calls.len(), 1);
    assert_eq!(run.context_factory_calls[0].cancel_name, "cancel");
    assert_eq!(run.context_factory_calls[0].factory_name, "WithTimeout");
    assert_eq!(run.busy_wait_lines, vec![14]);
    assert_eq!(run.json_loops, vec![21]);
}

#[test]
fn test_concurrency_db() {
    let source = go_parser_fixture!("concurrency_db");

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let run = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Run")
        .expect("Run should be parsed");

    assert_eq!(run.unmanaged_goroutines, vec![11]);
    assert_eq!(run.mutex_loops, vec![18]);
    assert_eq!(run.alloc_loops, vec![23]);
    assert_eq!(run.fmt_loops, vec![21]);
    assert_eq!(run.reflect_loops, vec![22]);
    assert_eq!(run.db_query_calls.len(), 1);
    assert!(run.db_query_calls[0].in_loop);
    assert_eq!(
        run.db_query_calls[0].query_text.as_deref(),
        Some("SELECT * FROM widgets WHERE name LIKE '%foo%'")
    );
}

#[test]
fn test_concat_goroutine() {
    let source = go_parser_fixture!("concat_goroutine");

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let build = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Build")
        .expect("Build should be parsed");

    assert_eq!(build.concat_loops, vec![6]);
    assert_eq!(build.goroutines, vec![7]);
    assert_eq!(build.loop_goroutines, vec![7]);
}

#[test]
fn test_pkg_literals() {
    let source = go_parser_fixture!("pkg_literals");

    let parsed = parse_file(Path::new("sample_test.go"), source).expect("parse should work");
    let test_fn = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "TestUser")
        .expect("TestUser should be parsed");

    assert!(parsed.is_test_file);
    assert_eq!(parsed.pkg_strings.len(), 1);
    assert_eq!(parsed.pkg_strings[0].name, "apiToken");
    assert_eq!(parsed.struct_tags.len(), 1);
    assert_eq!(parsed.struct_tags[0].field_name, "Name");
    assert!(parsed.symbols.iter().any(|symbol| {
        symbol.name == "NameValue"
            && symbol.receiver_type.as_deref() == Some("User")
            && symbol.receiver_is_pointer == Some(false)
    }));
    assert!(parsed.symbols.iter().any(|symbol| {
        symbol.name == "SetName"
            && symbol.receiver_type.as_deref() == Some("User")
            && symbol.receiver_is_pointer == Some(true)
    }));
    assert_eq!(test_fn.local_strings.len(), 1);
    assert_eq!(test_fn.local_strings[0].name, "token");
    assert!(test_fn.test_summary.is_some());
    assert_eq!(
        test_fn
            .test_summary
            .as_ref()
            .map(|summary| summary.production_calls),
        Some(2)
    );
}

#[test]
fn test_imports_preserve_source_order_and_lines() {
    let source = go_parser_fixture!("imports_preserve_source_order");

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let imports = parsed
        .imports
        .iter()
        .map(|import| (import.path.as_str(), import.line, import.group_line))
        .collect::<Vec<_>>();

    assert_eq!(
        imports,
        vec![
            ("github.com/acme/widgets", 4, 3),
            ("fmt", 5, 3),
            ("strings", 6, 3),
        ]
    );
}

#[test]
fn test_collects_package_vars_interfaces_structs_and_signature_text() {
    let source = go_parser_fixture!("package_vars_interfaces_structs_signature");

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    assert_eq!(parsed.package_vars.len(), 2);
    assert_eq!(parsed.package_vars[0].name, "DefaultStore");
    assert_eq!(parsed.package_vars[1].type_text.as_deref(), Some("int"));

    assert_eq!(parsed.interfaces.len(), 1);
    assert_eq!(parsed.interfaces[0].name, "Store");
    assert_eq!(parsed.interfaces[0].methods, vec!["Save", "Load"]);

    assert_eq!(parsed.go_structs.len(), 1);
    assert_eq!(parsed.go_structs[0].name, "Service");
    assert_eq!(parsed.go_structs[0].fields.len(), 2);
    assert_eq!(parsed.go_structs[0].fields[0].type_text, "Store");

    let run = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Run")
        .expect("Run should be parsed");
    assert!(run.signature_text.contains("dryRun bool"));
    assert_eq!(run.body_start_line, 16);
}

#[test]
fn test_collects_gorm_query_chain_summaries() {
    let source = go_parser_fixture!("gorm_query_chains");

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let handle = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Handle")
        .expect("Handle should be parsed");

    assert_eq!(handle.gorm_query_chains.len(), 2);
    assert_eq!(handle.gorm_query_chains[0].terminal_method, "Count");
    assert_eq!(handle.gorm_query_chains[1].terminal_method, "Find");
    assert_eq!(
        handle.gorm_query_chains[1]
            .steps
            .iter()
            .map(|step| step.method_name.as_str())
            .collect::<Vec<_>>(),
        vec!["Model", "Preload", "Offset", "Limit", "Find"]
    );
    assert!(handle.gorm_query_chains[1].in_loop == false);
}

#[test]
fn test_collects_gin_calls_and_parse_input_summaries() {
    let source = go_parser_fixture!("gin_calls_and_parse_input_summaries");

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let handle = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Handle")
        .expect("Handle should be parsed");

    assert_eq!(
        handle
            .gin_calls
            .iter()
            .map(|call| call.operation.as_str())
            .collect::<Vec<_>>(),
        vec![
            "get_raw_data",
            "read_request_body",
            "should_bind_json",
            "should_bind_query",
            "indented_json",
            "copy",
        ]
    );
    assert_eq!(handle.gin_calls[0].assigned_binding.as_deref(), Some("raw"));
    assert_eq!(handle.gin_calls[1].assigned_binding.as_deref(), Some("payload"));
    assert!(handle
        .gin_calls
        .iter()
        .find(|call| call.operation == "copy")
        .is_some_and(|call| call.in_loop));
    assert_eq!(handle.parse_input_calls.len(), 2);
    assert!(handle
        .parse_input_calls
        .iter()
        .all(|call| call.input_binding.as_deref() == Some("body")));
}
