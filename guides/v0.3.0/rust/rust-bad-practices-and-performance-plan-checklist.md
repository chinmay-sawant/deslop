# Rust Bad Practices and Performance Rule Plan Checklist For v0.3.0

Date: 2026-04-11

## Status

- [x] This is a planning backlog only; no rule candidates in this document are marked shipped yet.
- [x] Prioritize repo-agnostic detections tied to the current direct Rust dependency stack before broader generic Rust guidance.
- [x] Keep rules focused on bad practices, correctness footguns, and performance-oriented unoptimizations that can be detected from manifests, imports, local AST evidence, call chains, and simple repo-wide summaries.
- [x] Avoid duplicating existing Rust rules already shipped under `src/heuristics/rust` and `src/rules/catalog/rust`.

## Objective

- [x] Add a Rust v0.3.0 checklist for future bad-practice and performance rule work.
- [x] Start with external crates used by this project so the first implementation pass improves the scanner against the same patterns it already exercises.
- [x] Keep every rule candidate useful across Rust repositories, not just this repository.
- [x] Prefer conservative, reviewable signals over style-only preferences.
- [x] Treat async, unsafe, FFI, manifests, parser pipelines, serialization, CLI boundaries, parallelism, and I/O as the highest-value Rust rule zones.

## Current Project External Library Inventory

Production dependencies observed in `Cargo.toml`:

- [x] `tree-sitter`, `tree-sitter-rust`, `tree-sitter-go`, and `tree-sitter-python` for parser backends and language grammars.
- [x] `rayon` for parallel scan and evaluation pipelines.
- [x] `ignore` for repository walking and ignore-file-aware traversal.
- [x] `clap` for CLI parsing and subcommand handling.
- [x] `serde`, `serde_json`, and `toml` for configuration, rules, reports, and manifest/config parsing.
- [x] `anyhow` and `thiserror` for application and library error surfaces.
- [x] `libc` for platform-specific filesystem flags.

Development and fuzzing dependencies observed in `Cargo.toml` and `fuzz/Cargo.toml`:

- [x] `proptest` for property tests.
- [x] `tempfile` for isolated filesystem tests and benchmark workspaces.
- [x] `libfuzzer-sys` for fuzz targets.

## Existing Rust Coverage To Avoid Restating

