# Plan 2 - Interior Mutability And Shared State Exposure (Rust)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verification passed with `cargo test rust_advanceplan2`.
- [x] Parser evidence regression passed with `cargo test test_collects_advanceplan2_rust_summaries`.

## Objective

Add a Rust rule family for over-exposed interior mutability and global shared-state shapes that often show up as quick fixes in generated code.

## Existing Coverage Explicitly Excluded

This plan does not duplicate:

- `rust_lock_across_await`
- `rust_async_std_mutex_await`
- `rust_tokio_mutex_unnecessary`
- `rust_async_lock_order_cycle`
- `rust_domain_impossible_combination`

## Shipped Rules

- [x] `rust_pub_interior_mutability_field`
- [x] `rust_global_lock_state`
- [x] `rust_arc_mutex_option_state`
- [x] `rust_mutex_wrapped_collection`
- [x] `rust_rc_refcell_domain_model`

## Implementation Notes

- [x] Reused struct summaries and new static summaries instead of building a second Rust type model.
- [x] Kept `rust_pub_interior_mutability_field` scoped to directly public fields so public structs with private lock internals do not trip the rule.
- [x] Added conservative suppressions for graph-like and UI-oriented ownership shapes before flagging `Rc<RefCell<_>>`.

## Parser And Evidence Work

- [x] Added `rust_statics` summaries so global lock state can be detected without ad hoc source rescans.
- [x] Preserved nested generic type text for fields and statics such as `Arc<Mutex<Option<State>>>`.
- [x] Reused the Rust parser regression in `src/analysis/rust/parser.rs` for static and field-summary coverage.

## Fixtures And Tests

- [x] Added grouped shared-state positive and clean fixtures under `tests/fixtures/rust/advanceplan2/`.
- [x] Added grouped integration coverage in `tests/integration_scan/rust_advanceplan2.rs`.
- [x] Validated clean coverage for private lock fields, immutable handles, and test-style scaffolding.

## Acceptance

- [x] Findings explain the exposed shared-state shape rather than only naming containers.
- [x] The rule family stays separate from runtime lock-misuse checks already shipped elsewhere.
- [x] Ordinary internal synchronization for small helper types stays quiet on representative clean fixtures.

## Non-Goals

- [x] Proving contention or performance regressions.
- [x] Enforcing actor-style design across all Rust code.
- [x] Full thread-safety proof obligations beyond shape-based detection.