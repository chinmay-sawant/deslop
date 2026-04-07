# Plan 2 - Mutable Defaults And Model Contracts (Python)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verification passed with `cargo test python_advanceplan2`.
- [x] Parser evidence regression passed with `cargo test test_python_advanceplan2_parser_evidence`.

## Objective

Add a Python rule family for mutable defaults, dataclass model hazards, and weak public type contracts that are not part of the currently shipped maintainability and structure rules.

## Existing Coverage Explicitly Excluded

This plan does not duplicate:

- `variadic_public_api`
- `public_api_missing_type_hints`
- `god_class`
- `too_many_instance_attributes`
- `hardcoded_business_rule`
- `magic_value_branching`

## Shipped Rules

- [x] `mutable_default_argument`
- [x] `dataclass_mutable_default`
- [x] `dataclass_heavy_post_init`
- [x] `option_bag_model`
- [x] `public_any_type_leak`
- [x] `typeddict_unchecked_access`

## Implementation Notes

- [x] Reused shared `signature_text` for mutable-default and wide-contract checks.
- [x] Added Python model summaries for decorators, base classes, method names, field annotations, and field defaults.
- [x] Kept `public_any_type_leak` at `Info` and suppressed serializer or migration-style paths.

## Parser And Evidence Work

- [x] Preserved function signature text for default-expression checks.
- [x] Extended field extraction so `TypedDict` declaration lines contribute model metadata even without default values.
- [x] Kept dataclass and `TypedDict` evidence distinct so messages can stay specific.

## Fixtures And Tests

- [x] Added parser coverage in `src/analysis/python/parser/tests.rs`.
- [x] Added grouped positive and clean fixtures under `tests/fixtures/python/integration/advanceplan2/`.
- [x] Added grouped integration coverage in `tests/integration_scan/python/advanceplan2.rs`.

## Acceptance

- [x] Findings explain the contract hazard rather than only naming syntax.
- [x] Dataclass and `TypedDict` paths are handled through separate evidence checks.
- [x] Small, honest option sets stay quiet on representative clean fixtures.

## Non-Goals

- [x] Full runtime type checking or mypy-style inference.
- [x] Framework-specific model semantics in the first iteration.
- [x] Penalizing every use of `Any` regardless of context.