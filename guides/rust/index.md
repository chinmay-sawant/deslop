# deslop Rust Support Roadmap Index

## Purpose

This folder contains execution-focused handoff documents for adding Rust language support to deslop. These files are not about hardening deslop's own Rust implementation. They are a scoped implementation plan for scanning Rust repositories and detecting conservative, reviewable Rust bad-practice signals in the same spirit as the existing Go support.

Use this index together with `guides/implementation-guide.md` and `guides/features-and-detections.md`.

## Current state summary

The current repository already has some pieces that make Rust support feasible without a large rewrite:

- The scanner, index, benchmark, and report pipeline are already implemented as reusable Rust modules behind `src/lib.rs`.
- The analysis layer already has a backend abstraction in `src/analysis/mod.rs`.
- `Language::Rust` already exists as a placeholder in the internal language enum, but no Rust backend is registered yet.
- The current production implementation is Go-only. `src/analysis/go/...` is the only active language backend.
- The repository already has integration-test infrastructure and language-scoped fixture directories, which can be extended for Rust coverage.

Because of that, this roadmap should add Rust support as a new backend rather than rewrite the existing scanner architecture.

## File-count policy

This roadmap intentionally uses fewer active phase files so implementation can begin quickly without spreading work across too many documents.

- Active implementation phases: 4
- Default backend file layout for the first implementation pass:
  - `src/analysis/rust/mod.rs`
  - `src/analysis/rust/parser.rs`
  - `src/analysis/rust/fingerprint.rs` only if the parser file becomes too large or fingerprint logic genuinely needs separation
  - `tests/integration_scan/rust.rs`
  - `tests/fixtures/rust/`
- Do not mirror the full Go backend directory structure for Rust on day one unless the Rust backend actually grows large enough to need it.

## How to use these phase files

Every phase document in this folder must be treated as an implementation contract.

- No phase is complete until its acceptance criteria are satisfied.
- No phase is complete until its verification steps have been run or explicitly deferred with a reason.
- No phase is complete until the required documentation updates have been made.
- If a phase reveals missing groundwork from an earlier phase, fix the earlier phase before continuing.

## Required section contract

Every phase document in this folder must contain the following sections:

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

Checkpoints must describe observable outcomes, not vague intent. Acceptance criteria must define merge-quality completion conditions. Verification must name the files, tests, and commands that prove the phase is complete.

## Phase order

1. [Phase 1](phase-1.md): Rust backend scaffold and routing
2. [Phase 2](phase-2.md): Rust parser and normalized evidence extraction
3. [Phase 3](phase-3.md): Rust heuristics and findings
4. [Phase 4](phase-4.md): verification, performance, and rollout

## Parallel work policy

- Phase 1 must finish before Phase 2 begins.
- Phase 2 must establish stable parser output before Phase 3 begins.
- Phase 4 may draft tests and documentation expectations early, but final sign-off depends on Phases 1 through 3 being complete.
- If a Rust heuristic requires parser data that does not exist yet, extend Phase 2 first rather than adding parser work ad hoc inside Phase 3.

## Phase file map

- [phase-1.md](phase-1.md): backend scaffold and scan routing
- [phase-2.md](phase-2.md): parser contract and evidence model
- [phase-3.md](phase-3.md): Rust bad-practice heuristics and fixtures
- [phase-4.md](phase-4.md): verification, benchmark, documentation, and rollout

## Execution policy

When implementation starts, the default expectation is to work phase by phase and keep changes reviewable.

- Start with a minimal Rust backend that can discover and parse `.rs` files without heuristics.
- Add only the parser fields needed for the first Rust rule pack.
- Prefer fewer files and simpler module boundaries until real growth justifies further decomposition.
- Do not over-generalize Go and Rust into one abstraction if it makes both backends harder to understand.

## Related documents

- `guides/implementation-guide.md`
- `guides/features-and-detections.md`
- `guides/rust/release-checklist.md`