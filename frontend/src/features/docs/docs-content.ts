import { currentRelease } from '../../content/site-content'

// ─── Types ────────────────────────────────────────────────────────────────────

type Language = 'go' | 'python' | 'rust' | 'common'
type SectionId =
  | 'overview'
  | 'detection-rules'
  | 'cli-commands'
  | 'pipeline'
  | 'limitations'
  | 'about'

interface NavSection {
  id: SectionId
  label: string
  icon: string
}

interface Rule {
  id: string
  description: string
}

interface CliCommand {
  cmd: string
  desc: string
}

interface GitHubActionInput {
  name: string
  description: string
}

// ─── Static Data ──────────────────────────────────────────────────────────────

const languages: { id: Language; label: string }[] = [
  { id: 'go', label: 'Go' },
  { id: 'python', label: 'Python' },
  { id: 'rust', label: 'Rust' },
  { id: 'common', label: 'Common' },
]

const sections: NavSection[] = [
  { id: 'overview', label: 'Overview', icon: '◈' },
  { id: 'detection-rules', label: 'Detection Rules', icon: '⊹' },
  { id: 'cli-commands', label: 'CLI Commands', icon: '❯' },
  { id: 'pipeline', label: 'Pipeline', icon: '◎' },
  { id: 'limitations', label: 'Limitations', icon: '△' },
  { id: 'about', label: 'About', icon: '♡' },
]

// ─── Content Data ────────────────────────────────────────────────────────────

// GENERATED_RULES_START
const commonRules: Rule[] = [
  { id: 'comment_style_title_case', description: 'Heading-like Title Case documentation.' },
  { id: 'comment_style_tutorial', description: 'Tutorial-style documentation that narrates obvious implementation steps.' },
  { id: 'hallucinated_import_call', description: 'Package-qualified calls that do not match locally indexed symbols for the imported package.' },
  { id: 'hallucinated_local_call', description: 'Same-package calls to symbols not present in the scanned local package context.' },
  { id: 'generic_name', description: 'Function names that are overly generic without stronger contextual signals.' },
  { id: 'overlong_name', description: 'Very long identifiers with too many descriptive tokens.' },
  { id: 'weak_typing', description: 'Signatures that rely on any or empty interface types.' },
  { id: 'hardcoded_secret', description: 'Secret-like identifiers assigned direct string literals instead of environment lookups.' },
  { id: 'happy_path_only_test', description: 'Tests that assert success expectations without any obvious negative-path signal.' },
  { id: 'placeholder_test_body', description: 'Tests that look skipped, TODO-shaped, or otherwise placeholder-like.' },
  { id: 'test_without_assertion_signal', description: 'Tests that exercise production code without an obvious assertion or failure signal.' },
]

