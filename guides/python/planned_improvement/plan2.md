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

## Rule-to-Evidence and Ownership Map

### Performance rules

- `string_concat_in_loop`: structured `concat_loops`; fixtures `tests/fixtures/python/performance/loop_shapes_{positive,negative}.txt`; tests `test_python_performance_rule_family_{positive,negative}`.
- `blocking_sync_io_in_async`: structured `is_async`, `calls`, and file `imports`; fixtures `tests/fixtures/python/performance/async_boundaries_{positive,negative}.txt` plus parser-contract fixtures `async_io_{positive,negative}.txt`; tests `test_python_async_boundary_rules_{positive,negative}` and parser contract coverage.
- `full_dataset_load`: structured `calls` with `open(...).read*` and `Path(...).read_*` shapes; fixtures `tests/fixtures/python/performance/async_boundaries_{positive,negative}.txt`; tests `test_python_async_boundary_rules_{positive,negative}`.
- `list_materialization_first_element`: structured `list_materialization_lines`; fixtures `tests/fixtures/python/performance/loop_shapes_{positive,negative}.txt`; tests `test_python_performance_rule_family_{positive,negative}`.
- `deque_candidate_queue`: structured `deque_operation_lines`; fixtures `tests/fixtures/python/performance/loop_shapes_{positive,negative}.txt`; tests `test_python_performance_rule_family_{positive,negative}`.
- `temporary_collection_in_loop`: structured `temp_collection_lines`; fixtures `tests/fixtures/python/performance/loop_shapes_{positive,negative}.txt`; tests `test_python_performance_rule_family_{positive,negative}`.
- `recursive_traversal_risk`: structured `recursive_call_lines` and `fingerprint.line_count`; fixtures `tests/fixtures/python/performance/loop_shapes_{positive,negative}.txt`; tests `test_python_performance_rule_family_{positive,negative}`.
- `list_membership_in_loop`: structured `list_membership_loop_lines`; fixtures `tests/fixtures/python/performance/loop_shapes_{positive,negative}.txt`; tests `test_python_performance_rule_family_{positive,negative}`.
- `repeated_len_in_loop`: structured `repeated_len_loop_lines`; fixtures `tests/fixtures/python/performance/loop_shapes_{positive,negative}.txt`; tests `test_python_performance_rule_family_{positive,negative}`.

### Maintainability rules

- `exception_swallowed`: structured `exception_handlers` with `is_broad`, `suppresses`, and `action`; fixtures `tests/fixtures/python/maintainability/exception_and_debug_{positive,negative}.txt`; tests `test_python_exception_and_debug_rules_{positive,negative}`.
- `eval_exec_usage`: structured `calls`; fixtures `tests/fixtures/python/maintainability/exception_and_debug_{positive,negative}.txt`; tests `test_python_exception_and_debug_rules_{positive,negative}`.
- `print_debugging_leftover`: structured `calls` plus file/function test suppression metadata; fixtures `tests/fixtures/python/maintainability/exception_and_debug_{positive,negative}.txt` and `tests/fixtures/python/rule_pack_test_only.txt`; tests `test_python_exception_and_debug_rules_{positive,negative}` and baseline suppression coverage.
- `none_comparison`: structured `none_comparison_lines`; fixtures `tests/fixtures/python/maintainability/business_rules_{positive,negative}.txt` and `tests/fixtures/python/phase4_{positive,negative}.txt`; tests baseline phase-4 coverage.
- `side_effect_comprehension`: structured `side_effect_comprehension_lines`; fixtures `tests/fixtures/python/maintainability/business_rules_{positive,negative}.txt` and `tests/fixtures/python/phase4_{positive,negative}.txt`; tests baseline phase-4 coverage.
- `redundant_return_none`: structured `redundant_return_none_lines`; fixtures `tests/fixtures/python/maintainability/business_rules_{positive,negative}.txt` and `tests/fixtures/python/phase4_{positive,negative}.txt`; tests baseline phase-4 coverage.
- `hardcoded_path_string`: structured `local_strings`; fixtures `tests/fixtures/python/phase4_{positive,negative}.txt`; tests baseline phase-4 coverage.
- `hardcoded_business_rule`: structured `body_text` and business-context imports/calls; fixtures `tests/fixtures/python/maintainability/business_rules_{positive,negative}.txt`; tests `test_python_phase5_business_magic_and_utility_{rules,suppressions}`.
- `magic_value_branching`: structured `body_text`; fixtures `tests/fixtures/python/maintainability/business_rules_{positive,negative}.txt`; tests `test_python_phase5_business_magic_and_utility_{rules,suppressions}`.
- `reinvented_utility`: structured `body_text` plus file `imports`; fixtures `tests/fixtures/python/maintainability/business_rules_{positive,negative}.txt`; tests `test_python_phase5_business_magic_and_utility_{rules,suppressions}`.
- `variadic_public_api`: structured `has_varargs`, `has_kwargs`, and function fingerprint metadata; fixtures `tests/fixtures/python/phase4_{positive,negative}.txt`; tests baseline phase-4 coverage.
- `builtin_reduction_candidate`: structured `builtin_candidate_lines`; fixtures `tests/fixtures/python/performance/loop_shapes_{positive,negative}.txt` and `tests/fixtures/python/phase4_{positive,negative}.txt`; tests grouped performance coverage plus baseline phase-4 coverage.
- `network_boundary_without_timeout`: structured `calls`, file `imports`, and function `body_text`; fixtures `tests/fixtures/python/maintainability/boundaries_{positive,negative}.txt` and parser-contract fixtures `boundary_network_{positive,negative}.txt`; tests `test_python_phase5_boundary_robustness_{rules,suppressions}` and parser boundary coverage.
- `environment_boundary_without_fallback`: structured `calls`, file `imports`, and function `body_text`; fixtures `tests/fixtures/python/maintainability/boundaries_{positive,negative}.txt` and parser-contract fixtures `boundary_config_{positive,negative}.txt`; tests `test_python_phase5_boundary_robustness_{rules,suppressions}` and parser boundary coverage.
- `external_input_without_validation`: structured `calls`, file `imports`, and function `body_text`; fixtures `tests/fixtures/python/maintainability/boundaries_{positive,negative}.txt` and parser-contract fixtures `boundary_cli_{positive,negative}.txt`; tests `test_python_phase5_boundary_robustness_{rules,suppressions}` and parser boundary coverage.
- `broad_exception_handler`: structured `exception_handlers`; fixtures `tests/fixtures/python/maintainability/exception_and_debug_{positive,negative}.txt` and parser-contract fixtures `exception_shapes_{positive,negative}.txt`; tests grouped exception/debug coverage and parser exception coverage.
- `missing_context_manager`: structured `missing_context_manager_lines`; fixtures `tests/fixtures/python/phase4_{positive,negative}.txt`; tests baseline phase-4 coverage and parser phase-4 evidence coverage.
- `public_api_missing_type_hints`: structured `has_complete_type_hints` and function signature metadata; fixtures `tests/fixtures/python/maintainability/type_hints_{positive,negative}.txt` and `tests/fixtures/python/phase4_{positive,negative}.txt`; tests parser type-hint contract coverage and baseline phase-4 coverage.
- `commented_out_code`: structured file `comments`; fixtures `tests/fixtures/python/maintainability/exception_and_debug_{positive,negative}.txt`; tests `test_python_exception_and_debug_rules_{positive,negative}`.
- `mixed_sync_async_module`: structured file `imports`, function `calls`, `is_async`, and file-level function inventory; fixtures `tests/fixtures/python/phase4_{positive,negative}.txt`; tests baseline phase-4 coverage.

