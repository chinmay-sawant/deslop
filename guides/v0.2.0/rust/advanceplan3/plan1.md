# Plan 1 - Error, I/O, And Path Boundaries (Rust)

Date: 2026-04-05

## Status

- [x] Implemented in the Rust scan pipeline, heuristics tree, and catalog registry.
- [x] Backlog derived from the current `src/`, `tests/`, and Rust-only architecture review trail.
- [x] Required fixtures, integration coverage, catalog entries, and docs updates are now in place.

## Objective

Add a Rust rule family that turns the project's own internal best practices around typed errors, bounded I/O, and safe path handling into conservative scan-time heuristics for Rust repositories.

## Source Inputs

- [x] `src/error.rs`
- [x] `src/io.rs`
- [x] `src/config.rs`
- [x] `src/scan/walker.rs`
- [x] `tests/integration_scan/rust.rs`
- [x] `tests/integration_scan/rust_advanced.rs`
- [x] `guides/architecture_enhancement/RUST_ARCHITECTURE_REVIEW_7.md`
- [x] `guides/rust/advanceplan1/newplan4.md`
- [x] `guides/rust/advanceplan1/newplan_checklist.md`

## Existing Coverage Explicitly Excluded

- [x] `rust_public_anyhow_result`
- [x] `rust_path_join_absolute`
- [x] `rust_serde_unknown_fields_allowed`
- [x] `rust_workspace_missing_resolver`

## Proposed Rules

- [x] `rust_internal_anyhow_result`
  - Flag non-binary library code that still returns `anyhow::Result` or `anyhow::Error` instead of a crate-local error surface.
- [x] `rust_unbounded_read_to_string`
  - Flag production code that slurps entire files with `fs::read_to_string` or equivalent unbounded read helpers.
- [x] `rust_check_then_open_path`
  - Flag obvious check-then-open filesystem flows that can race between metadata checks and open/read calls.
- [x] `rust_secret_equality_compare`
  - Flag direct `==`/`!=` comparisons on secret-like fields or variables.
- [x] `rust_narrowing_numeric_cast`
  - Flag suspicious numeric narrowing via `as` where truncation or overflow would be easy to miss.
- [x] `rust_manual_tempdir_lifecycle`
  - Flag ad hoc temp-directory creation plus manual cleanup patterns that should usually use `tempfile`/RAII helpers.

## Parser And Evidence Work

- [x] Reused existing import, call, and literal evidence before adding new parser fields.
- [x] Added lightweight operator/comparison evidence only where needed for explainable secret and narrowing findings.
- [x] Kept `anyhow` detection path-aware so `src/bin/`, `main.rs`, and tooling scripts are lower-noise than library modules.

## Fixtures And Tests

- [x] Added grouped positive and clean fixtures under `tests/fixtures/rust/advanceplan3/`.
- [x] Added grouped integration coverage under a dedicated Rust expansion test module instead of bloating existing files.
- [x] Added at least one mixed fixture where multiple boundary rules fire together without duplicate findings.
- [x] Added one clean fixture showing typed errors plus bounded reads so this family stays anchored to the intended best practice.

## Acceptance

- [x] Findings point to the exact call, operator, or return shape that triggered the rule.
- [x] Tests, benches, and one-off binary entrypoints stay substantially quieter than library code.
- [x] The family stays syntactic and reviewable without pretending to prove exploitability.
- [x] Temp-workspace findings stay biased toward obviously manual lifecycle code rather than every legitimate temporary path usage.

## Non-Goals

- [ ] Full taint tracking for path safety.
- [ ] Proving a race or truncation is exploitable in production.
- [ ] Replacing Clippy, audit tools, or a security review for all low-level Rust risks.
