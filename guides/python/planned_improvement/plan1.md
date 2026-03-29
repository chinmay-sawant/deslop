# Plan 1 - Python Parser Evidence and Fixture Contract

Date: 2026-03-29

## Objective

Refocus the Python improvement plan on the support that already exists in this repository:

- Rust-only Python parsing and analysis under `src/analysis/python`
- Rust heuristics under `src/heuristics/python`
- Rust integration coverage under `tests/integration_scan`
- Text fixtures under `tests/fixtures/python`

This plan does not add Python runtime code, Python helper scripts, or generated Python modules to the application. All planned validation artifacts remain Rust tests plus `.txt` fixtures.

## Current Python Support Baseline

### Analysis entrypoints already present

- `src/analysis/python/mod.rs` registers the Python backend.
- `src/analysis/python/parser/mod.rs` parses `.py` files through tree-sitter.
- `src/analysis/python/parser/general.rs` extracts imports, functions, symbols, package strings, and class summaries.
- `src/analysis/python/parser/comments.rs` extracts comment summaries.
- `src/analysis/python/parser/performance.rs` extracts function-level evidence used by performance and maintainability rules.
- `src/analysis/python/parser/phase4.rs` extracts additional evidence that powers the current phase-4 and phase-5 heuristic families.

### Parsed evidence already available to heuristics

- File-level: imports, symbols, comments, package-level string literals, class summaries, syntax error flag, line count, byte size.
- Function-level: call sites, doc comments, exception handlers, local string bindings, normalized body text, validation signatures, varargs and kwargs flags, type-hint completeness.
- Phase-4 evidence: `none_comparison_lines`, `side_effect_comprehension_lines`, `redundant_return_none_lines`, `list_materialization_lines`, `deque_operation_lines`, `temp_collection_lines`, `recursive_call_lines`, `list_membership_loop_lines`, `repeated_len_loop_lines`, `builtin_candidate_lines`, `missing_context_manager_lines`.

### Current parser-to-heuristic contract

- `general.rs`
  - Produces file identity and discovery data: `package_name`, `is_test_file`, `imports`, `symbols`, `functions`.
  - Produces function-level evidence used directly by heuristics: `calls`, `exception_handlers`, `doc_comment`, `body_text`, `local_strings`, `local_binding_names`, `test_summary`, `is_async`, `is_test_function`.
  - Produces symbol and import evidence consumed by naming, import-resolution, coupling, and mixed-sync-async rules.
- `comments.rs`
  - Produces file-level `comments` and function docstring extraction.
  - Feeds `obvious_commentary`, `enthusiastic_commentary`, and `textbook_docstring_small_helper`.
- `performance.rs`
  - Produces `concat_loops` for `string_concat_in_loop`.
- `phase4.rs`
  - Produces `normalized_body`, `validation_signature`, `exception_block_signatures`, `none_comparison_lines`, `side_effect_comprehension_lines`, `redundant_return_none_lines`, `list_materialization_lines`, `deque_operation_lines`, `temp_collection_lines`, `recursive_call_lines`, `list_membership_loop_lines`, `repeated_len_loop_lines`, `builtin_candidate_lines`, `missing_context_manager_lines`, `has_complete_type_hints`, `has_varargs`, `has_kwargs`, and `class_summaries`.
  - Feeds the current phase-4 and phase-5 maintainability, performance, structure, duplication, and repo-level inheritance checks.

### Current heuristic consumers of parser evidence

- Performance heuristics consume `imports`, `calls`, `concat_loops`, `list_materialization_lines`, `deque_operation_lines`, `temp_collection_lines`, `recursive_call_lines`, `list_membership_loop_lines`, and `repeated_len_loop_lines`.
- Maintainability heuristics consume `calls`, `exception_handlers`, `comments`, `imports`, `body_text`, `none_comparison_lines`, `side_effect_comprehension_lines`, `redundant_return_none_lines`, `builtin_candidate_lines`, `missing_context_manager_lines`, `has_varargs`, `has_kwargs`, and `has_complete_type_hints`.
- Structure heuristics consume `package_name`, `imports`, `functions`, `calls`, `line_count`, `byte_size`, and `class_summaries` including `instance_attribute_count`, `public_method_count`, `base_classes`, and `constructor_collaborator_count`.
- Duplication heuristics consume `pkg_strings`, `local_strings`, `imports`, `calls`, `normalized_body`, `validation_signature`, and repo-wide `functions`.
- AI-smell heuristics consume `symbols`, `imports`, `comments`, `functions`, and docstrings already normalized into function evidence.

### Existing Python test and fixture layout

- Parser unit tests: `src/analysis/python/parser/tests.rs`
- Integration entrypoint: `tests/integration_scan.rs`
- Python integration module root: `tests/integration_scan/python/mod.rs`
- Baseline Python integration coverage: `tests/integration_scan/python/baseline.rs`
- Phase-5 Python integration coverage: `tests/integration_scan/python/phase5_rules.rs`
- Fixture root: `tests/fixtures/python`
- Existing text fixture groups: root parser and rule-pack fixtures, `tests/fixtures/python/parser/`, and `tests/fixtures/python/integration/{baseline,phase5}/`

## Detailed Checklist

### 1. Lock the current parser-to-heuristic contract

- [x] Document every `ParsedFile`, `ParsedFunction`, and `ClassSummary` field consumed by `src/heuristics/python`.
- [x] Record which fields come from `general.rs`, `comments.rs`, `performance.rs`, and `phase4.rs`.
- [x] Add a short contract comment or guide section for any field whose semantics are easy to misread.
- [x] Confirm that new heuristic work extends existing evidence instead of reparsing source inside heuristics.

