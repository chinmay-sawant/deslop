# Python Project-Agnostic Performance And Structure 200-Rule Checklist For v0.3.0

Date: 2026-04-10

## Objective

- Add 200 additional Python rule candidates aligned to the existing category vocabulary, with emphasis on performance, hot paths, maintainability, and structural clarity.
- Keep the checklist repo agnostic so it applies cleanly to libraries, CLIs, services, workers, and batch jobs.
- Prefer candidates that are detectable from AST evidence, import patterns, call sites, data flow hints, and limited cross-file context.
- Bias toward rules that prevent wasted work, reduce hidden coupling, and make long-term evolution easier.
- Verification status: cross-checked against `src/heuristics/python/specs/catalog.rs` on 2026-04-10, with all 200 checklist rule IDs present in the Python rule catalog.

## Hard Exclusions

- Avoid framework-specific assumptions about Django, FastAPI, Flask, SQLAlchemy, Celery, Pandas, NumPy, or any other single ecosystem.
- Avoid company-specific folder naming, package layout rules, or deployment conventions.
- Avoid style-only guidance where the structural or runtime impact is too subjective to promote as a rule.
- Avoid rules that only make sense for generated code, migrations, notebooks, or one-off scripts unless the evidence is unusually strong.

## Section Summary

| Section | Category | Rules |
| --- | --- | ---: |
| 1 | Architecture | 20 |
| 2 | Boundaries | 20 |
| 3 | Discipline | 18 |
| 4 | Hot Path | 20 |
| 5 | Hot Path Extended | 20 |
| 6 | Maintainability | 18 |
| 7 | Observability | 18 |
| 8 | Packaging | 12 |
| 9 | Performance | 20 |
| 10 | Quality | 18 |
| 11 | Structure | 16 |
| **Total** | | **200** |

## 1. Architecture (20 rules)

- [x] `constructor_reads_global_config_inline`: flag classes whose constructors pull environment variables or global config directly instead of receiving normalized dependencies.
- [x] `entrypoint_builds_dependency_graph_inside_hot_function`: flag request, batch, or loop entrypoints that reconstruct clients, caches, or providers on each call instead of wiring them once.
- [x] `domain_object_performs_external_io`: flag domain or value-like objects that open files, spawn subprocesses, or call network clients during core operations.
- [x] `business_rule_mixed_with_serialization_mapping`: flag functions that both enforce domain rules and convert to wire or storage payloads in the same block.
- [x] `function_returns_domain_value_and_transport_metadata`: flag APIs that mix domain results with transport metadata such as status-like fields or side-channel details in one return shape.
- [x] `storage_write_returns_driver_specific_object`: flag write helpers that return raw driver, session, or cursor objects instead of stable application data.
- [x] `module_exposes_mutable_singleton_client`: flag modules that export long-lived mutable client instances for direct cross-module mutation.
- [x] `feature_logic_embedded_in_process_entrypoint`: flag CLI, worker, or service entrypoints that own business branching instead of delegating to focused application services.
- [x] `transaction_scope_split_across_unrelated_helpers`: flag flows where begin, commit, and rollback responsibilities are scattered across helpers with no single owner.
- [x] `initializer_requires_half_built_instance_state`: flag methods that depend on object fields being patched in after construction before the object becomes usable.
- [x] `object_construction_triggers_network_or_disk_side_effect`: flag constructors that perform expensive I/O instead of separating configuration from execution.
- [x] `module_import_starts_runtime_bootstrap`: flag imports that eagerly start threads, background loops, watchers, or connection attempts at import time.
- [x] `command_or_task_mutates_shared_process_state_directly`: flag command handlers or task functions that directly rewrite module-level caches or registries used elsewhere.
- [x] `function_accepts_too_many_cross_cutting_dependencies`: flag functions or constructors that accept a broad mix of logging, config, cache, clock, auth, and persistence dependencies with no cohesive abstraction.
- [x] `data_mapper_contains_business_decision_tree`: flag mappers or adapters that choose policy outcomes while converting data.
- [x] `orchestrator_performs_low_level_tokenization_or_parsing`: flag high-level workflow functions that also own low-level parsing logic better isolated in helpers.
- [x] `core_model_reads_process_environment`: flag core classes or pure helper layers that directly access `os.environ`, `os.getenv`, or global settings.
- [x] `third_party_exception_type_leaks_across_architecture_boundary`: flag public or upper-layer APIs that expose low-level library exception types instead of stable local contracts.
- [x] `retry_policy_scattered_across_multiple_callers`: flag repeated retry or backoff loops around the same dependency instead of one owner boundary.
- [x] `adapter_boundary_missing_for_external_payload_shape`: flag code that lets raw third-party dicts, tuples, or response objects spread through unrelated modules without normalization.

