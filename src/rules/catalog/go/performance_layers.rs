use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

macro_rules! perf_layer_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Go,
            family: "performance",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: super::bindings::GO_PERFORMANCE,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    // Algorithmic complexity
    perf_layer_rule!(
        "go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans",
        "Layer: Algorithmic complexity. Flag nested slice scans that should be indexed or mapped before joining large collections."
    ),
    perf_layer_rule!(
        "go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline",
        "Layer: Algorithmic complexity. Flag append-based filter pipelines that rescan accumulated results for every input item."
    ),
    perf_layer_rule!(
        "go_perf_layer_algorithmic_complexity_sort_before_linear_dedup",
        "Layer: Algorithmic complexity. Flag full sorts used only to remove duplicates where map-backed membership would scale better."
    ),
    perf_layer_rule!(
        "go_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen_set",
        "Layer: Algorithmic complexity. Flag recursive graph or tree walks that do not track visited nodes for cyclic or shared inputs."
    ),
    perf_layer_rule!(
        "go_perf_layer_algorithmic_complexity_per_request_topk_full_sort",
        "Layer: Algorithmic complexity. Flag request paths that sort all candidates when heap-based top-k selection would bound work."
    ),
    // Data structure choice
    perf_layer_rule!(
        "go_perf_layer_data_structure_choice_map_string_bool_for_membership",
        "Layer: Data structure choice. Flag map[string]bool membership sets where map[string]struct{} avoids unnecessary value storage."
    ),
    perf_layer_rule!(
        "go_perf_layer_data_structure_choice_slice_queue_pop_front",
        "Layer: Data structure choice. Flag queue-style slice reslicing from the front that can retain backing arrays or shift too much data."
    ),
    perf_layer_rule!(
        "go_perf_layer_data_structure_choice_interface_map_for_typed_values",
        "Layer: Data structure choice. Flag map[string]any used for homogeneous hot-path values where typed structs avoid assertions."
    ),
    perf_layer_rule!(
        "go_perf_layer_data_structure_choice_linked_list_for_cache_iteration",
        "Layer: Data structure choice. Flag container/list use in cache iteration paths where slices or maps would improve locality."
    ),
    perf_layer_rule!(
        "go_perf_layer_data_structure_choice_small_enum_string_switch_map",
        "Layer: Data structure choice. Flag map lookups for tiny static enum-like string sets where switch can avoid allocation and hashing."
    ),
    // Memory allocation
    perf_layer_rule!(
        "go_perf_layer_memory_allocation_append_without_known_capacity",
        "Layer: Memory allocation. Flag slice appends in builders where an obvious count is available for make capacity."
    ),
    perf_layer_rule!(
        "go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record",
        "Layer: Memory allocation. Flag bytes.Buffer construction per record when a reusable buffer can be reset safely."
    ),
    perf_layer_rule!(
        "go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write",
        "Layer: Memory allocation. Flag []byte(string) conversions used only to write strings to writers."
    ),
    perf_layer_rule!(
        "go_perf_layer_memory_allocation_closure_capture_allocates_in_loop",
        "Layer: Memory allocation. Flag loop-local closures that capture variables in high-frequency paths and may force heap allocation."
    ),
    perf_layer_rule!(
        "go_perf_layer_memory_allocation_map_recreated_for_static_lookup",
        "Layer: Memory allocation. Flag static lookup maps rebuilt inside functions instead of package-level immutable tables."
    ),
    // Garbage collection and cleanup
    perf_layer_rule!(
        "go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate",
        "Layer: Garbage collection and cleanup. Flag long-lived slices truncated to zero that still retain large backing arrays."
    ),
    perf_layer_rule!(
        "go_perf_layer_garbage_collection_cleanup_ticker_not_stopped_on_exit",
        "Layer: Garbage collection and cleanup. Flag time.Ticker creation without a visible Stop path in services or workers."
    ),
    perf_layer_rule!(
        "go_perf_layer_garbage_collection_cleanup_response_body_not_drained_for_reuse",
        "Layer: Garbage collection and cleanup. Flag HTTP response handling that closes bodies without draining when connection reuse matters."
    ),
    perf_layer_rule!(
        "go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers",
        "Layer: Garbage collection and cleanup. Flag sync.Pool usage that stores large buffers without a cap before putting them back."
    ),
    perf_layer_rule!(
        "go_perf_layer_garbage_collection_cleanup_finalizer_used_for_regular_cleanup",
        "Layer: Garbage collection and cleanup. Flag runtime.SetFinalizer use for ordinary resource cleanup instead of explicit Close ownership."
    ),
    // String handling
    perf_layer_rule!(
        "go_perf_layer_string_handling_fmt_sprintf_for_simple_concat",
        "Layer: String handling. Flag fmt.Sprintf used for simple string concatenation on hot or repeated paths."
    ),
    perf_layer_rule!(
        "go_perf_layer_string_handling_regexp_compile_in_request_path",
        "Layer: String handling. Flag regexp.Compile or MustCompile inside handlers, loops, or repeated functions."
    ),
    perf_layer_rule!(
        "go_perf_layer_string_handling_string_lower_for_case_insensitive_compare",
        "Layer: String handling. Flag strings.ToLower comparisons where EqualFold avoids extra allocation."
    ),
    perf_layer_rule!(
        "go_perf_layer_string_handling_strings_join_single_element_loop",
        "Layer: String handling. Flag repeated strings.Join calls in loops where one builder outside the loop would do."
    ),
    perf_layer_rule!(
        "go_perf_layer_string_handling_byte_string_roundtrip_for_contains",
        "Layer: String handling. Flag string/[]byte roundtrips around Contains/Index checks that have native package variants."
    ),
    // Collection iteration
    perf_layer_rule!(
        "go_perf_layer_collection_iteration_range_over_map_for_deterministic_first",
        "Layer: Collection iteration. Flag map iteration used to pick a deterministic first value, causing unstable work and extra sorting later."
    ),
    perf_layer_rule!(
        "go_perf_layer_collection_iteration_len_called_after_materializing_channel",
        "Layer: Collection iteration. Flag channel streams collected into slices only to measure length or emptiness."
    ),
    perf_layer_rule!(
        "go_perf_layer_collection_iteration_copy_slice_before_readonly_range",
        "Layer: Collection iteration. Flag slice copies made immediately before read-only range loops."
    ),
    perf_layer_rule!(
        "go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need",
        "Layer: Collection iteration. Flag manual index loops over slices where range would avoid repeated bounds-sensitive indexing."
    ),
    perf_layer_rule!(
        "go_perf_layer_collection_iteration_multiple_passes_for_independent_counts",
        "Layer: Collection iteration. Flag multiple full passes over the same slice for independent counters that can be combined."
    ),
    // Async and concurrency
    perf_layer_rule!(
        "go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit",
        "Layer: Async and concurrency. Flag unbounded goroutine-per-item fanout without a semaphore or worker pool."
    ),
    perf_layer_rule!(
        "go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst",
        "Layer: Async and concurrency. Flag channels created with no or tiny buffer despite a known burst size."
    ),
    perf_layer_rule!(
        "go_perf_layer_async_concurrency_mutex_held_during_slow_call",
        "Layer: Async and concurrency. Flag mutex-protected regions that include network, disk, or database calls."
    ),
    perf_layer_rule!(
        "go_perf_layer_async_concurrency_context_timeout_allocated_per_inner_call",
        "Layer: Async and concurrency. Flag context.WithTimeout creation inside tight retry loops when an outer deadline would suffice."
    ),
    perf_layer_rule!(
        "go_perf_layer_async_concurrency_select_default_busy_poll",
        "Layer: Async and concurrency. Flag select loops with default branches that spin instead of blocking or using timers."
    ),
    // I/O operations
    perf_layer_rule!(
        "go_perf_layer_io_operations_readall_on_known_large_file",
        "Layer: I/O operations. Flag os.ReadFile or io.ReadAll on files that are processed sequentially."
    ),
    perf_layer_rule!(
        "go_perf_layer_io_operations_small_writes_without_bufio_writer",
        "Layer: I/O operations. Flag many Write calls to files or network connections without bufio buffering."
    ),
    perf_layer_rule!(
        "go_perf_layer_io_operations_scanner_used_for_large_token_stream",
        "Layer: I/O operations. Flag bufio.Scanner on potentially large tokens without Buffer sizing or Reader alternatives."
    ),
    perf_layer_rule!(
        "go_perf_layer_io_operations_temporary_file_for_stream_transform",
        "Layer: I/O operations. Flag temporary files used only to bridge streaming transformations."
    ),
    perf_layer_rule!(
        "go_perf_layer_io_operations_stat_before_open_without_branch",
        "Layer: I/O operations. Flag os.Stat checks immediately followed by open/read operations without using the stat result meaningfully."
    ),
    // Database access
    perf_layer_rule!(
        "go_perf_layer_database_access_query_inside_loop_without_batching",
        "Layer: Database access. Flag SQL queries issued inside loops where batching or IN queries can bound round trips."
    ),
    perf_layer_rule!(
        "go_perf_layer_database_access_select_star_on_hot_query",
        "Layer: Database access. Flag SELECT * in hot queries where narrow projections reduce scan and decode cost."
    ),
    perf_layer_rule!(
        "go_perf_layer_database_access_rows_scan_into_map_per_row",
        "Layer: Database access. Flag row scans into map[string]any where typed structs or column slices would reduce allocation."
    ),
    perf_layer_rule!(
        "go_perf_layer_database_access_transaction_per_row_bulk_write",
        "Layer: Database access. Flag bulk writes that open and commit one transaction per row."
    ),
    perf_layer_rule!(
        "go_perf_layer_database_access_count_query_before_paged_fetch",
        "Layer: Database access. Flag count queries paired with page fetches on every request without cache or user-visible need."
    ),
    // Network calls
    perf_layer_rule!(
        "go_perf_layer_network_calls_http_client_created_per_call",
        "Layer: Network calls. Flag http.Client construction per call instead of reusing clients and transports."
    ),
    perf_layer_rule!(
        "go_perf_layer_network_calls_transport_without_connection_limits",
        "Layer: Network calls. Flag custom http.Transport values missing MaxIdleConns or MaxIdleConnsPerHost for service clients."
    ),
    perf_layer_rule!(
        "go_perf_layer_network_calls_retry_loop_without_backoff",
        "Layer: Network calls. Flag network retry loops that immediately retry and amplify load."
    ),
    perf_layer_rule!(
        "go_perf_layer_network_calls_dns_lookup_per_request_path",
        "Layer: Network calls. Flag net.Lookup* calls in request paths without caching or resolver ownership."
    ),
    perf_layer_rule!(
        "go_perf_layer_network_calls_tls_config_built_per_request",
        "Layer: Network calls. Flag TLS configuration construction per request instead of reusing immutable configs."
    ),
    // Caching
    perf_layer_rule!(
        "go_perf_layer_caching_cache_key_built_with_fmt",
        "Layer: Caching. Flag cache keys built with fmt.Sprintf in hot paths where append/Builder or structured keys are cheaper."
    ),
    perf_layer_rule!(
        "go_perf_layer_caching_unbounded_cache_map",
        "Layer: Caching. Flag package-level cache maps without size, TTL, or eviction policy."
    ),
    perf_layer_rule!(
        "go_perf_layer_caching_cache_miss_does_duplicate_work",
        "Layer: Caching. Flag cache miss paths that allow concurrent duplicate recomputation for the same key."
    ),
    perf_layer_rule!(
        "go_perf_layer_caching_json_cache_value_stored_as_string",
        "Layer: Caching. Flag caches that store JSON strings only to unmarshal them again before every use."
    ),
    perf_layer_rule!(
        "go_perf_layer_caching_per_request_config_cache_rebuild",
        "Layer: Caching. Flag request-scoped caches rebuilt from config or environment on every call."
    ),
    // Serialization and deserialization
    perf_layer_rule!(
        "go_perf_layer_serialization_json_marshal_for_deep_equal",
        "Layer: Serialization and deserialization. Flag json.Marshal used only to compare structures or build equality keys."
    ),
    perf_layer_rule!(
        "go_perf_layer_serialization_json_decoder_without_reuse_for_stream",
        "Layer: Serialization and deserialization. Flag JSON decoders created for each element of a stream instead of decoding the stream once."
    ),
    perf_layer_rule!(
        "go_perf_layer_serialization_base64_roundtrip_for_binary_transport",
        "Layer: Serialization and deserialization. Flag base64 encode/decode roundtrips inside a single process boundary."
    ),
    perf_layer_rule!(
        "go_perf_layer_serialization_map_any_json_decode_in_hot_path",
        "Layer: Serialization and deserialization. Flag JSON decoding into map[string]any in hot paths where typed structs avoid reflection churn."
    ),
    perf_layer_rule!(
        "go_perf_layer_serialization_gzip_writer_created_per_small_payload",
        "Layer: Serialization and deserialization. Flag gzip writers created for tiny payloads where compression overhead can dominate."
    ),
    // Logging overhead
    perf_layer_rule!(
        "go_perf_layer_logging_overhead_log_fields_built_before_level_check",
        "Layer: Logging overhead. Flag expensive log field construction before checking whether the level is enabled."
    ),
    perf_layer_rule!(
        "go_perf_layer_logging_overhead_fmt_log_message_in_loop",
        "Layer: Logging overhead. Flag formatted log strings built inside loops instead of structured lazy fields."
    ),
    perf_layer_rule!(
        "go_perf_layer_logging_overhead_per_record_debug_log_in_batch",
        "Layer: Logging overhead. Flag debug logging per batch record on hot ingestion paths."
    ),
    perf_layer_rule!(
        "go_perf_layer_logging_overhead_logger_with_fields_per_request",
        "Layer: Logging overhead. Flag derived logger construction per request when stable fields can be attached once."
    ),
    perf_layer_rule!(
        "go_perf_layer_logging_overhead_log_payload_serialized_before_sampling",
        "Layer: Logging overhead. Flag payload serialization for logs before sampling or level filters run."
    ),
    // Error handling cost
    perf_layer_rule!(
        "go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop",
        "Layer: Error handling cost. Flag repeated error wrapping inside loops where one aggregated context would be cheaper."
    ),
    perf_layer_rule!(
        "go_perf_layer_error_handling_cost_panic_recover_for_control_flow",
        "Layer: Error handling cost. Flag panic/recover used for expected branch control in performance-sensitive code."
    ),
    perf_layer_rule!(
        "go_perf_layer_error_handling_cost_error_string_built_before_error_needed",
        "Layer: Error handling cost. Flag error message formatting before the code knows an error will be returned."
    ),
    perf_layer_rule!(
        "go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call",
        "Layer: Error handling cost. Flag identical sentinel errors allocated inside functions instead of package-level variables."
    ),
    perf_layer_rule!(
        "go_perf_layer_error_handling_cost_multierror_append_for_success_path",
        "Layer: Error handling cost. Flag multi-error aggregators allocated on success-heavy paths before any error exists."
    ),
    // Build and runtime configuration
    perf_layer_rule!(
        "go_perf_layer_runtime_configuration_gogc_forced_low_without_measurement",
        "Layer: Build and runtime configuration. Flag runtime/debug GC tuning in application code without benchmark notes."
    ),
    perf_layer_rule!(
        "go_perf_layer_runtime_configuration_gomaxprocs_set_in_library",
        "Layer: Build and runtime configuration. Flag libraries setting GOMAXPROCS and overriding host runtime policy."
    ),
    perf_layer_rule!(
        "go_perf_layer_runtime_configuration_reflection_config_loaded_per_call",
        "Layer: Build and runtime configuration. Flag reflection-based config decoding performed per operation instead of at startup."
    ),
    perf_layer_rule!(
        "go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary",
        "Layer: Build and runtime configuration. Flag performance-sensitive binaries that keep debug/profiling build tags enabled by default."
    ),
    perf_layer_rule!(
        "go_perf_layer_runtime_configuration_race_detector_assumed_in_benchmark_numbers",
        "Layer: Build and runtime configuration. Flag benchmark guidance or scripts that compare race-enabled and normal builds directly."
    ),
    // Hot path optimization
    perf_layer_rule!(
        "go_perf_layer_hot_path_optimization_invariant_parse_inside_handler",
        "Layer: Hot path optimization. Flag handlers that parse invariant templates, schemas, or expressions per request."
    ),
    perf_layer_rule!(
        "go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item",
        "Layer: Hot path optimization. Flag repeated time.Now calls inside per-item loops where one timestamp would suffice."
    ),
    perf_layer_rule!(
        "go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func",
        "Layer: Hot path optimization. Flag allocations inside sort comparison or hashing callbacks."
    ),
    perf_layer_rule!(
        "go_perf_layer_hot_path_optimization_reflection_on_hot_path",
        "Layer: Hot path optimization. Flag reflect-based field access in hot loops where typed access is available."
    ),
    perf_layer_rule!(
        "go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop",
        "Layer: Hot path optimization. Flag defer used for simple unlock/close work inside very hot loops."
    ),
    // Lazy loading
    perf_layer_rule!(
        "go_perf_layer_lazy_loading_eager_load_optional_config",
        "Layer: Lazy loading. Flag optional configuration or metadata loaded at startup before any feature uses it."
    ),
    perf_layer_rule!(
        "go_perf_layer_lazy_loading_eager_connect_to_all_backends",
        "Layer: Lazy loading. Flag services connecting to every optional backend at startup instead of lazy initialization."
    ),
    perf_layer_rule!(
        "go_perf_layer_lazy_loading_lazy_once_hides_slow_first_request",
        "Layer: Lazy loading. Flag sync.Once lazy initialization on request paths without warmup or latency accounting."
    ),
    perf_layer_rule!(
        "go_perf_layer_lazy_loading_eager_template_parse_for_unused_routes",
        "Layer: Lazy loading. Flag parsing templates for routes that may never be served in the current process mode."
    ),
    perf_layer_rule!(
        "go_perf_layer_lazy_loading_eager_metric_label_cardinality_build",
        "Layer: Lazy loading. Flag eager construction of high-cardinality metric labels before the metric is emitted."
    ),
    // Resource pooling
    perf_layer_rule!(
        "go_perf_layer_resource_pooling_db_pool_created_per_repository",
        "Layer: Resource pooling. Flag database pools constructed per repository instance instead of shared process ownership."
    ),
    perf_layer_rule!(
        "go_perf_layer_resource_pooling_worker_pool_without_shutdown_backpressure",
        "Layer: Resource pooling. Flag worker pools that accept unlimited jobs without bounded queues or shutdown draining."
    ),
    perf_layer_rule!(
        "go_perf_layer_resource_pooling_buffer_pool_without_max_capacity",
        "Layer: Resource pooling. Flag buffer pools that return arbitrarily large buffers and retain memory after spikes."
    ),
    perf_layer_rule!(
        "go_perf_layer_resource_pooling_http_transport_per_service_method",
        "Layer: Resource pooling. Flag new transports per service method instead of shared clients per upstream."
    ),
    perf_layer_rule!(
        "go_perf_layer_resource_pooling_rate_limiter_per_request",
        "Layer: Resource pooling. Flag rate limiter construction inside request handlers instead of per identity or dependency."
    ),
    // Framework-specific performance
    perf_layer_rule!(
        "go_perf_layer_framework_performance_gin_context_copied_for_sync_path",
        "Layer: Framework-specific performance. Flag Gin context Copy calls where work remains synchronous and request-scoped."
    ),
    perf_layer_rule!(
        "go_perf_layer_framework_performance_gorm_preload_all_associations",
        "Layer: Framework-specific performance. Flag GORM Preload of broad association sets on list endpoints."
    ),
    perf_layer_rule!(
        "go_perf_layer_framework_performance_sqlx_select_slice_unbounded",
        "Layer: Framework-specific performance. Flag sqlx Select into slices without LIMIT or streaming for large tables."
    ),
    perf_layer_rule!(
        "go_perf_layer_framework_performance_grpc_metadata_parsed_repeatedly",
        "Layer: Framework-specific performance. Flag repeated metadata extraction/parsing in gRPC interceptors and handlers."
    ),
    perf_layer_rule!(
        "go_perf_layer_framework_performance_template_execute_to_string_then_write",
        "Layer: Framework-specific performance. Flag template rendering to strings before writing to the response writer."
    ),
    // Profiling and benchmarking
    perf_layer_rule!(
        "go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report",
        "Layer: Profiling and benchmarking. Flag benchmarks for allocation-sensitive code that omit b.ReportAllocs."
    ),
    perf_layer_rule!(
        "go_perf_layer_profiling_benchmarking_benchmark_includes_setup_in_loop",
        "Layer: Profiling and benchmarking. Flag Go benchmarks that perform setup inside the timed loop without ResetTimer."
    ),
    perf_layer_rule!(
        "go_perf_layer_profiling_benchmarking_microbenchmark_dead_code_eliminated",
        "Layer: Profiling and benchmarking. Flag benchmarks that do not consume results and may let the compiler remove work."
    ),
    perf_layer_rule!(
        "go_perf_layer_profiling_benchmarking_pprof_endpoint_enabled_without_sampling_plan",
        "Layer: Profiling and benchmarking. Flag always-on profiling endpoints in production services without access or sampling controls."
    ),
    perf_layer_rule!(
        "go_perf_layer_profiling_benchmarking_optimization_comment_without_benchmark",
        "Layer: Profiling and benchmarking. Flag performance-claim comments that lack benchmark, profile, or trace evidence."
    ),
];
