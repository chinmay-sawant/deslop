# Plan 3 - File-Level and Repo-Level Python Heuristic Expansion

Date: 2026-03-29

## Objective

Convert the Python improvement plan into a concrete roadmap for the heuristics that already exist at the file and repository level. This plan is specifically about Rust heuristics, Rust integration tests, and text fixtures. It does not introduce framework packs, type inference systems, or other new subsystems that are not already part of the current application.

## Current File-Level and Repo-Level Rule Inventory

### Structure heuristics in `src/heuristics/python/structure.rs`

- `god_function`
- `monolithic_init_module`
- `too_many_instance_attributes`
- `monolithic_module`
- `god_class`
- `eager_constructor_collaborators`
- `over_abstracted_wrapper`
- `mixed_concerns_function`
- `name_responsibility_mismatch`
- `deep_inheritance_hierarchy`
- `tight_module_coupling`

### Duplication heuristics in `src/heuristics/python/duplication.rs`

- `repeated_string_literal`
- `duplicate_error_handler_block`
- `duplicate_validation_pipeline`
- `duplicate_test_utility_logic`
- `cross_file_copy_paste_function`
- `duplicate_transformation_pipeline`
- `cross_file_repeated_literal`
- `duplicate_query_fragment`

### AI-smell heuristics in `src/heuristics/python/ai_smells.rs`

- `textbook_docstring_small_helper`
- `mixed_naming_conventions`
- `unrelated_heavy_import`
- `obvious_commentary`
- `enthusiastic_commentary`

## Detailed Checklist

### 1. Separate file-level work from repo-level work

- [ ] Keep file-level heuristics validated with single-file fixtures whenever possible.
- [ ] Keep repo-level heuristics validated with multi-file fixture assemblies copied into temp workspaces.
- [ ] Do not mix file-level and repo-level expectations into the same fixture unless the relationship is the point of the test.

### 2. Add explicit fixture coverage for structure heuristics

- [ ] Add `.txt` fixtures for positive and negative class-shape cases:
  - [ ] `tests/fixtures/python/structure/god_class_positive.txt`
  - [ ] `tests/fixtures/python/structure/god_class_negative.txt`
  - [ ] `tests/fixtures/python/structure/too_many_instance_attributes_positive.txt`
  - [ ] `tests/fixtures/python/structure/too_many_instance_attributes_negative.txt`
  - [ ] `tests/fixtures/python/structure/over_abstracted_wrapper_positive.txt`
  - [ ] `tests/fixtures/python/structure/over_abstracted_wrapper_negative.txt`
- [ ] Add `.txt` fixtures for function-shape structure rules:
  - [ ] `tests/fixtures/python/structure/god_function_positive.txt`
  - [ ] `tests/fixtures/python/structure/god_function_negative.txt`
  - [ ] `tests/fixtures/python/structure/mixed_concerns_positive.txt`
  - [ ] `tests/fixtures/python/structure/mixed_concerns_negative.txt`
  - [ ] `tests/fixtures/python/structure/name_responsibility_positive.txt`
  - [ ] `tests/fixtures/python/structure/name_responsibility_negative.txt`
- [ ] Keep monolithic-module scenarios as dedicated large fixtures instead of inline strings where practical.

### 3. Add explicit fixture coverage for duplication heuristics

- [ ] Add file-level duplication fixtures:
  - [ ] `tests/fixtures/python/duplication/repeated_literals_positive.txt`
  - [ ] `tests/fixtures/python/duplication/repeated_literals_negative.txt`
  - [ ] `tests/fixtures/python/duplication/error_handlers_positive.txt`
  - [ ] `tests/fixtures/python/duplication/error_handlers_negative.txt`
  - [ ] `tests/fixtures/python/duplication/validation_pipeline_positive.txt`
  - [ ] `tests/fixtures/python/duplication/validation_pipeline_negative.txt`
