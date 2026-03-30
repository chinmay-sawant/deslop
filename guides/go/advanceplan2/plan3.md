# Plan 3 - Resource Cleanup And Transaction Hygiene (Go)

Date: 2026-03-30

## Objective

Add a conservative resource-lifetime rule family for files, rows, statements, transactions, and loop-local defer growth in Go.

## Existing Coverage Explicitly Excluded

This plan must not overlap with:

- `full_dataset_load`
- `n_plus_one_query`
- `likely_unindexed_query`
- `missing_context`
- `missing_context_propagation`
- `blocking_call_while_locked`

The new work focuses on cleanup and ownership discipline after a resource is opened, not on query shapes or context use.

## Candidate Rule Inventory

- [ ] `file_handle_without_close`
  - Detect `os.Open`, `os.Create`, or similar file-handle acquisition without a visible `Close()` or `defer Close()` in the owning function.
- [ ] `rows_without_close`
  - Detect `database/sql.Rows` results that are iterated or returned without an obvious close path when ownership appears local.
- [ ] `stmt_without_close`
  - Detect prepared statements or similar closable DB handles that are never closed.
- [ ] `tx_without_rollback_guard`
  - Detect transaction creation with `Begin` or `BeginTx` where `Commit()` exists but no early `Rollback()` guard or `defer tx.Rollback()` pattern is present.
- [ ] `defer_in_loop_resource_growth`
  - Detect repeated `defer` of closers or unlockers inside loops where resources accumulate until function exit.

## Why These Rules Belong In Advance Plan 2

- [ ] These are classic production bugs that simple happy-path tests often miss.
- [ ] AI-generated Go code frequently gets the first successful path right but skips cleanup paths and rollback guards.
- [ ] The current Go rule pack already reasons about query location and lock blocking, but not resource closure ownership.

## Parser And Evidence Work

- [ ] Extend call classification for closable resource families:
  - [ ] file handles
  - [ ] SQL rows
  - [ ] prepared statements
  - [ ] transactions
- [ ] Track matching `Close()`, `Rollback()`, and `Commit()` calls by local binding name.
- [ ] Track whether a close or rollback appears in `defer` form, direct form, or only in one branch.
- [ ] Add loop-context metadata for `defer` so loop-local accumulation is detectable.

## Implementation Checklist

- [ ] Add parser contract tests for closable resource summaries.
- [ ] Implement rule functions with clear ownership-transfer suppressions.
- [ ] Use `Warning` for transaction and row leaks; keep loop-local `defer` at `Info` first.
- [ ] Keep the rules function-local unless a repo-level ownership abstraction becomes necessary later.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `file_without_close_positive.txt`
  - [ ] `rows_without_close_positive.txt`
  - [ ] `stmt_without_close_positive.txt`
  - [ ] `tx_without_rollback_positive.txt`
  - [ ] `defer_in_loop_positive.txt`
- [ ] Negative fixtures:
  - [ ] `defer file.Close()` happy path
  - [ ] helper that intentionally returns an owned file handle
  - [ ] `defer tx.Rollback()` guard with later `Commit()`
  - [ ] loop using explicit close before the next iteration

## Acceptance Criteria

- [ ] Each finding names the resource family and the missing cleanup or rollback step.
- [ ] Straightforward ownership-transfer helpers are suppressed.
- [ ] The rule family does not require type checking to remain useful.
- [ ] Representative fixtures prove the rules are distinct from existing query and context findings.

## Non-Goals

- [ ] Proving all escape paths for returned or stored resources.
- [ ] Full transaction semantic correctness beyond the presence of a rollback guard.
- [ ] Detecting every third-party closable type in the first iteration.