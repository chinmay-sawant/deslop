# Go Future Repo Prevention Plan Checklist

Date: 2026-04-09

## Objective

- [x] Audit the future-repo prevention checklist against the live Go inventory in `rules/registry.json`.
- [x] Respect the current application structure instead of adding duplicate rule families or one-off repo conventions.
- [x] Keep only the remaining net-new prevention signals that were not already covered by the shipped Go architecture, context, security, idiom, and test-quality rules.

## Existing Registry Coverage Reused Instead Of Duplicated

| Checklist theme | Status | Existing registry coverage |
| --- | --- | --- |
| Layer boundaries, thin handlers, service ownership, repository ownership | [x] Covered | Already covered by the large `architecture` pack, including `handler_calls_repository_directly_without_service`, `gin_handler_executes_raw_sql_inline`, `service_method_accepts_gin_context`, and `repository_returns_framework_builder_to_upper_layer`. |
| Startup, bootstrap, graceful shutdown, middleware registration, worker ownership | [x] Covered | Already covered by `root_main_go_in_layered_service_repo`, `main_or_cmd_mixes_cli_parsing_server_bootstrap_and_business_config`, `background_jobs_registered_from_gin_packages_instead_of_bootstrap`, and `application_lifecycle_missing_shutdown_owner`. |
| Config governance and secret handling | [x] Covered | Already covered by `constructor_reads_env_directly`, `feature_flag_lookup_without_config_abstraction`, `package_level_mutable_config_used_by_handlers_services`, and the shared `hardcoded_secret` rule. |
| API semantics, validation, and client-error handling | [x] Covered | Already covered by `client_input_error_mapped_to_internal_server_error`, `validation_logic_duplicated_across_handlers`, `pagination_validation_missing_shared_bound_helper`, and related transport rules. |
| Service passthrough and interface/tooling slop | [x] Covered | Already covered by `service_write_passthrough_without_domain_validation`, `single_impl_interface`, `passthrough_wrapper_interface`, and `tool_appeasement_noop_type_in_production_package`. |
| DTO, model, and mapping separation | [x] Covered | Already covered by `same_struct_used_for_bind_persist_and_respond`, `request_dto_contains_gorm_tags`, `domain_entity_contains_json_tags_without_boundary_exception`, and related mapping rules. |
| Error contracts and not-found semantics | [x] Covered | Already covered by `repository_returns_http_status_errors`, `handler_switches_on_error_strings`, `same_domain_error_mapped_to_multiple_statuses`, and `not_found_semantics_implemented_by_nil_nil_return`. |
| Transaction and persistence discipline | [x] Covered | Already covered by `cross_repository_write_flow_without_shared_uow_boundary`, `event_publish_before_transaction_commit`, `cache_invalidation_before_transaction_commit`, and `gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary`. |
| Security placeholders and unsafe transport behavior | [x] Covered | Already covered by the existing Go `security` inventory plus architecture rules such as `migration_or_seed_logic_callable_from_request_handlers`. |
| Test signal quality and mock sprawl | [x] Covered | Already covered by `happy_path_only_test`, `placeholder_test_body`, `test_without_assertion_signal`, `mock_repository_types_duplicated_across_tests`, and the existing handler-test architecture rules. |

## Net-New Prevention Gaps Implemented

- [x] Add `repository_single_record_write_without_rows_affected_check` so single-record repository updates and deletes do not silently ignore meaningful write outcomes.
- [x] Add `placeholder_seed_function_in_production` so TODO-shaped seed entrypoints are flagged before they get mistaken for real repo bootstrap support.
- [x] Add `readme_migration_strategy_claim_conflicts_with_startup_code` so README migration claims are checked against visible `AutoMigrate` startup behavior.
- [x] Add `readme_claims_seeding_but_seed_entrypoint_is_placeholder` so README seeding guidance does not drift ahead of the actual seed implementation.

## Implementation

- [x] Extend the existing Go architecture catalog in `src/rules/catalog/go/architecture.rs` instead of creating a duplicate family.
- [x] Implement the new logic inside the existing split Go architecture heuristics under `src/heuristics/go/architecture/`.
- [x] Reuse the current repo-shape, startup, and seed-entrypoint helpers so the new rules stay explainable and low-noise.
- [x] Add focused positive and clean fixtures under `tests/fixtures/go/architecture/`.
- [x] Add targeted integration coverage under `tests/integration_scan/go/architecture.rs`.

## Verification

- [x] Regenerate `rules/registry.json` from the Rust-backed catalog.
- [x] Regenerate the synced docs surfaces, including `README.md`, `frontend/src/features/docs/docs-content.ts`, and `guides/features-and-detections.md`.
- [x] Update the inventory-count guards in `src/rules.rs` and `guides/v0.2.0/inventory-regression-guards.md`.
- [x] Re-run targeted Go architecture integration coverage after the new rules were added.
- [x] End with the future-repo prevention plan represented by the existing 217-rule architecture base plus these 4 new prevention checks, for 221 Go architecture rules and 637 Go rules overall.
