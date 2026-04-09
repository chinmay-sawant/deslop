use super::{FixtureWorkspace, assert_rules_absent, assert_rules_present};

pub(super) const ARCHITECTURE_RULES: &[&str] = &[
    "handler_calls_repository_directly_without_service",
    "handler_calls_gorm_directly_outside_repository",
    "handler_calls_database_sql_directly_outside_repository",
    "service_imports_gin_directly",
    "service_depends_on_transport_request_type",
    "repository_depends_on_gin_or_http",
    "repository_depends_on_service_package",
    "model_package_depends_on_transport_or_gin",
    "middleware_contains_business_orchestration",
    "cmd_or_main_contains_domain_rules",
    "helper_or_utils_package_contains_domain_logic",
    "route_registration_contains_business_logic",
    "domain_constants_declared_in_handler_package",
    "service_calls_handler_helper",
    "repository_returns_framework_builder_to_upper_layer",
    "service_type_outside_service_package",
    "repository_type_outside_repository_package",
    "gorm_model_outside_models_package",
    "request_dto_outside_transport_package",
    "response_dto_outside_transport_package",
    "validator_outside_validation_package",
    "mapper_outside_mapper_package_when_repo_uses_mappers",
    "route_setup_scattered_without_router_package",
    "middleware_type_outside_middleware_package",
    "sql_query_constants_outside_repository_package",
    "transaction_helper_outside_repository_or_uow_package",
    "api_error_type_outside_transport_package",
    "shared_package_named_common_base_utils_with_mixed_exports",
    "cross_layer_import_violation_by_package_name",
    "package_name_role_drift",
    "gin_handler_accepts_more_than_one_body_contract",
    "gin_handler_returns_multiple_response_shapes",
    "gin_handler_binds_directly_into_model",
    "gin_handler_returns_persistence_model_directly",
    "gin_handler_starts_transaction_inline",
    "gin_handler_runs_authorization_business_rules_inline",
    "gin_handler_executes_raw_sql_inline",
    "gin_handler_uses_action_param_switch_for_many_use_cases",
    "gin_handler_mixes_html_json_and_file_responses",
    "gin_handler_stores_context_in_struct_field",
    "gin_context_passed_beyond_request_boundary",
    "gin_handler_uses_global_singletons",
    "gin_handler_contains_retry_or_backoff_orchestration",
    "gin_handler_calls_multiple_repositories_directly",
    "gin_handler_parses_config_or_feature_flags_inline",
    "gin_engine_as_global_singleton",
    "gin_route_registration_anonymous_function_overuse",
    "gin_route_paths_repeated_as_raw_strings",
    "route_groups_created_inside_handlers",
    "middleware_calls_repository_directly",
    "middleware_starts_db_transaction",
    "middleware_writes_business_response_payloads",
    "middleware_mutates_domain_model_state",
    "custom_recovery_logic_repeated_across_handlers",
    "auth_or_tenant_extraction_duplicated_across_handlers",
    "request_id_generation_duplicated_outside_middleware",
    "pagination_binding_duplicated_outside_boundary_helper",
    "body_binding_done_in_middleware_and_handler",
    "response_envelope_shaping_duplicated_across_handlers",
    "router_constructor_builds_concrete_dependencies",
    "service_method_accepts_gin_context",
    "service_method_returns_http_status",
    "service_method_returns_transport_writer_or_gin_h",
    "service_method_returns_gorm_db_or_sql_rows",
    "service_constructor_instantiates_dependencies_internally",
    "service_struct_has_excessive_dependency_count",
    "service_method_accepts_map_string_any_input",
    "service_method_returns_map_string_any_output",
    "service_method_handles_pagination_or_query_parsing",
    "service_method_handles_request_binding_or_header_extraction",
    "service_method_combines_unrelated_write_paths_via_action_string",
    "service_method_mutates_transport_dto_in_place",
    "service_method_accepts_dto_and_persistence_model_together",
    "service_method_imports_http_status_or_abort_semantics",
    "generic_process_execute_handle_service_name_without_domain_noun",
    "same_struct_used_for_bind_persist_and_respond",
    "request_dto_contains_gorm_tags",
    "response_dto_contains_gorm_tags",
    "gorm_model_contains_binding_or_validate_tags",
    "domain_entity_contains_json_tags_without_boundary_exception",
    "persistence_model_contains_http_form_or_uri_tags",
    "nullable_sql_types_leak_into_api_dto",
    "dto_to_model_mapping_in_handler",
    "model_to_dto_mapping_in_handler",
    "mapping_logic_duplicated_across_handlers",
    "patch_dto_uses_non_pointer_fields_for_optional_updates",
    "create_and_update_share_same_dto_despite_conflicting_requiredness",
    "response_dto_uses_omitempty_on_required_contract_fields",
    "gorm_model_contains_calculated_response_fields",
    "giant_model_struct_spans_multiple_subdomains",
    "validation_logic_duplicated_across_handlers",
    "manual_required_checks_after_validate_tags_available",
    "validation_error_response_shape_inconsistent",
    "custom_validator_registration_inside_handler",
    "default_value_injection_scattered_across_handlers",
    "path_param_parsing_duplicated_across_handlers",
    "pagination_validation_missing_shared_bound_helper",
    "sort_or_filter_whitelist_logic_duplicated_across_handlers",
    "multiple_bind_sources_into_same_struct_without_precedence_contract",
    "query_path_and_body_merge_performed_in_handler",
    "business_validation_mixed_with_persistence_calls_in_handler",
    "validator_depends_on_repository_directly",
    "request_context_value_extraction_duplicated_across_handlers",
    "file_upload_validation_mixed_with_storage_write",
    "route_param_name_and_dto_field_name_drift_without_mapping_helper",
    "repository_returns_http_status_errors",
    "service_returns_preformatted_client_message",
    "domain_errors_declared_in_handler_package",
    "inline_error_to_status_mapping_duplicated",
    "multiple_error_envelope_shapes_same_api_module",
    "handler_switches_on_error_strings",
    "middleware_and_handler_translate_same_error_domain",
    "raw_db_error_exposed_to_client",
    "same_domain_error_mapped_to_multiple_statuses",
    "handler_and_service_both_log_same_error_chain",
    "not_found_semantics_implemented_by_nil_nil_return",
    "service_depends_on_gin_abort_or_context_error_response",
    "error_code_literals_duplicated_across_handlers",
    "transport_layer_uses_untyped_string_codes_without_catalog",
    "success_response_contains_error_field_or_mixed_contract",
    "raw_sql_literal_in_handler",
    "raw_sql_literal_in_service",
    "gorm_chain_built_in_handler",
    "gorm_chain_built_in_service",
    "repository_accepts_gin_context",
    "repository_accepts_http_request_dto",
    "repository_returns_transport_dto",
    "repository_returns_gorm_query_builder",
    "table_name_literals_duplicated_outside_repository",
    "column_name_literals_duplicated_outside_repository",
    "order_by_clause_literals_scattered_across_layers",
    "where_clause_templates_duplicated_across_repositories",
    "repository_mixes_raw_sql_and_gorm_same_method_without_adapter_boundary",
    "sql_rows_scan_logic_outside_repository",
    "sql_null_types_escape_repository_boundary",
    "gorm_scopes_defined_inline_repeatedly",
    "preload_rules_scattered_across_layers",
    "soft_delete_filters_written_manually_in_many_queries",
    "unscoped_query_without_explicit_danger_naming",
    "gorm_model_hook_contains_external_io",
    "gorm_model_hook_calls_service_or_repository",
    "gorm_hook_mutates_unrelated_tables",
    "repository_method_returns_partially_built_scopes_for_caller_chaining",
    "gorm_session_options_configured_outside_repository",
    "gorm_locking_clauses_built_outside_repository",
    "updates_with_struct_used_for_patch_without_field_intent_helper",
    "map_based_updates_passed_from_handler_to_repository",
    "generic_base_repository_with_reflection_dispatch",
    "shared_gorm_db_state_mutated_and_reused_across_requests",
    "table_name_override_or_scope_logic_duplicated_across_models",
    "handler_opens_transaction",
    "middleware_opens_transaction",
    "service_returns_tx_to_caller",
    "repository_begins_transaction_without_uow_or_callback",
    "transaction_object_crosses_more_than_one_layer_boundary",
    "same_service_method_accepts_tx_and_begins_tx",
    "optional_tx_nil_parameter_on_repository_api",
    "cross_repository_write_flow_without_shared_uow_boundary",
    "commit_or_rollback_split_across_functions_without_owner",
    "external_http_call_inside_transaction_scope",
    "event_publish_before_transaction_commit",
    "cache_invalidation_before_transaction_commit",
    "background_goroutine_started_inside_transaction_scope",
    "transaction_error_translation_done_in_repository_and_handler",
    "savepoint_or_nested_tx_logic_scattered_without_dedicated_helper",
    "constructor_reads_env_directly",
    "router_setup_runs_migrations",
    "bootstrap_builds_clients_inside_route_registration",
    "init_registers_routes_or_dependencies",
    "package_level_mutable_config_used_by_handlers_services",
    "service_constructor_accepts_untyped_config_map",
    "repository_constructor_accepts_gin_engine_or_router",
    "middleware_uses_global_logger_or_config_singleton",
    "same_dependency_wired_in_multiple_bootstrap_locations",
    "feature_flag_lookup_without_config_abstraction",
    "background_worker_started_from_http_handler_registration",
    "main_or_cmd_mixes_cli_parsing_server_bootstrap_and_business_config",
    "application_lifecycle_missing_shutdown_owner",
    "migration_runner_and_api_server_bootstrap_coupled",
    "test_bootstrap_package_reused_by_production_wiring",
    "service_tests_import_gin",
    "repository_tests_depend_on_http_transport_types",
    "handler_tests_use_real_database_without_seam",
    "test_helpers_duplicated_across_packages",
    "mock_repository_types_duplicated_across_tests",
    "test_fixture_builders_live_in_production_packages",
    "route_registration_tests_duplicate_full_bootstrap_per_file",
    "tests_assert_raw_json_strings_without_response_dto",
    "tests_couple_to_gorm_model_for_api_contract_assertions",
    "tests_stub_gin_context_instead_of_httptest_boundary",
    "transport_tests_bypass_service_interface_and_touch_repo_directly",
    "sql_query_text_asserted_in_handler_tests",
    "migration_tests_live_under_handler_packages",
    "table_driven_tests_mix_multiple_domains_in_one_cases_slice",
    "shared_integration_test_setup_not_centralized_under_test_support",
    "metrics_labels_built_inline_in_handlers",
    "tracing_span_names_duplicated_as_raw_strings",
    "transport_metrics_emitted_from_repository_layer",
    "repository_logs_with_http_status_or_route_labels",
    "audit_logging_executed_in_handler_before_service_success",
    "request_logging_fields_assembled_differently_across_handlers",
    "domain_identifiers_logged_under_inconsistent_field_keys",
    "health_or_readiness_handlers_reach_into_business_repositories_directly",
    "admin_or_debug_endpoint_registration_mixed_into_public_router_setup",
    "migration_or_seed_logic_callable_from_request_handlers",
    "background_jobs_registered_from_gin_packages_instead_of_bootstrap",
    "operational_command_handlers_reuse_http_services_without_adapter",
    "swagger_or_openapi_annotations_on_persistence_models",
    "api_examples_embedded_in_handlers_instead_of_transport_docs_helpers",
    "repository_or_service_packages_import_docs_or_generator_annotations",
];

