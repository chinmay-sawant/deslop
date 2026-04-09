# Go Project-Agnostic Optimization 50-Point Checklist For v0.3.0

Date: 2026-04-09

## Objective

- Resolve the requested 50 Go architecture and optimization themes against the live rule inventory.
- Keep the result project-agnostic instead of tied to one repository review.
- Close every checklist item either by shipping a rule, reusing existing registry coverage, or explicitly keeping the theme as review guidance when a static rule would be too noisy.

## Context And Application Lifecycle

1. [x] Added `request_context_passed_to_background_task_without_detach` for request-owned contexts that still appear to flow into unmanaged background goroutines without `context.WithoutCancel(...)` or an equivalent detach.
2. [x] Added `http_server_bootstrap_without_graceful_shutdown_flow` for startup paths that call `ListenAndServe*` without visible signal ownership plus `Shutdown(...)`.
3. [x] Added `timeoutless_http_default_client_or_helper_call` for `http.DefaultClient` and timeout-less `net/http` helper usage.
4. [x] Existing coverage remains via `http_server_without_timeouts`.
5. [x] Added `context_not_first_parameter` for context-aware call paths that accept `context.Context` but not as the first non-receiver parameter.
6. [x] Added `context_stored_in_struct_field` for long-lived structs that keep `context.Context` as state.
7. [x] Added `context_withvalue_used_for_dependencies_or_large_payloads` for `context.WithValue(...)` calls that carry dependency-like objects or payload-like data instead of lightweight request metadata.
8. [x] Added `context_key_uses_exported_or_builtin_type` for built-in or exported key shapes that increase collision risk.

## Concurrency And Goroutines

9. [x] Existing coverage remains via `goroutine_without_coordination`, `goroutine_without_shutdown_path`, and `goroutine_spawn_in_loop`.
10. [x] Added `waitgroup_fanout_without_errgroup_on_error_path` for WaitGroup-based goroutine fan-out that still carries explicit error-path coordination better suited to `errgroup`.
11. [x] Existing sleep-based synchronization coverage remains via `sleep_polling`.
12. [x] Existing `blocking_call_while_locked` already covers the promotable external-IO-under-lock case, so no duplicate rule was added.
13. [x] Added `rwmutex_without_clear_read_heavy_signal` for `sync.RWMutex` usage without a clear read-heavy access pattern.
14. [x] Existing shared-map concurrency coverage remains via `race_on_shared_map`.
15. [x] Kept channel-versus-mutex ownership advice as review guidance rather than a shipped rule because current parser evidence is still too ambiguous to distinguish ownership-transfer channels from localized shared-state coordination reliably.
16. [x] Added `ci_missing_go_test_race` as a repo-level automation check.

## Memory Management And Allocations

17. [x] Existing slice and map preallocation coverage remains via `slice_append_without_prealloc_known_bound`, `slice_grow_without_cap_hint`, `nested_append_without_outer_capacity`, and `map_of_slices_prealloc`.
18. [x] Existing `sync_pool_ignored_for_frequent_small_allocs` already covers the promotable high-throughput pooling case.
19. [x] Kept pointer-versus-value return tradeoffs as review guidance because local syntax alone is not a reliable enough heap-versus-stack signal for a low-noise rule.
20. [x] Existing `map_delete_in_loop_vs_new_map` already covers the promotable map-reset churn pattern.
21. [x] Kept backing-array retention guidance as review-only because the current parser does not have reliable evidence about source buffer size and slice lifetime.
22. [x] Kept escape-analysis usage as workflow guidance rather than a shipped rule because it depends on compiler output, not source shape alone.

## Bytes, Strings, And Iteration

