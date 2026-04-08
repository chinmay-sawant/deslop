# Go Architecture And Governance Rule Backlog For v0.3.0

Date: 2026-04-09

## Status

- [x] All planned `v0.3.0` implementation phases are complete as of 2026-04-09.
- [x] The shipped `v0.3.0` architecture and governance family now covers the full 210-rule backlog defined in this document.
- [x] Every checklist item below is complete and mapped into the source heuristics, catalog, fixtures, registry output, and generated docs surfaces.
- [x] The goal of this document remains to define net-new Go rules that do not simply repeat the existing performance or broad project-structure inventory already present in `rules/registry.json`.

## Objective

- [x] Add a large backlog of Go bad-practice detection rules centered on architecture, layering, boundary ownership, Gin transport design, service splitting, `models` ownership, DTO separation, error contracts, GORM/SQL governance, transaction discipline, bootstrap wiring, testing architecture, and operational consistency.
- [x] Keep this entire backlog separate from the already shipped Go performance, hot-path, request-path batching, connection churn, and coarse package-style rule packs.
- [x] Include explicit rules for splitting services into `service` or `services` packages, splitting persistence structs into `model` or `models` packages, and keeping Gin applications behind clear transport abstractions instead of allowing handlers to become the whole application.

## Hard Exclusions

- [x] Do not restate current performance, hot-path, or request-path DB rules that already exist in `v0.2.0`.
- [x] Do not restate generic repo-wide package-name or import-grouping checks already covered by `inconsistent_package_name` and `misgrouped_imports`.
- [x] Do not force one universal Go layout for small libraries, CLIs, or single-package tools.
- [x] Only fire folder or package ownership rules when the repository already looks like a layered Gin, HTTP, or GORM service with clear role packages.

## Detection Principles

- [x] Prefer explainable signals: imports, package names, file paths, route registration, struct tags, constructor signatures, method signatures, receiver types, and local call shapes.
- [x] Keep most rules package-aware and function-local first; only add repo-aware correlation when it materially reduces noise.
- [x] Treat `service`, `repository`, `model`, `models`, `dto`, `transport`, `router`, `routes`, `handler`, `handlers`, `middleware`, `api`, and `controller` as role hints, not mandatory names in every repository.
- [x] Allow legitimate exceptions for tiny repos, migrations, generated files, test code, adapters, and internal framework glue.

## Section Summary

| Section | Theme | Rules |
| --- | --- | ---: |
| 1 | Layer boundary violations | 15 |
| 2 | Package placement and ownership | 15 |
| 3 | Gin handler design | 15 |
| 4 | Gin router and middleware abstraction | 15 |
| 5 | Service layer contracts | 15 |
| 6 | DTOs, models, and mapping | 15 |
| 7 | Validation and binding governance | 15 |
| 8 | Error translation and API contracts | 15 |
| 9 | Repository and query construction discipline | 15 |
| 10 | GORM model, scope, and query governance | 15 |
| 11 | Transactions and unit-of-work | 15 |
| 12 | Config, bootstrap, and lifecycle wiring | 15 |
| 13 | Testing architecture and fixtures | 15 |
| 14 | Observability, docs, and operational governance | 15 |
| Total |  | 210 |

## Implementation Fit With The Existing Repository

- [x] Add new Go heuristic modules under `src/heuristics/go` instead of creating one oversized file.
- [x] Mirror the new families under `src/rules/catalog/go` so the guide maps cleanly to rule registration.
- [x] Keep new fixture coverage in `tests/fixtures/go` using the current positive and clean text-fixture style.
- [x] Keep new integration coverage in `tests/integration_scan/go` so new families can be exercised independently.
- [x] Reuse the existing parser summaries and import-aware evidence before introducing any heavier secondary analysis.

## 1. Layer Boundary Violations (15 rules)

Use these rules only when the repo already contains more than one clear layer such as `handler`, `service`, `repository`, `model`, or `router`.

