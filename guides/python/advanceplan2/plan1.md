# Plan 1 - Async Task And Lock Lifecycle (Python)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verification passed with `cargo test python_advanceplan2`.
- [x] Parser evidence regression passed with `cargo test test_python_advanceplan2_parser_evidence`.

## Objective

Add a Python-specific async rule family for task lifecycle, lock lifetime, and retry-loop behavior that is not covered by the current blocking-I/O and mixed-sync-async checks.

## Existing Coverage Explicitly Excluded

This plan does not duplicate:

- `blocking_sync_io_in_async`
- `mixed_sync_async_module`
- `network_boundary_without_timeout`
- `broad_exception_handler`
- `exception_swallowed`

## Shipped Rules

- [x] `untracked_asyncio_task`
- [x] `background_task_exception_unobserved`
- [x] `async_lock_held_across_await`
- [x] `async_retry_sleep_without_backoff`

## Implementation Notes

- [x] Kept the scope separate from the existing blocking-I/O and broader boundary rules.
- [x] Reused shared `signature_text`, `body_start_line`, and Python `await_points` evidence instead of building a separate asyncio analysis pass.
- [x] Added conservative suppressions for `TaskGroup` usage, later task observation, and explicit registry-style follow-up lines.
- [x] Kept `async_retry_sleep_without_backoff` at `Info`.

## Parser And Evidence Work

- [x] Added Python `await_points` collection.
- [x] Reused line-local function body evidence for task, lock, and retry checks.
- [x] Kept async lifecycle reasoning parser-driven and explainable.

## Fixtures And Tests

- [x] Added parser coverage in `src/analysis/python/parser/tests.rs`.
- [x] Added grouped async positive and clean fixtures under `tests/fixtures/python/integration/advanceplan2/`.
- [x] Added grouped integration coverage in `tests/integration_scan/python/advanceplan2.rs`.

## Acceptance

- [x] Each finding identifies the async construct and the missing lifecycle step.
- [x] Structured-concurrency clean patterns stay quiet on representative fixtures.
- [x] The shipped rules remain conservative and parser-backed.

## Non-Goals

- [x] Proving coroutine cancellation correctness across module boundaries.
- [x] Framework-specific scheduling semantics in the first pass.
- [x] Replacing production load or chaos testing.