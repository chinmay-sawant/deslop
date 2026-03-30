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

### Naming and abstraction signals

- `generic_name`: function names that are overly generic without stronger contextual signals.
- `overlong_name`: very long identifiers with too many descriptive tokens.

### Typing signals

- `weak_typing`: signatures that rely on `any` or `interface{}`.

### Error-handling signals

- `dropped_error`: blank identifier assignments that discard an `err`-like value.
- `panic_on_error`: `err != nil` branches that jump straight to `panic` or `log.Fatal` style exits.
- `error_wrapping_misuse`: `fmt.Errorf` calls that reference `err` without `%w`.

### Comment-style signals

- `comment_style_title_case`: heading-like Title Case doc comments.
- `comment_style_tutorial`: tutorial-style comments that narrate obvious implementation steps.

### Security signals

- `weak_crypto`: direct use of weak standard-library crypto packages such as `crypto/md5`, `crypto/sha1`, `crypto/des`, and `crypto/rc4`.
- `hardcoded_secret`: secret-like identifiers assigned direct string literals instead of environment or secret-manager lookups.
- `sql_string_concat`: query execution calls where SQL is constructed dynamically with concatenation or `fmt.Sprintf`.

### Rust-specific signals

- `todo_macro_leftover`: `todo!()` left in non-test Rust code.
- `unimplemented_macro_leftover`: `unimplemented!()` left in non-test Rust code.
- `dbg_macro_leftover`: `dbg!()` left in non-test Rust code.
- `panic_macro_leftover`: `panic!()` left in non-test Rust code.
- `unreachable_macro_leftover`: `unreachable!()` left in non-test Rust code.
- `unwrap_in_non_test_code`: `.unwrap()` used in non-test Rust code.
- `expect_in_non_test_code`: `.expect(...)` used in non-test Rust code.
- `unsafe_without_safety_comment`: `unsafe fn` or `unsafe` block without a nearby `SAFETY:` comment. The current nearby-comment policy accepts a `SAFETY:` comment on the same line or within the previous two lines.

### Rust async and runtime signals

- `rust_blocking_io_in_async`: blocking I/O or other blocking work observed in async Rust code.
- `rust_lock_across_await`: a lock appears to be held across an `.await` boundary.
- `rust_async_std_mutex_await`: `std::sync::Mutex` appears to be held across `.await` in async code.
- `rust_async_hold_permit_across_await`: a permit or pooled resource may be held across `.await`.
- `rust_async_spawn_cancel_at_await`: async work is spawned without an obvious cancellation path.
- `rust_async_missing_fuse_pin`: a `select!` loop appears to reuse futures without fuse or pin markers.
- `rust_async_recreate_future_in_select`: a `select!` loop may recreate futures instead of reusing long-lived ones.
- `rust_async_monopolize_executor`: an async function may monopolize the executor with blocking work and no `.await`.
- `rust_async_blocking_drop`: a `Drop` implementation does blocking work that can surface in async contexts.
- `rust_async_invariant_broken_at_await`: related state mutations appear split around an `.await` boundary.
- `rust_async_lock_order_cycle`: conflicting lock acquisition order suggests a lock-order cycle.

### Rust performance and layout signals

- `rust_unbuffered_file_writes`: file-like writes performed inside a loop without buffering or batching.
- `rust_lines_allocate_per_line`: `.lines()` iteration used inside a loop where per-item allocation may matter.
- `rust_hashmap_default_hasher`: `HashMap` default-hasher construction in a likely hot path.
- `rust_tokio_mutex_unnecessary`: `tokio::sync::Mutex` used in a fully synchronous critical path with no `.await`.
- `rust_blocking_drop`: a `Drop` implementation performs blocking work.
- `rust_pointer_chasing_vec_box`: pointer-heavy boxed vector-style storage that may hurt cache locality.
- `rust_path_join_absolute`: `Path::join` used with an absolute segment that discards the existing base path.
- `rust_utf8_validate_hot_path`: UTF-8 validation appears in a likely hot path and may deserve profiling.
- `rust_large_future_stack`: large allocations may be captured across await points and bloat future size.
- `rust_aos_hot_path`: repeated struct-field dereferences inside a loop may indicate an array-of-structs hot path.