- [x] `handler_calls_repository_directly_without_service`: flag handlers that bypass a visible service layer and call repositories or query helpers directly for business orchestration.
- [x] `handler_calls_gorm_directly_outside_repository`: flag handler packages that import `gorm.io/gorm` and build query chains directly.
- [x] `handler_calls_database_sql_directly_outside_repository`: flag handler packages that import `database/sql`, `sqlx`, or `pgx` and execute queries directly.
- [x] `service_imports_gin_directly`: flag service packages that depend on `github.com/gin-gonic/gin` instead of transport-neutral inputs.
- [x] `service_depends_on_transport_request_type`: flag service signatures that accept request DTOs declared in handler or transport packages.
- [x] `repository_depends_on_gin_or_http`: flag repository packages that import Gin, `net/http`, or transport-only response helpers.
- [x] `repository_depends_on_service_package`: flag repositories that import service packages, which inverts the expected dependency flow.
- [x] `model_package_depends_on_transport_or_gin`: flag `model` or `models` packages that import transport-facing libraries or handler DTO packages.
- [x] `middleware_contains_business_orchestration`: flag middleware that performs domain workflows instead of cross-cutting concerns such as auth, tracing, or request enrichment.
- [x] `cmd_or_main_contains_domain_rules`: flag `main` or `cmd` packages that contain business-rule branches instead of wiring and startup concerns.
- [x] `helper_or_utils_package_contains_domain_logic`: flag generic `helper`, `common`, or `utils` packages that host concrete domain decisions.
- [x] `route_registration_contains_business_logic`: flag router setup files that build queries, mutate models, or execute business actions while registering routes.
- [x] `domain_constants_declared_in_handler_package`: flag domain enums, statuses, and lifecycle constants that live in handler packages instead of domain-facing packages.
- [x] `service_calls_handler_helper`: flag service packages that reach upward into handler helpers for parsing, rendering, or Gin-specific utilities.
- [x] `repository_returns_framework_builder_to_upper_layer`: flag repositories that leak `*gorm.DB`, `sqlx.NamedStmt`, query builders, or raw rows to higher layers instead of returning results or domain abstractions.

## 2. Package Placement And Ownership (15 rules)

These rules are intentionally conditional. They should only fire when the repository already uses folder or package naming that suggests role ownership.

- [x] `service_type_outside_service_package`: flag concrete use-case services declared outside `service` or `services` packages in layered web-service repos.
- [x] `repository_type_outside_repository_package`: flag repository implementations declared outside `repository`, `repo`, or `store` packages when the repo already uses those roles elsewhere.
- [x] `gorm_model_outside_models_package`: flag GORM-backed structs with `gorm` tags declared outside `model` or `models` packages unless an explicit `entity` or `schema` package already exists.
- [x] `request_dto_outside_transport_package`: flag request binding structs that live in repositories or service packages instead of transport-facing packages.
- [x] `response_dto_outside_transport_package`: flag response envelope structs that live in persistence or repository packages instead of transport or API contract packages.
- [x] `validator_outside_validation_package`: flag reusable validators that are scattered across handlers instead of living in a dedicated validation helper package.
- [x] `mapper_outside_mapper_package_when_repo_uses_mappers`: flag mapper functions scattered across handlers and services when the repo already has a dedicated mapping package.
- [x] `route_setup_scattered_without_router_package`: flag route registration split across unrelated files without a clear `router`, `routes`, or `transport` home.
- [x] `middleware_type_outside_middleware_package`: flag shared middleware types and constructors declared in handlers or services instead of a middleware-focused package.
- [x] `sql_query_constants_outside_repository_package`: flag raw query templates stored in handlers, services, or DTO packages instead of repository-adjacent files.
- [x] `transaction_helper_outside_repository_or_uow_package`: flag transaction helpers buried in handlers or services instead of a repository or unit-of-work package.
- [x] `api_error_type_outside_transport_package`: flag API-facing error payload structs living in persistence or business packages instead of transport-boundary packages.
- [x] `shared_package_named_common_base_utils_with_mixed_exports`: flag generic packages that mix services, models, DTOs, validators, and queries behind one broad shared namespace.
- [x] `cross_layer_import_violation_by_package_name`: flag imports where package names clearly violate a layered direction such as `repository -> handler` or `model -> gin`.
- [x] `package_name_role_drift`: flag packages whose names advertise one role, such as `service` or `model`, but whose exported symbols primarily belong to another role.

## 3. Gin Handler Design (15 rules)

These rules focus on handlers becoming too large or too framework-coupled.

