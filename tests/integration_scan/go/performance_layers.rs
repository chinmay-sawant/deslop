use super::{assert_rules_absent, assert_rules_present, scan_files};

fn assert_go_perf_layer_pair(rule_id: &str, positive_fixture: &str, clean_fixture: &str) {
    let report = scan_files(&[("positive.go", positive_fixture)]);
    assert_rules_present(&report, &[rule_id]);

    let report = scan_files(&[("clean.go", clean_fixture)]);
    assert_rules_absent(&report, &[rule_id]);
}

const GO_PERFORMANCE_LAYER_BATCH_1: &[&str] = &[
    "go_perf_layer_string_handling_regexp_compile_in_request_path",
    "go_perf_layer_string_handling_fmt_sprintf_for_simple_concat",
    "go_perf_layer_memory_allocation_append_without_known_capacity",
    "go_perf_layer_garbage_collection_cleanup_ticker_not_stopped_on_exit",
];

#[test]
fn test_go_performance_layers_batch_1_positive() {
    let report = scan_files(&[(
        "performance_layers_positive.go",
        go_fixture!("performance_layers_positive.txt"),
    )]);

    assert_rules_present(&report, GO_PERFORMANCE_LAYER_BATCH_1);
}

#[test]
fn test_go_performance_layers_batch_1_clean() {
    let report = scan_files(&[(
        "performance_layers_clean.go",
        go_fixture!("performance_layers_clean.txt"),
    )]);

    assert_rules_absent(&report, GO_PERFORMANCE_LAYER_BATCH_1);
}

#[test]
fn test_go_performance_layer_retry_without_backoff_positive() {
    let report = scan_files(&[(
        "performance_layers_retry_positive.go",
        go_fixture!("performance_layers_retry_positive.txt"),
    )]);

    assert_rules_present(&report, &["go_perf_layer_network_calls_retry_loop_without_backoff"]);
}

#[test]
fn test_go_performance_layer_retry_without_backoff_clean() {
    let report = scan_files(&[(
        "performance_layers_retry_clean.go",
        go_fixture!("performance_layers_retry_clean.txt"),
    )]);

    assert_rules_absent(&report, &["go_perf_layer_network_calls_retry_loop_without_backoff"]);
}

