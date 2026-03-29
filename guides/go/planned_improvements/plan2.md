---
title: Deeper Goroutine Lifetime Analysis — Plan (checklist)
status: draft
date: 2026-03-29
---

# Deeper Goroutine Lifetime Analysis — Checklist

Purpose: A concise, actionable checklist for implementing a conservative, repo-local goroutine lifetime analysis for Go. Each item is intentionally granular so progress can be tracked and validated.

## Checklist

- [x] Confirm parser evidence fields are populated
  - [x] `ParsedFunction.context_factory_calls` contains derived context factories (WithCancel/WithTimeout/WithDeadline).
  - [x] `ParsedFunction.goroutines` contains `go` statement line numbers.
  - [x] `ParsedFunction.loop_goroutines` marks goroutines launched inside loops.
  - [x] `ParsedFunction.unmanaged_goroutines` identifies func-literals with loops and missing shutdown signals.
  - [x] `ParsedFunction.calls` includes observed `cancel()` call sites.

- [x] Design conservative heuristic (single-function, repo-local)
  - [x] For each `ContextFactoryCall`, find `go` statements with `line > factory.line`.
  - [x] Restrict to goroutines that are looped or appear unmanaged.
  - [x] If a `cancel()` call exists in the function, only flag goroutines launched before the earliest `cancel()`.
  - [x] Do not flag goroutines whose bodies show explicit shutdown (`ctx.Done()`), relying on existing `unmanaged_goroutines` filtering.

- [x] Implement heuristic in Rust
  - [x] Add `deeper_goroutine_lifetime_findings` to `src/heuristics/concurrency.rs`.
  - [x] Produce findings with rule ID: `goroutine_derived_context_unmanaged`.
  - [x] Use severity `Warning` for unmanaged-loop goroutines and `Info` for loop-adjacent cases.
  - [x] Include clear evidence: derived-context factory (factory_name & cancel variable), goroutine launch line, relationship to observed cancel() (or absence).

- [x] Register heuristic in pipeline
  - [x] Import and call the new function from `src/heuristics/mod.rs` within `evaluate_go_file` (near other goroutine/context heuristics).

- [x] Add textual fixtures (no compiled Go files)
  - [x] `tests/fixtures/go/goroutine_deeper_slop.txt`: derived context created, long-lived goroutine launched, `cancel()` after launch (should flag).
  - [x] `tests/fixtures/go/goroutine_deeper_clean.txt`: goroutine listens on `ctx.Done()` (should not flag).
  - [x] Ensure fixtures follow existing formatting and `go_fixture!` conventions used by integration tests.

- [x] Add integration tests (tests remain Rust — fixtures contain text)
  - [x] Extend `tests/integration_scan/concurrency.rs` with a test that writes the slop fixture and asserts `rule_id == "goroutine_derived_context_unmanaged"` appears.
  - [x] Add a complementary test using the clean fixture that asserts the rule does not appear.
  - [x] Keep test files devoid of raw Go compilation; use the same pattern as other goroutine tests.

- [x] Local validation and tuning
  - [x] Run `cargo test` and fix any compile errors.
  - [x] Run `cargo test` focusing on `integration_scan::concurrency` to validate behavior.
  - [x] Tune evidence messaging to minimize overlap with `missing_cancel_call` and `goroutine_without_shutdown_path` (explain why this finding is distinct).

- [x] Documentation & rollout
  - [x] Add rule description to `guides/features-and-detections.md` and update frontend docs (`frontend/src/features/docs/DocsPage.tsx` assets) if desired.
  - [x] Set initial severity conservatively (Info/Warning) and consider gating via a feature flag or opt-in ruleset.

Future follow-ups retained for a later pass:

- Consider conservative interprocedural propagation of derived contexts across helper calls and package boundaries.
- Replace the current text-shape shutdown detection with AST-resolved `ctx.Done()` matching inside goroutine bodies.
- Add extra parser-helper unit tests if future changes expose gaps around `context_factory_calls` or `unmanaged_goroutines`.

## Short-term next actions

- [x] Merge this checklist into the repo.
- [x] Optionally implement the Rust heuristic and fixtures now — I can proceed when you confirm.

---

End of checklist
