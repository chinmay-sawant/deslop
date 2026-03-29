use std::path::Path;

use super::parse_file;

macro_rules! python_parser_fixture {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/python/parser/",
            $path
        ))
    };
}

#[test]
fn test_python_parser_extracts_functions_imports_and_strings() {
    let source = python_parser_fixture!("async_calls_positive.txt");

    let parsed =
        parse_file(Path::new("pkg/service.py"), source).expect("python parsing should succeed");

    assert_eq!(parsed.package_name.as_deref(), Some("service"));
    assert_eq!(parsed.imports.len(), 2);
    assert_eq!(parsed.pkg_strings.len(), 1);
    assert_eq!(parsed.pkg_strings[0].name, "API_TOKEN");
    assert_eq!(parsed.functions.len(), 3);

    let fetch = &parsed.functions[0];
    assert_eq!(fetch.fingerprint.name, "fetch_profile");
    assert_eq!(fetch.fingerprint.kind, "async_function");
    assert_eq!(fetch.local_strings.len(), 2);
    assert_eq!(fetch.concat_loops, vec![15]);
    assert!(fetch.exception_handlers.is_empty());
    assert!(fetch.doc_comment.is_some());
    assert!(
        fetch
            .calls
            .iter()
            .any(|call| call.receiver.as_deref() == Some("req") && call.name == "get")
    );

    let render = &parsed.functions[1];
    assert_eq!(render.fingerprint.kind, "method");
    assert_eq!(
        render.fingerprint.receiver_type.as_deref(),
        Some("Renderer")
    );

    let helper = &parsed.functions[2];
    assert_eq!(helper.fingerprint.name, "helper");
    assert!(helper.calls.iter().any(|call| call.name == "print"));
}

#[test]
fn test_python_parser_marks_syntax_errors() {
    let source = r#"
def broken(
    return 1
"#;

    let parsed = parse_file(Path::new("broken.py"), source)
        .expect("python parsing should still return a parsed file");

    assert!(parsed.syntax_error);
}

#[test]
fn test_python_test_detection() {
    let source = r#"
class TestClient:
    def test_fetch(self):
        self.assertEqual(fetch(), 1)
"#;

    let parsed = parse_file(Path::new("tests/test_client.py"), source)
        .expect("python parsing should succeed");

    assert!(parsed.is_test_file);
    assert!(parsed.functions[0].is_test_function);
    assert!(parsed.functions[0].test_summary.is_some());
}

#[test]
fn test_python_exception_handler_evidence() {
    let positive = parse_file(
        Path::new("config.py"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/python/maintainability/exception_shapes_positive.txt"
        )),
    )
    .expect("python parsing should succeed");
    let negative = parse_file(
        Path::new("config_safe.py"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/python/maintainability/exception_shapes_negative.txt"
        )),
    )
    .expect("python parsing should succeed");

    let load_config = &positive.functions[0];
    assert_eq!(load_config.exception_handlers.len(), 1);
    assert!(load_config.exception_handlers[0].is_broad);
    assert!(load_config.exception_handlers[0].suppresses);
    assert_eq!(
        load_config.exception_handlers[0].action.as_deref(),
        Some("pass")
    );

    let recover_config = &negative.functions[0];
    assert_eq!(recover_config.exception_handlers.len(), 1);
    assert!(!recover_config.exception_handlers[0].is_broad);
}

#[test]
fn test_python_async_io_fixture_contract() {
    let positive = parse_file(
        Path::new("pkg/network_sync.py"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/python/performance/async_io_positive.txt"
        )),
    )
    .expect("python parsing should succeed");
    let negative = parse_file(
        Path::new("pkg/network_async.py"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/python/performance/async_io_negative.txt"
        )),
    )
    .expect("python parsing should succeed");

    let sync_reports = &positive.functions[0];
    assert!(sync_reports.fingerprint.kind.starts_with("async_"));
    assert!(sync_reports.is_async);
    assert!(sync_reports.calls.iter().any(|call| call.name == "get"));

    let async_reports = &negative.functions[0];
    assert!(async_reports.fingerprint.kind.starts_with("async_"));
    assert!(async_reports.is_async);
    assert!(async_reports.calls.iter().any(|call| call.name == "get"));
}

#[test]
fn test_python_type_hint_fixture_contract() {
    let positive = parse_file(
        Path::new("pkg/api_types.py"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/python/maintainability/type_hints_positive.txt"
        )),
    )
    .expect("python parsing should succeed");
    let negative = parse_file(
        Path::new("pkg/api_types_partial.py"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/python/maintainability/type_hints_negative.txt"
        )),
    )
    .expect("python parsing should succeed");

    assert!(positive.functions[0].has_complete_type_hints);
    assert!(!negative.functions[0].has_complete_type_hints);
}

