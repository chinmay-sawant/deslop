use std::fs;

use deslop::{ScanOptions, Severity, scan_repository};

use super::super::create_temp_workspace;
use super::write_files;

#[test]
fn test_python_phase5_instance_attribute_escalation() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/heavy_state.py",
            python_fixture!("structure/heavy_state_positive.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.rule_id == "too_many_instance_attributes")
        .expect("expected too_many_instance_attributes finding");
    assert!(matches!(finding.severity, Severity::Warning));
    assert!(
        finding
            .evidence
            .iter()
            .any(|evidence| evidence == "tier=20_plus"),
        "expected the escalated 20-plus evidence tier"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_duplicate_query_fragment_rule() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/base.py",
                python_fixture!("duplication/query_fragment_repo_a.txt"),
            ),
            (
                "pkg/helpers.py",
                python_fixture!("duplication/query_fragment_repo_b.txt"),
            ),
            (
                "pkg/services.py",
                python_fixture!("duplication/query_fragment_repo_c.txt"),
            ),
        ],
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
            .any(|finding| finding.rule_id == "duplicate_query_fragment"),
        "expected duplicate_query_fragment to fire"
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "cross_file_repeated_literal"),
        "did not expect generic cross_file_repeated_literal for query-like strings"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_duplicate_query_fragment_skips_shared_constants_and_migrations() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/query_constants.py",
                python_fixture!("duplication/query_fragment_shared_constants.txt"),
            ),
            (
                "pkg/query_templates.py",
                python_fixture!("duplication/query_fragment_shared_templates.txt"),
            ),
            (
                "pkg/service_a.py",
                python_fixture!("duplication/query_fragment_consumer_a.txt"),
            ),
            (
                "pkg/service_b.py",
                python_fixture!("duplication/query_fragment_consumer_b.txt"),
            ),
            (
                "migrations/0001_backfill_reports.py",
                python_fixture!("integration/phase5/migration_0001.txt"),
            ),
            (
                "migrations/0002_backfill_reports.py",
                python_fixture!("integration/phase5/migration_0002.txt"),
            ),
            (
                "migrations/0003_backfill_reports.py",
                python_fixture!("integration/phase5/migration_0003.txt"),
            ),
        ],
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
            .any(|finding| finding.rule_id == "duplicate_query_fragment"),
        "did not expect duplicate_query_fragment for centralized constants, shared templates, or migrations"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_cross_file_copy_paste_rule() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/service_a.py",
                python_fixture!("duplication/cross_file_copy_a.txt"),
            ),
            (
                "pkg/service_b.py",
                python_fixture!("duplication/cross_file_copy_b.txt"),
            ),
        ],
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
            .any(|finding| finding.rule_id == "cross_file_copy_paste_function"),
        "expected cross_file_copy_paste_function to fire"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_duplicate_transformation_pipeline_rule() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/ingest_a.py",
                python_fixture!("duplication/transformation_pipeline_a.txt"),
            ),
            (
                "pkg/ingest_b.py",
                python_fixture!("duplication/transformation_pipeline_b.txt"),
            ),
        ],
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
            .any(|finding| finding.rule_id == "duplicate_transformation_pipeline"),
        "expected duplicate_transformation_pipeline to fire"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_duplicate_transformation_pipeline_skips_short_helpers() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/helpers_a.py",
                python_fixture!("duplication/transformation_helpers_a.txt"),
            ),
            (
                "pkg/helpers_b.py",
                python_fixture!("duplication/transformation_helpers_b.txt"),
            ),
        ],
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
            .any(|finding| finding.rule_id == "duplicate_transformation_pipeline"),
        "did not expect duplicate_transformation_pipeline for short helper chains"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_monolithic_module_rule() {
    let temp_dir = create_temp_workspace();
    let mut module = String::from(python_fixture!(
        "integration/phase5/monolithic_module_prefix.txt"
    ));
    for index in 0..320 {
        module.push_str(&format!(
            "\ndef helper_{index}(payload):\n    record = str(payload).strip()\n    if not record:\n        return ''\n    return record.lower()\n"
        ));
    }
    write_files(&temp_dir, &[("pkg/module.py", &module)]);

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "monolithic_module"),
        "expected monolithic_module to fire"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_over_abstracted_wrapper_expansion() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/presenter.py",
            python_fixture!("structure/over_abstracted_wrapper_positive.txt"),
        )],
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
            .any(|finding| finding.rule_id == "over_abstracted_wrapper"),
        "expected over_abstracted_wrapper to fire for a ceremonial wrapper class"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_over_abstracted_wrapper_skips_lifecycle_classes() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/runtime.py",
            python_fixture!("structure/over_abstracted_wrapper_negative.txt"),
        )],
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
            .any(|finding| finding.rule_id == "over_abstracted_wrapper"),
        "did not expect over_abstracted_wrapper for lifecycle-heavy classes"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_name_responsibility_mismatch_expansion() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/parser.py",
                python_fixture!("structure/name_responsibility_parser_positive.txt"),
            ),
            (
                "pkg/report_helper.py",
                python_fixture!("structure/name_responsibility_helper_positive.txt"),
            ),
        ],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report.findings.iter().any(|finding| {
            finding.rule_id == "name_responsibility_mismatch"
                && (finding.function_name.as_deref() == Some("parse_user")
                    || finding.path.ends_with("pkg/report_helper.py"))
        }),
        "expected expanded name_responsibility_mismatch anchors to fire"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_name_responsibility_mismatch_skips_honest_transformers() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/parser.py",
            python_fixture!("structure/name_responsibility_negative.txt"),
        )],
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
            .any(|finding| finding.rule_id == "name_responsibility_mismatch"),
        "did not expect name_responsibility_mismatch for honest parse helpers"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_business_magic_and_utility_rules() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/policy.py",
                python_fixture!("maintainability/business_policy_positive.txt"),
            ),
            (
                "pkg/archive.py",
                python_fixture!("maintainability/business_archive_positive.txt"),
            ),
            (
                "pkg/flatteners.py",
                python_fixture!("maintainability/business_flatteners_positive.txt"),
            ),
        ],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "hardcoded_business_rule",
        "magic_value_branching",
        "reinvented_utility",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_business_magic_and_utility_suppressions() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/policy.py",
                python_fixture!("maintainability/business_policy_negative.txt"),
            ),
            (
                "pkg/archive.py",
                python_fixture!("maintainability/business_archive_negative.txt"),
            ),
            (
                "pkg/flatteners.py",
                python_fixture!("maintainability/business_flatteners_negative.txt"),
            ),
        ],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "hardcoded_business_rule",
        "magic_value_branching",
        "reinvented_utility",
    ] {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "did not expect rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_boundary_robustness_rules() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/network_sync.py",
                python_fixture!("maintainability/boundary_network_positive.txt"),
            ),
            (
                "pkg/config_loader.py",
                python_fixture!("maintainability/boundary_config_positive.txt"),
            ),
            (
                "pkg/cli.py",
                python_fixture!("maintainability/boundary_cli_positive.txt"),
            ),
        ],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "network_boundary_without_timeout",
        "environment_boundary_without_fallback",
        "external_input_without_validation",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_boundary_robustness_suppressions() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/network_sync.py",
                python_fixture!("maintainability/boundary_network_negative.txt"),
            ),
            (
                "pkg/config_loader.py",
                python_fixture!("maintainability/boundary_config_negative.txt"),
            ),
            (
                "pkg/cli.py",
                python_fixture!("maintainability/boundary_cli_negative.txt"),
            ),
        ],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "network_boundary_without_timeout",
        "environment_boundary_without_fallback",
        "external_input_without_validation",
    ] {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "did not expect rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_monolithic_module_skips_broad_legitimate_modules() {
    let temp_dir = create_temp_workspace();

    let mut registry_module = String::from(python_fixture!(
        "integration/phase5/legit_registry_prefix.txt"
    ));
    for index in 0..500 {
        registry_module.push_str(&format!(
            "\ndef provide_{index}():\n    value = 'entry_{index}'\n    register(value, value)\n    return REGISTRY[value]\n"
        ));
    }

    let mut schema_module = String::from(python_fixture!(
        "integration/phase5/legit_schemas_prefix.txt"
    ));
    for index in 0..320 {
        schema_module.push_str(&format!(
            "\nclass EventSchema{index}:\n    event_id = 'event_{index}'\n    source = 'api'\n    kind = 'schema'\n    version = {index}\n"
        ));
    }

    let mut api_surface_module = String::from(python_fixture!(
        "integration/phase5/legit_api_surface_prefix.txt"
    ));
    for index in 0..520 {
        api_surface_module.push_str(&format!(
            "\ndef route_{index}(request):\n    payload = {{'route': {index}, 'request': request}}\n    return render(payload)\n"
        ));
    }

    write_files(
        &temp_dir,
        &[
            ("pkg/registry.py", &registry_module),
            ("pkg/schemas.py", &schema_module),
            ("pkg/api_surface.py", &api_surface_module),
        ],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    let flagged_paths = report
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "monolithic_module")
        .map(|finding| finding.path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        flagged_paths.is_empty(),
        "did not expect broad-but-legitimate modules to be flagged: {flagged_paths:?}"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
