# Rust Architecture Review

## Scope

This review covers the current Rust-facing architecture in `src/` and `tests/`, with emphasis on backend layering, rule-system ownership, parser/evaluator decomposition, and test-suite maintainability.

## Rating

- Overall project structure: **9.2/10**
- `src/`: **9.4/10**
- `tests/`: **8.8/10**

## Executive Summary

The project has improved beyond the earlier **9/10** baseline.

The most important architectural concerns from the previous reviews have been addressed in substance:

- the crate now documents its intended dependency flow in `src/lib.rs`
- `src/analysis/`, `src/scan/`, and `src/heuristics/` have cleaner facade boundaries
- Rust evaluation now lives behind a thin backend facade in `src/analysis/rust/mod.rs`
- the Rust parser facade is small and orchestration-focused
- the old process-global toggle problem has been removed in favor of typed runtime flow through `scan_repository_with_go_semantic(...)`
- rule metadata is now Rust-owned and `scripts/sync_docs.py` derives `rules/registry.json` from the runtime registry instead of treating JSON as the primary source of truth
- the test suite now centralizes temp-dir cleanup in `tests/support/mod.rs`

This is no longer a codebase with major structural uncertainty. The remaining gap to a 10/10 is concentrated in a few hotspots, not in the overall architecture.

## Validation Snapshot

- `cargo test --quiet` passed locally during this review
- test result: **302 passed, 1 ignored, 0 failed**
- `src/rules/catalog.rs` contains **611** rule definitions in one file
- `tests/` still contains **138** direct scan invocations
- manual `remove_dir_all(...)` cleanup outside `tests/support/mod.rs`: **0**

## What Improved Since Review 2

### 1. The crate shape is now explicit and easier to preserve

`src/lib.rs` now documents the intended dependency direction across `analysis`, `index`, `heuristics`, `scan`, and `cli`. That matters because it turns the architecture from an implied convention into an explicit contract.

### 2. The root orchestration seams were split in the right places

The code now has better ownership boundaries in the layers that matter most:

- `src/analysis/mod.rs` is a clean facade over backend/config/types
- `src/scan/mod.rs` delegates to `evaluate`, `file_analysis`, `reporting`, `suppression`, and `walker`
- `src/heuristics/mod.rs` is a narrow facade over engine plus language families

This is the kind of decomposition that lowers review cost without changing product behavior.

### 3. Rust-specific flow is cleaner than before

The Rust backend is now easier to follow:

- `src/analysis/rust/mod.rs` is thin
- `src/analysis/rust/evaluate.rs` is focused on orchestration
- `src/analysis/rust/parser/mod.rs` is now a coordinator instead of a catch-all parser file

That is a real improvement over the earlier review baseline.

### 4. Rule ownership is substantially better

Earlier drift risk around metadata versus generated docs is much lower now.

`src/rules/catalog.rs` is the code-owned catalog, `src/rules.rs` exposes the runtime registry API, and `scripts/sync_docs.py` regenerates `rules/registry.json` by calling the binary with `rules --json`.

That is materially better than a hand-maintained JSON-first model.

### 5. The test harness direction is now correct

`tests/support/mod.rs` gives the suite a real shared abstraction with RAII cleanup. The old cleanup inconsistency is mostly gone, and that removes one of the more obvious sources of incidental noise in the test tree.

## Remaining Findings

### 1. The rule catalog has become a monolith

`src/rules/catalog.rs` is now the single largest architectural hotspot in the Rust backend.

- one file
- 8,584 lines
- 611 `RuleDefinition` entries

This is better than split sources of truth, but it creates a new problem: ownership and change locality have collapsed into a single giant file. That increases merge pressure, makes rule reviews noisier, and turns a valid architectural idea into a maintenance bottleneck.

**Why this matters:** the project has solved rule drift, but it has done so by centralizing too much rule inventory into one place.

**Recommended improvement:** split the catalog by language and rule family, then expose a generated or composed `RULE_CATALOG` from a thin root module.

### 2. Rust parser evidence collection is still over-concentrated

The old parser facade problem is fixed, but the complexity has largely moved into `src/analysis/rust/parser/functions.rs`.

That file currently mixes:

- function discovery
- call extraction
- literal extraction
- doc-comment extraction
- async evidence collection
- unsafe evidence collection
- loop and allocation evidence collection
- runtime-pattern evidence collection

This is a much better failure mode than a giant `mod.rs`, but it is still too much responsibility in one implementation file.

**Why this matters:** contributors working on async, unsafe, or performance evidence all have to edit the same parser file.

**Recommended improvement:** split `functions.rs` into smaller evidence-focused modules, such as `calls.rs`, `async_patterns.rs`, `unsafe_patterns.rs`, and `literals.rs`, while keeping one orchestration entrypoint.

### 3. Test support exists, but test ergonomics are not yet fully centralized

`tests/support/mod.rs` is the right abstraction, but many test modules still create a workspace and then call `scan_repository(&ScanOptions { ... })` directly.

There are still **138** direct scan invocations under `tests/`.

That means the project has centralized lifecycle cleanup, but not the full test workflow.

**Why this matters:** scan invocation boilerplate is still repeated across integration modules, and CLI or semantic-toggle tests still carry small custom wrappers that should live in shared support code.

**Recommended improvement:** extend `FixtureWorkspace` with helpers such as:

- `scan_report()` or equivalent naming to eliminate repeated `ScanOptions` construction
- a `scan_with_go_semantic(...)` helper for toggle coverage
- a lightweight CLI helper for command execution and common stdout/stderr assertions

### 4. Integration test topology is still harder to navigate than necessary

`tests/integration_scan.rs` remains a large dispatcher that pulls many modules in via `#[path = ...]` declarations.

It works, but it still adds navigation friction:

- the real test ownership is distributed
- module discovery is less direct than first-class submodules
- helper boundaries are harder to infer from the filesystem alone

**Why this matters:** the production crate now reads more cleanly than parts of the integration suite.

**Recommended improvement:** continue reorganizing integration tests into clearer module trees, or promote larger test families into their own integration crate entrypoints where that improves discoverability.

## What Keeps This From 10/10

The gap is no longer about architecture direction. It is about concentration and ergonomics.

The codebase is already well layered. The remaining work is to prevent a few successful abstractions from turning into new bottlenecks:

1. split the monolithic rule catalog
2. split Rust function-evidence extraction by concern
3. finish centralizing test scan and CLI helpers
4. simplify integration test module discovery

## Bottom Line

The current codebase is stronger than the earlier **9/10** review baseline.

My updated rating is **9.2/10**.

That increase is justified because the important architectural seams identified earlier were addressed at the root level:

- cleaner facades
- better scan and analysis ownership
- a real Rust-owned rule registry flow
- removal of the global toggle smell
- improved shared test infrastructure

The remaining issues are real, but they are concentrated and tractable. If the team breaks up the monolithic rule catalog and finishes the test-harness consolidation, this can credibly move into **9.5+** territory without a broader redesign.
