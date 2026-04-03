use deslop::{ScanReport, Severity};

use crate::support::{
    FixtureWorkspace, assert_rules_absent, assert_rules_present, find_rule, report_has_rule,
    scan_files,
};

fn scan_python_files(files: &[(&str, &str)]) -> ScanReport {
    scan_files(files)
}

fn scan_generated_files(setup: impl FnOnce(&FixtureWorkspace)) -> ScanReport {
    let workspace = FixtureWorkspace::new();
    setup(&workspace);
    workspace.scan()
}

#[test]
fn test_python_phase5_instance_attribute_escalation() {
    let report = scan_python_files(&[(
        "pkg/heavy_state.py",
        python_fixture!("structure/heavy_state_positive.txt"),
    )]);

    let finding = find_rule(&report, "too_many_instance_attributes")
        .expect("expected too_many_instance_attributes finding");
    assert!(matches!(finding.severity, Severity::Warning));
    assert!(
        finding
            .evidence
            .iter()
            .any(|evidence| evidence == "tier=20_plus"),
        "expected the escalated 20-plus evidence tier"
    );
}

#[test]
fn test_python_phase5_duplicate_query_fragment_rule() {
    let report = scan_python_files(&[
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
    ]);

    assert!(
        report_has_rule(&report, "duplicate_query_fragment"),
        "expected duplicate_query_fragment to fire"
    );
    assert!(
        !report_has_rule(&report, "cross_file_repeated_literal"),
        "did not expect generic cross_file_repeated_literal for query-like strings"
    );
}

#[test]
fn test_python_phase5_duplicate_query_fragment_skips_shared_constants_and_migrations() {
    let report = scan_python_files(&[
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
    ]);

    assert!(
        !report_has_rule(&report, "duplicate_query_fragment"),
        "did not expect duplicate_query_fragment for centralized constants, shared templates, or migrations"
    );
}

#[test]
fn test_python_phase5_cross_file_copy_paste_rule() {
    let report = scan_python_files(&[
        (
            "pkg/service_a.py",
            python_fixture!("duplication/cross_file_copy_a.txt"),
        ),
        (
            "pkg/service_b.py",
            python_fixture!("duplication/cross_file_copy_b.txt"),
        ),
    ]);

    assert!(
        report_has_rule(&report, "cross_file_copy_paste_function"),
        "expected cross_file_copy_paste_function to fire"
    );
}

#[test]
fn test_python_phase5_duplicate_transformation_pipeline_rule() {
    let report = scan_python_files(&[
        (
            "pkg/ingest_a.py",
            python_fixture!("duplication/transformation_pipeline_a.txt"),
        ),
        (
            "pkg/ingest_b.py",
            python_fixture!("duplication/transformation_pipeline_b.txt"),
        ),
    ]);

    assert!(
        report_has_rule(&report, "duplicate_transformation_pipeline"),
        "expected duplicate_transformation_pipeline to fire"
    );
}

#[test]
fn test_python_phase5_duplicate_transformation_pipeline_skips_short_helpers() {
    let report = scan_python_files(&[
        (
            "pkg/helpers_a.py",
            python_fixture!("duplication/transformation_helpers_a.txt"),
        ),
        (
            "pkg/helpers_b.py",
            python_fixture!("duplication/transformation_helpers_b.txt"),
        ),
    ]);

    assert!(
        !report_has_rule(&report, "duplicate_transformation_pipeline"),
        "did not expect duplicate_transformation_pipeline for short helper chains"
    );
}

