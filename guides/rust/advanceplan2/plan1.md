# Plan 1 - Public API And Error Surface (Rust)

Date: 2026-03-30

## Objective

Add a Rust rule family for public API signatures and error-surface choices that are common in generated code but reduce ergonomics, clarity, or library stability.

## Existing Coverage Explicitly Excluded

This plan must not duplicate the currently shipped Rust packs for:

- macro leftovers (`todo!`, `dbg!`, `panic!`, `unwrap`, `expect`)
- async runtime misuse
- performance/layout heuristics
- domain modeling and invariants
- unsafe soundness

The focus here is public-facing API design, not runtime behavior or unsafe correctness.

## Candidate Rule Inventory

- [ ] `rust_public_anyhow_result`
  - Detect public library-facing functions that return `anyhow::Result<_>` or equivalent opaque application-edge error types.
- [ ] `rust_public_box_dyn_error`
  - Detect public APIs that expose `Box<dyn Error>` rather than a concrete or domain-specific error surface.
- [ ] `rust_borrowed_string_api`
  - Detect `&String` in public argument positions where `&str` is the more general contract.
- [ ] `rust_borrowed_vec_api`
  - Detect `&Vec<T>` in public argument positions where `&[T]` is a better contract.
- [ ] `rust_borrowed_pathbuf_api`
  - Detect `&PathBuf` in public signatures where `&Path` would avoid over-specifying ownership.
- [ ] `rust_public_bool_parameter_api`
  - Detect public functions that expose raw boolean mode switches instead of named types or separate entry points.

## Why These Rules Belong In Advance Plan 2

- [ ] These are classic review findings that do not fit the current runtime-focused Rust packs.
- [ ] They are especially common in AI-authored Rust because example snippets often optimize for compiling quickly rather than shaping stable APIs.
- [ ] The rules are mostly signature-driven and should be implementable with parser and visibility evidence.

## Parser And Evidence Work

- [ ] Preserve signature type text and visibility for public functions, methods, and associated functions.
- [ ] Track return-type text for public APIs so error-surface rules can remain purely syntactic.
- [ ] Preserve parameter order and names so `rust_public_bool_parameter_api` can anchor the exact flag argument.
- [ ] Add small helpers that normalize common path spellings such as `std::path::PathBuf` versus imported `PathBuf`.

## Implementation Checklist

- [ ] Add parser tests for borrowed type spellings and public visibility.
- [ ] Implement the API-surface heuristics in a new Rust sub-module or a dedicated API-shape file.
- [ ] Keep findings scoped to public or crate-exposed APIs first; private helpers should stay mostly quiet.
- [ ] Use conservative suppressions for builder internals, trait impl requirements, and test-only helpers.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `public_anyhow_result_positive.rs.txt`
  - [ ] `public_box_dyn_error_positive.rs.txt`
  - [ ] `borrowed_string_api_positive.rs.txt`
  - [ ] `borrowed_vec_api_positive.rs.txt`
  - [ ] `borrowed_pathbuf_api_positive.rs.txt`
  - [ ] `public_bool_parameter_api_positive.rs.txt`
- [ ] Negative fixtures:
  - [ ] typed domain error enums
  - [ ] `&str`, `&[T]`, and `&Path` signatures
  - [ ] internal helpers that intentionally use concrete borrowed containers
  - [ ] explicit option or enum parameters instead of booleans

## Acceptance Criteria

- [ ] Findings reference the public signature element that narrows or weakens the API.
- [ ] The rule family remains low-noise for ordinary internal code.
- [ ] The output clearly explains why the alternative contract is more idiomatic or more stable.

## Non-Goals

- [ ] Enforcing one public API style for every Rust crate.
- [ ] Full trait-bound reasoning or semver analysis.
- [ ] Replacing Clippy or rustdoc review for all API concerns.