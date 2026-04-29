use deslop::ScanReport;

use super::super::support::{
    FixtureWorkspace, assert_rules_absent, assert_rules_present, scan_files,
};

fn scan(files: &[(&str, &str)]) -> ScanReport {
    scan_files(files)
}

#[test]
fn test_project_agnostic_architecture_rules_positive() {
    let report = scan(&[
        (
            "app/service_layer.py",
            python_fixture!("project_agnostic/architecture_service_layer_positive.txt"),
        ),
        (
            "app/main.py",
            python_fixture!("project_agnostic/architecture_main_positive.txt"),
        ),
    ]);

    assert_rules_present(
        &report,
        &[
            "constructor_reads_global_config_inline",
            "entrypoint_builds_dependency_graph_inside_hot_function",
            "function_returns_domain_value_and_transport_metadata",
            "module_exposes_mutable_singleton_client",
            "module_import_starts_runtime_bootstrap",
        ],
    );
}

#[test]
fn test_project_agnostic_architecture_rules_negative() {
    let report = scan(&[(
        "app/main.py",
        python_fixture!("project_agnostic/architecture_main_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "constructor_reads_global_config_inline",
            "entrypoint_builds_dependency_graph_inside_hot_function",
            "function_returns_domain_value_and_transport_metadata",
        ],
    );
}

#[test]
fn test_project_agnostic_architecture_rule_skips_http_exception_status_code() {
    let report = scan(&[(
        "app/api.py",
        python_fixture!("project_agnostic/architecture_http_exception_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &["function_returns_domain_value_and_transport_metadata"],
    );
}

#[test]
fn test_project_agnostic_boundaries_rule_skips_path_and_fstring_constants() {
    let report = scan(&[(
        "app/config.py",
        python_fixture!("project_agnostic/boundaries_path_constants_negative.txt"),
    )]);

    assert_rules_absent(&report, &["module_constant_rebound_after_public_import"]);
}

#[test]
fn test_project_agnostic_boundaries_rule_still_flags_mutable_constants() {
    let report = scan(&[(
        "app/detector.py",
        python_fixture!("project_agnostic/boundaries_mutable_constants_positive.txt"),
    )]);

    assert_rules_present(&report, &["module_constant_rebound_after_public_import"]);
}

#[test]
fn test_project_agnostic_quality_rule_skips_passthrough_any_wrapper() {
    let report = scan(&[(
        "app/contracts.py",
        python_fixture!("project_agnostic/quality_passthrough_any_negative.txt"),
    )]);

    assert_rules_absent(&report, &["public_any_type_leak"]);
}

#[test]
fn test_project_agnostic_boundaries_rule_requires_mutation_evidence() {
    let report = scan(&[(
        "app/cache.py",
        python_fixture!("project_agnostic/boundaries_live_collection_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &["helper_returns_live_internal_collection_reference"],
    );
}

#[test]
fn test_project_agnostic_boundaries_rule_flags_mutated_live_collection() {
    let report = scan(&[(
        "app/cache.py",
        python_fixture!("project_agnostic/boundaries_live_collection_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &["helper_returns_live_internal_collection_reference"],
    );
}

#[test]
fn test_project_agnostic_performance_rule_requires_same_dataset() {
    let report = scan(&[(
        "app/perf.py",
        python_fixture!("project_agnostic/performance_same_dataset_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &["same_dataset_normalized_in_multiple_full_passes"],
    );
}

#[test]
fn test_project_agnostic_discipline_rule_requires_loop_local_recovery_logging() {
    let report = scan(&[(
        "app/worker.py",
        python_fixture!("project_agnostic/discipline_loop_recovery_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &["loop_interleaves_core_work_logging_and_recovery_logic"],
    );
}

#[test]
fn test_project_agnostic_discipline_rule_flags_loop_local_recovery_logging() {
    let report = scan(&[(
        "app/worker.py",
        python_fixture!("project_agnostic/discipline_loop_recovery_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &["loop_interleaves_core_work_logging_and_recovery_logic"],
    );
}

#[test]
fn test_project_agnostic_discipline_rules_positive_batch_one() {
    let report = scan(&[(
        "app/discipline_batch_one.py",
        python_fixture!("project_agnostic/discipline_batch_one_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "boolean_flag_parameter_controls_unrelated_behaviors",
            "condition_tree_nests_past_two_business_decision_levels",
            "function_body_contains_setup_validation_execution_and_formatting_all_at_once",
            "expensive_work_starts_before_input_validation",
            "duplicated_cleanup_paths_instead_of_context_manager",
        ],
    );
}

#[test]
fn test_project_agnostic_discipline_rules_negative_batch_one() {
    let report = scan(&[(
        "app/discipline_batch_one.py",
        python_fixture!("project_agnostic/discipline_batch_one_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "boolean_flag_parameter_controls_unrelated_behaviors",
            "condition_tree_nests_past_two_business_decision_levels",
            "function_body_contains_setup_validation_execution_and_formatting_all_at_once",
            "expensive_work_starts_before_input_validation",
            "duplicated_cleanup_paths_instead_of_context_manager",
        ],
    );
}

#[test]
fn test_project_agnostic_hotpath_rule_skips_tuple_iteration_false_match() {
    let report = scan(&[(
        "app/io.py",
        python_fixture!("project_agnostic/hotpath_tuple_membership_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &["membership_test_against_list_or_tuple_literal_inside_loop"],
    );
}

#[test]
fn test_project_agnostic_hotpath_ext_rule_skips_data_dependent_fstrings() {
    let report = scan(&[(
        "app/export.py",
        python_fixture!("project_agnostic/hotpath_ext_data_dependent_fstrings_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &["invariant_template_or_prefix_string_reformatted_inside_loop"],
    );
}

#[test]
fn test_project_agnostic_boundary_and_discipline_rules_positive() {
    let report = scan(&[(
        "app/api.py",
        python_fixture!("project_agnostic/boundary_and_discipline_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "mutable_default_argument_leaks_state_across_calls",
            "closure_captures_loop_variable_without_binding",
            "path_boundary_accepts_unexpanded_or_relative_input_without_normalization",
            "function_accepts_mapping_protocol_but_mutates_input",
            "boolean_flag_parameter_controls_unrelated_behaviors",
            "expensive_work_starts_before_input_validation",
            "function_returns_multiple_unlabeled_shape_variants",
        ],
    );
}

#[test]
fn test_project_agnostic_hotpath_and_performance_rules_positive() {
    let report = scan(&[(
        "app/processor.py",
        python_fixture!("project_agnostic/hotpath_and_performance_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "regex_compiled_on_each_hot_call",
            "json_roundtrip_used_for_object_copy",
            "membership_test_against_list_or_tuple_literal_inside_loop",
            "list_of_keys_materialized_for_membership_check",
            "subprocess_or_shell_call_inside_record_processing_loop",
            "repeated_file_open_for_same_resource_within_single_operation",
            "bytes_text_bytes_roundtrip_without_transformation",
            "quadratic_string_building_via_plus_equals",
            "generator_materialized_to_tuple_or_list_only_for_len_or_truthiness",
            "any_or_all_wraps_list_comprehension_instead_of_generator",
        ],
    );
}

#[test]
fn test_project_agnostic_quality_maintainability_observability_structure_rules_positive() {
    let report = scan(&[
        (
            "pkg/helpers.py",
            python_fixture!("project_agnostic/quality_helpers_positive.txt"),
        ),
        (
            "pkg/common_manager.py",
            python_fixture!("project_agnostic/quality_common_manager_positive.txt"),
        ),
    ]);

    assert_rules_present(
        &report,
        &[
            "tuple_return_with_three_or_more_positional_fields_in_public_api",
            "parallel_lists_used_instead_of_record_object",
            "logger_instance_created_inside_function_body",
            "expensive_log_argument_built_without_is_enabled_guard",
            "correlation_id_recomputed_multiple_times_in_same_workflow",
            "public_api_returns_none_or_value_without_explicit_optional_contract",
            "fallback_branch_swallows_invariant_violation_and_returns_plausible_default",
            "module_global_registry_mutated_from_import_time_registration",
            "class_mixes_factory_parsing_persistence_and_presentation_roles",
            "sync_and_async_contracts_mixed_on_same_interface_family",
        ],
    );
}

#[test]
fn test_project_agnostic_quality_rule_skips_reraise_handlers() {
    let report = scan(&[(
        "app/api.py",
        python_fixture!("project_agnostic/quality_reraise_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &["fallback_branch_swallows_invariant_violation_and_returns_plausible_default"],
    );
}

#[test]
fn test_project_agnostic_observability_rule_skips_cheap_fstring_logging() {
    let report = scan(&[(
        "app/logging_example.py",
        python_fixture!("project_agnostic/observability_cheap_fstring_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &["expensive_log_argument_built_without_is_enabled_guard"],
    );
}

#[test]
fn test_project_agnostic_observability_rules_positive_batch_one() {
    let report = scan(&[(
        "app/observability_batch_one.py",
        python_fixture!("project_agnostic/observability_batch_one_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "metric_name_contains_dynamic_user_or_data_values",
            "metric_or_span_labels_use_high_cardinality_raw_inputs",
            "metric_emission_occurs_per_item_inside_inner_loop",
            "health_probe_executes_full_dependency_workflow",
            "operation_lacks_single_stable_name_across_logs_metrics_and_traces",
        ],
    );
}

#[test]
fn test_project_agnostic_observability_rules_negative_batch_one() {
    let report = scan(&[(
        "app/observability_batch_one.py",
        python_fixture!("project_agnostic/observability_batch_one_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "metric_name_contains_dynamic_user_or_data_values",
            "metric_or_span_labels_use_high_cardinality_raw_inputs",
            "metric_emission_occurs_per_item_inside_inner_loop",
            "health_probe_executes_full_dependency_workflow",
            "operation_lacks_single_stable_name_across_logs_metrics_and_traces",
        ],
    );
}

#[test]
fn test_project_agnostic_observability_rules_positive_batch_two() {
    let report = scan(&[(
        "app/observability_batch_two.py",
        python_fixture!("project_agnostic/observability_batch_two_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "retry_loop_logs_without_attempt_count_or_backoff_context",
            "exception_log_omits_operation_identifier_or_input_summary",
            "debug_log_serializes_full_large_object_graph",
            "success_and_failure_paths_use_inconsistent_structured_log_keys",
            "timing_metric_wraps_setup_and_teardown_noise_instead_of_core_operation",
        ],
    );
}

#[test]
fn test_project_agnostic_observability_rules_negative_batch_two() {
    let report = scan(&[(
        "app/observability_batch_two.py",
        python_fixture!("project_agnostic/observability_batch_two_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "retry_loop_logs_without_attempt_count_or_backoff_context",
            "exception_log_omits_operation_identifier_or_input_summary",
            "debug_log_serializes_full_large_object_graph",
            "success_and_failure_paths_use_inconsistent_structured_log_keys",
            "timing_metric_wraps_setup_and_teardown_noise_instead_of_core_operation",
        ],
    );
}

#[test]
fn test_project_agnostic_packaging_rules_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "pkg/__init__.py",
            python_fixture!("project_agnostic/packaging_init_positive.txt"),
        ),
        (
            "pkg/core.py",
            python_fixture!("project_agnostic/packaging_core_positive.txt"),
        ),
        (
            "pkg/alpha.py",
            python_fixture!("project_agnostic/packaging_alpha_positive.txt"),
        ),
        (
            "pkg/beta.py",
            python_fixture!("project_agnostic/packaging_beta_positive.txt"),
        ),
        (
            "pkg/gamma.py",
            python_fixture!("project_agnostic/packaging_gamma_positive.txt"),
        ),
        (
            "pkg/delta.py",
            python_fixture!("project_agnostic/packaging_delta_positive.txt"),
        ),
        (
            "pkg/fake_client.py",
            python_fixture!("project_agnostic/packaging_fake_client_positive.txt"),
        ),
        (
            "common/helpers.py",
            python_fixture!("project_agnostic/packaging_common_helpers_positive.txt"),
        ),
        (
            "common/config.py",
            python_fixture!("project_agnostic/packaging_common_config_positive.txt"),
        ),
    ]);

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "heavy_optional_dependency_imported_by_package_root",
            "cli_only_dependency_imported_by_library_entry_module",
            "package_init_performs_metadata_version_lookup_on_import",
            "circular_import_hidden_by_function_local_import_on_hot_path",
            "plugin_discovery_scans_filesystem_each_invocation",
            "test_helpers_shipped_inside_production_package_path",
            "public_api_surface_defined_only_by_import_side_effects",
            "package_root_reexports_large_dependency_tree_by_default",
            "monolithic_common_package_becomes_transitive_dependency_for_most_modules",
        ],
    );
}