#[test]
fn test_python_duplication_rule_family_positive() {
    let report = scan_python_files(&[
        (
            "pkg/file_literals.py",
            python_fixture!("duplication/repeated_literals_positive.txt"),
        ),
        (
            "pkg/file_error_handlers.py",
            python_fixture!("duplication/error_handlers_positive.txt"),
        ),
        (
            "pkg/file_validation.py",
            python_fixture!("duplication/validation_pipeline_positive.txt"),
        ),
        (
            "pkg/query_base.py",
            python_fixture!("duplication/query_fragment_repo_a.txt"),
        ),
        (
            "pkg/query_helpers.py",
            python_fixture!("duplication/query_fragment_repo_b.txt"),
        ),
        (
            "pkg/query_services.py",
            python_fixture!("duplication/query_fragment_repo_c.txt"),
        ),
        (
            "pkg/profile_builder.py",
            python_fixture!("duplication/cross_file_copy_a.txt"),
        ),
        (
            "pkg/account_builder.py",
            python_fixture!("duplication/cross_file_copy_b.txt"),
        ),
        (
            "pkg/pipeline_a.py",
            python_fixture!("duplication/transformation_pipeline_a.txt"),
        ),
        (
            "pkg/pipeline_b.py",
            python_fixture!("duplication/transformation_pipeline_b.txt"),
        ),
    ]);

    assert_rules_present(
        &report,
        &[
            "repeated_string_literal",
            "duplicate_error_handler_block",
            "duplicate_validation_pipeline",
            "duplicate_query_fragment",
            "cross_file_copy_paste_function",
            "duplicate_transformation_pipeline",
        ],
    );
}

#[test]
fn test_python_duplication_rule_family_negative() {
    let report = scan_python_files(&[
        (
            "pkg/file_literals.py",
            python_fixture!("duplication/repeated_literals_negative.txt"),
        ),
        (
            "pkg/file_error_handlers.py",
            python_fixture!("duplication/error_handlers_negative.txt"),
        ),
        (
            "pkg/file_validation.py",
            python_fixture!("duplication/validation_pipeline_negative.txt"),
        ),
        (
            "pkg/query_constants.py",
            python_fixture!("duplication/query_fragment_shared_constants.txt"),
        ),
        (
            "pkg/query_templates.py",
            python_fixture!("duplication/query_fragment_shared_templates.txt"),
        ),
        (
            "pkg/query_consumer_a.py",
            python_fixture!("duplication/query_fragment_consumer_a.txt"),
        ),
        (
            "pkg/query_consumer_b.py",
            python_fixture!("duplication/query_fragment_consumer_b.txt"),
        ),
        (
            "pkg/pipeline_helper_a.py",
            python_fixture!("duplication/transformation_helpers_a.txt"),
        ),
        (
            "pkg/pipeline_helper_b.py",
            python_fixture!("duplication/transformation_helpers_b.txt"),
        ),
        (
            "pkg/profile_builder.py",
            python_fixture!("duplication/cross_file_copy_a.txt"),
        ),
    ]);

    assert_rules_absent(
        &report,
        &[
            "repeated_string_literal",
            "duplicate_error_handler_block",
            "duplicate_validation_pipeline",
            "duplicate_query_fragment",
            "cross_file_copy_paste_function",
            "duplicate_transformation_pipeline",
        ],
    );
}

