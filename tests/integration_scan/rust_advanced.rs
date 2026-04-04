use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::FixtureWorkspace;

#[test]
fn test_rust_domain_modeling_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("domain_modeling/positive.txt"));

    let report = workspace.scan();

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
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected finding {rule_id:?}, got {:?}",
            report
                .findings
                .iter()
                .map(|finding| finding.rule_id.as_str())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_rust_async_and_performance_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("async/positive.txt"));

    let report = workspace.scan();

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
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected finding {rule_id:?}, got {:?}",
            report
                .findings
                .iter()
                .map(|finding| finding.rule_id.as_str())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_rust_unsafe_soundness_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("unsafe/positive.txt"));

    let report = workspace.scan();

    for rule_id in [
        "rust_unsafe_get_unchecked",
        "rust_unsafe_from_raw_parts",
        "rust_unsafe_set_len",
        "rust_unsafe_assume_init",
        "rust_unsafe_transmute",
        "rust_unsafe_raw_pointer_cast",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected finding {rule_id:?}, got {:?}",
            report
                .findings
                .iter()
                .map(|finding| finding.rule_id.as_str())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_rust_advanced_negative_fixtures() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "src/domain.rs",
        rust_fixture!("domain_modeling/negative.txt"),
    );
    workspace.write_file("src/async.rs", rust_fixture!("async/negative.txt"));
    workspace.write_file("src/unsafe.rs", rust_fixture!("unsafe/negative.txt"));

    let report = workspace.scan();

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
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "unexpected finding {rule_id:?}, got {:?}",
            report
                .findings
                .iter()
                .map(|finding| finding.rule_id.as_str())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_rust_phase4_runtime_boundary_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("phase4/positive.txt"));

    let report = workspace.scan();

    for rule_id in [
        "rust_tokio_runtime_built_per_call",
        "rust_env_var_read_in_request_path",
        "rust_axum_router_built_in_handler",
        "rust_tonic_channel_connect_per_request",
        "rust_clone_heavy_state_in_loop",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected finding {rule_id:?}, got {:?}",
            report
                .findings
                .iter()
                .map(|finding| finding.rule_id.as_str())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_rust_phase4_runtime_boundary_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("phase4/negative.txt"));

    let report = workspace.scan();

    for rule_id in [
        "rust_tokio_runtime_built_per_call",
        "rust_env_var_read_in_request_path",
        "rust_axum_router_built_in_handler",
        "rust_tonic_channel_connect_per_request",
        "rust_clone_heavy_state_in_loop",
    ] {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "unexpected finding {rule_id:?}, got {:?}",
            report
                .findings
                .iter()
                .map(|finding| finding.rule_id.as_str())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_rust_phase4_workspace_manifest_rule() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("Cargo.toml", "[workspace]\nmembers = [\"app\", \"lib\"]\n");
    workspace.write_file("src/lib.rs", rust_fixture!("phase4/negative.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "rust_workspace_missing_resolver"),
        "expected rust_workspace_missing_resolver, got {:?}",
        report
            .findings
            .iter()
            .map(|finding| finding.rule_id.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_rust_phase4_workspace_manifest_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\"app\", \"lib\"]\n",
    );
    workspace.write_file("src/lib.rs", rust_fixture!("phase4/negative.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "rust_workspace_missing_resolver"),
        "unexpected rust_workspace_missing_resolver, got {:?}",
        report
            .findings
            .iter()
            .map(|finding| finding.rule_id.as_str())
            .collect::<Vec<_>>()
    );
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

#[test]
fn test_rust_security_script() {
    let output = Command::new("bash")
        .arg("scripts/check-rust-security.sh")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("security script should run");

    assert!(output.status.success(), "security script should pass");

    let report_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("reports/rust-security-baseline/latest.txt");
    let report = fs::read_to_string(report_path).expect("security report should be readable");

    for heading in [
        "## narrowing_as_casts",
        "## split_at_and_indexing",
        "## toctou_fs_checks",
        "## toctou_check_then_open",
        "## secret_comparisons",
        "## shared_mutability",
        "## unsafe_globals",
        "## derive_default",
        "## thread_spawn_async",
        "## path_join_absolute",
    ] {
        assert!(report.contains(heading), "missing report heading {heading}");
    }
}

#[test]
fn test_production_source_avoids_unbounded_read_to_string() {
    let mut rust_files = Vec::new();
    collect_rust_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_files,
    );

    for file in rust_files {
        let contents = fs::read_to_string(&file).expect("source file should be readable");
        assert!(
            !contents.contains("fs::read_to_string("),
            "production source should avoid fs::read_to_string: {}",
            file.display()
        );
    }
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("directory should be readable") {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