### Rust API surface signals

- `rust_public_anyhow_result`: public library-facing APIs that return `anyhow`-style `Result` types instead of a clearer domain error surface.
- `rust_public_box_dyn_error`: public APIs that expose `Box<dyn Error>` rather than a clearer error contract.
- `rust_borrowed_string_api`: public signatures that borrow `&String` where `&str` would be more general.
- `rust_borrowed_vec_api`: public signatures that borrow `&Vec<T>` where `&[T]` would better preserve flexibility.
- `rust_borrowed_pathbuf_api`: public signatures that borrow `&PathBuf` where `&Path` is the more general contract.
- `rust_public_bool_parameter_api`: public functions or methods that expose a raw boolean mode switch in the signature.

### Rust shared-state and interior-mutability signals

- `rust_pub_interior_mutability_field`: public structs that expose `Mutex`, `RwLock`, `RefCell`, `Cell`, or similar interior-mutable fields directly.
- `rust_global_lock_state`: `static`, `Lazy`, or `OnceLock` globals that wrap mutable shared state in lock-based containers.
- `rust_arc_mutex_option_state`: `Arc<Mutex<Option<T>>>`-style state bags that hide lifecycle state behind nested mutation layers.
- `rust_mutex_wrapped_collection`: collection-plus-lock fields embedded directly in public or central state structs.
- `rust_rc_refcell_domain_model`: domain-style structs built around `Rc<RefCell<T>>` instead of clearer ownership boundaries.

### Rust serde and wire-contract signals

- `rust_serde_untagged_enum_boundary`: boundary-facing enums that derive `#[serde(untagged)]`, making wire formats easier to confuse.
- `rust_serde_default_on_required_field`: required-looking contract fields that opt into `#[serde(default)]`.
- `rust_serde_flatten_catchall`: `#[serde(flatten)]` catch-all maps or loose value bags that absorb unknown fields.
- `rust_serde_unknown_fields_allowed`: strict-looking config or request structs that deserialize without `deny_unknown_fields`.
- `rust_stringly_typed_enum_boundary`: enum-like boundary fields modeled as `String` instead of a dedicated enum.

### Rust builder and state-modeling signals

- `rust_option_bag_config`: config-like structs with many `Option<_>` fields and no obvious validation path.
- `rust_builder_without_validate`: builders that expose `build()` without an obvious validation step.
- `rust_constructor_many_flags`: constructor-like APIs that use multiple boolean flags to encode behavior.
- `rust_partial_init_escape`: constructor-like functions that return or store partially initialized struct shapes.
- `rust_boolean_state_machine`: stateful structs that encode state through multiple booleans instead of a dedicated enum.

### Rust domain-modeling signals

- `rust_domain_raw_primitive`: business-facing data is stored as a raw primitive instead of a stronger domain type.
- `rust_domain_float_for_money`: floating-point storage is used for money-like values.
- `rust_domain_impossible_combination`: a boolean toggle is mixed with optional credentials, creating invalid-state combinations.
- `rust_domain_default_produces_invalid`: `Default` is derived or implemented on a type that likely cannot have a safe default state.
- `rust_debug_secret`: `Debug` is derived on a type that carries secret-like fields.
- `rust_serde_sensitive_deserialize`: `Deserialize` is derived for sensitive fields without obvious validation.
- `rust_serde_sensitive_serialize`: `Serialize` is derived for secret-like fields that may need redaction or exclusion.
- `rust_domain_optional_secret_default`: a defaultable type includes optional secret-like fields, which can hide invalid configuration.

### Rust unsafe soundness signals

