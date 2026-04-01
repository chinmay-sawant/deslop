use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::super::create_temp_workspace;
use super::write_files;

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

const FRAMEWORK_RULES: &[&str] = &[
    "django_queryset_len_instead_of_count",
    "django_queryset_count_then_exists",
    "django_queryset_order_by_random",
    "django_all_without_limit_in_view",
    "django_create_single_in_loop",
    "django_save_full_model_in_loop",
    "django_delete_single_in_loop",
    "django_raw_sql_in_loop",
    "flask_request_body_parsed_multiple_times",
    "flask_global_db_connection_per_request",
    "flask_app_config_read_per_request",
    "flask_template_rendered_from_string_in_view",
    "flask_debug_mode_in_production_code",
    "fastapi_sync_def_with_blocking_io",
    "sqlalchemy_query_in_loop",
    "sqlalchemy_commit_per_row_in_loop",
    "middleware_creates_http_client_per_request",
    "middleware_compiles_regex_per_request",
    "middleware_loads_config_file_per_request",
    "upstream_call_without_timeout_in_handler",
    "response_json_dumps_then_response_object",
];

#[test]
fn test_python_framework_positive() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/framework_code.py",
            python_fixture!("integration/framework/framework_positive.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_present(&report, FRAMEWORK_RULES);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_framework_clean() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/framework_code.py",
            python_fixture!("integration/framework/framework_clean.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_absent(&report, FRAMEWORK_RULES);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
