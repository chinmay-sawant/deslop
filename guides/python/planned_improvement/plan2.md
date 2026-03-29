# Plan 2 - Function-Level Python Heuristic Expansion

Date: 2026-03-29

## Objective

Define a detailed implementation plan for improving the existing function-level Python heuristics without changing the application architecture. The scope is limited to Rust heuristics and Rust tests. Python support remains parser-driven and fixture-driven.

## Current Function-Level Rule Inventory

### Performance heuristics in `src/heuristics/python/performance.rs`

- `string_concat_in_loop`
- `blocking_sync_io_in_async`
- `full_dataset_load`
- `list_materialization_first_element`
- `deque_candidate_queue`
- `temporary_collection_in_loop`
- `recursive_traversal_risk`
- `list_membership_in_loop`
- `repeated_len_in_loop`

### Maintainability heuristics in `src/heuristics/python/maintainability.rs`

- `exception_swallowed`
- `eval_exec_usage`
- `print_debugging_leftover`
- `none_comparison`
- `side_effect_comprehension`
- `redundant_return_none`
- `hardcoded_path_string`
- `hardcoded_business_rule`
- `magic_value_branching`
- `reinvented_utility`
- `variadic_public_api`
- `builtin_reduction_candidate`
- `network_boundary_without_timeout`
- `environment_boundary_without_fallback`
- `external_input_without_validation`
- `broad_exception_handler`
- `missing_context_manager`
- `public_api_missing_type_hints`
- `commented_out_code`
- `mixed_sync_async_module`

## Detailed Checklist

### 1. Build a rule-to-evidence map before changing heuristics

- [ ] For each rule, list the exact parser evidence it depends on today.
- [ ] Mark whether the rule relies on structured evidence, raw `body_text`, imports, comments, or class summaries.
- [ ] Reject any new heuristic that would require reparsing Python text inside the heuristic layer.
- [ ] Capture missing-evidence requests as parser work in plan 1 first.

### 2. Split work into rule families

#### Async, I/O, and resource handling

- [ ] Revisit `blocking_sync_io_in_async` for broader sync-I/O coverage and better suppressions.
- [ ] Revisit `full_dataset_load` for `open(...).read*` and `Path(...).read_*` variants.
- [ ] Revisit `network_boundary_without_timeout` for timeout-bearing versus timeout-free call shapes.
- [ ] Revisit `missing_context_manager` for open-file and resource-handling shapes.
- [ ] Revisit `mixed_sync_async_module` to ensure mixed imports and async definitions are not over-flagged.

#### Control-flow and loop-shape heuristics

- [ ] Revisit `string_concat_in_loop`.
- [ ] Revisit `list_materialization_first_element`.
- [ ] Revisit `deque_candidate_queue`.
- [ ] Revisit `temporary_collection_in_loop`.
- [ ] Revisit `recursive_traversal_risk`.
- [ ] Revisit `list_membership_in_loop`.
- [ ] Revisit `repeated_len_in_loop`.
- [ ] Revisit `builtin_reduction_candidate`.

#### Safety and debugging leftovers

- [ ] Revisit `exception_swallowed` and `broad_exception_handler` together so they use a consistent exception taxonomy.
- [ ] Revisit `eval_exec_usage` and ensure test-only code remains suppressed.
- [ ] Revisit `print_debugging_leftover` so intentional CLI prints are not confused with debug prints.
- [ ] Revisit `commented_out_code` alongside comment-summary extraction.

#### API clarity and boundary robustness

- [ ] Revisit `variadic_public_api`.
- [ ] Revisit `public_api_missing_type_hints`.
- [ ] Revisit `environment_boundary_without_fallback`.
- [ ] Revisit `external_input_without_validation`.

#### Literal- and branch-driven maintainability heuristics

