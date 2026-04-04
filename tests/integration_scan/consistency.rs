
use deslop::{ScanOptions, scan_repository};

use super::FixtureWorkspace;

#[test]
fn test_mixed_receivers() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("model.go",
        go_fixture!("receiver_struct_slop.txt"),
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
            .any(|finding| finding.rule_id == "mixed_receiver_kinds")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "malformed_struct_tag")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "duplicate_struct_tag_key")
    );

    }

#[test]
fn test_clean_consistency() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("model.go",
        go_fixture!("receiver_struct_clean.txt"),
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
            .any(|finding| finding.rule_id == "mixed_receiver_kinds")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "malformed_struct_tag")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "duplicate_struct_tag_key")
    );

    }