## 2. Boundaries (20 rules)

- [x] `mutable_default_argument_leaks_state_across_calls`: flag function signatures that use mutable defaults such as `[]`, `{}`, or `set()` and allow state sharing between calls.
- [x] `helper_returns_live_internal_collection_reference`: flag helpers that hand out direct references to mutable internal lists, dicts, or sets instead of copies or read-only views.
- [x] `public_api_mutates_argument_in_place_without_signal`: flag public APIs that rewrite caller-owned containers or objects without naming or documentation that makes the mutation obvious.
- [x] `dataclass_mutable_default_without_default_factory`: flag dataclass fields that use mutable defaults without `default_factory`.
- [x] `context_manager_yields_global_mutable_resource`: flag context managers that expose a shared global mutable resource while implying call-local ownership.
- [x] `module_cache_exposed_without_invalidation_boundary`: flag modules that expose cache internals directly with no clear invalidation or ownership boundary.
- [x] `closure_captures_loop_variable_without_binding`: flag closures created in loops that capture the loop variable without binding the current value.
- [x] `function_accepts_mapping_protocol_but_mutates_input`: flag functions that advertise broad mapping or sequence inputs and then mutate the received object in place.
- [x] `iterator_argument_consumed_then_reused_later`: flag functions that exhaust an iterator and later treat it as if it were reusable data.
- [x] `public_api_forwards_library_specific_exception_shape`: flag public interfaces that require callers to understand raw exceptions from underlying libraries.
- [x] `datetime_boundary_mixes_naive_and_aware_values`: flag APIs that accept or combine timezone-aware and timezone-naive datetimes without normalization.
- [x] `text_bytes_boundary_relies_on_implicit_default_encoding`: flag code that crosses text and bytes boundaries without an explicit encoding contract.
- [x] `path_boundary_accepts_unexpanded_or_relative_input_without_normalization`: flag filesystem-facing APIs that accept relative paths or `~` forms without normalizing them before downstream use.
- [x] `sentinel_default_value_overlaps_valid_business_value`: flag APIs whose sentinel or fallback value can also be a valid domain value.
- [x] `sync_api_accepts_coroutine_object_as_regular_value`: flag synchronous interfaces that can accidentally receive coroutine objects and treat them as plain values.
- [x] `async_api_returns_plain_iterator_with_blocking_iteration`: flag async-facing APIs that return plain iterators whose consumption can still block.
- [x] `property_returns_live_internal_cache_object`: flag properties that expose live mutable cache objects directly to callers.
- [x] `lock_acquire_and_release_owned_by_different_callers`: flag APIs that require one caller to acquire a lock and another caller to release it.
- [x] `module_constant_rebound_after_public_import`: flag modules that expose apparent constants and later rebind them during normal runtime flow.
- [x] `helper_requires_caller_to_know_hidden_ordering_constraints`: flag helpers whose correctness depends on the caller invoking them in a specific undocumented order.

## 3. Discipline (18 rules)

