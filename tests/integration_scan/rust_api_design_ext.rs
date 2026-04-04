
use deslop::{ScanOptions, scan_repository};

use super::FixtureWorkspace;

fn scan_fixture(fixture_path: &str) -> deslop::ScanReport {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", fixture_path);

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

        report
}

fn assert_rules_present(report: &deslop::ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == *rule_id),
            "expected rule {rule_id} to fire"
        );
    }
}

fn assert_rules_absent(report: &deslop::ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == *rule_id),
            "did not expect rule {rule_id} to fire"
        );
    }
}

#[test]
fn test_rust_advanceplan2_api_rules() {
    let report = scan_fixture(rust_fixture!("api_design_ext/api_positive.txt"));

    assert_rules_present(
        &report,
        &[
            "rust_public_anyhow_result",
            "rust_public_box_dyn_error",
            "rust_borrowed_string_api",
            "rust_borrowed_vec_api",
            "rust_borrowed_pathbuf_api",
            "rust_public_bool_parameter_api",
        ],
    );
}

#[test]
fn test_rust_advanceplan2_api_clean() {
    let report = scan_fixture(rust_fixture!("api_design_ext/api_clean.txt"));

    assert_rules_absent(
        &report,
        &[
            "rust_public_anyhow_result",
            "rust_public_box_dyn_error",
            "rust_borrowed_string_api",
            "rust_borrowed_vec_api",
            "rust_borrowed_pathbuf_api",
            "rust_public_bool_parameter_api",
        ],
    );
}

#[test]
fn test_rust_advanceplan2_shared_state_rules() {
    let report = scan_fixture(rust_fixture!("api_design_ext/shared_state_positive.txt"));

    assert_rules_present(
        &report,
        &[
            "rust_pub_interior_mutability_field",
            "rust_global_lock_state",
            "rust_arc_mutex_option_state",
            "rust_mutex_wrapped_collection",
            "rust_rc_refcell_domain_model",
        ],
    );
}

#[test]
fn test_rust_advanceplan2_shared_state_clean() {
    let report = scan_fixture(rust_fixture!("api_design_ext/shared_state_clean.txt"));

    assert_rules_absent(
        &report,
        &[
            "rust_pub_interior_mutability_field",
            "rust_global_lock_state",
            "rust_arc_mutex_option_state",
            "rust_mutex_wrapped_collection",
            "rust_rc_refcell_domain_model",
        ],
    );
}

#[test]
fn test_rust_advanceplan2_serde_rules() {
    let report = scan_fixture(rust_fixture!("api_design_ext/serde_positive.txt"));

    assert_rules_present(
        &report,
        &[
            "rust_serde_untagged_enum_boundary",
            "rust_serde_default_on_required_field",
            "rust_serde_flatten_catchall",
            "rust_serde_unknown_fields_allowed",
            "rust_stringly_typed_enum_boundary",
        ],
    );
}

#[test]
fn test_rust_advanceplan2_serde_clean() {
    let report = scan_fixture(rust_fixture!("api_design_ext/serde_clean.txt"));

    assert_rules_absent(
        &report,
        &[
            "rust_serde_untagged_enum_boundary",
            "rust_serde_default_on_required_field",
            "rust_serde_flatten_catchall",
            "rust_serde_unknown_fields_allowed",
            "rust_stringly_typed_enum_boundary",
        ],
    );
}

#[test]
fn test_rust_advanceplan2_builder_rules() {
    let report = scan_fixture(rust_fixture!("api_design_ext/builder_positive.txt"));

    assert_rules_present(
        &report,
        &[
            "rust_option_bag_config",
            "rust_builder_without_validate",
            "rust_constructor_many_flags",
            "rust_partial_init_escape",
            "rust_boolean_state_machine",
        ],
    );
}

#[test]
fn test_rust_advanceplan2_builder_clean() {
    let report = scan_fixture(rust_fixture!("api_design_ext/builder_clean.txt"));

    assert_rules_absent(
        &report,
        &[
            "rust_option_bag_config",
            "rust_builder_without_validate",
            "rust_constructor_many_flags",
            "rust_partial_init_escape",
            "rust_boolean_state_machine",
        ],
    );
}
