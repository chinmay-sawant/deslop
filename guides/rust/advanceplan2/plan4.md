# Plan 4 - Builder, Config, And State-Machine Slop (Rust)

Date: 2026-03-30

## Objective

Add a Rust rule family for builder misuse, option-bag configuration types, and multi-flag state encoding that often compiles cleanly but leaves invalid states easy to construct.

## Existing Coverage Explicitly Excluded

This plan must not duplicate:

- `rust_domain_impossible_combination`
- `rust_domain_default_produces_invalid`
- `rust_domain_optional_secret_default`
- `rust_domain_raw_primitive`

The current domain pack already covers a subset of invalid-state modeling. This phase expands into builder workflows and broader configuration/state encodings.

## Candidate Rule Inventory

- [ ] `rust_option_bag_config`
  - Detect config or request structs with many `Option<_>` fields and little evidence of a validating constructor or build step.
- [ ] `rust_builder_without_validate`
  - Detect builder types that collect configuration state but expose `build()` without an obvious validation or required-field check.
- [ ] `rust_constructor_many_flags`
  - Detect constructors or public associated functions with multiple boolean parameters that encode behavior through flags.
- [ ] `rust_partial_init_escape`
  - Detect partially configured structs that are returned or stored before an explicit finalize or build step.
- [ ] `rust_boolean_state_machine`
  - Detect state encoded through multiple booleans or `Option` pairs when an enum or dedicated state type would be clearer.

## Why These Rules Belong In Advance Plan 2

- [ ] They address a recurrent class of generated Rust code: configuration types that grow by accretion rather than by explicit state modeling.
- [ ] They complement the existing domain-modeling pack without restating the same secret or default-value findings.
- [ ] The rules are realistically implementable with struct summaries, impl summaries, and simple method-shape analysis.

## Parser And Evidence Work

- [ ] Extend Rust summaries with lightweight impl and method metadata:
  - [ ] builder-like type names
  - [ ] presence of `build`, `finish`, `validate`, or `new` methods
  - [ ] constructor parameter counts and boolean parameter positions
- [ ] Preserve field counts for `Option<_>` and boolean fields on config-like structs.
- [ ] Track whether a `build()` method visibly checks for missing required state before constructing the output type.

## Implementation Checklist

- [ ] Add parser tests for impl summaries and constructor signature capture.
- [ ] Implement config and builder heuristics in a dedicated Rust module.
- [ ] Use conservative thresholds so small builder types or honest compatibility shims do not trigger.
- [ ] Add suppressions for generated schema types, serde migration shells, and clearly validated constructors.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `option_bag_config_positive.rs.txt`
  - [ ] `builder_without_validate_positive.rs.txt`
  - [ ] `constructor_many_flags_positive.rs.txt`
  - [ ] `partial_init_escape_positive.rs.txt`
  - [ ] `boolean_state_machine_positive.rs.txt`
- [ ] Negative fixtures:
  - [ ] enum-backed state models
  - [ ] builders with explicit validation and typed required fields
  - [ ] narrow constructors with one honest flag
  - [ ] compatibility DTOs kept private and wrapped by validated constructors

## Acceptance Criteria

- [ ] Findings explain the invalid-state risk, not just the raw field count.
- [ ] The rule family stays clearly separate from the shipped domain-modeling pack.
- [ ] Parser enrichment remains modest and reusable for future Rust API-shape heuristics.

## Non-Goals

- [ ] Full typestate verification.
- [ ] Ownership-proof analysis across multiple impl blocks or crates.
- [ ] Enforcing builders for every configuration type.