#[test]
fn test_python_performance_rule_family_positive() {
    let report = scan_python_files(&[(
        "pkg/loop_shapes.py",
        python_fixture!("performance/loop_shapes_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "string_concat_in_loop",
            "list_materialization_first_element",
            "deque_candidate_queue",
            "temporary_collection_in_loop",
            "recursive_traversal_risk",
            "list_membership_in_loop",
            "repeated_len_in_loop",
        ],
    );
}

#[test]
fn test_python_performance_rule_family_negative() {
    let report = scan_python_files(&[(
        "pkg/loop_shapes.py",
        python_fixture!("performance/loop_shapes_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "string_concat_in_loop",
            "list_materialization_first_element",
            "deque_candidate_queue",
            "temporary_collection_in_loop",
            "recursive_traversal_risk",
            "list_membership_in_loop",
            "repeated_len_in_loop",
        ],
    );
}

#[test]
fn test_python_async_boundary_rules_positive() {
    let report = scan_python_files(&[(
        "pkg/async_boundaries.py",
        python_fixture!("performance/async_boundaries_positive.txt"),
    )]);

    assert_rules_present(&report, &["blocking_sync_io_in_async", "full_dataset_load"]);
}

#[test]
fn test_python_async_boundary_rules_negative() {
    let report = scan_python_files(&[(
        "pkg/async_boundaries.py",
        python_fixture!("performance/async_boundaries_negative.txt"),
    )]);

    assert_rules_absent(&report, &["blocking_sync_io_in_async", "full_dataset_load"]);
}

#[test]
fn test_python_structure_rule_family_positive() {
    let report = scan_python_files(&[
        (
            "pkg/god_function.py",
            python_fixture!("structure/god_function_positive.txt"),
        ),
        (
            "pkg/mixed_concerns.py",
            python_fixture!("structure/mixed_concerns_positive.txt"),
        ),
        (
            "pkg/presenter.py",
            python_fixture!("structure/over_abstracted_wrapper_positive.txt"),
        ),
        (
            "pkg/session_state.py",
            python_fixture!("structure/too_many_instance_attributes_positive.txt"),
        ),
        (
            "pkg/parser.py",
            python_fixture!("structure/name_responsibility_positive.txt"),
        ),
        (
            "pkg/billing.py",
            python_fixture!("structure/god_class_positive.txt"),
        ),
    ]);

    assert_rules_present(
        &report,
        &[
            "god_function",
            "mixed_concerns_function",
            "over_abstracted_wrapper",
            "too_many_instance_attributes",
            "name_responsibility_mismatch",
            "god_class",
        ],
    );
}

#[test]
fn test_python_structure_rule_family_negative() {
    let report = scan_python_files(&[
        (
            "pkg/god_function.py",
            python_fixture!("structure/god_function_negative.txt"),
        ),
        (
            "pkg/mixed_concerns.py",
            python_fixture!("structure/mixed_concerns_negative.txt"),
        ),
        (
            "pkg/presenter.py",
            python_fixture!("structure/over_abstracted_wrapper_negative.txt"),
        ),
        (
            "pkg/session_state.py",
            python_fixture!("structure/too_many_instance_attributes_negative.txt"),
        ),
        (
            "pkg/parser.py",
            python_fixture!("structure/name_responsibility_negative.txt"),
        ),
        (
            "pkg/billing.py",
            python_fixture!("structure/god_class_negative.txt"),
        ),
    ]);

    assert_rules_absent(
        &report,
        &[
            "god_function",
            "mixed_concerns_function",
            "over_abstracted_wrapper",
            "too_many_instance_attributes",
            "name_responsibility_mismatch",
            "god_class",
        ],
    );
}

#[test]
fn test_python_ai_smells_rule_family_positive() {
    let report = scan_python_files(&[
        (
            "pkg/docstrings.py",
            python_fixture!("ai_smells/docstring_small_helper_positive.txt"),
        ),
        (
            "pkg/naming.py",
            python_fixture!("ai_smells/mixed_naming_positive.txt"),
        ),
        (
            "pkg/heavy.py",
            python_fixture!("ai_smells/heavy_imports_positive.txt"),
        ),
        (
            "pkg/comments.py",
            python_fixture!("ai_smells/commentary_positive.txt"),
        ),
    ]);

    assert_rules_present(
        &report,
        &[
            "textbook_docstring_small_helper",
            "mixed_naming_conventions",
            "unrelated_heavy_import",
            "obvious_commentary",
        ],
    );
}

#[test]
fn test_python_ai_smells_rule_family_negative() {
    let report = scan_python_files(&[
        (
            "pkg/docstrings.py",
            python_fixture!("ai_smells/docstring_small_helper_negative.txt"),
        ),
        (
            "pkg/naming.py",
            python_fixture!("ai_smells/mixed_naming_negative.txt"),
        ),
        (
            "pkg/heavy.py",
            python_fixture!("ai_smells/heavy_imports_negative.txt"),
        ),
        (
            "pkg/comments.py",
            python_fixture!("ai_smells/commentary_negative.txt"),
        ),
    ]);

    assert_rules_absent(
        &report,
        &[
            "textbook_docstring_small_helper",
            "mixed_naming_conventions",
            "unrelated_heavy_import",
            "obvious_commentary",
        ],
    );
}

#[test]
fn test_python_phase5_monolithic_module_rule() {
    let report = scan_generated_files(|workspace| {
        let mut module = String::from(python_fixture!(
            "integration/phase5/monolithic_module_prefix.txt"
        ));
        for index in 0..320 {
            module.push_str(&format!(
                "\ndef helper_{index}(payload):\n    record = str(payload).strip()\n    if not record:\n        return ''\n    return record.lower()\n"
            ));
        }
        workspace.write_file("pkg/module.py", &module);
    });

    assert!(
        report_has_rule(&report, "monolithic_module"),
        "expected monolithic_module to fire"
    );
}

#[test]
fn test_python_phase5_over_abstracted_wrapper_expansion() {
    let report = scan_python_files(&[(
        "pkg/presenter.py",
        python_fixture!("structure/over_abstracted_wrapper_positive.txt"),
    )]);

    assert!(
        report_has_rule(&report, "over_abstracted_wrapper"),
        "expected over_abstracted_wrapper to fire for a ceremonial wrapper class"
    );
}

#[test]
fn test_python_phase5_over_abstracted_wrapper_skips_lifecycle_classes() {
    let report = scan_python_files(&[(
        "pkg/runtime.py",
        python_fixture!("structure/over_abstracted_wrapper_negative.txt"),
    )]);

    assert!(
        !report_has_rule(&report, "over_abstracted_wrapper"),
        "did not expect over_abstracted_wrapper for lifecycle-heavy classes"
    );
}

#[test]
fn test_python_phase5_name_responsibility_mismatch_expansion() {
    let report = scan_python_files(&[
        (
            "pkg/parser.py",
            python_fixture!("structure/name_responsibility_parser_positive.txt"),
        ),
        (
            "pkg/report_helper.py",
            python_fixture!("structure/name_responsibility_helper_positive.txt"),
        ),
    ]);

    assert!(
        report.findings.iter().any(|finding| {
            finding.rule_id == "name_responsibility_mismatch"
                && (finding.function_name.as_deref() == Some("parse_user")
                    || finding.path.ends_with("pkg/report_helper.py"))
        }),
        "expected expanded name_responsibility_mismatch anchors to fire"
    );
}

#[test]
fn test_python_phase5_name_responsibility_mismatch_skips_honest_transformers() {
    let report = scan_python_files(&[(
        "pkg/parser.py",
        python_fixture!("structure/name_responsibility_negative.txt"),
    )]);

    assert!(
        !report_has_rule(&report, "name_responsibility_mismatch"),
        "did not expect name_responsibility_mismatch for honest parse helpers"
    );
}

#[test]
fn test_python_phase5_business_magic_and_utility_rules() {
    let report = scan_python_files(&[(
        "pkg/business_rules.py",
        python_fixture!("maintainability/business_rules_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "hardcoded_business_rule",
            "magic_value_branching",
            "reinvented_utility",
        ],
    );
}

#[test]
fn test_python_phase5_business_magic_and_utility_suppressions() {
    let report = scan_python_files(&[(
        "pkg/business_rules.py",
        python_fixture!("maintainability/business_rules_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "hardcoded_business_rule",
            "magic_value_branching",
            "reinvented_utility",
        ],
    );
}

#[test]
fn test_python_phase5_boundary_robustness_rules() {
    let report = scan_python_files(&[(
        "pkg/boundaries.py",
        python_fixture!("maintainability/boundaries_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "network_boundary_without_timeout",
            "environment_boundary_without_fallback",
            "external_input_without_validation",
        ],
    );
}

#[test]
fn test_python_phase5_boundary_robustness_suppressions() {
    let report = scan_python_files(&[(
        "pkg/boundaries.py",
        python_fixture!("maintainability/boundaries_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "network_boundary_without_timeout",
            "environment_boundary_without_fallback",
            "external_input_without_validation",
        ],
    );
}

#[test]
fn test_python_exception_and_debug_rules_positive() {
    let report = scan_python_files(&[(
        "pkg/debug_rules.py",
        python_fixture!("maintainability/exception_and_debug_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "exception_swallowed",
            "broad_exception_handler",
            "eval_exec_usage",
            "print_debugging_leftover",
            "commented_out_code",
        ],
    );
}

#[test]
fn test_python_exception_and_debug_rules_negative() {
    let report = scan_python_files(&[(
        "pkg/debug_rules.py",
        python_fixture!("maintainability/exception_and_debug_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "exception_swallowed",
            "broad_exception_handler",
            "eval_exec_usage",
            "print_debugging_leftover",
            "commented_out_code",
        ],
    );
}

#[test]
fn test_python_phase5_monolithic_module_skips_broad_legitimate_modules() {
    let report = scan_generated_files(|workspace| {
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

        workspace.write_file("pkg/registry.py", &registry_module);
        workspace.write_file("pkg/schemas.py", &schema_module);
        workspace.write_file("pkg/api_surface.py", &api_surface_module);
    });

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
}
