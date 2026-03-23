use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn test_unmanaged_goroutines() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "go_routine.go",
        go_fixture!("goroutine_slop.txt"),
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
            .any(|finding| finding.rule_id == "goroutine_without_coordination")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "goroutine_spawn_in_loop")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_coordination() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "go_routine.go",
        go_fixture!("goroutine_clean.txt"),
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
            .any(|finding| finding.rule_id == "goroutine_without_coordination")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "goroutine_spawn_in_loop")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_shutdown_mutex() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "concurrency.go",
        go_fixture!("concurrency_slop.txt"),
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
            .any(|finding| finding.rule_id == "goroutine_without_shutdown_path")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "mutex_in_loop")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "blocking_call_while_locked")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_no_slop() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "concurrency.go",
        go_fixture!("concurrency_clean.txt"),
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
            .any(|finding| finding.rule_id == "goroutine_without_shutdown_path")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "blocking_call_while_locked")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
