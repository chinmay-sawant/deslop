# Plan 1 - Channel, Timer, and Select Lifecycle (Go)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verification passed with `cargo test --test integration_scan test_go_advanceplan2`.
- [x] Parser evidence regression passed with `cargo test test_collects_package_vars_interfaces_structs_and_signature_text`.

## Objective

Add a conservative Go rule pack for channel ownership, timer lifecycle, and select-loop misuse that is not already covered by the current context and goroutine heuristics.

## Shipped Rules

- [x] `range_over_local_channel_without_close`
- [x] `double_close_local_channel`
- [x] `send_after_local_close_risk`
- [x] `time_after_in_loop`
- [x] `ticker_without_stop`

## Implementation Notes

- [x] Kept the scope separate from `goroutine_without_coordination`, `goroutine_without_shutdown_path`, `busy_waiting`, and the existing context rules.
- [x] Reused the existing Go parser plus new shared `signature_text` and `body_start_line` metadata.
- [x] Implemented conservative function-local line scanning so findings stay explainable without pretending to prove cross-goroutine happens-before semantics.
- [x] Anchored each finding to the relevant range, close, send, timer, or ticker line.
- [x] Used `Warning` for lifecycle-risk findings and `Info` for `time_after_in_loop`.

## Parser And Evidence Work

- [x] Extended shared function evidence with `signature_text` and `body_start_line` to support precise body-line heuristics.
- [x] Added Go parser coverage for the new summary fields in `src/analysis/go/parser/tests.rs`.
- [x] Kept channel and timer ownership reasoning local and parser-driven rather than introducing a second analysis pipeline.

## Fixtures And Tests

- [x] Added positive fixtures:
  - [x] `channel_range_without_close_positive.txt`
  - [x] `double_close_channel_positive.txt`
  - [x] `send_after_close_positive.txt`
  - [x] `time_after_in_loop_positive.txt`
  - [x] `ticker_without_stop_positive.txt`
- [x] Added clean coverage in `channel_lifecycle_clean.txt`.
- [x] Added grouped integration coverage in `tests/integration_scan/go_advanceplan2.rs`.

## Acceptance

- [x] Each rule emits a stable rule id and line-anchored evidence.
- [x] Clean representative fixtures stay quiet.
- [x] Cross-goroutine proofs remain intentionally out of scope.
- [x] The shipped implementation is conservative and explainable.

## Non-Goals

- [x] Proving general deadlocks across functions or packages.
- [x] Full happens-before reasoning for sends and receives across goroutines.
- [x] Replacing race detection or runtime tracing.