# Plan 3 - Serde And Wire Contract Robustness (Rust)

Date: 2026-03-30

## Objective

Add a Rust boundary rule family for serde configuration choices that make wire formats ambiguous, overly permissive, or hard to evolve safely.

## Existing Coverage Explicitly Excluded

This plan must not duplicate:

- `rust_serde_sensitive_deserialize`
- `rust_serde_sensitive_serialize`
- `rust_debug_secret`

The current domain-modeling pack already covers sensitive-field handling. This phase focuses on general wire-contract clarity and evolution hazards.

## Candidate Rule Inventory

- [ ] `rust_serde_untagged_enum_boundary`
  - Detect externally facing enums derived with `#[serde(untagged)]` where variant ambiguity is likely to confuse input or output handling.
- [ ] `rust_serde_default_on_required_field`
  - Detect fields that use `#[serde(default)]` even though the field name and surrounding type shape suggest the value is required for correctness.
- [ ] `rust_serde_flatten_catchall`
  - Detect `#[serde(flatten)]` maps or catch-all containers that absorb unknown fields into loosely typed bags.
- [ ] `rust_serde_unknown_fields_allowed`
  - Detect configuration-like structs that deserialize without `deny_unknown_fields` despite looking like strict config or request contracts.
- [ ] `rust_stringly_typed_enum_boundary`
  - Detect string fields with enum-like naming patterns where a real enum would make the wire contract more stable and explicit.

## Why These Rules Belong In Advance Plan 2

- [ ] They sit at the intersection of API design and data modeling, but are distinct from current secret-field rules.
- [ ] They are easy to miss in code review because the derive attributes look ergonomic and harmless.
- [ ] The candidate rules rely on parser-visible attributes and field names, not macro expansion or runtime schemas.

## Parser And Evidence Work

- [ ] Extend Rust parser summaries to retain serde-related attributes on structs, enums, and fields.
- [ ] Preserve enum variant counts and field names where helpful for ambiguity messaging.
- [ ] Retain attribute text such as `untagged`, `default`, `flatten`, `deny_unknown_fields`, and rename annotations.
- [ ] Reuse struct summaries and field-type text instead of building a separate serde model.

## Implementation Checklist

- [ ] Add parser tests for struct-, enum-, and field-level serde attributes.
- [ ] Implement the wire-contract heuristics in a dedicated Rust serde-focused module.
- [ ] Keep `rust_serde_unknown_fields_allowed` conservative by restricting it to configuration, request, and settings-like type names at first.
- [ ] Add suppressions for intentionally permissive ingestion types and migration shims.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `serde_untagged_enum_positive.rs.txt`
  - [ ] `serde_default_required_positive.rs.txt`
  - [ ] `serde_flatten_catchall_positive.rs.txt`
  - [ ] `serde_unknown_fields_positive.rs.txt`
  - [ ] `stringly_typed_enum_boundary_positive.rs.txt`
- [ ] Negative fixtures:
  - [ ] tagged enums with stable discriminators
  - [ ] truly optional defaultable fields
  - [ ] validated compatibility shims
  - [ ] explicit enum types in request or config models

## Acceptance Criteria

- [ ] Findings name the serde attribute or field shape that makes the boundary ambiguous.
- [ ] The rules stay explainable without procedural-macro expansion.
- [ ] Secret-field-specific findings remain separate from the new wire-contract pack.

## Non-Goals

- [ ] Full schema evolution analysis across versions.
- [ ] Generating or validating external OpenAPI or JSON Schema documents.
- [ ] Replacing dedicated serialization compatibility tests.