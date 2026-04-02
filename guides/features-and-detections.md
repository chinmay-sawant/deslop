# deslop Features and Detections

## Purpose

deslop is a static analyzer for Go, Python, and Rust repositories that looks for signals commonly associated with low-context or AI-assisted code. The goal is not to prove correctness. The goal is to surface suspicious patterns quickly, explain why they were flagged, and let a reviewer decide whether the code is actually a problem.

## Current feature set

### Scan modes

- `cargo run -- scan <path>` prints a compact summary plus findings.
- `cargo run -- scan --ignore rule1,rule2 <path>` filters specific rule IDs for a single scan invocation after analysis completes.
- `cargo run -- scan --details <path>` prints the full per-file and per-function breakdown.
- `cargo run -- scan --json <path>` prints structured JSON.
- `cargo run -- bench <path>` benchmarks the end-to-end pipeline.

Repository-local scan behavior can also be tuned with `.deslop.toml`, including `disabled_rules`, `severity_overrides`, `suppressed_paths`, `go_semantic_experimental`, and `rust_async_experimental`.

### Repository handling

- Walks a repository with `.gitignore` awareness by default.
- Skips `vendor/` and generated Go files.
- Parses Go syntax with `tree-sitter-go`, Python syntax with `tree-sitter-python`, and Rust syntax with `tree-sitter-rust`.
- Continues scanning even when some files contain syntax errors.

### Analysis pipeline

- Extracts package names, imports, declared symbols, call sites, and function fingerprints.
- Builds a repository-local index keyed by package plus directory.
- Runs explainable heuristics that emit rule IDs, messages, and evidence.
- Produces compact text output by default, or detailed text/JSON with `--details`.

## What deslop detects today

The shipped registry currently tracks **430 language-scoped rule entries** in deslop `0.1.0`.

| Language | Stable | Experimental | Research | Total |
| --- | ---: | ---: | ---: | ---: |
| common | 11 | 0 | 0 | 11 |
| go | 181 | 2 | 0 | 183 |
| python | 162 | 0 | 0 | 162 |
| rust | 62 | 12 | 0 | 74 |

The sections below are generated from the rule registry and grouped by language and family.
When the same rule ID is implemented in more than one backend, it appears once in each relevant language section.

### Common rules (11)

#### Comments (2)
- `comment_style_title_case`: Heading-like Title Case documentation.
- `comment_style_tutorial`: Tutorial-style documentation that narrates obvious implementation steps.

#### Hallucination (2)
- `hallucinated_import_call`: Package-qualified calls that do not match locally indexed symbols for the imported package.
- `hallucinated_local_call`: Same-package calls to symbols not present in the scanned local package context.

#### Naming (3)
- `generic_name`: Function names that are overly generic without stronger contextual signals.
- `overlong_name`: Very long identifiers with too many descriptive tokens.
- `weak_typing`: Signatures that rely on any or empty interface types.

#### Security (1)
- `hardcoded_secret`: Secret-like identifiers assigned direct string literals instead of environment lookups.

#### Test Quality (3)
- `happy_path_only_test`: Tests that assert success expectations without any obvious negative-path signal.
- `placeholder_test_body`: Tests that look skipped, TODO-shaped, or otherwise placeholder-like.
- `test_without_assertion_signal`: Tests that exercise production code without an obvious assertion or failure signal.

### Go rules (183)

#### Concurrency (6)
- `blocking_call_while_locked`: Potentially blocking calls observed between Lock and Unlock.
- `goroutine_derived_context_unmanaged`: Likely long-lived goroutines launched after a derived context is created and before the matching cancel call is observed.
- `goroutine_spawn_in_loop`: Raw go statements launched from inside loops without obvious WaitGroup coordination.
- `goroutine_without_coordination`: Raw go statements without an obvious context or WaitGroup-like coordination signal.
- `goroutine_without_shutdown_path`: Looping goroutine literals without an obvious ctx.Done() or done-channel shutdown path.
- `mutex_in_loop`: Repeated Lock or RLock acquisition inside loops.

#### Consistency (3)
- `duplicate_struct_tag_key`: Struct field tags that repeat the same key more than once.
- `malformed_struct_tag`: Struct field tags that do not parse as valid Go tag key/value pairs.
- `mixed_receiver_kinds`: Methods on the same receiver type mix pointer and value receivers.

#### Context (6)
- `busy_waiting`: select { default: ... } inside loops, which often spins instead of blocking.
- `context_background_used`: Functions that already accept context.Context but still create context.Background() or context.TODO() locally.
- `missing_cancel_call`: Derived contexts where deslop cannot find a local cancel() or defer cancel() call.
- `missing_context`: Standard-library context-aware calls from functions that do not accept context.Context.
- `missing_context_propagation`: Functions that already accept context.Context but still call context-free stdlib APIs like http.Get or exec.Command.
- `sleep_polling`: time.Sleep inside loops — often indicates polling or busy-wait style code.

