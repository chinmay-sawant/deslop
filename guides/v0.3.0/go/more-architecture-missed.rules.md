# Go Architecture And Performance Gap Audit For v0.3.0

Date: 2026-04-09

## Objective

- Keep the `guides/v0.3.0/go` material fully repo-agnostic.
- Translate the earlier repository review themes into reusable Go rule inventory work.
- Re-check every theme against the live `rules/registry.json` before adding new rule IDs.
- Add only the remaining net-new Go rule coverage plus a large performance expansion that does not duplicate the current registry.

## Coverage Audit Against The Current Registry

| Review theme | Status | Registry mapping |
| --- | --- | --- |
| Request context must not be discarded on request paths | Partially covered | `context_background_used`, `missing_context`, `missing_context_propagation`, `gin_context_passed_beyond_request_boundary`, `service_method_accepts_gin_context`, and `repository_accepts_gin_context` already cover adjacent misuse, but request handlers with an implicit request context still needed stronger direct coverage. |
| Duplicate startup or composition roots drift over time | Covered | `same_dependency_wired_in_multiple_bootstrap_locations`, `main_or_cmd_mixes_cli_parsing_server_bootstrap_and_business_config`, `migration_runner_and_api_server_bootstrap_coupled`, `router_constructor_builds_concrete_dependencies`, and `application_lifecycle_missing_shutdown_owner`. |
| Router and middleware ownership split across layers | Covered | `route_setup_scattered_without_router_package`, `route_registration_contains_business_logic`, `bootstrap_builds_clients_inside_route_registration`, `background_jobs_registered_from_gin_packages_instead_of_bootstrap`, `middleware_contains_business_orchestration`, and `middleware_calls_repository_directly`. |
| Ad hoc auth checks or hardcoded transport-side security | Covered | `hardcoded_secret`, `auth_or_tenant_extraction_duplicated_across_handlers`, `gin_handler_runs_authorization_business_rules_inline`, `http_handler_without_csrf_protection`, and `timing_attack_on_token_comparison`. |
| Client input failures incorrectly mapped to 500 responses | Gap | Existing error-contract rules cover duplication and drift, but there was no focused rule for parse or binding errors translated to `500` on request handlers. |
| Domain and service contracts drift across packages | Mostly covered | `service_depends_on_transport_request_type`, `upstream_consumed_interface_declared_in_provider_package`, `same_struct_used_for_bind_persist_and_respond`, `request_dto_outside_transport_package`, and `response_dto_outside_transport_package`. |
| Repository layer mixes transport concerns | Covered | `repository_depends_on_gin_or_http`, `repository_returns_http_status_errors`, `repository_logs_with_http_status_or_route_labels`, and `transport_metrics_emitted_from_repository_layer`. |
| GORM bootstrap mixed with raw SQL repositories | Covered | `gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary`. |
| Cache APIs and implementations ignore caller context | Gap | The registry already catches broad context misuse, but cache-specific APIs and cache methods still needed explicit Go coverage. |
| `pkg` or `utils` becomes a junk drawer | Covered | `helper_or_utils_package_contains_domain_logic` and `shared_package_named_common_base_utils_with_mixed_exports`. |
| Tests look present but carry weak signal | Covered | `happy_path_only_test`, `placeholder_test_body`, `test_without_assertion_signal`, plus the Go test-architecture rules already in the architecture family. |
| Comments or names drift from real behavior | Partially covered | Comment and naming rules already exist, but behavior-vs-comment drift remains better handled as review guidance than a generic static rule unless local evidence becomes much stronger. |

## Net-New Work To Implement

### 1. Extend `context_background_used` For Request Handlers

- Broaden the existing Go context rule so it also covers request handlers that already have an implicit request-scoped context available through Gin or `net/http`, but still start fresh from `context.Background()` or `context.TODO()`.
- Keep the rule conservative:
  - only fire on request-style handlers,
  - only fire when the handler visibly creates a fresh background context,
  - and keep detached background-worker flows out of scope.

### 2. Add `client_input_error_mapped_to_internal_server_error`

- Flag request handlers when:
  - obvious parse or binding work happens first, such as `strconv`, UUID parsing, JSON bind, or similar boundary parsing,
  - the immediate error branch responds with `500` or `http.StatusInternalServerError`,
  - and the shape looks like a client input problem rather than an unexpected server failure.
- Keep this distinct from duplication rules such as `inline_error_to_status_mapping_duplicated`.

### 3. Add `cache_interface_method_missing_context`

