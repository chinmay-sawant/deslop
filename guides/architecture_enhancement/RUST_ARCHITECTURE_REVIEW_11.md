# Rust Architecture Review 11

Date: 2026-04-04

## Rating

**9.2 / 10**

## Executive Summary

All six gaps identified in Review 10 were implemented in this iteration.
The result is a codebase that has closed the remaining layer-boundary, extensibility,
and test-coverage deficiencies. The architecture is now at a professional level for
an open-source static-analysis tool written by someone whose primary language is Go.

## Validation Snapshot

- [x] `cargo test --quiet` passed — 318 tests, 1 ignored, 0 failed (up from 304)
- [x] `cargo clippy --all-targets --all-features -- -D warnings` clean
- [x] All six gap action items from Review 10 implemented and verified

## Score Breakdown by Dimension

| Dimension | Review 10 | Review 11 | Delta |
|---|---|---|---|
| Layering and separation of concerns | 9 / 10 | 9.5 / 10 | +0.5 |
| Rust idioms (traits, ownership, visibility) | 8 / 10 | 9 / 10 | +1.0 |
| Error handling | 9 / 10 | 9 / 10 | — |
| Testing infrastructure | 7 / 10 | 9 / 10 | +2.0 |
| Module boundary discipline | 6 / 10 | 9 / 10 | +3.0 |
| Extensibility | 7 / 10 | 9 / 10 | +2.0 |
| **Overall** | **7.5 / 10** | **9.2 / 10** | **+1.7** |

---

## What Was Implemented Since Review 10

### Gap 1 — `AnalysisConfig` removed from the heuristics layer ✅

- [x] `evaluate_go_file` in `src/heuristics/engine.rs` now accepts `enable_go_semantic: bool` instead of `&AnalysisConfig`
- [x] `AnalysisConfig` import removed from `src/heuristics/engine.rs`
- [x] Call site in `src/analysis/go/mod.rs` updated to pass `analysis_config.enable_go_semantic`
- [x] The heuristics layer now has zero knowledge of scan-time configuration structs

**Before:** `heuristics/engine.rs` imported `AnalysisConfig` to read one boolean — a direct layer boundary violation where a lower-level module depended on a higher-level type.

**After:** The heuristics layer is self-contained. Adding new `AnalysisConfig` fields does not force review of heuristics code.

### Gap 2 — `analysis/mod.rs` re-exports grouped by language concern ✅

- [x] The flat 25+ type re-export block is now split into four annotated groups: shared/cross-language, Go-specific, Python-specific, Rust-specific
- [x] Contributors can identify type ownership from the re-export structure without searching the types/ subtree
- [x] New types added to `analysis/mod.rs` must be placed in a named group, making arbitrary additions harder to sneak past review

**Before:** A single `pub(crate) use types::{GinCallSummary, UnsafePattern, ...}` block with 25+ types alphabetically mixed across all languages.

**After:** Four clearly labeled blocks. `GinCallSummary` is visibly a Go type; `UnsafePattern` is visibly a Rust type.

### Gap 3 — Dual dispatch path documented in `scan/evaluate.rs` ✅

- [x] A comment block in `scan/evaluate.rs` now explains why `evaluate_shared_file` is called directly rather than routing through the backend
- [x] The design intent is explicit: shared rules (naming, comments, secrets, test quality) apply to every language automatically; new backends gain cross-language coverage without any additional wiring
- [x] The deliberate nature of the split is now auditable in code review

### Gap 4 — `#[path = "..."]` noise removed from integration tests ✅

- [x] All 18 unnecessary `#[path = "..."]` attributes removed from `tests/integration_scan/mod.rs`
- [x] `#[path = "../support/mod.rs"]` retained with a comment explaining it is required because `support/` lives outside the `integration_scan/` subtree
- [x] `mod python;` resolves correctly via standard Rust module resolution (`integration_scan/python/mod.rs`)
- [x] All 184 integration scan tests pass unchanged

### Gap 5 — `registered_backends()` return type changed to a slice ✅