- [ ] Add repo-level duplication fixture sets:
  - [ ] `tests/fixtures/python/duplication/query_fragment_repo_a.txt`
  - [ ] `tests/fixtures/python/duplication/query_fragment_repo_b.txt`
  - [ ] `tests/fixtures/python/duplication/query_fragment_shared_constants.txt`
  - [ ] `tests/fixtures/python/duplication/cross_file_copy_a.txt`
  - [ ] `tests/fixtures/python/duplication/cross_file_copy_b.txt`
  - [ ] `tests/fixtures/python/duplication/transformation_pipeline_a.txt`
  - [ ] `tests/fixtures/python/duplication/transformation_pipeline_b.txt`

### 4. Add explicit fixture coverage for AI-smell heuristics

- [ ] `tests/fixtures/python/ai_smells/docstring_small_helper_positive.txt`
- [ ] `tests/fixtures/python/ai_smells/docstring_small_helper_negative.txt`
- [ ] `tests/fixtures/python/ai_smells/mixed_naming_positive.txt`
- [ ] `tests/fixtures/python/ai_smells/mixed_naming_negative.txt`
- [ ] `tests/fixtures/python/ai_smells/heavy_imports_positive.txt`
- [ ] `tests/fixtures/python/ai_smells/heavy_imports_negative.txt`
- [ ] `tests/fixtures/python/ai_smells/commentary_positive.txt`
- [ ] `tests/fixtures/python/ai_smells/commentary_negative.txt`

### 5. Convert large inline integration sources into named text fixtures over time

- [ ] Replace large inline monolithic-module builders with fixture-backed sources where the generated size is not essential to the test.
- [ ] Replace repeated query, copy-paste, and transformation examples with dedicated fixture files assembled into temp repo layouts.
- [ ] Keep generated source strings only for cases where file size itself is the test signal.

### 6. Keep repo-level assertions grouped by behavior

- [ ] Group inheritance and coupling assertions together.
- [ ] Group duplication rules together.
- [ ] Group monolithic-module and wrapper-shape assertions together.
- [ ] Group AI-smell assertions separately from correctness and performance rules.

## Required Integration Test Layout

- [ ] `tests/integration_scan/python/baseline.rs`
  - parser and syntax smoke tests
  - baseline rule-pack coverage
  - phase-4 function-level and repo-level coverage
  - hallucination and import-resolution coverage
- [ ] `tests/integration_scan/python/phase5_rules.rs`
  - instance-attribute escalation
  - duplicate query fragment coverage
  - cross-file copy-paste coverage
  - duplicate transformation pipeline coverage
  - monolithic module coverage
  - over-abstracted wrapper coverage
  - name-responsibility mismatch coverage
  - business-rule and boundary-robustness coverage

## Specific Test Cases to Create or Refactor

- [ ] `test_python_structure_rule_family_positive`
- [ ] `test_python_structure_rule_family_negative`
- [ ] `test_python_duplication_rule_family_positive`
- [ ] `test_python_duplication_rule_family_negative`
- [ ] `test_python_ai_smells_rule_family_positive`
- [ ] `test_python_ai_smells_rule_family_negative`
- [ ] `test_python_repo_level_duplication_skips_shared_constants`
- [ ] `test_python_monolithic_module_skips_legitimate_large_modules`

## Review Gates

- [ ] Repo-level rules must prove they span more than one file before firing.
- [ ] File-level rules must not depend on repository-wide coincidences.
- [ ] Monolithic-module tests must include legitimate large-module suppressions.
- [ ] Duplication rules must show explicit negative coverage for centralized constants, shared templates, and migration-style paths.
- [ ] AI-smell rules must avoid flagging tests and intentionally tiny helpers.

## Acceptance Criteria

- [ ] The plan names the actual file-level and repo-level Python heuristics currently shipped.
- [ ] The fixture roadmap is text-based and grouped by heuristic family.
- [ ] The integration layout reflects the Python test-file split rather than a single expanding file.
- [ ] Large repo-style tests are planned as workspace assemblies, not as new application code.

## Definition of Done

- [ ] Future Python structure, duplication, and AI-smell work has a fixture and test target before coding starts.
- [ ] The plan explicitly covers both positive detections and suppressions.
- [ ] The repo-level test strategy is aligned with the Rust integration harness already in the repository.