- [x] `boolean_flag_parameter_controls_unrelated_behaviors`: flag parameters whose boolean value switches between materially different behaviors instead of selecting clear separate entrypoints.
- [x] `function_body_contains_setup_validation_execution_and_formatting_all_at_once`: flag functions that pack too many lifecycle phases into one body.
- [x] `condition_tree_nests_past_two_business_decision_levels`: flag functions with deeply nested business condition trees that would be clearer as smaller helpers or dispatch tables.
- [x] `expensive_work_starts_before_input_validation`: flag functions that begin heavy computation or I/O before validating cheap preconditions.
- [x] `duplicated_cleanup_paths_instead_of_context_manager`: flag repeated cleanup logic that should be centralized with a context manager or helper.
- [x] `helper_name_hides_mutation_or_io_side_effect`: flag helpers whose names sound pure but actually mutate state or perform I/O.
- [x] `method_mutates_state_and_emits_user_facing_representation`: flag methods that both change internal state and format user-facing output.
- [x] `loop_interleaves_core_work_logging_and_recovery_logic`: flag loops that mix primary business logic, logging, and error recovery in one dense block.
- [x] `type_branch_and_mode_branch_compounded_in_same_function`: flag functions that branch on both runtime type and mode string in the same decision tree.
- [x] `repeated_try_finally_release_pattern_not_extracted`: flag repeated manual release patterns that should live behind one helper or context manager.
- [x] `long_parameter_list_of_primitives_without_options_object`: flag functions whose many primitive parameters obscure meaning and encourage call-site mistakes.
- [x] `negated_boolean_reassigned_and_inverted_again`: flag control flow that repeatedly flips boolean state instead of naming the intended condition directly.
- [x] `method_mutates_self_and_peer_object_in_same_block`: flag methods that change both local state and a collaborator's state in the same unit of work.
- [x] `batch_api_silently_falls_back_to_single_item_semantics`: flag APIs that claim batch behavior but quietly process items one by one with different semantics.
- [x] `same_precondition_checked_in_multiple_sibling_branches`: flag functions that repeat the same guard in several branches instead of normalizing once.
- [x] `function_returns_multiple_unlabeled_shape_variants`: flag functions that return unrelated tuple or dict shapes depending on the path taken.
- [x] `module_mixes_constants_types_helpers_and_execution_flow`: flag modules that combine unrelated responsibilities and become hard to scan.
- [x] `correctness_depends_on_specific_call_order_not_encoded_in_api`: flag APIs whose safe use depends on an undocumented sequence of method calls.

## 4. Hot Path (20 rules)

- [x] `regex_compiled_on_each_hot_call`: flag hot functions that compile the same regular expression repeatedly instead of reusing a compiled pattern.
- [x] `json_roundtrip_used_for_object_copy`: flag code that serializes and deserializes objects only to make a copy.
- [x] `repeated_datetime_parse_inside_loop`: flag loops that parse timestamps repeatedly when values could be normalized once.
- [x] `repeated_split_or_join_on_invariant_separator_inside_loop`: flag loops that repeatedly rebuild the same separator-driven string operations.
- [x] `repeated_attribute_chain_lookup_inside_tight_loop`: flag tight loops that repeatedly traverse the same attribute chain instead of binding a local reference.
- [x] `exception_used_for_expected_lookup_miss_in_loop`: flag loops that rely on exceptions for common cache or dict misses.
- [x] `membership_test_against_list_or_tuple_literal_inside_loop`: flag repeated membership tests against small linear containers in a hot path.
- [x] `incremental_list_or_tuple_concatenation_in_accumulation_loop`: flag accumulation loops that repeatedly concatenate instead of appending and joining once.
- [x] `constant_frozenset_or_dict_rebuilt_on_each_call`: flag frequent call paths that reconstruct constant lookup tables each invocation.
- [x] `function_local_import_executed_in_frequent_path`: flag hot call paths that perform imports inside the function body.
- [x] `pathlib_path_reconstructed_from_same_base_in_loop`: flag loops that rebuild the same base `Path` object every iteration.
- [x] `environment_lookup_repeated_in_hot_path`: flag hot functions that repeatedly read environment variables or global config values that do not change.
- [x] `repeated_normalization_of_same_string_in_loop`: flag repeated `.strip()`, `.lower()`, `.casefold()`, or similar normalization on the same value inside a loop.
- [x] `full_sort_performed_inside_outer_iteration`: flag outer loops that sort a full collection during each iteration.
- [x] `list_of_keys_materialized_for_membership_check`: flag code that builds `list(mapping.keys())` before checking membership.
- [x] `lambda_or_closure_allocated_per_item_when_static_helper_suffices`: flag per-item closure allocation in hot loops when a stable helper would work.
- [x] `iterator_materialized_to_list_before_single_pass_loop`: flag iterators converted to lists in hot paths when only one pass is needed.
- [x] `subprocess_or_shell_call_inside_record_processing_loop`: flag per-record subprocess work that should be batched or hoisted.
- [x] `repeated_pure_helper_call_on_same_input_without_local_cache`: flag hot loops that recompute the same pure helper result for identical inputs in one scope.
- [x] `same_buffer_or_prefix_reencoded_each_iteration`: flag loops that re-encode identical prefixes or headers on each iteration.

