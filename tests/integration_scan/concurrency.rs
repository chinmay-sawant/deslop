use std::fs;

use deslop::{scan_repository, ScanOptions};

use super::{create_temp_workspace, write_fixture};

#[test]
fn flags_goroutines_without_coordination() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "go_routine.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/goroutine_slop.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "goroutine_without_coordination"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "goroutine_spawn_in_loop"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_goroutines_with_waitgroup_coordination() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "go_routine.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/goroutine_clean.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "goroutine_without_coordination"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "goroutine_spawn_in_loop"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_goroutine_shutdown_and_mutex_contention_patterns() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "concurrency.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/concurrency_slop.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "goroutine_without_shutdown_path"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "mutex_in_loop"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "blocking_call_while_locked"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_shutdown_and_mutex_patterns_when_signals_are_absent() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "concurrency.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/concurrency_clean.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "goroutine_without_shutdown_path"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "blocking_call_while_locked"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