#[test]
fn test_go_architecture_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "internal/transport/routes.go",
            go_fixture!("architecture/positive_transport_routes.txt"),
        ),
        (
            "internal/handler/user_handler.go",
            go_fixture!("architecture/positive_handler.txt"),
        ),
        (
            "internal/handler/advanced_handler.go",
            go_fixture!("architecture/positive_handler_advanced.txt"),
        ),
        (
            "internal/handler/auth_middleware.go",
            go_fixture!("architecture/positive_middleware_transport.txt"),
        ),
        (
            "internal/handler/advanced_routes.go",
            go_fixture!("architecture/positive_router_advanced.txt"),
        ),
        (
            "internal/validation/user_validator.go",
            go_fixture!("architecture/positive_validator.txt"),
        ),
        (
            "internal/utils/shared.go",
            go_fixture!("architecture/positive_utils.txt"),
        ),
        (
            "internal/service/user_service.go",
            go_fixture!("architecture/positive_service.txt"),
        ),
        (
            "internal/service/advanced_service.go",
            go_fixture!("architecture/positive_service_advanced.txt"),
        ),
        (
            "internal/application/domain_types.go",
            go_fixture!("architecture/positive_service_outside.txt"),
        ),
        (
            "internal/repository/user_repository.go",
            go_fixture!("architecture/positive_repository.txt"),
        ),
        (
            "internal/repository/advanced_repository.go",
            go_fixture!("architecture/positive_repository_advanced.txt"),
        ),
        (
            "internal/domain/persistence_types.go",
            go_fixture!("architecture/positive_repository_outside.txt"),
        ),
        (
            "internal/entity/user.go",
            go_fixture!("architecture/positive_model_outside.txt"),
        ),
        (
            "internal/models/user.go",
            go_fixture!("architecture/positive_models.txt"),
        ),
        (
            "internal/models/advanced_models.go",
            go_fixture!("architecture/positive_models_advanced.txt"),
        ),
        (
            "internal/contracts/user_contracts.go",
            go_fixture!("architecture/positive_contracts.txt"),
        ),
        (
            "internal/router/routes.go",
            go_fixture!("architecture/positive_router.txt"),
        ),
        (
            "internal/router/sloppy_routes.go",
            go_fixture!("architecture/positive_router_bootstrap.txt"),
        ),
        (
            "cmd/api/main.go",
            go_fixture!("architecture/positive_cmd_main.txt"),
        ),
        (
            "internal/handler/user_handler_test.go",
            go_fixture!("architecture/positive_handler_test.txt"),
        ),
        (
            "internal/service/user_service_test.go",
            go_fixture!("architecture/positive_service_test.txt"),
        ),
        (
            "internal/repository/user_repository_test.go",
            go_fixture!("architecture/positive_repository_test.txt"),
        ),
    ]);

    let report = workspace.scan();
    assert_rules_present(&report, ARCHITECTURE_RULES);
}

