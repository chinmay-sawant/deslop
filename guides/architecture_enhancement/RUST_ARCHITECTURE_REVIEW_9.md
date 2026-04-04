# Rust Architecture Review 9

Date: 2026-04-04

## Rating

9.8/10

## Executive Summary

Review 8's open checklist is now implemented.

This iteration did not change the core crate architecture, because the core architecture was already strong. The work instead removed the remaining maintenance friction around rule-binding metadata, documented the deliberate inventory guards, and added a non-failing scan workflow for expected-findings runs.

The most important change is that `binding_location` metadata is no longer maintained as repeated raw path strings across the catalog. The catalog now resolves through a centralized binding map backed by module-owned `BINDING_LOCATION` constants derived from `file!()`. That materially lowers metadata drift risk without weakening the explicit catalog model.

## Validation Snapshot

- [x] `cargo test --quiet` passed
- [x] 304 tests passed, 1 test ignored, 0 failed
- [x] `make lint` passed
- [x] `make scan-gopdfsuit-info` passed
- [x] `make scan-gopdfsuit` still returns non-zero when findings are present
- [x] the informational scan preserved findings output without failing `make`

Observed `scan-gopdfsuit-info` summary from `temp_gopdfsuit.txt`:

- source files discovered: 125
- source files analyzed: 125
- functions fingerprinted: 876
- findings: 1622
- parse failures: 0

## What Was Improved Since Review 8

### 1. Binding metadata is now constant-backed instead of raw-string repeated

- [x] implementation modules now expose `BINDING_LOCATION` constants derived from `file!()`
- [x] `src/rules/catalog/bindings.rs` now centralizes catalog binding references
- [x] catalog entries now refer to named binding constants instead of repeated literal path strings
- [x] the previous Go `library` family binding was refined from the family aggregator path to the leaf implementation file `src/heuristics/go/library_misuse/library.rs`
- [x] a regression test now keeps Go `library` bindings pinned to that leaf implementation

Impact:

- metadata drift is harder to introduce accidentally
- catalog maintenance is lower-friction during refactors
- binding precision improved where the maintenance win was clear

### 2. Inventory-count update flow is now documented where contributors will hit it

- [x] `guides/inventory-regression-guards.md` documents how to update the exact inventory guards intentionally
- [x] `src/rules.rs` now labels the source rule-id count as an intentional maintenance guard
- [x] `tests/parser_corpus_regression.rs` now labels the corpus entry count the same way
- [x] assertion messages now point maintainers at the documented update flow

Impact:

- exact-count tests remain strict without becoming opaque
- future inventory updates should be less confusing for contributors

### 3. Scan ergonomics are better without weakening policy

- [x] `makefile` now includes `scan-info`
- [x] `makefile` now includes `scan-gopdfsuit-info`
- [x] `makefile` now includes `scan-snapback-info`
- [x] `makefile` now includes `scan-claw-info`
- [x] strict scan targets were preserved unchanged for policy-enforcing workflows

Impact:

- expected-findings scans no longer need to fail the shell session to be useful
- CI-style strict behavior remains available and unchanged

## What Should Not Be "Improved" Away

### 1. Keep the explicit catalog layer

- [x] maintain a first-class rule catalog with language, family, severity, configurability, and binding metadata

Why:

- the new constant-backed binding map makes the catalog safer without removing its documentation and product value

### 2. Keep the fail-closed inventory guards

- [x] preserve exact source rule-id counts
- [x] preserve exact corpus entry counts

Why:

- the documentation improvement should reduce friction, not reduce strictness

### 3. Keep both strict and informational scan workflows

- [x] preserve strict `scan-*` targets for enforcement
- [x] preserve informational `scan-*-info` targets for review and exploration

Why:

- these two workflows serve different operational goals and should coexist

## Remaining Gap To 10/10

The current review chain is effectively closed. Any future work here is now just refinement, not an outstanding architectural issue.

## Architectural Verdict

The project is stronger than Review 8, but the increase is modest because the remaining Review 8 items were refinement work rather than structural defects.

The architecture is now in a state where:

- rule metadata is both validated and materially easier to maintain
- regression guards are strict and documented
- scan ergonomics are better for real repository triage
- the original layered design remains intact

My updated solution-architect rating is **9.8/10**.
