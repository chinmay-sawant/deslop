use super::FixtureWorkspace;

fn has_rule(report: &deslop::ScanReport, rule_id: &str) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.rule_id == rule_id)
}

#[test]
fn test_go_advanceplan2_channel_and_timer_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "channel_range.go",
        go_fixture!("channel_range_without_close_positive.txt"),
    );
    workspace.write_file(
        "double_close.go",
        go_fixture!("double_close_channel_positive.txt"),
    );
    workspace.write_file(
        "send_after_close.go",
        go_fixture!("send_after_close_positive.txt"),
    );
    workspace.write_file(
        "time_after.go",
        go_fixture!("time_after_in_loop_positive.txt"),
    );
    workspace.write_file("ticker.go", go_fixture!("ticker_without_stop_positive.txt"));

    let report = workspace.scan();

    assert!(has_rule(&report, "range_over_local_channel_without_close"));
    assert!(has_rule(&report, "double_close_local_channel"));
    assert!(has_rule(&report, "send_after_local_close_risk"));
    assert!(has_rule(&report, "time_after_in_loop"));
    assert!(has_rule(&report, "ticker_without_stop"));
}

#[test]
fn test_go_advanceplan2_channel_and_timer_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "channel_clean.go",
        go_fixture!("channel_lifecycle_clean.txt"),
    );

    let report = workspace.scan();

    for rule_id in [
        "range_over_local_channel_without_close",
        "double_close_local_channel",
        "send_after_local_close_risk",
        "time_after_in_loop",
        "ticker_without_stop",
    ] {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }
}

#[test]
fn test_go_advanceplan2_http_boundary_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "http_response.go",
        go_fixture!("http_response_close_positive.txt"),
    );
    workspace.write_file(
        "http_client.go",
        go_fixture!("http_client_timeout_positive.txt"),
    );
    workspace.write_file(
        "http_server.go",
        go_fixture!("http_server_timeout_positive.txt"),
    );
    workspace.write_file(
        "http_status.go",
        go_fixture!("http_status_check_positive.txt"),
    );
    workspace.write_file(
        "http_writeheader.go",
        go_fixture!("http_writeheader_order_positive.txt"),
    );

    let report = workspace.scan();

    assert!(has_rule(&report, "http_response_body_not_closed"));
    assert!(has_rule(&report, "http_client_without_timeout"));
    assert!(has_rule(&report, "http_server_without_timeouts"));
    assert!(has_rule(&report, "http_status_ignored_before_decode"));
    assert!(has_rule(&report, "http_writeheader_after_write"));
}

#[test]
fn test_go_advanceplan2_http_boundary_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("http_clean.go", go_fixture!("http_boundary_clean.txt"));

    let report = workspace.scan();

    for rule_id in [
        "http_response_body_not_closed",
        "http_client_without_timeout",
        "http_server_without_timeouts",
        "http_status_ignored_before_decode",
        "http_writeheader_after_write",
    ] {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }
}

#[test]
fn test_go_advanceplan2_resource_hygiene_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "file_close.go",
        go_fixture!("file_without_close_positive.txt"),
    );
    workspace.write_file(
        "rows_close.go",
        go_fixture!("rows_without_close_positive.txt"),
    );
    workspace.write_file(
        "stmt_close.go",
        go_fixture!("stmt_without_close_positive.txt"),
    );
    workspace.write_file(
        "tx_rollback.go",
        go_fixture!("tx_without_rollback_positive.txt"),
    );
    workspace.write_file("defer_loop.go", go_fixture!("defer_in_loop_positive.txt"));

    let report = workspace.scan();

    assert!(has_rule(&report, "file_handle_without_close"));
    assert!(has_rule(&report, "rows_without_close"));
    assert!(has_rule(&report, "stmt_without_close"));
    assert!(has_rule(&report, "tx_without_rollback_guard"));
    assert!(has_rule(&report, "defer_in_loop_resource_growth"));
}

#[test]
fn test_go_advanceplan2_resource_hygiene_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "resource_clean.go",
        go_fixture!("resource_hygiene_clean.txt"),
    );

    let report = workspace.scan();

    for rule_id in [
        "file_handle_without_close",
        "rows_without_close",
        "stmt_without_close",
        "tx_without_rollback_guard",
        "defer_in_loop_resource_growth",
    ] {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }
}

#[test]
fn test_request_query_param_does_not_trigger_rows_without_close() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("handler.go", go_fixture!("context_query_param_clean.txt"));

    let report = workspace.scan();

    assert!(!has_rule(&report, "rows_without_close"));
}

#[test]
fn test_go_advanceplan2_architecture_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "mutable_global.go",
        go_fixture!("mutable_package_global_positive.txt"),
    );
    workspace.write_file(
        "init_side_effect.go",
        go_fixture!("init_side_effect_positive.txt"),
    );
    workspace.write_file(
        "single_impl_a.go",
        go_fixture!("single_impl_interface_positive_a.txt"),
    );
    workspace.write_file(
        "single_impl_b.go",
        go_fixture!("single_impl_interface_positive_b.txt"),
    );
    workspace.write_file(
        "passthrough.go",
        go_fixture!("passthrough_wrapper_positive.txt"),
    );
    workspace.write_file(
        "public_bool.go",
        go_fixture!("public_bool_parameter_api_positive.txt"),
    );

    let report = workspace.scan();

    assert!(has_rule(&report, "mutable_package_global"));
    assert!(has_rule(&report, "init_side_effect"));
    assert!(has_rule(&report, "single_impl_interface"));
    assert!(has_rule(&report, "passthrough_wrapper_interface"));
    assert!(has_rule(&report, "public_bool_parameter_api"));
}

#[test]
fn test_go_advanceplan2_architecture_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "architecture_clean.go",
        go_fixture!("architecture_clean_a.txt"),
    );

    let report = workspace.scan();

    for rule_id in [
        "mutable_package_global",
        "init_side_effect",
        "single_impl_interface",
        "passthrough_wrapper_interface",
        "public_bool_parameter_api",
    ] {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }
}
