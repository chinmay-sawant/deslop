# Phase 2: Rust Parser And Normalized Evidence Extraction

## Overview

This phase turns the Rust backend from a routing stub into a useful source of reviewable evidence. The purpose is to extract only the Rust syntax and normalized signals needed for the first bad-practice rule pack. This phase should not attempt full Rust semantic understanding.

The parser should prefer a narrow, explicit contract over broad speculative modeling. If a field exists only for a possible future heuristic and has no immediate consumer, it should usually wait.

## In Scope

- Rust file parsing through the chosen runtime
- Extraction of Rust modules, functions, methods, and declared item names
- Visibility extraction for Rust items where visibility matters to heuristics or future indexing
- Collection of Rust call sites and macro invocations needed for the first rule pack
- Detection of `async`, `unsafe`, and test-related attributes when needed by initial rules
- Rust string-literal extraction where it supports initial findings
- Nearby `SAFETY:` comment extraction for unsafe-related heuristics
- Conservative syntax-error handling and parse-failure behavior for Rust files
- Minimal evolution of shared analysis types when a Rust signal genuinely belongs in the shared model

## Out Of Scope

- Borrow-checker reasoning
- Trait resolution, type inference, or cross-crate name resolution
- Cargo feature-flag expansion or conditional compilation simulation
- Procedural macro expansion
- Adding generic shared fields just to avoid a Rust-specific helper struct when the field is not actually cross-language

## Dependencies

- Depends on Phase 1 because the Rust backend must already exist and be wired into scan routing.
- Blocks Phase 3 because heuristics should consume stable parser evidence rather than raw syntax-tree access.

## Primary Code Areas

- `src/analysis/rust/parser.rs`
- `src/analysis/rust/mod.rs`
- `src/analysis/types.rs`
- `src/model/mod.rs`
- `tests/fixtures/rust/`
- `tests/integration_scan/rust.rs`

## Implementation Checkpoints

1. Decide which evidence stays shared and which evidence stays Rust-local.

	Required rule:

	- Reuse existing shared structures only when the concept is truly language-agnostic.
	- If a Rust-only signal would distort the Go-oriented shared model, keep it behind a Rust-specific parser helper or Rust-specific function evidence struct until a later generalization is justified.

2. Extract the minimum Rust file-level structure.

	The parser should identify at least:

	- module or crate-facing item names needed for reporting
	- `use` statements or import-like paths if they are needed by later heuristics
	- declared symbols such as free functions, methods, structs, enums, traits, and type aliases when those symbols matter to evidence output or later indexing

3. Extract the minimum Rust function-level structure.

	The parser should capture at least the signals needed for the first rule pack:

	- function and method names
	- function span and line information
	- whether a function is test-only or non-test code
	- call sites for direct calls and method-style calls when practical
	- macro invocations for `dbg!`, `todo!`, `unimplemented!`, `panic!`, and other initial-rule macros
	- `unsafe` blocks or `unsafe fn` markers if unsafe-related heuristics are in the first rule pack
	- nearby `SAFETY:` comments for unsafe-related heuristics
	- `unwrap` and `expect`-style calls where they can be detected syntactically

4. Define Rust test classification policy.

	The first implementation must document how Rust test code is identified.

	Preferred sources:

	- files under `tests/`
	- items with `#[test]`
	- modules or items under `#[cfg(test)]`

	If the first pass cannot fully model module-scoped test gating, the limitation must be documented and reflected in heuristic conservatism.

5. Define Rust parse-failure behavior.

	Required outcomes:

	- ordinary per-file Rust parser failures remain recoverable through `parse_failures`
	- syntax-tolerant parsing should be preferred where the parser runtime supports it
	- malformed Rust files must not abort the entire scan unless they trigger a fatal library-surface error outside ordinary per-file parsing

6. Add parser-focused tests.

	Required coverage:

	- free function extraction
	- impl method extraction
	- macro invocation detection
	- test-function or test-module recognition
	- syntax-error tolerance and failure classification where applicable

## Acceptance Criteria

- The Rust parser extracts the minimum evidence required for the first heuristic phase.
- Shared model changes are deliberate and minimal rather than broad speculative generalization.
- Rust parser behavior for tests, syntax errors, and macro detection is documented and covered by tests.
- Rust parser failures integrate cleanly with the existing recoverable failure path.
- The parser contract is stable enough that Phase 3 can consume it without re-parsing raw syntax trees inside heuristics.

## Verification

- Review `src/analysis/rust/parser.rs`, `src/analysis/types.rs`, and any Rust-specific tests against the evidence contract defined in this phase.
- Add focused parser tests that cover representative Rust snippets, not only repository-scale fixtures.
- Run `cargo test` after parser or shared-model changes.

## Document Update Obligations

- Update this file whenever a new Rust parser signal becomes part of the planned heuristic contract.
- Update `guides/implementation-guide.md` if Rust parser-stage outputs materially change the architecture description.
- Update `guides/features-and-detections.md` only once a parser signal becomes observable through a user-facing rule.

## Risks And Open Questions

- Rust syntax includes attributes, macros, and module scoping rules that can tempt the parser into over-modeling too early.
- Test detection in Rust can be file-based, item-based, or cfg-based; the first implementation must stay conservative where certainty is low.
- Shared model pressure will increase as Rust support grows. The project should resist forcing Go and Rust into one overly generic structure until repeated duplication proves the need.