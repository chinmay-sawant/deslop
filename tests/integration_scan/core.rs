use std::fs;

use goslop::{scan_repository, ScanOptions};

use super::{create_temp_workspace, write_fixture};

#[test]
fn scans_go_files_and_extracts_fingerprints() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "main.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/go/simple.go")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(report.functions_found, 2);
    assert!(report.parse_failures.is_empty());
    assert_eq!(report.files[0].package_name.as_deref(), Some("sample"));
    assert!(report.findings.is_empty());

    let names = report.files[0]
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["Add", "Run"]);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn respects_gitignore() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, ".gitignore", "ignored.go\n");
    write_fixture(
        &temp_dir,
        "main.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/go/simple.go")),
    );
    write_fixture(
        &temp_dir,
        "ignored.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/go/simple.go")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn skips_generated_files_and_keeps_syntax_error_flag() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "generated.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/go/generated.go")),
    );
    write_fixture(
        &temp_dir,
        "broken.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/malformed.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 2);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(
        report.files[0].path.file_name().and_then(|name| name.to_str()),
        Some("broken.go")
    );
    assert!(report.files[0].syntax_error);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