## 5. Hot Path Extended (20 rules)

- [x] `blocking_io_call_executed_per_item_without_batching`: flag per-item blocking I/O where batching or grouping is possible.
- [x] `repeated_directory_scan_inside_nested_loop`: flag nested loops that rescan the same directory or file listing.
- [x] `invariant_computation_not_hoisted_out_of_nested_loop`: flag nested loops that recompute invariant values on every inner iteration.
- [x] `any_or_all_wraps_list_comprehension_instead_of_generator`: flag `any([...])` and `all([...])` forms that force unnecessary list materialization.
- [x] `sum_max_min_wrap_list_comprehension_instead_of_generator`: flag reductions that build transient lists instead of using generator expressions.
- [x] `per_item_copy_of_large_config_or_context_object`: flag loops that copy large config or context structures for each item despite read-mostly behavior.
- [x] `same_sequence_scanned_multiple_times_for_related_aggregates`: flag code that makes several full passes over the same data for related summaries.
- [x] `generator_pipeline_materialized_between_each_transformation_stage`: flag pipelines that repeatedly materialize intermediate lists between stages.
- [x] `linear_search_helper_called_from_nested_loops`: flag nested loops that call helpers performing linear scans on each iteration.
- [x] `repeated_path_exists_check_before_open_or_replace_in_loop`: flag loops that perform a separate existence check before every file operation.
- [x] `serialization_then_deserialization_between_adjacent_helpers`: flag neighboring helpers that bounce data through serialized text or bytes without need.
- [x] `large_slice_copy_created_each_iteration_for_sliding_window`: flag windowed algorithms that copy large slices every step instead of tracking indexes.
- [x] `per_item_deduplication_uses_list_instead_of_hash_index`: flag high-cardinality deduplication done with linear container membership checks.
- [x] `expensive_sort_key_recomputed_without_preprojection`: flag repeated expensive key computation during sorting when values could be precomputed once.
- [x] `repeated_casefold_or_lower_calls_before_multiple_comparisons`: flag code that normalizes the same candidate repeatedly before several comparisons.
- [x] `formatted_log_or_debug_payload_built_for_each_item_without_guard`: flag per-item log payload construction without checking whether the level is enabled.
- [x] `repeated_open_read_close_of_same_small_file_in_single_workflow`: flag workflows that reopen the same file many times instead of caching its contents or handle.
- [x] `polling_loop_uses_tiny_sleep_instead_of_blocking_primitive`: flag loops that wake repeatedly on a tiny sleep interval instead of a blocking queue, event, or condition.
- [x] `invariant_template_or_prefix_string_reformatted_inside_loop`: flag loops that reformat constant template fragments on each iteration.
- [x] `lookup_table_derived_from_constants_rebuilt_per_invocation`: flag functions that reconstruct lookup tables from static constants every call.

## 6. Maintainability (18 rules)