## Family Fixture and Test Owners

- Async, I/O, and resource handling: `tests/fixtures/python/performance/async_boundaries_{positive,negative}.txt`, parser fixtures `performance/async_io_{positive,negative}.txt`, and `maintainability/boundaries_{positive,negative}.txt`; owned by `tests/integration_scan/python/phase5_rules.rs` plus parser contract tests in `src/analysis/python/parser/tests.rs`.
- Loop-shape heuristics: `tests/fixtures/python/performance/loop_shapes_{positive,negative}.txt`; owned by `tests/integration_scan/python/phase5_rules.rs`.
- Exception and debugging leftovers: `tests/fixtures/python/maintainability/exception_and_debug_{positive,negative}.txt`; owned by `tests/integration_scan/python/phase5_rules.rs`.
- Boundary robustness: `tests/fixtures/python/maintainability/boundaries_{positive,negative}.txt` plus parser contract fixtures `boundary_{network,config,cli}_{positive,negative}.txt`; owned by `tests/integration_scan/python/phase5_rules.rs` and `src/analysis/python/parser/tests.rs`.
- Business, literal, and branch heuristics: `tests/fixtures/python/maintainability/business_rules_{positive,negative}.txt`; owned by `tests/integration_scan/python/phase5_rules.rs`.

## Detailed Checklist

### 1. Build a rule-to-evidence map before changing heuristics

- [x] For each rule, list the exact parser evidence it depends on today.
- [x] Mark whether the rule relies on structured evidence, raw `body_text`, imports, comments, or class summaries.
- [x] Reject any new heuristic that would require reparsing Python text inside the heuristic layer.
- [x] Capture missing-evidence requests as parser work in plan 1 first.

### 2. Split work into rule families

#### Async, I/O, and resource handling

- [x] Revisit `blocking_sync_io_in_async` for broader sync-I/O coverage and better suppressions.
- [x] Revisit `full_dataset_load` for `open(...).read*` and `Path(...).read_*` variants.
- [x] Revisit `network_boundary_without_timeout` for timeout-bearing versus timeout-free call shapes.
- [x] Revisit `missing_context_manager` for open-file and resource-handling shapes.
- [x] Revisit `mixed_sync_async_module` to ensure mixed imports and async definitions are not over-flagged.

#### Control-flow and loop-shape heuristics