#### Data Access (59)
- `association_find_inside_loop`: GORM Association(...).Find(...) loaders observed inside loops.
- `automigrate_or_schema_probe_in_request_path`: AutoMigrate or schema probes running on request paths instead of startup.
- `bun_newdb_per_request`: Bun DB handles created directly on request paths instead of reused process-level state.
- `bun_select_scan_without_limit`: Bun select-and-scan request paths without a visible limit or pagination marker.
- `connection_pool_reconfigured_per_request`: DB pool sizing or lifetime settings changed on request paths.
- `count_inside_loop`: COUNT(...) or GORM Count(...) observed inside loops.
- `count_then_find_same_filter`: Request-path GORM flows that run Count(...) and then a broad Find(...) with the same filter shape.
- `create_single_in_loop_instead_of_batches`: GORM .Create(...) used inside loops with no visible CreateInBatches(...) path in the same function.
- `date_or_cast_wrapped_indexed_column`: DATE/CAST wrapping indexed columns in WHERE clauses, preventing index usage.
- `db_ping_per_request`: database Ping(...) or PingContext(...) called on request paths instead of startup or explicit health checks.
- `default_transaction_enabled_for_bulk_create`: Bulk creates performed with GORM default transaction enabled — SkipDefaultTransaction improves throughput.
- `delete_single_row_in_loop_without_batch`: GORM Delete(...) chains observed inside loops one row at a time.
- `distinct_wide_row_request_path`: Distinct on wide rows without Select projection — a key-only subquery is usually cheaper.
- `duplicate_find_then_preload_followup`: Initial Find followed by a separate Preload query that could be folded into one.
- `ent_open_per_request`: ent clients opened directly inside request handlers instead of shared startup wiring.
- `exec_inside_loop_without_batch`: Exec(...) or ExecContext(...) used for row-by-row SQL writes inside loops.
- `exists_via_count_star`: COUNT(*) used for an existence check instead of EXISTS or LIMIT 1.
- `find_all_then_manual_paginate_in_go`: All rows fetched and then sliced in Go instead of using database-level pagination.
- `findinbatches_candidate_for_large_scan`: Unbounded result sets that could use FindInBatches or cursor iteration.
- `first_or_create_in_loop`: GORM FirstOrCreate(...) chains observed inside loops.
- `gorm_debug_enabled_in_request_path`: GORM debug logging enabled on request paths.
- `gorm_find_without_limit_on_handler_path`: Request-path GORM Find(...) chains with no visible Limit(...) step.
- `gorm_joins_plus_preload_plus_find_without_limit`: GORM chains combining Joins, Preload, and unbounded Find on request paths.
- `gorm_open_per_request`: gorm.Open(...) called on request paths instead of process-level setup.
- `gorm_preload_clause_associations_on_wide_graph`: Request-path GORM chains that use Preload(clause.Associations) or other broad preload graphs.
- `gorm_select_missing_projection_on_wide_model`: GORM queries on wide models without a Select projection to limit fetched columns.
- `gorm_session_allocated_per_item`: GORM Session(...) chains allocated inside loops before issuing queries.
- `leading_wildcard_builder_chain`: LIKE queries with leading wildcards that prevent index usage.
- `lower_or_func_wrapped_indexed_column`: LOWER() or other functions wrapping indexed columns in WHERE clauses, preventing index usage.
- `many_column_or_filter_chain`: Query chains with many OR filter conditions that often scale poorly.
- `nested_transaction_in_request_path`: Multiple transactions started on a single request path.
- `offset_pagination_on_large_table`: Request-path GORM Find(...) chains that page with Offset(...), which often scales poorly on large lists.
- `order_by_random_request_path`: ORDER BY RAND()/RANDOM() observed on request paths.
- `order_by_without_limit_orm_chain`: ORM chains that order results without a Limit on request paths.
- `pgx_collectrows_unbounded_materialization`: pgx.CollectRows used on request paths without a visible LIMIT in the query.
- `pgxpool_acquire_in_loop`: Repeated pgxpool acquire calls inside loops that may churn pooled resources.
- `pgxpool_new_per_request`: pgxpool pools created on request paths instead of reused application-level state.
- `pgxpool_ping_per_request`: Per-request pgxpool connectivity probes that add latency outside health-check boundaries.
- `preload_inside_loop`: GORM Preload(...) queries configured and executed inside loops.
- `prepare_inside_loop`: Prepare(...) or PrepareContext(...) observed inside loops.
- `prepare_on_every_request_same_sql`: The same literal SQL is prepared multiple times on one request path.
- `queryrow_inside_loop_existence_check`: QueryRow(...) or QueryRowContext(...) used inside loops for point lookups that usually want a bulk prefetch path.
- `raw_scan_inside_loop`: GORM Raw(...).Scan(...) chains observed inside loops.
- `redis_client_created_per_request`: Redis clients created per request instead of reused as shared process infrastructure.
- `redis_command_loop_without_pipeline`: Redis command loops that issue round-trips without pipeline or batch usage.
- `redis_ping_per_request`: Per-request Redis ping checks instead of startup or explicit health-probe validation.
- `repeated_same_query_template_same_function`: The same query template executed multiple times in one function.
- `row_by_row_upsert_loop`: Upsert-style writes executed row by row inside loops instead of batched.
- `rows_to_struct_allocation_per_row_without_reuse`: New struct allocated for each row scan instead of reusing a scratch variable.
- `save_for_single_column_change`: GORM Save used for a single-column update instead of a targeted Update call.
- `save_in_loop_full_model`: GORM Save(...) writes full models inside loops.
- `scan_into_map_string_any_hot_path`: Rows scanned into map[string]any instead of typed structs on hot paths.
- `select_or_get_inside_loop_lookup`: ORM lookups (Select, Get, First, etc.) executed inside loops.
- `sql_open_per_request`: database/sql pools opened on request paths instead of process-level setup.
- `sqlx_select_large_slice_without_limit`: sqlx.Select used on request paths without a visible LIMIT in the query.
- `tx_begin_per_item_loop`: Transactions started inside loops instead of once around the wider batch.
- `unbounded_in_clause_expansion`: IN clauses built from request-driven collections without bound limits.
- `update_single_row_in_loop_without_batch`: GORM Update(...), UpdateColumn(...), or Updates(...) calls observed inside loops one row at a time.
- `updates_map_allocated_per_row`: GORM Updates(map[string]...) maps allocated inside loops for per-row updates.

#### Errors (3)
- `dropped_error`: Blank identifier assignments that discard an err-like value.
- `error_wrapping_misuse`: fmt.Errorf calls that reference err without %w.
- `panic_on_error`: err != nil branches that jump straight to panic or log.Fatal style exits.

#### Gin (34)
- `bindjson_into_map_any_hot_endpoint`: Gin handlers that bind JSON into map[string]any or map[string]interface{} on hot request paths.
- `bindquery_into_map_any_hot_endpoint`: Gin handlers that bind query parameters into map[string]any or map[string]interface{} on hot request paths.
- `dumprequest_or_dumpresponse_in_hot_path`: Request-path handlers that dump full HTTP requests or responses with httputil.
- `duplicate_upstream_calls_same_url_same_handler`: Same upstream URL called multiple times in one handler.
- `env_or_config_lookup_per_request`: Environment variable reads observed on request paths instead of cached configuration.
- `errgroup_fanout_without_limit_in_handler`: errgroup goroutine fanout without a visible concurrency limit in handlers.
- `file_or_template_read_per_request`: Request-path handlers that read files directly instead of using startup caching or dedicated file-serving paths.
- `formfile_open_readall_whole_upload`: Gin handlers that open uploaded form files and then materialize them with io.ReadAll(...).
- `get_raw_data_then_should_bindjson_duplicate_body`: Gin handlers that read GetRawData() and later bind JSON from the same request body.
- `gin_context_copy_for_each_item_fanout`: Gin handlers that call c.Copy() once per loop iteration before goroutine fanout.
- `gin_logger_debug_body_logging_on_hot_routes`: Verbose body or payload logging observed on likely high-volume Gin routes.
- `gzip_or_zip_writer_created_per_chunk`: Gzip or zip writers recreated per chunk inside handler loops instead of reusing per stream.
- `indentedjson_in_hot_path`: IndentedJSON(...) used on a request path instead of compact JSON rendering.
- `json_marshaled_manually_then_c_data`: Handlers that manually marshal JSON and then write it through gin.Context.Data(...).
- `large_csv_or_json_export_without_bufio`: Export data written in loops without visible buffering in handlers.
- `large_h_payload_built_only_for_json_response`: Large gin.H payloads built as transient dynamic maps right before JSON rendering.
- `loadhtmlglob_or_loadhtmlfiles_in_request_path`: LoadHTMLGlob or LoadHTMLFiles called on request paths instead of startup initialization.
- `middleware_allocates_db_or_gorm_handle_per_request`: Database or GORM connections opened inside handlers or middleware instead of process-level setup.
- `middleware_allocates_http_client_per_request`: HTTP client allocated inside Gin handlers or middleware instead of being shared.
- `middleware_allocates_regex_or_template_per_request`: Regexp compilation inside Gin handlers instead of using precompiled patterns.
- `middleware_rebinds_body_after_handler_bind`: Middleware or helper chains that parse the request body after the main handler has already bound it.
- `multiple_shouldbind_calls_same_handler`: Gin handlers that bind the request body multiple times in one function.
- `no_batching_on_handler_driven_db_write_loop`: Request handlers that drive row-by-row DB writes with no batch path.
- `no_streaming_for_large_export_handler`: Large list or export handlers that materialize everything before writing rather than using chunked or streaming output.
- `parsemultipartform_large_default_memory`: Gin handlers that call ParseMultipartForm(...) with large in-memory thresholds on request paths.
- `readall_body_then_bind_duplicate_deserialize`: Gin handlers that materialize c.Request.Body with io.ReadAll(...) and then bind the same body again.
- `repeated_body_rewind_for_multiple_decoders`: Gin handlers that read, rewind, and decode the same request body multiple times.
- `repeated_c_json_inside_stream_loop`: Gin handlers that call c.JSON(...) or c.PureJSON(...) from inside loops.
- `repeated_large_map_literal_response_construction`: Large map-literal response assembly on hot routes where a stable typed response would be cheaper.
- `servefile_via_readfile_then_c_data`: Handlers that load files into memory and then write them through gin.Context.Data(...) instead of using file helpers or streaming.
- `shouldbindbodywith_when_single_bind_is_enough`: Gin handlers that use ShouldBindBodyWith(...) even though only one body bind is observed.
- `template_parse_in_handler`: Template construction or parsing inside Gin handlers instead of startup-time caching.
- `upstream_http_call_per_item_in_handler_loop`: Upstream HTTP calls made per item inside handler loops.
- `upstream_json_decode_same_response_multiple_times`: One upstream HTTP response body decoded into multiple targets in the same handler.