- [x] `gin_handler_accepts_more_than_one_body_contract`: flag handlers that try to bind or manually parse multiple body DTOs in one request path.
- [x] `gin_handler_returns_multiple_response_shapes`: flag handlers that manually build several unrelated success payload shapes instead of delegating to clearer endpoints or render helpers.
- [x] `gin_handler_binds_directly_into_model`: flag handlers that bind request bodies directly into GORM or persistence structs.
- [x] `gin_handler_returns_persistence_model_directly`: flag handlers that serialize database models directly to the client instead of mapping to response DTOs.
- [x] `gin_handler_starts_transaction_inline`: flag handlers that create or own transactions instead of delegating transactional work to services or unit-of-work helpers.
- [x] `gin_handler_runs_authorization_business_rules_inline`: flag handlers that contain complex permission matrices instead of delegating to policy or service logic.
- [x] `gin_handler_executes_raw_sql_inline`: flag handlers that execute literal SQL or repository-unaware raw queries.
- [x] `gin_handler_uses_action_param_switch_for_many_use_cases`: flag handlers that branch on `action`, `type`, or mode strings to serve many workflows from one endpoint function.
- [x] `gin_handler_mixes_html_json_and_file_responses`: flag handlers that serve fundamentally different transport modes from one function.
- [x] `gin_handler_stores_context_in_struct_field`: flag code that saves `*gin.Context` onto long-lived structs.
- [x] `gin_context_passed_beyond_request_boundary`: flag code that passes `*gin.Context` into goroutines, repositories, or long-lived service objects instead of extracting `context.Context` and request values.
- [x] `gin_handler_uses_global_singletons`: flag handlers that reach for global DB, logger, cache, or config variables instead of injected dependencies.
- [x] `gin_handler_contains_retry_or_backoff_orchestration`: flag handlers that implement retry loops or backoff decisions inline instead of delegating to service or client layers.
- [x] `gin_handler_calls_multiple_repositories_directly`: flag handlers that coordinate several repositories directly, which usually signals missing service orchestration.
- [x] `gin_handler_parses_config_or_feature_flags_inline`: flag handlers that read feature flags, tenant configuration, or rollout toggles directly instead of using injected policy/config abstractions.

## 4. Gin Router And Middleware Abstraction (15 rules)

This section is about keeping Gin itself thin and reusable.

- [x] `gin_engine_as_global_singleton`: flag package-level `*gin.Engine` singletons reused across unrelated startup paths.
- [x] `gin_route_registration_anonymous_function_overuse`: flag route registration that uses many inline anonymous handlers instead of named handler methods or functions.
- [x] `gin_route_paths_repeated_as_raw_strings`: flag repeated route path literals that should be centralized or grouped more intentionally.
- [x] `route_groups_created_inside_handlers`: flag code that constructs or mutates route groups outside the startup router layer.
- [x] `middleware_calls_repository_directly`: flag middleware that reaches into repositories instead of delegating through focused services or auth abstractions.
- [x] `middleware_starts_db_transaction`: flag middleware that opens database transactions for the whole request path by default.
- [x] `middleware_writes_business_response_payloads`: flag middleware that emits domain success payloads instead of leaving response bodies to handlers or render helpers.
- [x] `middleware_mutates_domain_model_state`: flag middleware that writes business entities or persistence models as a side effect of request plumbing.
- [x] `custom_recovery_logic_repeated_across_handlers`: flag repeated recover or panic-to-response logic inside handlers instead of centralized recovery middleware.
- [x] `auth_or_tenant_extraction_duplicated_across_handlers`: flag repeated auth, tenant, or principal extraction that should live in middleware or a shared boundary helper.
- [x] `request_id_generation_duplicated_outside_middleware`: flag repeated request-id creation logic spread across handlers.
- [x] `pagination_binding_duplicated_outside_boundary_helper`: flag repeated pagination DTO binding and defaulting logic across handler files.
- [x] `body_binding_done_in_middleware_and_handler`: flag endpoints where middleware parses the body and the handler also owns body binding.
- [x] `response_envelope_shaping_duplicated_across_handlers`: flag repeated success-response envelope formatting that should live in a renderer or transport helper.
- [x] `router_constructor_builds_concrete_dependencies`: flag router setup that creates DB clients, external clients, or repositories directly instead of receiving prebuilt dependencies.

## 5. Service Layer Contracts (15 rules)

These rules define what the service layer should not leak or own.

