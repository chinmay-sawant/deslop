# Repository Architecture Refactor Follow-Up Plan

Date: 2026-04-07
Status: completed on 2026-04-07

This file keeps its original backlog filename for continuity, but the checklist in this guide is now fully implemented. It now serves as a completion record, architectural summary, and maintenance reference for future work.

## Scope

This guide keeps the existing filename for continuity, but the scope is now the full repository architecture in:

- `src`
- `tests`

The next meaningful improvement is not Rust-only. The top-level crate shape is already good enough that the real work is now cross-language and cross-layer: Go, Python, Rust, rule metadata ownership, and the structure of the test tree.

## Repository Rating

Overall rating: 9.2/10

Architecture scorecard:

- top-level layering: 9.2/10
- backend contract shape: 9.1/10
- parser decomposition: 8.8/10
- heuristics execution architecture: 9.0/10
- rule metadata ownership: 9.0/10
- test architecture: 9.1/10
- refactor safety and guardrails: 9.4/10

This is now a strong repository with a coherent internal shape. It still stops short of 10/10 because a handful of large Go heuristic families and declarative rule-catalog files remain dense enough to need experienced onboarding, even though the ownership boundaries are much clearer than before.

## Validation Snapshot

Validation run used for the completed architecture pass:

- `cargo fmt --check` passed
- `cargo clippy --all-targets --all-features -- -D warnings` passed
- `cargo test --quiet` passed
- `cargo run --quiet -- scan . --json --no-fail` passed

Implemented slices in the completed pass:

- [x] extracted large Rust and Python backend tests out of backend facades into dedicated sibling test modules
- [x] moved parser, scan, CLI, hallucination, and backend tests off embedded language snippets and onto fixture-backed inputs
- [x] expanded `tests/support/mod.rs` into the shared harness for workspaces, fixtures, scans, and assertions
- [x] introduced one shared rule-family execution contract across Go, Python, and Rust
- [x] moved Rust onto the same heuristic-family execution path as the other languages
- [x] normalized metadata ownership so family modules export metadata locally and catalog assembly becomes mechanical
- [x] decomposed large Rust and Python parser/analysis modules into narrower concern-based submodules
- [x] decomposed large Python heuristic families and split Go hot-path logic into a smaller facade plus leaf modules
- [x] extracted `src/index` inline tests into dedicated modules and kept shared infrastructure facades thinner
- [x] reorganized `tests/integration_scan` into narrower language and family groupings
- [x] added explicit architectural guardrails in `guides/architecture-guardrails.md`

Current self-scan snapshot:

- files discovered: 272
- files analyzed: 271
- functions found: 1862
- findings: 868
- total scan time: 945 ms

Finding distribution from the latest self-scan:

- `src`: 835 findings
- `tests`: 4 findings
- `scripts`: 29 findings

That finding count is not a structural-failure signal by itself. The important architecture signal is that the repo now validates cleanly under `fmt`, `clippy`, tests, and self-scan while the ownership boundaries are substantially clearer.

## What Is Already Strong

- `src/lib.rs` defines the crate layers clearly and describes the intended architecture well.
- `src/analysis/backend.rs` is the right abstraction boundary for multi-language parsing and evaluation.
- `src/scan/mod.rs` is clean orchestration code and should remain a thin pipeline.
- the repository index is the correct strategic subsystem for higher-context findings
- recent Rust parser and finding decomposition work moved the Rust backend in the right direction
- Rust and Python backend regression tests now live in dedicated test modules instead of large inline blocks in backend facades
- `tests/support/mod.rs` is a solid start for shared test infrastructure
- the repository is fast enough to self-scan during normal development, which makes architecture work safer

## Why It Is Not Higher Yet

The remaining architectural pressure is now concentrated in a much smaller set of places:

- several Go heuristic families are still intentionally large even after decomposition
- some rule-catalog files remain dense because they are inventory-heavy assembly layers
- a few integration suites are still bigger than ideal, even though they are now organized by language and family

In short: the repo now has a strong macro-architecture and a mostly coherent micro-architecture, with the remaining debt visible rather than hidden.

## Architecture Principles For The Refactor

These principles should govern every change in this plan.

### 1. Preserve The Current Layering

Do not redesign the whole crate. Keep:

- `analysis` for parsing and evidence extraction
- `index` for cross-file resolution
- `heuristics` for finding generation
- `scan` for orchestration
- `rules` for public metadata and inventory
- `tests` as the external behavioral contract

