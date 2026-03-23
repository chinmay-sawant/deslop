# deslop Python Support Roadmap Index

## Purpose

This folder contains the implementation contract for adding Python repository support to deslop. The plan is intentionally shaped around the abstractions already present in the codebase instead of introducing a separate Python-only pipeline.

The target is to make Python a first-class scanned language in the same overall flow already used for Go and Rust:

- repository discovery through the existing walker
- parsing through a dedicated `LanguageBackend`
- normalized evidence through shared analysis types where that remains honest
- Python-specific findings layered on top of shared heuristics and language-local rules
- fixture-backed integration coverage and mixed-language verification

These documents are not a plan for linting deslop's own Python tooling. They are a plan for scanning external Python repositories and surfacing conservative, reviewable bad-pattern signals derived from the attached Python rule ideas in `python.md`.

## Current State Summary

Python support is now implemented through the existing architecture:

- `Language::Python` is registered in `src/analysis/mod.rs`.
- `src/analysis/python/parser/` now extracts Python imports, symbols, call sites, docstrings, test classification, loop concatenation evidence, and conservative exception-handler evidence.
- `src/heuristics/python/` now hosts the first Python-specific rule pack.
- The shared `ParsedFile` and `ParsedFunction` model carries the cross-language evidence Python currently needs.
- The repository-local index remains language-scoped, so mixed Go, Python, and Rust scans stay isolated by backend.

The current shipped Python rule pack is intentionally small and syntactic:

- `string_concat_in_loop`
- `blocking_sync_io_in_async`
- `exception_swallowed`
- `eval_exec_usage`
- `print_debugging_leftover`
- shared `full_dataset_load`

Python also reuses existing shared heuristics where the parser evidence supports them, including comment-style findings, hardcoded secret detection, and some naming and test-quality signals.

## Why This Roadmap Uses 3 Phases

Rust support needed a separate scaffold phase before parser enrichment. Python can move a little faster because the repository already has Python placeholder directories and the shared analysis model has already been generalized enough to host another language.

Because of that, the Python roadmap combines backend registration and first parser contract work into one phase, then splits the remaining work into:

1. backend plus parser plus evidence contract
2. heuristics and first Python rule pack
3. verification, mixed-language hardening, documentation sync, and rollout

This keeps the plan compact without collapsing important acceptance boundaries.

## Planned Code Layout

The first implementation pass should fill the existing placeholder directories instead of inventing a new structure:

- `src/analysis/python/mod.rs`
- `src/analysis/python/parser/mod.rs`
- `src/analysis/python/parser/comments.rs`
- `src/analysis/python/parser/general.rs`
- `src/analysis/python/parser/performance.rs`
- `src/analysis/python/parser/tests.rs`
- `src/heuristics/python/mod.rs`
- `src/heuristics/python/performance.rs`
- `src/heuristics/python/maintainability.rs`
- `src/heuristics/python/structure.rs`
- `tests/integration_scan/python.rs`
- `tests/fixtures/python/`

This is a recommended starting layout, not a mandate to create every file immediately. The guiding rule is to use the existing abstractions and keep Python code together under the already-created Python folders.

If the first parser implementation stays small, `src/analysis/python/parser/mod.rs` may temporarily host most of the logic. Split files only when the boundaries become useful.

## Detection Triage Policy

The attached `python.md` contains a broad list of possible Python bad patterns. Not all of them are equally suitable for a first static-analysis release.

The roadmap intentionally separates them into three buckets:

1. Good first-release candidates:

- string concatenation in loops
- sync I/O inside `async def`
- broad or swallowed exception handling
- `eval` and `exec`
- print-style debugging leftovers
- obvious full-dataset loading calls
- shared naming, secret, comment, and test-quality signals when parser evidence supports them

2. Later candidates after stable parser evidence exists:

- monolithic `__init__.py`
- god functions and oversized classes
- tight coupling and over-abstraction signals
- Python-local hallucinated import or local-call checks if module resolution becomes stable enough

3. Explicitly deferred for now:

- profiling-only performance claims
- duplication detection
- deep architectural judgments that need data flow or repository-wide semantic understanding
- "looks AI-generated" rules that cannot be defined with a low-noise evidence policy

The roadmap should not pretend that static analysis can prove runtime performance claims that the attached notes explicitly describe as profile-driven.

## How To Use These Phase Files

Every phase document in this folder should be treated as a merge-quality contract.

- A phase is not complete until its acceptance criteria are met.
- A phase is not complete until its verification section has either been executed or explicitly deferred with a reason.
- A phase is not complete until required docs are updated.
- If a later phase exposes a missing prerequisite, fix the earlier phase instead of patching around it.

## Required Section Contract

Each phase document in this folder must contain the following sections:

1. `Overview`
2. `In Scope`
3. `Out Of Scope`
4. `Dependencies`
5. `Primary Code Areas`
6. `Implementation Checkpoints`
7. `Acceptance Criteria`
8. `Verification`
9. `Document Update Obligations`
10. `Risks And Open Questions`

Checkpoints must describe observable implementation outcomes, not vague intentions.

## Phase Order

1. [Phase 1](phase-1.md): Python backend, parser, and evidence contract
2. [Phase 2](phase-2.md): Python heuristics and first rule pack
3. [Phase 3](phase-3.md): verification, mixed-language hardening, and rollout

## Current Completion State

- [Phase 1](phase-1.md): implemented
- [Phase 2](phase-2.md): implemented for the first Python rule pack
- [Phase 3](phase-3.md): implemented for the current rollout baseline

## Rollout Artifacts

- [benchmark-note.md](benchmark-note.md): current Python fixture benchmark snapshot
- [release-checklist.md](release-checklist.md): release validation checklist for Python support

## Parallel Work Policy

- Phase 1 must establish stable parser output before Phase 2 adds rules that depend on it.
- Phase 2 may draft fixture files while Phase 1 is finishing, but rule implementation should not depend on parser behavior that is still changing.
- Phase 3 can start drafting documentation and benchmark conventions early, but final sign-off depends on Phases 1 and 2 being complete.
- If a proposed Python heuristic needs parser evidence that does not yet exist, extend Phase 1 deliberately rather than smuggling parser work into Phase 2.

## Python-Specific Planning Rules

- Reuse shared analysis fields only when the concept is genuinely cross-language.
- Do not overfit `ParsedFunction` to Python if a Python-local helper structure is cleaner.
- Prefer enabling shared heuristics through better parser evidence rather than cloning those rules into `src/heuristics/python/`.
- Keep Python-specific rules in `src/heuristics/python/` instead of scattering them across unrelated Go-oriented modules.
- Preserve the language-scoped index behavior so mixed-language repositories do not merge Python symbols with Go or Rust symbols.

## Related Documents

- `python.md`
- `guides/implementation-guide.md`
- `guides/features-and-detections.md`