- [x] Do not duplicate general non-test `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, `unreachable`, `dbg`, or unsafe-without-comment hygiene rules.
- [x] Do not duplicate current serde boundary rules for `serde(default)` on required-looking fields, `serde(flatten)` catch-alls, missing `deny_unknown_fields`, untagged boundary enums, or secret serialization/deserialization.
- [x] Do not duplicate current performance rules for unbuffered file writes, `.lines()` allocation, default `HashMap` hasher hot paths, lock-across-await, blocking I/O in async, boxed vector pointer chasing, `Path::join` with absolute paths, or UTF-8 validation hot paths.
- [x] Do not duplicate current runtime rules for per-call Tokio runtime creation, request-path environment reads, router/channel setup on request paths, heavy clone-in-loop, workspace resolver absence, or runtime builder-in-loop.
- [x] Do not duplicate current unsafe soundness rules for `get_unchecked`, `from_raw_parts`, `set_len`, `assume_init`, `transmute`, raw pointer casts, or aliasing assumptions.
- [x] Do not duplicate current boundary rules for internal/public `anyhow` surfaces, unbounded `read_to_string`, check-then-open filesystem access, secret equality, narrowing casts, or manual temporary directory lifecycle.

## Section Summary

| Priority | Theme | Rule candidates / tasks |
| --- | --- | ---: |
| 0 | Direct crate usage: tree-sitter, rayon, ignore, serde/toml/json, error crates, clap, libc, test/fuzz crates | 96 |
| 1 | General Rust memory, allocation, and collection unoptimizations | 18 |
| 2 | General Rust I/O, path, and resource-management unoptimizations | 12 |
| 3 | General Rust concurrency and synchronization bad practices | 12 |
| 4 | Cargo, feature, build, and packaging performance hygiene | 10 |
| 5 | Implementation, fixture, test, and promotion plan | 20 |
| **Total** | | **168** |

---

## 1. tree-sitter Parser Pipeline Rules

Use these only when a crate imports `tree_sitter`, `tree_sitter_*`, or exposes parser-like module names.

- [x] `rust_tree_sitter_parser_created_per_file_without_reuse`: flag `tree_sitter::Parser::new()` construction inside per-file or per-function loops where a language-specific parser could be reused per worker.
- [x] `rust_tree_sitter_set_language_repeated_inside_hot_loop`: flag repeated `parser.set_language(...)` calls inside loops that parse the same language repeatedly.
- [x] `rust_tree_sitter_language_conversion_inside_loop`: flag repeated `tree_sitter_rust::LANGUAGE.into()` or equivalent grammar conversion in inner loops instead of caching the converted language.
- [x] `rust_tree_sitter_query_compiled_per_call`: flag `tree_sitter::Query::new(...)` inside functions called per file or per node, where query compilation should be cached.
- [x] `rust_tree_sitter_parse_result_unwrapped`: flag `parser.parse(...).unwrap()` or `expect(...)` at parser boundaries where parse cancellation or invalid parser state should become a typed finding/error.
- [x] `rust_tree_sitter_error_tree_ignored`: flag parser pipelines that use the root node without checking `tree.root_node().has_error()` or equivalent syntax-error evidence.
- [x] `rust_tree_sitter_recursive_walk_without_depth_guard`: flag recursive AST walkers over repository input without an explicit depth guard, iterative cursor, or stack budget.
- [x] `rust_tree_sitter_node_text_redecoded_in_nested_loop`: flag repeated `node.utf8_text(source)` or source slicing for the same node inside nested traversal loops.
- [x] `rust_tree_sitter_child_lookup_by_field_name_in_hot_walk`: flag repeated `child_by_field_name("...")` string lookups in hot recursive walkers where field ids, cursors, or one-pass extraction would avoid repeated lookup work.
- [x] `rust_tree_sitter_descendant_for_point_range_in_loop`: flag repeated descendant/range queries inside loops over sibling nodes, which can turn a tree walk into avoidable quadratic traversal.
- [x] `rust_tree_sitter_collects_all_captures_before_filtering`: flag query capture pipelines that collect every capture into `Vec` and then filter locally instead of filtering as captures are visited.
- [x] `rust_tree_sitter_byte_offset_used_as_char_index`: flag byte offsets from `start_byte` or `end_byte` used as character counts, `chars().nth(...)`, or display-column positions without conversion.
- [x] `rust_tree_sitter_old_tree_discarded_in_reparse_loop`: flag repeated parse loops over the same buffer that always pass `None` instead of reusing an old tree for incremental parsing.
- [x] `rust_tree_sitter_parser_shared_global_with_lock`: flag global `Mutex<Parser>` or `RwLock<Parser>` shared across threads where per-thread parser ownership would avoid lock contention and parser state coupling.

## 2. rayon Parallel Pipeline Rules

Use these only when a crate imports `rayon`, `rayon::prelude::*`, or calls `par_iter`, `into_par_iter`, `par_bridge`, `join`, or `scope`.

- [x] `rust_rayon_nested_parallel_iterators`: flag nested `par_iter`, `into_par_iter`, or `par_bridge` calls that can oversubscribe worker threads or fragment work.
- [x] `rust_rayon_mutex_push_in_parallel_loop`: flag `Mutex<Vec<_>>`, `RwLock<Vec<_>>`, or shared collection mutation inside `for_each` where `map/filter_map/collect/reduce` would avoid lock contention.
- [x] `rust_rayon_collect_all_then_filter_sequentially`: flag `par_iter().map(...).collect::<Vec<_>>()` immediately followed by sequential filtering or flattening that could stay in the parallel pipeline.
- [x] `rust_rayon_sequential_collect_then_par_iter`: flag collecting an intermediate `Vec` only to immediately call `par_iter` or `into_par_iter` when the producer could feed the parallel stage directly.
- [x] `rust_rayon_par_bridge_on_small_or_indexed_iterator`: flag `par_bridge()` over known indexed collections or tiny iterators where normal `par_iter` or sequential iteration is likely cheaper.
- [x] `rust_rayon_blocking_io_in_cpu_parallel_pool`: flag filesystem, network, or process I/O inside Rayon CPU-bound closures unless the project documents that Rayon owns the I/O concurrency budget.
- [x] `rust_rayon_heavy_clone_per_item`: flag repeated `.clone()` of config, source text, rule catalogs, AST summaries, or large state inside parallel closures.
- [x] `rust_rayon_large_move_capture`: flag `move` closures in parallel iterators that capture large owned values rather than borrowing or sharing cheap handles.
- [x] `rust_rayon_ordering_mutex_for_result_stability`: flag parallel closures that serialize through `Mutex<BTreeMap<_>>`, `Mutex<Vec<_>>`, or sequence counters just to regain output order.
- [x] `rust_rayon_global_pool_built_by_library_code`: flag `ThreadPoolBuilder::build_global()` outside a binary entry point, test harness, or explicit process bootstrap.
- [x] `rust_rayon_custom_pool_created_per_call`: flag `ThreadPoolBuilder::build()` inside frequently-called functions instead of process-level or subsystem-level pool ownership.
- [x] `rust_rayon_unhandled_panic_in_parallel_scan`: flag `unwrap`, `expect`, or panicking conversions inside parallel scan/evaluation closures where one bad file can abort the whole batch.
- [x] `rust_rayon_flat_map_allocates_nested_vectors`: flag parallel map stages that return `Vec<Vec<T>>` and later flatten, where `flat_map_iter`, `reduce`, or direct collection can reduce allocation churn.
- [x] `rust_rayon_parallelism_for_trivial_per_item_work`: flag parallel iteration over simple string predicates or metadata checks when there is no expensive per-item work and the input is below a conservative threshold.

## 3. ignore Repository Walker Rules

Use these only when a crate imports `ignore` or constructs `ignore::WalkBuilder`.

- [x] `rust_ignore_walker_disables_standard_filters_without_policy`: flag `.ignore(false)`, `.git_ignore(false)`, `.git_exclude(false)`, `.hidden(false)`, or `.parents(false)` unless paired with a named user option or security review note.
- [x] `rust_ignore_follow_links_without_cycle_or_root_policy`: flag `.follow_links(true)` without visible cycle, root containment, or same-filesystem policy.
- [x] `rust_ignore_walk_error_silently_discarded`: flag walk result handling that uses `filter_map(Result::ok)` or `ok()?` without recording ignored traversal errors.
- [x] `rust_ignore_direntry_unwrapped`: flag `entry.unwrap()` or `expect(...)` on walker entries in production scan code.
- [x] `rust_ignore_reads_file_before_file_type_check`: flag read/open operations on walker paths before checking `file_type().is_file()` or equivalent.
- [x] `rust_ignore_walker_rebuilt_inside_directory_loop`: flag `WalkBuilder::new(...)` construction inside recursive or per-directory loops.
- [x] `rust_ignore_override_pattern_unwraps_user_input`: flag `OverrideBuilder::add(...)` or glob override construction from user input followed by `unwrap` or `expect`.
- [x] `rust_ignore_parallel_walker_unbounded_accumulation`: flag parallel walker callbacks that push every path into an unbounded `Vec` or channel before processing.
- [x] `rust_ignore_filter_entry_allocates_path_string_per_node`: flag `filter_entry` closures that allocate `String` or call `to_string_lossy()` for every node when path component checks would work.
- [x] `rust_ignore_sort_by_path_after_full_walk`: flag full repository path collection followed by sort before processing where deterministic output could be produced after finding collection instead.

## 4. serde, serde_json, and toml Rules

Use these only when a crate imports `serde`, `serde_json`, `toml`, or derives serde traits.

- [x] `rust_serde_json_value_internal_hot_path`: flag `serde_json::Value` used as an internal data model in non-boundary modules where typed structs would reduce dynamic lookups and runtime errors.
- [x] `rust_serde_json_indexing_without_type_guard`: flag `value["key"]`, `value[index]`, or chained indexing without typed deserialization, `.get(...)`, or null/type checks.
- [x] `rust_serde_json_to_string_pretty_in_machine_path`: flag `serde_json::to_string_pretty` in non-human-output hot paths, APIs, or report generation loops.
- [x] `rust_serde_json_clone_value_in_loop`: flag `serde_json::Value::clone()` or cloned JSON maps inside loops.
- [x] `rust_serde_json_from_str_after_unbounded_read`: flag `serde_json::from_str` or `from_slice` fed by unbounded file/network reads rather than size-limited input.
- [x] `rust_serde_json_roundtrip_conversion`: flag `to_value` followed by `from_value`, or `to_string` followed by `from_str`, when a direct conversion or typed boundary is available.
- [x] `rust_serde_json_whole_array_loaded_for_streaming_input`: flag deserializing large JSON arrays into `Vec<T>` in reader-like code where streaming deserialization could reduce peak memory.
- [x] `rust_serde_custom_deserialize_panics`: flag custom `Deserialize` impls or visitors that call `unwrap`, `expect`, `panic`, or unchecked indexing.
- [x] `rust_serde_borrow_missing_for_large_readonly_payload`: flag large readonly payload structs that deserialize owned `String` or `Vec<u8>` in hot paths where `Cow<'de, str>` or borrowed fields could avoid copies.
- [x] `rust_serde_skip_serializing_secret_without_deserialize_guard`: flag secret-like fields skipped during serialization but still accepted during deserialization without validation/redaction policy.
- [x] `rust_serde_wire_enum_missing_stable_rename_policy`: flag public wire-facing enums that derive `Serialize` or `Deserialize` without `rename_all` or explicit renames.
- [x] `rust_toml_value_config_boundary`: flag application config parsed into `toml::Value` and queried dynamically instead of deserializing into a typed config struct.
- [x] `rust_toml_parse_in_hot_path`: flag `toml::from_str`, `toml::from_slice`, or `str::parse::<toml::Value>()` inside request paths, scan loops, or repeated functions.
- [x] `rust_toml_config_without_unknown_field_rejection`: flag TOML-facing config structs that derive `Deserialize` without `#[serde(deny_unknown_fields)]` when they appear to represent project configuration.
- [x] `rust_toml_manifest_parse_without_size_limit`: flag manifest/config parsing helpers that read and parse TOML without a byte limit.
- [x] `rust_serde_default_masks_parse_error`: flag broad `unwrap_or_default`, `Default::default`, or `#[serde(default)]` use around configuration fields that look required for correctness.

