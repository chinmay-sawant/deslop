# Plan 4 - Builder, Config, And State-Machine Slop (Rust)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verification passed with `cargo test rust_advanceplan2`.
- [x] Parser evidence regression passed with `cargo test test_collects_advanceplan2_rust_summaries`.

## Objective

Add a Rust rule family for builder misuse, option-bag configuration types, and multi-flag state encoding that often compiles cleanly but leaves invalid states easy to construct.

## Existing Coverage Explicitly Excluded

This plan does not duplicate:

- `rust_domain_impossible_combination`
- `rust_domain_default_produces_invalid`
- `rust_domain_optional_secret_default`
- `rust_domain_raw_primitive`

## Shipped Rules

- [x] `rust_option_bag_config`
- [x] `rust_builder_without_validate`
- [x] `rust_constructor_many_flags`
- [x] `rust_partial_init_escape`
- [x] `rust_boolean_state_machine`

## Implementation Notes

- [x] Reused struct summaries and parsed function signatures rather than inventing impl-only metadata.
- [x] Kept `rust_option_bag_config` thresholded to larger config-like structs with many `Option<_>` fields and no obvious validation method.
- [x] Used conservative body-local markers for `build()` validation and partial-initialization escape checks.

## Parser And Evidence Work

- [x] Reused shared Rust function signature text for constructor-like boolean-flag detection.
- [x] Reused struct field counts plus existing `Option` and `bool` field summaries for config and state-shape checks.
- [x] Kept validation reasoning local to method names and body markers such as `validate`, `ok_or`, `Err`, and `missing`.

## Fixtures And Tests

- [x] Added grouped builder and config positive and clean fixtures under `tests/fixtures/rust/advanceplan2/`.
- [x] Added grouped integration coverage in `tests/integration_scan/rust_advanceplan2.rs`.
- [x] Validated enum-backed states, validated builders, and narrow constructors as representative clean coverage.

## Acceptance

- [x] Findings explain the invalid-state risk rather than only counting fields.
- [x] The rule family stays distinct from the shipped Rust domain-modeling pack.
- [x] The shipped heuristics remain modest, conservative, and reusable for future Rust API-shape work.

## Non-Goals

- [x] Full typestate verification.
- [x] Ownership-proof analysis across multiple impl blocks or crates.
- [x] Enforcing builders for every configuration type.