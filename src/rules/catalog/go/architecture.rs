use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

macro_rules! architecture_rule {
    ($id:literal, $severity:ident, $description:literal) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Go,
            family: "architecture",
            default_severity: RuleDefaultSeverity::$severity,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $description,
            binding_location: super::bindings::GO_ARCHITECTURE,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    architecture_rule!(
        "admin_or_debug_endpoint_registration_mixed_into_public_router_setup",
        Info,
        "Operational endpoints registered alongside public routes with no clear boundary."
    ),
    architecture_rule!(
        "api_error_type_outside_transport_package",
        Info,
        "API-facing error payload structs living in persistence or business packages instead of transport-boundary packages."
    ),
    architecture_rule!(
        "api_examples_embedded_in_handlers_instead_of_transport_docs_helpers",
        Info,
        "Long example payload blocks hardcoded in handlers rather than doc or example helpers."
    ),
    architecture_rule!(
        "application_lifecycle_missing_shutdown_owner",
        Info,
        "Shared resources with visible startup paths but no obvious shutdown owner or lifecycle wrapper."
    ),
    architecture_rule!(
        "audit_logging_executed_in_handler_before_service_success",
        Info,
        "Audit writes that happen before the service has confirmed success."
    ),
    architecture_rule!(
        "auth_or_tenant_extraction_duplicated_across_handlers",
        Info,
        "Repeated auth, tenant, or principal extraction that should live in middleware or a shared boundary helper."
    ),
    architecture_rule!(
        "background_goroutine_started_inside_transaction_scope",
        Warning,
        "Goroutines launched while relying on open transaction state."
    ),
    architecture_rule!(
        "background_jobs_registered_from_gin_packages_instead_of_bootstrap",
        Info,
        "Scheduler or worker registration hidden inside Gin transport packages."
    ),
    architecture_rule!(
        "background_worker_started_from_http_handler_registration",
        Info,
        "Route or middleware setup that also starts unrelated background jobs."
    ),
    architecture_rule!(
        "body_binding_done_in_middleware_and_handler",
        Info,
        "Endpoints where middleware parses the body and the handler also owns body binding."
    ),
    architecture_rule!(
        "bootstrap_builds_clients_inside_route_registration",
        Info,
        "Startup code that hides dependency construction inside route registration functions."
    ),
    architecture_rule!(
        "business_validation_mixed_with_persistence_calls_in_handler",
        Info,
        "Handlers that start querying or writing before finishing request validation."
    ),
    architecture_rule!(
        "cache_invalidation_before_transaction_commit",
        Warning,
        "Cache mutation or eviction that happens before a write transaction commits."
    ),
    architecture_rule!(
        "client_input_error_mapped_to_internal_server_error",
        Warning,
        "Request parsing or binding errors that are translated into `500` responses instead of client-error status codes."
    ),
    architecture_rule!(
        "cmd_or_main_contains_domain_rules",
        Info,
        "`main` or `cmd` packages that contain business-rule branches instead of wiring and startup concerns."
    ),
    architecture_rule!(
        "column_name_literals_duplicated_outside_repository",
        Info,
        "Repeated column-name strings outside repository or query packages."
    ),
    architecture_rule!(
        "commit_or_rollback_split_across_functions_without_owner",
        Info,
        "Transaction end-state logic spread across helper functions with unclear ownership."
    ),
    architecture_rule!(
        "constructor_reads_env_directly",
        Info,
        "Constructors that call `os.Getenv` instead of receiving normalized config."
    ),
    architecture_rule!(
        "db_query_argument_erased_to_any",
        Info,
        "Concrete DB arguments erased to `any` or `interface{}` immediately before query execution."
    ),
    architecture_rule!(
        "create_and_update_share_same_dto_despite_conflicting_requiredness",
        Info,
        "A single DTO reused for create and update when validation rules clearly differ."
    ),
    architecture_rule!(
        "cross_layer_import_violation_by_package_name",
        Warning,
        "Imports where package names clearly violate a layered direction such as `repository -> handler` or `model -> gin`."
    ),
    architecture_rule!(
        "cross_repository_write_flow_without_shared_uow_boundary",
        Warning,
        "Service methods coordinating several repositories with no explicit shared transaction or consistency boundary."
    ),
    architecture_rule!(
        "custom_recovery_logic_repeated_across_handlers",
        Info,
        "Repeated recover or panic-to-response logic inside handlers instead of centralized recovery middleware."
    ),
    architecture_rule!(
        "custom_validator_registration_inside_handler",
        Info,
        "Per-request validator registration instead of startup-time validator wiring."
    ),
    architecture_rule!(
        "default_value_injection_scattered_across_handlers",
        Info,
        "Repeated defaulting logic for pagination, filters, or booleans across handlers."
    ),
    architecture_rule!(
        "domain_constants_declared_in_handler_package",
        Info,
        "Domain enums, statuses, and lifecycle constants that live in handler packages instead of domain-facing packages."
    ),
    architecture_rule!(
        "domain_entity_contains_json_tags_without_boundary_exception",
        Info,
        "Core domain entities that are directly annotated for API serialization without a deliberate boundary exception."
    ),
    architecture_rule!(
        "domain_errors_declared_in_handler_package",
        Info,
        "Business error types declared only in handlers instead of a reusable domain or service boundary."
    ),
    architecture_rule!(
        "domain_identifiers_logged_under_inconsistent_field_keys",
        Info,
        "The same entity ID being logged under many different keys across packages."
    ),
    architecture_rule!(
        "dto_to_model_mapping_in_handler",
        Info,
        "Handlers that own transport-to-model mapping instead of dedicated mappers or services."
    ),
    architecture_rule!(
        "error_code_literals_duplicated_across_handlers",
        Info,
        "Repeated string error codes without a centralized catalog."
    ),
    architecture_rule!(
        "event_publish_before_transaction_commit",
        Warning,
        "Event or message publication before durable commit is confirmed."
    ),
    architecture_rule!(
        "external_http_call_inside_transaction_scope",
        Warning,
        "Network calls performed while a DB transaction is open."
    ),
    architecture_rule!(
        "feature_flag_lookup_without_config_abstraction",
        Info,
        "Handlers or services that query feature flags directly without a focused flag interface."
    ),
    architecture_rule!(
        "gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary",
        Info,
        "Codebases that bootstrap persistence with GORM but run repository queries through raw SQL without a clear adapter boundary."
    ),
    architecture_rule!(
        "file_upload_validation_mixed_with_storage_write",
        Info,
        "Functions that validate upload metadata and also persist the upload in the same boundary step."
    ),
    architecture_rule!(
        "generic_base_repository_with_reflection_dispatch",
        Info,
        "Catch-all repositories that depend on reflection-heavy generic CRUD instead of bounded repository APIs."
    ),
    architecture_rule!(
        "generic_process_execute_handle_service_name_without_domain_noun",
        Info,
        "Generic service names and methods that hide business intent instead of naming the actual use case."
    ),
    architecture_rule!(
        "giant_model_struct_spans_multiple_subdomains",
        Info,
        "Oversized model structs that appear to merge multiple bounded contexts into one persistence shape."
    ),
    architecture_rule!(
        "gin_context_passed_beyond_request_boundary",
        Warning,
        "Code that passes `*gin.Context` into goroutines, repositories, or long-lived service objects instead of extracting `context.Context` and request values."
    ),
    architecture_rule!(
        "gin_engine_as_global_singleton",
        Warning,
        "Package-level `*gin.Engine` singletons reused across unrelated startup paths."
    ),
    architecture_rule!(
        "gin_handler_accepts_more_than_one_body_contract",
        Info,
        "Handlers that try to bind or manually parse multiple body DTOs in one request path."
    ),
    architecture_rule!(
        "gin_handler_binds_directly_into_model",
        Warning,
        "Handlers that bind request bodies directly into GORM or persistence structs."
    ),
    architecture_rule!(
        "gin_handler_calls_multiple_repositories_directly",
        Info,
        "Handlers that coordinate several repositories directly, which usually signals missing service orchestration."
    ),
    architecture_rule!(
        "gin_handler_contains_retry_or_backoff_orchestration",
        Info,
        "Handlers that implement retry loops or backoff decisions inline instead of delegating to service or client layers."
    ),
    architecture_rule!(
        "gin_handler_executes_raw_sql_inline",
        Warning,
        "Handlers that execute literal SQL or repository-unaware raw queries."
    ),
    architecture_rule!(
        "gin_handler_mixes_html_json_and_file_responses",
        Info,
        "Handlers that serve fundamentally different transport modes from one function."
    ),
    architecture_rule!(
        "gin_handler_parses_config_or_feature_flags_inline",
        Info,
        "Handlers that read feature flags, tenant configuration, or rollout toggles directly instead of using injected policy/config abstractions."
    ),
    architecture_rule!(
        "gin_handler_returns_multiple_response_shapes",
        Info,
        "Handlers that manually build several unrelated success payload shapes instead of delegating to clearer endpoints or render helpers."
    ),
    architecture_rule!(
        "gin_handler_returns_persistence_model_directly",
        Warning,
        "Handlers that serialize database models directly to the client instead of mapping to response DTOs."
    ),
    architecture_rule!(
        "gin_handler_runs_authorization_business_rules_inline",
        Info,
        "Handlers that contain complex permission matrices instead of delegating to policy or service logic."
    ),
    architecture_rule!(
        "gin_handler_starts_transaction_inline",
        Warning,
        "Handlers that create or own transactions instead of delegating transactional work to services or unit-of-work helpers."
    ),
    architecture_rule!(
        "gin_handler_stores_context_in_struct_field",
        Warning,
        "Code that saves `*gin.Context` onto long-lived structs."
    ),
    architecture_rule!(
        "gin_handler_uses_action_param_switch_for_many_use_cases",
        Info,
        "Handlers that branch on `action`, `type`, or mode strings to serve many workflows from one endpoint function."
    ),
    architecture_rule!(
        "gin_handler_uses_global_singletons",
        Info,
        "Handlers that reach for global DB, logger, cache, or config variables instead of injected dependencies."
    ),
    architecture_rule!(
        "gin_route_paths_repeated_as_raw_strings",
        Info,
        "Repeated route path literals that should be centralized or grouped more intentionally."
    ),
    architecture_rule!(
        "gin_route_registration_anonymous_function_overuse",
        Info,
        "Route registration that uses many inline anonymous handlers instead of named handler methods or functions."
    ),
    architecture_rule!(
        "gorm_chain_built_in_handler",
        Warning,
        "Handlers that construct `Where`, `Joins`, `Scopes`, or `Order` chains directly."
    ),
    architecture_rule!(
        "gorm_chain_built_in_service",
        Warning,
        "Services that own detailed ORM query shape instead of delegating to repositories."
    ),
    architecture_rule!(
        "gorm_hook_mutates_unrelated_tables",
        Info,
        "Hooks that reach beyond the owning aggregate and write unrelated records."
    ),
    architecture_rule!(
        "gorm_locking_clauses_built_outside_repository",
        Warning,
        "`FOR UPDATE` or similar locking behavior built in handlers or services instead of repository-owned methods."
    ),
    architecture_rule!(
        "gorm_model_contains_binding_or_validate_tags",
        Warning,
        "GORM models that accumulate transport validation and binding concerns."
    ),
    architecture_rule!(
        "gorm_model_contains_calculated_response_fields",
        Info,
        "Persistence structs that carry API-only derived presentation fields."
    ),
    architecture_rule!(
        "gorm_model_hook_calls_service_or_repository",
        Warning,
        "Model hooks that depend on higher-layer services or repositories."
    ),
    architecture_rule!(
        "gorm_model_hook_contains_external_io",
        Warning,
        "Hooks that call HTTP clients, queues, file I/O, or external side effects."
    ),
    architecture_rule!(
        "gorm_model_outside_models_package",
        Info,
        "GORM-backed structs with `gorm` tags declared outside `model` or `models` packages unless an explicit `entity` or `schema` package already exists."
    ),
    architecture_rule!(
        "gorm_scopes_defined_inline_repeatedly",
        Info,
        "Repeated ad hoc scope functions instead of shared named scopes."
    ),
    architecture_rule!(
        "gorm_session_options_configured_outside_repository",
        Info,
        "Code above the repository layer that changes `Session`, `Clauses`, or transaction options directly."
    ),
    architecture_rule!(
        "handler_and_service_both_log_same_error_chain",
        Info,
        "Duplicated logging ownership where both service and handler log the same failure."
    ),
    architecture_rule!(
        "handler_calls_database_sql_directly_outside_repository",
        Warning,
        "Handler packages that import `database/sql`, `sqlx`, or `pgx` and execute queries directly."
    ),
    architecture_rule!(
        "handler_calls_gorm_directly_outside_repository",
        Warning,
        "Handler packages that import `gorm.io/gorm` and build query chains directly."
    ),
    architecture_rule!(
        "handler_calls_repository_directly_without_service",
        Warning,
        "Handlers that bypass a visible service layer and call repositories or query helpers directly for business orchestration."
    ),
    architecture_rule!(
        "handler_opens_transaction",
        Warning,
        "Handlers that call `Begin`, `Transaction`, or unit-of-work start helpers directly."
    ),
    architecture_rule!(
        "handler_switches_on_error_strings",
        Warning,
        "Handlers that branch on `err.Error()` text rather than typed errors."
    ),
    architecture_rule!(
        "handler_tests_use_real_database_without_seam",
        Info,
        "Handler tests that hit real DB setup even though repository or service seams exist."
    ),
    architecture_rule!(
        "health_or_readiness_handlers_reach_into_business_repositories_directly",
        Info,
        "Health endpoints that depend on full business repositories instead of focused probes."
    ),
    architecture_rule!(
        "helper_or_utils_package_contains_domain_logic",
        Info,
        "Generic `helper`, `common`, or `utils` packages that host concrete domain decisions."
    ),
    architecture_rule!(
        "init_registers_routes_or_dependencies",
        Warning,
        "`init()` functions that register handlers, routes, or runtime dependencies."
    ),
    architecture_rule!(
        "inline_error_to_status_mapping_duplicated",
        Info,
        "Repeated error-to-status translation switches across handlers."
    ),
    architecture_rule!(
        "main_or_cmd_mixes_cli_parsing_server_bootstrap_and_business_config",
        Info,
        "Entrypoints that do too much without startup composition helpers."
    ),
    architecture_rule!(
        "root_main_go_in_layered_service_repo",
        Info,
        "Layered service repos that keep the primary binary at repository-root `main.go` instead of `cmd/<name>/main.go`."
    ),
    architecture_rule!(
        "manual_required_checks_after_validate_tags_available",
        Info,
        "Handlers that re-check required fields manually even though struct-tag validation already exists."
    ),
    architecture_rule!(
        "map_based_updates_passed_from_handler_to_repository",
        Info,
        "Update maps built from request payloads and passed through layers without typed ownership."
    ),
    architecture_rule!(
        "mapper_outside_mapper_package_when_repo_uses_mappers",
        Info,
        "Mapper functions scattered across handlers and services when the repo already has a dedicated mapping package."
    ),
    architecture_rule!(
        "mapping_logic_duplicated_across_handlers",
        Info,
        "Repeated field mapping blocks across multiple handlers."
    ),
    architecture_rule!(
        "metrics_labels_built_inline_in_handlers",
        Info,
        "Handlers that hand-build metric labels repeatedly instead of using focused observation helpers."
    ),
    architecture_rule!(
        "middleware_and_handler_translate_same_error_domain",
        Info,
        "Double translation layers where middleware and handlers both map the same error family to responses."
    ),
    architecture_rule!(
        "middleware_calls_repository_directly",
        Info,
        "Middleware that reaches into repositories instead of delegating through focused services or auth abstractions."
    ),
    architecture_rule!(
        "middleware_contains_business_orchestration",
        Info,
        "Middleware that performs domain workflows instead of cross-cutting concerns such as auth, tracing, or request enrichment."
    ),
    architecture_rule!(
        "middleware_mutates_domain_model_state",
        Warning,
        "Middleware that writes business entities or persistence models as a side effect of request plumbing."
    ),
    architecture_rule!(
        "middleware_opens_transaction",
        Warning,
        "Middleware that owns transaction lifetime for all downstream business logic."
    ),
    architecture_rule!(
        "middleware_starts_db_transaction",
        Warning,
        "Middleware that opens database transactions for the whole request path by default."
    ),
    architecture_rule!(
        "middleware_type_outside_middleware_package",
        Info,
        "Shared middleware types and constructors declared in handlers or services instead of a middleware-focused package."
    ),
    architecture_rule!(
        "middleware_uses_global_logger_or_config_singleton",
        Info,
        "Middleware that depends on mutable globals instead of injected config and logger instances."
    ),
    architecture_rule!(
        "middleware_writes_business_response_payloads",
        Info,
        "Middleware that emits domain success payloads instead of leaving response bodies to handlers or render helpers."
    ),
    architecture_rule!(
        "migration_or_seed_logic_callable_from_request_handlers",
        Warning,
        "Request handlers that can trigger migration or seed workflows."
    ),
    architecture_rule!(
        "migration_runner_and_api_server_bootstrap_coupled",
        Info,
        "One startup path that always runs migrations and serves traffic together."
    ),
    architecture_rule!(
        "migration_tests_live_under_handler_packages",
        Info,
        "Migration-specific tests located in transport packages."
    ),
    architecture_rule!(
        "mock_repository_types_duplicated_across_tests",
        Info,
        "Many copy-pasted mock repository structs with the same method sets."
    ),
    architecture_rule!(
        "model_package_depends_on_transport_or_gin",
        Warning,
        "`model` or `models` packages that import transport-facing libraries or handler DTO packages."
    ),
    architecture_rule!(
        "model_to_dto_mapping_in_handler",
        Info,
        "Handlers that manually shape every response from persistence structs rather than using dedicated mapping helpers."
    ),
    architecture_rule!(
        "multiple_bind_sources_into_same_struct_without_precedence_contract",
        Info,
        "One DTO being filled from body, query, and path without a clear precedence policy."
    ),
    architecture_rule!(
        "multiple_error_envelope_shapes_same_api_module",
        Info,
        "Endpoints in the same API surface that serialize unrelated error payload structures."
    ),
    architecture_rule!(
        "not_found_semantics_implemented_by_nil_nil_return",
        Info,
        "Repository or service APIs that encode missing records as `nil, nil` with no explicit result contract."
    ),
    architecture_rule!(
        "nullable_sql_types_leak_into_api_dto",
        Info,
        "Response or request DTOs that expose `sql.Null*` types instead of boundary-friendly contract types."
    ),
    architecture_rule!(
        "operational_command_handlers_reuse_http_services_without_adapter",
        Info,
        "CLI or cron entrypoints that import Gin-only service contracts instead of neutral application services."
    ),
    architecture_rule!(
        "optional_tx_nil_parameter_on_repository_api",
        Info,
        "Repository APIs that use optional `tx *gorm.DB` parameters instead of explicit unit-of-work contracts."
    ),
    architecture_rule!(
        "order_by_clause_literals_scattered_across_layers",
        Info,
        "Raw sort expressions duplicated in handlers and services instead of query helpers."
    ),
    architecture_rule!(
        "package_level_mutable_config_used_by_handlers_services",
        Info,
        "Mutable package globals used as live config in request paths."
    ),
    architecture_rule!(
        "package_name_role_drift",
        Info,
        "Packages whose names advertise one role, such as `service` or `model`, but whose exported symbols primarily belong to another role."
    ),
    architecture_rule!(
        "tool_appeasement_noop_type_in_production_package",
        Info,
        "Production no-op or dummy types that appear to exist mainly to satisfy tooling instead of runtime behavior."
    ),
    architecture_rule!(
        "pagination_binding_duplicated_outside_boundary_helper",
        Info,
        "Repeated pagination DTO binding and defaulting logic across handler files."
    ),
    architecture_rule!(
        "pagination_validation_missing_shared_bound_helper",
        Info,
        "API modules that hand-roll page and page-size bounds in many handlers instead of a shared contract."
    ),
    architecture_rule!(
        "patch_dto_uses_non_pointer_fields_for_optional_updates",
        Warning,
        "PATCH-like DTOs that cannot distinguish omitted fields from zero values."
    ),
    architecture_rule!(
        "path_param_parsing_duplicated_across_handlers",
        Info,
        "Repeated `strconv` or UUID path-param parsing logic that should live in helpers."
    ),
    architecture_rule!(
        "persistence_model_contains_http_form_or_uri_tags",
        Warning,
        "Persistence models that are tied to form or URI binding tags."
    ),
    architecture_rule!(
        "preload_rules_scattered_across_layers",
        Info,
        "Preload decisions spread across handlers, services, and repositories instead of one query owner."
    ),
    architecture_rule!(
        "query_path_and_body_merge_performed_in_handler",
        Info,
        "Handlers that merge several request sources inline instead of normalizing inputs before service calls."
    ),
    architecture_rule!(
        "raw_db_error_exposed_to_client",
        Warning,
        "Direct serialization of SQL or GORM error text into API responses."
    ),
    architecture_rule!(
        "raw_sql_literal_in_handler",
        Warning,
        "Literal SQL strings declared and executed from handlers."
    ),
    architecture_rule!(
        "raw_sql_literal_in_service",
        Warning,
        "Literal SQL strings executed from services instead of repositories or query packages."
    ),
    architecture_rule!(
        "repository_accepts_gin_context",
        Warning,
        "Repositories that accept `*gin.Context` rather than `context.Context`."
    ),
    architecture_rule!(
        "repository_accepts_http_request_dto",
        Warning,
        "Repositories that accept transport DTOs instead of repository inputs or domain values."
    ),
    architecture_rule!(
        "repository_begins_transaction_without_uow_or_callback",
        Info,
        "Repositories that quietly start transactions inside generic CRUD methods."
    ),
    architecture_rule!(
        "repository_constructor_accepts_gin_engine_or_router",
        Warning,
        "Repository constructors that take transport objects."
    ),
    architecture_rule!(
        "repository_depends_on_gin_or_http",
        Warning,
        "Repository packages that import Gin, `net/http`, or transport-only response helpers."
    ),
    architecture_rule!(
        "repository_depends_on_service_package",
        Warning,
        "Repositories that import service packages, which inverts the expected dependency flow."
    ),
    architecture_rule!(
        "repository_logs_with_http_status_or_route_labels",
        Info,
        "Repositories that log transport metadata instead of persistence-domain details."
    ),
    architecture_rule!(
        "repository_method_returns_partially_built_scopes_for_caller_chaining",
        Info,
        "Repositories that expose scope-building fragments instead of complete query APIs."
    ),
    architecture_rule!(
        "repository_mixes_raw_sql_and_gorm_same_method_without_adapter_boundary",
        Info,
        "Repository methods that mix raw SQL and ORM clauses with no clear boundary."
    ),
    architecture_rule!(
        "repository_or_service_packages_import_docs_or_generator_annotations",
        Info,
        "Core business packages that depend on documentation-only tooling or generator-specific concerns."
    ),
    architecture_rule!(
        "repository_returns_framework_builder_to_upper_layer",
        Warning,
        "Repositories that leak `*gorm.DB`, `sqlx.NamedStmt`, query builders, or raw rows to higher layers instead of returning results or domain abstractions."
    ),
    architecture_rule!(
        "repository_returns_gorm_query_builder",
        Warning,
        "Repositories that return partially built `*gorm.DB` chains for callers to finish."
    ),
    architecture_rule!(
        "repository_returns_http_status_errors",
        Warning,
        "Repositories that construct transport-layer error objects or status-code wrappers."
    ),
    architecture_rule!(
        "repository_returns_transport_dto",
        Warning,
        "Repositories that know about response contracts instead of returning persistence or domain results."
    ),
    architecture_rule!(
        "repository_tests_depend_on_http_transport_types",
        Info,
        "Repository tests that assert HTTP DTOs or handler-layer types."
    ),
    architecture_rule!(
        "repository_type_outside_repository_package",
        Info,
        "Repository implementations declared outside `repository`, `repo`, or `store` packages when the repo already uses those roles elsewhere."
    ),
    architecture_rule!(
        "request_context_value_extraction_duplicated_across_handlers",
        Info,
        "Repeated current-user, tenant, locale, or auth extraction logic."
    ),
    architecture_rule!(
        "request_dto_contains_gorm_tags",
        Warning,
        "Request DTOs that carry persistence-only `gorm` tags."
    ),
    architecture_rule!(
        "request_dto_outside_transport_package",
        Info,
        "Request binding structs that live in repositories or service packages instead of transport-facing packages."
    ),
    architecture_rule!(
        "request_id_generation_duplicated_outside_middleware",
        Info,
        "Repeated request-id creation logic spread across handlers."
    ),
    architecture_rule!(
        "request_logging_fields_assembled_differently_across_handlers",
        Info,
        "Inconsistent request-log field sets across the same API module."
    ),
    architecture_rule!(
        "response_dto_contains_gorm_tags",
        Warning,
        "Response payload structs that carry persistence-only `gorm` tags."
    ),
    architecture_rule!(
        "response_dto_outside_transport_package",
        Info,
        "Response envelope structs that live in persistence or repository packages instead of transport or API contract packages."
    ),
    architecture_rule!(
        "response_dto_uses_omitempty_on_required_contract_fields",
        Info,
        "Response shapes that silently drop fields that should be stable parts of the API contract."
    ),
    architecture_rule!(
        "response_envelope_shaping_duplicated_across_handlers",
        Info,
        "Repeated success-response envelope formatting that should live in a renderer or transport helper."
    ),
    architecture_rule!(
        "route_groups_created_inside_handlers",
        Info,
        "Code that constructs or mutates route groups outside the startup router layer."
    ),
    architecture_rule!(
        "route_param_name_and_dto_field_name_drift_without_mapping_helper",
        Info,
        "Repeated manual renaming between route params and DTO fields that lacks a shared mapper."
    ),
    architecture_rule!(
        "route_registration_contains_business_logic",
        Info,
        "Router setup files that build queries, mutate models, or execute business actions while registering routes."
    ),
    architecture_rule!(
        "route_registration_tests_duplicate_full_bootstrap_per_file",
        Info,
        "Tests that rebuild the full application bootstrap repeatedly instead of using shared test setup."
    ),
    architecture_rule!(
        "route_setup_scattered_without_router_package",
        Info,
        "Route registration split across unrelated files without a clear `router`, `routes`, or `transport` home."
    ),
    architecture_rule!(
        "router_constructor_builds_concrete_dependencies",
        Info,
        "Router setup that creates DB clients, external clients, or repositories directly instead of receiving prebuilt dependencies."
    ),
    architecture_rule!(
        "router_setup_runs_migrations",
        Warning,
        "Gin router or handler setup code that also performs migrations."
    ),
    architecture_rule!(
        "same_dependency_wired_in_multiple_bootstrap_locations",
        Info,
        "Duplicated construction of the same service or client in several startup files."
    ),
    architecture_rule!(
        "same_domain_error_mapped_to_multiple_statuses",
        Info,
        "The same error type being converted to different HTTP status codes in the same module."
    ),
    architecture_rule!(
        "same_service_method_accepts_tx_and_begins_tx",
        Info,
        "Service methods that sometimes reuse a transaction and sometimes open one themselves."
    ),
    architecture_rule!(
        "same_struct_used_for_bind_persist_and_respond",
        Warning,
        "A single struct type reused for request binding, database persistence, and API response serialization."
    ),
    architecture_rule!(
        "savepoint_or_nested_tx_logic_scattered_without_dedicated_helper",
        Info,
        "Nested transaction or savepoint control scattered across services and repositories."
    ),
    architecture_rule!(
        "service_calls_handler_helper",
        Warning,
        "Service packages that reach upward into handler helpers for parsing, rendering, or Gin-specific utilities."
    ),
    architecture_rule!(
        "service_constructor_accepts_untyped_config_map",
        Info,
        "Services configured by `map[string]any` or similar untyped blobs."
    ),
    architecture_rule!(
        "service_constructor_instantiates_dependencies_internally",
        Info,
        "Constructors that create repositories, clients, or loggers internally instead of accepting them explicitly."
    ),
    architecture_rule!(
        "service_depends_on_gin_abort_or_context_error_response",
        Warning,
        "Services that own transport abort behavior."
    ),
    architecture_rule!(
        "service_depends_on_transport_request_type",
        Warning,
        "Service signatures that accept request DTOs declared in handler or transport packages."
    ),
    architecture_rule!(
        "service_imports_gin_directly",
        Warning,
        "Service packages that depend on `github.com/gin-gonic/gin` instead of transport-neutral inputs."
    ),
    architecture_rule!(
        "service_method_accepts_dto_and_persistence_model_together",
        Warning,
        "Signatures that mix transport DTOs and GORM models in the same service API."
    ),
    architecture_rule!(
        "service_method_accepts_gin_context",
        Warning,
        "Service methods that take `*gin.Context` instead of domain inputs plus `context.Context`."
    ),
    architecture_rule!(
        "service_method_accepts_map_string_any_input",
        Info,
        "Services that use map-shaped dynamic input instead of typed request contracts."
    ),
    architecture_rule!(
        "service_method_combines_unrelated_write_paths_via_action_string",
        Info,
        "Services that branch on action strings to implement many create, update, or delete variants in one method."
    ),
    architecture_rule!(
        "service_method_handles_pagination_or_query_parsing",
        Info,
        "Services that parse page, sort, filter, or URL values that should already be normalized at the transport boundary."
    ),
    architecture_rule!(
        "service_method_handles_request_binding_or_header_extraction",
        Info,
        "Services that extract headers, cookies, forms, or route params directly."
    ),
    architecture_rule!(
        "service_method_imports_http_status_or_abort_semantics",
        Warning,
        "Services that depend on transport-specific status constants, abort helpers, or Gin response behavior."
    ),
    architecture_rule!(
        "service_method_mutates_transport_dto_in_place",
        Info,
        "Services that rewrite handler DTOs instead of mapping to service-layer inputs."
    ),
    architecture_rule!(
        "service_method_returns_gorm_db_or_sql_rows",
        Warning,
        "Services that leak ORM or driver primitives upward instead of returning business results."
    ),
    architecture_rule!(
        "service_method_returns_http_status",
        Warning,
        "Services that return raw HTTP status codes or transport response metadata."
    ),
    architecture_rule!(
        "service_method_returns_map_string_any_output",
        Info,
        "Services that return dynamic maps instead of typed result contracts."
    ),
    architecture_rule!(
        "service_method_returns_transport_writer_or_gin_h",
        Warning,
        "Services that return `gin.H`, `http.ResponseWriter`, or response-writing helpers."
    ),
    architecture_rule!(
        "service_returns_preformatted_client_message",
        Info,
        "Services that return final client-facing message strings instead of typed domain errors."
    ),
    architecture_rule!(
        "service_write_passthrough_without_domain_validation",
        Info,
        "Write-style service methods that mostly forward to repositories without visible domain guard checks."
    ),
    architecture_rule!(
        "service_returns_tx_to_caller",
        Warning,
        "Services that hand transaction objects back to handlers or controllers."
    ),
    architecture_rule!(
        "service_struct_has_excessive_dependency_count",
        Info,
        "Services with too many unrelated dependencies, which often signals a god-service."
    ),
    architecture_rule!(
        "service_tests_import_gin",
        Info,
        "Service-level tests that depend on Gin when the service contract is supposed to be transport-neutral."
    ),
    architecture_rule!(
        "service_type_outside_service_package",
        Info,
        "Concrete use-case services declared outside `service` or `services` packages in layered web-service repos."
    ),
    architecture_rule!(
        "shared_gorm_db_state_mutated_and_reused_across_requests",
        Warning,
        "Code that mutates shared `*gorm.DB` state and reuses it as if it were immutable configuration."
    ),
    architecture_rule!(
        "shared_integration_test_setup_not_centralized_under_test_support",
        Info,
        "Repeated app, DB, or router bootstrapping that is not moved into shared test support."
    ),
    architecture_rule!(
        "shared_package_named_common_base_utils_with_mixed_exports",
        Info,
        "Generic packages that mix services, models, DTOs, validators, and queries behind one broad shared namespace."
    ),
    architecture_rule!(
        "soft_delete_filters_written_manually_in_many_queries",
        Info,
        "Repeated `deleted_at IS NULL` style filters rather than a shared repository or scope policy."
    ),
    architecture_rule!(
        "sort_or_filter_whitelist_logic_duplicated_across_handlers",
        Info,
        "Repeated client-field whitelist logic across endpoints."
    ),
    architecture_rule!(
        "sql_null_types_escape_repository_boundary",
        Info,
        "Repository methods that leak driver-specific null wrappers beyond the persistence boundary."
    ),
    architecture_rule!(
        "upstream_consumed_interface_declared_in_provider_package",
        Info,
        "Provider packages that declare an interface even though upstream packages appear to own the abstraction seam."
    ),
    architecture_rule!(
        "sql_query_constants_outside_repository_package",
        Info,
        "Raw query templates stored in handlers, services, or DTO packages instead of repository-adjacent files."
    ),
    architecture_rule!(
        "sql_query_text_asserted_in_handler_tests",
        Info,
        "SQL-shape assertions written at handler level instead of repository tests."
    ),
    architecture_rule!(
        "sql_rows_scan_logic_outside_repository",
        Info,
        "Code outside repositories that manually scans rows into structs."
    ),
    architecture_rule!(
        "success_response_contains_error_field_or_mixed_contract",
        Info,
        "Endpoints that blend success and error shapes into one ambiguous contract."
    ),
    architecture_rule!(
        "swagger_or_openapi_annotations_on_persistence_models",
        Info,
        "Persistence structs used as the public docs contract instead of transport DTOs."
    ),
    architecture_rule!(
        "table_driven_tests_mix_multiple_domains_in_one_cases_slice",
        Info,
        "Oversized case tables that mix unrelated behaviors into one monolithic test."
    ),
    architecture_rule!(
        "table_name_literals_duplicated_outside_repository",
        Info,
        "Repeated table-name strings across handlers, services, and jobs."
    ),
    architecture_rule!(
        "table_name_override_or_scope_logic_duplicated_across_models",
        Info,
        "Repeated table-name or model-scope customization across many model files without a shared convention."
    ),
    architecture_rule!(
        "test_bootstrap_package_reused_by_production_wiring",
        Info,
        "Production startup code that imports test-only bootstrap helpers."
    ),
    architecture_rule!(
        "test_fixture_builders_live_in_production_packages",
        Info,
        "Fixture and factory helpers defined in non-test production files."
    ),
    architecture_rule!(
        "test_helpers_duplicated_across_packages",
        Info,
        "Near-identical helper builders or setup code duplicated across test packages."
    ),
    architecture_rule!(
        "tests_assert_raw_json_strings_without_response_dto",
        Info,
        "Brittle raw-JSON string assertions when typed response DTOs exist."
    ),
    architecture_rule!(
        "tests_couple_to_gorm_model_for_api_contract_assertions",
        Info,
        "API tests that assert persistence model shapes instead of response contracts."
    ),
    architecture_rule!(
        "tests_stub_gin_context_instead_of_httptest_boundary",
        Info,
        "Handler tests that mock Gin internals directly instead of using `httptest` when boundary behavior matters."
    ),
    architecture_rule!(
        "tracing_span_names_duplicated_as_raw_strings",
        Info,
        "Repeated raw span-name literals across handlers and services."
    ),
    architecture_rule!(
        "transaction_error_translation_done_in_repository_and_handler",
        Info,
        "Both repository and handler layers translating the same transaction errors."
    ),
    architecture_rule!(
        "transaction_helper_outside_repository_or_uow_package",
        Info,
        "Transaction helpers buried in handlers or services instead of a repository or unit-of-work package."
    ),
    architecture_rule!(
        "transaction_object_crosses_more_than_one_layer_boundary",
        Info,
        "Transaction handles passed through multiple layers as ordinary arguments."
    ),
    architecture_rule!(
        "transport_layer_uses_untyped_string_codes_without_catalog",
        Info,
        "Response code strings that have no shared type or catalog package."
    ),
    architecture_rule!(
        "transport_metrics_emitted_from_repository_layer",
        Info,
        "Repositories that record HTTP-route or handler-oriented metrics."
    ),
    architecture_rule!(
        "transport_tests_bypass_service_interface_and_touch_repo_directly",
        Info,
        "Transport tests that skip the service seam and assert repository behavior."
    ),
    architecture_rule!(
        "unscoped_query_without_explicit_danger_naming",
        Info,
        "`Unscoped()` usage in generic repository methods without a clearly dangerous or admin-only name."
    ),
    architecture_rule!(
        "updates_with_struct_used_for_patch_without_field_intent_helper",
        Info,
        "PATCH-style repository writes that rely on struct zero-value semantics without explicit field intent."
    ),
    architecture_rule!(
        "validation_error_response_shape_inconsistent",
        Info,
        "Handlers in the same API module that emit different validation error contracts."
    ),
    architecture_rule!(
        "validation_logic_duplicated_across_handlers",
        Info,
        "Repeated required-field or enum-validation blocks across multiple handler files."
    ),
    architecture_rule!(
        "validator_depends_on_repository_directly",
        Info,
        "Reusable validator code that reaches into repositories instead of receiving precomputed facts or focused services."
    ),
    architecture_rule!(
        "validator_outside_validation_package",
        Info,
        "Reusable validators that are scattered across handlers instead of living in a dedicated validation helper package."
    ),
    architecture_rule!(
        "where_clause_templates_duplicated_across_repositories",
        Info,
        "Repeated filter templates that should be shared through scopes or query builders."
    ),
];
