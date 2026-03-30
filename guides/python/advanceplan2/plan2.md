# Plan 2 - Mutable Defaults And Model Contracts (Python)

Date: 2026-03-30

## Objective

Add a Python rule family for mutable defaults, dataclass model hazards, and weak public type contracts that are not part of the currently shipped maintainability and structure rules.

## Existing Coverage Explicitly Excluded

This plan must not duplicate:

- `variadic_public_api`
- `public_api_missing_type_hints`
- `god_class`
- `too_many_instance_attributes`
- `hardcoded_business_rule`
- `magic_value_branching`

The new work targets default-value and type-contract hazards rather than general structure size or public API completeness.

## Candidate Rule Inventory

- [ ] `mutable_default_argument`
  - Detect list, dict, set, or other mutable literals used directly as default function arguments.
- [ ] `dataclass_mutable_default`
  - Detect dataclass fields that use mutable defaults instead of `default_factory`.
- [ ] `dataclass_heavy_post_init`
  - Detect `__post_init__` methods that perform I/O, network calls, subprocesses, or heavy collaborator construction.
- [ ] `option_bag_model`
  - Detect dataclasses or model-like classes with many optional fields or boolean toggles that encode too many invalid combinations.
- [ ] `public_any_type_leak`
  - Detect public functions or model fields that use `Any`, `object`, or wide `dict[str, Any]` contracts where the rest of the signature suggests a specific domain shape.
- [ ] `typeddict_unchecked_access`
  - Detect `TypedDict`-style access patterns that immediately index optional or open-ended keys with no guard.

## Why These Rules Belong In Advance Plan 2

- [ ] These are frequent AI-authored Python mistakes because the code is syntactically correct and easy to generate from examples.
- [ ] The current Python pack checks whether public type hints exist, but not whether the default values and model shape undermine the contract.
- [ ] The candidate rules stay local enough to implement with parser enrichment instead of new runtime systems.

## Parser And Evidence Work

- [ ] Extend function signature parsing to preserve default-value expressions.
- [ ] Extend decorator and class evidence so dataclass-like models are easy to classify.
- [ ] Capture field default expressions, `default_factory` usage, and annotation text for dataclass fields.
- [ ] Preserve `Any`, `object`, `dict[str, Any]`, `Optional[...]`, and `TypedDict` annotation text in shared function and class summaries.

## Implementation Checklist

- [ ] Add parser unit tests for mutable defaults and dataclass field metadata.
- [ ] Add conservative thresholds for `option_bag_model` so small, honest option sets do not trigger.
- [ ] Implement the rule family with clear suppression cases for test helpers, migration models, and serializer glue code.
- [ ] Keep `public_any_type_leak` at `Info` until real-repo calibration proves it is precise enough.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `mutable_default_argument_positive.txt`
  - [ ] `dataclass_mutable_default_positive.txt`
  - [ ] `dataclass_post_init_positive.txt`
  - [ ] `option_bag_model_positive.txt`
  - [ ] `public_any_type_leak_positive.txt`
  - [ ] `typeddict_unchecked_access_positive.txt`
- [ ] Negative fixtures:
  - [ ] immutable default sentinel pattern
  - [ ] dataclass `default_factory` usage
  - [ ] lightweight `__post_init__` normalization only
  - [ ] narrow and validated public type contracts
  - [ ] guarded optional-key access on typed dictionaries

## Acceptance Criteria

- [ ] Findings clearly explain the contract hazard rather than just naming the syntax.
- [ ] Dataclass and plain-class paths are handled separately when needed.
- [ ] The rule family stays conservative and fixture-driven.

## Non-Goals

- [ ] Full runtime type checking or mypy-style inference.
- [ ] Framework-specific model semantics in the first iteration.
- [ ] Penalizing every use of `Any` regardless of context.