- [x] `service_method_accepts_gin_context`: flag service methods that take `*gin.Context` instead of domain inputs plus `context.Context`.
- [x] `service_method_returns_http_status`: flag services that return raw HTTP status codes or transport response metadata.
- [x] `service_method_returns_transport_writer_or_gin_h`: flag services that return `gin.H`, `http.ResponseWriter`, or response-writing helpers.
- [x] `service_method_returns_gorm_db_or_sql_rows`: flag services that leak ORM or driver primitives upward instead of returning business results.
- [x] `service_constructor_instantiates_dependencies_internally`: flag constructors that create repositories, clients, or loggers internally instead of accepting them explicitly.
- [x] `service_struct_has_excessive_dependency_count`: flag services with too many unrelated dependencies, which often signals a god-service.
- [x] `service_method_accepts_map_string_any_input`: flag services that use map-shaped dynamic input instead of typed request contracts.
- [x] `service_method_returns_map_string_any_output`: flag services that return dynamic maps instead of typed result contracts.
- [x] `service_method_handles_pagination_or_query_parsing`: flag services that parse page, sort, filter, or URL values that should already be normalized at the transport boundary.
- [x] `service_method_handles_request_binding_or_header_extraction`: flag services that extract headers, cookies, forms, or route params directly.
- [x] `service_method_combines_unrelated_write_paths_via_action_string`: flag services that branch on action strings to implement many create, update, or delete variants in one method.
- [x] `service_method_mutates_transport_dto_in_place`: flag services that rewrite handler DTOs instead of mapping to service-layer inputs.
- [x] `service_method_accepts_dto_and_persistence_model_together`: flag signatures that mix transport DTOs and GORM models in the same service API.
- [x] `service_method_imports_http_status_or_abort_semantics`: flag services that depend on transport-specific status constants, abort helpers, or Gin response behavior.
- [x] `generic_process_execute_handle_service_name_without_domain_noun`: flag generic service names and methods that hide business intent instead of naming the actual use case.

## 6. DTOs, Models, And Mapping (15 rules)

This section carries the specific split you requested: keep persistence structs in `model` or `models`, and keep transport contracts separate.

- [x] `same_struct_used_for_bind_persist_and_respond`: flag a single struct type reused for request binding, database persistence, and API response serialization.
- [x] `request_dto_contains_gorm_tags`: flag request DTOs that carry persistence-only `gorm` tags.
- [x] `response_dto_contains_gorm_tags`: flag response payload structs that carry persistence-only `gorm` tags.
- [x] `gorm_model_contains_binding_or_validate_tags`: flag GORM models that accumulate transport validation and binding concerns.
- [x] `domain_entity_contains_json_tags_without_boundary_exception`: flag core domain entities that are directly annotated for API serialization without a deliberate boundary exception.
- [x] `persistence_model_contains_http_form_or_uri_tags`: flag persistence models that are tied to form or URI binding tags.
- [x] `nullable_sql_types_leak_into_api_dto`: flag response or request DTOs that expose `sql.Null*` types instead of boundary-friendly contract types.
- [x] `dto_to_model_mapping_in_handler`: flag handlers that own transport-to-model mapping instead of dedicated mappers or services.
- [x] `model_to_dto_mapping_in_handler`: flag handlers that manually shape every response from persistence structs rather than using dedicated mapping helpers.
- [x] `mapping_logic_duplicated_across_handlers`: flag repeated field mapping blocks across multiple handlers.
- [x] `patch_dto_uses_non_pointer_fields_for_optional_updates`: flag PATCH-like DTOs that cannot distinguish omitted fields from zero values.
- [x] `create_and_update_share_same_dto_despite_conflicting_requiredness`: flag a single DTO reused for create and update when validation rules clearly differ.
- [x] `response_dto_uses_omitempty_on_required_contract_fields`: flag response shapes that silently drop fields that should be stable parts of the API contract.
- [x] `gorm_model_contains_calculated_response_fields`: flag persistence structs that carry API-only derived presentation fields.
- [x] `giant_model_struct_spans_multiple_subdomains`: flag oversized model structs that appear to merge multiple bounded contexts into one persistence shape.

## 7. Validation And Binding Governance (15 rules)

Keep transport validation consistent and close to the boundary.