const goRules: Rule[] = [
  { id: 'blocking_call_while_locked', description: 'Potentially blocking calls observed between Lock and Unlock.' },
  { id: 'goroutine_derived_context_unmanaged', description: 'Likely long-lived goroutines launched after a derived context is created and before the matching cancel call is observed.' },
  { id: 'goroutine_spawn_in_loop', description: 'Raw go statements launched from inside loops without obvious WaitGroup coordination.' },
  { id: 'goroutine_without_coordination', description: 'Raw go statements without an obvious context or WaitGroup-like coordination signal.' },
  { id: 'goroutine_without_shutdown_path', description: 'Looping goroutine literals without an obvious ctx.Done() or done-channel shutdown path.' },
  { id: 'mutex_in_loop', description: 'Repeated Lock or RLock acquisition inside loops.' },
  { id: 'duplicate_struct_tag_key', description: 'Struct field tags that repeat the same key more than once.' },
  { id: 'malformed_struct_tag', description: 'Struct field tags that do not parse as valid Go tag key/value pairs.' },
  { id: 'mixed_receiver_kinds', description: 'Methods on the same receiver type mix pointer and value receivers.' },
  { id: 'busy_waiting', description: 'select { default: ... } inside loops, which often spins instead of blocking.' },
  { id: 'context_background_used', description: 'Functions that already accept context.Context but still create context.Background() or context.TODO() locally.' },
  { id: 'missing_cancel_call', description: 'Derived contexts where deslop cannot find a local cancel() or defer cancel() call.' },
  { id: 'missing_context', description: 'Standard-library context-aware calls from functions that do not accept context.Context.' },
  { id: 'missing_context_propagation', description: 'Functions that already accept context.Context but still call context-free stdlib APIs like http.Get or exec.Command.' },
  { id: 'sleep_polling', description: 'time.Sleep inside loops — often indicates polling or busy-wait style code.' },
  { id: 'association_find_inside_loop', description: 'GORM Association(...).Find(...) loaders observed inside loops.' },
  { id: 'automigrate_or_schema_probe_in_request_path', description: 'AutoMigrate or schema probes running on request paths instead of startup.' },
  { id: 'connection_pool_reconfigured_per_request', description: 'DB pool sizing or lifetime settings changed on request paths.' },
  { id: 'count_inside_loop', description: 'COUNT(...) or GORM Count(...) observed inside loops.' },
  { id: 'count_then_find_same_filter', description: 'Request-path GORM flows that run Count(...) and then a broad Find(...) with the same filter shape.' },
  { id: 'create_single_in_loop_instead_of_batches', description: 'GORM .Create(...) used inside loops with no visible CreateInBatches(...) path in the same function.' },
  { id: 'date_or_cast_wrapped_indexed_column', description: 'DATE/CAST wrapping indexed columns in WHERE clauses, preventing index usage.' },
  { id: 'db_ping_per_request', description: 'database Ping(...) or PingContext(...) called on request paths instead of startup or explicit health checks.' },
  { id: 'default_transaction_enabled_for_bulk_create', description: 'Bulk creates performed with GORM default transaction enabled — SkipDefaultTransaction improves throughput.' },
  { id: 'delete_single_row_in_loop_without_batch', description: 'GORM Delete(...) chains observed inside loops one row at a time.' },
  { id: 'distinct_wide_row_request_path', description: 'Distinct on wide rows without Select projection — a key-only subquery is usually cheaper.' },
  { id: 'duplicate_find_then_preload_followup', description: 'Initial Find followed by a separate Preload query that could be folded into one.' },
  { id: 'exec_inside_loop_without_batch', description: 'Exec(...) or ExecContext(...) used for row-by-row SQL writes inside loops.' },
  { id: 'exists_via_count_star', description: 'COUNT(*) used for an existence check instead of EXISTS or LIMIT 1.' },
  { id: 'find_all_then_manual_paginate_in_go', description: 'All rows fetched and then sliced in Go instead of using database-level pagination.' },
  { id: 'findinbatches_candidate_for_large_scan', description: 'Unbounded result sets that could use FindInBatches or cursor iteration.' },
  { id: 'first_or_create_in_loop', description: 'GORM FirstOrCreate(...) chains observed inside loops.' },
  { id: 'gorm_debug_enabled_in_request_path', description: 'GORM debug logging enabled on request paths.' },
  { id: 'gorm_find_without_limit_on_handler_path', description: 'Request-path GORM Find(...) chains with no visible Limit(...) step.' },
  { id: 'gorm_joins_plus_preload_plus_find_without_limit', description: 'GORM chains combining Joins, Preload, and unbounded Find on request paths.' },
  { id: 'gorm_open_per_request', description: 'gorm.Open(...) called on request paths instead of process-level setup.' },
  { id: 'gorm_preload_clause_associations_on_wide_graph', description: 'Request-path GORM chains that use Preload(clause.Associations) or other broad preload graphs.' },
  { id: 'gorm_select_missing_projection_on_wide_model', description: 'GORM queries on wide models without a Select projection to limit fetched columns.' },
  { id: 'gorm_session_allocated_per_item', description: 'GORM Session(...) chains allocated inside loops before issuing queries.' },
  { id: 'leading_wildcard_builder_chain', description: 'LIKE queries with leading wildcards that prevent index usage.' },
  { id: 'lower_or_func_wrapped_indexed_column', description: 'LOWER() or other functions wrapping indexed columns in WHERE clauses, preventing index usage.' },
  { id: 'many_column_or_filter_chain', description: 'Query chains with many OR filter conditions that often scale poorly.' },
  { id: 'nested_transaction_in_request_path', description: 'Multiple transactions started on a single request path.' },
  { id: 'offset_pagination_on_large_table', description: 'Request-path GORM Find(...) chains that page with Offset(...), which often scales poorly on large lists.' },
  { id: 'order_by_random_request_path', description: 'ORDER BY RAND()/RANDOM() observed on request paths.' },
  { id: 'order_by_without_limit_orm_chain', description: 'ORM chains that order results without a Limit on request paths.' },
  { id: 'pgx_collectrows_unbounded_materialization', description: 'pgx.CollectRows used on request paths without a visible LIMIT in the query.' },
  { id: 'preload_inside_loop', description: 'GORM Preload(...) queries configured and executed inside loops.' },
  { id: 'prepare_inside_loop', description: 'Prepare(...) or PrepareContext(...) observed inside loops.' },
  { id: 'prepare_on_every_request_same_sql', description: 'The same literal SQL is prepared multiple times on one request path.' },
  { id: 'queryrow_inside_loop_existence_check', description: 'QueryRow(...) or QueryRowContext(...) used inside loops for point lookups that usually want a bulk prefetch path.' },
  { id: 'raw_scan_inside_loop', description: 'GORM Raw(...).Scan(...) chains observed inside loops.' },
  { id: 'repeated_same_query_template_same_function', description: 'The same query template executed multiple times in one function.' },
  { id: 'row_by_row_upsert_loop', description: 'Upsert-style writes executed row by row inside loops instead of batched.' },
  { id: 'rows_to_struct_allocation_per_row_without_reuse', description: 'New struct allocated for each row scan instead of reusing a scratch variable.' },
  { id: 'save_for_single_column_change', description: 'GORM Save used for a single-column update instead of a targeted Update call.' },
  { id: 'save_in_loop_full_model', description: 'GORM Save(...) writes full models inside loops.' },
  { id: 'scan_into_map_string_any_hot_path', description: 'Rows scanned into map[string]any instead of typed structs on hot paths.' },
  { id: 'select_or_get_inside_loop_lookup', description: 'ORM lookups (Select, Get, First, etc.) executed inside loops.' },
  { id: 'sql_open_per_request', description: 'database/sql pools opened on request paths instead of process-level setup.' },
  { id: 'sqlx_select_large_slice_without_limit', description: 'sqlx.Select used on request paths without a visible LIMIT in the query.' },
  { id: 'tx_begin_per_item_loop', description: 'Transactions started inside loops instead of once around the wider batch.' },
  { id: 'unbounded_in_clause_expansion', description: 'IN clauses built from request-driven collections without bound limits.' },
  { id: 'update_single_row_in_loop_without_batch', description: 'GORM Update(...), UpdateColumn(...), or Updates(...) calls observed inside loops one row at a time.' },
  { id: 'updates_map_allocated_per_row', description: 'GORM Updates(map[string]...) maps allocated inside loops for per-row updates.' },
  { id: 'dropped_error', description: 'Blank identifier assignments that discard an err-like value.' },
  { id: 'error_wrapping_misuse', description: 'fmt.Errorf calls that reference err without %w.' },
  { id: 'panic_on_error', description: 'err != nil branches that jump straight to panic or log.Fatal style exits.' },
  { id: 'bindjson_into_map_any_hot_endpoint', description: 'Gin handlers that bind JSON into map[string]any or map[string]interface{} on hot request paths.' },
  { id: 'bindquery_into_map_any_hot_endpoint', description: 'Gin handlers that bind query parameters into map[string]any or map[string]interface{} on hot request paths.' },
  { id: 'dumprequest_or_dumpresponse_in_hot_path', description: 'Request-path handlers that dump full HTTP requests or responses with httputil.' },
  { id: 'duplicate_upstream_calls_same_url_same_handler', description: 'Same upstream URL called multiple times in one handler.' },
  { id: 'env_or_config_lookup_per_request', description: 'Environment variable reads observed on request paths instead of cached configuration.' },
  { id: 'errgroup_fanout_without_limit_in_handler', description: 'errgroup goroutine fanout without a visible concurrency limit in handlers.' },
  { id: 'file_or_template_read_per_request', description: 'Request-path handlers that read files directly instead of using startup caching or dedicated file-serving paths.' },
  { id: 'formfile_open_readall_whole_upload', description: 'Gin handlers that open uploaded form files and then materialize them with io.ReadAll(...).' },
  { id: 'get_raw_data_then_should_bindjson_duplicate_body', description: 'Gin handlers that read GetRawData() and later bind JSON from the same request body.' },
  { id: 'gin_context_copy_for_each_item_fanout', description: 'Gin handlers that call c.Copy() once per loop iteration before goroutine fanout.' },
  { id: 'gin_logger_debug_body_logging_on_hot_routes', description: 'Verbose body or payload logging observed on likely high-volume Gin routes.' },
  { id: 'gzip_or_zip_writer_created_per_chunk', description: 'Gzip or zip writers recreated per chunk inside handler loops instead of reusing per stream.' },
  { id: 'indentedjson_in_hot_path', description: 'IndentedJSON(...) used on a request path instead of compact JSON rendering.' },
  { id: 'json_marshaled_manually_then_c_data', description: 'Handlers that manually marshal JSON and then write it through gin.Context.Data(...).' },
  { id: 'large_csv_or_json_export_without_bufio', description: 'Export data written in loops without visible buffering in handlers.' },
  { id: 'large_h_payload_built_only_for_json_response', description: 'Large gin.H payloads built as transient dynamic maps right before JSON rendering.' },
  { id: 'loadhtmlglob_or_loadhtmlfiles_in_request_path', description: 'LoadHTMLGlob or LoadHTMLFiles called on request paths instead of startup initialization.' },
  { id: 'middleware_allocates_db_or_gorm_handle_per_request', description: 'Database or GORM connections opened inside handlers or middleware instead of process-level setup.' },
  { id: 'middleware_allocates_http_client_per_request', description: 'HTTP client allocated inside Gin handlers or middleware instead of being shared.' },
  { id: 'middleware_allocates_regex_or_template_per_request', description: 'Regexp compilation inside Gin handlers instead of using precompiled patterns.' },
  { id: 'middleware_rebinds_body_after_handler_bind', description: 'Middleware or helper chains that parse the request body after the main handler has already bound it.' },
  { id: 'multiple_shouldbind_calls_same_handler', description: 'Gin handlers that bind the request body multiple times in one function.' },
  { id: 'no_batching_on_handler_driven_db_write_loop', description: 'Request handlers that drive row-by-row DB writes with no batch path.' },
  { id: 'no_streaming_for_large_export_handler', description: 'Large list or export handlers that materialize everything before writing rather than using chunked or streaming output.' },
  { id: 'parsemultipartform_large_default_memory', description: 'Gin handlers that call ParseMultipartForm(...) with large in-memory thresholds on request paths.' },
  { id: 'readall_body_then_bind_duplicate_deserialize', description: 'Gin handlers that materialize c.Request.Body with io.ReadAll(...) and then bind the same body again.' },
  { id: 'repeated_body_rewind_for_multiple_decoders', description: 'Gin handlers that read, rewind, and decode the same request body multiple times.' },
  { id: 'repeated_c_json_inside_stream_loop', description: 'Gin handlers that call c.JSON(...) or c.PureJSON(...) from inside loops.' },
  { id: 'repeated_large_map_literal_response_construction', description: 'Large map-literal response assembly on hot routes where a stable typed response would be cheaper.' },
  { id: 'servefile_via_readfile_then_c_data', description: 'Handlers that load files into memory and then write them through gin.Context.Data(...) instead of using file helpers or streaming.' },
  { id: 'shouldbindbodywith_when_single_bind_is_enough', description: 'Gin handlers that use ShouldBindBodyWith(...) even though only one body bind is observed.' },
  { id: 'template_parse_in_handler', description: 'Template construction or parsing inside Gin handlers instead of startup-time caching.' },
  { id: 'upstream_http_call_per_item_in_handler_loop', description: 'Upstream HTTP calls made per item inside handler loops.' },
  { id: 'upstream_json_decode_same_response_multiple_times', description: 'One upstream HTTP response body decoded into multiple targets in the same handler.' },
  { id: 'append_then_sort_each_iteration', description: 'Slice sorting observed inside loops — often cheaper to sort once after the loop.' },
  { id: 'append_then_trim_each_iteration', description: 'Slice append followed by reslice each iteration instead of batching.' },
  { id: 'bufio_reader_missing_for_small_read_loop', description: 'File or socket reads inside loops without visible bufio buffering.' },
  { id: 'bufio_writer_missing_in_bulk_export', description: 'File or socket writes inside loops without visible bufio buffering.' },
  { id: 'builder_or_buffer_recreated_per_iteration', description: 'strings.Builder, bytes.Buffer, or bytes.NewBuffer(...) constructions observed inside loops instead of being reset or reused.' },
  { id: 'byte_string_conversion_in_loop', description: 'Byte-to-string or string-to-byte conversion observed inside loops in short-lived lookup or append paths.' },
  { id: 'bytes_buffer_without_grow_known_bound', description: 'bytes.Buffer used without Grow when approximate output size is locally visible.' },
  { id: 'bytes_split_same_input_multiple_times', description: 'The same byte-slice input is passed through bytes.Split* or bytes.Fields* helpers multiple times in one function.' },
  { id: 'csv_writer_flush_per_row', description: 'csv.Writer.Flush() called inside per-row loops, reducing buffering effectiveness.' },
  { id: 'filter_then_count_then_iterate', description: 'Same collection traversed multiple times for filter, count, and process steps.' },
  { id: 'gzip_reader_writer_recreated_per_item', description: 'gzip.NewReader(...) or gzip.NewWriter(...) recreated inside iterative paths instead of per stream.' },
  { id: 'json_decoder_recreated_per_item', description: 'json.NewDecoder(...) constructed repeatedly inside loops instead of reusing a stable decoder per stream.' },
  { id: 'json_encoder_recreated_per_item', description: 'json.NewEncoder(...) constructed repeatedly inside loops instead of reusing a stable encoder per stream.' },
  { id: 'make_map_inside_hot_loop_same_shape', description: 'make(map[K]V, ...) scratch maps recreated inside loops instead of being reused or prebuilt.' },
  { id: 'make_slice_inside_hot_loop_same_shape', description: 'make([]T, ...) scratch slices recreated inside loops instead of being reused.' },
  { id: 'map_growth_without_size_hint', description: 'Map insertions inside loops without a visible size hint on the initial make call.' },
  { id: 'nested_append_without_outer_capacity', description: 'Append calls inside nested loops without visible preallocation on the outer slice.' },
  { id: 'nested_linear_join_map_candidate', description: 'Nested-loop lookups or joins that could use a map index for O(1) access.' },
  { id: 'read_then_decode_duplicate_materialization', description: 'io.ReadAll(...) materializes a payload and the same binding is then unmarshaled again instead of using a streaming decode path.' },
  { id: 'regexp_compile_in_hot_path', description: 'regexp.Compile or regexp.MustCompile observed inside obvious iterative paths.' },
  { id: 'repeated_map_clone_in_loop', description: 'maps.Clone or equivalent map-copy calls observed inside loops.' },
  { id: 'repeated_slice_clone_in_loop', description: 'slices.Clone(...) or similar whole-slice cloning observed inside loops.' },
  { id: 'slice_append_without_prealloc_known_bound', description: 'Slice append inside a range loop without visible preallocation when the bound is locally known.' },
  { id: 'slice_membership_in_loop_map_candidate', description: 'slices.Contains(...) or slices.Index(...) used inside loops against a stable-looking slice binding.' },
  { id: 'sort_before_first_or_membership_only', description: 'Sorting a collection when only the first element or min/max is needed.' },
  { id: 'stable_value_normalization_in_inner_loop', description: 'Stable value normalization (ToLower, TrimSpace, etc.) repeated inside inner loops.' },
  { id: 'strconv_repeat_on_same_binding', description: 'The same string binding is converted with strconv parsing helpers multiple times in one function.' },
  { id: 'strings_builder_without_grow_known_bound', description: 'strings.Builder used without Grow when approximate output size is locally visible.' },
  { id: 'strings_split_same_input_multiple_times', description: 'The same string input is passed through strings.Split* or strings.Fields* helpers multiple times in one function.' },
  { id: 'template_parse_in_hot_path', description: 'html/template or text/template parse calls observed on request-style paths instead of startup-time caching.' },
  { id: 'time_parse_layout_in_loop', description: 'time.Parse(...) or ParseInLocation(...) observed inside loops with a stable layout.' },
  { id: 'url_parse_in_loop_on_invariant_base', description: 'url.Parse(...) or ParseRequestURI(...) observed inside loops with a stable-looking base input.' },
  { id: 'uuid_hash_formatting_only_for_logs', description: 'UUID or hash formatting observed inside loops only for log output.' },
  { id: 'defer_in_loop_resource_growth', description: 'defer statements inside loops that can accumulate resources until function exit.' },
  { id: 'double_close_local_channel', description: 'The same locally created channel appears to be closed more than once in one function body.' },
  { id: 'file_handle_without_close', description: 'File handles opened via os.Open, os.Create, or os.OpenFile without an observed Close() path.' },
  { id: 'http_client_without_timeout', description: 'Local http.Client{} literals constructed without an explicit timeout.' },
  { id: 'http_response_body_not_closed', description: 'HTTP responses acquired locally without an observed resp.Body.Close() call.' },
  { id: 'http_server_without_timeouts', description: 'Explicit http.Server{} values that omit common timeout fields.' },
  { id: 'http_status_ignored_before_decode', description: 'Response decoding or body consumption that happens without an observed StatusCode check.' },
  { id: 'http_writeheader_after_write', description: 'Handlers that write the response body before calling WriteHeader(...).' },
  { id: 'init_side_effect', description: 'init() functions that perform network, file-system, or subprocess side effects.' },
  { id: 'mutable_package_global', description: 'Package-level variables that are mutated from function bodies instead of kept immutable.' },
  { id: 'passthrough_wrapper_interface', description: 'Wrapper structs that mostly forward one-to-one through an interface field with little added policy.' },
  { id: 'public_bool_parameter_api', description: 'Exported functions or methods that expose raw boolean mode switches in their signatures.' },
  { id: 'range_over_local_channel_without_close', description: 'Functions that range over a locally owned channel without an observed close path.' },
  { id: 'rows_without_close', description: 'Query result handles that appear locally owned but have no observed rows.Close() call.' },
  { id: 'send_after_local_close_risk', description: 'A locally owned channel is closed and later used in a send expression.' },
  { id: 'single_impl_interface', description: 'Repository-local interfaces with one obvious implementation and a very small consumer surface.' },
  { id: 'stmt_without_close', description: 'Prepared statements or similar DB handles without an observed Close() call.' },
  { id: 'ticker_without_stop', description: 'time.NewTicker(...) is created without an observed Stop() call.' },
  { id: 'time_after_in_loop', description: 'time.After(...) is allocated inside loops instead of reusing a timer or deadline.' },
  { id: 'tx_without_rollback_guard', description: 'Transactions begun and later committed with no observed rollback guard.' },
  { id: 'json_unmarshal_same_payload_multiple_times', description: 'The same local JSON payload binding is unmarshaled into multiple targets in one function.' },
  { id: 'proto_unmarshal_same_payload_multiple_times', description: 'The same local protobuf payload binding is unmarshaled into multiple targets in one function.' },
  { id: 'xml_unmarshal_same_payload_multiple_times', description: 'The same local XML payload binding is unmarshaled into multiple targets in one function.' },
  { id: 'yaml_unmarshal_same_payload_multiple_times', description: 'The same local YAML payload binding is unmarshaled into multiple targets in one function.' },
  { id: 'allocation_churn_in_loop', description: 'Obvious make, new, or buffer-construction calls inside loops.' },
  { id: 'fmt_hot_path', description: 'fmt formatting calls such as Sprintf inside loops.' },
  { id: 'full_dataset_load', description: 'Calls that load an entire payload into memory instead of streaming.' },
  { id: 'likely_n_squared_allocation', description: 'Opt-in deeper semantic signal for allocations that also sit inside nested loop structure.' },
  { id: 'likely_n_squared_string_concat', description: 'Opt-in deeper semantic signal for repeated string concatenation inside nested loops without obvious builder usage.' },
  { id: 'likely_unindexed_query', description: 'Query shapes like leading-wildcard LIKE or ORDER BY without LIMIT that often scale poorly.' },
  { id: 'n_plus_one_query', description: 'Database-style query calls issued inside loops. The opt-in semantic pack can raise severity when nested loops also appear.' },
  { id: 'reflection_hot_path', description: 'reflect package calls inside loops.' },
  { id: 'repeated_json_marshaling', description: 'encoding/json.Marshal or MarshalIndent inside loops — repeated allocation and serialization hot spots.' },
  { id: 'string_concat_in_loop', description: 'Repeated string concatenation inside loops (O(n^2) risk).' },
  { id: 'wide_select_query', description: 'Literal SELECT * query shapes.' },
  { id: 'sql_string_concat', description: 'Query execution calls where SQL is constructed dynamically with concatenation or fmt.Sprintf.' },
  { id: 'weak_crypto', description: 'Direct use of weak standard-library crypto packages such as crypto/md5, crypto/sha1, crypto/des, and crypto/rc4.' },
  { id: 'inconsistent_package_name', description: 'Directories that mix base Go package names after ignoring the _test suffix.' },
  { id: 'misgrouped_imports', description: 'Import blocks that place stdlib imports after third-party imports.' },
]