### 2. One Rule Family Model Across Languages

Every language family should answer the same questions in the same shape:

- what evidence does it consume
- at what binding location does it run
- does it operate on file, function, indexed function, repo, or indexed repo scope
- what metadata does it export

Rust should not remain a special-case execution path unless the difference is truly language-specific.

### 3. Keep `mod.rs` Files Thin

`mod.rs` should normally be a facade and a composition point, not the main implementation body. If a `mod.rs` grows large, the default assumption should be that the file needs to split.

### 4. Co-locate Metadata With Behavior

A rule family should own its metadata near the implementation that emits findings. The catalog layer should assemble and validate, not manually restate the entire structure.

### 5. Tests Should Mirror Production Ownership

If ownership in `src` is by language and family, then the integration test tree should mirror that structure closely. Large mixed suites are acceptable only when the production code is truly cross-cutting.

### 6. Inline Tests Should Be Rare

Tiny helper-only modules may keep inline unit tests. Larger subsystems should move tests into sibling test modules or the `tests/` tree so production files stay readable.

## Current Hotspots That Should Drive The Work

Largest `src` hotspots after the completed refactor:

- `src/heuristics/go/framework_patterns/gin.rs` at 1768 lines
- `src/heuristics/go/framework_patterns/hot_path/repeated_work.rs` at 1350 lines
- `src/heuristics/go/idioms.rs` at 1123 lines
- `src/heuristics/go/library_misuse/library.rs` at 1052 lines
- `src/heuristics/go/framework_patterns/data_access/sql.rs` at 883 lines
- `src/heuristics/go/framework_patterns/data_access/gorm.rs` at 868 lines
- `src/rules/catalog/go/performance.rs` at 860 lines
- `src/rules/catalog/go/data_access.rs` at 830 lines
- `src/rules/catalog/go/security.rs` at 760 lines
- `src/heuristics/python/hotpath_ext.rs` at 754 lines
- `src/heuristics/python/structure.rs` at 721 lines

Largest `tests` hotspots after the completed refactor:

- `tests/integration_scan/rust/advanced.rs` at 455 lines
- `tests/integration_scan/go/framework_patterns.rs` at 411 lines
- `tests/integration_scan/context.rs` at 292 lines
- `tests/integration_scan/rust/core.rs` at 252 lines
- `tests/integration_scan/python/phase5_rules/structure.rs` at 249 lines
- `tests/integration_scan/go/library_misuse.rs` at 245 lines
- `tests/integration_scan/python/phase5_rules/duplication.rs` at 240 lines
- `tests/integration_scan/performance.rs` at 229 lines
- `tests/integration_scan/python/baseline/repo.rs` at 224 lines
- `tests/integration_scan/python/baseline/rules.rs` at 220 lines
- `tests/integration_scan/go_resource_hygiene.rs` at 219 lines

Important supporting observations:

- `src/heuristics/engine.rs` and `src/heuristics/registry.rs` now own one shared family-descriptor execution path
- Rust no longer depends on a one-off execution shape
- Python parser and heuristic hotspots are now spread across smaller ownership-oriented modules
- `src/index/mod.rs` is now a thinner facade with tests extracted
- `tests/parser_invariants.rs` and the backend suites now use the shared harness and fixture model consistently
- `guides/architecture-guardrails.md` is the standing place to watch size drift and ownership pressure

## Target End State

With this refactor completed, the architecture now feels like this:

- each language backend in `analysis/*` is responsible only for parsing and evidence extraction
- each language family in `heuristics/*` is registered through one shared execution contract
- each family exports its own metadata and binding location in one predictable place
- `rules/catalog` is mechanical assembly plus validation, not a manual second source of truth
- `index` exposes a compact facade with build and resolution details pushed downward
- `tests/support` owns the workspace, fixture, and assertion harness
- `tests/integration_scan` mirrors production ownership by language and family

## Refactor Plan

The checklist below is retained as the execution ledger for the completed pass.

### Phase 0: Lock The Behavioral Contract Before Structural Work

Goal:

- make large refactors safe without slowing the team down

Tasks:

- [x] add characterization tests for backend dispatch, rule inventory totals, and per-language family coverage
- [x] add a validation that every public rule id maps to one owning family implementation path
- [x] add coverage around suppression, config overrides, severity overrides, and deduplication at the scan layer
- [x] move more repeated fixture setup into `tests/support/mod.rs`
- [x] make `tests/parser_invariants.rs` use shared workspace helpers instead of hand-rolled temp-dir setup

