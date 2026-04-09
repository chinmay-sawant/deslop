use super::{FixtureWorkspace, assert_rules_absent, assert_rules_present};

#[test]
fn test_missing_ctx_http() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("http.go", go_fixture!("missing_context.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "missing_context")
    );
}

#[test]
fn test_ctx_http() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("http.go", go_fixture!("context_aware_http.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "missing_context")
    );
}

#[test]
fn test_missing_cancel() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("cancel.go", go_fixture!("context_cancel_slop.txt"));

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "missing_cancel_call" && finding.function_name.as_deref() == Some("Run")
    }));
}

#[test]
fn test_ctx_cancel() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("cancel.go", go_fixture!("context_cancel_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "missing_cancel_call")
    );
}

#[test]
fn test_sleep_loops() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("poll.go", go_fixture!("sleep_polling.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "sleep_polling")
    );
}

#[test]
fn test_sleep_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("poll.go", go_fixture!("sleep_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "sleep_polling")
    );
}

#[test]
fn test_busy_wait() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("wait.go", go_fixture!("busy_waiting_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "busy_waiting")
    );
}

#[test]
fn test_select_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("wait.go", go_fixture!("busy_waiting_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "busy_waiting")
    );
}

#[test]
fn test_missing_ctx_exec() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("exec.go", go_fixture!("missing_context_exec.txt"));

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "missing_context" && finding.message.contains("context-aware work")
    }));
}

#[test]
fn test_ctx_exec() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("exec.go", go_fixture!("context_aware_exec.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "missing_context")
    );
}

#[test]
fn test_context_wrapper_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("wrapper.go", go_fixture!("context_wrapper_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "missing_context_propagation")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "context_background_used")
    );
}

#[test]
fn test_context_wrapper_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("wrapper.go", go_fixture!("context_wrapper_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "missing_context_propagation")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "context_background_used")
    );
}

#[test]
fn test_context_wrapper_alias_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "wrapper_alias.go",
        go_fixture!("context_wrapper_alias_slop.txt"),
    );

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "missing_context_propagation"
            && finding.function_name.as_deref() == Some("Fetch")
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "context_background_used"
            && finding.function_name.as_deref() == Some("Fetch")
    }));
}

#[test]
fn test_context_receiver_wrapper_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "receiver_wrapper.go",
        go_fixture!("context_receiver_wrapper_slop.txt"),
    );

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "missing_context_propagation"
            && finding.function_name.as_deref() == Some("Fetch")
    }));
}

#[test]
fn test_context_nested_wrapper_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "nested_wrapper.go",
        go_fixture!("context_nested_wrapper_slop.txt"),
    );

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "missing_context_propagation"
            && finding.function_name.as_deref() == Some("Fetch")
            && finding.message.contains("wrapper chain")
    }));
}

#[test]
fn test_request_handler_background_context_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "handler.go",
        go_fixture!("context_request_handler_background_slop.txt"),
    );

    let report = workspace.scan();
    assert_rules_present(&report, &["context_background_used"]);
}

#[test]
fn test_request_handler_background_context_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "handler.go",
        go_fixture!("context_request_handler_background_clean.txt"),
    );

    let report = workspace.scan();
    assert_rules_absent(&report, &["context_background_used"]);
}

#[test]
fn test_cache_interface_missing_context() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/cache/patient_cache.go",
        go_fixture!("context_cache_interface_positive.txt"),
    );

    let report = workspace.scan();
    assert_rules_present(&report, &["cache_interface_method_missing_context"]);
}

#[test]
fn test_cache_interface_missing_context_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/cache/patient_cache.go",
        go_fixture!("context_cache_interface_clean.txt"),
    );

    let report = workspace.scan();
    assert_rules_absent(&report, &["cache_interface_method_missing_context"]);
}

#[test]
fn test_cache_method_uses_context_background() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/cache/patient_cache.go",
        go_fixture!("context_cache_background_positive.txt"),
    );

    let report = workspace.scan();
    assert_rules_present(&report, &["cache_method_uses_context_background"]);
}

#[test]
fn test_cache_method_uses_context_background_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/cache/patient_cache.go",
        go_fixture!("context_cache_background_clean.txt"),
    );

    let report = workspace.scan();
    assert_rules_absent(&report, &["cache_method_uses_context_background"]);
}

#[test]
fn test_context_db_query_wrapper_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("db_wrapper.go", go_fixture!("context_db_query_slop.txt"));

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "missing_context_propagation"
            && finding.function_name.as_deref() == Some("Load")
            && finding.message.contains("context-aware DB variant")
    }));
}

#[test]
fn test_request_query_param_is_not_treated_as_missing_context_propagation() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("handler.go", go_fixture!("context_query_param_clean.txt"));

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "missing_context_propagation"
            && finding.function_name.as_deref() == Some("Handle")
    }));
}

#[test]
fn test_documented_context_detach_is_allowed() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "detach.go",
        go_fixture!("context_documented_detach_clean.txt"),
    );

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        matches!(
            finding.rule_id.as_str(),
            "missing_context_propagation" | "context_background_used"
        )
    }));
}

#[test]
fn test_context_propagation_severity_override() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "receiver_wrapper.go",
        go_fixture!("context_receiver_wrapper_slop.txt"),
    );
    workspace.write_file(
        ".deslop.toml",
        "[severity_overrides]\nmissing_context_propagation = \"error\"\n",
    );

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "missing_context_propagation"
            && matches!(finding.severity, deslop::Severity::Error)
    }));
}
