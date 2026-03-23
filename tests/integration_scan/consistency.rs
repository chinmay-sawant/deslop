use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn test_mixed_receivers() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "model.go",
        go_fixture!("receiver_struct_slop.txt"),
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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_clean_consistency() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "model.go",
        go_fixture!("receiver_struct_clean.txt"),
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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