const pythonRules: Rule[] = [
  { id: 'enthusiastic_commentary', description: 'Unusually enthusiastic or emoji-heavy production comments.' },
  { id: 'mixed_naming_conventions', description: 'File mixes snake_case and camelCase function naming conventions.' },
  { id: 'obvious_commentary', description: 'Comments that narrate obvious implementation steps instead of explaining intent.' },
  { id: 'textbook_docstring_small_helper', description: 'Very small helper functions that have unusually long, textbook-style docstrings.' },
  { id: 'unrelated_heavy_import', description: 'Heavy ecosystem imports with little local evidence of real need.' },
  { id: 'cross_file_copy_paste_function', description: 'Highly similar non-test function bodies repeated across multiple Python files.' },
  { id: 'cross_file_repeated_literal', description: 'Project repeats the same long string literal across multiple files.' },
  { id: 'duplicate_error_handler_block', description: 'Repeated exception-handling block shapes in one file.' },
  { id: 'duplicate_query_fragment', description: 'Repository repeats the same SQL-like query fragment across multiple files.' },
  { id: 'duplicate_test_utility_logic', description: 'Highly similar utility logic shared between test and production code.' },
  { id: 'duplicate_transformation_pipeline', description: 'Repository repeats the same data transformation pipeline stages across multiple functions.' },
  { id: 'duplicate_validation_pipeline', description: 'Repeated validation guard pipelines across functions in one file.' },
  { id: 'repeated_string_literal', description: 'Project repeats the same long string literal multiple times in one file.' },
  { id: 'celery_delay_in_loop_without_canvas', description: 'Celery tasks dispatch .delay(...) or .apply_async(...) inside loops without an obvious canvas primitive such as group() or chord().' },
  { id: 'celery_result_get_inside_task', description: 'Celery tasks synchronously wait on AsyncResult.get(...) instead of handing work off asynchronously.' },
  { id: 'celery_task_reads_env_per_invocation', description: 'Celery task bodies repeatedly read environment configuration instead of using startup-time bootstrap.' },
  { id: 'click_typer_config_file_loaded_per_command', description: 'click or typer commands parse config files on each invocation instead of using shared bootstrap or dependency setup.' },
  { id: 'click_typer_env_lookup_per_command', description: 'click or typer commands scatter repeated environment lookups through the command body.' },
  { id: 'click_typer_http_client_created_per_command', description: 'click or typer commands allocate HTTP clients inside command bodies instead of reusing a shared client factory.' },
  { id: 'django_all_without_limit_in_view', description: 'Django views call .all() without visible pagination, slicing, or limiting.' },
  { id: 'django_create_single_in_loop', description: 'Django code creates one model at a time inside loops instead of using bulk_create().' },
  { id: 'django_delete_single_in_loop', description: 'Django code deletes individual models inside loops instead of using set-based queryset deletion.' },
  { id: 'django_migration_code_in_view', description: 'Django views or request paths reference schema or migration operations that belong in migrations.' },
  { id: 'django_n_plus_one_no_select_related', description: 'Django queryset iteration shows N+1 risk with no visible select_related(...) or prefetch_related(...).' },
  { id: 'django_queryset_count_then_exists', description: 'Django querysets use count() for existence checks instead of exists().' },
  { id: 'django_queryset_evaluated_multiple_times', description: 'The same Django queryset appears to be evaluated multiple times in one function.' },
  { id: 'django_queryset_len_instead_of_count', description: 'len(queryset) is used where queryset.count() would avoid loading every row.' },
  { id: 'django_queryset_order_by_random', description: 'Django querysets use order_by(\\"?\\") or equivalent random ordering on request paths.' },
  { id: 'django_raw_sql_in_loop', description: 'Django request or service code executes raw SQL inside loops instead of batching.' },
  { id: 'django_save_full_model_in_loop', description: 'Django code saves full model instances in loops without update_fields or bulk updates.' },
  { id: 'django_update_single_in_loop', description: 'Django code updates one row at a time inside loops instead of using bulk or set-based updates.' },
  { id: 'django_values_vs_full_model_in_loop', description: 'Django loops hydrate full model instances where values(), values_list(), or only() would likely be cheaper.' },
  { id: 'fastapi_background_task_exception_silent', description: 'FastAPI background task dispatches appear to rely on default exception behavior without visible error handling.' },
  { id: 'fastapi_dependency_creates_client_per_request', description: 'FastAPI dependencies construct HTTP clients per request instead of using app lifespan or shared setup.' },
  { id: 'fastapi_response_model_without_orm_mode', description: 'FastAPI response models use ORM conversion paths without visible ORM compatibility configuration.' },
  { id: 'fastapi_sync_def_with_blocking_io', description: 'FastAPI sync route handlers perform blocking I/O instead of using async routes or executor offload.' },
  { id: 'flask_app_config_read_per_request', description: 'Flask views repeatedly read app.config on request paths instead of consuming bootstrapped settings.' },
  { id: 'flask_debug_mode_in_production_code', description: 'Flask code enables debug=True in application runtime paths.' },
  { id: 'flask_file_read_per_request', description: 'Flask views read files directly on request paths instead of using cached or static responses.' },
  { id: 'flask_global_db_connection_per_request', description: 'Flask views create database connections per request instead of using pooled or app-scoped access.' },
  { id: 'flask_json_encoder_per_request', description: 'Flask views instantiate JSON encoders per request instead of reusing app-level serialization setup.' },
  { id: 'flask_no_streaming_for_large_response', description: 'Flask views build large in-memory responses where generator or streaming responses would fit better.' },
  { id: 'flask_request_body_parsed_multiple_times', description: 'Flask request handlers parse the same request body multiple times.' },
  { id: 'flask_template_rendered_from_string_in_view', description: 'Flask views render templates from inline strings instead of using file-backed templates.' },
  { id: 'large_dict_literal_response_in_handler', description: 'Handlers build large inline dict responses where typed response models would be clearer and cheaper.' },
  { id: 'middleware_compiles_regex_per_request', description: 'Middleware compiles regex patterns per request instead of precompiling them.' },
  { id: 'middleware_creates_http_client_per_request', description: 'Middleware creates HTTP clients per request instead of reusing app-scoped clients.' },
  { id: 'middleware_loads_config_file_per_request', description: 'Middleware loads config files on request paths instead of using startup configuration.' },
  { id: 'pydantic_model_dump_then_json_dumps', description: 'Pydantic v2 code serializes model_dump() output through json.dumps(...) instead of using model_dump_json().' },
  { id: 'pydantic_model_validate_after_json_loads', description: 'Pydantic v2 validation is preceded by json.loads(...) even though model_validate_json() could validate raw JSON directly.' },
  { id: 'response_json_dumps_then_response_object', description: 'Handlers manually json.dumps(...) payloads and then wrap them in framework Response objects.' },
  { id: 'sqlalchemy_commit_per_row_in_loop', description: 'SQLAlchemy sessions commit inside loops instead of batching changes and committing once.' },
  { id: 'sqlalchemy_create_engine_per_request', description: 'SQLAlchemy engines are created on request or handler paths instead of being process-scoped.' },
  { id: 'sqlalchemy_expire_on_commit_default_in_async', description: 'Async SQLAlchemy sessions rely on the default expire_on_commit behavior instead of making the async access pattern explicit.' },
  { id: 'sqlalchemy_n_plus_one_lazy_load', description: 'SQLAlchemy query shapes suggest lazy-loaded N+1 access with no visible eager loading.' },
  { id: 'sqlalchemy_query_in_loop', description: 'SQLAlchemy code issues queries inside loops instead of batching or prefetching.' },
  { id: 'sqlalchemy_session_not_closed', description: 'SQLAlchemy Session objects are created without context-manager or close handling.' },
  { id: 'sqlmodel_commit_per_row_in_loop', description: 'SQLModel sessions commit inside loops instead of applying one transaction after batched updates.' },
  { id: 'sqlmodel_session_exec_in_loop', description: 'SQLModel Session.exec(...) is called inside loops instead of combining the query shape.' },
  { id: 'sqlmodel_unbounded_select_in_handler', description: 'Handlers execute SQLModel select().all() paths without visible limits or pagination.' },
  { id: 'template_render_in_loop', description: 'Template rendering appears inside loops instead of rendering once over prepared data.' },
  { id: 'upstream_call_without_timeout_in_handler', description: 'Request handlers issue upstream HTTP calls without visible timeout configuration.' },
  { id: 'upstream_http_call_per_item_in_handler', description: 'Request handlers make sequential upstream HTTP calls inside loops instead of batching or bounded concurrency.' },
  { id: 'upstream_response_not_checked_before_decode', description: 'Handlers decode upstream responses without visible status checks such as raise_for_status() or status_code guards.' },
  { id: 'append_then_sort_each_iteration', description: 'A collection is appended to and then sorted on each iteration instead of sorting once after accumulation.' },
  { id: 'csv_writer_flush_per_row', description: 'csv.Writer flushes on each row instead of buffering a larger batch.' },
  { id: 'filter_then_count_then_iterate', description: 'The same collection is traversed repeatedly for filtering, counting, and later iteration.' },
  { id: 'json_encoder_recreated_per_item', description: 'A JSON encoder object is recreated per item instead of being reused for the stream.' },
  { id: 'dict_items_or_keys_materialized_in_loop', description: 'dict.items(), keys(), or values() are repeatedly materialized inside loops.' },
  { id: 'enumerate_on_range_len', description: 'enumerate(range(len(...))) style loops that add indexing ceremony without extra value.' },
  { id: 'in_check_on_list_literal', description: 'Membership tests against list literals where a tuple or set would be clearer or cheaper.' },
  { id: 'list_comprehension_only_for_length', description: 'A list comprehension is built only so len(...) can be called on it.' },
  { id: 'read_then_splitlines', description: 'File contents are fully read and then splitlines() is called instead of streaming lines.' },
  { id: 'readlines_then_iterate', description: 'readlines() materializes the whole file before line-by-line iteration.' },
  { id: 'regex_compile_in_hot_path', description: 're.compile(...) or similar regex compilation repeated inside hot code paths.' },
  { id: 'repeated_open_same_file_in_function', description: 'The same file appears to be opened multiple times within one function.' },
  { id: 'sorted_only_for_first_element', description: 'A sequence is fully sorted even though only the first or smallest element is used.' },
  { id: 'string_startswith_endswith_chain', description: 'Repeated startswith(...) or endswith(...) checks that can often be combined into tuple-based calls.' },
  { id: 'write_without_buffering_in_loop', description: 'Repeated writes inside loops with no visible buffering or batching.' },
  { id: 'concatenation_in_comprehension_body', description: 'String or collection concatenation happens inside a comprehension body, creating avoidable churn.' },
  { id: 'gzip_open_per_chunk', description: 'gzip open/create calls are repeated per chunk instead of per stream.' },
  { id: 'nested_list_search_map_candidate', description: 'Nested linear list searches that look like they want a temporary map or set index.' },
  { id: 'pickle_dumps_in_loop_same_structure', description: 'pickle.dumps(...) is called repeatedly for the same structural shape in a loop.' },
  { id: 'repeated_datetime_strptime_same_format', description: 'datetime.strptime(...) is repeated with the same format string instead of reusing a parsed shape or preprocessing once.' },
  { id: 'repeated_dict_get_same_key_no_cache', description: 'The same dictionary key is fetched repeatedly instead of storing the value in a local binding.' },
  { id: 'repeated_hashlib_new_same_algorithm', description: 'The same hashing algorithm is repeatedly constructed in a loop or tight path.' },
  { id: 'repeated_isinstance_chain_same_object', description: 'The same object goes through repeated isinstance(...) checks that could be consolidated.' },
  { id: 'repeated_list_index_lookup', description: 'The same list index lookup is performed repeatedly instead of caching the accessed value.' },
  { id: 'repeated_string_format_invariant_template', description: 'An invariant string template is formatted repeatedly in a loop instead of being partially precomputed.' },
  { id: 'sort_then_first_or_membership_only', description: 'A collection is sorted even though only the first element or a membership-style check is needed.' },
  { id: 'string_join_without_generator', description: 'String joins that materialize an unnecessary list comprehension instead of using a generator or direct iterable.' },
  { id: 'tuple_unpacking_in_tight_loop', description: 'Tuple unpacking is repeated in tight loops where reducing per-iteration overhead may help.' },
  { id: 'xml_parse_same_payload_multiple_times', description: 'The same XML payload is parsed repeatedly within one function.' },
  { id: 'yaml_load_same_payload_multiple_times', description: 'The same YAML payload is parsed repeatedly within one function.' },
  { id: 'broad_exception_handler', description: 'Broad except Exception: style handlers that still obscure failure shape even when not fully swallowed.' },
  { id: 'builtin_reduction_candidate', description: 'Loop shapes that look like obvious sum, any, or all candidates.' },
  { id: 'commented_out_code', description: 'Blocks of commented-out source code left in production files.' },
  { id: 'environment_boundary_without_fallback', description: 'Environment-variable lookups that omit a default value or explicit failure handler.' },
  { id: 'eval_exec_usage', description: 'Direct eval() or exec() usage in non-test Python code.' },
  { id: 'exception_swallowed', description: 'Broad exception handlers like except: or except Exception: that immediately suppress the error with pass, continue, break, or return.' },
  { id: 'external_input_without_validation', description: 'Request or CLI entry points that trust external input without obvious validation or guard checks.' },
  { id: 'hardcoded_business_rule', description: 'Hardcoded threshold, rate-limit, or pricing-style literals assigned inside non-test Python functions.' },
  { id: 'hardcoded_path_string', description: 'Hardcoded filesystem path literals assigned inside non-test Python functions.' },
  { id: 'magic_value_branching', description: 'Repeated branch-shaping numeric or string literals that likely want an explicit constant or policy name.' },
  { id: 'missing_context_manager', description: 'Resource management (files, network connections) inside non-test Python functions that omits with-statement context managers.' },
  { id: 'mixed_sync_async_module', description: 'Modules that expose public sync and async entry points together.' },
  { id: 'network_boundary_without_timeout', description: 'Request, sync, or job-style Python functions that call HTTP boundaries with no obvious timeout or retry policy.' },
  { id: 'none_comparison', description: '== None or != None checks instead of is None or is not None.' },
  { id: 'print_debugging_leftover', description: 'print() calls left in non-test Python functions that do not look like obvious main-entrypoint output.' },
  { id: 'public_api_missing_type_hints', description: 'Public Python functions that omit complete parameter or return annotations.' },
  { id: 'redundant_return_none', description: 'Explicit return None in simple code paths where Python would already return None implicitly.' },
  { id: 'reinvented_utility', description: 'Obvious locally implemented utility helpers that overlap with already-imported standard-library style helpers.' },
  { id: 'side_effect_comprehension', description: 'List, set, or dicit comprehensions used as standalone statements where the result is discarded.' },
  { id: 'variadic_public_api', description: 'Public Python functions that expose *args or **kwargs instead of a clearer interface.' },
  { id: 'blocking_sync_io_in_async', description: 'Synchronous network, subprocess, sleep, or file I/O calls made from async def functions.' },
  { id: 'deque_candidate_queue', description: 'Queue-style list operations like pop(0) or insert(0, ...) that may want collections.deque.' },
  { id: 'full_dataset_load', description: 'Calls that load an entire payload into memory instead of streaming.' },
  { id: 'list_materialization_first_element', description: 'list(...)[0] style access that materializes a whole list just to read the first element.' },
  { id: 'list_membership_in_loop', description: 'Repeated membership checks against obviously list-like containers inside loops.' },
  { id: 'recursive_traversal_risk', description: 'Direct recursion in traversal-style helpers that may be safer as iterative walks for deep inputs.' },
  { id: 'repeated_len_in_loop', description: 'Repeated len(...) checks inside loops when the receiver appears unchanged locally.' },
  { id: 'string_concat_in_loop', description: 'Repeated string concatenation inside loops can create O(n^2) growth and extra allocations.' },
  { id: 'temporary_collection_in_loop', description: 'Loop-local list, dict, or set construction that likely adds avoidable allocation churn.' },
  { id: 'async_lock_held_across_await', description: 'Async lock scopes or explicit acquire/release regions that continue across unrelated await points.' },
  { id: 'async_retry_sleep_without_backoff', description: 'Retry-style async loops that sleep a fixed interval without visible backoff, jitter, or bounded retry policy.' },
  { id: 'background_task_exception_unobserved', description: 'Background task bindings with no obvious await, callback, supervisor, or observation path.' },
  { id: 'dataclass_heavy_post_init', description: 'Dataclass __post_init__ methods that perform I/O, subprocess, network, or heavyweight client setup.' },
  { id: 'dataclass_mutable_default', description: 'Dataclass fields that use mutable defaults instead of default_factory.' },
  { id: 'import_time_config_load', description: 'Module-scope configuration or secret loading that runs during import instead of an explicit startup path.' },
  { id: 'import_time_file_io', description: 'Module-scope file reads, writes, or directory scans that happen during import.' },
  { id: 'import_time_network_call', description: 'Module-scope HTTP or socket calls executed while the module is imported.' },
  { id: 'import_time_subprocess', description: 'Subprocess launches triggered from module scope during import.' },
  { id: 'module_singleton_client_side_effect', description: 'Eagerly constructed network, database, or cloud clients bound at module scope.' },
  { id: 'mutable_default_argument', description: 'Function parameters that use mutable defaults such as [], {}, or set() directly in the signature.' },
  { id: 'mutable_module_global_state', description: 'Mutable module globals updated from multiple functions.' },
  { id: 'option_bag_model', description: 'Dataclass or TypedDict models that accumulate many optional fields and boolean switches.' },
  { id: 'pickle_deserialization_boundary', description: 'pickle.load(s) or dill.load(s) style deserialization in production code.' },
  { id: 'public_any_type_leak', description: 'Public functions or model fields that expose Any, object, or similarly wide contracts.' },
  { id: 'subprocess_shell_true', description: 'Subprocess boundaries that enable shell=True.' },
  { id: 'tar_extractall_unfiltered', description: 'tarfile.extractall(...) without an obvious filter, members list, or path-validation helper.' },
  { id: 'tempfile_without_cleanup', description: 'Temporary files or directories created without a visible cleanup or context-manager ownership path.' },
  { id: 'typeddict_unchecked_access', description: 'Direct indexing of optional TypedDict keys without an obvious guard path.' },
  { id: 'unsafe_yaml_loader', description: 'yaml.load(...) or full_load(...) style loaders used where safe loading is more appropriate.' },
  { id: 'untracked_asyncio_task', description: 'asyncio.create_task(...) or similar task creation whose handle is discarded immediately.' },
  { id: 'deep_inheritance_hierarchy', description: 'Repository-local Python class chains with unusually deep inheritance depth.' },
  { id: 'eager_constructor_collaborators', description: 'Constructors that instantiate several collaborators eagerly inside __init__.' },
  { id: 'god_class', description: 'Python classes that concentrate unusually high method count, public surface area, and mutable instance state.' },
  { id: 'god_function', description: 'Very large Python functions with high control-flow and call-surface concentration.' },
  { id: 'mixed_concerns_function', description: 'Functions that mix HTTP, persistence, and filesystem-style concerns in one body.' },
  { id: 'monolithic_init_module', description: '__init__.py files that carry enough imports and behavior to look like monolithic modules.' },
  { id: 'monolithic_module', description: 'Non-__init__.py modules that are unusually large and combine many imports with orchestration-heavy behavior.' },
  { id: 'name_responsibility_mismatch', description: 'Read-style, transformation-style, or utility-style names that still perform mutation or own multiple infrastructure concerns.' },
  { id: 'over_abstracted_wrapper', description: 'Ceremonial wrapper-style or tiny data-container classes that add little beyond storing constructor state.' },
  { id: 'tight_module_coupling', description: 'Modules that depend on a large number of repository-local Python modules.' },
  { id: 'too_many_instance_attributes', description: 'Classes that assign an unusually large number of instance attributes across their methods.' },
]

