# Phase 1: Python Backend, Parser, And Evidence Contract

## Overview

This phase makes Python a real input language for deslop. The outcome is not just backend registration. The outcome is a stable parser contract that fits the current scan pipeline, produces conservative normalized evidence, and enables both shared heuristics and the first Python-specific rules without re-parsing syntax trees inside the heuristic layer.

The implementation should fill the existing Python placeholder directories already present in the repository:

- `src/analysis/python/parser/`
- `src/heuristics/python/`

This phase should not try to model the whole Python language. The goal is to support `.py` scanning with enough file-level and function-level evidence for the first shippable Python rule pack.

## In Scope

- Registering a Python backend in `src/analysis/mod.rs`
- Adding the parser runtime dependency needed for Python source parsing
- Creating `src/analysis/python/mod.rs`
- Filling `src/analysis/python/parser/` with the first parser implementation
- Routing `.py` files through the existing scan pipeline
- Extracting the minimum normalized evidence needed for shared heuristics and first Python-specific heuristics
- Defining conservative Python test-file and test-function classification
- Deciding which evidence belongs in shared analysis types versus Python-local helper structures
- Ensuring parse failures and syntax errors remain recoverable in the existing report model
- Establishing the initial Python package and module naming policy used by reports and any future local index resolution

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

	- `registered_backends()` includes Python alongside Go and Rust.
	- `supported_extensions()` includes `py`.
	- `.py` files are discovered and routed without changing the existing walker policy.
	- The CLI and report flow stay language-agnostic.

2. Choose and wire the parser runtime.

	Required outcomes:

	- Add the Python tree-sitter dependency in the same spirit as the Go and Rust parser dependencies.
	- Parser configuration failures surface as ordinary per-file errors where possible.
	- Syntax-tolerant parsing is preferred so malformed Python files can still contribute partial structure when safe.

3. Define the Python parser module boundaries.

	Recommended starting split:

	- `general.rs`: imports, declared symbols, class and function discovery, decorators, module naming
	- `comments.rs`: docstring extraction, TODO or FIXME comment capture if needed later, nearby comment helpers
	- `performance.rs`: loop-local string concatenation, obvious full-dataset loading calls, sync I/O inside async functions
	- `tests.rs`: parser-focused unit coverage

	Rule:

	- Do not create extra parser files until the split is helping readability.

4. Define the minimum file-level evidence contract.

	The parser should extract at least:

	- language and path
	- conservative module or package name for reporting
	- whether the file appears to be test code
	- syntax error state
	- imports with useful aliases
	- declared symbols such as module-level functions, classes, methods, and assignments that matter to the index or heuristics
	- package-level named string literals when they support secret or hardcoded-value heuristics

	Python-specific note:

	- For package naming, prefer a repository-relative module path policy that can represent both `package/module.py` and `package/__init__.py` cleanly.

5. Define the minimum function-level evidence contract.

	The parser should capture at least:

	- function and method names
	- line span and fingerprint basics
	- direct call sites and attribute-style call targets where syntax makes that practical
	- `async def` classification
	- test-function classification
	- local binding names
	- docstring content when it is available at function scope
	- local string literals used by shared secret-style heuristics
	- evidence for loop-based string concatenation
	- evidence for obvious sync-I/O calls inside async functions
	- evidence for broad or swallowed exception handling if it belongs to the first rule pack

	Shared-model rule:

	- If a Python signal is only consumed by Python heuristics and would distort existing shared fields, keep it in a Python-local evidence helper first.

6. Define Python test detection policy.

	The first implementation must document how Python test code is identified.

	Preferred sources:

	- files under `tests/`
	- filenames matching `test_*.py` or `*_test.py`
	- functions named `test_*`
	- methods inside classes whose names or bases make test usage obvious

	The first pass should stay conservative. If certainty is low, prefer not to over-classify production code as tests.

7. Decide how much existing shared heuristics Python can unlock immediately.

	At the end of this phase it should be explicit whether Python parser evidence is sufficient to enable these shared rule families:

	- generic and overlong naming
	- hardcoded secret detection from named string literals
	- tutorial-style or heading-style comment detection if docstrings are normalized as comments
	- test-quality findings if Python test summaries are populated reliably

	If a shared rule cannot be enabled honestly, leave it off and document the gap.

8. Define parse-failure behavior.

	Required outcomes:

	- malformed Python files should not abort the entire scan
	- per-file parse failures should flow through the existing parse-failure reporting path
	- `syntax_error` behavior should remain consistent with current language backends where tree-sitter recovers enough structure

9. Add Python parser and routing coverage.

	Required tests:

	- backend registration and extension routing
	- module-level function extraction
	- class and method extraction
	- import alias capture
	- async-function classification
	- test detection
	- syntax-error tolerance or recoverable failure behavior

## Acceptance Criteria

- Python files are discoverable and analyzable through the existing scan pipeline.
- The Python backend produces `ParsedFile` output with the minimum file and function evidence needed by Phase 2.
- Shared analysis-model changes are deliberate and minimal.
- Parse failures remain recoverable and visible through the current reporting model.
- Parser-focused tests exist for Python routing, extraction, test classification, and syntax tolerance.
- The code layout uses the existing Python directories instead of creating a separate side-channel implementation.

## Verification

- Review `src/analysis/mod.rs`, `src/analysis/python/mod.rs`, and `src/analysis/python/parser/` against the evidence contract defined here.
- Run `cargo test` after parser and shared-model changes.
- Add or extend `tests/integration_scan/python.rs` to prove `.py` discovery and scan execution.
- Verify that mixed-language scans still only route files to the matching backend.

## Document Update Obligations

- Update this file whenever the Python parser contract changes materially.
- Update `guides/implementation-guide.md` once Python becomes a real backend.
- Update `guides/features-and-detections.md` only after Python findings become user-visible.

## Risks And Open Questions

- Python syntax is simple to parse but difficult to classify semantically; this phase must resist over-promising semantic precision.
- Python decorators, nested functions, and dynamic imports can tempt the evidence model into becoming too broad too early.
- Test detection can be file-based, naming-based, or framework-based; the first pass should choose conservative heuristics and document limitations.
- A poor package and module naming policy here will make any future Python-local index work harder than necessary.