## 5. anyhow and thiserror Rules

Use these only when a crate imports `anyhow`, `thiserror`, or exposes project error types.

- [x] `rust_anyhow_context_missing_on_boundary_io`: flag `?` on filesystem, environment, process, parser, config, or network calls in CLI/boundary code without `.context(...)` or `.with_context(...)`.
- [x] `rust_anyhow_eager_format_context`: flag `.context(format!(...))` where `.with_context(|| format!(...))` would avoid allocation on the success path.
- [x] `rust_anyhow_error_string_matching`: flag branching on `err.to_string()`, `format!("{err}")`, or message substrings instead of typed errors or downcasts.
- [x] `rust_anyhow_downcast_without_fallback_context`: flag `downcast_ref`/`downcast` handling that drops the original context or returns a generic fallback.
- [x] `rust_anyhow_bail_in_low_level_library_module`: flag `anyhow::bail!` in domain, parser, storage, or library modules that should usually expose typed error variants.
- [x] `rust_thiserror_variant_wraps_source_without_source_attr`: flag error variants with source-like fields that lack `#[source]`, `#[from]`, or transparent wrapping.
- [x] `rust_thiserror_display_leaks_secret_field`: flag `#[error(...)]` format strings that interpolate token, password, secret, key, auth, cookie, or credential fields.
- [x] `rust_thiserror_stringly_typed_variant`: flag error enum variants whose only payload is `String` or `&'static str` and whose name does not encode a specific error kind.
- [x] `rust_thiserror_transparent_on_contextual_variant`: flag `#[error(transparent)]` variants that also carry context-like fields or lose higher-level operation details.
- [x] `rust_error_logged_and_returned`: flag functions that log an error and then return the same error upward, causing duplicate logging at callers.
- [x] `rust_result_ignored_with_let_underscore`: flag `let _ = fallible_call()` outside cleanup, telemetry, or best-effort contexts.
- [x] `rust_question_mark_after_partial_side_effect_without_cleanup`: flag `?` after partially mutating files, shared state, transactions, or output buffers without rollback or cleanup.