- [x] `validation_logic_duplicated_across_handlers`: flag repeated required-field or enum-validation blocks across multiple handler files.
- [x] `manual_required_checks_after_validate_tags_available`: flag handlers that re-check required fields manually even though struct-tag validation already exists.
- [x] `validation_error_response_shape_inconsistent`: flag handlers in the same API module that emit different validation error contracts.
- [x] `custom_validator_registration_inside_handler`: flag per-request validator registration instead of startup-time validator wiring.
- [x] `default_value_injection_scattered_across_handlers`: flag repeated defaulting logic for pagination, filters, or booleans across handlers.
- [x] `path_param_parsing_duplicated_across_handlers`: flag repeated `strconv` or UUID path-param parsing logic that should live in helpers.
- [x] `pagination_validation_missing_shared_bound_helper`: flag API modules that hand-roll page and page-size bounds in many handlers instead of a shared contract.
- [x] `sort_or_filter_whitelist_logic_duplicated_across_handlers`: flag repeated client-field whitelist logic across endpoints.
- [x] `multiple_bind_sources_into_same_struct_without_precedence_contract`: flag one DTO being filled from body, query, and path without a clear precedence policy.
- [x] `query_path_and_body_merge_performed_in_handler`: flag handlers that merge several request sources inline instead of normalizing inputs before service calls.
- [x] `business_validation_mixed_with_persistence_calls_in_handler`: flag handlers that start querying or writing before finishing request validation.
- [x] `validator_depends_on_repository_directly`: flag reusable validator code that reaches into repositories instead of receiving precomputed facts or focused services.
- [x] `request_context_value_extraction_duplicated_across_handlers`: flag repeated current-user, tenant, locale, or auth extraction logic.
- [x] `file_upload_validation_mixed_with_storage_write`: flag functions that validate upload metadata and also persist the upload in the same boundary step.
- [x] `route_param_name_and_dto_field_name_drift_without_mapping_helper`: flag repeated manual renaming between route params and DTO fields that lacks a shared mapper.

## 8. Error Translation And API Contracts (15 rules)

These rules focus on consistent and explainable error behavior.

- [x] `repository_returns_http_status_errors`: flag repositories that construct transport-layer error objects or status-code wrappers.
- [x] `service_returns_preformatted_client_message`: flag services that return final client-facing message strings instead of typed domain errors.
- [x] `domain_errors_declared_in_handler_package`: flag business error types declared only in handlers instead of a reusable domain or service boundary.
- [x] `inline_error_to_status_mapping_duplicated`: flag repeated error-to-status translation switches across handlers.
- [x] `multiple_error_envelope_shapes_same_api_module`: flag endpoints in the same API surface that serialize unrelated error payload structures.
- [x] `handler_switches_on_error_strings`: flag handlers that branch on `err.Error()` text rather than typed errors.
- [x] `middleware_and_handler_translate_same_error_domain`: flag double translation layers where middleware and handlers both map the same error family to responses.
- [x] `raw_db_error_exposed_to_client`: flag direct serialization of SQL or GORM error text into API responses.
- [x] `same_domain_error_mapped_to_multiple_statuses`: flag the same error type being converted to different HTTP status codes in the same module.
- [x] `handler_and_service_both_log_same_error_chain`: flag duplicated logging ownership where both service and handler log the same failure.
- [x] `not_found_semantics_implemented_by_nil_nil_return`: flag repository or service APIs that encode missing records as `nil, nil` with no explicit result contract.
- [x] `service_depends_on_gin_abort_or_context_error_response`: flag services that own transport abort behavior.
- [x] `error_code_literals_duplicated_across_handlers`: flag repeated string error codes without a centralized catalog.
- [x] `transport_layer_uses_untyped_string_codes_without_catalog`: flag response code strings that have no shared type or catalog package.
- [x] `success_response_contains_error_field_or_mixed_contract`: flag endpoints that blend success and error shapes into one ambiguous contract.

## 9. Repository And Query Construction Discipline (15 rules)

This section intentionally covers SQL and GORM bad practices from an architecture angle rather than repeating existing request-path performance rules.