const rustRules: Rule[] = [
  { id: 'rust_arc_mutex_option_state', description: 'Arc<Mutex<Option<T>>>-style state bags that hide lifecycle state behind nested mutation layers.' },
  { id: 'rust_boolean_state_machine', description: 'State structs encoded with multiple booleans instead of a dedicated enum.' },
  { id: 'rust_borrowed_pathbuf_api', description: 'Public signatures that borrow &PathBuf instead of &Path.' },
  { id: 'rust_borrowed_string_api', description: 'Public signatures that borrow &String instead of &str.' },
  { id: 'rust_borrowed_vec_api', description: 'Public signatures that borrow &Vec<T> instead of &[T].' },
  { id: 'rust_builder_without_validate', description: 'Builders that expose build() without an obvious validation step.' },
  { id: 'rust_constructor_many_flags', description: 'Constructor-like APIs that use multiple boolean flags.' },
  { id: 'rust_global_lock_state', description: 'Static or lazy global state wrapped in mutable lock-based containers.' },
  { id: 'rust_mutex_wrapped_collection', description: 'Collection-plus-lock fields embedded directly in public or central state structs.' },
  { id: 'rust_option_bag_config', description: 'Config-like structs with many Option fields and no obvious validation path.' },
  { id: 'rust_partial_init_escape', description: 'Constructor-like functions that return or store partially initialized structs.' },
  { id: 'rust_pub_interior_mutability_field', description: 'Public structs that expose Mutex, RwLock, RefCell, Cell, or similar fields directly.' },
  { id: 'rust_public_anyhow_result', description: 'Public library-facing APIs that return anyhow-style Result types instead of a clearer domain error.' },
  { id: 'rust_public_bool_parameter_api', description: 'Public APIs that expose a raw boolean mode switch.' },
  { id: 'rust_public_box_dyn_error', description: 'Public APIs that expose Box<dyn Error> rather than a clearer error surface.' },
  { id: 'rust_rc_refcell_domain_model', description: 'Domain-style structs built around Rc<RefCell<T>> instead of clearer ownership boundaries.' },
  { id: 'rust_serde_default_on_required_field', description: 'Required-looking contract fields that opt into #[serde(default)].' },
  { id: 'rust_serde_flatten_catchall', description: '#[serde(flatten)] catch-all maps or loose value bags that absorb unknown fields.' },
  { id: 'rust_serde_unknown_fields_allowed', description: 'Strict-looking config or request structs that deserialize without deny_unknown_fields.' },
  { id: 'rust_serde_untagged_enum_boundary', description: 'Boundary-facing enums that derive #[serde(untagged)] and risk ambiguous wire formats.' },
  { id: 'rust_stringly_typed_enum_boundary', description: 'Enum-like boundary fields kept as String instead of a dedicated enum.' },
  { id: 'rust_async_blocking_drop', description: 'A Drop implementation does blocking work that can surface in async contexts.' },
  { id: 'rust_async_hold_permit_across_await', description: 'A permit or pooled resource may be held across an .await.' },
  { id: 'rust_async_invariant_broken_at_await', description: 'Related state mutations appear split around an await boundary.' },
  { id: 'rust_async_lock_order_cycle', description: 'Conflicting lock acquisition order suggests a lock-order cycle.' },
  { id: 'rust_async_missing_fuse_pin', description: 'select! reuse lacks pinning or fusing markers for repeated polling.' },
  { id: 'rust_async_monopolize_executor', description: 'An async function may monopolize the executor with blocking work and no await.' },
  { id: 'rust_async_recreate_future_in_select', description: 'A select! loop may recreate futures instead of reusing long-lived ones.' },
  { id: 'rust_async_spawn_cancel_at_await', description: 'Async work is spawned without an obvious cancellation path.' },
  { id: 'rust_async_std_mutex_await', description: 'std::sync::Mutex appears to be held across .await in async code.' },
  { id: 'rust_debug_secret', description: 'Debug is derived on a type that carries secret-like fields.' },
  { id: 'rust_domain_default_produces_invalid', description: 'Default is derived or implemented on a type that likely cannot have a safe default state.' },
  { id: 'rust_domain_float_for_money', description: 'Floating-point storage is used for money-like values.' },
  { id: 'rust_domain_impossible_combination', description: 'A boolean toggle is mixed with optional credentials, creating invalid-state combinations.' },
  { id: 'rust_domain_optional_secret_default', description: 'A defaultable type includes optional secret-like fields, which can hide invalid configuration.' },
  { id: 'rust_domain_raw_primitive', description: 'Business-facing data is stored as a raw primitive instead of a stronger domain type.' },
  { id: 'rust_serde_sensitive_deserialize', description: 'Deserialize is derived for sensitive fields without obvious validation.' },
  { id: 'rust_serde_sensitive_serialize', description: 'Serialize is derived for secret-like fields that may need redaction or exclusion.' },
  { id: 'dbg_macro_leftover', description: 'dbg!() left in non-test Rust code.' },
  { id: 'expect_in_non_test_code', description: '.expect(...) used in non-test Rust code.' },
  { id: 'fixme_doc_comment_leftover', description: 'Rust doc comments that still contain a FIXME marker in non-test code.' },
  { id: 'hack_doc_comment_leftover', description: 'Rust doc comments that still contain a HACK marker in non-test code.' },
  { id: 'panic_macro_leftover', description: 'panic!() left in non-test Rust code.' },
  { id: 'todo_doc_comment_leftover', description: 'Rust doc comments that still contain a TODO marker in non-test code.' },
  { id: 'todo_macro_leftover', description: 'todo!() left in non-test Rust code.' },
  { id: 'unimplemented_macro_leftover', description: 'unimplemented!() left in non-test Rust code.' },
  { id: 'unreachable_macro_leftover', description: 'unreachable!() left in non-test Rust code.' },
  { id: 'unsafe_without_safety_comment', description: 'unsafe fn or unsafe block without a nearby SAFETY: comment within the previous two lines.' },
  { id: 'unwrap_in_non_test_code', description: '.unwrap() used in non-test Rust code.' },
  { id: 'rust_aos_hot_path', description: 'Repeated struct-field dereferences inside a loop that may indicate an array-of-structs hot path.' },
  { id: 'rust_blocking_drop', description: 'A Drop implementation performs blocking work.' },
  { id: 'rust_blocking_io_in_async', description: 'Blocking I/O or blocking work observed in async Rust code.' },
  { id: 'rust_hashmap_default_hasher', description: 'HashMap default-hasher construction in a likely hot path.' },
  { id: 'rust_large_future_stack', description: 'Large allocations may be captured across await points and bloat future size.' },
  { id: 'rust_lines_allocate_per_line', description: '.lines() iteration used in a loop where per-item allocation may matter.' },
  { id: 'rust_lock_across_await', description: 'A lock appears to be held across an .await boundary.' },
  { id: 'rust_path_join_absolute', description: 'Path::join used with an absolute segment that discards the existing base path.' },
  { id: 'rust_pointer_chasing_vec_box', description: 'Pointer-heavy boxed vector-style storage that may hurt cache locality.' },
  { id: 'rust_tokio_mutex_unnecessary', description: 'tokio::sync::Mutex used in a fully synchronous critical path with no await.' },
  { id: 'rust_unbuffered_file_writes', description: 'File-like writes performed inside a loop without buffering or batching.' },
  { id: 'rust_utf8_validate_hot_path', description: 'UTF-8 validation appears in a likely hot path and may deserve profiling.' },
  { id: 'rust_unsafe_aliasing_assumption', description: 'Unsafe code mixes interior mutability and mutable references in ways that need careful aliasing review.' },
  { id: 'rust_unsafe_assume_init', description: 'Unsafe MaybeUninit::assume_init use without proof of full initialization.' },
  { id: 'rust_unsafe_from_raw_parts', description: 'Unsafe raw slice construction that depends on lifetime and length invariants.' },
  { id: 'rust_unsafe_get_unchecked', description: 'Unsafe use of get_unchecked without proof of bounds invariants.' },
  { id: 'rust_unsafe_raw_pointer_cast', description: 'Unsafe raw pointer cast that depends on aliasing and lifetime guarantees.' },
  { id: 'rust_unsafe_set_len', description: 'Unsafe Vec::set_len use that requires initialized elements and correct capacity invariants.' },
  { id: 'rust_unsafe_transmute', description: 'Unsafe transmute use that requires layout and validity proof.' },
]
// GENERATED_RULES_END