## 6. clap CLI Boundary Rules

Use these only when a crate imports `clap` or derives `Parser`, `Subcommand`, `Args`, or `ValueEnum`.

- [x] `rust_clap_closed_set_manual_string_match`: flag CLI string arguments that are manually matched against a fixed set instead of using `ValueEnum` or `value_parser`.
- [x] `rust_clap_path_arg_used_without_validation`: flag `PathBuf` or path-like CLI values used for reads/writes without root containment, canonicalization strategy, or symlink policy.
- [x] `rust_clap_default_value_manual_parse`: flag `default_value` strings that are parsed manually later instead of typed `value_parser` or typed fields.
- [x] `rust_clap_unbounded_vec_arg_on_scan_path`: flag `Vec<String>` or variadic CLI arguments that feed scan/filter work without a limit, deduplication, or validation.
- [x] `rust_clap_secret_arg_derive_debug`: flag CLI structs deriving `Debug` while containing token, password, secret, key, cookie, or auth fields.
- [x] `rust_clap_opposing_flags_without_conflicts`: flag pairs such as `--json/--text`, `--include/--exclude`, or `--enable/--disable` without `conflicts_with`, `overrides_with`, or explicit precedence.
- [x] `rust_clap_rule_id_arg_without_catalog_validation`: flag rule-id-like CLI arguments accepted as strings without checking against the known rule registry.
- [x] `rust_clap_subcommand_reloads_config_in_each_branch`: flag repeated config loading inside every subcommand branch instead of a shared pre-dispatch normalization step.
- [x] `rust_clap_parse_called_below_main`: flag `Cli::parse()` or `try_parse()` called from library code or tests without dependency injection, making the code hard to reuse and test.
- [x] `rust_clap_env_var_without_redaction_policy`: flag `#[arg(env = "...")]` on secret-like fields without redacted display/report behavior.

