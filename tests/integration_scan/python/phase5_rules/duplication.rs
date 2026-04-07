use super::{assert_rules_absent, assert_rules_present, report_has_rule, scan_python_files};

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