// ─── CLI commands by language ─────────────────────────────────────────────────

const cliCommands: Record<Language, CliCommand[]> = {
  go: [
    { cmd: 'cargo run -- scan /path/to/repo', desc: 'Scan a Go repository and print a compact finding summary.' },
    { cmd: 'cargo run -- scan --details /path/to/repo', desc: 'Include full per-function fingerprint details and detail-only findings.' },
    { cmd: 'cargo run -- scan --json /path/to/repo', desc: 'Emit structured JSON output for pipeline integration.' },
    { cmd: 'cargo run -- scan --json --details /path/to/repo', desc: 'Combine JSON and full detail output.' },
    { cmd: 'cargo run -- scan --enable-semantic /path/to/repo', desc: 'Enable the opt-in deeper semantic Go pack for nested-loop allocation, concat, and stronger N+1 correlation.' },
    { cmd: 'cargo run -- scan --ignore dropped_error,panic_on_error /path/to/repo', desc: 'Ignore selected Go rule IDs for one run without changing repository config.' },
    { cmd: 'cargo run -- scan /path/to/repo > results.txt', desc: 'Write the text report directly to a file.' },
    { cmd: 'cargo run -- scan --no-ignore /path/to/repo', desc: 'Scan without .gitignore filtering.' },
    { cmd: 'cargo run -- bench /path/to/repo', desc: 'Benchmark the full pipeline against a local repository.' },
    { cmd: 'cargo run -- bench --enable-semantic /path/to/repo', desc: 'Benchmark the Go pipeline with the opt-in semantic pack enabled.' },
    { cmd: 'cargo run -- bench --warmups 2 --repeats 5 /path/to/repo', desc: 'Benchmark with explicit warmup and repeat counts.' },
    { cmd: 'cargo run -- bench --json /path/to/repo', desc: 'Emit benchmarking data as JSON.' },
  ],
  python: [
    { cmd: 'cargo run -- scan /path/to/repo', desc: 'Auto-detect and scan Python files alongside any Go or Rust files in the repository.' },
    { cmd: 'cargo run -- scan --details /path/to/repo', desc: 'Include full Python per-function fingerprint breakdown.' },
    { cmd: 'cargo run -- scan --json /path/to/repo', desc: 'Emit findings for Python files in structured JSON.' },
    { cmd: 'cargo run -- scan --json --details /path/to/repo', desc: 'Combine JSON output with full Python detail-only diagnostics.' },
    { cmd: 'cargo run -- scan --ignore exception_swallowed,print_debugging_leftover /path/to/repo', desc: 'Ignore selected Python rule IDs for one run without changing repository config.' },
    { cmd: 'cargo run -- scan /path/to/repo > results.txt', desc: 'Save the Python scan report to a file for review.' },
    { cmd: 'cargo run -- scan --no-ignore /path/to/repo', desc: 'Override .gitignore filtering when scanning Python projects.' },
    { cmd: 'cargo run -- bench /path/to/repo', desc: 'Benchmark discovery, parse, index, heuristic, and total runtime stages for a Python-heavy repository.' },
    { cmd: 'cargo run -- bench --warmups 2 --repeats 5 /path/to/repo', desc: 'Benchmark Python scans with explicit warmup and repeat counts.' },
    { cmd: 'cargo run -- bench --json /path/to/repo', desc: 'Emit benchmarking data as JSON for CI or local comparisons.' },
  ],
  rust: [
    { cmd: 'cargo run -- scan /path/to/repo', desc: 'Auto-detect and scan Rust files in the repository using the Rust rule pack.' },
    { cmd: 'cargo run -- scan --details /path/to/repo', desc: 'Include full Rust per-function fingerprint details.' },
    { cmd: 'cargo run -- scan --json /path/to/repo', desc: 'Emit Rust findings in structured JSON.' },
    { cmd: 'cargo run -- scan --json --details /path/to/repo', desc: 'Combine JSON output with full Rust detail-only diagnostics.' },
    { cmd: 'cargo run -- scan --ignore rust_async_std_mutex_await,rust_lock_across_await /path/to/repo', desc: 'Ignore specific rule IDs for one scan invocation without changing repository config.' },
    { cmd: 'cargo run -- scan /path/to/repo > results.txt', desc: 'Save the Rust scan report to a file.' },
    { cmd: 'cargo run -- scan --no-ignore /path/to/repo', desc: 'Override .gitignore filtering when scanning Rust projects.' },
    { cmd: 'cargo run -- bench /path/to/repo', desc: 'Benchmark discovery, parse, index, heuristic, and total runtime stages.' },
    { cmd: 'cargo run -- bench --warmups 2 --repeats 5 /path/to/repo', desc: 'Benchmark with explicit warmup and repeat counts.' },
    { cmd: 'cargo run -- bench --json /path/to/repo', desc: 'Emit benchmarking data as JSON.' },
  ],
  common: [
    { cmd: 'cargo run -- scan /path/to/repo', desc: 'Scan any supported repository to trigger the shared heuristic layer.' },
    { cmd: 'cargo run -- scan --ignore hardcoded_secret,generic_name /path/to/repo', desc: 'Ignore specific shared rules for the current scan.' },
  ],
}