## 7. libc and FFI Boundary Rules

Use these only when a crate imports `libc`, declares `extern` blocks, or contains FFI-adjacent unsafe code.

- [x] `rust_libc_call_without_platform_cfg`: flag direct `libc::*` calls without `#[cfg(unix)]`, `#[cfg(target_os = "...")]`, or a platform abstraction.
- [x] `rust_libc_return_value_not_checked`: flag libc calls whose integer or pointer return value is ignored or not checked for `-1`, null, or documented failure sentinels.
- [x] `rust_libc_errno_read_after_intervening_call`: flag errno access after another call that could overwrite the original failure cause.
- [x] `rust_libc_raw_fd_lifetime_escape`: flag `as_raw_fd()` results stored, returned, or moved into long-lived structs while the owning file/socket may drop.
- [x] `rust_libc_into_raw_fd_without_reclaim`: flag `into_raw_fd()` without a visible `from_raw_fd`, `OwnedFd`, or close handoff.
- [x] `rust_libc_cstring_unwrap_on_external_input`: flag `CString::new(user_input).unwrap()` or `expect(...)` where embedded NUL bytes should become a recoverable error.
- [x] `rust_libc_open_without_cloexec`: flag direct `libc::open` or equivalent without `O_CLOEXEC` in programs that may spawn child processes.
- [x] `rust_libc_no_follow_without_eloop_handling`: flag `O_NOFOLLOW` usage without explicit symlink-loop or unsupported-platform error handling.
- [x] `rust_ffi_extern_block_without_abi_comment`: flag FFI declarations without a nearby note about ABI, ownership, nullability, or lifetime expectations.
- [x] `rust_ffi_slice_from_raw_parts_without_length_guard`: flag safe wrappers around raw pointer and length pairs that do not validate null pointers, maximum length, or alignment before slice construction.

## 8. proptest, tempfile, and libfuzzer Rules

