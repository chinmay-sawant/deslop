# Plan 3 - Resource Cleanup And Transaction Hygiene (Go)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verified by the Go advanceplan2 integration suite.

## Objective

Add a conservative resource-lifetime rule family for files, rows, statements, transactions, and loop-local defer growth in Go.

## Shipped Rules

- [x] `file_handle_without_close`
- [x] `rows_without_close`
- [x] `stmt_without_close`
- [x] `tx_without_rollback_guard`
- [x] `defer_in_loop_resource_growth`

## Implementation Notes

- [x] Kept the scope separate from `full_dataset_load`, `n_plus_one_query`, `likely_unindexed_query`, and the context rules.
- [x] Implemented local binding and cleanup matching through conservative body-line reasoning.
- [x] Kept row, statement, file, and transaction findings at `Warning` and loop-local defer growth at `Info`.
- [x] Suppressed straightforward ownership-transfer cases when the local binding is returned.

## Fixtures And Tests

- [x] Added positive fixtures:
  - [x] `file_without_close_positive.txt`
  - [x] `rows_without_close_positive.txt`
  - [x] `stmt_without_close_positive.txt`
  - [x] `tx_without_rollback_positive.txt`
  - [x] `defer_in_loop_positive.txt`
- [x] Added clean coverage in `resource_hygiene_clean.txt`.
- [x] Added grouped integration coverage in `tests/integration_scan/go_advanceplan2.rs`.

## Acceptance

- [x] Each finding names the resource family and the missing cleanup or rollback step.
- [x] The rules remain useful without full type checking.
- [x] Representative clean fixtures stay quiet.

## Non-Goals

- [x] Proving all escape paths for returned or stored resources.
- [x] Full transaction semantic correctness beyond the presence of a rollback guard.
- [x] Detecting every third-party closable type in the first iteration.