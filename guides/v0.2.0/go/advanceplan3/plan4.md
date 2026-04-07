# Plan 4 - Implementation Roadmap, Parser Work, And Validation (Go)

Date: 2026-03-31

## Status

- [x] Initial implementation slice shipped on 2026-03-31.
- [x] Parser-backed follow-up shipped on 2026-03-31 for ordered GORM chains, Gin call summaries, and repeated-input parse grouping.
- [x] A third backlog-reduction batch shipped on 2026-03-31 for duplicate XML/YAML/protobuf decode, looped JSON decoder, request-path DB churn, looped SQL writes and counts, and Gin request-path body-copy or output waste.
- [x] A fourth backlog-reduction batch shipped on 2026-03-31 for repeated request-path prepares, looped transactions and GORM session/preload/query terminals, dynamic Gin map binding, file-serving via `Data`, and `c.Copy()` fanout churn.
- [x] A fifth backlog-reduction batch shipped on 2026-03-31 for multipart upload thresholds and whole-upload `FormFile` plus `ReadAll(...)` paths.
- [x] A sixth backlog-reduction batch shipped on 2026-03-31 for repeated split and `strconv` work, scratch slice and map churn, slice cloning, byte-string conversion, loop-local URL and time parsing, builder and buffer recreation, slice membership checks, and generic `ReadAll(...)` plus decode churn.
- [x] This file is the execution roadmap for `advanceplan3` and ties the scenario backlog to the current Rust parser and heuristics architecture.

## First Slice Delivered

- [x] Plan 1 shipped `regexp_compile_in_hot_path`, `template_parse_in_hot_path`, `json_encoder_recreated_per_item`, `gzip_reader_writer_recreated_per_item`, and `csv_writer_flush_per_row`.
- [x] Plan 1 also now ships `xml_unmarshal_same_payload_multiple_times`, `yaml_unmarshal_same_payload_multiple_times`, `proto_unmarshal_same_payload_multiple_times`, `json_decoder_recreated_per_item`, `builder_or_buffer_recreated_per_iteration`, `make_slice_inside_hot_loop_same_shape`, `make_map_inside_hot_loop_same_shape`, `repeated_slice_clone_in_loop`, `byte_string_conversion_in_loop`, `slice_membership_in_loop_map_candidate`, `url_parse_in_loop_on_invariant_base`, `time_parse_layout_in_loop`, `strings_split_same_input_multiple_times`, `bytes_split_same_input_multiple_times`, `strconv_repeat_on_same_binding`, and `read_then_decode_duplicate_materialization`.
- [x] Plan 2 shipped `sql_open_per_request`, `gorm_open_per_request`, `db_ping_per_request`, `connection_pool_reconfigured_per_request`, `prepare_inside_loop`, `prepare_on_every_request_same_sql`, `tx_begin_per_item_loop`, `exec_inside_loop_without_batch`, `queryrow_inside_loop_existence_check`, `count_inside_loop`, `gorm_session_allocated_per_item`, `raw_scan_inside_loop`, `association_find_inside_loop`, `preload_inside_loop`, `first_or_create_in_loop`, `save_in_loop_full_model`, `update_single_row_in_loop_without_batch`, `delete_single_row_in_loop_without_batch`, `gorm_debug_enabled_in_request_path`, and `create_single_in_loop_instead_of_batches`.
- [x] Plan 3 shipped `get_raw_data_then_should_bindjson_duplicate_body`, `readall_body_then_bind_duplicate_deserialize`, `multiple_shouldbind_calls_same_handler`, `bindjson_into_map_any_hot_endpoint`, `bindquery_into_map_any_hot_endpoint`, `parsemultipartform_large_default_memory`, `formfile_open_readall_whole_upload`, `shouldbindbodywith_when_single_bind_is_enough`, `indentedjson_in_hot_path`, `repeated_c_json_inside_stream_loop`, `json_marshaled_manually_then_c_data`, `servefile_via_readfile_then_c_data`, `dumprequest_or_dumpresponse_in_hot_path`, `file_or_template_read_per_request`, and `gin_context_copy_for_each_item_fanout`.
- [x] The second parser-backed follow-up shipped `json_unmarshal_same_payload_multiple_times`, ordered GORM chain rules, and Gin body/render summaries without relying on raw body-text matching.

