# Rust Architecture Review 10

Date: 2026-04-04

## Rating

**7.5 / 10**

## Executive Summary

This is a cold-eye architectural assessment of the codebase as it stands after Review 9. The purpose is to establish a new baseline score against an external Rust architect perspective, identify what the codebase already does well, surface the remaining structural gaps that the previous review cycles did not address, and produce an action plan for closing them.

The author came from Go (PDF/UA-2 and PDF/A-4 work on gopdfsuit) with no prior Rust experience. The core Go instincts — pipeline architecture, interface-based dispatch, explicit error propagation — translated into correct Rust patterns almost directly. The gaps are refinement issues, not design failures. Nothing here requires structural reversal.

## Validation Snapshot

- [x] `cargo test --quiet` passed — 304 tests, 1 ignored, 0 failed
- [x] `make lint` clean
- [x] `src/` + `tests/` span a multi-language static-analysis pipeline (Go, Python, Rust targets)
- [x] `rules/registry.json` is generated from Rust-backed catalog, not manually maintained
- [x] Binding metadata is constant-backed via `file!()` macros

## Score Breakdown by Dimension

| Dimension | Score |
|---|---|
| Layering and separation of concerns | 9 / 10 |
| Rust idioms (traits, ownership, visibility) | 8 / 10 |
| Error handling | 9 / 10 |
| Testing infrastructure | 7 / 10 |
| Module boundary discipline | 6 / 10 |
| Extensibility | 7 / 10 |
| **Overall** | **7.5 / 10** |

---

## What Is Working Well

### 1. Layered pipeline — textbook clean

- [x] `src/lib.rs` documents the pipeline intent in the crate doc comment
- [x] Implementation follows the documented order: `analysis` (parse) → `index` (cross-file) → `heuristics` (rules) → `scan` (orchestrate) → `cli` (UX)
- [x] Each subsystem has a thin facade module (`scan/mod.rs`, `analysis/mod.rs`, `heuristics/mod.rs`) that pushes implementation into focused leaf modules
- [x] The pipeline direction is unidirectional — lower layers do not import from higher layers

### 2. Trait-based backend dispatch

- [x] `LanguageBackend` trait in `src/analysis/backend.rs` defines three clear contract points: `parse_file`, `evaluate_file`, `evaluate_repo`
- [x] `&'static dyn LanguageBackend` with static singletons avoids heap allocation on every dispatch
- [x] Adding a fourth language (e.g. TypeScript) requires implementing the trait and adding one entry to `registered_backends()` — the rest of the pipeline picks it up automatically
- [x] `backend_for_path` and `backend_for_language` provide two clean lookup axes

### 3. Rule dispatch via const function pointer arrays

- [x] Eight typed rule function-pointer aliases are defined in `src/heuristics/registry.rs`: `FileRule`, `FunctionRule`, `OptionalFunctionRule`, `IndexedFunctionRule`, `FileFunctionRule`, `ConfigurableFunctionRule`, `RepoRule`, `IndexedRepoRule`
- [x] Each alias enforces at compile time what context a rule function is allowed to receive
- [x] Rule arrays are `const` — zero-cost, zero-allocation, no runtime registration
- [x] Adding a new rule is a one-line addition to the appropriate const array

### 4. Error hierarchy

- [x] `thiserror` is used throughout with contextual path information on every variant
- [x] `#[source]` chaining preserves the full error cause chain
- [x] `pub(crate)` constructor methods (`Error::io`, `Error::walk`, etc.) keep call sites clean
- [x] No `unwrap()` or `expect()` in scan hot paths
- [x] `InputTooLarge`, `SymlinkRejected`, `PathOutsideRoot` errors indicate deliberate security boundary enforcement

### 5. Parallelism

- [x] `rayon::par_iter()` is used for both file parsing in `scan/file_analysis.rs` and finding evaluation in `scan/evaluate.rs`
- [x] Output is deterministically sorted after parallel collection to prevent non-deterministic report ordering
- [x] No async runtime — correct choice for a CPU-bound CLI tool; `async` would add executor overhead with no throughput benefit here