- `rust_unsafe_get_unchecked`: unsafe use of `get_unchecked` without proof of bounds invariants.
- `rust_unsafe_from_raw_parts`: unsafe raw slice construction that depends on lifetime and length invariants.
- `rust_unsafe_set_len`: unsafe `Vec::set_len` use that requires initialized elements and correct capacity invariants.
- `rust_unsafe_assume_init`: unsafe `MaybeUninit::assume_init` use without proof of full initialization.
- `rust_unsafe_transmute`: unsafe `transmute` use that requires layout and validity proof.
- `rust_unsafe_raw_pointer_cast`: unsafe raw pointer cast that depends on aliasing and lifetime guarantees.
- `rust_unsafe_aliasing_assumption`: unsafe code mixes interior mutability and mutable references in ways that need careful aliasing review.

### Python-specific signals

- `blocking_sync_io_in_async`: obvious synchronous network, subprocess, sleep, or file I/O calls made from `async def` functions.
- `exception_swallowed`: broad exception handlers such as `except:` or `except Exception:` that immediately suppress the error with `pass`, `continue`, `break`, or `return`.
- `eval_exec_usage`: direct `eval()` or `exec()` usage in non-test Python code.
- `print_debugging_leftover`: `print()` calls left in non-test Python functions when they do not look like obvious `main`-entrypoint output.
- `none_comparison`: `== None` or `!= None` checks instead of `is None` or `is not None`.
- `side_effect_comprehension`: list, set, or dict comprehensions used as standalone statements where the result is discarded.
- `redundant_return_none`: explicit `return None` in simple code paths where Python would already return `None` implicitly.
- `hardcoded_path_string`: hardcoded filesystem path literals assigned inside non-test Python functions.
- `variadic_public_api`: public Python functions that expose `*args` or `**kwargs` instead of a clearer interface.
- `list_materialization_first_element`: `list(...)[0]` style access that materializes a whole list just to read the first element.
- `deque_candidate_queue`: queue-style list operations such as `pop(0)` or `insert(0, ...)` that may want `collections.deque`.
- `temporary_collection_in_loop`: loop-local list, dict, or set construction that likely adds avoidable allocation churn.
- `recursive_traversal_risk`: direct recursion in traversal-style helpers that may be safer or clearer as iterative walks for deep inputs.
- `list_membership_in_loop`: repeated membership checks against obviously list-like containers inside loops.
- `repeated_len_in_loop`: repeated `len(...)` checks inside loops when the receiver appears unchanged locally.
- `builtin_reduction_candidate`: loop shapes that look like obvious `sum`, `any`, or `all` candidates.
- `untracked_asyncio_task`: `asyncio.create_task(...)` or similar task creation whose handle is immediately discarded.
- `background_task_exception_unobserved`: background task bindings with no obvious await, callback, supervisor, or observation path.
- `async_lock_held_across_await`: async lock scopes or explicit `acquire()` / `release()` regions that continue across unrelated `await` points.
- `async_retry_sleep_without_backoff`: retry-style async loops that sleep a fixed interval with no visible backoff, jitter, or bounded retry policy.
- `god_function`: very large Python functions with high control-flow and call-surface concentration.
- `god_class`: Python classes that concentrate unusually high method count, public surface area, and mutable instance state.
- `monolithic_init_module`: `__init__.py` files that carry enough imports and behavior to look like monolithic modules.
- `monolithic_module`: non-`__init__.py` modules that are unusually large and still combine many imports with orchestration-heavy or mixed-concern behavior. The current policy now requires a 1500-line floor, a substantially larger byte threshold, broad declaration surface, and coordination-style evidence so registries, schema catalogs, and API-surface modules do not trigger on size alone.
- `too_many_instance_attributes`: classes that assign an unusually large number of instance attributes across their methods. The current policy flags 10-plus attributes conservatively and escalates at 20-plus attributes when the class still carries multiple methods.
- `eager_constructor_collaborators`: constructors that instantiate several collaborators eagerly inside `__init__`.
- `over_abstracted_wrapper`: ceremonial wrapper-style or tiny data-container classes that add little beyond storing constructor state and forwarding one small behavior.
- `mutable_default_argument`: function parameters that use mutable defaults such as `[]`, `{}`, or `set()` directly in the signature.
- `dataclass_mutable_default`: dataclass fields that use mutable defaults instead of `default_factory`.
- `dataclass_heavy_post_init`: dataclass `__post_init__` methods that perform I/O, subprocess, network, or heavyweight client setup.
- `option_bag_model`: dataclass or `TypedDict` models that accumulate many optional fields and boolean switches.
- `public_any_type_leak`: public functions or model fields that expose `Any`, `object`, or similarly wide contracts.
- `typeddict_unchecked_access`: direct indexing of optional `TypedDict` keys without an obvious guard path.
- `mixed_concerns_function`: functions that mix HTTP, persistence, and filesystem-style concerns in one body.
- `name_responsibility_mismatch`: read-style, transformation-style, or utility-style names that still perform mutation or own multiple infrastructure concerns.
- `hardcoded_business_rule`: business-facing functions that embed policy thresholds or status outcomes directly in branch logic instead of naming that policy explicitly.
- `magic_value_branching`: repeated branch-shaping numeric or string literals that likely want an explicit constant or policy name.
- `reinvented_utility`: obvious locally implemented utility helpers that overlap with already-imported standard-library style helpers.
- `network_boundary_without_timeout`: request, sync, or job-style Python functions that call HTTP boundaries with no obvious timeout or retry policy.
- `environment_boundary_without_fallback`: startup or configuration functions that read environment state with no visible fallback or validation path.
- `external_input_without_validation`: request or CLI entry points that trust external input without obvious validation or guard checks.
- `unsafe_yaml_loader`: `yaml.load(...)` or `full_load(...)` style loaders used where safe loading is more appropriate.
- `pickle_deserialization_boundary`: `pickle.load(s)` or `dill.load(s)` style deserialization in production code.
- `subprocess_shell_true`: subprocess boundaries that enable `shell=True`.
- `tar_extractall_unfiltered`: `tarfile.extractall(...)` without an obvious filter, members list, or path-validation helper.
- `tempfile_without_cleanup`: temporary files or directories created without a visible cleanup or context-manager ownership path.
- `deep_inheritance_hierarchy`: repository-local Python class chains with unusually deep inheritance depth.
- `tight_module_coupling`: modules that depend on a large number of repository-local Python modules.
- `textbook_docstring_small_helper`: tiny helpers with long, textbook-style docstrings that restate obvious behavior.
- `mixed_naming_conventions`: Python files that mix snake_case and camelCase function naming conventions.
- `unrelated_heavy_import`: heavy ecosystem imports with little local evidence of real need.
- `obvious_commentary`: comments that narrate obvious implementation steps instead of domain context.
- `enthusiastic_commentary`: unusually enthusiastic or emoji-heavy production comments.
- `commented_out_code`: comments that look like disabled code blocks instead of documentation.
- `broad_exception_handler`: broad `except Exception:` style handlers that still obscure failure shape even when they are not fully swallowed.
- `missing_context_manager`: file or resource acquisition patterns without an obvious `with` block.
- `public_api_missing_type_hints`: public Python functions that omit complete parameter or return annotations.
- `mixed_sync_async_module`: modules that expose public sync and async entry points together.
- `repeated_string_literal`: repeated long string literals in one file that likely want a shared constant.
- `duplicate_error_handler_block`: repeated exception-handling block shapes in one file.
- `duplicate_validation_pipeline`: repeated validation guard pipelines across functions in one file.
- `duplicate_test_utility_logic`: highly similar function shapes shared between test and production Python code.
- `cross_file_copy_paste_function`: highly similar non-test function bodies repeated across multiple Python files.
- `cross_file_repeated_literal`: repeated long literals across multiple Python files in the same repository slice.
- `duplicate_query_fragment`: repeated SQL-like or query-like string fragments across multiple Python files after case and whitespace normalization.
- `duplicate_transformation_pipeline`: repeated ordered parse, validate, transform, enrich, aggregate, or serialize stage sequences across functions in multiple Python files.
- `import_time_network_call`: module-scope HTTP or socket calls executed while the module is imported.
- `import_time_file_io`: module-scope file reads, writes, or directory scans that happen during import.
- `import_time_subprocess`: subprocess launches triggered from module scope during import.
- `module_singleton_client_side_effect`: eagerly constructed network, database, or cloud clients bound at module scope.
- `mutable_module_global_state`: mutable module globals updated from multiple functions.
- `import_time_config_load`: module-scope configuration or secret loading that runs during import instead of an explicit startup path.

