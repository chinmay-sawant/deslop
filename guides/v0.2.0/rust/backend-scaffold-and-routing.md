# Phase 1: Rust Backend Scaffold And Routing

## Overview

This phase adds the minimum Rust backend needed for deslop to recognize Rust source files and route them through a dedicated analyzer. The goal is not to ship Rust heuristics yet. The goal is to make `.rs` files a first-class input to the scan pipeline, with a lean backend layout that is small enough to evolve quickly.

The backend should start with as few files as practical:

- `src/analysis/rust/mod.rs`
- `src/analysis/rust/parser.rs`
- `tests/integration_scan/rust.rs`
- `tests/fixtures/rust/`

Only split `parser.rs` into a directory or add `fingerprint.rs` if the Rust backend actually grows large enough to justify it.

## In Scope

- Rust backend registration in `src/analysis/mod.rs`
- Parser runtime choice for Rust source, preferably aligned with current Go backend architecture
- Rust backend module creation under `src/analysis/rust`
- `.rs` file routing through the existing scan pipeline
- Minimal parser entry point that returns a `ParsedFile` for Rust input
- Smoke-test coverage for Rust file discovery and scan execution
- Documentation updates that state Rust support is being added as a new backend

## Out Of Scope

- Rich Rust heuristics
- Deep Rust AST extraction beyond what is required to parse files safely
- Cargo workspace resolution, crate graph resolution, or macro expansion
- Type-aware semantic analysis
- Reorganizing the full analysis module tree to mirror the Go backend exactly

## Dependencies

- No prior phase dependency. This is the starting implementation phase for Rust support.
- Blocks Phase 2 because parser and evidence work should build on a registered backend rather than on temporary ad hoc code.
- Blocks Phase 3 because heuristics should not start before Rust files can be scanned consistently.

## Primary Code Areas

- `Cargo.toml`
- `src/analysis/mod.rs`
- `src/analysis/rust/mod.rs`
- `src/analysis/rust/parser.rs`
- `src/scan/mod.rs`
- `tests/integration_scan.rs`
- `tests/integration_scan/rust.rs`
- `tests/fixtures/rust/`

## Implementation Checkpoints

1. Choose the Rust parser runtime.

	Preferred decision:

	- Use `tree-sitter-rust` for the first implementation pass so the Rust backend follows the same broad parsing model as the Go backend.
	- Keep the parser integration local to `src/analysis/rust/parser.rs` rather than introducing extra adapter layers.

2. Add a Rust backend module and register it in `analysis::registered_backends()`.

	Required outcomes:

	- `backend_for_path()` must resolve `.rs` files to a Rust backend.
	- `supported_extensions()` must include `rs`.
	- Existing Go routing must remain unchanged.

3. Define the minimal Rust parser contract.

	The initial parser does not need to extract every Rust signal yet. It must at least:

	- return a `ParsedFile`
	- preserve file path and byte size
	- classify syntax error state consistently with the existing model
	- produce stable empty-or-conservative collections for fields that Rust heuristics do not use yet

4. Keep the initial backend layout intentionally small.

	Required rule:

	- Do not add `src/analysis/rust/parser/` as a directory in Phase 1 unless one-file parser ownership becomes clearly unmaintainable.
	- Do not add Rust-specific helper modules outside `src/analysis/rust` unless they are shared across languages.

5. Add smoke tests for Rust scan routing.

	Required outcomes:

	- a scan over a Rust fixture repository discovers `.rs` files
	- the Rust backend parses those files without panicking
	- Rust files appear in the final `ScanReport`
	- Rust scan support does not regress Go scan behavior

## Acceptance Criteria

- `.rs` files are recognized by the scanner and routed to a Rust backend.
- A Rust backend module exists under `src/analysis/rust` and is registered centrally.
- The initial Rust parser can return a stable `ParsedFile` for representative Rust input.
- The backend layout remains intentionally small and does not prematurely mirror the Go backend tree.
- Rust smoke tests exist and pass.
- Existing Go scans continue to work.

## Verification

- Review `Cargo.toml`, `src/analysis/mod.rs`, `src/analysis/rust/mod.rs`, and `src/analysis/rust/parser.rs` against the routing and parser-runtime decisions in this phase.
- Add unit or smoke tests that cover Rust backend registration and a minimal Rust parse path.
- Run `cargo test` and `cargo build --release` after Rust backend registration lands.

## Document Update Obligations

- Update this file if the Rust parser runtime choice changes.
- Update `guides/implementation-guide.md` when Rust becomes a supported backend in the architecture description.
- Update `README.md` only when Rust support is observable from the user-facing CLI contract.

## Risks And Open Questions

- A Rust backend skeleton that overcommits to a large file tree too early will slow implementation down.
- A parser contract that tries to encode every future Rust concept in Phase 1 will create churn before the first rule ships.
- If `tree-sitter-rust` cannot provide enough fidelity for the desired first heuristics, the parser runtime choice may need to be revisited early in Phase 2.