### 6. Module visibility discipline

- [x] `pub(crate)` and `pub(super)` are used throughout rather than blanket `pub`
- [x] The public API surface exposed from `src/lib.rs` is intentionally narrow and stable
- [x] Internal types in `analysis/`, `heuristics/`, `index/`, and `scan/` are correctly visibility-gated

### 7. Testing infrastructure

- [x] `tests/support/mod.rs` provides `FixtureWorkspace` backed by `tempfile::TempDir` (RAII cleanup)
- [x] `include_str!` macros in `tests/integration_scan/mod.rs` embed fixture content at compile time — no file I/O at test runtime
- [x] `assert_rules_present` and `assert_rules_absent` helpers prevent test repetition
- [x] Test strategy has healthy diversity: unit tests, parser tests, integration scan tests, CLI tests, corpus regression tests

### 8. Release profile

- [x] `lto = "fat"` and `codegen-units = 1` maximize cross-module optimization
- [x] `opt-level = "z"` targets binary size, appropriate for a distributed CLI tool
- [x] `panic = "abort"` removes unwinding machinery and reduces binary size
- [x] `overflow-checks = true` keeps checked arithmetic in release for a security-oriented tool
- [x] `strip = "symbols"` reduces distributed binary size

---

## Gaps and Action Items

### Gap 1 — `AnalysisConfig` leaks into the heuristics layer

**Problem:** `src/heuristics/engine.rs` accepts `AnalysisConfig` as a parameter solely to read a single boolean (`enable_go_semantic`). The heuristics layer should not need to know about scan-time configuration structs — that is a layer boundary violation. If the heuristics layer grows, this import becomes a maintenance drag.

**Impact:** Heuristics and scan config become coupled. A change to `AnalysisConfig` fields forces review of all heuristics call sites.

- [x] Extract `enable_go_semantic` out of `AnalysisConfig` and pass it as a plain `bool` to the configurable rule dispatch path
- [x] Remove the `AnalysisConfig` import from `src/heuristics/engine.rs`
- [x] Verify `GO_CONFIGURABLE_FUNCTION_RULES` still receives its flag through the new path
- [x] Add a compile-time check or doc comment on `AnalysisConfig` documenting that it must not be imported by `heuristics/`

### Gap 2 — Wide re-export surface in `analysis/mod.rs`

**Problem:** `src/analysis/mod.rs` re-exports 25+ types from the types submodule in a single `pub(crate) use types::{...}` block. Callers have no signal about which types belong to which concern — `GinCallSummary` and `UnsafePattern` appear to live in the same namespace with equal weight.

**Impact:** Contributors cannot discover the ownership of a type from its import path. Refactoring the types submodule requires auditing all call sites of the flat re-export.

- [x] Group re-exports in `analysis/mod.rs` by sub-namespace with inline comments marking language boundaries (Go types, Python types, Rust types, shared types)
- [x] Consider whether Go-specific types (`GinCallSummary`, `GoFieldSummary`, `GoFunctionEvidence`, etc.) should be accessible via `analysis::go::*` rather than the flat `analysis::*` surface
- [x] Add a test or lint that catches any new type being added to the flat re-export without a grouping comment

### Gap 3 — Dual dispatch path for shared vs. language-specific findings

**Problem:** `src/scan/evaluate.rs` calls `evaluate_shared_file()` directly from `heuristics/engine.rs`, then separately calls `backend.evaluate_file()` which also dispatches through the engine. Two code paths serve one job (per-file evaluation), and they are invoked from different call sites.

**Impact:** It is not obvious to a new contributor which path applies when, or what happens if shared-rule and backend-rule findings conflict. Adding a new evaluation phase requires updating both paths.

- [x] Document explicitly in `scan/evaluate.rs` why `evaluate_shared_file` is called separately from `backend.evaluate_file` (if there is a deliberate reason)
- [x] If there is no deliberate reason, consolidate by having each backend's `evaluate_file` call `evaluate_shared_file` internally and remove the direct call from `scan/evaluate.rs`
- [x] Add an integration test that verifies shared rules fire on all three languages (Go, Python, Rust) to guard against a regression if the dispatch is unified

