# Plan 1 - Public API And Error Surface (Rust)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verification passed with `cargo test rust_advanceplan2`.
- [x] Parser evidence regression passed with `cargo test test_collects_advanceplan2_rust_summaries`.

## Objective

Add a Rust rule family for public API signatures and error-surface choices that are common in generated code but weaken ergonomics, clarity, or library stability.

## Existing Coverage Explicitly Excluded

This plan does not duplicate:

- macro leftovers (`todo!`, `dbg!`, `panic!`, `unwrap`, `expect`)
- async runtime misuse
- performance and layout heuristics
- domain modeling and invariants
- unsafe soundness

## Shipped Rules

- [x] `rust_public_anyhow_result`
- [x] `rust_public_box_dyn_error`
- [x] `rust_borrowed_string_api`
- [x] `rust_borrowed_vec_api`
- [x] `rust_borrowed_pathbuf_api`
- [x] `rust_public_bool_parameter_api`

## Implementation Notes

- [x] Reused shared `signature_text` and public-visibility detection from parsed Rust functions.
- [x] Kept error-surface checks library-biased so `main.rs` and `bin/` entrypoints stay quieter.
- [x] Suppressed builder-style setters and receiver-internal configuration methods for borrowed-container and bool-flag checks.

## Parser And Evidence Work

- [x] Preserved full Rust signature text for parameter and return-type analysis.
- [x] Fixed parameter-list extraction to use the matching closing parenthesis so return types like `Result<()>` do not break bool-parameter detection.
- [x] Kept the rule family signature-driven and explainable without trait or type resolution.

## Fixtures And Tests

- [x] Added grouped API positive and clean fixtures under `tests/fixtures/rust/advanceplan2/`.
- [x] Added grouped integration coverage in `tests/integration_scan/rust_advanceplan2.rs`.
- [x] Reused the Rust parser regression in `src/analysis/rust/parser.rs` for signature and summary coverage.

## Acceptance

- [x] Findings anchor the exact public parameter or return-shape choice.
- [x] Private helpers and builder internals stay quiet on representative clean fixtures.
- [x] The shipped rules remain conservative and parser-backed.

## Non-Goals

- [x] Enforcing one public API style for every Rust crate.
- [x] Full trait-bound reasoning or semver analysis.
- [x] Replacing Clippy or rustdoc review for all API concerns.