Python also reuses shared signals when the parser evidence supports them, including `hardcoded_secret`, comment-style findings based on docstrings, `full_dataset_load`, `string_concat_in_loop`, and conservative test-quality findings.

### Consistency and tag signals

- `inconsistent_package_name`: files in the same Go directory resolve to different base package names after ignoring the `_test` suffix.
- `misgrouped_imports`: a Go import block places stdlib imports after third-party imports.
- `mixed_receiver_kinds`: methods on the same receiver type mix pointer and value receivers.
- `malformed_struct_tag`: struct field tags that do not parse as valid Go tag key/value pairs.
- `duplicate_struct_tag_key`: struct field tags that repeat the same key more than once.

### Context and blocking signals

- `missing_context`: obvious standard-library context-aware calls such as `http.Get`, `http.NewRequest`, `exec.Command`, or `net.Dial` made from functions that do not accept `context.Context`.
- `missing_context_propagation`: functions that already accept `context.Context` but still call context-free stdlib APIs such as `http.Get`, `http.NewRequest`, `exec.Command`, or `net.Dial`.
- `missing_context_propagation`: now also covers receiver-field wrappers, local wrapper chains, and `Query` versus `QueryContext`-style DB mismatches when the enclosing function already accepts `context.Context`.
- `context_background_used`: functions that already accept `context.Context` but still create `context.Background()` or `context.TODO()` locally instead of forwarding the incoming context.
- `missing_cancel_call`: derived contexts created with `context.WithCancel`, `context.WithTimeout`, or `context.WithDeadline` where deslop cannot find a local `cancel()` or `defer cancel()` call.
- `sleep_polling`: `time.Sleep` inside loops, which often indicates polling or busy-wait style code.
- `busy_waiting`: `select { default: ... }` inside loops, which often spins instead of blocking on a channel, timer, or context.