### Gap 4 — Unconventional `#[path = "..."]` module declarations in integration tests

**Problem:** `tests/integration_scan/mod.rs` uses `#[path = "concurrency.rs"] mod concurrency;` for every submodule. Because the files are siblings in the same directory, standard `mod concurrency;` without a `#[path]` attribute would resolve identically. The attribute adds noise and will confuse contributors unfamiliar with this pattern.

**Impact:** Maintenance friction. Contributors adding a new test module must follow the non-standard pattern or discover that both approaches work, creating inconsistency.

- [x] Remove all `#[path = "..."]` attributes from `tests/integration_scan/mod.rs` that point to sibling files
- [x] Verify that `cargo test` still compiles and all integration scan tests pass after removal
- [x] Document in `tests/integration_scan/mod.rs` (or `guides/`) the reason for any `#[path]` usage that legitimately cannot be removed

### Gap 5 — `registered_backends()` returns a fixed-size array

**Problem:** `src/analysis/backend.rs` returns `[&'static dyn LanguageBackend; 3]`. The array size `3` is hardcoded in the function signature. Adding a fourth language backend requires changing the return type, which is a mechanical change that serves no design purpose.

**Impact:** Language addition touches the function signature unnecessarily. Callers that destructure the array by index would break silently at compile time in subtle ways.

- [x] Change `registered_backends()` to return `&'static [&'static dyn LanguageBackend]`
- [x] Replace the inline array literal with a `static` slice or use `once_cell`/`OnceLock` if needed
- [x] Verify all callers (in `scan/evaluate.rs`, `analysis/mod.rs`, etc.) still compile and behave correctly
- [x] Add a test that iterates all registered backends and asserts each has at least one supported extension

### Gap 6 — `proptest` dependency is present but underused in parser-critical paths

**Problem:** `proptest` is in `[dev-dependencies]` but is only exercised in `src/io.rs`. For a static-analysis tool whose correctness depends entirely on parser output, the parser modules are the highest-risk surface. A panic or incorrect parse result on adversarial input is a real attack vector.

**Impact:** The parser can silently accept malformed code and produce incorrect evidence, leading to false negatives. There are no invariant-level tests confirming "clean fixtures never produce findings."

- [x] Add a `proptest`-backed invariant test: given any valid Go/Python/Rust source string, `parse_source_file` must not panic
- [x] Add a fixture-level invariant: for every `*_clean.txt` fixture in `tests/fixtures/`, assert that scanning it produces zero findings for the rules it is meant to validate
- [x] Add a fixture-level invariant: for every `*_slop.txt` or `*_positive.txt` fixture, assert that scanning it produces at least one finding
- [x] Consider enabling the `cargo fuzz` targets in CI on a short time budget to surface parser panics on random inputs
- [x] Document which properties are considered invariants vs. which findings are expected to change as heuristics evolve

---

## What Should Not Be Changed

### Keep the const function pointer rule arrays

- [x] Do not replace `const fn` pointer arrays with a trait-object `Vec` or runtime-registered plugin system — the current approach is zero-cost, type-checked at compile time, and easy to audit
- [x] Do not add a macro to "register" rules automatically — explicit arrays are readable and searchable

### Keep `rayon` over async

- [x] Do not introduce `tokio` or `async-std` for parallelism — `rayon` is the correct choice for CPU-bound workloads, and the current parallel scan is already fast
- [x] Do not add an async interface to `scan_repository()` unless an embedding use case genuinely requires it

### Keep the explicit rule catalog

- [x] Do not replace the structured `RuleMetadata` catalog with convention-based auto-discovery — the explicit catalog is the source of truth for documentation, CLI listing, and configurability contracts
- [x] Do not merge `rules/` and `heuristics/` — rule metadata and rule implementation are different concerns

### Keep `panic = "abort"` and `overflow-checks = true` in the release profile

- [x] These are correct for a security-aware CLI tool — do not remove them to gain marginal performance
