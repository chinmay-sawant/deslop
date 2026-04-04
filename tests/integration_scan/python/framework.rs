use super::super::FixtureWorkspace;

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

const PHASE3_FRAMEWORK_RULES: &[&str] = &[
    "celery_delay_in_loop_without_canvas",
    "celery_result_get_inside_task",
    "celery_task_reads_env_per_invocation",
    "click_typer_config_file_loaded_per_command",
    "click_typer_env_lookup_per_command",
    "click_typer_http_client_created_per_command",
    "pydantic_model_dump_then_json_dumps",
    "pydantic_model_validate_after_json_loads",
    "sqlmodel_commit_per_row_in_loop",
    "sqlmodel_session_exec_in_loop",
    "sqlmodel_unbounded_select_in_handler",
];

#[test]
fn test_python_framework_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[(
        "pkg/framework_code.py",
        python_fixture!("integration/framework/framework_positive.txt"),
    )]);

    let report = workspace.scan();

    assert_rules_present(&report, FRAMEWORK_RULES);
}

#[test]
fn test_python_framework_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[(
        "pkg/framework_code.py",
        python_fixture!("integration/framework/framework_clean.txt"),
    )]);

    let report = workspace.scan();

    assert_rules_absent(&report, FRAMEWORK_RULES);
}

#[test]
fn test_python_framework_phase3_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[(
        "pkg/framework_phase3.py",
        python_fixture!("integration/framework/framework_phase3_positive.txt"),
    )]);

    let report = workspace.scan();

    assert_rules_present(&report, PHASE3_FRAMEWORK_RULES);
}

#[test]
fn test_python_framework_phase3_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[(
        "pkg/framework_phase3.py",
        python_fixture!("integration/framework/framework_phase3_clean.txt"),
    )]);

    let report = workspace.scan();

    assert_rules_absent(&report, PHASE3_FRAMEWORK_RULES);
}
