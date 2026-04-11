use std::path::Path;

use proptest::prelude::*;

use super::parse_file;
use crate::analysis::{Language, test_support::load_fixture};

#[test]
fn test_parse_functions() {
    let source = load_fixture("rust/parser/parse_functions.txt");

    let parsed = parse_file(Path::new("src/main.rs"), &source)
        .expect("rust source should parse successfully");

    assert_eq!(parsed.language, Language::Rust);
    assert_eq!(parsed.package_name.as_deref(), Some("main"));
    assert!(!parsed.syntax_error);
    assert_eq!(parsed.functions.len(), 2);
    assert_eq!(parsed.functions[0].fingerprint.name, "sum_pair");
    assert_eq!(parsed.functions[1].fingerprint.kind, "method");
    assert_eq!(
        parsed.functions[1].fingerprint.receiver_type.as_deref(),
        Some("Runner")
    );
    assert!(!parsed.functions[0].is_test_function);
    assert_eq!(parsed.symbols.len(), 2);
}

#[test]
fn test_extract_evidence() {
    let source = load_fixture("rust/parser/extract_evidence.txt");

    let parsed = parse_file(Path::new("src/lib.rs"), &source)
        .expect("rust source should parse successfully");

    assert_eq!(parsed.imports.len(), 3);
    assert!(
        parsed
            .imports
            .iter()
            .any(|import| { import.alias == "fmt" && import.path.contains("std::fmt") })
    );
    assert!(parsed.imports.iter().any(|import| {
        import.alias == "FmtDisplay"
            && import.path == "std::fmt::Display"
            && import.namespace_path.as_deref() == Some("std::fmt")
            && import.imported_name.as_deref() == Some("Display")
    }));
    assert!(
        parsed
            .imports
            .iter()
            .any(|import| { import.alias == "*" && import.path.contains("crate::config") })
    );

    assert_eq!(parsed.pkg_strings.len(), 1);
    assert_eq!(parsed.pkg_strings[0].name, "API_TOKEN");

    assert!(parsed.symbols.iter().any(|symbol| {
        symbol.name == "Runner" && matches!(symbol.kind, crate::model::SymbolKind::Struct)
    }));
    assert!(parsed.symbols.iter().any(|symbol| {
        symbol.name == "Mode" && matches!(symbol.kind, crate::model::SymbolKind::Type)
    }));
    assert!(parsed.symbols.iter().any(|symbol| {
        symbol.name == "Render" && matches!(symbol.kind, crate::model::SymbolKind::Interface)
    }));
    assert!(parsed.symbols.iter().any(|symbol| {
        symbol.name == "Output" && matches!(symbol.kind, crate::model::SymbolKind::Type)
    }));

    let execute = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "execute")
        .expect("execute should be parsed");
    assert!(!execute.is_test_function);
    assert!(
        execute
            .local_binding_names
            .iter()
            .any(|name| name == "self")
    );
    assert_eq!(execute.local_strings.len(), 1);
    assert_eq!(execute.local_strings[0].name, "password");
    assert!(
        execute
            .local_binding_names
            .iter()
            .any(|name| name == "password")
    );
    assert!(execute.calls.iter().any(|call| call.name == "dbg!"));
    assert!(execute.calls.iter().any(|call| call.name == "todo!"));
    assert!(
        execute
            .calls
            .iter()
            .any(|call| { call.receiver.as_deref() == Some("value") && call.name == "unwrap" })
    );
    assert!(execute.rust_evidence().safety_comment_lines.is_empty());
    assert_eq!(execute.rust_evidence().unsafe_lines.len(), 2);

    let test_fn = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "detects_test_only_code")
        .expect("test function should be parsed");
    assert!(test_fn.is_test_function);
}

#[test]
fn test_collects_advanceplan2_rust_summaries() {
    let source = load_fixture("rust/parser/advanceplan2_summaries.txt");

    let parsed = parse_file(Path::new("src/lib.rs"), &source)
        .expect("rust source should parse successfully");

    let config = parsed
        .structs()
        .iter()
        .find(|summary| summary.name == "RequestConfig")
        .expect("RequestConfig should be summarized");
    assert!(
        config
            .attributes
            .iter()
            .any(|attribute| attribute.contains("deny_unknown_fields"))
    );
    assert!(
        config.fields[0]
            .attributes
            .iter()
            .any(|attribute| attribute.contains("serde(default)"))
    );
    assert!(
        config.fields[1]
            .attributes
            .iter()
            .any(|attribute| attribute.contains("serde(flatten)"))
    );

    assert_eq!(parsed.rust_enums().len(), 1);
    assert_eq!(parsed.rust_enums()[0].name, "WireValue");
    assert_eq!(parsed.rust_enums()[0].variant_count, 2);
    assert!(
        parsed.rust_enums()[0]
            .attributes
            .iter()
            .any(|attribute| attribute.contains("untagged"))
    );

    assert_eq!(parsed.rust_statics().len(), 1);
    assert_eq!(parsed.rust_statics()[0].name, "CACHE");
    assert!(parsed.rust_statics()[0].type_text.contains("OnceLock"));
}

#[test]
fn test_collects_include_declarations() {
    let source = load_fixture("rust/parser/include_declarations.txt");

    let parsed = parse_file(Path::new("src/architecture.rs"), &source)
        .expect("rust source should parse successfully");

    let include_paths = parsed
        .rust_include_declarations()
        .iter()
        .map(|declaration| declaration.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        include_paths,
        vec!["architecture/file_rules.rs", "architecture/helpers.rs"]
    );
}

proptest! {
    #[test]
    fn parses_valid_function_names(name in "[a-z][a-z0-9_]{0,12}") {
        let template = load_fixture("rust/parser/valid_function_template.txt");
        let source = template.replace("__FUNCTION_NAME__", &name);

        let parsed = parse_file(Path::new("src/lib.rs"), &source)
            .expect("generated Rust source should parse successfully");

        prop_assert_eq!(parsed.functions.len(), 1);
        prop_assert_eq!(&parsed.functions[0].fingerprint.name, &name);
        prop_assert!(!parsed.syntax_error);
    }
}

#[test]
fn test_syntax_error() {
    let source = load_fixture("rust/parser/syntax_error.txt");

    let parsed = parse_file(Path::new("src/lib.rs"), &source)
        .expect("tree-sitter should recover from syntax errors");

    assert!(parsed.syntax_error);
    assert_eq!(parsed.functions.len(), 1);
}