#[test]
fn test_go_architecture_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "internal/handler/user_handler.go",
            go_fixture!("architecture/clean_handler.txt"),
        ),
        (
            "internal/service/user_service.go",
            go_fixture!("architecture/clean_service.txt"),
        ),
        (
            "internal/repository/user_repository.go",
            go_fixture!("architecture/clean_repository.txt"),
        ),
        (
            "internal/models/user.go",
            go_fixture!("architecture/clean_models.txt"),
        ),
        (
            "internal/transport/contracts.go",
            go_fixture!("architecture/clean_contracts.txt"),
        ),
        (
            "internal/router/routes.go",
            go_fixture!("architecture/clean_router.txt"),
        ),
        (
            "internal/service/user_service_test.go",
            go_fixture!("architecture/clean_service_test.txt"),
        ),
        (
            "internal/repository/user_repository_test.go",
            go_fixture!("architecture/clean_repository_test.txt"),
        ),
    ]);

    let report = workspace.scan();
    assert_rules_absent(&report, ARCHITECTURE_RULES);
}

#[test]
fn test_upstream_consumed_interface_declared_in_provider_package() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "internal/repository/user_repository.go",
            go_fixture!("architecture/positive_provider_interface_repository.txt"),
        ),
        (
            "internal/service/user_service.go",
            go_fixture!("architecture/positive_provider_interface_service.txt"),
        ),
    ]);

    let report = workspace.scan();
    assert_rules_present(
        &report,
        &["upstream_consumed_interface_declared_in_provider_package"],
    );
}

