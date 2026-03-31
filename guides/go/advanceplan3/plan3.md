# Plan 3 - Gin And HTTP Request-Path Performance Backlog (Go)

Date: 2026-03-31

## Status

- [x] Initial slice implemented on 2026-03-31.
- [x] Parser-backed Gin body and render summaries replaced the original body-text matching on 2026-03-31.
- [x] Multipart upload follow-up shipped on 2026-03-31 for large `ParseMultipartForm(...)` thresholds and whole-upload `FormFile` plus `ReadAll(...)` paths.
- [x] This plan targets `gin-gonic/gin` and HTTP handler performance patterns that are currently outside the shipped Go heuristics.
- [x] The emphasis is on request-body duplication, response shaping, request-scope allocation churn, and handler fanout patterns that typical lint packs rarely model.

## Already Covered And Excluded From This Plan

- [x] `http_response_body_not_closed`
- [x] `http_client_without_timeout`
- [x] `http_server_without_timeouts`
- [x] `http_status_ignored_before_decode`
- [x] `http_writeheader_after_write`
- [x] `full_dataset_load` already covers coarse whole-payload reads, but the Gin-specific request/response duplication cases below are still missing.

## Objective

Add a Gin-aware request-path pack that understands `*gin.Context`, common bind/render helpers, body reuse, export patterns, and handler-driven upstream fanout. These rules should stay focused on obvious throughput regressions rather than stylistic concerns.

## Shipped Rules

- [x] `get_raw_data_then_should_bindjson_duplicate_body`
- [x] `readall_body_then_bind_duplicate_deserialize`
- [x] `multiple_shouldbind_calls_same_handler`
- [x] `bindjson_into_map_any_hot_endpoint`
- [x] `bindquery_into_map_any_hot_endpoint`
- [x] `parsemultipartform_large_default_memory`
- [x] `formfile_open_readall_whole_upload`
- [x] `shouldbindbodywith_when_single_bind_is_enough`
- [x] `indentedjson_in_hot_path`
- [x] `repeated_c_json_inside_stream_loop`
- [x] `json_marshaled_manually_then_c_data`
- [x] `servefile_via_readfile_then_c_data`
- [x] `dumprequest_or_dumpresponse_in_hot_path`
- [x] `file_or_template_read_per_request`
- [x] `gin_context_copy_for_each_item_fanout`

## Fixtures And Verification

- [x] Added `tests/fixtures/go/advanceplan3_gin_positive.txt`.
- [x] Added `tests/fixtures/go/advanceplan3_gin_clean.txt`.
- [x] Added `tests/integration_scan/go_advanceplan3.rs` coverage for the first Gin request-path family.
- [x] Expanded the Gin fixtures with `ShouldBindBodyWith(...)`, looped `c.JSON(...)`, request dumps, and request-path file reads.
- [x] Expanded the Gin fixtures with dynamic `BindJSON` and `BindQuery` targets, file-serving-via-`Data`, and looped `c.Copy()` fanout coverage.
- [x] Expanded the Gin parser and request fixtures with multipart threshold and uploaded-file `ReadAll(...)` coverage.
- [x] Verified with `cargo test --test integration_scan go_advanceplan3` and the full `cargo test --test integration_scan` suite.
- [x] Verified the multipart upload follow-up with `cargo test test_collects_gin_calls_and_parse_input_summaries` and `cargo test --test integration_scan go_advanceplan3`.
- [x] Validated the shipped Gin request-path rules against `eddycjy/go-gin-example`; the application code stayed quiet for the new rule IDs.

## Candidate Scenario Backlog (34 scenarios)

### Request Body Duplication And Binding Waste

- [x] `get_raw_data_then_should_bindjson_duplicate_body`: detect `c.GetRawData()` followed by `ShouldBindJSON`, `BindJSON`, or equivalent body parsing in the same handler.
- [x] `readall_body_then_bind_duplicate_deserialize`: detect `io.ReadAll(c.Request.Body)` followed by a second bind or decode path.
- [x] `shouldbindbodywith_when_single_bind_is_enough`: detect `ShouldBindBodyWith` in handlers that only bind once and therefore pay for an unnecessary body copy.
- [x] `multiple_shouldbind_calls_same_handler`: detect multiple Gin bind helpers on the same request body in one handler.
- [x] `bindjson_into_map_any_hot_endpoint`: detect body binding into `map[string]any` or similarly dynamic containers on hot request paths.
- [x] `bindquery_into_map_any_hot_endpoint`: detect repeated query binding into dynamic maps or wide generic containers when a stable struct contract exists.
- [x] `parsemultipartform_large_default_memory`: detect `ParseMultipartForm` with large in-memory thresholds on regular request handlers.
- [x] `formfile_open_readall_whole_upload`: detect upload paths that open a form file and immediately `ReadAll` the full payload.
- [ ] `repeated_body_rewind_for_multiple_decoders`: detect handlers that read, rewind, and decode the same body multiple times.
- [ ] `middleware_rebinds_body_after_handler_bind`: detect middleware or helper chains that parse the request body after the main handler has already bound it.