- [x] `raw_sql_literal_in_handler`: flag literal SQL strings declared and executed from handlers.
- [x] `raw_sql_literal_in_service`: flag literal SQL strings executed from services instead of repositories or query packages.
- [x] `gorm_chain_built_in_handler`: flag handlers that construct `Where`, `Joins`, `Scopes`, or `Order` chains directly.
- [x] `gorm_chain_built_in_service`: flag services that own detailed ORM query shape instead of delegating to repositories.
- [x] `repository_accepts_gin_context`: flag repositories that accept `*gin.Context` rather than `context.Context`.
- [x] `repository_accepts_http_request_dto`: flag repositories that accept transport DTOs instead of repository inputs or domain values.
- [x] `repository_returns_transport_dto`: flag repositories that know about response contracts instead of returning persistence or domain results.
- [x] `repository_returns_gorm_query_builder`: flag repositories that return partially built `*gorm.DB` chains for callers to finish.
- [x] `table_name_literals_duplicated_outside_repository`: flag repeated table-name strings across handlers, services, and jobs.
- [x] `column_name_literals_duplicated_outside_repository`: flag repeated column-name strings outside repository or query packages.
- [x] `order_by_clause_literals_scattered_across_layers`: flag raw sort expressions duplicated in handlers and services instead of query helpers.
- [x] `where_clause_templates_duplicated_across_repositories`: flag repeated filter templates that should be shared through scopes or query builders.
- [x] `repository_mixes_raw_sql_and_gorm_same_method_without_adapter_boundary`: flag repository methods that mix raw SQL and ORM clauses with no clear boundary.
- [x] `sql_rows_scan_logic_outside_repository`: flag code outside repositories that manually scans rows into structs.
- [x] `sql_null_types_escape_repository_boundary`: flag repository methods that leak driver-specific null wrappers beyond the persistence boundary.

## 10. GORM Model, Scope, And Query Governance (15 rules)

These rules target maintainability, ownership, and SQL/GORM bottleneck patterns that often become chronic problems in Gin applications.

- [x] `gorm_scopes_defined_inline_repeatedly`: flag repeated ad hoc scope functions instead of shared named scopes.
- [x] `preload_rules_scattered_across_layers`: flag preload decisions spread across handlers, services, and repositories instead of one query owner.
- [x] `soft_delete_filters_written_manually_in_many_queries`: flag repeated `deleted_at IS NULL` style filters rather than a shared repository or scope policy.
- [x] `unscoped_query_without_explicit_danger_naming`: flag `Unscoped()` usage in generic repository methods without a clearly dangerous or admin-only name.
- [x] `gorm_model_hook_contains_external_io`: flag hooks that call HTTP clients, queues, file I/O, or external side effects.
- [x] `gorm_model_hook_calls_service_or_repository`: flag model hooks that depend on higher-layer services or repositories.
- [x] `gorm_hook_mutates_unrelated_tables`: flag hooks that reach beyond the owning aggregate and write unrelated records.
- [x] `repository_method_returns_partially_built_scopes_for_caller_chaining`: flag repositories that expose scope-building fragments instead of complete query APIs.
- [x] `gorm_session_options_configured_outside_repository`: flag code above the repository layer that changes `Session`, `Clauses`, or transaction options directly.
- [x] `gorm_locking_clauses_built_outside_repository`: flag `FOR UPDATE` or similar locking behavior built in handlers or services instead of repository-owned methods.
- [x] `updates_with_struct_used_for_patch_without_field_intent_helper`: flag PATCH-style repository writes that rely on struct zero-value semantics without explicit field intent.
- [x] `map_based_updates_passed_from_handler_to_repository`: flag update maps built from request payloads and passed through layers without typed ownership.
- [x] `generic_base_repository_with_reflection_dispatch`: flag catch-all repositories that depend on reflection-heavy generic CRUD instead of bounded repository APIs.
- [x] `shared_gorm_db_state_mutated_and_reused_across_requests`: flag code that mutates shared `*gorm.DB` state and reuses it as if it were immutable configuration.
- [x] `table_name_override_or_scope_logic_duplicated_across_models`: flag repeated table-name or model-scope customization across many model files without a shared convention.

## 11. Transactions And Unit-Of-Work (15 rules)

These rules emphasize ownership, commit timing, and cross-layer consistency.

