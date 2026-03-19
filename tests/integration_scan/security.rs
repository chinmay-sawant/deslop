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
