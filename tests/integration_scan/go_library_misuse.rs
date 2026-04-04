use super::FixtureWorkspace;

fn has_rule(report: &deslop::ScanReport, rule_id: &str) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.rule_id == rule_id)
}

const PERF_RULES: &[&str] = &[
    "strings_contains_vs_index",
    "string_to_byte_for_single_char_check",
    "string_concatenation_for_path_join",
    "sprintf_for_simple_int_to_string",
    "sprintf_for_simple_string_format",
    "strings_replace_all_for_single_char",
    "repeated_string_trim_normalize",
    "len_string_for_empty_check",
    "string_format_for_error_wrap",
    "strings_hasprefix_then_trimprefix",
    "strings_hassuffix_then_trimsuffix",
    "string_builder_write_string_vs_plus",
    "copy_append_idiom_waste",
    "map_delete_in_loop_vs_new_map",
    "sort_slice_vs_sort_sort",
    "range_over_string_by_index",
    "map_lookup_double_access",
    "slice_grow_without_cap_hint",
    "interface_slice_allocation",
    "map_of_slices_prealloc",
    "clear_map_go121",
    "unnecessary_slice_copy_for_readonly",
    "three_index_slice_for_append_safety",
    "range_copy_large_struct",
    "unnecessary_map_for_set_of_ints",
    "sync_mutex_for_atomic_counter",
    "sync_mutex_for_readonly_config",
    "sync_pool_ignored_for_frequent_small_allocs",
    "mutex_value_receiver",
    "time_now_in_tight_loop",
    "defer_in_tight_loop",
    "select_with_single_case",
    "goroutine_for_sync_work",
    "unbuffered_channel_for_known_producer_count",
    "waitgroup_add_inside_loop",
    "ioutil_readall_still_used",
    "json_marshal_then_write",
    "binary_read_for_single_field",
    "json_number_vs_float64_decode",
    "xml_decoder_without_strict",
    "csv_reader_reuse_record",
    "bufio_scanner_small_buffer_for_large_lines",
    "http_body_readall_without_limitreader",
    "type_assertion_without_comma_ok",
    "type_switch_vs_repeated_assertions",
    "errors_new_for_static_sentinel",
    "fmt_errorf_without_wrap_verb",
    "error_string_comparison",
    "empty_interface_parameter_overuse",
    "panic_for_expected_errors",
];

const SECURITY_RULES: &[&str] = &[
    "insecure_random_for_security",
    "hardcoded_tls_skip_verify",
    "hardcoded_tls_min_version_too_low",
    "weak_hash_for_integrity",
    "constant_encryption_key",
    "constant_iv_or_nonce",
    "ecb_mode_cipher",
    "jwt_none_algorithm_risk",
    "bcrypt_cost_too_low",
    "rsa_key_size_too_small",
    "os_exec_command_with_user_input",
    "template_html_unescaped",
    "text_template_for_html",
    "filepath_join_with_user_path",
    "url_redirect_without_validation",
    "ssrf_via_user_controlled_url",
    "ldap_injection_via_string_concat",
    "header_injection_via_user_input",
    "xml_decoder_without_entity_limit",
    "yaml_unmarshal_untrusted_input",
    "cookie_without_secure_flag",
    "cookie_without_httponly",
    "cookie_without_samesite",
    "cors_allow_all_origins",
    "jwt_secret_in_source",
    "timing_attack_on_token_comparison",
    "missing_rate_limiting_on_auth_endpoint",
    "password_stored_as_plaintext",
    "race_on_shared_map",
    "toctou_file_check_then_open",
    "shared_slice_append_race",
    "goroutine_captures_loop_variable",
    "unsafe_pointer_cast",
    "cgo_string_lifetime",
    "global_rand_source_contention",
    "http_handler_without_csrf_protection",
    "http_handler_missing_security_headers",
    "http_listen_non_tls",
    "dns_lookup_for_access_control",
    "websocket_without_origin_check",
    "grpc_without_tls_credentials",
    "ssh_host_key_callback_insecure",
    "smtp_plaintext_auth",
    "sensitive_data_in_log",
    "error_detail_leaked_to_client",
    "debug_endpoint_in_production",
    "struct_field_exposed_in_json",
    "temp_file_predictable_name",
    "world_readable_file_permissions",
    "fmt_print_of_sensitive_struct",
    "panic_stack_trace_to_client",
    "env_var_in_error_message",
];

const LIBRARY_RULES: &[&str] = &[
    "redis_ping_per_request",
    "redis_get_set_without_pipeline",
    "redis_keys_command_in_handler",
    "redis_connection_per_request",
    "redis_large_value_without_compression",
    "redis_no_ttl_on_cache_keys",
    "grpc_dial_per_request",
    "grpc_large_message_without_streaming",
    "grpc_context_not_propagated",
    "grpc_no_keepalive_config",
    "grpc_unary_interceptor_per_rpc",
    "log_level_check_after_format",
    "logger_created_per_request",
    "string_format_in_structured_logger",
    "log_printf_for_production",
    "error_logged_and_returned",
    "viper_get_in_hot_path",
    "os_getenv_in_hot_path",
    "config_file_read_per_request",
    "cobra_flag_lookup_in_run",
    "env_parsing_repeated_in_init",
    "prometheus_counter_created_per_request",
    "prometheus_high_cardinality_labels",
    "prometheus_observe_without_timer",
    "prometheus_unregistered_metric",
    "aws_session_per_request",
    "s3_getobject_without_range",
    "aws_credential_hardcoded",
    "s3_listobjects_without_pagination",
    "dynamodb_scan_in_handler",
];

// ── Performance rules (plan1) ──

#[test]
fn test_go_library_misuse_perf_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "perf_positive.go",
        go_fixture!("library_misuse_perf_positive.txt"),
    );

    let report = workspace.scan();

    for rule_id in PERF_RULES {
        assert!(has_rule(&report, rule_id), "missing rule: {rule_id}");
    }
}

#[test]
fn test_go_library_misuse_perf_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "perf_clean.go",
        go_fixture!("library_misuse_perf_clean.txt"),
    );

    let report = workspace.scan();

    for rule_id in PERF_RULES {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }
}

// ── Security rules (plan2) ──

#[test]
fn test_go_library_misuse_security_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "sec_positive.go",
        go_fixture!("library_misuse_security_positive.txt"),
    );

    let report = workspace.scan();

    for rule_id in SECURITY_RULES {
        assert!(has_rule(&report, rule_id), "missing rule: {rule_id}");
    }
}

#[test]
fn test_go_library_misuse_security_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "sec_clean.go",
        go_fixture!("library_misuse_security_clean.txt"),
    );

    let report = workspace.scan();

    for rule_id in SECURITY_RULES {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }
}

// ── Library rules (plan3) ──

#[test]
fn test_go_library_misuse_library_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "lib_positive.go",
        go_fixture!("library_misuse_library_positive.txt"),
    );

    let report = workspace.scan();

    for rule_id in LIBRARY_RULES {
        assert!(has_rule(&report, rule_id), "missing rule: {rule_id}");
    }
}

#[test]
fn test_go_library_misuse_library_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "lib_clean.go",
        go_fixture!("library_misuse_library_clean.txt"),
    );

    let report = workspace.scan();

    for rule_id in LIBRARY_RULES {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }
}
