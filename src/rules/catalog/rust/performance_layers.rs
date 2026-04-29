use super::{
    RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus, bindings,
};

macro_rules! perf_layer_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Rust,
            family: "performance",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: bindings::RUST_PERFORMANCE,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    // Algorithmic complexity
    perf_layer_rule!(
        "rust_perf_layer_algorithmic_complexity_nested_vec_scan_without_index",
        "Layer: Algorithmic complexity. Flag nested Vec scans that should build a HashMap, BTreeMap, or index before joining large collections."
    ),
    perf_layer_rule!(
        "rust_perf_layer_algorithmic_complexity_sort_for_top_k",
        "Layer: Algorithmic complexity. Flag full Vec sorts used only to keep top-k elements where BinaryHeap or select_nth_unstable would bound work."
    ),
    perf_layer_rule!(
        "rust_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen",
        "Layer: Algorithmic complexity. Flag recursive graph walks that do not track visited nodes for cyclic or shared inputs."
    ),
    perf_layer_rule!(
        "rust_perf_layer_algorithmic_complexity_quadratic_string_or_vec_growth",
        "Layer: Algorithmic complexity. Flag algorithms that repeatedly grow and rescan accumulated String or Vec state."
    ),
    perf_layer_rule!(
        "rust_perf_layer_algorithmic_complexity_iterator_chain_recomputes_expensive_predicate",
        "Layer: Algorithmic complexity. Flag iterator chains that recompute an expensive predicate instead of caching per item."
    ),
    // Data structure choice
    perf_layer_rule!(
        "rust_perf_layer_data_structure_choice_hashmap_for_tiny_static_set",
        "Layer: Data structure choice. Flag HashMap use for tiny static sets where match, arrays, or phf-like tables may avoid hashing cost."
    ),
    perf_layer_rule!(
        "rust_perf_layer_data_structure_choice_vec_remove_zero_queue",
        "Layer: Data structure choice. Flag Vec::remove(0) queue behavior where VecDeque fits front removal."
    ),
    perf_layer_rule!(
        "rust_perf_layer_data_structure_choice_boxed_trait_objects_in_hot_collection",
        "Layer: Data structure choice. Flag Vec<Box<dyn Trait>> in hot homogeneous collections where enums or generics can avoid dispatch and pointer chasing."
    ),
    perf_layer_rule!(
        "rust_perf_layer_data_structure_choice_string_keys_for_enum_domain",
        "Layer: Data structure choice. Flag String keys for finite domains where enums or interned IDs avoid allocation and hashing."
    ),
    perf_layer_rule!(
        "rust_perf_layer_data_structure_choice_btreemap_for_unordered_hot_lookup",
        "Layer: Data structure choice. Flag BTreeMap lookups in unordered hot paths where HashMap or indexed Vec may be cheaper."
    ),
    // Memory allocation
    perf_layer_rule!(
        "rust_perf_layer_memory_allocation_vec_push_without_capacity",
        "Layer: Memory allocation. Flag Vec pushes in builders where an obvious capacity is known."
    ),
    perf_layer_rule!(
        "rust_perf_layer_memory_allocation_string_reallocated_per_record",
        "Layer: Memory allocation. Flag String allocation per record where a reusable buffer can be cleared safely."
    ),
    perf_layer_rule!(
        "rust_perf_layer_memory_allocation_clone_large_value_for_readonly_use",
        "Layer: Memory allocation. Flag Clone of large values immediately before read-only operations."
    ),
    perf_layer_rule!(
        "rust_perf_layer_memory_allocation_collect_vec_only_to_iterate_once",
        "Layer: Memory allocation. Flag collect::<Vec<_>>() used only to iterate once into another consumer."
    ),
    perf_layer_rule!(
        "rust_perf_layer_memory_allocation_format_allocates_for_writer",
        "Layer: Memory allocation. Flag format! used before write!/push_str where direct writing avoids temporary allocation."
    ),
    // Garbage collection and cleanup
    perf_layer_rule!(
        "rust_perf_layer_garbage_collection_cleanup_large_vec_clear_retains_spike_capacity",
        "Layer: Garbage collection and cleanup. Flag long-lived Vec buffers cleared after spikes without capacity trimming or replacement policy."
    ),
    perf_layer_rule!(
        "rust_perf_layer_garbage_collection_cleanup_drop_impl_does_io_or_blocking_work",
        "Layer: Garbage collection and cleanup. Flag Drop implementations that perform I/O, locking, or other blocking cleanup."
    ),
    perf_layer_rule!(
        "rust_perf_layer_garbage_collection_cleanup_arc_cycle_without_weak_break",
        "Layer: Garbage collection and cleanup. Flag Arc graphs that appear cyclic without Weak ownership edges."
    ),
    perf_layer_rule!(
        "rust_perf_layer_garbage_collection_cleanup_bufwriter_not_flushed_at_owner_boundary",
        "Layer: Garbage collection and cleanup. Flag buffered writers whose flush ownership is unclear at the boundary."
    ),
    perf_layer_rule!(
        "rust_perf_layer_garbage_collection_cleanup_leaked_box_for_runtime_singleton",
        "Layer: Garbage collection and cleanup. Flag Box::leak used for runtime singletons where OnceLock or LazyLock would express ownership better."
    ),
    // String handling
    perf_layer_rule!(
        "rust_perf_layer_string_handling_format_in_loop_for_simple_append",
        "Layer: String handling. Flag format! in loops for simple string appends where write! or push_str can reuse buffers."
    ),
    perf_layer_rule!(
        "rust_perf_layer_string_handling_regex_new_in_function_path",
        "Layer: String handling. Flag Regex::new in repeated functions or loops instead of LazyLock or once_cell."
    ),
    perf_layer_rule!(
        "rust_perf_layer_string_handling_to_lowercase_compare",
        "Layer: String handling. Flag to_lowercase/to_uppercase comparisons where eq_ignore_ascii_case or normalized keys avoid allocation."
    ),
    perf_layer_rule!(
        "rust_perf_layer_string_handling_string_from_utf8_roundtrip",
        "Layer: String handling. Flag String/Vec<u8> roundtrips without transformation."
    ),
    perf_layer_rule!(
        "rust_perf_layer_string_handling_split_collect_for_first_segment",
        "Layer: String handling. Flag split().collect::<Vec<_>>() used only to access one segment."
    ),
    // Collection iteration
    perf_layer_rule!(
        "rust_perf_layer_collection_iteration_multiple_passes_over_same_slice",
        "Layer: Collection iteration. Flag independent passes over the same slice where aggregation can be fused."
    ),
    perf_layer_rule!(
        "rust_perf_layer_collection_iteration_iter_cloned_before_filter",
        "Layer: Collection iteration. Flag iter().cloned() before filters that could borrow until ownership is needed."
    ),
    perf_layer_rule!(
        "rust_perf_layer_collection_iteration_collect_for_len_or_is_empty",
        "Layer: Collection iteration. Flag iterator collection used only to call len or check emptiness."
    ),
    perf_layer_rule!(
        "rust_perf_layer_collection_iteration_manual_index_loop_over_slice",
        "Layer: Collection iteration. Flag manual index loops over slices where iterators or chunks avoid repeated indexing."
    ),
    perf_layer_rule!(
        "rust_perf_layer_collection_iteration_hashset_rebuilt_per_lookup_batch",
        "Layer: Collection iteration. Flag HashSet rebuilt inside loops instead of once per batch."
    ),
    // Async and concurrency
    perf_layer_rule!(
        "rust_perf_layer_async_concurrency_tokio_spawn_per_item_unbounded",
        "Layer: Async and concurrency. Flag unbounded tokio::spawn per item without JoinSet, semaphore, or bounded queue ownership."
    ),
    perf_layer_rule!(
        "rust_perf_layer_async_concurrency_blocking_mutex_in_async_path",
        "Layer: Async and concurrency. Flag std::sync::Mutex use in async paths where blocking can stall executor workers."
    ),
    perf_layer_rule!(
        "rust_perf_layer_async_concurrency_lock_guard_held_across_slow_call",
        "Layer: Async and concurrency. Flag lock guards held during network, disk, database, or expensive compute work."
    ),
    perf_layer_rule!(
        "rust_perf_layer_async_concurrency_timeout_created_for_inner_retry",
        "Layer: Async and concurrency. Flag timeout futures created inside tight retry loops where an outer deadline would bound the operation."
    ),
    perf_layer_rule!(
        "rust_perf_layer_async_concurrency_busy_poll_future_loop",
        "Layer: Async and concurrency. Flag async loops that poll with try_recv or zero sleeps instead of awaiting notifications."
    ),
    // I/O operations
    perf_layer_rule!(
        "rust_perf_layer_io_operations_read_to_string_for_line_processing",
        "Layer: I/O operations. Flag read_to_string/read_to_end before sequential line or chunk processing."
    ),
    perf_layer_rule!(
        "rust_perf_layer_io_operations_unbuffered_writes_in_loop",
        "Layer: I/O operations. Flag repeated file or socket writes without BufWriter or batching."
    ),
    perf_layer_rule!(
        "rust_perf_layer_io_operations_metadata_before_open_without_need",
        "Layer: I/O operations. Flag fs::metadata calls immediately followed by open/read without using metadata meaningfully."
    ),
    perf_layer_rule!(
        "rust_perf_layer_io_operations_tempfile_for_stream_transform",
        "Layer: I/O operations. Flag temporary files used only to pass data between streaming transformations."
    ),
    perf_layer_rule!(
        "rust_perf_layer_io_operations_command_spawn_per_item",
        "Layer: I/O operations. Flag process spawning inside item loops where batching or long-lived workers may fit."
    ),
    // Database access
    perf_layer_rule!(
        "rust_perf_layer_database_access_sql_query_inside_loop",
        "Layer: Database access. Flag SQL queries inside loops where batching or joins can collapse round trips."
    ),
    perf_layer_rule!(
        "rust_perf_layer_database_access_select_star_for_typed_projection",
        "Layer: Database access. Flag SELECT * when typed row mapping uses only a subset of columns."
    ),
    perf_layer_rule!(
        "rust_perf_layer_database_access_transaction_per_row_bulk_insert",
        "Layer: Database access. Flag bulk inserts that begin and commit a transaction for every row."
    ),
    perf_layer_rule!(
        "rust_perf_layer_database_access_row_to_hashmap_decode_hot_path",
        "Layer: Database access. Flag database row decoding into HashMap<String, Value> on hot typed paths."
    ),
    perf_layer_rule!(
        "rust_perf_layer_database_access_count_before_page_fetch_each_request",
        "Layer: Database access. Flag count queries paired with every paginated fetch without caching or product need."
    ),
    // Network calls
    perf_layer_rule!(
        "rust_perf_layer_network_calls_reqwest_client_created_per_call",
        "Layer: Network calls. Flag reqwest::Client construction per call instead of reuse per upstream."
    ),
    perf_layer_rule!(
        "rust_perf_layer_network_calls_hyper_connector_rebuilt_per_request",
        "Layer: Network calls. Flag HTTP connector or TLS connector construction per request."
    ),
    perf_layer_rule!(
        "rust_perf_layer_network_calls_retry_without_backoff_jitter",
        "Layer: Network calls. Flag network retry loops without backoff and jitter."
    ),
    perf_layer_rule!(
        "rust_perf_layer_network_calls_dns_resolution_in_request_path",
        "Layer: Network calls. Flag DNS or endpoint resolution in request paths without cache or resolver ownership."
    ),
    perf_layer_rule!(
        "rust_perf_layer_network_calls_tls_config_loaded_per_request",
        "Layer: Network calls. Flag rustls/native-tls configuration loading per request instead of shared immutable state."
    ),
    // Caching
    perf_layer_rule!(
        "rust_perf_layer_caching_cache_key_allocates_string_hot_path",
        "Layer: Caching. Flag cache keys built as allocated Strings in hot paths where borrowed or structured keys fit."
    ),
    perf_layer_rule!(
        "rust_perf_layer_caching_unbounded_hashmap_cache",
        "Layer: Caching. Flag process-level HashMap caches without size, TTL, or eviction policy."
    ),
    perf_layer_rule!(
        "rust_perf_layer_caching_cache_miss_duplicate_async_work",
        "Layer: Caching. Flag cache miss paths that allow concurrent duplicate recomputation for the same key."
    ),
    perf_layer_rule!(
        "rust_perf_layer_caching_cached_json_string_deserialized_each_use",
        "Layer: Caching. Flag caches storing JSON strings only to deserialize before every read."
    ),
    perf_layer_rule!(
        "rust_perf_layer_caching_once_lock_lazy_cost_on_first_request",
        "Layer: Caching. Flag OnceLock/LazyLock initialization that puts large first-use cost on a user request without warmup."
    ),
    // Serialization and deserialization
    perf_layer_rule!(
        "rust_perf_layer_serialization_serde_json_to_value_hot_path",
        "Layer: Serialization and deserialization. Flag serde_json::Value decoding in hot typed paths where concrete structs avoid dynamic lookup."
    ),
    perf_layer_rule!(
        "rust_perf_layer_serialization_serialize_for_hash_or_eq",
        "Layer: Serialization and deserialization. Flag serialization used only to hash, compare, or deduplicate values."
    ),
    perf_layer_rule!(
        "rust_perf_layer_serialization_base64_roundtrip_same_process",
        "Layer: Serialization and deserialization. Flag base64 encode/decode roundtrips inside one process boundary."
    ),
    perf_layer_rule!(
        "rust_perf_layer_serialization_bincode_or_postcard_for_tiny_messages",
        "Layer: Serialization and deserialization. Flag binary serialization setup for tiny in-process messages where direct structs or slices would do."
    ),
    perf_layer_rule!(
        "rust_perf_layer_serialization_gzip_encoder_per_tiny_payload",
        "Layer: Serialization and deserialization. Flag compression writers created for tiny payloads where CPU overhead dominates."
    ),
    // Logging overhead
    perf_layer_rule!(
        "rust_perf_layer_logging_overhead_log_fields_allocated_before_enabled",
        "Layer: Logging overhead. Flag expensive tracing/log fields allocated before level or subscriber filters can skip them."
    ),
    perf_layer_rule!(
        "rust_perf_layer_logging_overhead_per_item_info_span_in_batch",
        "Layer: Logging overhead. Flag tracing spans or info logs emitted per record in large batch loops."
    ),
    perf_layer_rule!(
        "rust_perf_layer_logging_overhead_debug_format_large_value_unconditional",
        "Layer: Logging overhead. Flag debug formatting before log level checks."
    ),
    perf_layer_rule!(
        "rust_perf_layer_logging_overhead_span_created_inside_tight_loop",
        "Layer: Logging overhead. Flag tracing span construction in very tight loops without sampling or aggregation."
    ),
    perf_layer_rule!(
        "rust_perf_layer_logging_overhead_payload_serialized_for_disabled_trace",
        "Layer: Logging overhead. Flag payload serialization for tracing even when tracing is disabled."
    ),
    // Error handling cost
    perf_layer_rule!(
        "rust_perf_layer_error_handling_cost_anyhow_context_in_hot_loop",
        "Layer: Error handling cost. Flag anyhow/context formatting inside hot loops where one boundary context would suffice."
    ),
    perf_layer_rule!(
        "rust_perf_layer_error_handling_cost_error_enum_allocates_string_success_path",
        "Layer: Error handling cost. Flag error strings built before the failing branch is known."
    ),
    perf_layer_rule!(
        "rust_perf_layer_error_handling_cost_panic_catch_unwind_for_control_flow",
        "Layer: Error handling cost. Flag catch_unwind used for expected branch control in performance-sensitive code."
    ),
    perf_layer_rule!(
        "rust_perf_layer_error_handling_cost_clone_error_context_large_payload",
        "Layer: Error handling cost. Flag error context cloning large payloads only for diagnostic strings."
    ),
    perf_layer_rule!(
        "rust_perf_layer_error_handling_cost_vec_errors_allocated_before_error",
        "Layer: Error handling cost. Flag error aggregation Vec allocated on success-heavy paths before any error exists."
    ),
    // Build and runtime configuration
    perf_layer_rule!(
        "rust_perf_layer_runtime_configuration_debug_assertions_expected_in_bench",
        "Layer: Build and runtime configuration. Flag benchmark or performance docs that mix debug-assertion builds with release numbers."
    ),
    perf_layer_rule!(
        "rust_perf_layer_runtime_configuration_rayon_threads_set_in_library",
        "Layer: Build and runtime configuration. Flag library code configuring global Rayon thread pools and overriding host policy."
    ),
    perf_layer_rule!(
        "rust_perf_layer_runtime_configuration_env_parsed_per_operation",
        "Layer: Build and runtime configuration. Flag environment parsing and type conversion per operation instead of startup config load."
    ),
    perf_layer_rule!(
        "rust_perf_layer_runtime_configuration_feature_flags_loaded_per_request",
        "Layer: Build and runtime configuration. Flag feature flag or config files loaded in request paths."
    ),
    perf_layer_rule!(
        "rust_perf_layer_runtime_configuration_lto_codegen_claim_without_profile",
        "Layer: Build and runtime configuration. Flag Cargo profile performance claims without benchmark notes or profile evidence."
    ),
    // Hot path optimization
    perf_layer_rule!(
        "rust_perf_layer_hot_path_optimization_schema_parsed_inside_handler",
        "Layer: Hot path optimization. Flag schemas, templates, regex sets, or expressions parsed per request."
    ),
    perf_layer_rule!(
        "rust_perf_layer_hot_path_optimization_instant_now_called_many_times_per_item",
        "Layer: Hot path optimization. Flag repeated Instant::now calls per item where one timestamp or measurement scope would suffice."
    ),
    perf_layer_rule!(
        "rust_perf_layer_hot_path_optimization_allocation_inside_sort_comparator",
        "Layer: Hot path optimization. Flag allocation, parsing, or formatting inside sort comparator closures."
    ),
    perf_layer_rule!(
        "rust_perf_layer_hot_path_optimization_dynamic_dispatch_in_deep_loop",
        "Layer: Hot path optimization. Flag trait-object dynamic dispatch inside deep loops where generics or enum dispatch may fit."
    ),
    perf_layer_rule!(
        "rust_perf_layer_hot_path_optimization_bounds_checks_from_repeated_indexing",
        "Layer: Hot path optimization. Flag repeated indexed access patterns where iterators, windows, or chunks can reduce bounds-check pressure."
    ),
    // Lazy loading
    perf_layer_rule!(
        "rust_perf_layer_lazy_loading_eager_load_optional_assets",
        "Layer: Lazy loading. Flag optional assets, dictionaries, or models loaded at startup before any feature uses them."
    ),
    perf_layer_rule!(
        "rust_perf_layer_lazy_loading_lazy_static_hides_tail_latency",
        "Layer: Lazy loading. Flag lazy statics initialized on first request without warmup or latency accounting."
    ),
    perf_layer_rule!(
        "rust_perf_layer_lazy_loading_eager_connect_all_upstreams",
        "Layer: Lazy loading. Flag connecting to every optional upstream during startup instead of lazy or feature-gated initialization."
    ),
    perf_layer_rule!(
        "rust_perf_layer_lazy_loading_eager_compile_unused_templates",
        "Layer: Lazy loading. Flag compiling templates or route metadata for disabled features."
    ),
    perf_layer_rule!(
        "rust_perf_layer_lazy_loading_optional_crate_init_on_default_path",
        "Layer: Lazy loading. Flag optional crate initialization on the default path when feature-gated initialization would avoid cost."
    ),
    // Resource pooling
    perf_layer_rule!(
        "rust_perf_layer_resource_pooling_pool_created_per_request",
        "Layer: Resource pooling. Flag database, HTTP, or worker pools created per request instead of process or dependency ownership."
    ),
    perf_layer_rule!(
        "rust_perf_layer_resource_pooling_buffer_pool_returns_oversized_vec",
        "Layer: Resource pooling. Flag reusable Vec/String pools that retain oversized buffers after spikes."
    ),
    perf_layer_rule!(
        "rust_perf_layer_resource_pooling_semaphore_created_per_operation",
        "Layer: Resource pooling. Flag Semaphore or rate limiter construction per operation instead of shared dependency ownership."
    ),
    perf_layer_rule!(
        "rust_perf_layer_resource_pooling_threadpool_without_shutdown_owner",
        "Layer: Resource pooling. Flag custom thread pools without explicit shutdown/drain ownership."
    ),
    perf_layer_rule!(
        "rust_perf_layer_resource_pooling_connection_pool_without_limits",
        "Layer: Resource pooling. Flag connection pools missing max size, idle, or timeout controls."
    ),
    // Framework-specific performance
    perf_layer_rule!(
        "rust_perf_layer_framework_performance_axum_extension_clones_heavy_state",
        "Layer: Framework-specific performance. Flag Axum handler state extraction that clones heavy state instead of cheap Arc handles."
    ),
    perf_layer_rule!(
        "rust_perf_layer_framework_performance_actix_blocking_pool_for_light_work",
        "Layer: Framework-specific performance. Flag actix web::block for light synchronous work where scheduling overhead can dominate."
    ),
    perf_layer_rule!(
        "rust_perf_layer_framework_performance_sqlx_fetch_all_unbounded",
        "Layer: Framework-specific performance. Flag sqlx fetch_all on unbounded queries where streaming or limits would control memory."
    ),
    perf_layer_rule!(
        "rust_perf_layer_framework_performance_tonic_metadata_parsed_repeatedly",
        "Layer: Framework-specific performance. Flag repeated tonic metadata parsing across interceptors and handlers."
    ),
    perf_layer_rule!(
        "rust_perf_layer_framework_performance_askama_render_to_string_then_body",
        "Layer: Framework-specific performance. Flag template rendering to String before writing to streaming response bodies."
    ),
    // Profiling and benchmarking
    perf_layer_rule!(
        "rust_perf_layer_profiling_benchmarking_criterion_benchmark_includes_setup",
        "Layer: Profiling and benchmarking. Flag Criterion benchmarks that include setup in the measured closure instead of setup/measurement separation."
    ),
    perf_layer_rule!(
        "rust_perf_layer_profiling_benchmarking_benchmark_missing_black_box",
        "Layer: Profiling and benchmarking. Flag microbenchmarks that do not use black_box or otherwise consume results."
    ),
    perf_layer_rule!(
        "rust_perf_layer_profiling_benchmarking_alloc_sensitive_benchmark_without_alloc_counts",
        "Layer: Profiling and benchmarking. Flag allocation-sensitive benchmarks that omit allocation counting or heap profiling."
    ),
    perf_layer_rule!(
        "rust_perf_layer_profiling_benchmarking_optimization_comment_without_measurement",
        "Layer: Profiling and benchmarking. Flag optimization comments without benchmark, profile, or trace evidence."
    ),
    perf_layer_rule!(
        "rust_perf_layer_profiling_benchmarking_load_test_payload_too_small",
        "Layer: Profiling and benchmarking. Flag performance tests using tiny payloads despite targeting large-input behavior."
    ),
];
