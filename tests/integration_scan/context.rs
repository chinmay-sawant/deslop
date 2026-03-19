use std::fs;

use goslop::{scan_repository, ScanOptions};

use super::{create_temp_workspace, write_fixture};

#[test]
fn flags_missing_context_for_http_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "http.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/missing_context.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "missing_context"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_context_aware_http_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "http.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/context_aware_http.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "missing_context"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_missing_cancel_calls_for_derived_contexts() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "cancel.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/context_cancel_slop.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "missing_cancel_call" && finding.function_name.as_deref() == Some("Run")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_derived_contexts_with_cancel_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "cancel.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/context_cancel_clean.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "missing_cancel_call"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_sleep_polling_patterns() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "poll.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/sleep_polling.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "sleep_polling"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_sleep_outside_loops() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "poll.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/sleep_clean.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "sleep_polling"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_busy_waiting_select_defaults() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "wait.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/busy_waiting_slop.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "busy_waiting"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_blocking_select_loops_without_default() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "wait.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/busy_waiting_clean.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "busy_waiting"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_missing_context_for_exec_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "exec.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/missing_context_exec.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "missing_context" && finding.message.contains("context-aware work")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_context_aware_exec_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "exec.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/context_aware_exec.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "missing_context"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
