use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn test_missing_ctx_http() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "http.go", go_fixture!("missing_context.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "missing_context")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_ctx_http() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "http.go", go_fixture!("context_aware_http.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "missing_context")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_missing_cancel() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "cancel.go",
        go_fixture!("context_cancel_slop.txt"),
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
fn test_ctx_cancel() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "cancel.go",
        go_fixture!("context_cancel_clean.txt"),
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
            .any(|finding| finding.rule_id == "missing_cancel_call")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_sleep_loops() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "poll.go", go_fixture!("sleep_polling.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "sleep_polling")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_sleep_ok() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "poll.go", go_fixture!("sleep_clean.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "sleep_polling")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_busy_wait() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "wait.go", go_fixture!("busy_waiting_slop.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "busy_waiting")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_select_ok() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "wait.go", go_fixture!("busy_waiting_clean.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "busy_waiting")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_missing_ctx_exec() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "exec.go",
        go_fixture!("missing_context_exec.txt"),
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
fn test_ctx_exec() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "exec.go", go_fixture!("context_aware_exec.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "missing_context")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
