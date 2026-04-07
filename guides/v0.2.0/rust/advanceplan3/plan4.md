# Plan 4 - Unsafe, Global State, And Security Expansion (Rust)

Date: 2026-04-05

## Status

- [x] Implemented in the Rust scan pipeline, heuristics tree, and catalog registry.
- [x] Backlog derived from the current unsafe pack, the internal security checklist, and best-practice examples already present in the repository.
- [x] Overlap management with existing domain-modeling, async, and unsafe rules has been handled conservatively.

## Objective

Extend Rust support into adjacent security and footgun scenarios that are still missing from the shipped pack: unchecked slice boundaries, async-on-thread misuse, Rc cycle shapes, and unsafe string/byte boundary shortcuts.

## Source Inputs

- [x] `src/heuristics/rust/unsafe_soundness.rs`
- [x] `src/heuristics/rust/domain_modeling.rs`
- [x] `tests/integration_scan/rust_advanced.rs`
- [x] `guides/rust/advanceplan1/newplan_checklist.md`
- [x] `guides/rust/advanceplan1/newplan1.md`
- [x] `guides/architecture_enhancement/RUST_ARCHITECTURE_REVIEW_7.md`

## Existing Coverage Explicitly Excluded

- [x] `rust_global_lock_state`
- [x] `rust_rc_refcell_domain_model`
- [x] `rust_unsafe_get_unchecked`
- [x] `rust_unsafe_from_raw_parts`
- [x] `rust_unsafe_raw_pointer_cast`

## Proposed Rules

- [x] `rust_split_at_unchecked_external_input`
  - Flag `split_at`, direct slice indexing, or range slicing on externally-derived offsets without a visible bounds guard.
- [x] `rust_from_utf8_unchecked_boundary`
  - Flag `from_utf8_unchecked` and similar unchecked text conversion on boundary-facing inputs.
- [x] `rust_thread_spawn_async_without_runtime`
  - Flag raw `std::thread::spawn` blocks that call async work without an explicit runtime handoff.
- [x] `rust_rc_cycle_parent_link`
  - Flag obvious `Rc` parent/child ownership pairs that should likely use `Weak` for the back-reference.
- [x] `rust_static_mut_global`
  - Flag `static mut` and similarly blunt mutable-global patterns that bypass the safer global-state shapes already modeled elsewhere.
- [x] `rust_release_profile_missing_overflow_checks`
  - Flag Cargo manifests whose release profile omits `overflow-checks = true` for safety-sensitive codebases.
- [x] `rust_release_profile_panic_unwind`
  - Flag Cargo manifests that still ship `panic = "unwind"` when an abort-oriented CLI/service profile is the safer operational default.

## Parser And Evidence Work

- [x] Reused current unsafe-call evidence and struct-field summaries before adding a deeper aliasing model.
- [x] Added simple slice/index evidence only where current parser summaries were not enough for explainable messages.
- [x] Kept `Rc` cycle detection deliberately shape-based and biased toward obvious parent/back-reference patterns.

## Fixtures And Tests

- [x] Added unsafe/security fixtures under `tests/fixtures/rust/advanceplan3/`.
- [x] Added clean coverage for `Weak` back-references, guarded slicing, explicit runtime handoff, and checked UTF-8 conversion.
- [x] Added at least one negative fixture showing a legitimate `split_at` with an immediately preceding bounds assertion.

## Acceptance

- [x] Findings name the exact API or ownership shape that needs review.
- [x] Overlap with the existing unsafe pack is documented and kept minimal.
- [x] The rules remain conservative enough to help code review without turning into a pseudo-borrow-checker.
- [x] Cargo-profile findings stay explicitly profile-scoped and explainable from manifest text alone.

## Non-Goals

- [ ] Proving memory unsafety or UB.
- [ ] Full lifecycle analysis across multiple modules or crates.
- [ ] Replacing Miri, sanitizer runs, or dedicated security auditing tools.