23. [x] Existing helper-upgrade rules already cover the stable `bytes.Trim`, `bytes.Replace`, and `bytes.ReplaceAll` promotions we can prove locally, so no broader duplicate rule was added.
24. [x] Existing string-concatenation hot-path coverage remains via `string_concat_in_loop`, `likely_n_squared_string_concat`, and `strings_builder_without_grow_known_bound`.
25. [x] Explicitly kept ASCII-only string-indexing advice as non-promoted guidance because it conflicts with the shipped rune-safety rule and ASCII guarantees are rarely provable from local syntax alone.
26. [x] Existing buffered streaming coverage remains via `bufio_reader_missing_for_small_read_loop`, `formfile_open_readall_whole_upload`, `full_dataset_load`, and related streaming rules.
27. [x] Kept regex-versus-string-helper advice as review guidance because precompiled regex hot-path intent is still too ambiguous for a low-noise static rule.

## Database Abstractions And Executions

28. [x] Existing mixed ORM and raw-SQL governance remains via `gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary` and `repository_mixes_raw_sql_and_gorm_same_method_without_adapter_boundary`.
29. [x] Existing single-record write outcome coverage remains via `repository_single_record_write_without_rows_affected_check`.
30. [x] Kept persistence-operator advice as review guidance because detecting `jsonb_set`-style partial-update opportunities reliably would currently be too datastore-specific and noisy.
31. [x] Added `db_pool_limits_not_configured_at_boot` for bootstrap paths that open long-lived DB clients without visible pool sizing or lifetime limits.
32. [x] Existing missing-`Rows.Close()` coverage remains via `rows_without_close`.
33. [x] Added `rows_iterated_without_rows_err_check` for row iteration paths that never call `Rows.Err()`.
34. [x] Added `slow_work_inside_transaction_scope` for loop-heavy or obviously slow work performed while a transaction appears open.

## Networking And HTTP Clients

35. [x] Added `http_client_allocated_per_call_without_reuse` for regular call paths that allocate `http.Client` inline instead of reusing shared client state.
36. [x] Added `http_response_body_not_drained_before_close` for ignored upstream responses that are closed without being drained or otherwise consumed.
37. [x] Added `request_body_read_without_size_limit` for handler paths that read or decode request bodies without an observed size limit.

## Design Principles And Code Structure

38. [x] Existing `single_impl_interface`, `passthrough_wrapper_interface`, and `upstream_consumed_interface_declared_in_provider_package` already cover the promotable over-abstraction cases, so no broader return-concrete-types rule was added.
39. [x] Existing consumer-owned interface placement coverage remains via `upstream_consumed_interface_declared_in_provider_package`.
40. [x] Existing panic misuse coverage remains via `panic_on_error` and `panic_for_expected_errors`.
41. [x] Existing `init()` minimization coverage remains via `init_registers_routes_or_dependencies` and `init_side_effect`.
42. [x] Existing global mutable-state coverage remains via `package_level_mutable_config_used_by_handlers_services`, `mutable_package_global`, `gin_handler_uses_global_singletons`, and related singleton rules.
43. [x] Explicitly kept feature-first package-layout guidance as non-promoted because deslop intentionally avoids universal package-layout enforcement across all Go repository shapes.

## Tooling, Error Handling, And Logging

44. [x] Existing error-wrapping coverage remains via `error_wrapping_misuse`, `fmt_errorf_without_wrap_verb`, and `string_format_for_error_wrap`.
45. [x] Existing single-owner error-logging coverage remains via `error_logged_and_returned` and `handler_and_service_both_log_same_error_chain`.
46. [x] Kept functional-options preference as review guidance because constructor optionality heuristics are currently too style-heavy and API-specific for a stable static rule.
47. [x] Existing `log_printf_for_production` and `string_format_in_structured_logger` already cover the high-signal unstructured-logging cases, so no logger-brand preference rule was added.
48. [x] Kept profiling-first guidance as workflow guidance rather than a shipped source rule because `pprof` evidence is external to the source tree.
49. [x] Kept table-driven-test preference as review guidance because table-driven tests are not universally superior and the current test-quality pack already flags the highest-signal table misuse.
50. [x] Kept zero-value usefulness as review guidance because exported API zero-value quality is still too semantic for a low-noise parser-only rule.

## Resolution Summary

- Resolved as new shipped rules: 16.
- Resolved by existing shipped registry coverage: 22.
- Resolved as explicit review-guidance-only items that are intentionally not promoted to static rules today: 12.
