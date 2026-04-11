use super::{FixtureWorkspace, assert_rules_absent, assert_rules_present};

const MANIFEST_BAD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/rust/bad_practices/manifest_bad.toml"
));
const MANIFEST_CLEAN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/rust/bad_practices/manifest_clean.toml"
));

#[test]
fn bad_practices_positive_fixture_reports_representative_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("bad_practices/positive.txt"));

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "rust_tree_sitter_language_conversion_inside_loop",
            "rust_rayon_collect_all_then_filter_sequentially",
            "rust_ignore_follow_links_without_cycle_or_root_policy",
            "rust_serde_json_to_string_pretty_in_machine_path",
            "rust_anyhow_eager_format_context",
            "rust_clap_secret_arg_derive_debug",
            "rust_libc_cstring_unwrap_on_external_input",
            "rust_mutex_lock_unwrap_panics_on_poison",
        ],
    );
}

#[test]
fn bad_practices_clean_fixture_stays_clean_for_representative_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("bad_practices/clean.txt"));

    let report = workspace.scan();

    assert_rules_absent(
        &report,
        &[
            "rust_tree_sitter_language_conversion_inside_loop",
            "rust_rayon_collect_all_then_filter_sequentially",
            "rust_ignore_follow_links_without_cycle_or_root_policy",
            "rust_serde_json_to_string_pretty_in_machine_path",
            "rust_anyhow_eager_format_context",
            "rust_clap_secret_arg_derive_debug",
            "rust_libc_cstring_unwrap_on_external_input",
            "rust_mutex_lock_unwrap_panics_on_poison",
        ],
    );
}

#[test]
fn bad_practices_honors_rule_ignore_directives() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("bad_practices/suppression.txt"));

    let report = workspace.scan();

    assert_rules_absent(&report, &["rust_collect_then_single_iteration"]);
    assert_rules_present(&report, &["rust_vec_remove_zero_in_loop"]);
}

#[test]
fn bad_practices_manifest_rules_fire_on_bad_manifest_and_build_script() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("Cargo.toml", MANIFEST_BAD);
    workspace.write_file("src/main.rs", "fn main() {}");
    workspace.write_file(
        "build.rs",
        "fn main() { let _ = std::process::Command::new(\"git\"); }",
    );
    workspace.write_file(
        "app/Cargo.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\nclap=\"3.2\"\n",
    );
    workspace.write_file(
        "fuzz/Cargo.toml",
        "[package]\nname=\"fuzz\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    );

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "rust_manifest_wildcard_dependency_version",
            "rust_manifest_dependency_default_features_unreviewed",
            "rust_manifest_duplicate_direct_dependency_versions",
            "rust_manifest_workspace_dependency_not_centralized",
            "rust_manifest_release_lto_missing_for_cli_binary",
            "rust_manifest_bench_or_fuzz_target_in_default_members",
            "rust_build_script_missing_rerun_if_changed",
            "rust_build_script_network_or_git_call",
        ],
    );
}

#[test]
fn bad_practices_manifest_rules_stay_clean_with_reviewed_config() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("Cargo.toml", MANIFEST_CLEAN);
    workspace.write_file("src/main.rs", "fn main() {}");
    workspace.write_file(
        "build.rs",
        "fn main() { println!(\"cargo:rerun-if-changed=build.rs\"); }",
    );
    workspace.write_file(
        "app/Cargo.toml",
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\nclap={ workspace=true, default-features=false }\n",
    );
    workspace.write_file(
        "tools/Cargo.toml",
        "[package]\nname=\"tools\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    );

    let report = workspace.scan();

    assert_rules_absent(
        &report,
        &[
            "rust_manifest_wildcard_dependency_version",
            "rust_manifest_dependency_default_features_unreviewed",
            "rust_manifest_duplicate_direct_dependency_versions",
            "rust_manifest_workspace_dependency_not_centralized",
            "rust_manifest_release_lto_missing_for_cli_binary",
            "rust_manifest_bench_or_fuzz_target_in_default_members",
            "rust_build_script_missing_rerun_if_changed",
            "rust_build_script_network_or_git_call",
        ],
    );
}