- [x] `handler_opens_transaction`: flag handlers that call `Begin`, `Transaction`, or unit-of-work start helpers directly.
- [x] `middleware_opens_transaction`: flag middleware that owns transaction lifetime for all downstream business logic.
- [x] `service_returns_tx_to_caller`: flag services that hand transaction objects back to handlers or controllers.
- [x] `repository_begins_transaction_without_uow_or_callback`: flag repositories that quietly start transactions inside generic CRUD methods.
- [x] `transaction_object_crosses_more_than_one_layer_boundary`: flag transaction handles passed through multiple layers as ordinary arguments.
- [x] `same_service_method_accepts_tx_and_begins_tx`: flag service methods that sometimes reuse a transaction and sometimes open one themselves.
- [x] `optional_tx_nil_parameter_on_repository_api`: flag repository APIs that use optional `tx *gorm.DB` parameters instead of explicit unit-of-work contracts.
- [x] `cross_repository_write_flow_without_shared_uow_boundary`: flag service methods coordinating several repositories with no explicit shared transaction or consistency boundary.
- [x] `commit_or_rollback_split_across_functions_without_owner`: flag transaction end-state logic spread across helper functions with unclear ownership.
- [x] `external_http_call_inside_transaction_scope`: flag network calls performed while a DB transaction is open.
- [x] `event_publish_before_transaction_commit`: flag event or message publication before durable commit is confirmed.
- [x] `cache_invalidation_before_transaction_commit`: flag cache mutation or eviction that happens before a write transaction commits.
- [x] `background_goroutine_started_inside_transaction_scope`: flag goroutines launched while relying on open transaction state.
- [x] `transaction_error_translation_done_in_repository_and_handler`: flag both repository and handler layers translating the same transaction errors.
- [x] `savepoint_or_nested_tx_logic_scattered_without_dedicated_helper`: flag nested transaction or savepoint control scattered across services and repositories.

## 12. Config, Bootstrap, And Lifecycle Wiring (15 rules)

The goal here is to keep startup explicit and request paths clean.

- [x] `constructor_reads_env_directly`: flag constructors that call `os.Getenv` instead of receiving normalized config.
- [x] `router_setup_runs_migrations`: flag Gin router or handler setup code that also performs migrations.
- [x] `bootstrap_builds_clients_inside_route_registration`: flag startup code that hides dependency construction inside route registration functions.
- [x] `init_registers_routes_or_dependencies`: flag `init()` functions that register handlers, routes, or runtime dependencies.
- [x] `package_level_mutable_config_used_by_handlers_services`: flag mutable package globals used as live config in request paths.
- [x] `service_constructor_accepts_untyped_config_map`: flag services configured by `map[string]any` or similar untyped blobs.
- [x] `repository_constructor_accepts_gin_engine_or_router`: flag repository constructors that take transport objects.
- [x] `middleware_uses_global_logger_or_config_singleton`: flag middleware that depends on mutable globals instead of injected config and logger instances.
- [x] `same_dependency_wired_in_multiple_bootstrap_locations`: flag duplicated construction of the same service or client in several startup files.
- [x] `feature_flag_lookup_without_config_abstraction`: flag handlers or services that query feature flags directly without a focused flag interface.
- [x] `background_worker_started_from_http_handler_registration`: flag route or middleware setup that also starts unrelated background jobs.
- [x] `main_or_cmd_mixes_cli_parsing_server_bootstrap_and_business_config`: flag entrypoints that do too much without startup composition helpers.
- [x] `application_lifecycle_missing_shutdown_owner`: flag shared resources with visible startup paths but no obvious shutdown owner or lifecycle wrapper.
- [x] `migration_runner_and_api_server_bootstrap_coupled`: flag one startup path that always runs migrations and serves traffic together.
- [x] `test_bootstrap_package_reused_by_production_wiring`: flag production startup code that imports test-only bootstrap helpers.

## 13. Testing Architecture And Fixtures (15 rules)

These rules stay Go-specific without duplicating the existing generic test-quality checks.

