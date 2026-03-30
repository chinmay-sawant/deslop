# Plan 1 - Async Task And Lock Lifecycle (Python)

Date: 2026-03-30

## Objective

Add a Python-specific async rule family for task lifecycle, lock lifetime, and retry-loop behavior that is not covered by the current blocking-I/O and mixed-sync-async checks.

## Existing Coverage Explicitly Excluded

This plan must not duplicate:

- `blocking_sync_io_in_async`
- `mixed_sync_async_module`
- `network_boundary_without_timeout`
- `broad_exception_handler`
- `exception_swallowed`

The new work focuses on async orchestration mistakes after a coroutine boundary already exists.

## Candidate Rule Inventory

- [ ] `untracked_asyncio_task`
  - Detect `asyncio.create_task(...)` or equivalent task creation whose return value is ignored or immediately discarded.
- [ ] `background_task_exception_unobserved`
  - Detect fire-and-forget task creation with no callback, join, task group, or result observation path.
- [ ] `async_lock_held_across_await`
  - Detect `async with lock:` or `await lock.acquire()` scopes that continue across unrelated await points and therefore serialize too much work.
- [ ] `async_retry_sleep_without_backoff`
  - Detect coroutine retry loops that repeatedly `await asyncio.sleep(constant)` with no backoff growth, jitter, or bounded attempt policy.

## Why These Rules Belong In Advance Plan 2

- [ ] The shipped Python pack already catches obvious synchronous misuse inside async code, but not the next layer of task orchestration slop.
- [ ] These patterns are common in generated async wrappers because they pass small tests and fail only under error or load.
- [ ] The candidate rules are still explainable with local parser evidence and line-order reasoning.

## Parser And Evidence Work

- [ ] Extend Python parser output with task-creation summaries:
  - [ ] task factory call lines (`create_task`, `ensure_future`, `TaskGroup.create_task`)
  - [ ] assigned variable names or immediate discard patterns
  - [ ] callback or result-observation calls on known task bindings
- [ ] Add async lock summaries:
  - [ ] `async with` blocks
  - [ ] explicit acquire and release calls
  - [ ] await points within the protected region
- [ ] Add retry-loop metadata:
  - [ ] loop-local sleep calls
  - [ ] constant sleep values
  - [ ] obvious attempt counters or exponential backoff updates

## Implementation Checklist

- [ ] Add parser unit tests for task creation, task binding, and lock-scope evidence.
- [ ] Implement async lifecycle heuristics in a dedicated Python sub-module or a clearly separated section of the existing maintainability pack.
- [ ] Use conservative suppressions for:
  - [ ] test helpers
  - [ ] explicit task registries or supervisor lists
  - [ ] `TaskGroup` or `gather` patterns that obviously observe completion
- [ ] Keep `async_retry_sleep_without_backoff` at `Info` initially.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `async_untracked_task_positive.txt`
  - [ ] `async_background_exception_positive.txt`
  - [ ] `async_lock_across_await_positive.txt`
  - [ ] `async_retry_backoff_positive.txt`
- [ ] Negative fixtures:
  - [ ] task stored and awaited later
  - [ ] task supervised through `TaskGroup`
  - [ ] short lock section that releases before unrelated await work
  - [ ] retry loop with exponential or bounded backoff

## Acceptance Criteria

- [ ] Each finding identifies the concrete async construct and the missing lifecycle step.
- [ ] The rules remain parser-driven and do not require runtime event-loop inspection.
- [ ] Normal structured-concurrency patterns stay quiet on representative fixtures.

## Non-Goals

- [ ] Proving coroutine cancellation correctness across module boundaries.
- [ ] Framework-specific scheduling semantics in the first pass.
- [ ] Replacing production load or chaos testing.