#### Hot Path (33)
- `append_then_sort_each_iteration`: Slice sorting observed inside loops — often cheaper to sort once after the loop.
- `append_then_trim_each_iteration`: Slice append followed by reslice each iteration instead of batching.
- `bufio_reader_missing_for_small_read_loop`: File or socket reads inside loops without visible bufio buffering.
- `bufio_writer_missing_in_bulk_export`: File or socket writes inside loops without visible bufio buffering.
- `builder_or_buffer_recreated_per_iteration`: strings.Builder, bytes.Buffer, or bytes.NewBuffer(...) constructions observed inside loops instead of being reset or reused.
- `byte_string_conversion_in_loop`: Byte-to-string or string-to-byte conversion observed inside loops in short-lived lookup or append paths.
- `bytes_buffer_without_grow_known_bound`: bytes.Buffer used without Grow when approximate output size is locally visible.
- `bytes_split_same_input_multiple_times`: The same byte-slice input is passed through bytes.Split* or bytes.Fields* helpers multiple times in one function.
- `csv_writer_flush_per_row`: csv.Writer.Flush() called inside per-row loops, reducing buffering effectiveness.
- `filter_then_count_then_iterate`: Same collection traversed multiple times for filter, count, and process steps.
- `gzip_reader_writer_recreated_per_item`: gzip.NewReader(...) or gzip.NewWriter(...) recreated inside iterative paths instead of per stream.
- `json_decoder_recreated_per_item`: json.NewDecoder(...) constructed repeatedly inside loops instead of reusing a stable decoder per stream.
- `json_encoder_recreated_per_item`: json.NewEncoder(...) constructed repeatedly inside loops instead of reusing a stable encoder per stream.
- `make_map_inside_hot_loop_same_shape`: make(map[K]V, ...) scratch maps recreated inside loops instead of being reused or prebuilt.
- `make_slice_inside_hot_loop_same_shape`: make([]T, ...) scratch slices recreated inside loops instead of being reused.
- `map_growth_without_size_hint`: Map insertions inside loops without a visible size hint on the initial make call.
- `nested_append_without_outer_capacity`: Append calls inside nested loops without visible preallocation on the outer slice.
- `nested_linear_join_map_candidate`: Nested-loop lookups or joins that could use a map index for O(1) access.
- `read_then_decode_duplicate_materialization`: io.ReadAll(...) materializes a payload and the same binding is then unmarshaled again instead of using a streaming decode path.
- `regexp_compile_in_hot_path`: regexp.Compile or regexp.MustCompile observed inside obvious iterative paths.
- `repeated_map_clone_in_loop`: maps.Clone or equivalent map-copy calls observed inside loops.
- `repeated_slice_clone_in_loop`: slices.Clone(...) or similar whole-slice cloning observed inside loops.
- `slice_append_without_prealloc_known_bound`: Slice append inside a range loop without visible preallocation when the bound is locally known.
- `slice_membership_in_loop_map_candidate`: slices.Contains(...) or slices.Index(...) used inside loops against a stable-looking slice binding.
- `sort_before_first_or_membership_only`: Sorting a collection when only the first element or min/max is needed.
- `stable_value_normalization_in_inner_loop`: Stable value normalization (ToLower, TrimSpace, etc.) repeated inside inner loops.
- `strconv_repeat_on_same_binding`: The same string binding is converted with strconv parsing helpers multiple times in one function.
- `strings_builder_without_grow_known_bound`: strings.Builder used without Grow when approximate output size is locally visible.
- `strings_split_same_input_multiple_times`: The same string input is passed through strings.Split* or strings.Fields* helpers multiple times in one function.
- `template_parse_in_hot_path`: html/template or text/template parse calls observed on request-style paths instead of startup-time caching.
- `time_parse_layout_in_loop`: time.Parse(...) or ParseInLocation(...) observed inside loops with a stable layout.
- `url_parse_in_loop_on_invariant_base`: url.Parse(...) or ParseRequestURI(...) observed inside loops with a stable-looking base input.
- `uuid_hash_formatting_only_for_logs`: UUID or hash formatting observed inside loops only for log output.

#### Idioms (20)
- `defer_in_loop_resource_growth`: defer statements inside loops that can accumulate resources until function exit.
- `double_close_local_channel`: The same locally created channel appears to be closed more than once in one function body.
- `file_handle_without_close`: File handles opened via os.Open, os.Create, or os.OpenFile without an observed Close() path.
- `http_client_without_timeout`: Local http.Client{} literals constructed without an explicit timeout.
- `http_response_body_not_closed`: HTTP responses acquired locally without an observed resp.Body.Close() call.
- `http_server_without_timeouts`: Explicit http.Server{} values that omit common timeout fields.
- `http_status_ignored_before_decode`: Response decoding or body consumption that happens without an observed StatusCode check.
- `http_writeheader_after_write`: Handlers that write the response body before calling WriteHeader(...).
- `init_side_effect`: init() functions that perform network, file-system, or subprocess side effects.
- `mutable_package_global`: Package-level variables that are mutated from function bodies instead of kept immutable.
- `passthrough_wrapper_interface`: Wrapper structs that mostly forward one-to-one through an interface field with little added policy.
- `public_bool_parameter_api`: Exported functions or methods that expose raw boolean mode switches in their signatures.
- `range_over_local_channel_without_close`: Functions that range over a locally owned channel without an observed close path.
- `rows_without_close`: Query result handles that appear locally owned but have no observed rows.Close() call.
- `send_after_local_close_risk`: A locally owned channel is closed and later used in a send expression.
- `single_impl_interface`: Repository-local interfaces with one obvious implementation and a very small consumer surface.
- `stmt_without_close`: Prepared statements or similar DB handles without an observed Close() call.
- `ticker_without_stop`: time.NewTicker(...) is created without an observed Stop() call.
- `time_after_in_loop`: time.After(...) is allocated inside loops instead of reusing a timer or deadline.
- `tx_without_rollback_guard`: Transactions begun and later committed with no observed rollback guard.