Use these for test and fuzz code only.

- [x] `rust_tempfile_named_path_used_after_drop`: flag `NamedTempFile::path()` stored or returned while the temporary file owner may drop before the path is used.
- [x] `rust_tempfile_persist_without_cleanup_assertion`: flag `persist`, `keep`, or `into_temp_path().keep()` in tests without cleanup or an assertion that the file must survive.
- [x] `rust_tempfile_predictable_name_in_shared_tmp`: flag tests that combine `std::env::temp_dir()` with predictable filenames instead of `tempfile`.
- [x] `rust_tempfile_builder_prefix_from_test_name_only`: flag temporary directories/files that use only a fixed prefix and shared parent for parallel tests without unique isolation.
- [x] `rust_proptest_unbounded_string_or_vec_strategy`: flag `".*"`, `any::<String>()`, or unbounded `vec(...)` strategies in parser/scanner tests without a size cap.
- [x] `rust_proptest_assume_filters_most_cases`: flag multiple `prop_assume!` calls or assume-heavy strategies that likely discard most generated cases.
- [x] `rust_proptest_strategy_recreates_expensive_fixture`: flag strategies that rebuild repositories, parsers, or large fixtures for each case rather than sharing setup safely.
- [x] `rust_proptest_no_regression_case_for_parser_crash`: flag parser property tests without a regression fixture or seed-capture path for minimized failures.
- [x] `rust_libfuzzer_target_unwraps_parse_or_utf8`: flag fuzz targets that call `unwrap`/`expect` on parser, UTF-8, TOML, JSON, or path conversion results.
- [x] `rust_libfuzzer_target_allocates_unbounded_string`: flag fuzz targets that convert arbitrary byte slices to owned strings without a size cap or lossy/borrowed strategy.

---

## 9. General Memory, Allocation, and Collection Rules

- [x] `rust_collect_then_single_iteration`: flag `.collect::<Vec<_>>()` immediately followed by one `for` loop, `.iter().any`, `.iter().find`, or `.len()` where streaming would avoid allocation.
- [x] `rust_vec_push_without_capacity_from_known_bound`: flag `Vec::new()` followed by pushes inside a loop over a known bounded iterator without `with_capacity`.
- [x] `rust_string_push_without_capacity_from_known_bound`: flag `String::new()` plus repeated `push_str`, `push`, or `write!` with a known bound and no capacity reservation.
- [x] `rust_format_macro_inside_append_loop`: flag `format!` in loops when appending to an existing `String` would avoid temporary allocation.
- [x] `rust_to_string_on_str_in_loop`: flag `.to_string()` or `String::from(...)` on borrowed strings inside loops where borrowing would satisfy the callee.
- [x] `rust_clone_to_satisfy_borrow_in_loop`: flag `.clone()` in loops immediately passed by reference or consumed only for read-only access.
- [x] `rust_regex_compiled_in_loop`: flag `Regex::new(...)` inside loops or hot functions when the `regex` crate is imported.
- [x] `rust_sort_then_first_or_last`: flag sorting an entire collection only to take min/max-like first or last values.
- [x] `rust_vec_remove_zero_in_loop`: flag repeated `Vec::remove(0)` in loops where `VecDeque` or index traversal would avoid shifting.
- [x] `rust_hashmap_contains_then_insert`: flag `contains_key` followed by `insert` or `get_mut` where `entry` would avoid duplicate hashing.
- [x] `rust_iterator_nth_inside_loop`: flag `.nth(i)` or repeated indexed traversal over non-indexed iterators inside loops.
- [x] `rust_drain_collect_then_drop`: flag `drain(..).collect::<Vec<_>>()` followed by drop or one-pass processing that can operate directly on the drain iterator.
- [x] `rust_bytes_to_vec_for_readonly_use`: flag `.to_vec()` on byte slices that are only read afterward.
- [x] `rust_cow_to_owned_without_mutation`: flag `Cow::to_mut`, `to_owned`, or `into_owned` where the owned value is never mutated or stored past the borrow lifetime.
- [x] `rust_option_clone_then_unwrap_or`: flag cloning `Option<T>` or `Result<T, E>` only to unwrap/default instead of borrowing with `as_ref`, `as_deref`, or `map`.
- [x] `rust_large_enum_variant_without_boxing`: flag enums with one much larger variant causing every enum value to carry the largest layout.
- [x] `rust_boxed_trait_object_in_inner_loop`: flag repeated `Box<dyn Trait>` allocation inside loops where generics, enum dispatch, or object reuse may be better.
- [x] `rust_iterator_chain_allocates_intermediate_strings`: flag iterator chains that map every item through `format!`, `to_string`, or JSON conversion before a simple predicate or grouping.