// GENERATED_ACTION_INPUTS_START
const githubActionInputs: GitHubActionInput[] = [
  { name: 'version', description: 'Release tag to install, for example v0.1.0. Defaults to the current action ref when it is a full release tag, otherwise latest. Optional.' },
  { name: 'command', description: 'Subcommand to run. Supported values are scan and bench. Defaults to scan. Optional.' },
  { name: 'path', description: 'Path to the repository to analyze. Defaults to .. Optional.' },
  { name: 'json', description: 'Emit JSON output. Defaults to false. Optional.' },
  { name: 'details', description: 'Include detail-only scan findings. Applies only to the scan command. Defaults to false. Optional.' },
  { name: 'no-ignore', description: 'Scan without respecting .gitignore. Defaults to false. Optional.' },
  { name: 'enable-semantic', description: 'Enable the opt-in deeper semantic Go heuristics. Defaults to false. Optional.' },
  { name: 'fail-on-findings', description: 'Exit with a non-zero status code when scan findings are present. Applies only to the scan command. Defaults to true. Optional.' },
  { name: 'repeats', description: 'Benchmark repeat count. Applies only to the bench command. Defaults to 5. Optional.' },
  { name: 'warmups', description: 'Benchmark warmup count. Applies only to the bench command. Defaults to 1. Optional.' },
]
// GENERATED_ACTION_INPUTS_END