### Performance signals

- `string_concat_in_loop`: repeated string concatenation inside loops when the function is clearly building a string value incrementally.
- `likely_n_squared_string_concat`: opt-in deeper semantic signal for repeated string concatenation inside nested loops without obvious builder usage.
- `repeated_json_marshaling`: `encoding/json.Marshal` or `MarshalIndent` inside loops, which can turn iterative paths into repeated allocation and serialization hot spots.
- `allocation_churn_in_loop`: obvious `make`, `new`, or buffer-construction calls inside loops.
- `likely_n_squared_allocation`: opt-in deeper semantic signal for allocation churn that also appears inside nested loop structure.
- `fmt_hot_path`: `fmt` formatting calls such as `Sprintf` inside loops.
- `reflection_hot_path`: `reflect` package calls inside loops.
- `full_dataset_load`: calls such as `io.ReadAll`, `ioutil.ReadAll`, or `os.ReadFile` that load an entire payload into memory instead of streaming it.

### Concurrency signals

- `goroutine_without_coordination`: raw `go` statements where deslop cannot find an obvious context or WaitGroup-like coordination signal in the same function.
- `goroutine_spawn_in_loop`: raw `go` statements launched from inside loops without an obvious context or WaitGroup-like coordination signal.
- `goroutine_without_shutdown_path`: looping goroutine literals that do not show an obvious `ctx.Done()` or done-channel shutdown path.
- `goroutine_derived_context_unmanaged`: a derived context is created and then used around a likely long-lived goroutine launch before the matching cancel call is observed.
- `mutex_in_loop`: repeated `Lock` or `RLock` acquisition inside loops.
- `blocking_call_while_locked`: potentially blocking calls observed between `Lock` and `Unlock`.