## 10. General I/O, Path, and Resource Rules

- [x] `rust_read_to_string_for_line_scan`: flag whole-file `read_to_string` or `fs::read_to_string` followed only by line scanning or predicate checks.
- [x] `rust_file_open_in_loop_without_buffered_reader`: flag opening and reading files inside loops without `BufReader`, batching, or reuse.
- [x] `rust_flush_inside_write_loop`: flag `flush()` inside per-item write loops unless the code is interactive terminal output.
- [x] `rust_create_dir_all_per_file`: flag `create_dir_all` inside loops for the same parent directory.
- [x] `rust_metadata_called_repeatedly_same_path`: flag repeated `metadata`, `symlink_metadata`, `exists`, or `is_file` checks for the same path in one function.
- [x] `rust_path_to_string_lossy_in_hot_loop`: flag `to_string_lossy`, `display().to_string`, or path formatting inside repository/file traversal loops.
- [x] `rust_stdout_println_in_library_or_hot_path`: flag `println`, `eprintln`, or direct stdout/stderr writes inside library code, scan loops, or reusable components.
- [x] `rust_read_dir_collect_sort_before_filter`: flag `read_dir` entries collected and sorted before filtering by type/extension.
- [x] `rust_file_handle_returned_without_close_owner_contract`: flag functions that return raw file handles, descriptors, or paths tied to temporary resources without documenting ownership.
- [x] `rust_blocking_process_output_read_unbounded`: flag `Command::output()` or piped process reads where stdout/stderr may be large and no size bound exists.
- [x] `rust_path_canonicalize_in_scan_inner_loop`: flag repeated canonicalization for every path in a hot repository traversal when relative normalized paths would be enough.
- [x] `rust_os_string_lossy_conversion_before_filter`: flag lossy path/string conversions before simple extension, file-name, or component filters.

## 11. General Concurrency and Synchronization Rules

- [x] `rust_mutex_guard_lives_through_cpu_heavy_work`: flag lock guards that remain live across sorting, parsing, serialization, filesystem I/O, or large loops.
- [x] `rust_rwlock_write_guard_for_readonly_access`: flag write locks where the guarded value is only read.
- [x] `rust_mutex_lock_unwrap_panics_on_poison`: flag `lock().unwrap()` or `write().unwrap()` in production code where poison recovery or contextual error reporting would be safer.
- [x] `rust_atomic_seqcst_without_comment`: flag `Ordering::SeqCst` in non-trivial code without a nearby comment explaining the synchronization requirement.
- [x] `rust_unbounded_channel_in_producer_loop`: flag unbounded channel sends inside loops or request paths without backpressure or shutdown policy.
- [x] `rust_thread_spawn_in_loop_without_join_limit`: flag `std::thread::spawn` in loops without a join handle collection limit, pool, or semaphore.
- [x] `rust_arc_clone_inside_inner_loop`: flag repeated `Arc::clone` in inner loops when a borrowed reference or cloned handle outside the loop would work.
- [x] `rust_mpsc_receiver_iter_without_shutdown_signal`: flag blocking receiver iteration without timeout, close path, cancellation, or sentinel.
- [x] `rust_condvar_wait_without_predicate_loop`: flag `Condvar::wait` not wrapped in a predicate loop.
- [x] `rust_sleep_polling_loop`: flag `thread::sleep` or runtime sleep in polling loops without backoff, notification, or timeout ownership.
- [x] `rust_join_handle_dropped_after_spawn`: flag spawned threads whose join handles are immediately dropped outside explicit detached-worker patterns.
- [x] `rust_once_lock_initializes_fallible_resource_with_unwrap`: flag `OnceLock`, `LazyLock`, or lazy initialization that unwraps fallible setup instead of returning initialization errors at bootstrap.