- [x] `magic_thresholds_duplicated_across_modules`: flag repeated numeric thresholds with shared meaning that are copied across modules.
- [x] `tuple_return_with_three_or_more_positional_fields_in_public_api`: flag public APIs that return positional tuples too wide to be self-documenting.
- [x] `parallel_lists_used_instead_of_record_object`: flag code that keeps related values synchronized across multiple lists instead of using one record type.
- [x] `normalization_logic_duplicated_across_call_sites`: flag the same value-cleaning or canonicalization logic repeated in several places.
- [x] `mutable_class_attribute_used_as_instance_storage`: flag class attributes that accidentally store per-instance mutable state.
- [x] `helper_module_accumulates_unrelated_cross_domain_utilities`: flag helper modules that mix string, filesystem, caching, parsing, and process logic in one place.
- [x] `string_sentinel_values_duplicated_instead_of_constant_or_enum`: flag repeated mode or status strings that should be centralized.
- [x] `same_contextmanager_pattern_copied_across_modules`: flag duplicate context manager behavior implemented in several modules.
- [x] `wrapper_function_only_renames_arguments_and_passthroughs_behavior`: flag wrappers that add no policy, safety, or abstraction value.
- [x] `anonymous_dict_shape_repeated_without_shared_type_or_builder`: flag repeated ad hoc dict payload shapes with no shared type or builder.
- [x] `branching_on_file_suffix_or_mode_string_scattered_across_codebase`: flag suffix or mode dispatch logic duplicated across distant modules.
- [x] `hidden_dependency_arrives_via_import_time_side_effect`: flag code that becomes initialized only because another import happened first.
- [x] `cache_object_exists_without_size_or_eviction_policy_documentation`: flag caches that exist as long-lived process state with no documented retention policy.
- [x] `comment_required_to_explain_opaque_branching_that_code_could_express`: flag code whose control flow stays unclear even with comments because the structure itself is weak.
- [x] `helper_returns_index_based_tuple_instead_of_named_structure`: flag helper returns that require callers to remember tuple positions by convention.
- [x] `mixed_mutation_and_query_methods_share_same_manager_class`: flag manager-style classes that both mutate and answer read-model queries with no boundary.
- [x] `monolithic_utils_module_becomes_default_dependency_sink`: flag utility modules that attract unrelated dependencies and spread coupling.
- [x] `single_feature_requires_edits_in_many_unrelated_modules_due_to_scattered_policy`: flag feature flows whose policy is fragmented across many unrelated modules.

## 7. Observability (18 rules)

- [x] `logger_instance_created_inside_function_body`: flag functions that create loggers repeatedly instead of reusing module-level logger references.
- [x] `expensive_log_argument_built_without_is_enabled_guard`: flag log calls that eagerly build costly payloads without first checking the log level.
- [x] `metric_name_contains_dynamic_user_or_data_values`: flag dynamic metric names that explode the series count.
- [x] `metric_or_span_labels_use_high_cardinality_raw_inputs`: flag labels or attributes built from raw IDs, paths, or user-provided values.
- [x] `metric_emission_occurs_per_item_inside_inner_loop`: flag inner loops that emit one metric per item instead of aggregating.
- [x] `health_probe_executes_full_dependency_workflow`: flag health checks that run the full production path instead of a cheap signal.
- [x] `operation_lacks_single_stable_name_across_logs_metrics_and_traces`: flag operations that are named inconsistently across observability surfaces.
- [x] `retry_loop_logs_without_attempt_count_or_backoff_context`: flag retry logging that omits attempt number, delay, or terminal outcome.
- [x] `exception_log_omits_operation_identifier_or_input_summary`: flag exception logs that lose the operation context needed to diagnose failures.
- [x] `correlation_id_recomputed_multiple_times_in_same_workflow`: flag code that regenerates correlation IDs instead of propagating one value.
- [x] `debug_log_serializes_full_large_object_graph`: flag debug logging that walks and serializes large object graphs.
- [x] `success_and_failure_paths_use_inconsistent_structured_log_keys`: flag logging paths that use different keys for the same concept.
- [x] `timing_metric_wraps_setup_and_teardown_noise_instead_of_core_operation`: flag timers that measure unrelated setup work and make latency metrics noisy.
- [x] `instrumentation_helper_mutates_business_return_shape`: flag instrumentation wrappers that alter the shape of business return values.
- [x] `observability_context_extracted_manually_at_many_call_sites`: flag repeated manual extraction of the same tracing or logging context.
- [x] `warning_or_error_logs_emit_unbounded_payload_text`: flag logs that dump unbounded input or result payloads.
- [x] `synchronous_log_handler_or_flush_called_on_fast_path`: flag code that forces synchronous log flushing in latency-sensitive paths.
- [x] `instrumentation_import_or_setup_occurs_on_first_live_request`: flag observability setup that waits until the first request or task instead of initializing predictably.

## 8. Packaging (12 rules)

