use std::fs;

use goslop::{scan_repository, ScanOptions};

use super::{create_temp_workspace, write_fixture};

#[test]
fn flags_weak_crypto_usage() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "crypto.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/weak_crypto.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "weak_crypto"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_hardcoded_secret_patterns() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "secrets.go",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/generic/hardcoded_secret_slop.txt"
        )),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "hardcoded_secret"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_environment_loaded_secrets() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "secrets.go",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/generic/hardcoded_secret_clean.txt"
        )),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "hardcoded_secret"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_dynamic_sql_query_construction() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "query.go",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/generic/sql_string_concat_slop.txt"
        )),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "sql_string_concat"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_parameterized_sql_queries() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "query.go",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/generic/sql_string_concat_clean.txt"
        )),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "sql_string_concat"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
