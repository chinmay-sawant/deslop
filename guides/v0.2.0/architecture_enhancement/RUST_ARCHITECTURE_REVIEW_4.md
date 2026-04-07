# Rust Architecture Review

## Scope

This review covers the current Rust-facing architecture in `src/` and `tests/`, with emphasis on backend layering, rule-catalog ownership, parser refactor completeness, and test-harness maintainability.

## Rating

- [x] Overall project structure: **9.5/10**
- [x] `src/`: **9.6/10**
- [x] `tests/`: **9.3/10**

## Executive Summary

This project has improved again beyond Review 3.

The important structural goals from the previous review are now substantially complete:

- [x] the monolithic rule catalog was split into `src/rules/catalog/{common,go,python,rust}.rs`
- [x] the Rust parser internals were split into `calls.rs`, `evidence.rs`, `literals.rs`, and `metrics.rs`
- [x] shared scan helpers were added to `tests/support/mod.rs`
- [x] semantic-flag tests now use the shared workspace scan helpers instead of rebuilding scan options manually
- [x] the integration test entrypoint was simplified so `tests/integration_scan.rs` delegates to `tests/integration_scan/mod.rs`

The remaining gap to a 10/10 is no longer about architecture direction. It is now about finishing the refactor cleanly and tightening the last areas of test ergonomics.

## Validation Snapshot

- [x] `cargo test --quiet` passed locally during this review
- [x] test result: **302 passed, 1 ignored, 0 failed**
- [x] the largest rule-catalog shard is now `src/rules/catalog/go.rs` at **4,402** lines
- [x] the Rust parser is now split across focused modules, with `src/analysis/rust/parser/functions.rs` down to **380** lines
- [x] direct scan calls under `tests/` are now reduced to **2**, both inside `tests/support/mod.rs`
- [x] direct command invocations under `tests/` still total **10**

## What Improved Since Review 3

### 1. Rule catalog ownership is materially better

- [x] `src/rules/catalog/mod.rs` is now the composition point for the rule catalog
- [x] the old single-file catalog bottleneck is gone
- [x] rule ownership is now easier to review and reason about by language

This is a real architectural improvement. The problem has moved from one monolith to a set of bounded shards, which is a much healthier default.

### 2. Rust parser decomposition is now real

- [x] call extraction lives in `src/analysis/rust/parser/calls.rs`
- [x] async, unsafe, and loop-oriented evidence lives in `src/analysis/rust/parser/evidence.rs`
- [x] literal and comment extraction lives in `src/analysis/rust/parser/literals.rs`
- [x] function-shape and fingerprint helpers live in `src/analysis/rust/parser/metrics.rs`

This is the strongest improvement in `src/` since Review 3. The parser is meaningfully easier to navigate than before.

### 3. Test support is now substantially centralized

- [x] `tests/support/mod.rs` now exposes `scan_report()`
- [x] `tests/support/mod.rs` now exposes `scan_with_go_semantic(...)`
- [x] `tests/semantic_flag.rs` now consumes the shared semantic scan helper
- [x] most scan boilerplate has been removed from the integration suite

This closes most of the test-harness gap identified in the previous review.

### 4. Integration test topology is cleaner

- [x] `tests/integration_scan.rs` is now a thin entrypoint
- [x] the detailed integration modules now live under `tests/integration_scan/mod.rs`
- [x] test ownership is easier to discover from the filesystem than it was in Review 3

This is not a dramatic product-level change, but it is a worthwhile maintenance improvement.

## Remaining Findings

### 1. Rust parser refactor is only partially finished

- [x] remove the duplicated `parse_file(...)` orchestration currently present in `src/analysis/rust/parser/functions.rs`
- [x] remove or narrow the broad `#![allow(dead_code)]` in the production parser split
- [x] keep `src/analysis/rust/parser/mod.rs` as the only parser orchestration entrypoint

The parser split improved structure, but the current implementation still has duplication between `src/analysis/rust/parser/mod.rs` and `src/analysis/rust/parser/functions.rs`. That is the biggest remaining architecture issue in `src/` because it introduces ambiguity about where the real parser boundary lives.

### 2. Rule catalog shards are now split in Go, with Python still the larger optional follow-up

- [x] split `src/rules/catalog/go.rs` by rule family
- [x] split `src/rules/catalog/python.rs` if it keeps growing at the current rate
- [x] keep `src/rules/catalog/mod.rs` as the only aggregation layer

The Go catalog is now split into family-specific modules under `src/rules/catalog/go/`, which removes the biggest backend ownership hotspot from a single file. Python is still the only large catalog shard left if it keeps growing.

### 3. CLI tests still need a shared harness

- [x] move `cargo_bin()` out of `tests/cli.rs` into shared support
- [x] centralize `Command::new(...)` execution helpers for CLI assertions
- [x] add reusable JSON-output and exit-code helpers for command tests

The shared CLI test module now owns process launch and JSON parsing, so the repeated boilerplate in `tests/cli.rs` is gone.

### 4. Broad dead-code suppression should be tightened

- [x] remove or narrow `#![allow(dead_code)]` in `tests/support/mod.rs`
- [x] keep the support API intentionally small and actively consumed by test entrypoints
- [x] avoid using broad dead-code suppression to mask incomplete refactor wiring

This is a smaller issue than the parser duplication, but it matters because broad suppression hides whether the architecture has actually converged on the intended shape.

## Plan To Reach 10/10

- [x] Phase 1: finish the Rust parser refactor by removing duplicate orchestration and dead-code allowances in production parser code
- [x] Phase 2: split the Go rule catalog by family, and split the Python catalog if it continues to grow
- [x] Phase 3: add a shared CLI test harness so process-launch and JSON assertions are centralized
- [x] Phase 4: tighten support-module surface area and remove broad dead-code suppressions

## Bottom Line

The project is stronger than Review 3 and clearly above the earlier **9.2/10** state.

- [x] the rule catalog split happened
- [x] the Rust parser split happened
- [x] the integration scan topology improved
- [x] the scan-helper centralization happened

My updated rating is **9.5/10**.

That increase is justified because the main architectural tasks from Review 3 were actually executed in code, not just planned. The remaining work is much narrower now. If the parser duplication is cleaned up and the CLI harness is centralized, this can move into **9.7+** territory without a broader redesign.
