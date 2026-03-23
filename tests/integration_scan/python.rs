use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn test_python_fingerprints() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "app.py", python_fixture!("simple.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(report.functions_found, 2);
    assert!(report.parse_failures.is_empty());
    assert_eq!(report.files[0].package_name.as_deref(), Some("app"));

    let names = report.files[0]
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["build_summary", "render"]);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_syntax() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "broken.py", python_fixture!("broken.txt"));

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
fn test_python_mixed_repo() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "app.py", python_fixture!("simple.txt"));
    write_fixture(&temp_dir, "main.go", go_fixture!("simple.go"));
    write_fixture(&temp_dir, "src/main.rs", rust_fixture!("simple.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 3);
    assert_eq!(report.files_analyzed, 3);
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
    assert_eq!(analyzed_paths, vec!["app.py", "main.go", "src/main.rs"]);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_rust_mixed_repo() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/render/service.py",
        python_fixture!("simple.txt"),
    );
    write_fixture(&temp_dir, "pkg/render/lib.rs", rust_fixture!("simple.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 2);
    assert_eq!(report.files_analyzed, 2);
    assert!(report.parse_failures.is_empty());

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_same_directory_mixed_repo() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/render/__init__.py",
        python_fixture!("simple.txt"),
    );
    write_fixture(&temp_dir, "pkg/render/main.go", go_fixture!("simple.go"));
    write_fixture(&temp_dir, "pkg/render/lib.rs", rust_fixture!("simple.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 3);
    assert_eq!(report.files_analyzed, 3);
    assert!(report.parse_failures.is_empty());

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "service.py",
        python_fixture!("rule_pack_positive.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "blocking_sync_io_in_async")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "exception_swallowed")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "eval_exec_usage")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "print_debugging_leftover")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_rule_suppressions() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "service.py",
        python_fixture!("rule_pack_negative.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "blocking_sync_io_in_async")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "exception_swallowed")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "eval_exec_usage")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "print_debugging_leftover")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_test_rule_suppressions() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "tests/test_service.py",
        python_fixture!("rule_pack_test_only.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        matches!(
            finding.rule_id.as_str(),
            "string_concat_in_loop"
                | "blocking_sync_io_in_async"
                | "full_dataset_load"
                | "exception_swallowed"
                | "eval_exec_usage"
                | "print_debugging_leftover"
        )
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
