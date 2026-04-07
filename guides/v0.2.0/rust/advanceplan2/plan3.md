# Plan 3 - Serde And Wire Contract Robustness (Rust)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verification passed with `cargo test rust_advanceplan2`.
- [x] Parser evidence regression passed with `cargo test test_collects_advanceplan2_rust_summaries`.

## Objective

Add a Rust boundary rule family for serde configuration choices that make wire formats ambiguous, overly permissive, or hard to evolve safely.

## Existing Coverage Explicitly Excluded

This plan does not duplicate:

- `rust_serde_sensitive_deserialize`
- `rust_serde_sensitive_serialize`
- `rust_debug_secret`

## Shipped Rules

- [x] `rust_serde_untagged_enum_boundary`
- [x] `rust_serde_default_on_required_field`
- [x] `rust_serde_flatten_catchall`
- [x] `rust_serde_unknown_fields_allowed`
- [x] `rust_stringly_typed_enum_boundary`

## Implementation Notes

- [x] Reused raw struct, enum, and field attribute text instead of building a separate serde model.
- [x] Kept `rust_serde_unknown_fields_allowed` conservative by limiting it to strict-looking config, settings, request, and params shapes.
- [x] Scoped stringly boundary checks to public or serde-facing models with enum-like field names.

## Parser And Evidence Work

- [x] Added Rust enum summaries with variant counts, derive information, and raw attribute text.
- [x] Preserved struct- and field-level attribute text for `default`, `flatten`, and `deny_unknown_fields` checks.
- [x] Reused the Rust parser regression in `src/analysis/rust/parser.rs` for enum and serde-attribute coverage.

## Fixtures And Tests

- [x] Added grouped serde positive and clean fixtures under `tests/fixtures/rust/advanceplan2/`.
- [x] Added grouped integration coverage in `tests/integration_scan/rust_advanceplan2.rs`.
- [x] Validated tagged enums, explicit enums, and stricter contracts as representative clean coverage.

## Acceptance

- [x] Findings name the serde attribute or field shape that weakens the boundary contract.
- [x] The rules stay explainable without procedural-macro expansion.
- [x] Secret-field-specific findings remain separate from the new wire-contract pack.

## Non-Goals

- [x] Full schema evolution analysis across versions.
- [x] Generating or validating external OpenAPI or JSON Schema documents.
- [x] Replacing dedicated serialization compatibility tests.