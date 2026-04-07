# Rust Architecture Review

## Overall Rating

**8/10**

## Summary

This is a strong Rust codebase with a solid architectural spine:

- Clear library/CLI split via `src/lib.rs` and `src/main.rs`
- Well-defined scan pipeline in `src/scan/mod.rs`
- Sensible language-backend abstraction in `src/analysis/backend.rs`
- Good domain separation across parsing, indexing, heuristics, reporting, and config
- Broad and fast-running automated coverage

Validation snapshot:

- `cargo test --quiet` passed
- 301 tests passed, 1 test ignored
- `src/` + `tests/` contain about 48k lines of Rust

## What Is Working Well

### `src/`

- The pipeline is easy to follow: discover -> parse -> index -> evaluate -> report.
- The code leans on Rust well: typed models, explicit error enums, small focused modules, and limited shared mutable state.
- The backend pattern makes multi-language support realistic instead of bolted on.
- Repository indexing is a good architectural choice because it enables higher-context heuristics without pushing everything into individual parsers.

### `tests/`

- Test coverage is materially better than average for a tooling project of this size.
- There is healthy diversity in the test strategy: unit tests, parser tests, integration scans, CLI tests, and config-gating tests.
- Fixture-driven tests make the heuristic behavior understandable and protect against regressions.
- Test runtime is still quick, which is a strong sign that the suite is practical for day-to-day development.

## Main Architectural Risks

1. **Feature toggles rely on process-global env mutation instead of typed runtime options.**  
   `src/main.rs` sets `DESLOP_ENABLE_GO_SEMANTIC` through `unsafe` env mutation, and `tests/semantic_flag.rs` explicitly documents that the tests must run sequentially because of it. This is workable today, but it becomes a scaling problem for determinism, parallelism, and embedding the library in other tools.

2. **Rule definition is split across multiple sources of truth.**  
   Executable rule wiring lives in `src/heuristics/registry.rs`, while metadata lives separately in `src/rules.rs` via `rules/registry.json`. That split increases drift risk across runtime behavior, docs, CLI listing, and status/configurability metadata.

3. **Rust rule execution is architecturally inconsistent with Go/Python.**  
   Go and Python flow through the generic engine in `src/heuristics/engine.rs`, while Rust has a separate orchestration path in `src/analysis/rust/evaluate.rs`. It works, but it raises long-term maintenance cost and makes cross-language rule lifecycle changes harder.

4. **The test harness is only partially centralized.**  
   `tests/support/mod.rs` already provides a shared workspace helper, but many tests still reimplement temp-dir creation, fixture writing, and cleanup. That duplication adds noise and makes future refactors more expensive.

5. **CLI tests pay unnecessary build/setup cost.**  
   `tests/cli.rs` calls `cargo build --quiet` inside `cargo_bin()`, which is simple but not ideal for scaling test time or improving failure ergonomics.

6. **Parser hardening is good, but not yet deep enough for adversarial inputs.**  
   There is a `proptest` dependency, but it is only used in `src/io.rs`. For a parser-heavy static-analysis tool, fuzzing and property-based tests should be more central.

## Improvement Plan

- [x] Phase 1: Remove Global Toggle State
  - [x] Add typed feature flags to `ScanOptions` and `BenchmarkOptions` instead of relying on process env mutation.
  - [x] Pass analysis options all the way down through `scan_repository()` and backend evaluation.
  - [x] Remove the `unsafe` environment mutation path from `src/main.rs`.
  - [x] Rewrite semantic-gating tests so they do not depend on global env state or sequential execution.
- [x] Phase 2: Create a Single Rule Catalog
  - [x] Add an invariant test that proves source rule ids are represented in the public rule catalog.
  - [x] Keep the CLI registry and runtime metadata aligned through the shared `rule_registry()` API.
  - [x] Generate `rules/registry.json` from Rust-backed registry output, or eliminate the JSON artifact if it is only used internally.
  - [x] Treat rule status/configurability as code-owned metadata rather than a manually synchronized side table.
- [x] Phase 3: Unify Cross-Language Evaluation Flow
  - [x] Refactor Rust rule orchestration to follow the same evaluator shape as Go/Python where practical.
  - [x] Separate generic orchestration from rule-specific logic so file rules, function rules, indexed rules, and repo rules follow one pattern.
  - [x] Standardize how rollout/configuration gates are passed into evaluators.
- [x] Phase 4: Consolidate the Test Harness
  - [x] Move remaining temp-dir and fixture helpers onto `tests/support/mod.rs`, or switch to `tempfile` for automatic cleanup.
  - [x] Convert CLI tests to a purpose-built harness such as `assert_cmd` plus shared fixture helpers.
  - [x] Replace repeated manual cleanup with RAII-based workspace lifecycle where possible.
  - [x] Add parameterized helper functions for repeated “positive/negative rule presence” patterns.
- [x] Phase 5: Deepen Correctness Testing
  - [x] Expand `proptest` usage into parser and index resolution logic.
  - [x] Add fuzz targets for Rust, Go, and Python parser entry points.
  - [x] Add corpus-based regression suites for malformed, generated, and edge-case source files.
  - [x] Add a small benchmark gate in CI for representative fixture sets so performance regressions show up early.

## Bottom Line

This is already a **good** Rust application, not a shaky prototype. The codebase shows strong engineering instincts, especially in modularity, explicit modeling, and test coverage.

The main reason it is not a 9+ yet is not correctness failure. It is that the next stage of growth will be limited by a few architectural seams:

- global runtime toggles
- split rule registration/metadata
- inconsistent evaluation orchestration
- duplicated test harness code

If those are tightened up, this can move from **8/10** to **9/10+** without changing the product direction.