#[test]
fn test_upstream_consumed_interface_declared_in_provider_package_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "internal/service/user_service.go",
            go_fixture!("architecture/clean_provider_interface_service.txt"),
        ),
        (
            "internal/repository/user_repository.go",
            go_fixture!("architecture/clean_provider_interface_repository.txt"),
        ),
    ]);

    let report = workspace.scan();
    assert_rules_absent(
        &report,
        &["upstream_consumed_interface_declared_in_provider_package"],
    );
}

#[test]
fn test_tool_appeasement_noop_type_in_production_package() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/repository/noop_patient_repository.go",
        go_fixture!("architecture/positive_tool_noop.txt"),
    );

    let report = workspace.scan();
    assert_rules_present(
        &report,
        &["tool_appeasement_noop_type_in_production_package"],
    );
}

#[test]
fn test_tool_appeasement_noop_type_in_production_package_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "internal/repository/noop_audit_logger.go",
            go_fixture!("architecture/clean_tool_noop_repository.txt"),
        ),
        (
            "internal/service/audit_service.go",
            go_fixture!("architecture/clean_tool_noop_service.txt"),
        ),
    ]);

    let report = workspace.scan();
    assert_rules_absent(
        &report,
        &["tool_appeasement_noop_type_in_production_package"],
    );
}