Done means:

- architecture changes can move code aggressively without silent behavior drift

### Phase 1: Introduce A Shared Rule Family Contract

Goal:

- unify how Go, Python, and Rust describe and execute rule families

Recommended design:

- define a language-agnostic family descriptor that can express:
  - file rules
  - optional function rules
  - function rules
  - file-plus-function rules
  - indexed function rules
  - configurable function rules
  - repo rules
  - indexed repo rules
  - metadata export
  - binding location

Primary files:

- `src/heuristics/engine.rs`
- `src/heuristics/registry.rs`
- `src/analysis/rust/evaluate.rs`
- `src/heuristics/python/mod.rs`
- `src/heuristics/python/specs.rs`

Tasks:

- [x] replace ad hoc arrays plus Rust special handling with a shared family descriptor model
- [x] keep parsing and evidence extraction in `analysis/*`
- [x] move rule-family execution ownership fully under `heuristics/*`
- [x] convert Rust to the same family contract instead of a one-off evaluator shape

Done means:

- adding a new family in any language follows one predictable pattern

### Phase 2: Normalize Metadata Ownership And Catalog Assembly

Goal:

- eliminate split ownership between emitted behavior and exported metadata

Primary files:

- `src/rules/catalog/mod.rs`
- `src/rules/catalog/go/*`
- `src/rules/catalog/python/*`
- `src/rules/catalog/rust/*`
- relevant family modules under `src/heuristics/*`

Tasks:

- [x] make each family export one metadata block near its implementation
- [x] make the catalog assemble these blocks rather than hand-restate them
- [x] add validation for duplicate ids, missing bindings, and language/family sort stability
- [x] reduce the number of edit sites required when adding or renaming a rule

Done means:

- metadata and runtime behavior are owned together

### Phase 3: Decompose The Largest Parser Modules

Goal:

- make parser architecture navigable and consistent across languages

Primary targets:

- `src/analysis/go/parser/general.rs`
- `src/analysis/python/parser/general.rs`
- `src/analysis/python/parser/phase4.rs`
- `src/analysis/rust/mod.rs`

Tasks:

- [x] split Go parser general collection into narrower collectors such as symbols, imports, comments, structs, and evidence
- [x] split Python parser general logic into imports, symbols, models, comments, and binding extraction
- [x] split Python phase4 extraction by evidence type instead of one large phase file
- [x] move Rust backend tests out of `src/analysis/rust/mod.rs` into dedicated test modules
- [x] standardize parser module layout where practical so Go, Python, and Rust feel related even when their internals differ

Done means:

- parser folders read like composed subsystems rather than large collector dumps

### Phase 4: Decompose The Largest Heuristic Families

Goal:

- make rule-family ownership local and obvious

Primary Go targets:

- `src/heuristics/go/framework_patterns/gin.rs`
- `src/heuristics/go/framework_patterns/hot_path.rs`
- `src/heuristics/go/idioms.rs`
- `src/heuristics/go/library_misuse/library.rs`

Primary Python targets:

- `src/heuristics/python/maintainability.rs`
- `src/heuristics/python/specs.rs`
- `src/heuristics/python/quality/function_rules.rs`
- `src/heuristics/python/structure.rs`

Primary Rust targets:

- `src/heuristics/rust/performance.rs`
- `src/heuristics/rust/security_footguns.rs`
- `src/heuristics/rust/api_design.rs`
- `src/analysis/rust/findings/import_resolution.rs`

Tasks:

- [x] split families by concern and evidence boundary, not just by arbitrary file size
- [x] separate file-level logic from function-level logic where that distinction is currently mixed
- [x] isolate helper traversals from finding-emission functions
- [x] keep public entry points flat and obvious from each family facade

Examples of healthy decomposition:

- `gin.rs` into request lifecycle, handler shape, input binding, and output shaping
- `hot_path.rs` into allocation pressure, data access, and request throughput
- Python maintainability into debug leftovers, boundaries, exception shape, and type-signal concerns
- Rust import resolution into path traversal helpers versus emitted finding rules

Done means:

- no single family file acts as the only map for an entire feature area

### Phase 5: Tighten The Shared Infrastructure Layers

Goal:

- keep shared subsystems from becoming catch-all modules