#### Mod (4)
- `json_unmarshal_same_payload_multiple_times`: The same local JSON payload binding is unmarshaled into multiple targets in one function.
- `proto_unmarshal_same_payload_multiple_times`: The same local protobuf payload binding is unmarshaled into multiple targets in one function.
- `xml_unmarshal_same_payload_multiple_times`: The same local XML payload binding is unmarshaled into multiple targets in one function.
- `yaml_unmarshal_same_payload_multiple_times`: The same local YAML payload binding is unmarshaled into multiple targets in one function.

#### Performance (11)
- `allocation_churn_in_loop`: Obvious make, new, or buffer-construction calls inside loops.
- `fmt_hot_path`: fmt formatting calls such as Sprintf inside loops.
- `full_dataset_load`: Calls that load an entire payload into memory instead of streaming.
- `likely_n_squared_allocation`: Opt-in deeper semantic signal for allocations that also sit inside nested loop structure. *(status: experimental)*
- `likely_n_squared_string_concat`: Opt-in deeper semantic signal for repeated string concatenation inside nested loops without obvious builder usage. *(status: experimental)*
- `likely_unindexed_query`: Query shapes like leading-wildcard LIKE or ORDER BY without LIMIT that often scale poorly.
- `n_plus_one_query`: Database-style query calls issued inside loops. The opt-in semantic pack can raise severity when nested loops also appear.
- `reflection_hot_path`: reflect package calls inside loops.
- `repeated_json_marshaling`: encoding/json.Marshal or MarshalIndent inside loops — repeated allocation and serialization hot spots.
- `string_concat_in_loop`: Repeated string concatenation inside loops (O(n^2) risk).
- `wide_select_query`: Literal SELECT * query shapes.

#### Security (2)
- `sql_string_concat`: Query execution calls where SQL is constructed dynamically with concatenation or fmt.Sprintf.
- `weak_crypto`: Direct use of weak standard-library crypto packages such as crypto/md5, crypto/sha1, crypto/des, and crypto/rc4.

#### Style (2)
- `inconsistent_package_name`: Directories that mix base Go package names after ignoring the _test suffix.
- `misgrouped_imports`: Import blocks that place stdlib imports after third-party imports.

### Python rules (162)

#### Ai Smells (5)
- `enthusiastic_commentary`: Unusually enthusiastic or emoji-heavy production comments.
- `mixed_naming_conventions`: File mixes snake_case and camelCase function naming conventions.
- `obvious_commentary`: Comments that narrate obvious implementation steps instead of explaining intent.
- `textbook_docstring_small_helper`: Very small helper functions that have unusually long, textbook-style docstrings.
- `unrelated_heavy_import`: Heavy ecosystem imports with little local evidence of real need.

#### Duplication (8)
- `cross_file_copy_paste_function`: Highly similar non-test function bodies repeated across multiple Python files.
- `cross_file_repeated_literal`: Project repeats the same long string literal across multiple files.
- `duplicate_error_handler_block`: Repeated exception-handling block shapes in one file.
- `duplicate_query_fragment`: Repository repeats the same SQL-like query fragment across multiple files.
- `duplicate_test_utility_logic`: Highly similar utility logic shared between test and production code.
- `duplicate_transformation_pipeline`: Repository repeats the same data transformation pipeline stages across multiple functions.
- `duplicate_validation_pipeline`: Repeated validation guard pipelines across functions in one file.
- `repeated_string_literal`: Project repeats the same long string literal multiple times in one file.