// GENERATED_ACTION_EXAMPLES_START
const githubActionWorkflow = `name: Deslop

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ${currentRelease.actionRef}
        with:
          path: .`

const githubActionJsonExample = `- uses: actions/checkout@v4
- uses: ${currentRelease.actionRef}
  with:
    path: .
    json: 'true'
    details: 'true'
    fail-on-findings: 'false'`

const githubActionBenchExample = `- uses: actions/checkout@v4
- uses: ${currentRelease.actionRef}
  with:
    command: bench
    path: .
    repeats: '10'
    warmups: '2'`
// GENERATED_ACTION_EXAMPLES_END

const repositoryConfigExample = `go_semantic_experimental = true
rust_async_experimental = true
disabled_rules = ["panic_macro_leftover"]
suppressed_paths = ["tests/fixtures"]

[severity_overrides]
unwrap_in_non_test_code = "error"
missing_context_propagation = "error"`

const overviewContent = {
  go: {
    title: 'Go Analysis',
    lead: 'deslop ships its broadest heuristic surface area for Go. It walks the repository with .gitignore awareness, parses source structure with tree-sitter-go, builds a local package index keyed by package plus directory, and now covers repo-wide style checks, receiver-field and nested wrapper propagation, derived-context goroutine lifetime analysis, parser-backed Gin and GORM request-path performance analysis, duplicate decode hot spots, multipart upload waste, repeated split and strconv churn, scratch container and slice-clone churn, loop-local URL and time parsing, dynamic Gin bind churn, request-path DB churn, looped GORM CRUD and association churn, body-rewind duplication, large dynamic map response construction, export streaming gaps, debug-body logging on hot routes, upstream response decode duplication, handler-driven DB write batch gaps, non-request workload suppression for missing-limit findings, security, and an opt-in deeper semantic loop pass with explainable rules.',
    bullets: [
      '.gitignore-aware walk; skips vendor/ and generated files by default',
      'Parses package names, imports, declared symbols, call sites, and function fingerprints',
      'Builds a repository-local symbol index for same-package and import hallucination checks',
      'Includes repo-wide package and import style checks plus wrapper-level context propagation heuristics',
      'Covers receiver-field wrappers, local wrapper chains, and Query versus QueryContext mismatches',
      'Adds parser-backed Gin request-body, multipart upload, and query-bind summaries plus GORM chain summaries for duplicate decode, file-serving, request-path DB churn, body-rewind detection, large dynamic map response detection, export streaming gaps, debug-body logging, upstream decode duplication, handler-driven batch gaps, and repeated local transform findings',
      'Adds an opt-in semantic pack for nested-loop allocation, concat, and stronger N+1 signals',
      'Produces compact text output by default; full detail and JSON via flags',
      'Supports standalone Go repos and mixed Go + Python + Rust repositories',
    ],
  },
  python: {
    title: 'Python Analysis',
    lead: 'Python support now spans function-level, file-level, and repo-level heuristics backed by fixture-driven parser evidence. The shipped rule pack covers async boundary misuse, exception handling, duplication families, class and module structure smells, AI-style commentary signals, and repository-local hallucination checks without crossing language boundaries in mixed repos.',
    bullets: [
      'Parses .py files with tree-sitter-python alongside Go and Rust files',
      'Extracts imports, declared symbols, call sites, docstrings, class summaries, and phase-4 evidence used by maintainability and structure rules',
      'Runs 60-plus Python-specific and shared heuristics across function, file, and repository scopes',
      'Language-scoped local index prevents symbol cross-contamination in mixed repos and powers Python hallucination checks',
      'Fixture-backed parser and integration coverage keeps rule families stable as the Python surface area grows',
      'Conservative about flagging policy: favors lower false-positive rates over exhaustive coverage',
    ],
  },
  rust: {
    title: 'Rust Analysis',
    lead: 'Rust support now covers hygiene leftovers, crate-local hallucination checks, async/runtime hazards, performance smells, domain-modeling anti-patterns, and unsafe-soundness hot spots. The Rust rule pack runs on the same fast pipeline as Go and Python: tree-sitter parsing, a language-scoped local index, and explainable heuristic output.',
    bullets: [
      'Parses .rs files with tree-sitter-rust',
      'Detects leftovers, unsafe hygiene, crate-local hallucinations, async runtime hazards, and domain-modeling issues',
      'Flags unsafe blocks and functions without a nearby SAFETY: comment plus soundness-sensitive unsafe operations',
      'Covers crate::, self::, and super:: import-call hallucinations via a local Rust module index',
      'Adds Rust-specific async, performance, and domain-modeling heuristics without crossing language boundaries',
      'Language-scoped index prevents symbol merging with Go or Python in mixed repositories',
    ],
  },
  common: {
    title: 'Shared Heuristics',
    lead: 'deslop includes a core layer of cross-language heuristics that apply regardless of the specific source language. These rules focus on naming quality, documentation hygiene, repository-local hallucination checks, and common anti-patterns like hardcoded secrets or unbuffered I/O.',
    bullets: [
      'Universal naming quality checks (generic, overlong, or weak identifiers)',
      'Language-scoped local index for internal symbol hallucination detection',
      'Documentation-style and commentary hygiene checks',
      'Basic test quality and placeholder detection',
      'Common security smells like hardcoded secrets',
      'General performance signals like unbuffered full-dataset loads',
    ],
  },
}