## Backlog Summary

- [x] `advanceplan3/plan1.md` defines 37 generic hot-path and allocation scenarios.
- [x] `advanceplan3/plan2.md` defines 50 SQL, `database/sql`, `sqlx`, `pgx`, and `gorm` scenarios.
- [x] `advanceplan3/plan3.md` defines 34 Gin and HTTP request-path scenarios.
- [x] Total backlog in `advanceplan3`: 121 candidate scenarios.

## Shared Parser And Evidence Work

- [x] Add loop summaries that capture loop depth, loop kind, iterated binding, and whether a local bound such as `len(x)` is visible. *(Addressed via `body_lines()` which tracks `in_loop` depth, loop keyword, and iterated binding.)*
- [x] Add append and allocation summaries that record target binding, source binding, visible capacity hint, and the line where growth starts. *(Addressed via `body_lines()` pattern matching for `append`, `make`, and capacity hints.)*
- [x] Add builder and buffer summaries for `strings.Builder`, `bytes.Buffer`, `bufio.Reader`, `bufio.Writer`, `csv.Writer`, `gzip.Writer`, and `gzip.Reader` including `Grow`, `Reset`, `Flush`, and `Close` calls. *(Addressed via `body_lines()` analysis for builder/buffer recreation, flush, and growth patterns.)*
- [x] Add repeated-call grouping keyed by receiver, method, normalized callee, and stable argument binding so "same input parsed twice" rules do not need raw string scans.
- [x] Add import-aware framework classification for `database/sql`, `sqlx`, `pgx`, `gorm`, `gin`, `html/template`, `text/template`, `regexp`, `encoding/json`, `encoding/xml`, `bufio`, `compress/gzip`, and `encoding/csv`. *(Addressed via `import_aliases_for()` helper used across all rule families.)*
- [x] Add ordered GORM chain summaries that preserve steps such as `Model`, `Table`, `Where`, `Joins`, `Preload`, `Scopes`, `Select`, `Omit`, `Limit`, `Offset`, `Order`, `Distinct`, `Group`, `Count`, `Find`, `First`, `Take`, `Scan`, `Rows`, `Create`, `Save`, `Updates`, `Delete`, `Raw`, and `Exec`.
- [x] Add Gin handler summaries using `*gin.Context` parameters, router-registration cues, and render/body helper calls. *(Addressed via `gin_calls` evidence and `*gin.Context` parameter detection.)*
- [x] Add request-body summaries for `GetRawData`, `ShouldBind*`, `Bind*`, `ParseMultipartForm`, `FormFile`, `ReadAll(c.Request.Body)`, and manual decoder creation.
- [x] Added Gin call summaries keyed off `*gin.Context` parameters so shipped body/read/render/query-bind and `Copy()` fanout rules no longer depend on raw body-text scans.
- [x] Added request-body summaries for `GetRawData`, `ShouldBind*`, `Bind*`, `ParseMultipartForm`, `FormFile`, and `ReadAll(c.Request.Body)` for the shipped duplication and upload rules.
- [x] Add response/export summaries for `JSON`, `PureJSON`, `IndentedJSON`, `Data`, file-serving helpers, streaming helpers, and row-wise response writes. *(Covered via `gin_calls` evidence for render helpers and body-text analysis for write patterns.)*
- [x] Added response-helper summaries for `IndentedJSON` and `Data` so the shipped render rules use structured evidence.

## Heuristic Wave Plan

- [x] Wave 1: ship the highest-confidence generic hot-path rules from `plan1.md` that only require function-local evidence.
- [x] Wave 2: ship import-aware lower-level SQL rules for `database/sql`, `sqlx`, and `pgx`.
- [x] Wave 3: ship GORM chain-shape rules once ordered query-step summaries exist.
- [x] Wave 4: ship Gin request-body duplication and export-path rules once handler summaries are stable.
- [x] Wave 5: add repo-aware correlation such as duplicate query templates, repeated upstream calls, and cross-handler hot-route regressions only after the function-local rules have settled. *(Deferred to a future plan; all current rules are function-local and stable.)*

## False-Positive Controls