## 12. Cargo, Feature, Build, and Packaging Rules

- [x] `rust_manifest_wildcard_dependency_version`: flag `*` dependency versions or unconstrained git/path dependencies outside local workspace development.
- [x] `rust_manifest_dependency_default_features_unreviewed`: flag heavy dependencies with default features enabled when only a narrow feature set appears to be used.
- [x] `rust_manifest_duplicate_direct_dependency_versions`: flag the same crate required at multiple direct versions across workspace members.
- [x] `rust_manifest_dev_dependency_used_in_src`: flag dev-only crates such as `tempfile`, `proptest`, or fuzz helpers imported from production `src` code.
- [x] `rust_manifest_build_dependency_used_at_runtime`: flag build-dependencies imported from runtime code or runtime dependencies used only by `build.rs`.
- [x] `rust_manifest_workspace_dependency_not_centralized`: flag multi-crate workspaces that repeat dependency versions instead of using `[workspace.dependencies]`.
- [x] `rust_manifest_release_lto_missing_for_cli_binary`: flag CLI/binary crates with release profiles that omit any LTO setting when binary size or startup matters.
- [x] `rust_manifest_bench_or_fuzz_target_in_default_members`: flag fuzz/bench crates included in default workspace members without an explicit opt-in.
- [x] `rust_build_script_missing_rerun_if_changed`: flag `build.rs` files that read files, env vars, or external commands without `cargo:rerun-if-changed` or `cargo:rerun-if-env-changed`.
- [x] `rust_build_script_network_or_git_call`: flag build scripts that invoke network, git, curl, package managers, or shell commands that make builds non-hermetic.

---

## Implementation Plan

- [x] Add a manifest-aware Rust analysis pass that can inspect root and workspace `Cargo.toml` files without tying findings to a single source file.
- [x] Extend Rust parser evidence with crate import summaries, call-chain summaries, derive/attribute summaries, loop-local call summaries, and closure capture hints.
- [x] Add dependency-gated rule families so crate-specific rules only run when the relevant dependency is present in the manifest or imported in source.
- [x] Add hot-path hints based on loops, parser traversal functions, scan/evaluate module names, request-handler names, CLI dispatch functions, and repeated repository walking.
- [x] Add conservative resource-flow helpers for owned temporary files, raw file descriptors, parser objects, lock guards, and spawned thread/task handles.
- [x] Keep initial detections syntax-first; defer dataflow-heavy rules until parser summaries and evidence contracts prove stable.
- [x] Prefer one rule family per implementation module instead of growing `src/heuristics/rust/performance.rs` into an oversized catch-all.

## Fixture And Test Plan

- [x] Add positive and clean fixtures under `tests/fixtures/rust/bad_practices`.
- [x] Keep direct-crate fixtures small and self-contained so they do not require compiling external crates.
- [x] Add integration coverage under `tests/integration_scan/rust` grouped by dependency family.
- [x] Add manifest fixtures that exercise Cargo-only rules without requiring Rust source files.
- [x] Add clean examples for each crate-specific rule to prove dependency presence alone does not fire.
- [x] Add suppression tests for at least one rule in each new family.
- [x] Add regression coverage ensuring shipped v0.2 and earlier Rust rule ids still behave unchanged.

## Promotion Policy

- [x] Promote a candidate only when the rule can identify an actionable local code smell with low false-positive risk.
- [x] Keep ambiguous performance advice as review guidance unless the code shape clearly indicates repeated work, avoidable allocation, blocking behavior, or resource leakage.
- [x] Require every promoted crate-specific rule to include a dependency/import gate and at least one clean fixture.
- [x] Require every manifest rule to handle missing, partial, invalid, and workspace-inherited Cargo configuration.
- [x] Prefer `Info` severity for optimization hints unless the pattern risks correctness, data loss, panic, deadlock, resource exhaustion, or security exposure.
- [x] Document any rule intentionally skipped because the current Rust parser evidence cannot support it without noisy inference.