- [x] `heavy_optional_dependency_imported_by_package_root`: flag package roots that import heavy optional dependencies by default.
- [x] `cli_only_dependency_imported_by_library_entry_module`: flag library entry modules that pull in CLI-only dependencies on import.
- [x] `package_init_performs_metadata_version_lookup_on_import`: flag package `__init__` files that do runtime metadata lookups just to expose a version.
- [x] `environment_or_config_read_during_package_import`: flag package import paths that read environment or config eagerly.
- [x] `circular_import_hidden_by_function_local_import_on_hot_path`: flag circular import workarounds that push imports into frequent code paths.
- [x] `plugin_discovery_scans_filesystem_each_invocation`: flag plugin or extension discovery that rescans disk on every invocation.
- [x] `package_exports_same_symbol_name_from_multiple_submodules_with_different_meanings`: flag packages that reuse one exported name for unrelated implementations.
- [x] `runtime_data_file_assumption_in_implicit_namespace_package`: flag implicit namespace packages that assume local data-file discovery at runtime.
- [x] `test_helpers_shipped_inside_production_package_path`: flag test-only helpers living under the production import path.
- [x] `public_api_surface_defined_only_by_import_side_effects`: flag packages whose public surface is created indirectly by import order.
- [x] `package_root_reexports_large_dependency_tree_by_default`: flag root packages that re-export many heavy submodules automatically.
- [x] `monolithic_common_package_becomes_transitive_dependency_for_most_modules`: flag package designs where one vague common package becomes a hidden dependency hub.

## 9. Performance (20 rules)

- [x] `repeated_file_open_for_same_resource_within_single_operation`: flag workflows that reopen the same file repeatedly during one logical operation.
- [x] `eager_full_file_or_stream_read_when_incremental_iteration_suffices`: flag code that reads whole files or streams into memory before simple sequential processing.
- [x] `bytes_text_bytes_roundtrip_without_transformation`: flag code that decodes and re-encodes data without changing it.
- [x] `quadratic_string_building_via_plus_equals`: flag loops that grow large strings with repeated `+=`.
- [x] `multiple_regex_passes_over_same_text_without_precompiled_plan`: flag code that re-runs several overlapping regex passes on the same text.
- [x] `full_response_or_export_buffered_before_incremental_consumer_use`: flag producers that fully buffer large outputs before handing them to a consumer.
- [x] `temporary_file_used_for_pure_in_memory_transformation`: flag workflows that spill to disk despite an in-memory transform being sufficient.
- [x] `thread_pool_or_process_pool_created_and_destroyed_per_call`: flag per-call executor allocation for repeatable work.
- [x] `large_object_cloned_before_read_only_operation`: flag code that deep-copies large structures even when the next steps are read-only.
- [x] `repeated_stat_or_exists_calls_before_single_followup_operation`: flag paths that perform duplicate filesystem checks before one operation.
- [x] `batchable_writes_executed_one_at_a_time`: flag repeated write operations that could be grouped or buffered.
- [x] `same_dataset_normalized_in_multiple_full_passes`: flag code that walks the same dataset several times for normalization steps that can be fused.
- [x] `generator_materialized_to_tuple_or_list_only_for_len_or_truthiness`: flag iterator pipelines that materialize only to test truthiness or size.
- [x] `full_collection_sorted_when_partial_order_or_selection_suffices`: flag full sorts used where top-k or one-pass selection would work.
- [x] `compression_hashing_or_encoding_performed_before_cheap_reject_checks`: flag expensive transforms performed before simple guard checks that could skip the work.
- [x] `event_loop_path_executes_cpu_bound_transformation_synchronously`: flag async paths that perform large CPU-bound transforms inline.
- [x] `repeated_small_writes_without_buffering_or_join`: flag code that emits many tiny writes instead of buffering.
- [x] `copy_of_mapping_created_only_to_read_values`: flag mappings copied defensively even though the next code only reads.
- [x] `serialization_cost_paid_only_to_compare_or_hash_intermediate_state`: flag serialization used only for equality, cache key, or hashing comparisons.
- [x] `large_in_memory_intermediate_created_where_streaming_pipeline_would_do`: flag workflows that build large temporary structures where streaming would suffice.

## 10. Quality (18 rules)

