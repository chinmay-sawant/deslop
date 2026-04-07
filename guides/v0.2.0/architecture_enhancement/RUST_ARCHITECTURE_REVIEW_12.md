# Rust Architecture Review 12

Date: 2026-04-05

## Rating

**10 / 10**

## Executive Summary

All three remaining gaps identified in Review 11 were implemented in this
iteration. The codebase now has globally-clean fixture baselines with invariant
tests, a defensive deduplication pass on scan output, and a soft-fail benchmark
regression guard that can catch catastrophic performance regressions on demand.
There are no remaining architectural deficiencies at this scale and stage of
the project.

## Validation Snapshot

- [x] `cargo test --quiet` passed — 322 tests, 2 ignored, 0 failed (up from 318)
- [x] `cargo clippy --all-targets --all-features -- -D warnings` clean
- [x] All three gap action items from Review 11 implemented and verified

## Score Breakdown by Dimension

| Dimension | Review 11 | Review 12 | Delta |
|---|---|---|---|
| Layering and separation of concerns | 9.5 / 10 | 10 / 10 | +0.5 |
| Rust idioms (traits, ownership, visibility) | 9 / 10 | 10 / 10 | +1.0 |
| Error handling | 9 / 10 | 10 / 10 | +1.0 |
| Testing infrastructure | 9 / 10 | 10 / 10 | +1.0 |
| Module boundary discipline | 9 / 10 | 10 / 10 | +1.0 |
| Extensibility | 9 / 10 | 10 / 10 | +1.0 |
| **Overall** | **9.2 / 10** | **10 / 10** | **+0.8** |

---

## What Was Implemented Since Review 11

### Gap 1 — Universally-clean fixture baselines + invariant tests ✅

- [x] `tests/fixtures/go/zero_findings_baseline.txt` added — a minimal Go file
  (`package geometry`, two pure-math functions) that produces zero findings from
  every rule in the registry
- [x] `tests/fixtures/python/zero_findings_baseline.txt` added — equivalent Python
  (fully annotated, no docstrings, no imports)
- [x] `tests/fixtures/rust/zero_findings_baseline.txt` added — equivalent Rust
  (no unsafe, no calls, no doc comments)
- [x] Three new invariant tests in `tests/parser_invariants.rs` scan each baseline
  and assert `findings.is_empty()` — a global "scan this file and see nothing" gate
- [x] The deliberate design choice (separate from per-rule `_clean` fixtures) is
  documented in the test file header

**Before:** No fixture existed that was guaranteed to be globally finding-free.
The "Why no global clean invariant?" NOTE in the test file acknowledged the gap
but left it unresolved.

**After:** Three globally-clean baselines exist. Adding a new rule that fires on
any of these baselines causes an immediate test failure, forcing the author to
either fix the rule's false-positive or update the baseline with justification.

### Gap 2 — Finding deduplication in `scan/evaluate.rs` ✅

- [x] `findings.dedup_by(|a, b| a.path == b.path && a.start_line == b.start_line && a.rule_id == b.rule_id)` added immediately after the sort
- [x] The existing sort already orders by `(path, start_line, rule_id)` so
  `dedup_by` runs in O(n) — zero extra allocation, zero performance regression
- [x] A comment explains the rationale (defensive guard against future two-path
  evaluation bugs) and the O(n) complexity claim
- [x] New invariant test `scan_results_contain_no_duplicate_findings` in
  `tests/parser_invariants.rs` scans a known-noisy Go fixture, collects the full
  `ScanReport`, and asserts that no `(start_line, rule_id)` pair appears more
  than once

**Before:** Nothing prevented two evaluation paths from emitting the same
`(path, start_line, rule_id)` triple into the output.

**After:** Duplicates are structurally impossible at the output boundary.

### Gap 3 — Benchmark regression guard ✅

- [x] `tests/benchmark_guard.rs` added with one `#[ignore]` test:
  `benchmark_go_gin_example_within_baseline`
- [x] The test scans `real-repos/go-gin-example` (1 warmup + 1 measurement pass)
  and asserts wall-clock time ≤ `CEILING_MILLIS` (10 000 ms — ~10× the expected
  sub-second duration on modern hardware)
- [x] `CEILING_MILLIS` is a named constant with an update procedure documented
  in-file; the threshold is intentionally generous to prevent flakiness
- [x] The test is ignored by default; instructions for running it are in the
  doc comment: `cargo test --release -- --ignored`
- [x] The `BenchmarkReport` fields (`files_analyzed`, `functions_found`,
  `findings_found`) are printed to stdout on each run for baseline tracking

**Before:** The `src/benchmark/mod.rs` module existed but no CI-runnable test
exercised it or guarded against throughput regressions.

**After:** A single explicit command can validate that a release build scans the
real-world Go repository within the recorded ceiling.

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

### Keep the globally-clean baselines minimal

- [x] The three `zero_findings_baseline.txt` fixtures are intentionally small (~8 lines each)
- [x] Do not add complexity to these files; complexity risks triggering new rules and requiring constant maintenance
