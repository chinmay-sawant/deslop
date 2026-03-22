use std::fs;

use deslop::{scan_repository, ScanOptions};

use super::{create_temp_workspace, write_fixture};

#[test]
fn flags_placeholder_and_low_signal_tests() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "quality_test.go",
        go_fixture!("test_quality_slop.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "test_without_assertion_signal"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "happy_path_only_test"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "placeholder_test_body"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_tests_with_positive_and_negative_signals() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "quality_test.go",
        go_fixture!("test_quality_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "test_without_assertion_signal"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "happy_path_only_test"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "placeholder_test_body"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}