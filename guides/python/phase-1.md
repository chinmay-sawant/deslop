# Phase 1: Python Backend, Parser, And Evidence Contract

## Overview

This phase makes Python a real input language for deslop. The outcome is not just backend registration. The outcome is a stable parser contract that fits the current scan pipeline, produces conservative normalized evidence, and enables both shared heuristics and the first Python-specific rules without re-parsing syntax trees inside the heuristic layer.

The implementation should fill the existing Python placeholder directories already present in the repository:

- `src/analysis/python/parser/`
- `src/heuristics/python/`

This phase should not try to model the whole Python language. The goal is to support `.py` scanning with enough file-level and function-level evidence for the first shippable Python rule pack.

## In Scope

- [x] Register a Python backend in `src/analysis/mod.rs`
- [x] Add the parser runtime dependency needed for Python source parsing
- [x] Create `src/analysis/python/mod.rs`
- [x] Fill `src/analysis/python/parser/` with the first parser implementation
- [x] Route `.py` files through the existing scan pipeline
- [x] Extract the minimum normalized evidence needed for shared heuristics and first Python-specific heuristics
- [x] Define conservative Python test-file and test-function classification
- [x] Decide which evidence belongs in shared analysis types versus Python-local helper structures
- [x] Ensure parse failures and syntax errors remain recoverable in the existing report model
- [x] Establish the initial Python package and module naming policy used by reports and any future local index resolution

## Out Of Scope

- Full Python name resolution across virtual environments or installed packages
- Import execution, module loading, or runtime introspection
- Type inference, mypy-style reasoning, or control-flow-sensitive narrowing
- Duplication detection
- Deep architectural scoring such as proving god objects or tight coupling
- Shipping a broad Python rule pack before parser evidence is stable

## Dependencies

- No prior Python phase dependency. This is the starting implementation phase.
- Blocks Phase 2 because Python heuristics must consume normalized evidence rather than raw syntax-tree traversal.
- Blocks Phase 3 because rollout work depends on a real Python backend and fixture coverage.

## Primary Code Areas

- `Cargo.toml`
- `src/analysis/mod.rs`
- `src/analysis/types.rs`
- `src/analysis/python/mod.rs`
- `src/analysis/python/parser/mod.rs`
- `src/analysis/python/parser/comments.rs`
- `src/analysis/python/parser/general.rs`
- `src/analysis/python/parser/performance.rs`
- `src/index/mod.rs`
- `src/model/mod.rs`
- `tests/integration_scan.rs`
- `tests/integration_scan/python.rs`
- `tests/fixtures/python/`

## Implementation Checkpoints

1. Register the backend without changing the scanner contract.

	Required outcomes:

	- [x] `registered_backends()` includes Python alongside Go and Rust.
	- [x] `supported_extensions()` includes `py`.
	- [x] `.py` files are discovered and routed without changing the existing walker policy.
	- [x] The CLI and report flow stay language-agnostic.

2. Choose and wire the parser runtime.

	Required outcomes:

	- [x] Add the Python tree-sitter dependency in the same spirit as the Go and Rust parser dependencies.
	- [x] Parser configuration failures surface as ordinary per-file errors where possible.
	- [x] Syntax-tolerant parsing is preferred so malformed Python files can still contribute partial structure when safe.

3. Define the Python parser module boundaries.

	Recommended starting split:

	- [x] `general.rs`: imports, declared symbols, class and function discovery, decorators, module naming
	- [x] `comments.rs`: docstring extraction, TODO or FIXME comment capture if needed later, nearby comment helpers
	- [x] `performance.rs`: loop-local string concatenation, obvious full-dataset loading calls, sync I/O inside async functions
	- [x] `tests.rs`: parser-focused unit coverage

	Rule:

	- [x] Keep the parser split limited to boundaries that help readability.