- Flag cache-oriented interfaces when:
  - the package path or interface name clearly looks cache-related,
  - methods such as `Get`, `Set`, `Delete`, `Load`, `Store`, `Fetch`, or `Remember` omit `context.Context`,
  - and the interface shape otherwise suggests outbound IO or remote cache access.
- Suppress for tiny in-memory only helpers that are visibly not doing request-scoped IO.

### 4. Add `cache_method_uses_context_background`

- Flag cache-oriented implementation methods when:
  - the method body calls `context.Background()` or `context.TODO()`,
  - the surrounding file or receiver clearly represents cache behavior,
  - and the body also shows cache or remote-store interaction.

## Final Implemented Architecture And Context Coverage

- Enhanced `context_background_used` so it also fires when Gin and `net/http` request handlers already have an implicit caller-owned request context available but still create a fresh background context.
- Added `client_input_error_mapped_to_internal_server_error` for parse, bind, and other client-input failures that are immediately returned as `500`.
- Added `cache_interface_method_missing_context` for cache-shaped interfaces whose IO-facing methods omit `context.Context`.
- Added `cache_method_uses_context_background` for cache implementations that create fresh background contexts instead of propagating the caller context.

## Go Performance Expansion

This change should also add a new repo-agnostic Go performance pack with 100 net-new rules that do not duplicate the current Go `performance`, `hot_path`, `data_access`, `gin`, `idioms`, `library`, `context`, or `concurrency` inventories already present in `rules/registry.json`.

### Final Category Breakdown

| Category | Implemented rules |
| --- | ---: |
| Text, `strings`, `bytes`, and reader/writer conversions | 40 |
| `fmt` and `strconv` conversion shortcuts | 15 |
| Path and prefix/suffix helper upgrades | 11 |
| Timers, `rand`, runtime, and loop allocation patterns | 17 |
| Encoding, hashing, and parse-path repeated work | 17 |
| Total | 100 |

### Implementation Notes

- The text and conversion bucket covers direct `strings`, `bytes`, builder, buffer, reader, splitter, and case-folding shortcut misuse.
- The `fmt` and `strconv` bucket focuses on expensive formatting paths that only need a direct conversion or a builder or buffer write.
- The path and prefix or suffix bucket focuses on split helpers and manual slice checks that should use the stdlib helpers directly.
- The timers, `rand`, runtime, and loop-allocation bucket targets repeatable helper construction or global runtime knobs on hot or request paths.
- The encoding and hashing bucket targets repeated validation, marshal, encode, decode, checksum, and digest work inside loops or other high-frequency flows.

### Guardrails

- Keep the new performance pack conservative and explainable.
- Prefer direct local evidence:
  - exact stdlib call shapes,
  - narrow conversion patterns,
  - loop-local expensive calls,
  - request-path construction of repeatable helpers.
- Avoid duplicating existing rules such as:
  - `sprintf_for_simple_int_to_string`
  - `strings_hasprefix_then_trimprefix`
  - `strings_hassuffix_then_trimsuffix`
  - `strings_contains_vs_index`
  - `strings_replace_all_for_single_char`
  - `sync_pool_ignored_for_frequent_small_allocs`
  - `time_after_in_loop`
  - the current Gin, hot-path, and data-access request-path performance packs

## Implementation Surfaces

- `src/heuristics/go/context.rs`
- `src/heuristics/go/architecture.rs`
- `src/heuristics/go/architecture/function_rules_transport.rs`
- `src/heuristics/go/architecture/file_rules.rs`
- `src/heuristics/go/mod.rs`
- `src/heuristics/registry.rs`
- `src/rules/catalog/go/context.rs`
- `src/rules/catalog/go/architecture.rs`
- `src/rules/catalog/go/mod.rs`
- `src/rules/catalog/bindings.rs`
- new Go performance heuristic and catalog modules for the additional 100-rule pack
- `tests/fixtures/go`
- `tests/integration_scan/context.rs`
- `tests/integration_scan/go/architecture.rs`
- `tests/integration_scan/performance.rs`
- `rules/registry.json`

## Acceptance Criteria

- Every `v0.3.0` Go guide remains repo-agnostic.
- Every new rule is justified by overlap analysis against the live registry.
- The cache and client-error rules use generic Go signals rather than repository naming conventions alone.
- The performance expansion adds 100 new Go rules without reusing existing rule IDs or semantics.
- New rules are wired through heuristics, catalog metadata, fixtures, integration tests, and the generated registry output.