#[test]
fn test_python_phase4_parser_evidence() {
    let source = python_parser_fixture!("class_summary_positive.txt");

    let parsed =
        parse_file(Path::new("pkg/service.py"), source).expect("python parsing should succeed");

    let process_items = &parsed.functions[0];
    assert_eq!(process_items.none_comparison_lines, vec![3]);
    assert_eq!(process_items.redundant_return_none_lines, vec![4]);
    assert_eq!(process_items.side_effect_comprehension_lines, vec![6]);
    assert_eq!(process_items.list_materialization_lines, vec![7]);
    assert_eq!(process_items.deque_operation_lines, vec![9]);
    assert_eq!(process_items.temp_collection_lines, vec![13]);
    assert_eq!(process_items.list_membership_loop_lines, vec![15]);
    assert_eq!(process_items.repeated_len_loop_lines, vec![12]);
    assert_eq!(process_items.builtin_candidate_lines, vec![12]);
    assert!(process_items.has_varargs);
    assert!(process_items.has_kwargs);
    assert!(!process_items.has_complete_type_hints);
    assert!(process_items.validation_signature.is_some());
    assert!(!process_items.normalized_body.is_empty());

    let scan_tree = &parsed.functions[1];
    assert_eq!(scan_tree.recursive_call_lines, vec![23]);

    let read_config = &parsed.functions[2];
    assert_eq!(read_config.missing_context_manager_lines, vec![26]);

    assert_eq!(parsed.class_summaries.len(), 2);
    let payload_manager = &parsed.class_summaries[1];
    assert_eq!(payload_manager.name, "PayloadManager");
    assert_eq!(payload_manager.method_count, 3);
    assert_eq!(payload_manager.instance_attribute_count, 13);
    assert_eq!(payload_manager.public_method_count, 2);
    assert_eq!(payload_manager.base_classes, vec!["BaseManager"]);
    assert_eq!(payload_manager.constructor_collaborator_count, 3);
}

#[test]
fn test_python_init_reexports_are_indexed_as_symbols() {
    let source = python_parser_fixture!("reexports_positive.txt");

    let parsed = parse_file(Path::new("pkg/widgets/__init__.py"), source)
        .expect("python parsing should succeed");

    assert!(parsed.imports.iter().all(|import| import.is_public));
    assert!(
        parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "WidgetTemplate")
    );
    assert!(
        parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "LayoutConfig")
    );
    assert!(parsed.symbols.iter().any(|symbol| symbol.name == "Heading"));
    assert!(
        parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "render_widget")
    );
}

#[test]
fn test_python_non_init_relative_imports_do_not_become_public_reexports() {
    let source = python_parser_fixture!("reexports_negative.txt");

    let parsed = parse_file(Path::new("pkg/widgets/helpers.py"), source)
        .expect("python parsing should succeed");

    assert!(parsed.imports.iter().all(|import| !import.is_public));
    assert!(parsed.symbols.iter().all(|symbol| {
        !matches!(
            symbol.name.as_str(),
            "WidgetTemplate" | "LayoutConfig" | "_HiddenWidget" | "render_widget"
        )
    }));
}

#[test]
fn test_python_parenthesized_from_import_ignores_inline_comments() {
    let source = python_parser_fixture!("parenthesized_imports_positive.txt");

    let parsed = parse_file(Path::new("tests/test_widgets.py"), source)
        .expect("python parsing should succeed");

    assert!(
        parsed
            .imports
            .iter()
            .any(|import| import.alias == "WidgetTemplate")
    );
    assert!(
        parsed
            .imports
            .iter()
            .any(|import| import.alias == "LayoutConfig")
    );
    assert!(
        parsed
            .imports
            .iter()
            .any(|import| import.alias == "Heading")
    );
}

#[test]
fn test_python_symbol_extraction_preserves_naming_styles() {
    let source = r#"
class HTTPClient:
    pass

class widget_renderer:
    pass

def render_json():
    return HTTPClient()

def buildHTML():
    return widget_renderer()
"#;

    let parsed =
        parse_file(Path::new("pkg/naming_mix.py"), source).expect("python parsing should succeed");

    let symbol_names = parsed
        .symbols
        .iter()
        .map(|symbol| symbol.name.as_str())
        .collect::<Vec<_>>();

    assert!(symbol_names.contains(&"HTTPClient"));
    assert!(symbol_names.contains(&"widget_renderer"));
    assert!(symbol_names.contains(&"render_json"));
    assert!(symbol_names.contains(&"buildHTML"));
}
