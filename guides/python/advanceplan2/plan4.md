# Plan 4 - Boundary Safety And Serialization Slop (Python)

Date: 2026-03-30

## Objective

Add a conservative Python boundary-safety pack for dangerous serialization and subprocess helpers that frequently appear in example-driven or AI-authored code.

## Existing Coverage Explicitly Excluded

This plan must not duplicate:

- `eval_exec_usage`
- `external_input_without_validation`
- `missing_context_manager`
- `hardcoded_secret`

This phase targets unsafe helper APIs at the trust boundary rather than generic input validation or context-manager guidance.

## Candidate Rule Inventory

- [ ] `unsafe_yaml_loader`
  - Detect `yaml.load(...)`, `full_load(...)`, or similar unguarded loader usage where safe loading is more appropriate.
- [ ] `pickle_deserialization_boundary`
  - Detect `pickle.loads`, `pickle.load`, or direct `dill`-style deserialization in non-test code.
- [ ] `subprocess_shell_true`
  - Detect `subprocess.run`, `Popen`, or related calls with `shell=True` in production code.
- [ ] `tar_extractall_unfiltered`
  - Detect `tarfile.extractall(...)` without visible filtering or path validation.
- [ ] `tempfile_without_cleanup`
  - Detect named temporary files or directories created in ordinary code paths with no visible cleanup or context-manager ownership.

## Why These Rules Belong In Advance Plan 2

- [ ] They are high-signal and easy to review.
- [ ] They represent a different class of AI slop than the current maintainability and duplication families.
- [ ] The required evidence is mostly local call classification plus small argument inspection.

## Parser And Evidence Work

- [ ] Expand Python call classification to preserve callee names and named arguments for:
  - [ ] `yaml`
  - [ ] `pickle` and similar serializers
  - [ ] `subprocess`
  - [ ] `tarfile`
  - [ ] `tempfile`
- [ ] Preserve keyword-argument text such as `shell=True` and common safe-loader variants.
- [ ] Track context-manager ownership around temporary resources where possible.

## Implementation Checklist

- [ ] Add parser tests for keyword-argument capture and module alias handling.
- [ ] Implement the boundary-safety rule family in `src/heuristics/python/maintainability.rs` or a new sub-module if separation improves readability.
- [ ] Add suppressions for:
  - [ ] test-only fixtures
  - [ ] explicitly documented safe wrappers
  - [ ] one-shot cleanup helpers using context managers or explicit `unlink()`/`cleanup()` calls
- [ ] Keep severity messaging factual and boundary-focused rather than alarmist.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `unsafe_yaml_loader_positive.txt`
  - [ ] `pickle_boundary_positive.txt`
  - [ ] `subprocess_shell_true_positive.txt`
  - [ ] `tar_extractall_positive.txt`
  - [ ] `tempfile_without_cleanup_positive.txt`
- [ ] Negative fixtures:
  - [ ] `yaml.safe_load(...)`
  - [ ] explicit allow-listed test pickle usage
  - [ ] subprocess calls with argument lists and `shell=False`
  - [ ] tar extraction with validation or safe filtering
  - [ ] temporary resources managed with `with` blocks or explicit cleanup

## Acceptance Criteria

- [ ] Each rule is backed by clear positive and negative fixtures.
- [ ] Alias imports and keyword-argument spelling do not break detection.
- [ ] The rule family stays conservative enough for default-on use.

## Non-Goals

- [ ] Full taint tracking from external inputs to dangerous APIs.
- [ ] Broad security auditing across third-party frameworks.
- [ ] Replacing dedicated security tooling.