4. Define the minimum file-level evidence contract.

	The parser should extract at least:

	- [x] language and path
	- [x] conservative module or package name for reporting
	- [x] whether the file appears to be test code
	- [x] syntax error state
	- [x] imports with useful aliases
	- [x] declared symbols such as module-level functions, classes, methods, and assignments that matter to the index or heuristics
	- [x] package-level named string literals when they support secret or hardcoded-value heuristics

	Python-specific note:

	- [x] Use a package naming policy that can represent both `package/module.py` and `package/__init__.py` cleanly.

5. Define the minimum function-level evidence contract.

	The parser should capture at least:

	- [x] function and method names
	- [x] line span and fingerprint basics
	- [x] direct call sites and attribute-style call targets where syntax makes that practical
	- [x] `async def` classification
	- [x] test-function classification
	- [x] local binding names
	- [x] docstring content when it is available at function scope
	- [x] local string literals used by shared secret-style heuristics
	- [x] evidence for loop-based string concatenation
	- [x] evidence for obvious sync-I/O calls inside async functions
	- [x] evidence for broad or swallowed exception handling if it belongs to the first rule pack

	Shared-model rule:

	- [x] Keep Python-only signals out of shared fields when they would distort the cross-language model.

6. Define Python test detection policy.

	The first implementation must document how Python test code is identified.

	Preferred sources:

	- [x] files under `tests/`
	- [x] filenames matching `test_*.py` or `*_test.py`
	- [x] functions named `test_*`
	- [x] methods inside classes whose names or bases make test usage obvious

	The first pass should stay conservative. If certainty is low, prefer not to over-classify production code as tests.

7. Decide how much existing shared heuristics Python can unlock immediately.

	At the end of this phase it should be explicit whether Python parser evidence is sufficient to enable these shared rule families:

	- [x] generic and overlong naming
	- [x] hardcoded secret detection from named string literals
	- [x] tutorial-style or heading-style comment detection if docstrings are normalized as comments
	- [x] test-quality findings if Python test summaries are populated reliably

	If a shared rule cannot be enabled honestly, leave it off and document the gap.

8. Define parse-failure behavior.

	Required outcomes:

	- [x] malformed Python files do not abort the entire scan
	- [x] per-file parse failures flow through the existing parse-failure reporting path
	- [x] `syntax_error` behavior remains consistent with current language backends where tree-sitter recovers enough structure

9. Add Python parser and routing coverage.

	Required tests:

	- [x] backend registration and extension routing
	- [x] module-level function extraction
	- [x] class and method extraction
	- [x] import alias capture
	- [x] async-function classification
	- [x] test detection
	- [x] syntax-error tolerance or recoverable failure behavior

## Acceptance Criteria

- [x] Python files are discoverable and analyzable through the existing scan pipeline.
- [x] The Python backend produces `ParsedFile` output with the minimum file and function evidence needed by Phase 2.
- [x] Shared analysis-model changes are deliberate and minimal.
- [x] Parse failures remain recoverable and visible through the current reporting model.
- [x] Parser-focused tests exist for Python routing, extraction, test classification, and syntax tolerance.
- [x] The code layout uses the existing Python directories instead of creating a separate side-channel implementation.

## Verification

- [x] Review `src/analysis/mod.rs`, `src/analysis/python/mod.rs`, and `src/analysis/python/parser/` against the evidence contract defined here.
- [x] Run `cargo test` after parser and shared-model changes.
- [x] Add or extend `tests/integration_scan/python.rs` to prove `.py` discovery and scan execution.
- [x] Verify that mixed-language scans still only route files to the matching backend.

## Document Update Obligations

- [x] Keep this file updated when the Python parser contract changes materially.
- [x] Update `guides/implementation-guide.md` once Python becomes a real backend.
- [x] Update `guides/features-and-detections.md` after Python findings become user-visible.

## Risks And Open Questions

- Python syntax is simple to parse but difficult to classify semantically; this phase must resist over-promising semantic precision.
- Python decorators, nested functions, and dynamic imports can tempt the evidence model into becoming too broad too early.
- Test detection can be file-based, naming-based, or framework-based; the first pass should choose conservative heuristics and document limitations.
- A poor package and module naming policy here will make any future Python-local index work harder than necessary.