use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn scans_rust_files_and_extracts_fingerprints() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "src/main.rs", rust_fixture!("simple.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(report.functions_found, 2);
    assert!(report.parse_failures.is_empty());
    assert_eq!(report.files[0].package_name.as_deref(), Some("main"));

    let names = report.files[0]
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["sum_pair", "render_summary"]);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn keeps_rust_syntax_error_files_in_the_report() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "src/lib.rs", rust_fixture!("broken.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert!(report.files[0].syntax_error);
    assert!(report.parse_failures.is_empty());

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn scans_mixed_go_and_rust_repositories_without_a_language_flag() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "main.go", go_fixture!("simple.go"));
    write_fixture(&temp_dir, "src/main.rs", rust_fixture!("simple.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 2);
    assert_eq!(report.files_analyzed, 2);
    assert!(report.parse_failures.is_empty());

    let analyzed_paths = report
        .files
        .iter()
        .map(|file| {
            file.path
                .strip_prefix(&temp_dir)
                .expect("report path should stay under the temp dir")
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(analyzed_paths, vec!["main.go", "src/main.rs"]);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_the_initial_rust_rule_pack() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "src/lib.rs", rust_fixture!("rule_pack_positive.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "todo_macro_leftover"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "unimplemented_macro_leftover"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "dbg_macro_leftover"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "panic_macro_leftover"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "unreachable_macro_leftover"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "todo_doc_comment_leftover"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "fixme_doc_comment_leftover"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "hack_doc_comment_leftover"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "unwrap_in_non_test_code"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "expect_in_non_test_code"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "unsafe_without_safety_comment"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn suppresses_test_only_rust_rule_hits_and_accepts_documented_unsafe() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "src/lib.rs", rust_fixture!("rule_pack_negative.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "todo_macro_leftover"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "unimplemented_macro_leftover"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "dbg_macro_leftover"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "panic_macro_leftover"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "unreachable_macro_leftover"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "todo_doc_comment_leftover"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "fixme_doc_comment_leftover"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "hack_doc_comment_leftover"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "unwrap_in_non_test_code"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "expect_in_non_test_code"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "unsafe_without_safety_comment"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_rust_local_imported_call_hallucinations() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "src/lib.rs",
        rust_fixture!("hallucinated_import_positive_main.txt"),
    );
    write_fixture(
        &temp_dir,
        "src/config/render.rs",
        rust_fixture!("hallucinated_import_positive_render.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && finding.function_name.as_deref() == Some("run_render")
            && finding.message.contains("render::missing_formatter")
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && finding.function_name.as_deref() == Some("run_missing_module")
            && finding.message.contains("helpers::load")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn resolves_rust_local_imported_calls_for_self_and_super_modules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "src/config/mod.rs",
        rust_fixture!("hallucinated_import_negative_mod.txt"),
    );
    write_fixture(
        &temp_dir,
        "src/config/render.rs",
        rust_fixture!("hallucinated_import_negative_render.txt"),
    );
    write_fixture(
        &temp_dir,
        "src/config/sub/helpers.rs",
        rust_fixture!("hallucinated_import_negative_helpers.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && matches!(
                finding.function_name.as_deref(),
                Some("run_render") | Some("run_helper")
            )
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_rust_direct_call_hallucinations_for_imported_functions_and_same_module_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "src/lib.rs",
        rust_fixture!("direct_call_hallucination_positive.txt"),
    );
    write_fixture(
        &temp_dir,
        "src/config/render.rs",
        rust_fixture!("direct_call_hallucination_render.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && finding.function_name.as_deref() == Some("run_direct_import")
            && finding.message.contains("render_missing")
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_local_call"
            && finding.function_name.as_deref() == Some("run_same_module")
            && finding.message.contains("missing_local")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn accepts_rust_direct_calls_when_imports_locals_and_closures_exist() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "src/lib.rs",
        rust_fixture!("direct_call_hallucination_negative.txt"),
    );
    write_fixture(
        &temp_dir,
        "src/config/render.rs",
        rust_fixture!("direct_call_hallucination_render.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        matches!(
            finding.function_name.as_deref(),
            Some("run_direct_import") | Some("run_same_module") | Some("run_local_closure") | Some("run_constructor")
        ) && matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        )
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}