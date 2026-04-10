use deslop::ScanReport;

use super::super::support::{assert_rules_absent, assert_rules_present, scan_files};

fn scan(files: &[(&str, &str)]) -> ScanReport {
    scan_files(files)
}

// ─── Architecture family ─────────────────────────────────────────────────────

#[test]
fn test_architecture_rules_positive() {
    // File named "service.py" to satisfy SERVICE_ROLES path check
    let report = scan(&[(
        "app/service.py",
        python_fixture!("v03/architecture_positive.txt"),
    )]);
    assert_rules_present(
        &report,
        &[
            "service_method_accepts_http_request_object",
            "coroutine_result_discarded_without_await",
            "asyncio_sleep_zero_busy_wait_pattern",
            "threading_lock_acquired_blocking_inside_async_def",
            "loop_run_until_complete_inside_running_loop",
        ],
    );
}

#[test]
fn test_architecture_rules_negative() {
    let report = scan(&[(
        "app/service.py",
        python_fixture!("v03/architecture_negative.txt"),
    )]);
    assert_rules_absent(
        &report,
        &[
            "service_method_accepts_http_request_object",
            "coroutine_result_discarded_without_await",
            "asyncio_sleep_zero_busy_wait_pattern",
            "threading_lock_acquired_blocking_inside_async_def",
            "loop_run_until_complete_inside_running_loop",
        ],
    );
}

// ─── Discipline family ────────────────────────────────────────────────────────

#[test]
fn test_discipline_rules_positive() {
    let report = scan(&[(
        "app/logic.py",
        python_fixture!("v03/discipline_positive.txt"),
    )]);
    assert_rules_present(
        &report,
        &[
            "exception_raised_without_chaining_original_cause",
            "bare_except_clause_catches_system_exit",
            "assert_used_for_runtime_input_validation_in_production",
        ],
    );
}

#[test]
fn test_discipline_rules_negative() {
    let report = scan(&[(
        "app/logic.py",
        python_fixture!("v03/discipline_negative.txt"),
    )]);
    assert_rules_absent(
        &report,
        &[
            "exception_raised_without_chaining_original_cause",
            "bare_except_clause_catches_system_exit",
            "assert_used_for_runtime_input_validation_in_production",
        ],
    );
}

// ─── Boundaries / Security family ────────────────────────────────────────────

#[test]
fn test_boundaries_rules_positive() {
    let report = scan(&[(
        "app/data_access.py",
        python_fixture!("v03/boundaries_positive.txt"),
    )]);
    assert_rules_present(
        &report,
        &[
            "sql_query_built_with_string_formatting_instead_of_parameters",
            "subprocess_invoked_with_shell_true_and_user_derived_input",
            "insecure_hash_algorithm_used_for_security_sensitive_purpose",
            "weak_random_function_used_for_security_token_generation",
            "yaml_config_loaded_without_safe_loader",
        ],
    );
}

#[test]
fn test_boundaries_rules_negative() {
    let report = scan(&[(
        "app/data_access.py",
        python_fixture!("v03/boundaries_negative.txt"),
    )]);
    assert_rules_absent(
        &report,
        &[
            "sql_query_built_with_string_formatting_instead_of_parameters",
            "subprocess_invoked_with_shell_true_and_user_derived_input",
            "insecure_hash_algorithm_used_for_security_sensitive_purpose",
            "weak_random_function_used_for_security_token_generation",
            "yaml_config_loaded_without_safe_loader",
        ],
    );
}

// ─── Observability / Algorithm family ────────────────────────────────────────

#[test]
fn test_observability_rules_positive() {
    let report = scan(&[(
        "app/processor.py",
        python_fixture!("v03/observability_positive.txt"),
    )]);
    assert_rules_present(
        &report,
        &[
            "f_string_evaluated_eagerly_inside_logging_call",
            "logger_error_inside_except_without_exc_info",
            "sorted_full_collection_to_extract_top_n_elements",
            "zip_range_len_used_instead_of_enumerate",
        ],
    );
}

#[test]
fn test_observability_rules_negative() {
    let report = scan(&[(
        "app/processor.py",
        python_fixture!("v03/observability_negative.txt"),
    )]);
    assert_rules_absent(
        &report,
        &[
            "f_string_evaluated_eagerly_inside_logging_call",
            "logger_error_inside_except_without_exc_info",
            "sorted_full_collection_to_extract_top_n_elements",
            "zip_range_len_used_instead_of_enumerate",
        ],
    );
}
