# Go Architecture Review Gap Checklist For v0.3.0

Date: 2026-04-09

## Objective

- [x] Audit broadly useful Go architecture and maintainability themes against the shipped rule inventory in `rules/registry.json`.
- [x] Prioritize project-agnostic best-practice rules rather than patterns tied to a single repository, domain, or codebase review.
- [x] Avoid proposing duplicate Go rules when equivalent or near-equivalent registry coverage already exists.
- [ ] Add only the remaining net-new, generally reusable rule candidates through the standard heuristics, catalog, fixtures, integration, and registry flow.

## Coverage Audit Against The Current Registry

| Best-practice theme | Status | Registry mapping |
| --- | --- | --- |
| Consumer-owned interface placement | Partial | `single_impl_interface` and `passthrough_wrapper_interface` cover some over-abstraction and one-impl interface cases, but there is no explicit rule for consumer-owned interfaces being declared in concrete provider packages. |
| Tool-appeasement `Noop*` types in production files | Missing | No current Go or common rule explicitly targets tool-appeasement stubs or no-op production types created only to satisfy analyzers or linters. |
| Entrypoint placement in layered service repos | Partial | `main_or_cmd_mixes_cli_parsing_server_bootstrap_and_business_config` covers overloaded entrypoints, but there is no layout-specific rule for layered service repos that keep the primary binary at repository root instead of `cmd/<name>/main.go`. |
| DB query argument type erasure via `any` | Partial | `weak_typing` and `empty_interface_parameter_overuse` cover broad `any` usage, and `sql_null_types_escape_repository_boundary` covers null-wrapper leakage, but there is no focused rule for erasing typed DB arguments to `any` immediately before `Query*` or `Exec*` calls. |
| Transaction or unit-of-work propagation | Covered | Already represented by `handler_opens_transaction`, `service_returns_tx_to_caller`, `optional_tx_nil_parameter_on_repository_api`, `transaction_object_crosses_more_than_one_layer_boundary`, and `cross_repository_write_flow_without_shared_uow_boundary`. |
| Package naming and role drift | Covered | Already represented by `service_type_outside_service_package`, `repository_type_outside_repository_package`, `shared_package_named_common_base_utils_with_mixed_exports`, and `package_name_role_drift`. |

## Net-New Rule Candidates

Only the items below should move forward as new work. Each candidate should encode a reusable Go best-practice signal rather than a one-off project convention. The covered items above should not be re-added under new IDs unless the heuristics are materially different.

### 1. `upstream_consumed_interface_declared_in_provider_package`

- [ ] Flag interfaces declared inside `repository`, `repo`, `store`, `service`, or `services` packages when:
  - [ ] the same package also contains the obvious concrete implementation,
  - [ ] upstream packages are the visible consumers,
  - [ ] and the interface mostly exists as the seam the upstream layer depends on.
- [ ] Treat this as a focused ownership-and-dependency rule, not a duplicate of `single_impl_interface`.
- [ ] Suppress for adapters, generated code, mocks, `_test.go`, and intentionally exported SDK-style interfaces.

### 2. `tool_appeasement_noop_type_in_production_package`

- [ ] Flag concrete types such as `NoopPatientRepository`, `NoopPatientService`, or similarly named stubs in production packages when nearby comments or symbol names indicate they exist only to satisfy a tool, analyzer, or linter.
- [ ] Prefer explainable evidence:
  - [ ] names like `Noop`, `Dummy`, `Stub`, or `Mock` in non-test files,
  - [ ] comments mentioning `lint`, `linter`, `analyzer`, `tooling`, or `satisfy`,
  - [ ] methods that only return zero values or `nil` with no production call sites.
- [ ] Suppress for explicit adapter packages, sample apps, intentionally shipped null-object implementations, and test-only files.

### 3. `root_main_go_in_layered_service_repo`

- [ ] Flag repository-root `main.go` only when the repo already looks like a layered service application and the rule would reinforce a broadly accepted entrypoint layout.
- [ ] Use conservative signals:
  - [ ] visible role packages such as `internal/service`, `internal/repository`, `internal/handler`, `pkg`, or `routes`,
  - [ ] more than one package beyond the entrypoint,
  - [ ] no existing `cmd/<binary>/main.go` layout.
- [ ] Do not fire for tiny one-binary utilities, single-package CLIs, examples, or intentionally small service repos.

### 4. `db_query_argument_erased_to_any`

- [ ] Flag local variables typed as `any` or `interface{}` that are populated from a concrete nullable field and then passed directly into `database/sql`, `sqlx`, `pgx`, or GORM query execution calls.
- [ ] Prefer evidence that keeps the rule explainable:
  - [ ] a local `var arg any` or `var arg interface{}`,
  - [ ] assignments from typed fields or dereferenced pointers,
  - [ ] the same symbol appearing in `QueryRowContext`, `ExecContext`, `QueryContext`, `Raw`, `Create`, or `Updates` argument lists.
- [ ] Suppress when the `any` value is part of a deliberate heterogeneous parameter list that cannot be expressed more strongly.

## Implementation Checklist

- [ ] Re-run the overlap audit before coding so the new IDs remain project-agnostic and do not duplicate `single_impl_interface`, `passthrough_wrapper_interface`, `weak_typing`, `empty_interface_parameter_overuse`, or the existing transaction and package-ownership rules.
- [ ] Add new rule definitions in the appropriate catalog modules:
  - [ ] `src/rules/catalog/go/architecture.rs` for the provider-package interface, no-op production type, and root-main layout rules.
  - [ ] `src/rules/catalog/go/idioms.rs` or `src/rules/catalog/go/architecture.rs` for the DB-argument-erasure rule, depending on the final ownership decision.
- [ ] Implement the heuristics under `src/heuristics/go/architecture.rs` and `src/heuristics/go/idioms.rs` with repo-shape guards where needed.
- [ ] Add positive and clean fixtures under `tests/fixtures/go`.
- [ ] Add integration coverage under `tests/integration_scan/go`.
- [ ] Update `rules/registry.json` only after the heuristics and catalog entries exist.
- [ ] Add guide or release-note references only after the promoted rules are actually shipped.

## Acceptance Checklist

- [ ] Each candidate maps to one stable snake_case rule ID with no duplicate semantic overlap in `rules/registry.json`.
- [ ] Each candidate represents a broadly reusable Go architecture or maintainability best practice rather than a repo-specific convention.
- [ ] Each candidate has at least one positive and one clean fixture.
- [ ] Each candidate is conditioned to avoid noise on tiny Go repos.
- [ ] Each candidate is explainable from local evidence rather than subjective style preference alone.
- [ ] Representative scan runs on layered Go service repos stay low-noise after the new rules are added.