### Channel and timer lifecycle signals

- `range_over_local_channel_without_close`: a function ranges over a locally owned channel without an observed `close(ch)` path in the same function.
- `double_close_local_channel`: the same locally created channel appears to be closed more than once in one function body.
- `send_after_local_close_risk`: a locally owned channel is closed and later used in a send expression.
- `time_after_in_loop`: `time.After(...)` is allocated inside a loop instead of reusing a timer or deadline mechanism.
- `ticker_without_stop`: `time.NewTicker(...)` is created without an observed `Stop()` call in the owning function.

### HTTP boundary signals

- `http_response_body_not_closed`: an HTTP response is acquired locally without an observed `resp.Body.Close()` call.
- `http_client_without_timeout`: a local `http.Client{}` literal is constructed without an explicit timeout.
- `http_server_without_timeouts`: an explicit `http.Server{}` literal omits common timeout fields such as `ReadTimeout`, `WriteTimeout`, or `IdleTimeout`.
- `http_status_ignored_before_decode`: response decoding or body consumption happens with no visible `StatusCode` check.
- `http_writeheader_after_write`: a handler writes the response body before calling `WriteHeader(...)`, making the later status-setting call misleading.

### Resource cleanup signals

- `file_handle_without_close`: a file handle opened via `os.Open`, `os.Create`, or `os.OpenFile` lacks an observed `Close()` path in the owning function.
- `rows_without_close`: a query result handle looks locally owned but no `rows.Close()` call is observed.
- `stmt_without_close`: a prepared statement or similar closable DB handle lacks an observed `Close()` call.
- `tx_without_rollback_guard`: a transaction is begun and later committed with no observed rollback guard.
- `defer_in_loop_resource_growth`: a `defer` statement appears inside a loop, which can accumulate resources until function exit.

### Package state and abstraction signals

- `mutable_package_global`: a package-level variable is mutated from function bodies rather than being kept immutable or wrapped behind ownership boundaries.
- `init_side_effect`: an `init()` function performs network, file-system, or subprocess side effects.
- `single_impl_interface`: a repository-local interface currently has one obvious implementation and a very small consumer surface, suggesting ceremonial abstraction.
- `passthrough_wrapper_interface`: a wrapper struct mostly forwards one-to-one through an interface field with little added policy.
- `public_bool_parameter_api`: an exported function or method exposes raw boolean mode switches in its signature.

### Data-access signals

- `n_plus_one_query`: database-style query calls issued inside loops. When `go_semantic_experimental = true` or `--enable-semantic` is enabled, nested-loop correlation can raise severity and add stronger evidence.
- `wide_select_query`: literal `SELECT *` query shapes.
- `likely_unindexed_query`: query shapes such as leading-wildcard `LIKE` or `ORDER BY` without `LIMIT` that often scale poorly.

### Local hallucination signals

- `hallucinated_import_call`: package-qualified calls that do not match locally indexed symbols for the imported package.
- `hallucinated_local_call`: same-package calls to symbols that are not present in the scanned local package context.

For Rust, `hallucinated_import_call` currently covers conservative local-module imports built from `crate::`, `self::`, and `super::` paths when deslop can map them back to locally indexed Rust modules, plus direct calls through locally imported Rust function aliases.

For Rust, `hallucinated_local_call` now also covers direct same-module calls when the callee name is not locally bound and does not exist in the indexed Rust module.

### Rust-specific comment-leftover signals

- `todo_doc_comment_leftover`: Rust doc comments that still contain a `TODO` marker in non-test code.
- `fixme_doc_comment_leftover`: Rust doc comments that still contain a `FIXME` marker in non-test code.

### Test-quality signals

- `test_without_assertion_signal`: tests that call production code without any obvious assertion or failure signal.
- `happy_path_only_test`: tests that assert success expectations without any obvious negative-path signal.
- `placeholder_test_body`: tests that look skipped, TODO-shaped, or otherwise placeholder-like rather than validating behavior.

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