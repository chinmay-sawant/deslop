use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn test_db_slop() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "db.go", go_fixture!("db_query_slop.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "n_plus_one_query")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "wide_select_query")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "likely_unindexed_query")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_db_clean() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "db.go", go_fixture!("db_query_clean.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "n_plus_one_query")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "wide_select_query")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "likely_unindexed_query")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