const pipelineStages = [
  {
    name: 'Discover',
    summary: 'Walk the repository with .gitignore awareness. Skip vendor/ and known generated-code paths. Keep file selection independent from later analysis.',
    detail: 'Discovery runs before any parsing so the pipeline stays composable. Supported file extensions are routed to the correct language backend. The --no-ignore flag disables .gitignore filtering when needed.',
  },
  {
    name: 'Parse',
    summary: 'Parse source structure, declared symbols, and call patterns using tree-sitter grammars without forcing a heavy semantic stack.',
    detail: 'Go files are parsed with tree-sitter-go, Python files with tree-sitter-python, and Rust files with tree-sitter-rust. The parser is syntax-tolerant: even files with errors will still yield partial structure for downstream heuristics.',
  },
  {
    name: 'Index',
    summary: 'Build a lightweight repository-local symbol index keyed by package plus directory. Scope the index per language for mixed repositories.',
    detail: 'The index is intentionally modest — it improves same-package and import-qualified call checks without pretending to replace full type analysis. In mixed-language repos, Go, Python, and Rust symbols are tracked separately so hallucination checks stay correct.',
  },
  {
    name: 'Heuristics',
    summary: 'Run explainable rule families that emit rule IDs, severity, messages, and evidence. Hold detail-only diagnostics back from default output.',
    detail: 'Each finding includes a rule ID, severity level, file path, line number, and an evidence payload written for human review. The --details flag adds full per-function fingerprint breakdowns. JSON output is available for pipeline integration.',
  },
]

const limitations = {
  go: [
    'No authoritative Go type checking. Heuristics use structural patterns, not go/types.',
    'No full interprocedural or type-aware context propagation. Wrapper-chain reasoning stays repository-local and conservative.',
    'No proof of goroutine leaks, N+1 queries, or runtime performance regressions — only pattern signals.',
    'Package-method and local-symbol checks are repository-local; external packages are not indexed.',
    'The opt-in deeper semantic Go pack is still heuristic: it correlates nested-loop structure but does not prove asymptotic complexity or schema-aware DB cost.',
  ],
  python: [
    'No Python module graph resolution or installed-package awareness.',
    'No authoritative Python type analysis — hints are structural and conservative.',
    'No interprocedural propagation. Checks are local to individual functions or files.',
    'No proof of runtime behavior or end-to-end asyncio correctness — async findings remain syntax-driven heuristics.',
    'Cross-file duplicate detection is conservative and normalized; it is not exhaustive pairwise semantic comparison.',
  ],
  rust: [
    'No Rust trait resolution, cargo workspace modeling, or macro expansion.',
    'Rust rule pack is still growing, but current coverage already includes hygiene, hallucination, async/runtime, performance, domain-modeling, and unsafe-soundness checks.',
    'No proof of memory safety violations or lifetime errors from static analysis alone.',
    'Hallucination checks cover crate-local imports only; external crates are not indexed.',
    'No interprocedural analysis or cross-crate symbol resolution.',
  ],
  common: [
    'Shared heuristics are strictly syntax-driven and do not perform deep interprocedural data-flow or points-to analysis.',
    'No cross-language semantic bridge (e.g., deslop does not model Go-to-Python FFI calling conventions).',
    'Naming and commentary hygiene checks are suggestive and do not account for project-specific jargon or acronyms.',
    'Hallucination checks are strictly repository-local and scoped by language to prevent false-positive cross-pollination.',
    'General performance signals like unbuffered I/O are pattern-based and do not account for OS-level buffering or hardware-specific optimizations.',
  ],
}

export {
  cliCommands,
  commonRules,
  githubActionInputs,
  githubActionBenchExample,
  githubActionJsonExample,
  githubActionWorkflow,
  goRules,
  languages,
  limitations,
  overviewContent,
  pipelineStages,
  pythonRules,
  repositoryConfigExample,
  rustRules,
  sections,
}

export type { CliCommand, GitHubActionInput, Language, NavSection, Rule, SectionId }