#[test]
fn test_go_performance_layer_string_lower_compare_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_string_handling_string_lower_for_case_insensitive_compare",
        go_fixture!("performance_layers_string_lower_positive.txt"),
        go_fixture!("performance_layers_string_lower_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_temp_byte_slice_write_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write",
        go_fixture!("performance_layers_temp_byte_write_positive.txt"),
        go_fixture!("performance_layers_temp_byte_write_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_static_lookup_map_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_memory_allocation_map_recreated_for_static_lookup",
        go_fixture!("performance_layers_static_lookup_positive.txt"),
        go_fixture!("performance_layers_static_lookup_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_base64_roundtrip_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_serialization_base64_roundtrip_for_binary_transport",
        go_fixture!("performance_layers_base64_roundtrip_positive.txt"),
        go_fixture!("performance_layers_base64_roundtrip_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_copy_slice_readonly_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_collection_iteration_copy_slice_before_readonly_range",
        go_fixture!("performance_layers_copy_slice_positive.txt"),
        go_fixture!("performance_layers_copy_slice_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_select_default_busy_poll_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_async_concurrency_select_default_busy_poll",
        go_fixture!("busy_waiting_slop.txt"),
        go_fixture!("busy_waiting_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_strings_join_single_element_loop_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_string_handling_strings_join_single_element_loop",
        go_fixture!("performance_layers_strings_join_positive.txt"),
        go_fixture!("performance_layers_strings_join_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_bytes_buffer_per_record_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record",
        go_fixture!("performance_layers_bytes_buffer_positive.txt"),
        go_fixture!("performance_layers_bytes_buffer_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_response_body_drain_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_garbage_collection_cleanup_response_body_not_drained_for_reuse",
        go_fixture!("performance_layers_response_body_positive.txt"),
        go_fixture!("performance_layers_response_body_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_json_cache_string_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_caching_json_cache_value_stored_as_string",
        go_fixture!("performance_layers_json_cache_positive.txt"),
        go_fixture!("performance_layers_json_cache_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_time_now_many_times_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item",
        go_fixture!("performance_layers_time_now_positive.txt"),
        go_fixture!("performance_layers_time_now_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_stat_before_open_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_io_operations_stat_before_open_without_branch",
        go_fixture!("performance_layers_stat_open_positive.txt"),
        go_fixture!("performance_layers_stat_open_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_sentinel_error_per_call_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call",
        go_fixture!("performance_layers_sentinel_error_positive.txt"),
        go_fixture!("performance_layers_sentinel_error_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_tls_config_per_request_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_network_calls_tls_config_built_per_request",
        go_fixture!("performance_layers_tls_config_positive.txt"),
        go_fixture!("performance_layers_tls_config_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_map_first_value_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_collection_iteration_range_over_map_for_deterministic_first",
        go_fixture!("performance_layers_map_first_positive.txt"),
        go_fixture!("performance_layers_map_first_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_cache_key_fmt_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_caching_cache_key_built_with_fmt",
        go_fixture!("performance_layers_cache_key_fmt_positive.txt"),
        go_fixture!("performance_layers_cache_key_fmt_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_fmt_log_loop_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_logging_overhead_fmt_log_message_in_loop",
        go_fixture!("performance_layers_fmt_log_loop_positive.txt"),
        go_fixture!("performance_layers_fmt_log_loop_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_byte_string_contains_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_string_handling_byte_string_roundtrip_for_contains",
        go_fixture!("performance_layers_byte_string_contains_positive.txt"),
        go_fixture!("performance_layers_byte_string_contains_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_map_string_bool_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_data_structure_choice_map_string_bool_for_membership",
        go_fixture!("performance_layers_map_string_bool_positive.txt"),
        go_fixture!("performance_layers_map_string_bool_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_small_writes_without_bufio_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_io_operations_small_writes_without_bufio_writer",
        go_fixture!("performance_layers_small_writes_positive.txt"),
        go_fixture!("performance_layers_small_writes_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_readall_large_file_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_io_operations_readall_on_known_large_file",
        go_fixture!("performance_layers_readall_large_positive.txt"),
        go_fixture!("performance_layers_readall_large_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_unbounded_cache_map_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_caching_unbounded_cache_map",
        go_fixture!("performance_layers_unbounded_cache_positive.txt"),
        go_fixture!("performance_layers_unbounded_cache_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_multiple_passes_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_collection_iteration_multiple_passes_for_independent_counts",
        go_fixture!("performance_layers_multiple_passes_positive.txt"),
        go_fixture!("performance_layers_multiple_passes_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_scanner_large_token_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_io_operations_scanner_used_for_large_token_stream",
        go_fixture!("performance_layers_scanner_large_positive.txt"),
        go_fixture!("performance_layers_scanner_large_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_json_marshal_deep_equal_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_serialization_json_marshal_for_deep_equal",
        go_fixture!("performance_layers_json_deep_equal_positive.txt"),
        go_fixture!("performance_layers_json_deep_equal_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_error_string_built_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_error_handling_cost_error_string_built_before_error_needed",
        go_fixture!("performance_layers_error_string_positive.txt"),
        go_fixture!("performance_layers_error_string_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_benchmark_missing_allocs_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report",
        go_fixture!("performance_layers_benchmark_allocs_positive.txt"),
        go_fixture!("performance_layers_benchmark_allocs_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_benchmark_setup_in_loop_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_profiling_benchmarking_benchmark_includes_setup_in_loop",
        go_fixture!("performance_layers_benchmark_setup_positive.txt"),
        go_fixture!("performance_layers_benchmark_setup_clean.txt"),
    );
}

#[test]
fn test_go_performance_layer_dead_code_benchmark_pair() {
    assert_go_perf_layer_pair(
        "go_perf_layer_profiling_benchmarking_microbenchmark_dead_code_eliminated",
        go_fixture!("performance_layers_benchmark_dead_code_positive.txt"),
        go_fixture!("performance_layers_benchmark_dead_code_clean.txt"),
    );
}