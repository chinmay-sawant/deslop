use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn test_low_signal() {
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

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "test_without_assertion_signal")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "happy_path_only_test")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "placeholder_test_body")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_high_signal() {
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

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "test_without_assertion_signal")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "happy_path_only_test")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "placeholder_test_body")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
