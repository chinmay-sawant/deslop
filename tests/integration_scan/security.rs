use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn flags_weak_crypto_usage() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "crypto.go", go_fixture!("weak_crypto.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "weak_crypto")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_secrets() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "secrets.go",
        go_fixture!("hardcoded_secret_slop.txt"),
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
            .any(|finding| finding.rule_id == "hardcoded_secret")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_env_secrets() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "secrets.go",
        go_fixture!("hardcoded_secret_clean.txt"),
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
            .any(|finding| finding.rule_id == "hardcoded_secret")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_sql_slop() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "query.go",
        go_fixture!("sql_string_concat_slop.txt"),
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
            .any(|finding| finding.rule_id == "sql_string_concat")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_sql_ok() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "query.go",
        go_fixture!("sql_string_concat_clean.txt"),
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
            .any(|finding| finding.rule_id == "sql_string_concat")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
