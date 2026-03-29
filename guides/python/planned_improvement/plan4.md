# Plan 4 - Python Test Harness, Fixture Migration, and Rollout

Date: 2026-03-29

## Objective

Make the Python roadmap operational by defining how tests, fixtures, and integration modules should be organized as Python support grows. The immediate concern was that the old flat Python integration file had become too large. The fix is the current nested Rust integration module layout plus `.txt` fixtures.

## Immediate Repository Constraints

- Python support is implemented in Rust. We should not add Python application scripts as part of this plan.
- Test inputs should live in `.txt` fixtures under `tests/fixtures/python`.
- `tests/integration_scan.rs` remains the top-level test harness and shared helper location.
- New Python integration modules should be sibling Rust files under `tests/integration_scan/`.

## Target Integration Layout

- [ ] `tests/integration_scan.rs`
  - shared fixture macros
  - shared temp-workspace helpers
  - `mod python;`
- [ ] `tests/integration_scan/python/mod.rs`
  - module split and shared Python test helper
- [ ] `tests/integration_scan/python/baseline.rs`
  - parser smoke coverage
  - syntax-error coverage
  - baseline rule-pack coverage
  - phase-4 coverage
  - mixed-language smoke coverage
  - hallucination/import-resolution coverage
- [ ] `tests/integration_scan/python/phase5_rules.rs`
  - phase-5 expansion coverage and related suppressions

## Detailed Checklist

### 1. Split the current Python integration surface

- [ ] Keep phase-5 tests out of `tests/integration_scan/python/baseline.rs`.
- [ ] Register the new Rust module in `tests/integration_scan.rs`.
- [ ] Keep shared helpers in the root integration harness so there is no duplication.
- [ ] Keep import lists minimal in each child module so moved tests do not leave stale imports behind.

### 2. Define file ownership so the split stays stable

- [ ] `baseline.rs` owns baseline, parser, phase-4, and hallucination tests.
- [ ] `phase5_rules.rs` owns phase-5 rule expansion and suppression tests.
- [ ] If Python repo-level coverage grows again, introduce a third module such as `python_repo.rs` instead of re-growing one file past the limit.

### 3. Standardize fixture migration away from inline source blobs

- [ ] Prefer fixture-backed sources for reusable test cases.
- [ ] Reserve inline source generation only for tests where size itself is the heuristic trigger.
- [ ] For repo-level scenarios, assemble temp workspaces from multiple `.txt` fixtures rather than embedding the same source repeatedly.

### 4. Define a fixture naming convention that matches test responsibilities

- [ ] Baseline parser fixtures: `parser_*`
- [ ] Function-level heuristic fixtures: `performance_*`, `maintainability_*`
- [ ] File-level heuristic fixtures: `structure_*`, `ai_smells_*`
- [ ] Repo-level heuristic fixtures: `duplication_*`, `repo_*`, `hallucination_*`
- [ ] Positive and negative pairs must stay adjacent by name.

### 5. Make rollout incremental and low-risk

- [ ] First land the integration-file split.
- [ ] Next convert the most repeated inline Python sources into `.txt` fixtures.
- [ ] Then add new heuristic coverage family by family.
- [ ] Avoid mixing test-harness refactors with unrelated heuristic behavior changes in the same PR when possible.

## Proposed Conversion Batches

### Batch A - Structural cleanup

- [ ] Add `tests/integration_scan/python/phase5_rules.rs`.
- [ ] Update `tests/integration_scan.rs`.
- [ ] Trim `tests/integration_scan/python/baseline.rs` as coverage grows by adding sibling modules instead of regrowing a catch-all file.

### Batch B - Fixture extraction

- [ ] Extract reusable phase-5 positive cases into `tests/fixtures/python/structure/`, `tests/fixtures/python/duplication/`, and `tests/fixtures/python/maintainability/`.
- [ ] Extract reusable hallucination cases into `tests/fixtures/python/hallucination/`.
- [ ] Leave only large generated-module scenarios inline if size is the point.

### Batch C - Coverage tightening

- [ ] Add explicit negative fixtures for every new phase-5 rule family.
- [ ] Add per-family integration tests that assert grouped rule IDs.
- [ ] Add parser unit tests for any evidence introduced to support new heuristics.

## Review Gates

- [ ] No new Python integration file should grow into another 1500-line catch-all.
- [ ] Every moved test must continue to use shared helper functions from `tests/integration_scan.rs`.
- [ ] New fixtures must be text files, not checked-in `.py` sources under the root fixture directory.
- [ ] Test refactors must preserve coverage for both positive detections and suppression cases.

## Acceptance Criteria

- [ ] The Python integration suite is split across multiple Rust modules.
- [ ] The plan makes fixture-backed coverage the default path for future Python heuristic work.
- [ ] The roadmap is explicitly tied to existing Python support in the repository.
- [ ] No plan step relies on adding Python application scripts.

## Definition of Done

- [ ] The integration split is reflected in both documentation and code.
- [ ] Fixture migration has a clear batch order.
- [ ] Future Python heuristic work can proceed without re-litigating where tests and fixtures belong.