### 2. Close parser evidence gaps before adding new heuristics

- [x] Audit async evidence used by `blocking_sync_io_in_async` and `mixed_sync_async_module`.
- [ ] Audit boundary-call metadata used by `network_boundary_without_timeout`, `environment_boundary_without_fallback`, and `external_input_without_validation`.
- [x] Audit import-resolution evidence used by the hallucination rules and package re-export tests.
- [ ] Audit comment extraction quality for `obvious_commentary` and `enthusiastic_commentary`.
- [x] Audit class-summary quality for `god_class`, `too_many_instance_attributes`, `eager_constructor_collaborators`, and `deep_inheritance_hierarchy`.
- [ ] Capture any missing evidence additions as Rust parser work only. Do not add Python-side preprocessing.

### 3. Standardize fixture authoring around `.txt` source files

- [ ] Keep all Python test inputs as `.txt` fixtures under `tests/fixtures/python`.
- [ ] Stop adding new inline mega-strings to integration tests when the same source can live as a fixture.
- [ ] Introduce grouped fixture folders for future work:
  - [x] `tests/fixtures/python/parser/`
  - [x] `tests/fixtures/python/performance/`
  - [x] `tests/fixtures/python/maintainability/`
  - [ ] `tests/fixtures/python/structure/`
  - [ ] `tests/fixtures/python/duplication/`
  - [ ] `tests/fixtures/python/ai_smells/`
  - [ ] `tests/fixtures/python/hallucination/`
- [ ] Keep fixture names explicit and paired: `_positive.txt`, `_negative.txt`, `_repo_a.txt`, `_repo_b.txt`, `_test_only.txt`.

### 4. Expand parser unit coverage before expanding heuristics

- [ ] Add parser-only unit tests for each new evidence field in `src/analysis/python/parser/tests.rs`.
- [ ] Prefer parser tests for evidence extraction and integration tests for full scanner behavior.
- [ ] Add positive and negative parser assertions for:
  - [x] import re-exports through `__init__.py`
  - [x] parenthesized imports with inline comments
  - [x] async function call-site capture
  - [x] broad versus specific exception handlers
  - [x] missing context manager evidence
  - [x] constructor collaborator counting
  - [x] naming-style symbol extraction

### 5. Split integration coverage by responsibility

- [x] Keep `tests/integration_scan/python/baseline.rs` focused on baseline parser behavior, syntax handling, baseline rule-pack coverage, phase-4 coverage, and hallucination coverage.
- [x] Move phase-5 expansion tests into `tests/integration_scan/python/phase5_rules.rs` so one file no longer carries the entire Python surface area.
- [x] Use `tests/integration_scan.rs` as the single registration point for all Python integration modules.
- [x] Keep helper macros and temp-workspace helpers in `tests/integration_scan.rs` so sibling modules share the same fixture loading path.

## Proposed Fixture Matrix

### Parser and import-resolution fixtures

- [x] `tests/fixtures/python/parser/reexports_positive.txt`
- [x] `tests/fixtures/python/parser/reexports_negative.txt`
- [x] `tests/fixtures/python/parser/parenthesized_imports_positive.txt`
- [x] `tests/fixtures/python/parser/async_calls_positive.txt`
- [x] `tests/fixtures/python/parser/class_summary_positive.txt`

### Function-level evidence fixtures

- [x] `tests/fixtures/python/performance/async_io_positive.txt`
- [x] `tests/fixtures/python/performance/async_io_negative.txt`
- [x] `tests/fixtures/python/maintainability/exception_shapes_positive.txt`
- [x] `tests/fixtures/python/maintainability/exception_shapes_negative.txt`
- [x] `tests/fixtures/python/maintainability/type_hints_positive.txt`
- [x] `tests/fixtures/python/maintainability/type_hints_negative.txt`

### File-level and repo-level evidence fixtures

- [ ] `tests/fixtures/python/structure/god_class_positive.txt`
- [ ] `tests/fixtures/python/structure/god_class_negative.txt`
- [ ] `tests/fixtures/python/duplication/query_fragment_repo_a.txt`
- [ ] `tests/fixtures/python/duplication/query_fragment_repo_b.txt`
- [ ] `tests/fixtures/python/hallucination/import_resolution_positive.txt`
- [ ] `tests/fixtures/python/hallucination/import_resolution_negative.txt`

## Required Rust File Changes for This Plan

- [x] `src/analysis/python/parser/tests.rs`
- [x] `tests/integration_scan.rs`
- [x] `tests/integration_scan/python/mod.rs`
- [x] `tests/integration_scan/python/baseline.rs`
- [x] `tests/integration_scan/python/phase5_rules.rs`
- [x] new `.txt` fixtures under `tests/fixtures/python/parser/**`
- [x] new `.txt` fixtures under `tests/fixtures/python/integration/**`

## Acceptance Criteria

- [ ] Every new Python heuristic change is traceable to parser evidence already exposed through Rust structs.
- [ ] Every new heuristic ships with at least one positive `.txt` fixture and one negative `.txt` fixture.
- [x] Parser-only behavior is covered in `src/analysis/python/parser/tests.rs` before the same behavior is relied on by higher-level heuristics.
- [x] Python integration coverage is split across multiple Rust modules instead of growing a single 1500-plus-line file.
- [x] No new Python application scripts are introduced as part of this work.

## Definition of Done

- [x] The planned parser work is described in terms of existing Rust modules.
- [ ] The fixture strategy is standardized around `.txt` files under `tests/fixtures/python`.
- [x] The integration suite layout is updated to match the documented split.
- [x] Future heuristic work can be scheduled against a stable parser-evidence contract instead of ad hoc inline test sources.