### Response Construction And Rendering Cost

- [x] `indentedjson_in_hot_path`: detect `IndentedJSON` or pretty-print JSON rendering in non-debug request paths.
- [x] `json_marshaled_manually_then_c_data`: detect `json.Marshal` followed by `c.Data` or `c.Writer.Write` instead of a direct Gin JSON renderer.
- [x] `repeated_c_json_inside_stream_loop`: detect `c.JSON` or `c.PureJSON` inside streaming loops where encoder-based streaming would be more stable.
- [ ] `no_streaming_for_large_export_handler`: detect large list/export handlers that materialize everything before writing rather than using chunked or streaming output.
- [ ] `template_parse_in_handler`: detect template parsing or loading directly inside handlers.
- [ ] `loadhtmlglob_or_loadhtmlfiles_in_request_path`: detect Gin HTML template loading APIs called at request time.
- [x] `servefile_via_readfile_then_c_data`: detect file-serving paths that load files into memory and then write through Gin instead of using streaming/file helpers.
- [ ] `repeated_large_map_literal_response_construction`: detect large map-literal response assembly on hot routes where a stable typed response or reusable encoder path would be cheaper.
- [ ] `large_h_payload_built_only_for_json_response`: detect large `gin.H` payloads built as transient dynamic maps right before JSON rendering.

### Middleware And Request-Scope Allocation Churn

- [ ] `middleware_allocates_http_client_per_request`: detect `http.Client` or transport construction inside Gin middleware.
- [ ] `middleware_allocates_db_or_gorm_handle_per_request`: detect DB or GORM handle creation in middleware instead of shared setup.
- [ ] `middleware_allocates_regex_or_template_per_request`: detect regex or template compile/parse work in middleware.
- [x] `dumprequest_or_dumpresponse_in_hot_path`: detect `httputil.DumpRequest`, `DumpRequestOut`, or `DumpResponse` on hot request paths.
- [ ] `env_or_config_lookup_per_request`: detect repeated config file loads, environment parsing, or heavyweight config decoding in handlers and middleware.
- [x] `file_or_template_read_per_request`: detect file reads for templates, policy docs, or static fragments inside handlers.
- [x] `gin_context_copy_for_each_item_fanout`: detect `c.Copy()` once per item in fanout loops rather than once per goroutine family or once per request.
- [ ] `gin_logger_debug_body_logging_on_hot_routes`: detect verbose body or payload logging on likely high-volume routes.

### Upstream Fanout, Export Paths, And Batch Gaps

- [ ] `upstream_http_call_per_item_in_handler_loop`: detect one upstream HTTP call per element in a request loop.
- [ ] `duplicate_upstream_calls_same_url_same_handler`: detect the same upstream URL or request template being called multiple times in one handler.
- [ ] `upstream_json_decode_same_response_multiple_times`: detect one upstream response body decoded into multiple targets in the same handler.
- [ ] `errgroup_fanout_without_limit_in_handler`: detect `errgroup.Go` or goroutine fanout in handlers without a visible concurrency limit when the fanout size is request-driven.
- [ ] `no_batching_on_handler_driven_db_write_loop`: detect request handlers that drive row-by-row DB writes with no batch path.
- [ ] `gzip_or_zip_writer_created_per_chunk`: detect compression writers being recreated repeatedly during chunked response generation.
- [ ] `large_csv_or_json_export_without_bufio`: detect export handlers that write row-by-row to the response writer without buffering.

## Shared Implementation Checklist

- [ ] Add `GinHandlerSummary` style parser evidence using `*gin.Context` parameters plus router registration cues where available.
- [x] Capture request-body access summaries for `GetRawData`, `ShouldBind*`, `Bind*`, `ReadAll(c.Request.Body)`, `ParseMultipartForm`, and `FormFile`.
- [ ] Capture render summaries for `JSON`, `PureJSON`, `IndentedJSON`, `Data`, streaming helpers, template load helpers, and response-writer flush/write sites.
- [x] Added Gin call summaries for `GetRawData`, `ShouldBind*`, `Bind*`, `ShouldBindQuery`, `ParseMultipartForm`, `FormFile`, `Copy`, `IndentedJSON`, `Data`, and request-body reads so the shipped rules no longer rely on raw body-text matching.
- [x] Reuse the generic import-alias machinery so Gin-specific rules can still understand mixed `net/http` and Gin code in the same handler.
- [ ] Add positive and clean fixtures for body-duplication, export, middleware-allocation, and upstream-fanout families before promoting any rule.

## Acceptance Criteria

- [x] Each new rule explains whether the cost comes from duplicate body reads, repeated binding, response materialization, request-scope allocation churn, or handler fanout.
- [x] Clean handlers that bind once, stream large exports, and reuse process-level clients stay quiet.
- [x] Startup-only template loading or router setup should not trigger request-path findings.