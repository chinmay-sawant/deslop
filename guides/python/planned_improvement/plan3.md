 # Type-aware Data Flow Analysis — Plan 3

 ## Checklist-based Plan

 Use this checklist to track actionable items derived from the full plan. The full original plan is preserved below in the "Original plan (unchanged)" section to ensure no details are lost.

 ### Top-level acceptance & rollout

 - [ ] Acceptance criteria
   - Type lattice implemented with unit tests and >= 90% coverage for core ops.
   - Local flow inference correctly infers types in the baseline fixtures (positive/negative tests pass).
   - Heuristics updated in sample cases show measurable reduction in high-confidence false positives (canary evaluation required).
   - Feature can be toggled via config/CLI and caches are invalidated when format changes.

 - [ ] Rollout plan
   - [ ] Feature gate: default OFF; opt-in for developers and CI.
   - [ ] Canary run: enable for a subset of rule packs; measure FP/FN changes and runtime overhead.
   - [ ] Gradual enable: after canary acceptance, default to ON for nightly runs and then for CI.

 ### Implementation checklist

 - [ ] Spike & prototype (1–3 days)
   - [ ] Implement a tiny prototype in `src/analysis/python/parser/tests.rs` or a sandbox file that infers types for simple assignments and returns.
   - [ ] Verify using `tests/fixtures/python/simple.txt`.

 - [ ] Type system and utilities (2–4 days)
   - [ ] Implement `Type` and `TypeSet` in `src/analysis/python/type_lattice.rs`.
   - [ ] Add unit tests for join/contains/pretty-print.

 - [ ] Local flow analysis (4–7 days)
   - [ ] Implement `src/analysis/python/typeflow.rs` with a visitor over the parser AST/IR used by `phase4`.
   - [ ] Handle assignments, name resolution, attribute access, index expressions, container literals, function defs, and returns.
   - [ ] Create a symbol table per-scope with shadowing rules.

 - [ ] Interprocedural summaries (4–8 days)
   - [ ] Generate `FunctionSummary` per-def: param types (if inferable), return `TypeSet`, and side-effect markers (globals mutated, module-level writes).
   - [ ] Make summaries serializable and cacheable (add `src/analysis/python/summaries.rs`).

 - [ ] Integration with heuristics (2–3 days)
   - [ ] Add read-only accessors in `src/heuristics/python/mod.rs` to query `TypeFacts`.
   - [ ] Update sample heuristics (e.g., `ai_smells.rs`, `structure.rs`) to consume the type facts.

 - [ ] Tests & fixtures (2–4 days)
   - [ ] Add targeted fixtures in `tests/fixtures/python/` for edge cases (containers, nested functions, closures, dynamic load).
   - [ ] Add unit tests in `src/analysis/python` and integration tests that exercise the pipeline end-to-end.

 - [ ] Performance, caching and tuning (3–7 days)
   - [ ] Add per-file cache and a configuration flag to enable/disable type-aware analysis.
   - [ ] Add benchmarks under `benchmark/` and test time/memory budgets; document expected runtimes.

 - [ ] Documentation & guide updates (1–2 days)
   - [ ] Update `guides/python` (this file) and add migration notes for heuristics maintainers.

 ### Files & integration checklist

 - [ ] Create `src/analysis/python/type_lattice.rs` — `Type`, `TypeSet`, join/meet operations.
 - [ ] Create `src/analysis/python/typeflow.rs` — main walker and flow engine: `analyze_file_types()`, `analyze_function()`.
 - [ ] Create `src/analysis/python/summaries.rs` — `FunctionSummary` serialization and caching helpers.
 - [ ] Edit `src/analysis/python/parser/phase4.rs` — call into the new `typeflow` stage and attach `TypeFacts` to the file analysis result.
 - [ ] Edit `src/heuristics/python/mod.rs` — add `TypeFacts` consumer helpers.

 ### Safety & correctness checklist (worst practices to avoid)

 - [ ] Conservative defaults: ensure dynamic constructs (`eval`, `exec`, `getattr`, etc.) produce `Unknown` rather than unsafe assumptions.
 - [ ] Track global/module mutations and monkeypatching as side-effects that invalidate assumptions.
 - [ ] Avoid brittle string-based type heuristics.
 - [ ] Add tests for corner-cases: `None`, `Optional[...]`, empty containers, comprehensions, nested scopes.
 - [ ] Prevent unbounded context sensitivity — prefer summary-based interprocedural analysis.
 - [ ] Maintain a stable read-only API to decouple heuristics from analysis internals.
 - [ ] Surface analysis errors instead of panicking; fall back to conservative results if needed.
 - [ ] Expose a feature-flag so slow analysis is not enabled by default.
 - [ ] Provide a migration/deprecation path for heuristics when summaries change.

 ### Best-practices checklist (current gaps to address)

 - [ ] Modular separation: ensure a clear API boundary for `TypeFacts` between parser and heuristics.
 - [ ] Conservative defaults: audit `src/heuristics/python/ai_smells.rs` for direct type assumptions.
 - [ ] Unit & integration tests: add targeted tests for lattice ops and flow transforms.
 - [ ] Benchmarks: add structured benchmark scripts under `benchmark/`.
 - [ ] Feature-flag rollout: add a CLI/config flag such as `--enable-typeflow`.
 - [ ] Versioned serialization for summaries: design a versioned summary format.
 - [ ] Documentation: update `guides/python` and add migration guidance.

 ### Testing & QA checklist

 - [ ] Unit tests
   - [ ] `type_lattice` join/meet/equality
   - [ ] `typeflow` local analysis for assignment patterns
 - [ ] Integration tests
   - [ ] Use `tests/fixtures/python/*` to exercise assignments, return types, container typing, attribute reads.
 - [ ] Edge-case tests
   - [ ] dynamic imports, `eval`, `exec`, decorator-modified functions, monkeypatch patterns.
 - [ ] Performance tests
   - [ ] Add a `bench_typeflow` benchmark to measure time and memory on large files and set acceptable budgets.
 - [ ] Fuzz tests / robustness
   - [ ] Add a quick fuzz harness to ensure the analyzer doesn't panic on unexpected AST shapes.

 ### Monitoring & metrics checklist

 - [ ] Analysis time per-file and total pipeline run time.
 - [ ] Memory usage during analysis.
 - [ ] Count of rules that used type facts and their FP/FN delta.
 - [ ] Cache hit rate and cache invalidation frequency.

 ### Maintenance & owners

 - [ ] Keep `FunctionSummary` formats versioned.
 - [ ] Document patterns that force conservative fallbacks.
 - [ ] Owners & rough estimates:
   - Design & prototype: 1 engineer, 5–10 days.
   - Full implementation + tests: 1–2 engineers, 3–6 weeks.
   - Rollout & monitoring: 1 engineer, 1–2 weeks.

 ### Next actionable choices (pick one or more)

 - [ ] Scaffold `src/analysis/python/type_lattice.rs` and `src/analysis/python/typeflow.rs` with minimal interfaces.
 - [ ] Add initial unit tests for lattice operations.
 - [ ] Update `src/analysis/python/parser/phase4.rs` to call a placeholder analysis function.

 ---

 ## Original plan (unchanged)

 This section preserves the original plan content verbatim to ensure no details were removed during conversion to checklist format.

 # Type-aware Data Flow Analysis — Plan 3

 ## Summary

 This document defines a pragmatic, implementation-focused plan to add a type-aware data-flow analysis for Python sources to the existing codebase. The goal is to support the current Python analysis logic while introducing precise type propagation (flow-sensitive, interprocedural summaries) that improves detection quality for heuristics and rules in `src/heuristics/python`.

 This plan covers design, implementation steps, testing, rollout, worst practices to avoid, and a checklist of best practices currently missing or partially followed in the repo.

 ## Why this change

 - Reduce false positives/negatives in Python rules by tracking types across assignments and calls.
 - Enable richer heuristics that rely on type information (e.g., taint sinks only apply to str, certain API misuse only for specific types).
 - Provide an extensible type-summary API for downstream heuristics.

 ## Scope

 - Input: Python source files already parsed by the existing pipeline.
 - Output: Per-file and per-function type summaries / flow facts that heuristics can query.
 - Non-goal: Full runtime type reconstruction of every dynamic runtime feature (we'll conservatively handle dynamic features).

 ## Relevant existing code (quick map)

 - Parser and analysis entrypoints:
   - `src/analysis/python/mod.rs`
   - `src/analysis/python/parser/mod.rs`
   - `src/analysis/python/parser/phase4.rs` (phase4 is already a pipeline stage)
   - `src/analysis/python/parser/general.rs`, `comments.rs`, `performance.rs`
 - Heuristics that will consume types:
   - `src/heuristics/python/mod.rs`
   - `src/heuristics/python/structure.rs`
   - `src/heuristics/python/performance.rs`
   - `src/heuristics/python/maintainability.rs`
   - `src/heuristics/python/duplication.rs`
   - `src/heuristics/python/ai_smells.rs`
 - Existing fixtures and test vectors:
   - `tests/fixtures/python/phase4_positive.txt`
   - `tests/fixtures/python/phase4_negative.txt`
   - `tests/fixtures/python/simple.txt`

 Use these as starting points for tests and integration.

 ## High-level design

 1. Representation: design a compact `Type` enum and a `TypeSet` (union/unknown/Any) that models common Python types: primitives (`int`, `str`, `bool`, `float`), container types (`list[T]`, `dict[K,V]`, `set[T]`, `tuple[...]`), `callable`, `module`, `None`, and `Unknown`.

 2. Lattice and merging rules: define lattice joins and subtype/is-a checks. Joins must be conservative and predictable.

 3. Local / intra-procedural analysis: implement a flow-sensitive walker that updates a symbol table (name -> TypeSet) while traversing a function body in control-flow order (a simple block-level ordering is acceptable to start; consider CFG later).

 4. Interprocedural summaries: produce compact function summaries capturing parameter type sets and return type sets. Use summary-based analysis to avoid unlimited blowup.

 5. Conservative handling of dynamic features: treat `eval`, `exec`, `getattr`, `setattr`, dynamic imports and reflection as sources of `Unknown`/taint; add optional heuristics to try best-effort resolution but never unsafely assume a concrete type.

 6. Integration contract: expose an API (Rust structs) such as `TypeFacts`, `FunctionSummary`, and `FlowQuery` that heuristics can use. Keep the API read-only for heuristics to avoid accidental mutation.

 7. Caching & incremental updates: store per-file analysis caches (hash keyed) so repeated runs don't re-analyze unchanged files.

 ## Concrete implementation files and placements

 Proposed new/edited files (suggested names):

 - Add: `src/analysis/python/type_lattice.rs` — `Type`, `TypeSet`, join/meet operations.
 - Add: `src/analysis/python/typeflow.rs` — main walker and flow engine: `analyze_file_types()`, `analyze_function()` and helpers.
 - Edit: `src/analysis/python/parser/phase4.rs` — call into the new `typeflow` stage at the end of phase4 (or add as phase5), and attach `TypeFacts` to the file analysis result.
 - Add: `src/analysis/python/summaries.rs` — `FunctionSummary` serialization and caching helpers.
 - Edit: `src/heuristics/python/mod.rs` — add a `TypeFacts` consumer helper and examples for heuristics to call `type_of(symbol)` or `types_at(node_range)`.

 Keep changes minimally invasive: add new modules and small integration points rather than rewriting existing parsers.

 ## Implementation steps (detailed)

 1) Spike & prototype (1–3 days)
   - Implement a tiny prototype in `src/analysis/python/parser/tests.rs` or a sandbox file that infers types for simple assignments and returns.
   - Verify using `tests/fixtures/python/simple.txt`.

 2) Type system and utilities (2–4 days)
   - Implement `Type` and `TypeSet` in `type_lattice.rs`.
   - Unit tests for join/contains/pretty-print.

 3) Local flow analysis (4–7 days)
   - Implement `typeflow.rs` with a visitor over the parser AST/IR used by `phase4`.
   - Handle assignments, name resolution, attribute access, index expressions, container literals, function defs, and returns.
   - Create a symbol table per-scope with shadowing rules.

 4) Interprocedural summaries (4–8 days)
   - Generate `FunctionSummary` per-def: param types (if inferable), return TypeSet, and side-effect markers (globals mutated, module-level writes).
   - Summaries should be serializable and cacheable.

 5) Integration with heuristics (2–3 days)
   - Add read-only accessors in `src/heuristics/python/mod.rs`.
   - Update a couple of high-impact heuristics to use the type facts (e.g., `ai_smells.rs`, `structure.rs`) as an example.

 6) Tests & fixtures (2–4 days)
   - Add targeted fixtures in `tests/fixtures/python/` for edge cases (containers, nested functions, closures, dynamic load).
   - Add unit tests in `src/analysis/python` and integration tests that execute the pipeline end-to-end.

 7) Performance, caching and tuning (3–7 days)
   - Add per-file cache and a configuration flag to enable/disable type-aware analysis.
   - Add benchmarks under `benchmark/` and test time/memory budgets; document expected runtimes.

 8) Documentation & guide updates (1–2 days)
   - Add `guides/python` edits (this file should be added to docs index) and a migration note for heuristics maintainers.

 9) Rollout (1–3 days)
   - Introduce a feature flag / config option; run canary on a subset of rule packs; monitor false positive changes.

 ## Worst practices to avoid (explicit)

 1. Failing to be conservative: do not assume concrete types for dynamic constructs — prefer `Unknown` instead of making unsafe assumptions.

 2. Global mutation blindness: do not ignore module/global assignments and monkeypatches; they should invalidate conservative assumptions or be tracked as side-effects.

 3. String-based heuristics for types: brittle string-matching (e.g., inferring `str` from variable naming) produces fragile rules.

 4. No tests for corner-cases: missing tests for `None`, `Optional[...]`, empty containers, comprehensions, and nested scopes.

 5. Unbounded context sensitivity: attempting naively to analyze every call context will blow up memory/time; prefer summaries.

 6. Tight coupling: changing heuristics' internals rather than adding a stable read-only API will make future maintenance harder.

 7. Silent failures: analysis errors must be surfaced and handled (no panics), with graceful fallback to a conservative unknown type.

 8. Performance ignorance: shipping slow analysis as default will slow CI and developer workflows — provide opting and budgets.

 9. No migration plan: updating heuristics without a compatibility or deprecation path will break downstream rule authors.

 ## Best-practices checklist (and checks for what is not followed today)

 - **Modular separation**: keep parser, type analysis, and heuristics decoupled. Current repo mostly follows this but needs a clear API boundary for `TypeFacts` (missing).
 - **Conservative defaults**: unknown/dynamic features handled conservatively — current heuristics sometimes assume types (spot any direct assumptions in `ai_smells.rs`).
 - **Unit and integration tests**: repo has fixtures; add targeted unit tests for lattice ops and flow transforms (currently limited).
 - **Benchmarks**: add explicit performance tests under `benchmark/` (currently `benchmark/` exists but add structured scripts).
 - **Feature-flag rollout**: add `--enable-typeflow` or a config flag to avoid breaking default runs (not present currently).
 - **Stable serialization for summaries**: design a versioned summary format so cached summaries survive minor changes (not present).
 - **Documentation**: update `guides/python/` with API expectations and migration notes (this file is part of that effort).

 ## Testing & QA plan

 - Unit tests
   - `type_lattice` join/meet/equality
   - `typeflow` local analysis for assignment patterns
 - Integration tests
   - Use `tests/fixtures/python/*` to exercise: assignments, return types, container typing, attribute reads.
 - Edge-cases
   - dynamic imports, `eval`, `exec`, decorator-modified functions, monkeypatch patterns.
 - Performance tests
   - Add a `bench_typeflow` benchmark to measure time and memory on large files; set acceptable budgets.
 - Fuzz / random AST generation
   - Add quick fuzz harness to ensure analyzer doesn't panic on unexpected AST shapes.

 ## Acceptance criteria

 - Type lattice implemented with unit tests and >= 90% coverage for core ops.
 - Local flow inference correctly infers types in the baseline fixtures (positive/negative tests pass).
 - Heuristics updated in sample cases show measurable reduction in high-confidence false positives (canary evaluation required).
 - Feature can be toggled via config/CLI and caches are invalidated when format changes.

 ## Rollout strategy

 1. Feature gate: default OFF; opt-in for developers and CI.
 2. Canary run: enable for a subset of rule packs; measure FP/FN changes and runtime overhead.
 3. Gradual enable: after canary acceptance, default to ON for nightly runs and then for CI.

 ## Metrics to monitor

 - Analysis time per-file and total pipeline run time.
 - Memory usage during analysis.
 - Count of rules that used type facts and their FP/FN delta.
 - Cache hit rate and cache invalidation frequency.

 ## Maintenance notes

 - Keep `FunctionSummary` formats versioned.
 - Document patterns that force conservative fallbacks so heuristic authors understand limits.

 ## Owners & rough estimates

 - Design & prototype: 1 engineer, 5–10 days.
 - Full implementation + tests: 1–2 engineers, 3–6 weeks.
 - Rollout & monitoring: 1 engineer, 1–2 weeks.

 ---

 If you want, I will:

 - scaffold the new files (`type_lattice.rs`, `typeflow.rs`) with minimal interfaces,
 - add initial unit tests for lattice operations, and
 - update `src/analysis/python/parser/phase4.rs` to call into a placeholder analysis function.

 Tell me which of these you want next and I will start the corresponding implementation step.

---

If you want, I will:

- scaffold the new files (`type_lattice.rs`, `typeflow.rs`) with minimal interfaces,
- add initial unit tests for lattice operations, and
- update `src/analysis/python/parser/phase4.rs` to call into a placeholder analysis function.

Tell me which of these you want next and I will start the corresponding implementation step.
