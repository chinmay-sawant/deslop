use std::path::Path;

use super::{evaluate_rust_findings, parser};
use crate::analysis::rust::{alias_lookup, call_matches_import, import_matches_item};
use crate::analysis::test_support::{find_parsed_file, parse_fixture_bundle};
use crate::index::{RepositoryIndex, build_repository_index};

fn parse_source(path: &str, source: &str) -> crate::analysis::ParsedFile {
    let parsed = parser::parse_file(Path::new(path), source);
    assert!(parsed.is_ok(), "rust source should parse");
    match parsed {
        Ok(file) => file,
        Err(_) => unreachable!("asserted above"),
    }
}

fn parse_bundle(relative_path: &str) -> Vec<crate::analysis::ParsedFile> {
    parse_fixture_bundle(relative_path, parser::parse_file)
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
fn grouped_imported_function_matches_current_module_index() {
    let files = parse_bundle("rust/backend/grouped_imported_function.txt");
    let current = find_parsed_file(&files, "/repo/src/analysis/go/mod.rs");
    let index = build_repository_index(Path::new("/repo"), &files);
    let import_aliases = alias_lookup(&current.imports);
    let import_spec = import_aliases
        .get("evaluate_go_file")
        .expect("grouped import should be indexed");

    assert!(call_matches_import(&index, &current.path, import_spec));
    assert_no_rule(current, &index, "hallucinated_import_call", None);
}

#[test]
fn self_group_imported_function_matches_current_module_index() {
    let files = parse_bundle("rust/backend/self_group_imported_function.txt");
    let current = find_parsed_file(&files, "/repo/src/analysis/go/parser/mod.rs");
    let index = build_repository_index(Path::new("/repo"), &files);
    let import_aliases = alias_lookup(&current.imports);

    assert!(call_matches_import(
        &index,
        &current.path,
        import_aliases
            .get("collect_calls")
            .expect("collect_calls import should exist"),
    ));
    assert!(call_matches_import(
        &index,
        &current.path,
        import_aliases
            .get("find_package_name")
            .expect("find_package_name import should exist"),
    ));
    assert_no_rule(current, &index, "hallucinated_import_call", None);
}

#[test]
fn imported_type_receiver_is_not_treated_as_missing_module() {
    let files = parse_bundle("rust/backend/imported_type_receiver.txt");
    let current = find_parsed_file(&files, "/repo/src/analysis/go/parser/mod.rs");
    let index = build_repository_index(Path::new("/repo"), &files);
    let import_aliases = alias_lookup(&current.imports);
    let import_spec = import_aliases
        .get("Error")
        .expect("type import should be indexed");

    assert!(import_matches_item(&index, &current.path, import_spec));
    assert_no_rule(
        current,
        &index,
        "hallucinated_import_call",
        Some("Error::parser_configuration"),
    );
}

#[test]
fn rust_prelude_drop_is_not_flagged_as_local_hallucination() {
    let files = parse_bundle("rust/backend/rust_prelude_drop.txt");
    let current = find_parsed_file(&files, "/repo/src/lib.rs");
    let index = build_repository_index(Path::new("/repo"), &files);

    assert_no_rule(current, &index, "hallucinated_local_call", Some("drop"));
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

#[test]
fn nested_test_module_imports_do_not_leak_into_outer_function_checks() {
    let files = parse_bundle("rust/backend/nested_test_module_outer_scope.txt");
    let current = find_parsed_file(&files, "/repo/src/analysis/python/mod.rs");
    let index = build_repository_index(Path::new("/repo"), &files);

    assert_no_rule(
        current,
        &index,
        "hallucinated_import_call",
        Some("parser::parse_file"),
    );
}

#[test]
fn nested_test_module_super_import_matches_local_child_module() {
    let files = parse_bundle("rust/backend/nested_test_module_super_import.txt");
    let current = find_parsed_file(&files, "/repo/src/analysis/python/mod.rs");
    let index = build_repository_index(Path::new("/repo"), &files);
    let import_aliases = alias_lookup(&current.imports);
    let import_spec = import_aliases
        .get("parser")
        .expect("parser import should exist");

    assert!(import_matches_item(&index, &current.path, import_spec));
    assert_no_rule(
        current,
        &index,
        "hallucinated_import_call",
        Some("parser::parse_file"),
    );
}

#[test]
fn function_scoped_import_before_call_is_not_flagged_as_local() {
    let files = parse_bundle("rust/backend/function_scoped_import_before_call.txt");
    let current = find_parsed_file(&files, "/repo/src/io.rs");
    let index = build_repository_index(Path::new("/repo"), &files);

    assert_no_rule(
        current,
        &index,
        "hallucinated_local_call",
        Some("create_fs_link"),
    );
}

#[test]
fn expression_receivers_are_not_treated_as_import_paths() {
    let files = parse_bundle("rust/backend/expression_receivers.txt");
    let current = find_parsed_file(&files, "/repo/src/lib.rs");
    let index = build_repository_index(Path::new("/repo"), &files);

    assert_no_rule(
        current,
        &index,
        "hallucinated_import_call",
        Some("parser::parse_file"),
    );
}

#[test]
fn path_attribute_child_module_import_is_resolved() {
    let files = parse_bundle("rust/backend/path_attribute_child_module.txt");
    let current = find_parsed_file(&files, "/repo/src/analysis/rust/evaluate.rs");
    let index = build_repository_index(Path::new("/repo"), &files);

    assert_no_rule(current, &index, "hallucinated_import_call", Some("helper"));
}

#[test]
fn path_attribute_parent_module_import_is_resolved() {
    let files = parse_bundle("rust/backend/path_attribute_parent_module.txt");
    let benchmark = find_parsed_file(&files, "/repo/tests/integration_scan/benchmarking.rs");
    let index = build_repository_index(Path::new("/repo"), &files);

    assert_no_rule(
        benchmark,
        &index,
        "hallucinated_import_call",
        Some("FixtureWorkspace::new"),
    );
}

#[test]
fn wildcard_parent_import_resolves_parent_visible_imports() {
    let files = parse_bundle("rust/backend/wildcard_parent_import.txt");
    let clients = find_parsed_file(&files, "/repo/src/data_access/clients.rs");
    let index = build_repository_index(Path::new("/repo"), &files);

    assert_no_rule(
        clients,
        &index,
        "hallucinated_local_call",
        Some("has_sql_like_import"),
    );
}

#[test]
fn for_loop_callable_bindings_are_not_flagged_as_missing() {
    let files = parse_bundle("rust/backend/for_loop_callable_bindings.txt");
    let current = find_parsed_file(&files, "/repo/src/heuristics/engine.rs");
    let index = build_repository_index(Path::new("/repo"), &files);

    assert_no_rule(current, &index, "hallucinated_local_call", Some("rule"));
}

#[test]
fn test_functions_skip_hallucinated_call_checks() {
    let files = parse_bundle("rust/backend/test_functions_skip_hallucinated_call_checks.txt");
    let current = find_parsed_file(&files, "/repo/src/lib.rs");
    let index = build_repository_index(Path::new("/repo"), &files);

    assert_no_rule(current, &index, "hallucinated_import_call", None);
    assert_no_rule(current, &index, "hallucinated_local_call", None);
}