- [ ] Revisit `none_comparison`.
- [ ] Revisit `side_effect_comprehension`.
- [ ] Revisit `redundant_return_none`.
- [ ] Revisit `hardcoded_path_string`.
- [ ] Revisit `hardcoded_business_rule`.
- [ ] Revisit `magic_value_branching`.
- [ ] Revisit `reinvented_utility`.

### 3. Require positive and negative `.txt` fixtures per rule family

- [ ] Add fixture pairs for async and I/O rules:
  - [ ] `tests/fixtures/python/performance/async_boundaries_positive.txt`
  - [ ] `tests/fixtures/python/performance/async_boundaries_negative.txt`
- [ ] Add fixture pairs for loop-shape rules:
  - [ ] `tests/fixtures/python/performance/loop_shapes_positive.txt`
  - [ ] `tests/fixtures/python/performance/loop_shapes_negative.txt`
- [ ] Add fixture pairs for exception and debug rules:
  - [ ] `tests/fixtures/python/maintainability/exception_and_debug_positive.txt`
  - [ ] `tests/fixtures/python/maintainability/exception_and_debug_negative.txt`
- [ ] Add fixture pairs for boundary robustness rules:
  - [ ] `tests/fixtures/python/maintainability/boundaries_positive.txt`
  - [ ] `tests/fixtures/python/maintainability/boundaries_negative.txt`
- [ ] Add fixture pairs for branch and literal rules:
  - [ ] `tests/fixtures/python/maintainability/business_rules_positive.txt`
  - [ ] `tests/fixtures/python/maintainability/business_rules_negative.txt`

### 4. Make integration tests match rule families instead of one giant catch-all case

- [ ] Keep baseline function-level tests in `tests/integration_scan/python/baseline.rs`.
- [ ] Place phase-5 expansion tests in `tests/integration_scan/python/phase5_rules.rs`.
- [ ] Add one integration test per family instead of one test per rule where possible.
- [ ] Keep at least one dedicated assertion per rule ID so failures stay local and obvious.

### 5. Add suppression-focused negative cases

- [ ] Every new or revised heuristic must include a suppression-oriented negative case.
- [ ] Negative cases must cover test files, small helpers, honest transformer functions, centralized constants, explicit timeouts, and explicit validation.
- [ ] Prefer a named negative fixture over embedding the suppression scenario inline.

## Required Test Cases

### Performance-focused integration cases

- [ ] `test_python_performance_rule_family_positive`
- [ ] `test_python_performance_rule_family_negative`
- [ ] `test_python_async_boundary_rules_positive`
- [ ] `test_python_async_boundary_rules_negative`

### Maintainability-focused integration cases

- [ ] `test_python_exception_and_debug_rules_positive`
- [ ] `test_python_exception_and_debug_rules_negative`
- [ ] `test_python_boundary_robustness_rules_positive`
- [ ] `test_python_boundary_robustness_rules_negative`
- [ ] `test_python_branch_literal_rules_positive`
- [ ] `test_python_branch_literal_rules_negative`

## Review Gates

- [ ] Rule message text names the actual smell and does not oversell confidence.
- [ ] Evidence strings point to the exact syntactic reason the rule fired.
- [ ] Test-only code remains suppressed where intended.
- [ ] Function-level heuristics do not silently rely on repository-wide state.
- [ ] Parser changes needed for these rules are tracked separately from heuristic-message tuning.

## Acceptance Criteria

- [ ] Every function-level heuristic has an explicit fixture owner and test owner.
- [ ] Positive and negative fixture coverage exists for each rule family.
- [ ] The function-level heuristic plan reflects only the rules that exist in `performance.rs` and `maintainability.rs`.
- [ ] No plan item requires adding Python code to the application itself.

## Definition of Done

- [ ] The function-level roadmap is grouped by current heuristic families.
- [ ] Fixture creation is explicitly text-based.
- [ ] Integration tests are planned around the new module split.
- [ ] The next implementation pass can be scheduled rule-family by rule-family without re-scoping the repo.