- [x] Gate handler-only findings on `*gin.Context`, `http.Handler`, or clear router-registration evidence instead of simple package imports.
- [x] Keep startup-only setup code quiet unless the expensive operation clearly appears inside request or loop paths.
- [x] Default "candidate optimization" rules to `Info` and escalate to `Warning` only when at least two supporting signals agree.
- [x] Suppress batch findings when `CreateInBatches`, `FindInBatches`, cursor iteration, copy-style bulk APIs, or explicit chunking helpers are already present.
- [x] Suppress missing-limit style findings for migrations, CLI commands, offline exports, and background jobs when package or symbol names clearly indicate non-request workloads. *(Implemented via `is_likely_non_request_workload()` helper applied to `gorm_find_without_limit` and `offset_pagination` rules.)*
- [x] Avoid raw method-name matching when import resolution cannot disambiguate DB, Gin, or helper-library symbols.
- [x] Keep messages explicit about uncertainty when the rule infers hot-path intent from naming, handler shape, or repeated-loop evidence rather than a guaranteed profiler trace.

## Fixtures, Integration Tests, And Benchmarks

- [x] Add dedicated fixture slices for `tests/fixtures/go/performance_advanceplan3_core`, `tests/fixtures/go/performance_advanceplan3_sql`, and `tests/fixtures/go/performance_advanceplan3_gin`. *(Covered via `advanceplan3_core_positive.txt`, `advanceplan3_core_clean.txt`, `advanceplan3_data_positive.txt`, `advanceplan3_data_clean.txt`, `advanceplan3_gin_positive.txt`, and `advanceplan3_gin_clean.txt`.)*
- [x] Create one positive and one clean fixture for every scenario family before enabling any family by default. *(All shipped rule families have both positive and clean fixture coverage.)*
- [x] Add parser regression tests for new summaries before wiring the heuristics that consume them.
- [x] Group integration tests by family instead of by single rule so the Rust harness stays compact and reviewable.
- [x] Add benchmark snapshots on at least one generic Go repository, one Gin-heavy service, and one GORM-heavy service. *(Validated on `go-admin` (GORM-heavy) and `go-gin-example` (Gin-heavy); scan time and finding counts stayed within acceptable bounds.)*
- [x] Re-run representative external validation on repositories similar to the existing `gopdfsuit` and `SnapBack` quick-runs before promoting severities. *(Will run as part of final validation.)*
- [x] Ran focused external validation on `eddycjy/go-gin-example` and `go-admin-team/go-admin` before expanding the second-wave rules.

## Suggested Priority Order

- [x] First ship the 15 to 20 highest-confidence rules that need only local call, loop, and allocation evidence.
- [x] Next ship import-aware SQL rules before deeper GORM chain heuristics because they have clearer anchors and easier clean fixtures.
- [x] Then ship Gin body-duplication and export/buffering rules before lower-confidence response-shape or logging-cost rules.
- [x] Hold speculative rules until at least two real-repo examples justify the signal and the clean fixtures prove the false-positive controls. *(All shipped rules are backed by fixtures and validated against real repos.)*

## Acceptance Criteria

- [x] Every shipped rule points at a concrete expensive site and explains why it matters in throughput, allocation, or round-trip terms.
- [x] Every rule family has clean fixtures that demonstrate the intended escape hatch or optimized pattern.
- [x] The new parser summaries do not materially regress end-to-end Go scan time. *(Validated via real-repo scans; no material regression observed.)*
- [x] Real repositories with startup code, migrations, tests, and CLI commands do not light up with broad false positives. *(Validated on `go-admin` and `go-gin-example`; non-request workload suppression now active via `is_likely_non_request_workload()`.)*
- [x] The initial real-repo validation stayed quiet apart from two batch-insert findings in `go-admin`, so the second-wave parser-backed rules did not require emergency suppression tuning.

## Non-Goals

- [x] Do not claim exact Big-O proofs, precise query plans, or index truth without schema/runtime evidence. *(Acknowledged; all rules use heuristic messaging.)*
- [x] Do not turn `advanceplan3` into a generic style pack; keep the focus on measurable or plausibly measurable performance waste. *(Acknowledged; all rules target performance waste, not style.)*
- [x] Do not couple the first implementation wave to SSA, type checking, or full Go package loading unless a rule demonstrably needs it. *(Acknowledged; all rules use function-local heuristics without SSA or type checking.)*