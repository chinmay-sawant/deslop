---
title: Deeper Goroutine Lifetime Analysis — Plan (checklist)
status: draft
date: 2026-03-29
---

# Deeper Goroutine Lifetime Analysis — Checklist

Purpose: A concise, actionable checklist for implementing a conservative, repo-local goroutine lifetime analysis for Go. Each item is intentionally granular so progress can be tracked and validated.

## Checklist

- [ ] Confirm parser evidence fields are populated
  - [ ] `ParsedFunction.context_factory_calls` contains derived context factories (WithCancel/WithTimeout/WithDeadline).
  - [ ] `ParsedFunction.goroutines` contains `go` statement line numbers.
  - [ ] `ParsedFunction.loop_goroutines` marks goroutines launched inside loops.
  - [ ] `ParsedFunction.unmanaged_goroutines` identifies func-literals with loops and missing shutdown signals.
  - [ ] `ParsedFunction.calls` includes observed `cancel()` call sites.

- [ ] Design conservative heuristic (single-function, repo-local)
  - [ ] For each `ContextFactoryCall`, find `go` statements with `line > factory.line`.
  - [ ] Restrict to goroutines that are looped or appear unmanaged.
  - [ ] If a `cancel()` call exists in the function, only flag goroutines launched before the earliest `cancel()`.
  - [ ] Do not flag goroutines whose bodies show explicit shutdown (`ctx.Done()`), relying on existing `unmanaged_goroutines` filtering.

- [ ] Implement heuristic in Rust
  - [ ] Add `deeper_goroutine_lifetime_findings` to `src/heuristics/concurrency.rs`.
  - [ ] Produce findings with rule ID: `goroutine_derived_context_unmanaged`.
  - [ ] Use severity `Warning` for unmanaged-loop goroutines and `Info` for loop-adjacent cases.
  - [ ] Include clear evidence: derived-context factory (factory_name & cancel variable), goroutine launch line, relationship to observed cancel() (or absence).

- [ ] Register heuristic in pipeline
  - [ ] Import and call the new function from `src/heuristics/mod.rs` within `evaluate_go_file` (near other goroutine/context heuristics).

- [ ] Add textual fixtures (no compiled Go files)
  - [ ] `tests/fixtures/go/goroutine_deeper_slop.txt`: derived context created, long-lived goroutine launched, `cancel()` after launch (should flag).
  - [ ] `tests/fixtures/go/goroutine_deeper_clean.txt`: goroutine listens on `ctx.Done()` (should not flag).
  - [ ] Ensure fixtures follow existing formatting and `go_fixture!` conventions used by integration tests.

- [ ] Add integration tests (tests remain Rust — fixtures contain text)
  - [ ] Extend `tests/integration_scan/concurrency.rs` with a test that writes the slop fixture and asserts `rule_id == "goroutine_derived_context_unmanaged"` appears.
  - [ ] Add a complementary test using the clean fixture that asserts the rule does not appear.
  - [ ] Keep test files devoid of raw Go compilation; use the same pattern as other goroutine tests.

- [ ] Local validation and tuning
  - [ ] Run `cargo test` and fix any compile errors.
  - [ ] Run `cargo test` focusing on `integration_scan::concurrency` to validate behavior.
  - [ ] Tune evidence messaging to minimize overlap with `missing_cancel_call` and `goroutine_without_shutdown_path` (explain why this finding is distinct).

- [ ] Documentation & rollout
  - [ ] Add rule description to `guides/features-and-detections.md` and update frontend docs (`frontend/src/features/docs/DocsPage.tsx` assets) if desired.
  - [ ] Set initial severity conservatively (Info/Warning) and consider gating via a feature flag or opt-in ruleset.

- [ ] Follow-ups (future phases)
  - [ ] Consider conservative interprocedural propagation of derived contexts (create/propagate context handles across helper calls and package boundaries).
  - [ ] Replace simple text searches with AST-resolved detection of `ctx.Done()` in goroutine bodies.
  - [ ] Add unit tests for parser helpers that populate `context_factory_calls` and `unmanaged_goroutines` if coverage gaps are discovered.

## Short-term next actions

- [ ] Merge this checklist into the repo.
- [ ] Optionally implement the Rust heuristic and fixtures now — I can proceed when you confirm.

---

End of checklist
