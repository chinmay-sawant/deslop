# Go Data Access Gap Plan For v0.3.0

Date: 2026-04-10

## Audit

- [x] Re-check the live Go registry against the current integration test references.
- [x] Confirm the remaining unreferenced Go rules are isolated to the `data_access` family.
- [x] Keep the plan repo-agnostic and scoped to reusable Go query and persistence patterns.

## Remaining Coverage Gaps

| Rule ID | Validation approach |
| --- | --- |
| `default_transaction_enabled_for_bulk_create` | Add a positive and clean GORM bulk-create fixture pair that differs only by `SkipDefaultTransaction`. |
| `findinbatches_candidate_for_large_scan` | Add a request-path GORM fixture pair that contrasts `Find(...)` with `FindInBatches(...)`. |
| `many_column_or_filter_chain` | Add a request-path SQL/GORM fixture pair that contrasts a long `Or(...)` chain with a shorter alternative. |
| `pgx_collectrows_unbounded_materialization` | Add a pgx request-path fixture pair that contrasts an unbounded `CollectRows(...)` call with a bounded query. |
| `rows_to_struct_allocation_per_row_without_reuse` | Add a row-scanning fixture pair that contrasts per-row allocation with a reused scratch struct. |
| `sqlx_select_large_slice_without_limit` | Add a request-path sqlx fixture pair that contrasts an unbounded `Select(...)` with a bounded query. |
| `unbounded_in_clause_expansion` | Add a request-path SQL fixture pair that contrasts variadic `IN (...)` expansion with a bounded argument shape. |
| `updates_map_allocated_per_row` | Add a looped GORM update fixture pair that contrasts per-row map allocation with a reused update payload. |

## Implementation Checklist

- [x] Add new positive and clean Go fixtures under `tests/fixtures/go/` for the eight remaining data-access rules.
- [x] Extend `tests/integration_scan/data_access.rs` so each missing rule has a targeted positive and negative assertion.
- [x] Keep the new fixtures minimal and explainable so they stay readable as regression tests.

## Verification Checklist

- [x] Run the Go integration tests that cover the new data-access coverage.
- [x] Re-run the full integration test suite to confirm the remaining Go rules are now referenced.
- [x] Confirm the Go registry no longer has completely unreferenced rules in the test surface.
