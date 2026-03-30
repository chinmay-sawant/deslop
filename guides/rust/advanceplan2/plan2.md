# Plan 2 - Interior Mutability And Shared State Exposure (Rust)

Date: 2026-03-30

## Objective

Add a Rust rule family for over-exposed interior mutability and global shared-state shapes that often show up as quick fixes in generated code.

## Existing Coverage Explicitly Excluded

This plan must not duplicate:

- `rust_lock_across_await`
- `rust_async_std_mutex_await`
- `rust_tokio_mutex_unnecessary`
- `rust_async_lock_order_cycle`
- `rust_domain_impossible_combination`

The current async packs already reason about locks in runtime flow. This phase focuses on the type and ownership shapes that make shared state hard to reason about before runtime.

## Candidate Rule Inventory

- [ ] `rust_pub_interior_mutability_field`
  - Detect public structs that expose `Mutex`, `RwLock`, `RefCell`, `Cell`, or similar interior-mutable fields directly.
- [ ] `rust_global_lock_state`
  - Detect `static`, `Lazy`, `OnceLock`, or singleton-style globals that wrap mutable lock-based state for ordinary application logic.
- [ ] `rust_arc_mutex_option_state`
  - Detect `Arc<Mutex<Option<T>>>` or closely related state bags that hide lifecycle state behind nested mutation layers.
- [ ] `rust_mutex_wrapped_collection`
  - Detect public or central domain structs that embed `Mutex<Vec<_>>`, `Mutex<HashMap<_, _>>`, or similar collection-plus-lock combinations instead of isolating mutation behind methods.
- [ ] `rust_rc_refcell_domain_model`
  - Detect non-test domain structs built from `Rc<RefCell<T>>` where the type shape suggests application logic rather than UI or graph tooling.

## Why These Rules Belong In Advance Plan 2

- [ ] They target ownership and design slop that is different from current async-runtime findings.
- [ ] They are common in AI-authored Rust because `Arc<Mutex<_>>` and `Rc<RefCell<_>>` are easy escape hatches when borrow-checker pressure appears.
- [ ] The evidence is largely type-text based and therefore realistic for a conservative static rule pack.

## Parser And Evidence Work

- [ ] Preserve type text for struct fields, statics, and local singleton definitions.
- [ ] Record visibility, derive metadata, and whether a field belongs to a public type.
- [ ] Capture `static` and lazy-initializer declarations with their contained type text.
- [ ] Reuse existing struct summaries where possible instead of inventing a second Rust type model.

## Implementation Checklist

- [ ] Add parser tests for nested generic type text such as `Arc<Mutex<Option<State>>>`.
- [ ] Implement helper classifiers for `Mutex`, `RwLock`, `RefCell`, `Cell`, `Arc`, `Rc`, `OnceLock`, and `Lazy` spellings.
- [ ] Start most rules at `Info`; escalate to `Warning` only for public API exposure or global mutable state.
- [ ] Add suppressions for clear graph-like data structures, GUI-oriented ownership trees, and test-only scaffolding.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `pub_interior_mutability_field_positive.rs.txt`
  - [ ] `global_lock_state_positive.rs.txt`
  - [ ] `arc_mutex_option_positive.rs.txt`
  - [ ] `mutex_collection_positive.rs.txt`
  - [ ] `rc_refcell_domain_positive.rs.txt`
- [ ] Negative fixtures:
  - [ ] private lock fields hidden behind clear methods
  - [ ] immutable shared handles
  - [ ] UI or tree structures where `Rc` and `Weak` are clearly intentional
  - [ ] test-only shared-state fixtures

## Acceptance Criteria

- [ ] Findings explain the exposed shared-state shape, not just the container names.
- [ ] The rule family clearly distinguishes design exposure from runtime misuse already handled elsewhere.
- [ ] Ordinary internal synchronization for small helper types does not become noisy.

## Non-Goals

- [ ] Proving contention or performance regressions.
- [ ] Enforcing actor-style design across all Rust code.
- [ ] Full thread-safety proof obligations beyond shape-based detection.