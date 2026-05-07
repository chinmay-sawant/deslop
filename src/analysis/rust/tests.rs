use std::path::Path;

use super::{evaluate_rust_findings, parser};
use crate::analysis::rust::{alias_lookup, call_matches_import, import_matches_item};
use crate::analysis::test_support::find_parsed_file;
use crate::index::{RepositoryIndex, build_repository_index};

fn parse_source(path: &str, source: &str) -> crate::analysis::ParsedFile {
    let parsed = parser::parse_file(Path::new(path), source);
    assert!(parsed.is_ok(), "rust source should parse");
    match parsed {
        Ok(file) => file,
        Err(_) => unreachable!("asserted above"),
    }
}

fn assert_no_rule(
    file: &crate::analysis::ParsedFile,
    index: &RepositoryIndex,
    rule_id: &str,
    message_fragment: Option<&str>,
) {
    assert!(
        !evaluate_rust_findings(file, index).iter().any(|finding| {
            finding.rule_id == rule_id
                && message_fragment.is_none_or(|fragment| finding.message.contains(fragment))
        }),
        "unexpected {rule_id} finding for {}",
        file.path.display()
    );
}

#[test]
fn actual_go_module_imported_heuristic_call_is_resolved() {
    let current = parse_source(
        "/repo/src/scan/evaluate.rs",
        include_str!("../../scan/evaluate.rs"),
    );
    let heuristics = parse_source(
        "/repo/src/heuristics/mod.rs",
        include_str!("../../heuristics/mod.rs"),
    );
    let files = vec![current, heuristics];
    let current = find_parsed_file(&files, "/repo/src/scan/evaluate.rs");
    let index = build_repository_index(Path::new("/repo"), &files);
    let import_aliases = alias_lookup(&current.imports);
    let import_spec = import_aliases
        .get("evaluate_file")
        .expect("evaluate_file import should exist");

    assert!(call_matches_import(&index, &current.path, import_spec));
}

#[test]
fn actual_rust_rule_helpers_reexported_from_heuristics_are_resolved() {
    let current = parse_source(
        "/repo/src/analysis/rust/evaluate.rs",
        include_str!("evaluate.rs"),
    );
    let heuristics = parse_source(
        "/repo/src/heuristics/mod.rs",
        include_str!("../../heuristics/mod.rs"),
    );
    let engine = parse_source(
        "/repo/src/heuristics/engine.rs",
        include_str!("../../heuristics/engine.rs"),
    );
    let files = vec![current, heuristics, engine];
    let current = find_parsed_file(&files, "/repo/src/analysis/rust/evaluate.rs");
    let index = build_repository_index(Path::new("/repo"), &files);

    assert_no_rule(
        current,
        &index,
        "hallucinated_import_call",
        Some("extend_file_rules"),
    );
    assert_no_rule(
        current,
        &index,
        "hallucinated_import_call",
        Some("extend_function_rules"),
    );
}

#[test]
fn actual_go_parser_error_import_is_indexed_as_item() {
    let current = parse_source(
        "/repo/src/analysis/go/parser/mod.rs",
        include_str!("../go/parser/mod.rs"),
    );
    let analysis = parse_source("/repo/src/analysis/mod.rs", include_str!("../mod.rs"));
    let error = parse_source("/repo/src/analysis/error.rs", include_str!("../error.rs"));
    let files = vec![current, analysis, error];
    let current = find_parsed_file(&files, "/repo/src/analysis/go/parser/mod.rs");
    let index = build_repository_index(Path::new("/repo"), &files);
    let import_aliases = alias_lookup(&current.imports);
    let import_spec = import_aliases
        .get("Error")
        .expect("Error import should exist");

    assert!(import_matches_item(&index, &current.path, import_spec));
}
