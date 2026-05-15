use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

macro_rules! perf_layer_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "performance",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: super::bindings::PYTHON_PERFORMANCE,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    // Algorithmic complexity
    perf_layer_rule!(
        "python_perf_layer_algorithmic_complexity_nested_membership_scan_without_set",
        "Layer: Algorithmic complexity. Flag nested membership checks over lists where a set or dict index should be built once."
    ),
    perf_layer_rule!(
        "python_perf_layer_algorithmic_complexity_repeated_dataframe_filter_chain",
        "Layer: Algorithmic complexity. Flag repeated pandas filters over the same frame where one boolean mask can combine predicates."
    ),
    perf_layer_rule!(
        "python_perf_layer_algorithmic_complexity_full_sort_for_top_n",
        "Layer: Algorithmic complexity. Flag sorted(...)[0:n] or DataFrame sort_values().head() where nlargest/nsmallest or heap selection would bound work."
    ),
    perf_layer_rule!(
        "python_perf_layer_algorithmic_complexity_recursive_walk_without_iterative_guard",
        "Layer: Algorithmic complexity. Flag recursive traversals over untrusted depth without iterative fallback or visited tracking."
    ),
    perf_layer_rule!(
        "python_perf_layer_algorithmic_complexity_cartesian_product_materialized_before_filter",
        "Layer: Algorithmic complexity. Flag itertools product or nested loops materialized before cheap filters narrow the pairs."
    ),
    // Data structure choice
    perf_layer_rule!(
        "python_perf_layer_data_structure_choice_list_as_fifo_queue",
        "Layer: Data structure choice. Flag list pop(0) or insert(0, ...) queue usage where collections.deque fits the access pattern."
    ),
    perf_layer_rule!(
        "python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records",
        "Layer: Data structure choice. Flag dict-of-dicts records in hot loops where dataclass, tuple, or typed containers reduce lookup cost."
    ),
    perf_layer_rule!(
        "python_perf_layer_data_structure_choice_set_rebuilt_for_each_lookup_batch",
        "Layer: Data structure choice. Flag sets rebuilt inside loops instead of once per lookup batch."
    ),
    perf_layer_rule!(
        "python_perf_layer_data_structure_choice_pandas_object_dtype_for_numeric_hot_columns",
        "Layer: Data structure choice. Flag numeric pandas columns kept as object dtype on transformation-heavy paths."
    ),
    perf_layer_rule!(
        "python_perf_layer_data_structure_choice_heapq_reimplemented_with_sorted_list",
        "Layer: Data structure choice. Flag sorted-list priority queues where heapq avoids repeated full ordering."
    ),
    // Memory allocation
    perf_layer_rule!(
        "python_perf_layer_memory_allocation_list_append_without_generator_stream",
        "Layer: Memory allocation. Flag large intermediate lists built only to feed another iterator-compatible consumer."
    ),
    perf_layer_rule!(
        "python_perf_layer_memory_allocation_deepcopy_before_readonly_transform",
        "Layer: Memory allocation. Flag copy.deepcopy of large objects before operations that only read them."
    ),
    perf_layer_rule!(
        "python_perf_layer_memory_allocation_dataclass_asdict_in_hot_path",
        "Layer: Memory allocation. Flag dataclasses.asdict calls in hot paths when shallow field access would suffice."
    ),
    perf_layer_rule!(
        "python_perf_layer_memory_allocation_temporary_dataframe_created_per_row",
        "Layer: Memory allocation. Flag DataFrame construction per row or per record instead of vectorized batch construction."
    ),
    perf_layer_rule!(
        "python_perf_layer_memory_allocation_closure_or_lambda_allocated_in_inner_loop",
        "Layer: Memory allocation. Flag lambdas or closures allocated inside tight loops when a named function or hoisted callable works."
    ),
    // Garbage collection and cleanup
    perf_layer_rule!(
        "python_perf_layer_garbage_collection_cleanup_large_list_cleared_but_retained",
        "Layer: Garbage collection and cleanup. Flag long-lived lists cleared after spikes while still retaining oversized capacity-sensitive state."
    ),
    perf_layer_rule!(
        "python_perf_layer_garbage_collection_cleanup_file_handle_closed_by_gc",
        "Layer: Garbage collection and cleanup. Flag file or socket resources relying on garbage collection instead of context-managed close."
    ),
    perf_layer_rule!(
        "python_perf_layer_garbage_collection_cleanup_temporary_directory_not_cleaned_promptly",
        "Layer: Garbage collection and cleanup. Flag TemporaryDirectory or temp files held across long workflows after their last use."
    ),
    perf_layer_rule!(
        "python_perf_layer_garbage_collection_cleanup_lru_cache_on_unbounded_argument_space",
        "Layer: Garbage collection and cleanup. Flag lru_cache usage without maxsize on functions fed user-controlled or high-cardinality inputs."
    ),
    perf_layer_rule!(
        "python_perf_layer_garbage_collection_cleanup_cycle_heavy_objects_without_break",
        "Layer: Garbage collection and cleanup. Flag cyclic object graphs that keep large buffers alive after request or batch completion."
    ),
    // String handling
    perf_layer_rule!(
        "python_perf_layer_string_handling_plus_equals_string_in_loop",
        "Layer: String handling. Flag repeated += string growth in loops where join or StringIO avoids quadratic copying."
    ),
    perf_layer_rule!(
        "python_perf_layer_string_handling_regex_compiled_per_call",
        "Layer: String handling. Flag re.compile calls in repeated functions, handlers, or loops instead of module-level compiled patterns."
    ),
    perf_layer_rule!(
        "python_perf_layer_string_handling_lowercase_compare_allocates",
        "Layer: String handling. Flag lower()/upper() equality checks where casefold once or normalized keys avoid repeated allocation."
    ),
    perf_layer_rule!(
        "python_perf_layer_string_handling_json_string_concat_manual_build",
        "Layer: String handling. Flag manual JSON string assembly in loops instead of streaming encoders or structured dumps."
    ),
    perf_layer_rule!(
        "python_perf_layer_string_handling_bytes_decode_encode_roundtrip",
        "Layer: String handling. Flag bytes-to-text-to-bytes roundtrips without transformation."
    ),
    // Collection iteration
    perf_layer_rule!(
        "python_perf_layer_collection_iteration_multiple_passes_over_same_iterable",
        "Layer: Collection iteration. Flag independent loops over the same collection where aggregation can be fused."
    ),
    perf_layer_rule!(
        "python_perf_layer_collection_iteration_list_comprehension_only_for_side_effect",
        "Layer: Collection iteration. Flag list comprehensions used only for side effects, forcing unnecessary list allocation."
    ),
    perf_layer_rule!(
        "python_perf_layer_collection_iteration_enumerate_list_materialized",
        "Layer: Collection iteration. Flag list(enumerate(...)) materialization before immediate iteration."
    ),
    perf_layer_rule!(
        "python_perf_layer_collection_iteration_generator_materialized_for_truthiness",
        "Layer: Collection iteration. Flag list(generator) or tuple(generator) used only for truthiness checks."
    ),
    perf_layer_rule!(
        "python_perf_layer_collection_iteration_pandas_iterrows_for_numeric_transform",
        "Layer: Collection iteration. Flag pandas iterrows loops for numeric transformations that can be vectorized or applied in batches."
    ),
    // Async and concurrency
    perf_layer_rule!(
        "python_perf_layer_async_concurrency_asyncio_task_per_item_unbounded",
        "Layer: Async and concurrency. Flag unbounded create_task fanout without a semaphore, queue, or batch limit."
    ),
    perf_layer_rule!(
        "python_perf_layer_async_concurrency_blocking_requests_in_async_route",
        "Layer: Async and concurrency. Flag requests.* calls inside async routes or coroutines instead of async clients or executor boundaries."
    ),
    perf_layer_rule!(
        "python_perf_layer_async_concurrency_threadpool_created_per_request",
        "Layer: Async and concurrency. Flag ThreadPoolExecutor or ProcessPoolExecutor construction per request or call."
    ),
    perf_layer_rule!(
        "python_perf_layer_async_concurrency_lock_held_during_await_or_io",
        "Layer: Async and concurrency. Flag locks held while awaiting, doing network I/O, or touching disk."
    ),
    perf_layer_rule!(
        "python_perf_layer_async_concurrency_busy_async_poll_sleep_zero",
        "Layer: Async and concurrency. Flag async polling loops that sleep(0) or spin instead of awaiting a real event."
    ),
    // I/O operations
    perf_layer_rule!(
        "python_perf_layer_io_operations_read_entire_file_for_line_processing",
        "Layer: I/O operations. Flag read()/readlines() on files that are processed line by line."
    ),
    perf_layer_rule!(
        "python_perf_layer_io_operations_small_file_writes_without_buffer",
        "Layer: I/O operations. Flag many tiny writes where writelines, join, or buffering would reduce syscalls."
    ),
    perf_layer_rule!(
        "python_perf_layer_io_operations_path_exists_before_open_race",
        "Layer: I/O operations. Flag exists/stat checks immediately followed by open when exception handling can avoid duplicate syscalls."
    ),
    perf_layer_rule!(
        "python_perf_layer_io_operations_temporary_file_for_bytes_transform",
        "Layer: I/O operations. Flag temporary files used only for pure bytes transformations that can stream in memory."
    ),
    perf_layer_rule!(
        "python_perf_layer_io_operations_subprocess_per_item",
        "Layer: I/O operations. Flag subprocess launches inside item loops where batching or a long-lived worker should be considered."
    ),
    // Database access
    perf_layer_rule!(
        "python_perf_layer_database_access_orm_query_inside_loop",
        "Layer: Database access. Flag ORM queries inside loops where select_related, prefetch_related, or bulk queries can collapse round trips."
    ),
    perf_layer_rule!(
        "python_perf_layer_database_access_select_all_columns_for_serializer_subset",
        "Layer: Database access. Flag ORM or SQL queries loading all columns when serializers use a small subset."
    ),
    perf_layer_rule!(
        "python_perf_layer_database_access_count_then_fetch_every_page",
        "Layer: Database access. Flag count queries paired with every paginated fetch without product need or caching."
    ),
    perf_layer_rule!(
        "python_perf_layer_database_access_row_by_row_bulk_insert",
        "Layer: Database access. Flag bulk inserts implemented one row at a time instead of executemany or ORM bulk APIs."
    ),
    perf_layer_rule!(
        "python_perf_layer_database_access_dataframe_to_sql_row_loop",
        "Layer: Database access. Flag DataFrame writes to SQL through row loops instead of chunked to_sql or bulk copy."
    ),
    // Network calls
    perf_layer_rule!(
        "python_perf_layer_network_calls_http_session_created_per_call",
        "Layer: Network calls. Flag requests.Session or client objects created per call instead of reused per upstream."
    ),
    perf_layer_rule!(
        "python_perf_layer_network_calls_dns_or_url_parse_repeated_per_request",
        "Layer: Network calls. Flag repeated DNS, URL parsing, or endpoint normalization for stable upstreams."
    ),
    perf_layer_rule!(
        "python_perf_layer_network_calls_retry_without_backoff_or_jitter",
        "Layer: Network calls. Flag network retries without backoff and jitter, which amplifies latency and load."
    ),
    perf_layer_rule!(
        "python_perf_layer_network_calls_large_response_downloaded_before_status_check",
        "Layer: Network calls. Flag response bodies read before checking status, headers, or content length guards."
    ),
    perf_layer_rule!(
        "python_perf_layer_network_calls_tls_context_built_per_request",
        "Layer: Network calls. Flag SSLContext or certificate bundle loading per request instead of process-level reuse."
    ),
    // Caching
    perf_layer_rule!(
        "python_perf_layer_caching_cache_key_uses_json_dumps_in_hot_path",
        "Layer: Caching. Flag cache keys built with json.dumps in hot paths when tuple or stable string keys would work."
    ),
    perf_layer_rule!(
        "python_perf_layer_caching_unbounded_dict_cache",
        "Layer: Caching. Flag module-level dict caches without max size, TTL, or eviction."
    ),
    perf_layer_rule!(
        "python_perf_layer_caching_cache_miss_duplicate_concurrent_work",
        "Layer: Caching. Flag async or threaded cache miss paths that recompute the same key concurrently."
    ),
    perf_layer_rule!(
        "python_perf_layer_caching_cached_value_immediately_deserialized",
        "Layer: Caching. Flag caches storing serialized data only to deserialize before every use."
    ),
    perf_layer_rule!(
        "python_perf_layer_caching_per_request_settings_cache_rebuilt",
        "Layer: Caching. Flag settings or feature-flag caches rebuilt for each request."
    ),
    // Serialization and deserialization
    perf_layer_rule!(
        "python_perf_layer_serialization_json_dumps_for_equality_or_hash",
        "Layer: Serialization and deserialization. Flag json.dumps used only for equality checks, hashing, or dedup keys."
    ),
    perf_layer_rule!(
        "python_perf_layer_serialization_pickle_in_request_hot_path",
        "Layer: Serialization and deserialization. Flag pickle serialization in request paths where structured fields or lighter codecs would suffice."
    ),
    perf_layer_rule!(
        "python_perf_layer_serialization_pydantic_model_recreated_for_internal_dict",
        "Layer: Serialization and deserialization. Flag Pydantic model construction around trusted internal dicts in tight loops."
    ),
    perf_layer_rule!(
        "python_perf_layer_serialization_base64_roundtrip_same_process",
        "Layer: Serialization and deserialization. Flag base64 encode/decode roundtrips inside one process boundary."
    ),
    perf_layer_rule!(
        "python_perf_layer_serialization_gzip_compress_tiny_payloads",
        "Layer: Serialization and deserialization. Flag gzip compression of tiny payloads where CPU overhead can exceed transfer savings."
    ),
    // Logging overhead
    perf_layer_rule!(
        "python_perf_layer_logging_overhead_log_message_formatted_before_level_check",
        "Layer: Logging overhead. Flag f-string or format log messages built before logger level checks can skip them."
    ),
    perf_layer_rule!(
        "python_perf_layer_logging_overhead_per_item_info_log_in_batch",
        "Layer: Logging overhead. Flag info/debug logging per record in large batch processing loops."
    ),
    perf_layer_rule!(
        "python_perf_layer_logging_overhead_extra_fields_computed_before_sampling",
        "Layer: Logging overhead. Flag expensive log extra fields computed before sampling or level filters."
    ),
    perf_layer_rule!(
        "python_perf_layer_logging_overhead_logger_created_per_request",
        "Layer: Logging overhead. Flag getLogger or adapter construction per request when stable module loggers are enough."
    ),
    perf_layer_rule!(
        "python_perf_layer_logging_overhead_trace_payload_serialized_unconditionally",
        "Layer: Logging overhead. Flag trace or log payload serialization even when tracing is disabled."
    ),
    // Error handling cost
    perf_layer_rule!(
        "python_perf_layer_error_handling_cost_exception_used_for_loop_control",
        "Layer: Error handling cost. Flag exceptions used for ordinary loop control in hot paths."
    ),
    perf_layer_rule!(
        "python_perf_layer_error_handling_cost_raise_from_none_hides_retriable_error_context",
        "Layer: Error handling cost. Flag error translation that drops cause context needed to avoid repeated failed retries."
    ),
    perf_layer_rule!(
        "python_perf_layer_error_handling_cost_error_message_formatted_on_success_path",
        "Layer: Error handling cost. Flag expensive error messages built before checking whether an error exists."
    ),
    perf_layer_rule!(
        "python_perf_layer_error_handling_cost_broad_except_retries_cpu_work",
        "Layer: Error handling cost. Flag broad except retry loops that repeat CPU-heavy work instead of isolating the failing call."
    ),
    perf_layer_rule!(
        "python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure",
        "Layer: Error handling cost. Flag error accumulation lists allocated on success-heavy paths before any error occurs."
    ),
    // Build and runtime configuration
    perf_layer_rule!(
        "python_perf_layer_runtime_configuration_debug_mode_enabled_in_perf_sensitive_path",
        "Layer: Build and runtime configuration. Flag framework debug or reload mode enabled in performance-sensitive runtime config."
    ),
    perf_layer_rule!(
        "python_perf_layer_runtime_configuration_pythonpath_mutated_per_call",
        "Layer: Build and runtime configuration. Flag sys.path or import path mutation inside functions instead of startup configuration."
    ),
    perf_layer_rule!(
        "python_perf_layer_runtime_configuration_locale_timezone_loaded_per_request",
        "Layer: Build and runtime configuration. Flag locale or timezone data loaded for every request instead of cached process state."
    ),
    perf_layer_rule!(
        "python_perf_layer_runtime_configuration_env_parsed_per_operation",
        "Layer: Build and runtime configuration. Flag environment parsing and type conversion per operation instead of startup settings load."
    ),
    perf_layer_rule!(
        "python_perf_layer_runtime_configuration_profiling_hooks_enabled_by_default",
        "Layer: Build and runtime configuration. Flag cProfile, tracing, or coverage hooks enabled in default runtime paths."
    ),
    // Hot path optimization
    perf_layer_rule!(
        "python_perf_layer_hot_path_optimization_schema_parsed_inside_handler",
        "Layer: Hot path optimization. Flag schemas, templates, or expressions parsed per request instead of precompiled once."
    ),
    perf_layer_rule!(
        "python_perf_layer_hot_path_optimization_datetime_now_called_per_field",
        "Layer: Hot path optimization. Flag repeated datetime.now calls per record when one timestamp can be reused."
    ),
    perf_layer_rule!(
        "python_perf_layer_hot_path_optimization_attribute_lookup_in_deep_inner_loop",
        "Layer: Hot path optimization. Flag repeated dynamic getattr or attribute lookup in deep loops where local binding helps."
    ),
    perf_layer_rule!(
        "python_perf_layer_hot_path_optimization_reflection_inspection_in_hot_path",
        "Layer: Hot path optimization. Flag inspect/signature/reflection work in hot paths."
    ),
    perf_layer_rule!(
        "python_perf_layer_hot_path_optimization_numpy_python_loop_over_array",
        "Layer: Hot path optimization. Flag Python loops over large NumPy arrays where vectorized operations fit the computation."
    ),
    // Lazy loading
    perf_layer_rule!(
        "python_perf_layer_lazy_loading_optional_model_loaded_at_startup",
        "Layer: Lazy loading. Flag optional ML/model assets loaded at startup before any route or job needs them."
    ),
    perf_layer_rule!(
        "python_perf_layer_lazy_loading_lazy_init_on_first_user_request",
        "Layer: Lazy loading. Flag lazy initialization that shifts a large one-time cost onto the first user request without warmup."
    ),
    perf_layer_rule!(
        "python_perf_layer_lazy_loading_eager_import_heavy_optional_dependency",
        "Layer: Lazy loading. Flag heavy optional dependencies imported at module import time for rarely used features."
    ),
    perf_layer_rule!(
        "python_perf_layer_lazy_loading_eager_template_compile_for_unused_views",
        "Layer: Lazy loading. Flag compiling templates for disabled or rarely used views during startup."
    ),
    perf_layer_rule!(
        "python_perf_layer_lazy_loading_feature_flag_data_loaded_before_needed",
        "Layer: Lazy loading. Flag feature-specific lookup data loaded before the feature is enabled or requested."
    ),
    // Resource pooling
    perf_layer_rule!(
        "python_perf_layer_resource_pooling_db_engine_created_per_repository",
        "Layer: Resource pooling. Flag database engine/session factory construction per repository instance."
    ),
    perf_layer_rule!(
        "python_perf_layer_resource_pooling_http_pool_not_reused",
        "Layer: Resource pooling. Flag HTTP connection pools that are recreated rather than shared per upstream."
    ),
    perf_layer_rule!(
        "python_perf_layer_resource_pooling_process_pool_without_shutdown",
        "Layer: Resource pooling. Flag ProcessPoolExecutor ownership without shutdown or bounded queue behavior."
    ),
    perf_layer_rule!(
        "python_perf_layer_resource_pooling_large_buffer_pool_without_cap",
        "Layer: Resource pooling. Flag reusable buffer pools that keep oversized buffers after traffic spikes."
    ),
    perf_layer_rule!(
        "python_perf_layer_resource_pooling_rate_limiter_created_per_request",
        "Layer: Resource pooling. Flag rate limiter construction per request instead of per identity or dependency."
    ),
    // Framework-specific performance
    perf_layer_rule!(
        "python_perf_layer_framework_performance_django_queryset_evaluated_in_template_loop",
        "Layer: Framework-specific performance. Flag Django QuerySets evaluated repeatedly from templates or per-row template helpers."
    ),
    perf_layer_rule!(
        "python_perf_layer_framework_performance_fastapi_dependency_builds_client_per_request",
        "Layer: Framework-specific performance. Flag FastAPI dependencies that build heavy clients for every request."
    ),
    perf_layer_rule!(
        "python_perf_layer_framework_performance_sqlalchemy_lazy_load_in_serializer",
        "Layer: Framework-specific performance. Flag SQLAlchemy lazy loads triggered from serializers or response mapping loops."
    ),
    perf_layer_rule!(
        "python_perf_layer_framework_performance_celery_task_loads_config_each_run",
        "Layer: Framework-specific performance. Flag Celery tasks loading settings, models, or clients on every invocation without reuse."
    ),
    perf_layer_rule!(
        "python_perf_layer_framework_performance_pandas_apply_for_vectorizable_operation",
        "Layer: Framework-specific performance. Flag pandas apply calls for operations that have vectorized Series/DataFrame equivalents."
    ),
    // Profiling and benchmarking
    perf_layer_rule!(
        "python_perf_layer_profiling_benchmarking_benchmark_missing_warmup",
        "Layer: Profiling and benchmarking. Flag benchmarks that time import/setup/warmup along with the operation under test."
    ),
    perf_layer_rule!(
        "python_perf_layer_profiling_benchmarking_timeit_result_not_consumed",
        "Layer: Profiling and benchmarking. Flag microbenchmarks whose results are unused and may measure optimized-away or irrelevant work."
    ),
    perf_layer_rule!(
        "python_perf_layer_profiling_benchmarking_profile_claim_without_profile_artifact",
        "Layer: Profiling and benchmarking. Flag optimization comments or docs without profile, benchmark, or trace evidence."
    ),
    perf_layer_rule!(
        "python_perf_layer_profiling_benchmarking_benchmark_allocations_not_tracked",
        "Layer: Profiling and benchmarking. Flag memory-sensitive benchmarks that omit tracemalloc, memory_profiler, or allocation accounting."
    ),
    perf_layer_rule!(
        "python_perf_layer_profiling_benchmarking_load_test_without_realistic_payload_size",
        "Layer: Profiling and benchmarking. Flag performance tests that use tiny payloads despite rules targeting large-input behavior."
    ),
];
