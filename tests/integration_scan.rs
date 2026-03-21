#[path = "integration_scan/benchmarking.rs"]
mod benchmarking;
#[path = "integration_scan/consistency.rs"]
mod consistency;
#[path = "integration_scan/concurrency.rs"]
mod concurrency;
#[path = "integration_scan/context.rs"]
mod context;
#[path = "integration_scan/core.rs"]
mod core;
#[path = "integration_scan/data_access.rs"]
mod data_access;
#[path = "integration_scan/hallucination.rs"]
mod hallucination;
#[path = "integration_scan/naming.rs"]
mod naming;
#[path = "integration_scan/performance.rs"]
mod performance;
#[path = "integration_scan/security.rs"]
mod security;
#[path = "integration_scan/test_quality.rs"]
mod test_quality;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use deslop::{scan_repository, ScanOptions};

#[test]
fn flags_error_handling_slop_patterns() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "error_handling.go",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/generic/error_handling_slop.txt"
        )),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "dropped_error"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "panic_on_error"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "error_wrapping_misuse"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_wrapped_error_handling_as_misuse() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "error_handling.go",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/generic/error_handling_clean.txt"
        )),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "error_wrapping_misuse"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "dropped_error"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "panic_on_error"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

fn create_temp_workspace() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("deslop-test-{nonce}"));
    fs::create_dir_all(&dir).expect("temp dir creation should succeed");
    dir
}

fn write_fixture(root: &Path, relative_path: &str, contents: &str) {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dir creation should succeed");
    }
    fs::write(path, contents).expect("fixture write should succeed");
}