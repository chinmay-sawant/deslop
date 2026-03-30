# Plan 1 - Channel, Timer, and Select Lifecycle (Go)

Date: 2026-03-30

## Objective

Add a conservative Go rule pack for channel ownership, timer lifecycle, and select-loop misuse that is not already covered by the current context and goroutine heuristics.

## Existing Coverage Explicitly Excluded

This plan must not re-implement or rename the currently shipped rules below:

- `goroutine_without_coordination`
- `goroutine_spawn_in_loop`
- `goroutine_without_shutdown_path`
- `goroutine_derived_context_unmanaged`
- `busy_waiting`
- `sleep_polling`
- `missing_cancel_call`
- `context_background_used`
- `missing_context_propagation`

The goal here is lifecycle correctness around channels and timers, not a second pass on context propagation or general goroutine shutdown detection.

## Candidate Rule Inventory

- [ ] `range_over_local_channel_without_close`
  - Detect a function or local closure that ranges over a channel it appears to create or own locally, with no visible `close(ch)` path in the same ownership scope.
- [ ] `double_close_local_channel`
  - Detect the same locally scoped channel being closed more than once in one function body.
- [ ] `send_after_local_close_risk`
  - Detect a local channel that is closed and later used in a `ch <- value` send within the same function or goroutine literal.
- [ ] `time_after_in_loop`
  - Detect `time.After(...)` allocations inside loops where a reusable timer or context deadline would be more stable and cheaper.
- [ ] `ticker_without_stop`
  - Detect `time.NewTicker(...)` created in a function that appears to own the ticker but never calls `Stop()`.

## Why These Rules Belong In Advance Plan 2

- [ ] These patterns are common in low-context generated Go code because they compile cleanly but fail under load, shutdown, or longer runtimes.
- [ ] The current Go rule pack already catches obvious coordination gaps, but it does not model channel close ownership or timer cleanup.
- [ ] All candidate rules can be implemented conservatively with local parser evidence plus limited line-order reasoning.

## Parser And Evidence Work

- [ ] Extend Go parser evidence with explicit channel summaries:
  - [ ] channel creations (`make(chan ...)`)
  - [ ] close calls and their line numbers
  - [ ] send expressions and receive/range sites
  - [ ] whether the channel appears to escape through return values, struct fields, or function arguments
- [ ] Extend function evidence with timer summaries:
  - [ ] `time.After(...)` call lines
  - [ ] `time.NewTimer(...)` and `time.NewTicker(...)` constructor lines
  - [ ] `Stop()` calls paired to known local timer/ticker bindings
- [ ] Capture `for range ch` and `select` case metadata so findings can anchor the exact lifecycle edge.

## Implementation Checklist

- [ ] Confirm current parser output does not already expose enough channel and timer metadata.
- [ ] Add parser tests under `src/analysis/go/parser/tests.rs` for:
  - [ ] local channel creation and close detection
  - [ ] send-after-close ordering in a single function
  - [ ] `for range` channel loops
  - [ ] ticker creation and stop calls
- [ ] Implement the rule functions in a Go-focused heuristic module.
- [ ] Register the new findings in the Go evaluation pipeline without affecting Python or Rust.
- [ ] Keep severities conservative:
  - [ ] lifecycle leaks and double-close risks start at `Warning`
  - [ ] `time_after_in_loop` starts at `Info` unless nested loop evidence raises confidence

## Fixture Plan

- [ ] Add positive fixtures under `tests/fixtures/go/` for each rule:
  - [ ] `channel_range_without_close_positive.txt`
  - [ ] `double_close_channel_positive.txt`
  - [ ] `send_after_close_positive.txt`
  - [ ] `time_after_in_loop_positive.txt`
  - [ ] `ticker_without_stop_positive.txt`
- [ ] Add negative fixtures that demonstrate the intended suppressions:
  - [ ] channels closed by documented owner goroutines
  - [ ] timers reused safely outside loops
  - [ ] tickers stopped through `defer ticker.Stop()`
  - [ ] channels intentionally returned to a caller-owned lifecycle

## Acceptance Criteria

- [ ] Each rule emits a stable rule id, a line-anchored message, and evidence naming the owned channel or timer binding.
- [ ] Cross-goroutine proofs are intentionally avoided unless ownership is local and obvious.
- [ ] Clean ownership-transfer patterns do not trigger on representative fixtures.
- [ ] The rule pack remains parser-driven and explainable.

## Non-Goals

- [ ] Proving general deadlocks across functions or packages.
- [ ] Full happens-before reasoning for sends and receives across goroutines.
- [ ] Replacing race detection or runtime tracing.