#### Framework (51)
- `celery_delay_in_loop_without_canvas`: Celery tasks dispatch .delay(...) or .apply_async(...) inside loops without an obvious canvas primitive such as group() or chord().
- `celery_result_get_inside_task`: Celery tasks synchronously wait on AsyncResult.get(...) instead of handing work off asynchronously.
- `celery_task_reads_env_per_invocation`: Celery task bodies repeatedly read environment configuration instead of using startup-time bootstrap.
- `click_typer_config_file_loaded_per_command`: click or typer commands parse config files on each invocation instead of using shared bootstrap or dependency setup.
- `click_typer_env_lookup_per_command`: click or typer commands scatter repeated environment lookups through the command body.
- `click_typer_http_client_created_per_command`: click or typer commands allocate HTTP clients inside command bodies instead of reusing a shared client factory.
- `django_all_without_limit_in_view`: Django views call .all() without visible pagination, slicing, or limiting.
- `django_create_single_in_loop`: Django code creates one model at a time inside loops instead of using bulk_create().
- `django_delete_single_in_loop`: Django code deletes individual models inside loops instead of using set-based queryset deletion.
- `django_migration_code_in_view`: Django views or request paths reference schema or migration operations that belong in migrations.
- `django_n_plus_one_no_select_related`: Django queryset iteration shows N+1 risk with no visible select_related(...) or prefetch_related(...).
- `django_queryset_count_then_exists`: Django querysets use count() for existence checks instead of exists().
- `django_queryset_evaluated_multiple_times`: The same Django queryset appears to be evaluated multiple times in one function.
- `django_queryset_len_instead_of_count`: len(queryset) is used where queryset.count() would avoid loading every row.
- `django_queryset_order_by_random`: Django querysets use order_by(\"?\") or equivalent random ordering on request paths.
- `django_raw_sql_in_loop`: Django request or service code executes raw SQL inside loops instead of batching.
- `django_save_full_model_in_loop`: Django code saves full model instances in loops without update_fields or bulk updates.
- `django_update_single_in_loop`: Django code updates one row at a time inside loops instead of using bulk or set-based updates.
- `django_values_vs_full_model_in_loop`: Django loops hydrate full model instances where values(), values_list(), or only() would likely be cheaper.
- `fastapi_background_task_exception_silent`: FastAPI background task dispatches appear to rely on default exception behavior without visible error handling.
- `fastapi_dependency_creates_client_per_request`: FastAPI dependencies construct HTTP clients per request instead of using app lifespan or shared setup.
- `fastapi_response_model_without_orm_mode`: FastAPI response models use ORM conversion paths without visible ORM compatibility configuration.
- `fastapi_sync_def_with_blocking_io`: FastAPI sync route handlers perform blocking I/O instead of using async routes or executor offload.
- `flask_app_config_read_per_request`: Flask views repeatedly read app.config on request paths instead of consuming bootstrapped settings.
- `flask_debug_mode_in_production_code`: Flask code enables debug=True in application runtime paths.
- `flask_file_read_per_request`: Flask views read files directly on request paths instead of using cached or static responses.
- `flask_global_db_connection_per_request`: Flask views create database connections per request instead of using pooled or app-scoped access.
- `flask_json_encoder_per_request`: Flask views instantiate JSON encoders per request instead of reusing app-level serialization setup.
- `flask_no_streaming_for_large_response`: Flask views build large in-memory responses where generator or streaming responses would fit better.
- `flask_request_body_parsed_multiple_times`: Flask request handlers parse the same request body multiple times.
- `flask_template_rendered_from_string_in_view`: Flask views render templates from inline strings instead of using file-backed templates.
- `large_dict_literal_response_in_handler`: Handlers build large inline dict responses where typed response models would be clearer and cheaper.
- `middleware_compiles_regex_per_request`: Middleware compiles regex patterns per request instead of precompiling them.
- `middleware_creates_http_client_per_request`: Middleware creates HTTP clients per request instead of reusing app-scoped clients.
- `middleware_loads_config_file_per_request`: Middleware loads config files on request paths instead of using startup configuration.
- `pydantic_model_dump_then_json_dumps`: Pydantic v2 code serializes model_dump() output through json.dumps(...) instead of using model_dump_json().
- `pydantic_model_validate_after_json_loads`: Pydantic v2 validation is preceded by json.loads(...) even though model_validate_json() could validate raw JSON directly.
- `response_json_dumps_then_response_object`: Handlers manually json.dumps(...) payloads and then wrap them in framework Response objects.
- `sqlalchemy_commit_per_row_in_loop`: SQLAlchemy sessions commit inside loops instead of batching changes and committing once.
- `sqlalchemy_create_engine_per_request`: SQLAlchemy engines are created on request or handler paths instead of being process-scoped.
- `sqlalchemy_expire_on_commit_default_in_async`: Async SQLAlchemy sessions rely on the default expire_on_commit behavior instead of making the async access pattern explicit.
- `sqlalchemy_n_plus_one_lazy_load`: SQLAlchemy query shapes suggest lazy-loaded N+1 access with no visible eager loading.
- `sqlalchemy_query_in_loop`: SQLAlchemy code issues queries inside loops instead of batching or prefetching.
- `sqlalchemy_session_not_closed`: SQLAlchemy Session objects are created without context-manager or close handling.
- `sqlmodel_commit_per_row_in_loop`: SQLModel sessions commit inside loops instead of applying one transaction after batched updates.
- `sqlmodel_session_exec_in_loop`: SQLModel Session.exec(...) is called inside loops instead of combining the query shape.
- `sqlmodel_unbounded_select_in_handler`: Handlers execute SQLModel select().all() paths without visible limits or pagination.
- `template_render_in_loop`: Template rendering appears inside loops instead of rendering once over prepared data.
- `upstream_call_without_timeout_in_handler`: Request handlers issue upstream HTTP calls without visible timeout configuration.
- `upstream_http_call_per_item_in_handler`: Request handlers make sequential upstream HTTP calls inside loops instead of batching or bounded concurrency.
- `upstream_response_not_checked_before_decode`: Handlers decode upstream responses without visible status checks such as raise_for_status() or status_code guards.

#### Hot Path (4)
- `append_then_sort_each_iteration`: A collection is appended to and then sorted on each iteration instead of sorting once after accumulation.
- `csv_writer_flush_per_row`: csv.Writer flushes on each row instead of buffering a larger batch.
- `filter_then_count_then_iterate`: The same collection is traversed repeatedly for filtering, counting, and later iteration.
- `json_encoder_recreated_per_item`: A JSON encoder object is recreated per item instead of being reused for the stream.

#### Hotpath (11)
- `dict_items_or_keys_materialized_in_loop`: dict.items(), keys(), or values() are repeatedly materialized inside loops.
- `enumerate_on_range_len`: enumerate(range(len(...))) style loops that add indexing ceremony without extra value.
- `in_check_on_list_literal`: Membership tests against list literals where a tuple or set would be clearer or cheaper.
- `list_comprehension_only_for_length`: A list comprehension is built only so len(...) can be called on it.
- `read_then_splitlines`: File contents are fully read and then splitlines() is called instead of streaming lines.
- `readlines_then_iterate`: readlines() materializes the whole file before line-by-line iteration.
- `regex_compile_in_hot_path`: re.compile(...) or similar regex compilation repeated inside hot code paths.
- `repeated_open_same_file_in_function`: The same file appears to be opened multiple times within one function.
- `sorted_only_for_first_element`: A sequence is fully sorted even though only the first or smallest element is used.
- `string_startswith_endswith_chain`: Repeated startswith(...) or endswith(...) checks that can often be combined into tuple-based calls.
- `write_without_buffering_in_loop`: Repeated writes inside loops with no visible buffering or batching.

#### Hotpath Ext (15)
- `concatenation_in_comprehension_body`: String or collection concatenation happens inside a comprehension body, creating avoidable churn.
- `gzip_open_per_chunk`: gzip open/create calls are repeated per chunk instead of per stream.
- `nested_list_search_map_candidate`: Nested linear list searches that look like they want a temporary map or set index.
- `pickle_dumps_in_loop_same_structure`: pickle.dumps(...) is called repeatedly for the same structural shape in a loop.
- `repeated_datetime_strptime_same_format`: datetime.strptime(...) is repeated with the same format string instead of reusing a parsed shape or preprocessing once.
- `repeated_dict_get_same_key_no_cache`: The same dictionary key is fetched repeatedly instead of storing the value in a local binding.
- `repeated_hashlib_new_same_algorithm`: The same hashing algorithm is repeatedly constructed in a loop or tight path.
- `repeated_isinstance_chain_same_object`: The same object goes through repeated isinstance(...) checks that could be consolidated.
- `repeated_list_index_lookup`: The same list index lookup is performed repeatedly instead of caching the accessed value.
- `repeated_string_format_invariant_template`: An invariant string template is formatted repeatedly in a loop instead of being partially precomputed.
- `sort_then_first_or_membership_only`: A collection is sorted even though only the first element or a membership-style check is needed.
- `string_join_without_generator`: String joins that materialize an unnecessary list comprehension instead of using a generator or direct iterable.
- `tuple_unpacking_in_tight_loop`: Tuple unpacking is repeated in tight loops where reducing per-iteration overhead may help.
- `xml_parse_same_payload_multiple_times`: The same XML payload is parsed repeatedly within one function.
- `yaml_load_same_payload_multiple_times`: The same YAML payload is parsed repeatedly within one function.

#### Maintainability (20)
- `broad_exception_handler`: Broad except Exception: style handlers that still obscure failure shape even when not fully swallowed.
- `builtin_reduction_candidate`: Loop shapes that look like obvious sum, any, or all candidates.
- `commented_out_code`: Blocks of commented-out source code left in production files.
- `environment_boundary_without_fallback`: Environment-variable lookups that omit a default value or explicit failure handler.
- `eval_exec_usage`: Direct eval() or exec() usage in non-test Python code.
- `exception_swallowed`: Broad exception handlers like except: or except Exception: that immediately suppress the error with pass, continue, break, or return.
- `external_input_without_validation`: Request or CLI entry points that trust external input without obvious validation or guard checks.
- `hardcoded_business_rule`: Hardcoded threshold, rate-limit, or pricing-style literals assigned inside non-test Python functions.
- `hardcoded_path_string`: Hardcoded filesystem path literals assigned inside non-test Python functions.
- `magic_value_branching`: Repeated branch-shaping numeric or string literals that likely want an explicit constant or policy name.
- `missing_context_manager`: Resource management (files, network connections) inside non-test Python functions that omits with-statement context managers.
- `mixed_sync_async_module`: Modules that expose public sync and async entry points together.
- `network_boundary_without_timeout`: Request, sync, or job-style Python functions that call HTTP boundaries with no obvious timeout or retry policy.
- `none_comparison`: == None or != None checks instead of is None or is not None.
- `print_debugging_leftover`: print() calls left in non-test Python functions that do not look like obvious main-entrypoint output.
- `public_api_missing_type_hints`: Public Python functions that omit complete parameter or return annotations.
- `redundant_return_none`: Explicit return None in simple code paths where Python would already return None implicitly.
- `reinvented_utility`: Obvious locally implemented utility helpers that overlap with already-imported standard-library style helpers.
- `side_effect_comprehension`: List, set, or dicit comprehensions used as standalone statements where the result is discarded.
- `variadic_public_api`: Public Python functions that expose *args or **kwargs instead of a clearer interface.

#### Mlops (3)
- `langchain_chain_built_per_request`: LangChain or LlamaIndex prompt and chain wiring rebuilt on each request path.
- `tokenizer_encode_in_loop_without_cache`: Tokenizer encode calls repeated inside loops without caching or batching signals.
- `vector_store_client_created_per_request`: Vector-store clients created on request paths instead of reused application state.

#### Packaging (4)
- `cross_package_internal_import`: Local Python packages reaching into another package's internal or private modules.
- `pyproject_missing_requires_python`: pyproject metadata missing an explicit Python runtime requirement.
- `pyproject_script_entrypoint_unresolved`: pyproject script entrypoints that do not resolve to a locally indexed module callable.
- `python_public_api_any_contract`: Public Python APIs that expose Any in parameter or return contracts.

#### Performance (9)
- `blocking_sync_io_in_async`: Synchronous network, subprocess, sleep, or file I/O calls made from async def functions.
- `deque_candidate_queue`: Queue-style list operations like pop(0) or insert(0, ...) that may want collections.deque.
- `full_dataset_load`: Calls that load an entire payload into memory instead of streaming.
- `list_materialization_first_element`: list(...)[0] style access that materializes a whole list just to read the first element.
- `list_membership_in_loop`: Repeated membership checks against obviously list-like containers inside loops.
- `recursive_traversal_risk`: Direct recursion in traversal-style helpers that may be safer as iterative walks for deep inputs.
- `repeated_len_in_loop`: Repeated len(...) checks inside loops when the receiver appears unchanged locally.
- `string_concat_in_loop`: Repeated string concatenation inside loops can create O(n^2) growth and extra allocations.
- `temporary_collection_in_loop`: Loop-local list, dict, or set construction that likely adds avoidable allocation churn.

#### Quality (21)
- `async_lock_held_across_await`: Async lock scopes or explicit acquire/release regions that continue across unrelated await points.
- `async_retry_sleep_without_backoff`: Retry-style async loops that sleep a fixed interval without visible backoff, jitter, or bounded retry policy.
- `background_task_exception_unobserved`: Background task bindings with no obvious await, callback, supervisor, or observation path.
- `dataclass_heavy_post_init`: Dataclass __post_init__ methods that perform I/O, subprocess, network, or heavyweight client setup.
- `dataclass_mutable_default`: Dataclass fields that use mutable defaults instead of default_factory.
- `import_time_config_load`: Module-scope configuration or secret loading that runs during import instead of an explicit startup path.
- `import_time_file_io`: Module-scope file reads, writes, or directory scans that happen during import.
- `import_time_network_call`: Module-scope HTTP or socket calls executed while the module is imported.
- `import_time_subprocess`: Subprocess launches triggered from module scope during import.
- `module_singleton_client_side_effect`: Eagerly constructed network, database, or cloud clients bound at module scope.
- `mutable_default_argument`: Function parameters that use mutable defaults such as [], {}, or set() directly in the signature.
- `mutable_module_global_state`: Mutable module globals updated from multiple functions.
- `option_bag_model`: Dataclass or TypedDict models that accumulate many optional fields and boolean switches.
- `pickle_deserialization_boundary`: pickle.load(s) or dill.load(s) style deserialization in production code.
- `public_any_type_leak`: Public functions or model fields that expose Any, object, or similarly wide contracts.
- `subprocess_shell_true`: Subprocess boundaries that enable shell=True.
- `tar_extractall_unfiltered`: tarfile.extractall(...) without an obvious filter, members list, or path-validation helper.
- `tempfile_without_cleanup`: Temporary files or directories created without a visible cleanup or context-manager ownership path.
- `typeddict_unchecked_access`: Direct indexing of optional TypedDict keys without an obvious guard path.
- `unsafe_yaml_loader`: yaml.load(...) or full_load(...) style loaders used where safe loading is more appropriate.
- `untracked_asyncio_task`: asyncio.create_task(...) or similar task creation whose handle is discarded immediately.

#### Structure (11)
- `deep_inheritance_hierarchy`: Repository-local Python class chains with unusually deep inheritance depth.
- `eager_constructor_collaborators`: Constructors that instantiate several collaborators eagerly inside __init__.
- `god_class`: Python classes that concentrate unusually high method count, public surface area, and mutable instance state.
- `god_function`: Very large Python functions with high control-flow and call-surface concentration.
- `mixed_concerns_function`: Functions that mix HTTP, persistence, and filesystem-style concerns in one body.
- `monolithic_init_module`: __init__.py files that carry enough imports and behavior to look like monolithic modules.
- `monolithic_module`: Non-__init__.py modules that are unusually large and combine many imports with orchestration-heavy behavior.
- `name_responsibility_mismatch`: Read-style, transformation-style, or utility-style names that still perform mutation or own multiple infrastructure concerns.
- `over_abstracted_wrapper`: Ceremonial wrapper-style or tiny data-container classes that add little beyond storing constructor state.
- `tight_module_coupling`: Modules that depend on a large number of repository-local Python modules.
- `too_many_instance_attributes`: Classes that assign an unusually large number of instance attributes across their methods.

### Rust rules (74)

#### Api Design (21)
- `rust_arc_mutex_option_state`: Arc<Mutex<Option<T>>>-style state bags that hide lifecycle state behind nested mutation layers.
- `rust_boolean_state_machine`: State structs encoded with multiple booleans instead of a dedicated enum.
- `rust_borrowed_pathbuf_api`: Public signatures that borrow &PathBuf instead of &Path.
- `rust_borrowed_string_api`: Public signatures that borrow &String instead of &str.
- `rust_borrowed_vec_api`: Public signatures that borrow &Vec<T> instead of &[T].
- `rust_builder_without_validate`: Builders that expose build() without an obvious validation step.
- `rust_constructor_many_flags`: Constructor-like APIs that use multiple boolean flags.
- `rust_global_lock_state`: Static or lazy global state wrapped in mutable lock-based containers.
- `rust_mutex_wrapped_collection`: Collection-plus-lock fields embedded directly in public or central state structs.
- `rust_option_bag_config`: Config-like structs with many Option fields and no obvious validation path.
- `rust_partial_init_escape`: Constructor-like functions that return or store partially initialized structs.
- `rust_pub_interior_mutability_field`: Public structs that expose Mutex, RwLock, RefCell, Cell, or similar fields directly.
- `rust_public_anyhow_result`: Public library-facing APIs that return anyhow-style Result types instead of a clearer domain error.
- `rust_public_bool_parameter_api`: Public APIs that expose a raw boolean mode switch.
- `rust_public_box_dyn_error`: Public APIs that expose Box<dyn Error> rather than a clearer error surface.
- `rust_rc_refcell_domain_model`: Domain-style structs built around Rc<RefCell<T>> instead of clearer ownership boundaries.
- `rust_serde_default_on_required_field`: Required-looking contract fields that opt into #[serde(default)].
- `rust_serde_flatten_catchall`: #[serde(flatten)] catch-all maps or loose value bags that absorb unknown fields.
- `rust_serde_unknown_fields_allowed`: Strict-looking config or request structs that deserialize without deny_unknown_fields.
- `rust_serde_untagged_enum_boundary`: Boundary-facing enums that derive #[serde(untagged)] and risk ambiguous wire formats.
- `rust_stringly_typed_enum_boundary`: Enum-like boundary fields kept as String instead of a dedicated enum.

#### Async Patterns (9)
- `rust_async_blocking_drop`: A Drop implementation does blocking work that can surface in async contexts. *(status: experimental)*
- `rust_async_hold_permit_across_await`: A permit or pooled resource may be held across an .await. *(status: experimental)*
- `rust_async_invariant_broken_at_await`: Related state mutations appear split around an await boundary. *(status: experimental)*
- `rust_async_lock_order_cycle`: Conflicting lock acquisition order suggests a lock-order cycle. *(status: experimental)*
- `rust_async_missing_fuse_pin`: select! reuse lacks pinning or fusing markers for repeated polling. *(status: experimental)*
- `rust_async_monopolize_executor`: An async function may monopolize the executor with blocking work and no await. *(status: experimental)*
- `rust_async_recreate_future_in_select`: A select! loop may recreate futures instead of reusing long-lived ones. *(status: experimental)*
- `rust_async_spawn_cancel_at_await`: Async work is spawned without an obvious cancellation path. *(status: experimental)*
- `rust_async_std_mutex_await`: std::sync::Mutex appears to be held across .await in async code. *(status: experimental)*

#### Domain Modeling (8)
- `rust_debug_secret`: Debug is derived on a type that carries secret-like fields.
- `rust_domain_default_produces_invalid`: Default is derived or implemented on a type that likely cannot have a safe default state.
- `rust_domain_float_for_money`: Floating-point storage is used for money-like values.
- `rust_domain_impossible_combination`: A boolean toggle is mixed with optional credentials, creating invalid-state combinations.
- `rust_domain_optional_secret_default`: A defaultable type includes optional secret-like fields, which can hide invalid configuration.
- `rust_domain_raw_primitive`: Business-facing data is stored as a raw primitive instead of a stronger domain type.
- `rust_serde_sensitive_deserialize`: Deserialize is derived for sensitive fields without obvious validation.
- `rust_serde_sensitive_serialize`: Serialize is derived for secret-like fields that may need redaction or exclusion.

#### Hygiene (11)
- `dbg_macro_leftover`: dbg!() left in non-test Rust code.
- `expect_in_non_test_code`: .expect(...) used in non-test Rust code.
- `fixme_doc_comment_leftover`: Rust doc comments that still contain a FIXME marker in non-test code.
- `hack_doc_comment_leftover`: Rust doc comments that still contain a HACK marker in non-test code.
- `panic_macro_leftover`: panic!() left in non-test Rust code.
- `todo_doc_comment_leftover`: Rust doc comments that still contain a TODO marker in non-test code.
- `todo_macro_leftover`: todo!() left in non-test Rust code.
- `unimplemented_macro_leftover`: unimplemented!() left in non-test Rust code.
- `unreachable_macro_leftover`: unreachable!() left in non-test Rust code.
- `unsafe_without_safety_comment`: unsafe fn or unsafe block without a nearby SAFETY: comment within the previous two lines.
- `unwrap_in_non_test_code`: .unwrap() used in non-test Rust code.

#### Performance (12)
- `rust_aos_hot_path`: Repeated struct-field dereferences inside a loop that may indicate an array-of-structs hot path.
- `rust_blocking_drop`: A Drop implementation performs blocking work.
- `rust_blocking_io_in_async`: Blocking I/O or blocking work observed in async Rust code. *(status: experimental)*
- `rust_hashmap_default_hasher`: HashMap default-hasher construction in a likely hot path.
- `rust_large_future_stack`: Large allocations may be captured across await points and bloat future size.
- `rust_lines_allocate_per_line`: .lines() iteration used in a loop where per-item allocation may matter.
- `rust_lock_across_await`: A lock appears to be held across an .await boundary. *(status: experimental)*
- `rust_path_join_absolute`: Path::join used with an absolute segment that discards the existing base path.
- `rust_pointer_chasing_vec_box`: Pointer-heavy boxed vector-style storage that may hurt cache locality.
- `rust_tokio_mutex_unnecessary`: tokio::sync::Mutex used in a fully synchronous critical path with no await. *(status: experimental)*
- `rust_unbuffered_file_writes`: File-like writes performed inside a loop without buffering or batching.
- `rust_utf8_validate_hot_path`: UTF-8 validation appears in a likely hot path and may deserve profiling.

#### Runtime Boundary (6)
- `rust_axum_router_built_in_handler`: Axum routers assembled inside handler call paths instead of startup wiring.
- `rust_clone_heavy_state_in_loop`: Likely heavy state cloned repeatedly inside loops in Rust application code.
- `rust_env_var_read_in_request_path`: Rust request handlers reading environment configuration on the hot request path.
- `rust_tokio_runtime_built_per_call`: Tokio runtimes created per call instead of being owned at process or bootstrap boundaries.
- `rust_tonic_channel_connect_per_request`: tonic transport channels dialed on request paths instead of reusing configured clients.
- `rust_workspace_missing_resolver`: Workspace Cargo manifests with multiple members but no explicit resolver version.

#### Unsafe Soundness (7)
- `rust_unsafe_aliasing_assumption`: Unsafe code mixes interior mutability and mutable references in ways that need careful aliasing review.
- `rust_unsafe_assume_init`: Unsafe MaybeUninit::assume_init use without proof of full initialization.
- `rust_unsafe_from_raw_parts`: Unsafe raw slice construction that depends on lifetime and length invariants.
- `rust_unsafe_get_unchecked`: Unsafe use of get_unchecked without proof of bounds invariants.
- `rust_unsafe_raw_pointer_cast`: Unsafe raw pointer cast that depends on aliasing and lifetime guarantees.
- `rust_unsafe_set_len`: Unsafe Vec::set_len use that requires initialized elements and correct capacity invariants.
- `rust_unsafe_transmute`: Unsafe transmute use that requires layout and validity proof.
## Detection philosophy

- Findings are heuristics, not compile-time proof.
- The analyzer is intentionally conservative where full type information is missing.
- Rules are designed to produce readable evidence so humans can validate them quickly.
- Local repository context is used where possible, but deslop does not replace `go/types`.

## Current limitations

- No authoritative Go, Python, or Rust type checking yet.
- No full interprocedural context propagation or type-aware Go data flow.
- No proof of goroutine leaks, N+1 queries, or runtime performance regressions.
- Package-method and local-symbol checks are repository-local and now language-scoped for mixed-language repositories.
- No Python module graph resolution or installed-package awareness yet.
- No Rust trait resolution, cargo workspace modeling, or macro expansion yet.

## Phase status

### Implemented so far

- Phase 1 rule pack: naming, weak typing, comment style, weak crypto, early error-handling checks, and local hallucination checks.
- Phase 2 parser enrichment: context-parameter detection, derived-context factory tracking, raw goroutine launch tracking, goroutine-in-loop tracking, goroutine shutdown-path tracking, looped `time.Sleep` detection, looped `select default` detection, looped JSON marshal detection, mutex lock-in-loop tracking, allocation tracking, fmt and reflect hot-path tracking, looped database query extraction, and string-concatenation-in-loop tracking.
- Phase 2 heuristic additions: broader `missing_context`, `missing_cancel_call`, `sleep_polling`, `busy_waiting`, `repeated_json_marshaling`, `string_concat_in_loop`, `goroutine_spawn_in_loop`, `goroutine_without_shutdown_path`, `mutex_in_loop`, `blocking_call_while_locked`, `allocation_churn_in_loop`, `fmt_hot_path`, `reflection_hot_path`, `full_dataset_load`, `n_plus_one_query`, `wide_select_query`, `likely_unindexed_query`, and the first conservative goroutine-coordination pass.
- Phase 3 heuristic additions: `hardcoded_secret`, `sql_string_concat`, `mixed_receiver_kinds`, `malformed_struct_tag`, `duplicate_struct_tag_key`, `test_without_assertion_signal`, `happy_path_only_test`, and `placeholder_test_body`.
- Python backend additions so far: `.py` routing, Python parser coverage for imports, symbols, call sites, docstrings, test classification, loop concatenation, and conservative exception-handler evidence.
- Python parser-contract and rollout additions so far: fixture-backed parser coverage under `src/analysis/python/parser/tests.rs`, standardized `.txt` fixture families under `tests/fixtures/python/**`, a split Python integration harness under `tests/integration_scan/python/{baseline,phase5_rules,advanceplan2}.rs`, grouped advanceplan2 fixture families for async, contract, import-time, and boundary checks, and fixture-backed multi-file assemblies for repo-level duplication, coupling, and hallucination coverage.
- Python heuristic additions so far: `blocking_sync_io_in_async`, `exception_swallowed`, `eval_exec_usage`, `print_debugging_leftover`, `none_comparison`, `side_effect_comprehension`, `redundant_return_none`, `hardcoded_path_string`, `hardcoded_business_rule`, `magic_value_branching`, `reinvented_utility`, `variadic_public_api`, `list_materialization_first_element`, `deque_candidate_queue`, `temporary_collection_in_loop`, `recursive_traversal_risk`, `list_membership_in_loop`, `repeated_len_in_loop`, `builtin_reduction_candidate`, `untracked_asyncio_task`, `background_task_exception_unobserved`, `async_lock_held_across_await`, `async_retry_sleep_without_backoff`, `mutable_default_argument`, `dataclass_mutable_default`, `dataclass_heavy_post_init`, `option_bag_model`, `public_any_type_leak`, `typeddict_unchecked_access`, `broad_exception_handler`, `missing_context_manager`, `network_boundary_without_timeout`, `environment_boundary_without_fallback`, `external_input_without_validation`, `unsafe_yaml_loader`, `pickle_deserialization_boundary`, `subprocess_shell_true`, `tar_extractall_unfiltered`, `tempfile_without_cleanup`, `public_api_missing_type_hints`, `mixed_sync_async_module`, `import_time_network_call`, `import_time_file_io`, `import_time_subprocess`, `module_singleton_client_side_effect`, `mutable_module_global_state`, `import_time_config_load`, `god_function`, `god_class`, `monolithic_init_module`, `monolithic_module`, `too_many_instance_attributes`, `eager_constructor_collaborators`, `over_abstracted_wrapper`, `mixed_concerns_function`, `name_responsibility_mismatch`, `deep_inheritance_hierarchy`, `tight_module_coupling`, `textbook_docstring_small_helper`, `mixed_naming_conventions`, `unrelated_heavy_import`, `obvious_commentary`, `enthusiastic_commentary`, `commented_out_code`, `repeated_string_literal`, `duplicate_error_handler_block`, `duplicate_validation_pipeline`, `duplicate_test_utility_logic`, `cross_file_copy_paste_function`, `cross_file_repeated_literal`, `duplicate_query_fragment`, `duplicate_transformation_pipeline`, Python reuse of `full_dataset_load`, and Python reuse of `string_concat_in_loop`.
- Rust hygiene and hallucination additions so far: `todo_macro_leftover`, `unimplemented_macro_leftover`, `dbg_macro_leftover`, `panic_macro_leftover`, `unreachable_macro_leftover`, `todo_doc_comment_leftover`, `fixme_doc_comment_leftover`, `unwrap_in_non_test_code`, `expect_in_non_test_code`, `unsafe_without_safety_comment`, Rust-local `hallucinated_import_call`, and Rust-local `hallucinated_local_call`.
- Rust async and performance additions so far: `rust_blocking_io_in_async`, `rust_lock_across_await`, `rust_async_std_mutex_await`, `rust_async_hold_permit_across_await`, `rust_async_spawn_cancel_at_await`, `rust_async_missing_fuse_pin`, `rust_async_recreate_future_in_select`, `rust_async_monopolize_executor`, `rust_async_blocking_drop`, `rust_async_invariant_broken_at_await`, `rust_async_lock_order_cycle`, `rust_unbuffered_file_writes`, `rust_lines_allocate_per_line`, `rust_hashmap_default_hasher`, `rust_tokio_mutex_unnecessary`, `rust_blocking_drop`, `rust_pointer_chasing_vec_box`, `rust_path_join_absolute`, `rust_utf8_validate_hot_path`, `rust_large_future_stack`, and `rust_aos_hot_path`.
- Rust API, shared-state, wire-contract, and builder-state additions so far: `rust_public_anyhow_result`, `rust_public_box_dyn_error`, `rust_borrowed_string_api`, `rust_borrowed_vec_api`, `rust_borrowed_pathbuf_api`, `rust_public_bool_parameter_api`, `rust_pub_interior_mutability_field`, `rust_global_lock_state`, `rust_arc_mutex_option_state`, `rust_mutex_wrapped_collection`, `rust_rc_refcell_domain_model`, `rust_serde_untagged_enum_boundary`, `rust_serde_default_on_required_field`, `rust_serde_flatten_catchall`, `rust_serde_unknown_fields_allowed`, `rust_stringly_typed_enum_boundary`, `rust_option_bag_config`, `rust_builder_without_validate`, `rust_constructor_many_flags`, `rust_partial_init_escape`, and `rust_boolean_state_machine`.
- Rust domain-modeling and unsafe-soundness additions so far: `rust_domain_raw_primitive`, `rust_domain_float_for_money`, `rust_domain_impossible_combination`, `rust_domain_default_produces_invalid`, `rust_debug_secret`, `rust_serde_sensitive_deserialize`, `rust_serde_sensitive_serialize`, `rust_domain_optional_secret_default`, `rust_unsafe_get_unchecked`, `rust_unsafe_from_raw_parts`, `rust_unsafe_set_len`, `rust_unsafe_assume_init`, `rust_unsafe_transmute`, `rust_unsafe_raw_pointer_cast`, and `rust_unsafe_aliasing_assumption`.

### Still pending

- Stronger repo-wide style checks.
- Deeper goroutine lifetime analysis beyond local shutdown-path heuristics.
- Better context propagation through wrappers and helper functions.
- Python installed-package awareness, module-graph resolution, and deeper interprocedural asyncio reasoning.
- Optional deeper semantic analysis for harder cases such as type-aware data flow, true index awareness, struct layout analysis, and O(n²) detection.