#[test]
fn test_root_main_go_in_layered_service_repo() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "main.go",
            go_fixture!("architecture/positive_root_main.txt"),
        ),
        (
            "internal/service/user_service.go",
            go_fixture!("architecture/layered_service_file.txt"),
        ),
        (
            "internal/repository/user_repository.go",
            go_fixture!("architecture/layered_repository_file.txt"),
        ),
        (
            "internal/handler/user_handler.go",
            go_fixture!("architecture/layered_handler_file.txt"),
        ),
    ]);

    let report = workspace.scan();
    assert_rules_present(&report, &["root_main_go_in_layered_service_repo"]);
}

#[test]
fn test_root_main_go_in_layered_service_repo_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "cmd/api/main.go",
            go_fixture!("architecture/clean_cmd_main.txt"),
        ),
        (
            "internal/service/user_service.go",
            go_fixture!("architecture/layered_service_file.txt"),
        ),
        (
            "internal/repository/user_repository.go",
            go_fixture!("architecture/layered_repository_file.txt"),
        ),
        (
            "internal/handler/user_handler.go",
            go_fixture!("architecture/layered_handler_file.txt"),
        ),
    ]);

    let report = workspace.scan();
    assert_rules_absent(&report, &["root_main_go_in_layered_service_repo"]);
}

#[test]
fn test_db_query_argument_erased_to_any() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/repository/user_repository.go",
        go_fixture!("architecture/positive_db_any.txt"),
    );

    let report = workspace.scan();
    assert_rules_present(&report, &["db_query_argument_erased_to_any"]);
}

#[test]
fn test_db_query_argument_erased_to_any_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/repository/user_repository.go",
        go_fixture!("architecture/clean_db_any.txt"),
    );

    let report = workspace.scan();
    assert_rules_absent(&report, &["db_query_argument_erased_to_any"]);
}

#[test]
fn test_gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "cmd/api/main.go",
            go_fixture!("architecture/positive_gorm_bootstrap_main.txt"),
        ),
        (
            "internal/service/patient_service.go",
            go_fixture!("architecture/layered_service_file.txt"),
        ),
        (
            "internal/repository/patient_repository.go",
            go_fixture!("architecture/positive_gorm_bootstrap_repository.txt"),
        ),
    ]);

    let report = workspace.scan();
    assert_rules_present(
        &report,
        &["gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary"],
    );
}

#[test]
fn test_gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "cmd/api/main.go",
            go_fixture!("architecture/positive_gorm_bootstrap_main.txt"),
        ),
        (
            "internal/service/patient_service.go",
            go_fixture!("architecture/layered_service_file.txt"),
        ),
        (
            "internal/repository/patient_repository.go",
            go_fixture!("architecture/clean_gorm_bootstrap_repository.txt"),
        ),
    ]);

    let report = workspace.scan();
    assert_rules_absent(
        &report,
        &["gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary"],
    );
}

#[test]
fn test_service_write_passthrough_without_domain_validation() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/service/patient_service.go",
        go_fixture!("architecture/positive_service_passthrough.txt"),
    );

    let report = workspace.scan();
    assert_rules_present(
        &report,
        &["service_write_passthrough_without_domain_validation"],
    );
}

#[test]
fn test_service_write_passthrough_without_domain_validation_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/service/patient_service.go",
        go_fixture!("architecture/clean_service_passthrough.txt"),
    );

    let report = workspace.scan();
    assert_rules_absent(
        &report,
        &["service_write_passthrough_without_domain_validation"],
    );
}

#[test]
fn test_client_input_error_mapped_to_internal_server_error() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/handler/user_handler.go",
        go_fixture!("architecture/positive_client_input_500.txt"),
    );

    let report = workspace.scan();
    assert_rules_present(
        &report,
        &["client_input_error_mapped_to_internal_server_error"],
    );
}

#[test]
fn test_client_input_error_mapped_to_internal_server_error_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "internal/handler/user_handler.go",
        go_fixture!("architecture/clean_client_input_500.txt"),
    );

    let report = workspace.scan();
    assert_rules_absent(
        &report,
        &["client_input_error_mapped_to_internal_server_error"],
    );
}
