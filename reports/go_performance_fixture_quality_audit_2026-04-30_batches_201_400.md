# Go Performance Fixture Quality Audit - 2026-04-30 (Batches 201-400)

## Scope

- Audited the next 200 sorted Go performance rule-coverage fixtures, covering sorted indices 201-400.
- Audit execution used GPT-5.4 subagents in 2 parallel 100-file batches.
- Audit standard:
  - `_positive.txt` must clearly contain the named rule behavior.
  - `_negative.txt` must clearly avoid the named rule behavior while staying semantically close.
- Repairs in this pass were limited to high-confidence direct mismatches with local, concrete fixes.

## Confirmed Findings

- Batch 201-300: 48 confirmed mismatches.
- Batch 301-400: 39 confirmed mismatches.
- Total confirmed mismatches across sorted performance fixtures 201-400: 87.

## Repairs Applied In This Pass

### Batch 301-400

- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_string_handling_string_lower_for_case_insensitive_compare_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_string_handling_strings_join_single_element_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_string_handling_regexp_compile_in_request_path_negative.txt`
- `tests/fixtures/go/rule_coverage/performance/goroutine_for_sync_work_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/hmac_new_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/ioutil_readall_still_used_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/json_indent_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/json_marshal_then_write_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/len_string_for_empty_check_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/panic_for_expected_errors_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/rand_new_per_call_negative.txt`
- `tests/fixtures/go/rule_coverage/performance/rand_newsource_per_call_negative.txt`
- `tests/fixtures/go/rule_coverage/performance/rand_seed_per_call_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/range_over_string_by_index_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/runtime_numcpu_inside_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/select_with_single_case_positive.txt`

### Batch 201-300

- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_garbage_collection_cleanup_response_body_not_drained_for_reuse_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_garbage_collection_cleanup_ticker_not_stopped_on_exit_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_io_operations_scanner_used_for_large_token_stream_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_io_operations_small_writes_without_bufio_writer_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_io_operations_stat_before_open_without_branch_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_network_calls_http_client_created_per_call_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_network_calls_retry_loop_without_backoff_positive.txt`

### Additional repairs landed in the next continuation

#### Batch 201-300

- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_framework_performance_grpc_metadata_parsed_repeatedly_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_framework_performance_template_execute_to_string_then_write_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_framework_performance_sqlx_select_slice_unbounded_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_garbage_collection_cleanup_finalizer_used_for_regular_cleanup_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_hot_path_optimization_invariant_parse_inside_handler_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_io_operations_temporary_file_for_stream_transform_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_lazy_loading_eager_connect_to_all_backends_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_lazy_loading_eager_load_optional_config_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_lazy_loading_eager_template_parse_for_unused_routes_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_lazy_loading_lazy_once_hides_slow_first_request_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_logging_overhead_log_fields_built_before_level_check_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_logging_overhead_log_payload_serialized_before_sampling_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_logging_overhead_logger_with_fields_per_request_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_logging_overhead_per_record_debug_log_in_batch_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_memory_allocation_closure_capture_allocates_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_network_calls_dns_lookup_per_request_path_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_network_calls_tls_config_built_per_request_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_network_calls_transport_without_connection_limits_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_profiling_benchmarking_microbenchmark_dead_code_eliminated_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_profiling_benchmarking_pprof_endpoint_enabled_without_sampling_plan_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_resource_pooling_db_pool_created_per_repository_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_resource_pooling_http_transport_per_service_method_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_resource_pooling_rate_limiter_per_request_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_resource_pooling_worker_pool_without_shutdown_backpressure_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_runtime_configuration_gogc_forced_low_without_measurement_positive.txt`

#### Batch 301-400

- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_serialization_base64_roundtrip_for_binary_transport_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_serialization_gzip_writer_created_per_small_payload_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_serialization_json_decoder_without_reuse_for_stream_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_serialization_json_marshal_for_deep_equal_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_serialization_map_any_json_decode_in_hot_path_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/go_perf_layer_string_handling_byte_string_roundtrip_for_contains_positive.txt`

#### Batch 301-400 (latest continuation)

- `tests/fixtures/go/rule_coverage/performance/http_body_readall_without_limitreader_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/interface_slice_allocation_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/json_number_vs_float64_decode_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/likely_n_squared_allocation_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/likely_n_squared_string_concat_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/likely_unindexed_query_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/map_delete_in_loop_vs_new_map_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/map_lookup_double_access_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/map_of_slices_prealloc_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/mutex_value_receiver_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/path_split_base_only_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/path_split_dir_only_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/range_copy_large_struct_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/reflection_hot_path_positive.txt`

Total repairs landed from the 201-400 continuation so far: 75 files.

## Validation

Passed after repairs:

- `cargo test go_rule_fixture_batch_300_399_is_parseable_scenario_code`
- `cargo test go_rule_fixture_batch_200_299_is_parseable_scenario_code`
- `cargo test go_rule_fixture_batch_200_299_is_parseable_scenario_code` (rerun after framework/logging burst)
- `cargo test go_rule_fixture_batch_200_299_is_parseable_scenario_code` (rerun after network/resource burst)
- `cargo test go_rule_fixture_batch_200_299_is_parseable_scenario_code` (rerun after lazy-loading/profiling burst)
- `cargo test go_rule_fixture_batch_300_399_is_parseable_scenario_code` (rerun after serialization burst)
- `cargo test go_rule_fixture_batch_300_399_is_parseable_scenario_code` (rerun after HTTP/map/path/interface burst)
- `cargo test go_rule_fixture_batch_300_399_is_parseable_scenario_code` (rerun after n-squared/reflection burst)

## Remaining Work From This 201-400 Audit

- Remaining confirmed mismatches from sorted performance fixtures 201-400: 12.
- Remaining work is now concentrated in the harder benchmark-style, hot-path/runtime-configuration, and a small set of nuanced 201-300 positives that are not simple API-swap repairs.
- Next practical target is to finish the last 201-400 benchmark/runtime/hot-path positives, then recheck whether any residual 301-400 direct cases remain.