- [x] Revisit `string_concat_in_loop`.
- [x] Revisit `list_materialization_first_element`.
- [x] Revisit `deque_candidate_queue`.
- [x] Revisit `temporary_collection_in_loop`.
- [x] Revisit `recursive_traversal_risk`.
- [x] Revisit `list_membership_in_loop`.
- [x] Revisit `repeated_len_in_loop`.
- [x] Revisit `builtin_reduction_candidate`.

#### Safety and debugging leftovers

- [x] Revisit `exception_swallowed` and `broad_exception_handler` together so they use a consistent exception taxonomy.
- [x] Revisit `eval_exec_usage` and ensure test-only code remains suppressed.
- [x] Revisit `print_debugging_leftover` so intentional CLI prints are not confused with debug prints.
- [x] Revisit `commented_out_code` alongside comment-summary extraction.

#### API clarity and boundary robustness

- [x] Revisit `variadic_public_api`.
- [x] Revisit `public_api_missing_type_hints`.
- [x] Revisit `environment_boundary_without_fallback`.
- [x] Revisit `external_input_without_validation`.

#### Literal- and branch-driven maintainability heuristics

- [x] Revisit `none_comparison`.
- [x] Revisit `side_effect_comprehension`.
- [x] Revisit `redundant_return_none`.
- [x] Revisit `hardcoded_path_string`.
- [x] Revisit `hardcoded_business_rule`.
- [x] Revisit `magic_value_branching`.
- [x] Revisit `reinvented_utility`.

### 3. Require positive and negative `.txt` fixtures per rule family

- [x] Add fixture pairs for async and I/O rules:
  - [x] `tests/fixtures/python/performance/async_boundaries_positive.txt`
  - [x] `tests/fixtures/python/performance/async_boundaries_negative.txt`
- [x] Add fixture pairs for loop-shape rules:
  - [x] `tests/fixtures/python/performance/loop_shapes_positive.txt`
  - [x] `tests/fixtures/python/performance/loop_shapes_negative.txt`
- [x] Add fixture pairs for exception and debug rules:
  - [x] `tests/fixtures/python/maintainability/exception_and_debug_positive.txt`
  - [x] `tests/fixtures/python/maintainability/exception_and_debug_negative.txt`
- [x] Add fixture pairs for boundary robustness rules:
  - [x] `tests/fixtures/python/maintainability/boundaries_positive.txt`
  - [x] `tests/fixtures/python/maintainability/boundaries_negative.txt`
- [x] Add fixture pairs for branch and literal rules:
  - [x] `tests/fixtures/python/maintainability/business_rules_positive.txt`
  - [x] `tests/fixtures/python/maintainability/business_rules_negative.txt`

### 4. Make integration tests match rule families instead of one giant catch-all case

- [x] Keep baseline function-level tests in `tests/integration_scan/python/baseline.rs`.
- [x] Place phase-5 expansion tests in `tests/integration_scan/python/phase5_rules.rs`.
- [x] Add one integration test per family instead of one test per rule where possible.
- [x] Keep at least one dedicated assertion per rule ID so failures stay local and obvious.

### 5. Add suppression-focused negative cases

- [x] Every new or revised heuristic must include a suppression-oriented negative case.
- [x] Negative cases must cover test files, small helpers, honest transformer functions, centralized constants, explicit timeouts, and explicit validation.
- [x] Prefer a named negative fixture over embedding the suppression scenario inline.

## Required Test Cases

### Performance-focused integration cases

- [x] `test_python_performance_rule_family_positive`
- [x] `test_python_performance_rule_family_negative`
- [x] `test_python_async_boundary_rules_positive`
- [x] `test_python_async_boundary_rules_negative`

### Maintainability-focused integration cases

- [x] `test_python_exception_and_debug_rules_positive`
- [x] `test_python_exception_and_debug_rules_negative`
- [x] `test_python_phase5_boundary_robustness_rules`
- [x] `test_python_phase5_boundary_robustness_suppressions`
- [x] `test_python_phase5_business_magic_and_utility_rules`
- [x] `test_python_phase5_business_magic_and_utility_suppressions`

## Review Gates

- [x] Rule message text names the actual smell and does not oversell confidence.
- [x] Evidence strings point to the exact syntactic reason the rule fired.
- [x] Test-only code remains suppressed where intended.
- [x] Function-level heuristics do not silently rely on repository-wide state.
- [x] Parser changes needed for these rules are tracked separately from heuristic-message tuning.

## Acceptance Criteria

- [x] Every function-level heuristic has an explicit fixture owner and test owner.
- [x] Positive and negative fixture coverage exists for each rule family.
- [x] The function-level heuristic plan reflects only the rules that exist in `performance.rs` and `maintainability.rs`.
- [x] No plan item requires adding Python code to the application itself.

## Definition of Done

- [x] The function-level roadmap is grouped by current heuristic families.
- [x] Fixture creation is explicitly text-based.
- [x] Integration tests are planned around the new module split.
- [x] The next implementation pass can be scheduled rule-family by rule-family without re-scoping the repo.