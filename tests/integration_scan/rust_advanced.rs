use std::fs;
use std::process::Command;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn test_rust_domain_modeling_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "src/lib.rs",
        rust_fixture!("domain_modeling/positive.rs"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "rust_domain_raw_primitive",
        "rust_domain_float_for_money",
        "rust_domain_impossible_combination",
        "rust_domain_default_produces_invalid",
        "rust_debug_secret",
        "rust_serde_sensitive_deserialize",
        "rust_serde_sensitive_serialize",
    ] {
        assert!(
            report.findings.iter().any(|finding| finding.rule_id == rule_id),
            "expected finding {rule_id:?}, got {:?}",
            report.findings.iter().map(|finding| finding.rule_id.as_str()).collect::<Vec<_>>()
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_rust_async_and_performance_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "src/lib.rs", rust_fixture!("async/positive.rs"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "rust_blocking_io_in_async",
        "rust_unbuffered_file_writes",
        "rust_lines_allocate_per_line",
        "rust_hashmap_default_hasher",
        "rust_lock_across_await",
        "rust_async_std_mutex_await",
        "rust_async_hold_permit_across_await",
        "rust_async_spawn_cancel_at_await",
        "rust_async_missing_fuse_pin",
        "rust_async_recreate_future_in_select",
        "rust_async_lock_order_cycle",
    ] {
        assert!(
            report.findings.iter().any(|finding| finding.rule_id == rule_id),
            "expected finding {rule_id:?}, got {:?}",
            report.findings.iter().map(|finding| finding.rule_id.as_str()).collect::<Vec<_>>()
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_rust_unsafe_soundness_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "src/lib.rs", rust_fixture!("unsafe/positive.rs"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "rust_unsafe_get_unchecked",
        "rust_unsafe_from_raw_parts",
        "rust_unsafe_set_len",
        "rust_unsafe_assume_init",
        "rust_unsafe_transmute",
        "rust_unsafe_raw_pointer_cast",
    ] {
        assert!(
            report.findings.iter().any(|finding| finding.rule_id == rule_id),
            "expected finding {rule_id:?}, got {:?}",
            report.findings.iter().map(|finding| finding.rule_id.as_str()).collect::<Vec<_>>()
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_rust_advanced_negative_fixtures() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "src/domain.rs",
        rust_fixture!("domain_modeling/negative.rs"),
    );
    write_fixture(&temp_dir, "src/async.rs", rust_fixture!("async/negative.rs"));
    write_fixture(&temp_dir, "src/unsafe.rs", rust_fixture!("unsafe/negative.rs"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    let blocked = [
        "rust_domain_raw_primitive",
        "rust_domain_float_for_money",
        "rust_domain_impossible_combination",
        "rust_domain_default_produces_invalid",
        "rust_debug_secret",
        "rust_serde_sensitive_deserialize",
        "rust_serde_sensitive_serialize",
        "rust_blocking_io_in_async",
        "rust_unbuffered_file_writes",
        "rust_lines_allocate_per_line",
        "rust_hashmap_default_hasher",
        "rust_lock_across_await",
        "rust_async_std_mutex_await",
        "rust_async_hold_permit_across_await",
        "rust_async_spawn_cancel_at_await",
        "rust_async_missing_fuse_pin",
        "rust_async_recreate_future_in_select",
        "rust_unsafe_get_unchecked",
        "rust_unsafe_from_raw_parts",
        "rust_unsafe_set_len",
        "rust_unsafe_assume_init",
        "rust_unsafe_transmute",
        "rust_unsafe_raw_pointer_cast",
    ];

    for rule_id in blocked {
        assert!(
            !report.findings.iter().any(|finding| finding.rule_id == rule_id),
            "unexpected finding {rule_id:?}, got {:?}",
            report.findings.iter().map(|finding| finding.rule_id.as_str()).collect::<Vec<_>>()
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_rust_hygiene_script() {
    let status = Command::new("bash")
        .arg("scripts/check_rust_hygiene.sh")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("hygiene script should run");

    assert!(status.success(), "hygiene script should pass");
}