- [x] `service_tests_import_gin`: flag service-level tests that depend on Gin when the service contract is supposed to be transport-neutral.
- [x] `repository_tests_depend_on_http_transport_types`: flag repository tests that assert HTTP DTOs or handler-layer types.
- [x] `handler_tests_use_real_database_without_seam`: flag handler tests that hit real DB setup even though repository or service seams exist.
- [x] `test_helpers_duplicated_across_packages`: flag near-identical helper builders or setup code duplicated across test packages.
- [x] `mock_repository_types_duplicated_across_tests`: flag many copy-pasted mock repository structs with the same method sets.
- [x] `test_fixture_builders_live_in_production_packages`: flag fixture and factory helpers defined in non-test production files.
- [x] `route_registration_tests_duplicate_full_bootstrap_per_file`: flag tests that rebuild the full application bootstrap repeatedly instead of using shared test setup.
- [x] `tests_assert_raw_json_strings_without_response_dto`: flag brittle raw-JSON string assertions when typed response DTOs exist.
- [x] `tests_couple_to_gorm_model_for_api_contract_assertions`: flag API tests that assert persistence model shapes instead of response contracts.
- [x] `tests_stub_gin_context_instead_of_httptest_boundary`: flag handler tests that mock Gin internals directly instead of using `httptest` when boundary behavior matters.
- [x] `transport_tests_bypass_service_interface_and_touch_repo_directly`: flag transport tests that skip the service seam and assert repository behavior.
- [x] `sql_query_text_asserted_in_handler_tests`: flag SQL-shape assertions written at handler level instead of repository tests.
- [x] `migration_tests_live_under_handler_packages`: flag migration-specific tests located in transport packages.
- [x] `table_driven_tests_mix_multiple_domains_in_one_cases_slice`: flag oversized case tables that mix unrelated behaviors into one monolithic test.
- [x] `shared_integration_test_setup_not_centralized_under_test_support`: flag repeated app, DB, or router bootstrapping that is not moved into shared test support.

## 14. Observability, Docs, And Operational Governance (15 rules)

These rules cover the consistency work that often gets ignored until a Gin service becomes difficult to operate.

- [x] `metrics_labels_built_inline_in_handlers`: flag handlers that hand-build metric labels repeatedly instead of using focused observation helpers.
- [x] `tracing_span_names_duplicated_as_raw_strings`: flag repeated raw span-name literals across handlers and services.
- [x] `transport_metrics_emitted_from_repository_layer`: flag repositories that record HTTP-route or handler-oriented metrics.
- [x] `repository_logs_with_http_status_or_route_labels`: flag repositories that log transport metadata instead of persistence-domain details.
- [x] `audit_logging_executed_in_handler_before_service_success`: flag audit writes that happen before the service has confirmed success.
- [x] `request_logging_fields_assembled_differently_across_handlers`: flag inconsistent request-log field sets across the same API module.
- [x] `domain_identifiers_logged_under_inconsistent_field_keys`: flag the same entity ID being logged under many different keys across packages.
- [x] `health_or_readiness_handlers_reach_into_business_repositories_directly`: flag health endpoints that depend on full business repositories instead of focused probes.
- [x] `admin_or_debug_endpoint_registration_mixed_into_public_router_setup`: flag operational endpoints registered alongside public routes with no clear boundary.
- [x] `migration_or_seed_logic_callable_from_request_handlers`: flag request handlers that can trigger migration or seed workflows.
- [x] `background_jobs_registered_from_gin_packages_instead_of_bootstrap`: flag scheduler or worker registration hidden inside Gin transport packages.
- [x] `operational_command_handlers_reuse_http_services_without_adapter`: flag CLI or cron entrypoints that import Gin-only service contracts instead of neutral application services.
- [x] `swagger_or_openapi_annotations_on_persistence_models`: flag persistence structs used as the public docs contract instead of transport DTOs.
- [x] `api_examples_embedded_in_handlers_instead_of_transport_docs_helpers`: flag long example payload blocks hardcoded in handlers rather than doc or example helpers.
- [x] `repository_or_service_packages_import_docs_or_generator_annotations`: flag core business packages that depend on documentation-only tooling or generator-specific concerns.

## Acceptance Checklist For Future Implementation

- [x] Every promoted rule should map cleanly to a stable snake_case rule ID.
- [x] Every promoted rule should have at least one positive and one clean Go fixture under `tests/fixtures/go`.
- [x] Every promoted rule family should have integration coverage under `tests/integration_scan/go`.
- [x] Folder and package placement rules should remain conditional and should not punish small or intentionally flat repos.
- [x] Gin and GORM rules should prefer layered ownership and contract clarity over cosmetic stylistic enforcement.
- [x] SQL and GORM governance rules should remain distinct from the already shipped performance-oriented DB rule families.