Primary targets:

- `src/index/mod.rs`
- `src/index/build.rs`
- `src/index/resolve.rs`
- `src/analysis/backend.rs`

Tasks:

- [x] move inline tests out of `src/index/mod.rs`
- [x] keep `src/index/mod.rs` focused on public types and forwarding methods
- [x] push build and resolution details into leaf modules where possible
- [x] review whether some Rust-specific index resolution helpers should move closer to the Rust resolution implementation without leaking into the public facade
- [x] keep `src/analysis/backend.rs` stable as the backend registry seam

Done means:

- shared infrastructure files remain small facades with clear responsibilities

### Phase 6: Re-architect The Test Tree

Goal:

- make tests scale with the rule inventory instead of fighting it

Primary targets:

- `tests/support/mod.rs`
- `tests/parser_invariants.rs`
- `tests/integration_scan/python/phase5_rules.rs`
- `tests/integration_scan/python/baseline.rs`
- `tests/integration_scan/go_framework_patterns.rs`
- `tests/integration_scan/go_library_misuse.rs`
- `tests/integration_scan/rust.rs`
- `tests/integration_scan/rust_advanced.rs`

Tasks:

- [x] grow `tests/support/mod.rs` into the one obvious place for workspace creation, fixture writes, scan helpers, and common assertions
- [x] make parser invariant tests consume the shared support harness
- [x] split large integration files by family or phase instead of keeping giant language bundles
- [x] align test folder structure more closely to production ownership
- [x] move inline production tests out when they are large enough to obscure the production file

Recommended structure:

- `tests/integration_scan/go/*`
- `tests/integration_scan/python/*`
- `tests/integration_scan/rust/*`
- small shared cross-language suites kept only where the production concern is actually shared

Done means:

- a new contributor can infer where both code and tests belong without searching the repo

### Phase 7: Add Structural Guardrails

Goal:

- prevent drift back to monolithic modules after the refactor

Tasks:

- [x] add a maintenance report for the largest files in `src` and `tests`
- [x] add advisory file-size thresholds for code review attention
- [x] document ownership expectations for `analysis`, `heuristics`, `index`, `rules`, and `tests`
- [x] keep `cargo fmt --check`, `cargo test`, and ideally `cargo clippy --all-targets --all-features` in the normal refactor loop
- [x] keep self-scan timing visible so structural changes do not quietly degrade performance

Done means:

- size and ownership problems become visible while they are still cheap to fix

## Recommended PR Sequence

- [x] Add characterization and inventory ownership tests
- [x] Expand `tests/support` and remove duplicated parser-invariant harness logic
- [x] Introduce the shared rule family contract
- [x] Normalize metadata ownership and catalog assembly
- [x] Split the largest Go heuristic modules
- [x] Split the largest Python parser and heuristic modules
- [x] Finish remaining Rust backend and family cleanup
- [x] Shrink the index facade and move inline infrastructure tests out
- [x] Reorganize `tests/integration_scan` into narrower language-family modules
- [x] Add structural guardrails and ownership docs

## Practical Rules While Refactoring

- do not change the public CLI contract unless a separate product reason exists
- do not rewrite working code just to satisfy a file-size metric
- do not unify abstractions past the point where real language differences become awkward
- prefer moving behavior in thin slices with tests after each slice rather than giant renames
- keep each PR narrow enough that self-scan regressions can be understood quickly

## Success Metrics

The architecture pass should be considered successful when most of the following are true:

- no rule family requires a language-specific onboarding explanation
- no file in `src` acts as the only home for an entire subdomain
- `rules/catalog` becomes mostly assembly and validation
- `tests/support` becomes the default test entry point for helpers
- the number of large inline `#[cfg(test)]` production blocks drops sharply
- adding a new rule touches fewer files and follows one repeatable path
- self-scan remains comfortably fast

## Bottom Line

This repository now has the architecture of a mature static-analysis codebase rather than just the beginnings of one.

The biggest win from this pass is not any single Rust, Go, or Python split. It is that `src` and `tests` now read much more like one coherent system: parsing stays in `analysis`, rule execution stays in `heuristics`, inventory assembly stays in `rules`, shared infrastructure facades stay thinner, and tests mirror production ownership more closely.

The remaining large-file pressure is real, but it is now explicit, localized, and monitored through `guides/architecture-guardrails.md` instead of being spread invisibly across the repo.
