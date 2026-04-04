
use deslop::{ScanOptions, scan_repository};

use super::FixtureWorkspace;

#[test]
fn test_low_signal() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("quality_test.go",
        go_fixture!("test_quality_slop.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
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

    }

#[test]
fn test_high_signal() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("quality_test.go",
        go_fixture!("test_quality_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
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

    }
