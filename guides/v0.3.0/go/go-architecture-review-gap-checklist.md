# Go Architecture Review Gap Checklist For v0.3.0

Date: 2026-04-09

## Objective

- [x] Audit broadly useful Go architecture and maintainability themes against the shipped rule inventory in `rules/registry.json`.
- [x] Prioritize project-agnostic best-practice rules rather than patterns tied to a single repository, domain, or codebase review.
- [x] Avoid proposing duplicate Go rules when equivalent or near-equivalent registry coverage already exists.
- [x] Add only the remaining net-new, generally reusable rule candidates through the standard heuristics, catalog, fixtures, integration, and registry flow.

## Coverage Audit Against The Current Registry

| Best-practice theme | Status | Registry mapping |
| --- | --- | --- |
| Consumer-owned interface placement | Covered | Covered by `upstream_consumed_interface_declared_in_provider_package`, with the existing `single_impl_interface` and `passthrough_wrapper_interface` rules still covering adjacent over-abstraction cases. |
| Tool-appeasement `Noop*` types in production files | Covered | Covered by `tool_appeasement_noop_type_in_production_package`. |
| Entrypoint placement in layered service repos | Covered | Covered by `root_main_go_in_layered_service_repo` alongside the existing `main_or_cmd_mixes_cli_parsing_server_bootstrap_and_business_config` rule. |
| DB query argument type erasure via `any` | Covered | Covered by `db_query_argument_erased_to_any`, while `weak_typing`, `empty_interface_parameter_overuse`, and `sql_null_types_escape_repository_boundary` continue to cover broader nearby patterns. |
| Transaction or unit-of-work propagation | Covered | Already represented by `handler_opens_transaction`, `service_returns_tx_to_caller`, `optional_tx_nil_parameter_on_repository_api`, `transaction_object_crosses_more_than_one_layer_boundary`, and `cross_repository_write_flow_without_shared_uow_boundary`. |
| Package naming and role drift | Covered | Already represented by `service_type_outside_service_package`, `repository_type_outside_repository_package`, `shared_package_named_common_base_utils_with_mixed_exports`, and `package_name_role_drift`. |

## Net-New Rule Candidates

Only the items below should move forward as new work. Each candidate should encode a reusable Go best-practice signal rather than a one-off project convention. The covered items above should not be re-added under new IDs unless the heuristics are materially different.

### 1. `upstream_consumed_interface_declared_in_provider_package`

- [x] Flag interfaces declared inside `repository`, `repo`, `store`, `service`, or `services` packages when:
  - [x] the same package also contains the obvious concrete implementation,
  - [x] upstream packages are the visible consumers,
  - [x] and the interface mostly exists as the seam the upstream layer depends on.
- [x] Treat this as a focused ownership-and-dependency rule, not a duplicate of `single_impl_interface`.
- [x] Suppress for adapters, generated code, mocks, `_test.go`, and intentionally exported SDK-style interfaces.

### 2. `tool_appeasement_noop_type_in_production_package`

- [x] Flag concrete types such as `NoopPatientRepository`, `NoopPatientService`, or similarly named stubs in production packages when nearby comments or symbol names indicate they exist only to satisfy a tool, analyzer, or linter.
- [x] Prefer explainable evidence:
  - [x] names like `Noop`, `Dummy`, `Stub`, or `Mock` in non-test files,
  - [x] comments mentioning `lint`, `linter`, `analyzer`, `tooling`, or `satisfy`,
  - [x] methods that only return zero values or `nil` with no production call sites.
- [x] Suppress for explicit adapter packages, sample apps, intentionally shipped null-object implementations, and test-only files.

### 3. `root_main_go_in_layered_service_repo`

- [x] Flag repository-root `main.go` only when the repo already looks like a layered service application and the rule would reinforce a broadly accepted entrypoint layout.
- [x] Use conservative signals:
  - [x] visible role packages such as `internal/service`, `internal/repository`, `internal/handler`, `pkg`, or `routes`,
  - [x] more than one package beyond the entrypoint,
  - [x] no existing `cmd/<binary>/main.go` layout.
- [x] Do not fire for tiny one-binary utilities, single-package CLIs, examples, or intentionally small service repos.

### 4. `db_query_argument_erased_to_any`

- [x] Flag local variables typed as `any` or `interface{}` that are populated from a concrete nullable field and then passed directly into `database/sql`, `sqlx`, `pgx`, or GORM query execution calls.
- [x] Prefer evidence that keeps the rule explainable:
  - [x] a local `var arg any` or `var arg interface{}`,
  - [x] assignments from typed fields or dereferenced pointers,
  - [x] the same symbol appearing in `QueryRowContext`, `ExecContext`, `QueryContext`, `Raw`, `Create`, or `Updates` argument lists.
- [x] Suppress when the `any` value is part of a deliberate heterogeneous parameter list that cannot be expressed more strongly.

## Implementation Checklist

- [x] Re-run the overlap audit before coding so the new IDs remain project-agnostic and do not duplicate `single_impl_interface`, `passthrough_wrapper_interface`, `weak_typing`, `empty_interface_parameter_overuse`, or the existing transaction and package-ownership rules.
- [x] Add new rule definitions in the appropriate catalog modules:
  - [x] `src/rules/catalog/go/architecture.rs` for the provider-package interface, no-op production type, and root-main layout rules.
  - [x] `src/rules/catalog/go/architecture.rs` for the DB-argument-erasure rule after keeping it in the architecture family.
- [x] Implement the heuristics under `src/heuristics/go/architecture.rs` with repo-shape guards where needed.
- [x] Add positive and clean fixtures under `tests/fixtures/go`.
- [x] Add integration coverage under `tests/integration_scan/go`.
- [x] Update `rules/registry.json` only after the heuristics and catalog entries exist.
- [x] Add guide references once the promoted rules are shipped.

## Acceptance Checklist

- [x] Each candidate maps to one stable snake_case rule ID with no duplicate semantic overlap in `rules/registry.json`.
- [x] Each candidate represents a broadly reusable Go architecture or maintainability best practice rather than a repo-specific convention.
- [x] Each candidate has at least one positive and one clean fixture.
- [x] Each candidate is conditioned to avoid noise on tiny Go repos.
- [x] Each candidate is explainable from local evidence rather than subjective style preference alone.
- [x] Representative scan runs on layered Go service repos stay low-noise after the new rules are added.