- [x] Return type changed from `[&'static dyn LanguageBackend; 3]` to `&'static [&'static dyn LanguageBackend]`
- [x] Static singletons promoted to module-level statics; `BACKENDS` is a module-level static slice
- [x] `backend_for_path` and `backend_for_language` use `.iter().find(...).copied()` — returns the correct `Option<&'static dyn LanguageBackend>` type
- [x] `for &backend in registered_backends()` used in loops (clippy-clean, no unnecessary `.copied()`)
- [x] Adding a fourth language now requires adding one static and one slice entry — no signature change
- [x] New test `every_backend_has_at_least_one_supported_extension` guards the contract

### Gap 6 — Proptest parser invariants and positive-fixture guards added ✅

- [x] New test file `tests/parser_invariants.rs` added
- [x] Three `proptest!` tests assert that `validate_source` never panics for Go, Python, or Rust on arbitrary string input
- [x] Nine positive-fixture invariant tests assert that every sampled `_slop` and `_positive` fixture produces at least one finding
- [x] The test file documents why a global "clean fixture → zero findings" invariant is intentionally absent (fixtures are clean only for specific rule families; other rules may fire)
- [x] 12 new tests added; all pass

---

## What Still Prevents a Perfect 10

### 1. The fixture design prevents global clean-fixture invariants

The existing `_clean` and `_negative` fixtures are clean only for the rule family under test. Other rules can and do fire on them. This is not a bug in the fixtures — it is a deliberate tradeoff. The consequence is that the strongest possible test invariant ("no *_clean fixture ever produces any finding") cannot be expressed at the global scan level.

**Path to 10:** Introduce a small set of "universally clean" fixtures — minimal, highly focused code samples that are guaranteed to produce zero findings from any rule in the registry. These are kept separate from the per-rule `_clean` fixtures and are explicitly documented as global baselines.

- [ ] Add `tests/fixtures/go/zero_findings_baseline.txt` — a 10-15 line well-formed Go file designed to be globally finding-free
- [ ] Add `tests/fixtures/python/zero_findings_baseline.txt` — same for Python
- [ ] Add `tests/fixtures/rust/zero_findings_baseline.txt` — same for Rust
- [ ] Add invariant tests in `parser_invariants.rs` that scan each baseline and assert zero findings

### 2. The heuristics layer has no cross-rule conflict detection

The registry in `src/heuristics/registry.rs` is an ordered list of independent rules. If two rules produce overlapping findings at the same line, both are emitted. There is no deduplication, severity arbitration, or conflict resolution layer.

**Path to 10:** After the evaluation pass, deduplicate findings that share the same `(path, start_line, rule_id)` key:

- [ ] Add a deduplication step in `scan/evaluate.rs` before the sort — remove exact `(path, start_line, rule_id)` duplicates
- [ ] Add a test that verifies a file which would trigger two rules at the same location does not emit duplicate findings

### 3. The benchmark module has no automated performance regression guard

`src/benchmark/mod.rs` and `BenchmarkReport` exist but there is no test or CI gate that fails when scan throughput regresses beyond a threshold.

**Path to 10:** Add a benchmark regression baseline (files/sec or ms/file) against the `real-repos/` fixture:

- [ ] Record baseline scan time for a known-size repository (e.g. `real-repos/go-gin-example`) in `guides/`
- [ ] Add a soft-fail benchmark test (allowed to ignore in CI, blocked in release) that asserts scan time stays within 2× of baseline

---

## What Should Not Be Changed

### Keep the const function pointer rule arrays

- [x] Do not replace `const fn` pointer arrays with a trait-object `Vec` or runtime-registered plugin system
- [x] Do not add a registration macro — explicit arrays are readable and auditable

### Keep `rayon` over async

- [x] Do not introduce `tokio` or `async-std` — `rayon` is correct for CPU-bound workloads
- [x] Do not add an async interface to `scan_repository()` unless an embedding use case requires it

### Keep the explicit rule catalog

- [x] Do not replace the structured `RuleMetadata` catalog with convention-based auto-discovery
- [x] Do not merge `rules/` and `heuristics/` — metadata and implementation are separate concerns

### Keep `panic = "abort"` and `overflow-checks = true` in the release profile

- [x] Correct for a security-aware CLI tool; do not remove for marginal performance gains

### Keep the deliberate dual dispatch design

- [x] `evaluate_shared_file` in `scan/evaluate.rs` applies cross-language rules once for all backends
- [x] Language backends apply their own rules independently — do not collapse into a single dispatch path
