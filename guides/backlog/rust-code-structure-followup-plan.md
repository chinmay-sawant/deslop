# Rust Code Structure Follow-Up Plan

Date: 2026-04-07

## Status After This Refactor

Current rating: 8.8/10

Why it improved:

- `src/analysis/rust/parser/items.rs` was split into focused parser item modules.
- `src/analysis/rust/findings.rs` was split into focused findings modules.
- `src/analysis/rust/evaluate.rs` now uses explicit rule tables instead of one large hard-coded block.
- Rust integration tests now lean more on the shared support harness instead of repeating long assertion chains inline.

Validation snapshot:

- `cargo fmt` passed
- `cargo test` passed
- `deslop scan . --json --no-fail` completed successfully
- Repo scan summary:
  - files discovered: 241
  - files analyzed: 240
  - functions found: 1833
  - findings: 168
  - total scan time: 753 ms

Why it is still not 10/10:

- Rust evaluation orchestration is cleaner, but still more bespoke than the Go/Python path.
- Rust rule metadata ownership is still split between `src/rules/catalog/rust/*` and heuristic-local definitions.
- Several Rust heuristic families are still large enough to create review drag.
- `src/analysis/rust/mod.rs` still carries a very large test block that should be split into dedicated test modules.
- The parser/item split reduced the biggest single hotspot, but `src/analysis/rust/findings/import_resolution.rs` is now the next concentration point.

## What Was Completed

### 1. Parser item decomposition

Completed:

- `src/analysis/rust/parser/items/mod.rs`
- `src/analysis/rust/parser/items/symbols.rs`
- `src/analysis/rust/parser/items/modules.rs`
- `src/analysis/rust/parser/items/pkg_strings.rs`
- `src/analysis/rust/parser/items/trait_impls.rs`
- `src/analysis/rust/parser/items/structs.rs`
- `src/analysis/rust/parser/items/statics.rs`
- `src/analysis/rust/parser/items/attributes.rs`
- `src/analysis/rust/parser/items/enums.rs`

Result:

- The old catch-all parser item file is gone.
- The parser now has clearer ownership per item family.

### 2. Findings decomposition

Completed:

- `src/analysis/rust/findings/mod.rs`
- `src/analysis/rust/findings/hygiene.rs`
- `src/analysis/rust/findings/docs.rs`
- `src/analysis/rust/findings/import_resolution.rs`
- `src/analysis/rust/findings/local_calls.rs`

Result:

- Hygiene, docs, import resolution, and local-call checks are no longer mixed in one file.

### 3. Rust evaluator cleanup

Completed:

- Introduced explicit non-test macro and call rule tables in `src/analysis/rust/evaluate.rs`
- Introduced grouped file-rule and function-rule tables in `src/analysis/rust/evaluate.rs`

Result:

- Rust evaluation is still custom, but it is easier to scan and extend than before.

### 4. Shared Rust test harness improvements

Completed:

- Strengthened shared assertion helpers in `tests/support/mod.rs`
- Moved more Rust integration assertions onto shared helpers
- Removed local duplicate rule-assert helpers from `tests/integration_scan/rust_advanced.rs`

Result:

- Rust integration tests are shorter and more uniform.

## Remaining Highest-Value Improvements

### 1. Unify Rust evaluation with the shared engine

Current gap:

- Rust still depends on `src/analysis/rust/evaluate.rs` as a special execution path, while Go and Python are more obviously routed through the generic heuristics engine and registry shape.

Next step:

- Introduce a Rust family registry that models file rules, function rules, indexed rules, and config-gated rules in one place.
- Either extend `src/heuristics/engine.rs` to support the Rust shape cleanly or add a shared execution abstraction used by all backends.

Success condition:

- Adding a new Rust family should not require hand-editing several independent execution sites.

### 2. Normalize Rust rule metadata ownership

Current gap:

- Metadata is still partly centralized and partly family-local.

Next step:

- Pick one ownership model and apply it everywhere.
- Recommended direction: let each Rust family own its metadata and let `src/rules/catalog/mod.rs` assemble the catalog mechanically.

Success condition:

- A Rust family owns both its runtime logic and its rule metadata contract.

### 3. Break up the next Rust hotspots

Highest-priority remaining files:

- `src/heuristics/rust/performance.rs`
- `src/heuristics/rust/security_footguns.rs`
- `src/heuristics/rust/api_design.rs`
- `src/analysis/rust/mod.rs`
- `src/analysis/rust/findings/import_resolution.rs`

Recommended split order:

1. `src/analysis/rust/mod.rs`
   - move large backend tests into `src/analysis/rust/tests/*.rs`
2. `src/analysis/rust/findings/import_resolution.rs`
   - split wildcard/import traversal from direct import matching
3. `src/heuristics/rust/performance.rs`
   - split async costs, container/layout checks, and path/text checks
4. `src/heuristics/rust/security_footguns.rs`
   - split manifest checks from source checks
5. `src/heuristics/rust/api_design.rs`
   - keep facade thin and move parsing helpers into local helper modules

### 4. Push more invariants onto shared support

Current gap:

- `tests/parser_invariants.rs` still has local temp-dir and scan helpers that overlap with `tests/support/mod.rs`.

Next step:

- Move invariant-friendly fixture setup into `tests/support/mod.rs`.
- Keep the invariant file focused on invariants, not workspace plumbing.

## Recommended Next PR Sequence

1. Split `src/analysis/rust/mod.rs` tests into dedicated files under a Rust test module tree.
2. Split `src/analysis/rust/findings/import_resolution.rs`.
3. Introduce a Rust family registry and shrink `src/analysis/rust/evaluate.rs` again.
4. Normalize Rust metadata ownership.
5. Split `src/heuristics/rust/performance.rs`.
6. Split `src/heuristics/rust/security_footguns.rs`.
7. Split `src/heuristics/rust/api_design.rs`.
8. Move parser invariant setup onto shared support.

## Bottom Line

This refactor materially improved the Rust structure and test ergonomics, but it did not finish the architecture journey.

The code is in better shape now because the parser item collectors and findings logic are no longer monolithic. The next real jump to 9+ comes from finishing the evaluator unification and metadata ownership cleanup, then attacking the remaining large Rust rule-family files.