- [x] `public_api_returns_none_or_value_without_explicit_optional_contract`: flag public APIs that sometimes return `None` and sometimes a value without an explicit contract.
- [x] `fallback_branch_swallows_invariant_violation_and_returns_plausible_default`: flag fallback logic that hides broken invariants by returning believable defaults.
- [x] `broad_except_used_to_mask_type_or_shape_bug`: flag broad exception handling that conceals structural bugs in the input or code.
- [x] `order_dependent_set_to_list_conversion_exposed_in_public_result`: flag public outputs that rely on unstable set iteration order.
- [x] `default_timeout_missing_on_external_boundary_wrapper`: flag wrappers around external boundaries that omit a timeout policy.
- [x] `float_equality_controls_branching_on_computed_values`: flag control flow that depends on exact float equality after computation.
- [x] `recursive_walk_over_untrusted_input_lacks_depth_limit`: flag recursion over caller-provided structures with no depth guard.
- [x] `public_iterator_yields_heterogeneous_item_shapes`: flag iterators that yield different shapes or types without an explicit sum-type contract.
- [x] `partial_update_mutates_input_before_validation_succeeds`: flag update helpers that mutate caller data before all validation passes.
- [x] `cache_key_derived_from_stringified_mutable_object`: flag cache keys built from unstable string forms of mutable objects.
- [x] `sort_order_depends_on_non_explicit_mapping_iteration_semantics`: flag ordering logic that depends on implicit mapping iteration instead of explicit keys.
- [x] `duplicate_items_silently_dropped_without_contract_signal`: flag code that deduplicates caller data without making that behavior explicit.
- [x] `timezone_naive_datetime_accepted_in_public_contract`: flag public contracts that accept datetimes with ambiguous timezone semantics.
- [x] `atomic_replace_semantics_implemented_with_non_atomic_file_write`: flag code that intends atomic replacement but uses non-atomic file writes.
- [x] `string_mode_parameter_replaces_enum_or_literal_contract`: flag string mode parameters that should be constrained by an enum or literal contract.
- [x] `helper_returns_success_shape_even_when_substeps_partially_fail`: flag helpers that claim success while hiding partial failures.
- [x] `comparison_or_merge_logic_assumes_unique_keys_without_assertion`: flag merge logic that silently assumes uniqueness of keys or identifiers.
- [x] `validation_only_happens_after_expensive_side_effect_has_started`: flag flows where validation happens only after I/O or irreversible work has begun.

## 11. Structure (16 rules)

- [x] `monolithic_module_owns_parsing_validation_execution_and_rendering`: flag modules that contain the whole pipeline and become hard to reason about.
- [x] `class_mixes_factory_parsing_persistence_and_presentation_roles`: flag classes that combine unrelated lifecycle roles.
- [x] `abstract_contracts_and_heavy_concrete_implementations_live_in_same_file`: flag files that mix contracts with large concrete implementations and grow unstable dependencies.
- [x] `bidirectional_import_between_feature_modules`: flag feature modules that depend on each other directly.
- [x] `generic_manager_or_processor_class_controls_many_unrelated_modes`: flag vague manager-style classes that centralize unrelated behaviors behind mode flags.
- [x] `composition_candidate_for_optional_behavior_implemented_as_inheritance`: flag optional behavior modeled through inheritance when composition would isolate concerns better.
- [x] `base_class_exists_only_to_share_data_fields_not_behavior`: flag base classes that provide only data containers with no meaningful shared behavior.
- [x] `constructor_performs_real_work_beyond_state_initialization`: flag constructors that do heavy work instead of leaving execution to explicit methods.
- [x] `module_global_registry_mutated_from_import_time_registration`: flag global registries that are populated by import side effects.
- [x] `same_feature_path_crosses_many_layers_for_simple_data_transform`: flag simple transformations that bounce through too many layers or wrappers.
- [x] `read_and_write_paths_share_mutable_internal_cache_without_boundary`: flag read and write paths that couple through one mutable cache object.
- [x] `sync_and_async_contracts_mixed_on_same_interface_family`: flag interface families that mix sync and async methods without a clear separation.
- [x] `helper_collection_object_also_owns_process_lifecycle`: flag collection-like helper objects that also start, stop, or supervise process lifecycle work.
- [x] `sibling_modules_depend_on_private_helpers_from_each_other`: flag sibling modules that reach into each other's private helpers instead of using a shared boundary.
- [x] `cross_cutting_policies_embedded_in_leaf_modules_instead_of_shared_boundary`: flag retries, normalization, or policy checks embedded in leaf modules instead of one shared boundary.
- [x] `abstractions_named_utils_helpers_common_or_manager_hide_true_ownership`: flag vague abstractions whose names obscure actual ownership and responsibility.
