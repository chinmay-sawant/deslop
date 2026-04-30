use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{FixtureWorkspace, assert_rules_absent, assert_rules_present};

#[test]
fn test_rust_domain_modeling_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("domain_modeling/positive.txt"));

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "rust_domain_raw_primitive",
            "rust_domain_float_for_money",
            "rust_domain_impossible_combination",
            "rust_domain_default_produces_invalid",
            "rust_debug_secret",
            "rust_serde_sensitive_deserialize",
            "rust_serde_sensitive_serialize",
        ],
    );
}

#[test]
fn test_rust_async_and_performance_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "rust_async_experimental = true\n");
    workspace.write_file("src/lib.rs", rust_fixture!("async/positive.txt"));

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
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
        ],
    );
}

#[test]
fn test_rust_unsafe_soundness_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("unsafe/positive.txt"));

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "rust_unsafe_get_unchecked",
            "rust_unsafe_from_raw_parts",
            "rust_unsafe_set_len",
            "rust_unsafe_assume_init",
            "rust_unsafe_transmute",
            "rust_unsafe_raw_pointer_cast",
        ],
    );
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

    assert_rules_absent(
        &report,
        &[
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
        ],
    );
}

#[test]
fn test_rust_phase4_runtime_boundary_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("phase4/positive.txt"));

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "rust_tokio_runtime_built_per_call",
            "rust_env_var_read_in_request_path",
            "rust_axum_router_built_in_handler",
            "rust_tonic_channel_connect_per_request",
            "rust_clone_heavy_state_in_loop",
        ],
    );
}

#[test]
fn test_rust_phase4_runtime_boundary_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("phase4/negative.txt"));

    let report = workspace.scan();

    assert_rules_absent(
        &report,
        &[
            "rust_tokio_runtime_built_per_call",
            "rust_env_var_read_in_request_path",
            "rust_axum_router_built_in_handler",
            "rust_tonic_channel_connect_per_request",
            "rust_clone_heavy_state_in_loop",
        ],
    );
}

#[test]
fn test_rust_phase4_workspace_manifest_rule() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("Cargo.toml", "[workspace]\nmembers = [\"app\", \"lib\"]\n");
    workspace.write_file("src/lib.rs", rust_fixture!("phase4/negative.txt"));

    let report = workspace.scan();

    assert_rules_present(&report, &["rust_workspace_missing_resolver"]);
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

    assert_rules_absent(&report, &["rust_workspace_missing_resolver"]);
}

#[test]
fn test_rust_advanceplan3_plan1_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("advanceplan3/plan1_positive.txt"),
    );

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "rust_internal_anyhow_result",
            "rust_unbounded_read_to_string",
            "rust_check_then_open_path",
            "rust_secret_equality_compare",
            "rust_narrowing_numeric_cast",
            "rust_manual_tempdir_lifecycle",
        ],
    );
}

#[test]
fn test_rust_advanceplan3_plan1_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("advanceplan3/plan1_negative.txt"),
    );

    let report = workspace.scan();

    assert_rules_absent(
        &report,
        &[
            "rust_internal_anyhow_result",
            "rust_unbounded_read_to_string",
            "rust_check_then_open_path",
            "rust_secret_equality_compare",
            "rust_narrowing_numeric_cast",
            "rust_manual_tempdir_lifecycle",
        ],
    );
}

#[test]
fn test_rust_advanceplan3_plan2_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "src/lib.rs",
            rust_fixture!("advanceplan3/plan2_lib_positive.txt"),
        ),
        (
            "src/feature/mod.rs",
            rust_fixture!("advanceplan3/plan2_mod_positive.txt"),
        ),
    ]);

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "rust_oversized_module_file",
            "rust_pub_use_glob_surface",
            "rust_root_reexport_wall",
            "rust_mod_rs_catchall",
            "rust_duplicate_bootstrap_sequence",
            "rust_redundant_path_attribute",
            "rust_broad_allow_dead_code",
        ],
    );
}

#[test]
fn test_rust_advanceplan3_plan2_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "src/lib.rs",
            rust_fixture!("advanceplan3/plan2_lib_negative.txt"),
        ),
        (
            "src/feature/mod.rs",
            rust_fixture!("advanceplan3/plan2_mod_negative.txt"),
        ),
    ]);

    let report = workspace.scan();

    assert_rules_absent(
        &report,
        &[
            "rust_oversized_module_file",
            "rust_pub_use_glob_surface",
            "rust_root_reexport_wall",
            "rust_mod_rs_catchall",
            "rust_duplicate_bootstrap_sequence",
            "rust_redundant_path_attribute",
            "rust_broad_allow_dead_code",
        ],
    );
}

#[test]
fn test_rust_advanceplan3_plan3_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("advanceplan3/plan3_positive.txt"),
    );

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "rust_detached_spawn_without_handle",
            "rust_channel_created_per_request",
            "rust_block_in_place_request_path",
            "rust_runtime_builder_in_loop",
            "rust_notify_without_shutdown_contract",
            "rust_process_global_env_toggle",
        ],
    );
}

#[test]
fn test_rust_advanceplan3_plan3_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("advanceplan3/plan3_negative.txt"),
    );

    let report = workspace.scan();

    assert_rules_absent(
        &report,
        &[
            "rust_detached_spawn_without_handle",
            "rust_channel_created_per_request",
            "rust_block_in_place_request_path",
            "rust_runtime_builder_in_loop",
            "rust_notify_without_shutdown_contract",
            "rust_process_global_env_toggle",
        ],
    );
}

#[test]
fn test_rust_advanceplan3_plan4_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "Cargo.toml",
        "[package]\nname = \"advanceplan3\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[profile.release]\npanic = \"unwind\"\n",
    );
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("advanceplan3/plan4_positive.txt"),
    );

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "rust_split_at_unchecked_external_input",
            "rust_from_utf8_unchecked_boundary",
            "rust_thread_spawn_async_without_runtime",
            "rust_rc_cycle_parent_link",
            "rust_static_mut_global",
            "rust_release_profile_missing_overflow_checks",
            "rust_release_profile_panic_unwind",
        ],
    );
}

#[test]
fn test_rust_advanceplan3_plan4_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "Cargo.toml",
        "[package]\nname = \"advanceplan3\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[profile.release]\npanic = \"abort\"\noverflow-checks = true\n",
    );
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("advanceplan3/plan4_negative.txt"),
    );

    let report = workspace.scan();

    assert_rules_absent(
        &report,
        &[
            "rust_split_at_unchecked_external_input",
            "rust_from_utf8_unchecked_boundary",
            "rust_thread_spawn_async_without_runtime",
            "rust_rc_cycle_parent_link",
            "rust_static_mut_global",
            "rust_release_profile_missing_overflow_checks",
            "rust_release_profile_panic_unwind",
        ],
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
