# Phase 3: Rust Heuristics And Findings

## Overview

This phase adds the first Rust-specific bad-practice detectors. The rule pack should stay conservative, syntactic, and explainable. The target is not to replicate Clippy or to prove semantic bugs. The target is to flag reviewable Rust slop patterns with clear evidence, low surprise, and a fixture set that keeps false positives under control.

The first implementation should favor a small set of high-signal rules rather than a broad but noisy detector list.

## In Scope

- The first Rust rule pack
- Rust-specific heuristic evaluation entry points
- Fixture-driven tests for Rust findings
- User-facing rule IDs, messages, and evidence payloads
- Documentation updates for newly supported Rust detections

## Out Of Scope

- Full Clippy-style lint coverage
- Type-driven or borrow-checker-aware diagnostics
- Cargo dependency or security-audit rules
- Rules that require macro expansion, trait resolution, or project-wide semantic inference before they can be reliable

## Dependencies

- Depends on Phase 2 because heuristics must consume stable parser evidence.
- Blocks Phase 4 because verification, docs, and rollout should reflect the actual shipped Rust rule set.

## Primary Code Areas

- `src/analysis/rust/mod.rs`
- `src/analysis/rust/parser.rs`
- `src/heuristics/mod.rs`
- `src/heuristics/common.rs`
- `tests/integration_scan/rust.rs`
- `tests/fixtures/rust/`
- `guides/features-and-detections.md`
- `guides/implementation-guide.md`

## Implementation Checkpoints

1. Define the first Rust rule pack explicitly.

	Recommended initial rules for the first shippable pass:

	- `todo_macro_leftover`: flags `todo!()` in non-test Rust code
	- `unimplemented_macro_leftover`: flags `unimplemented!()` in non-test Rust code
	- `dbg_macro_leftover`: flags `dbg!()` in non-test Rust code
	- `panic_macro_leftover`: flags `panic!()` in non-test Rust code when it appears as an obvious leftover or control-flow stub
	- `unreachable_macro_leftover`: flags `unreachable!()` in non-test Rust code when it appears as an obvious leftover or control-flow stub
	- `unwrap_in_non_test_code`: flags `.unwrap()` in non-test Rust code
	- `expect_in_non_test_code`: flags `.expect(...)` in non-test Rust code
	- `unsafe_without_safety_comment`: flags `unsafe` blocks or `unsafe fn` where no nearby `SAFETY:` comment policy is satisfied

	These rules are intentionally small, Rust-specific, and implementable from parser evidence without semantic overreach.

2. Define Rust test-aware suppression policy.

	Required outcomes:

	- The rule pack must document which findings are suppressed in test-only code.
	- `dbg!`, `todo!`, `unwrap`, and `expect` rules should normally avoid noisy test-only findings unless the team explicitly wants them.
	- If test classification is uncertain, the heuristic must choose the less noisy path.

3. Implement Rust heuristic entry points.

	Required outcomes:

	- Rust backend evaluation should live with the Rust backend and plug into the shared heuristic pipeline cleanly.
	- Do not force Rust findings through Go-specific helper assumptions.
	- Rule IDs, severity, and evidence strings must remain consistent with the existing report model.

4. Build a compact but complete Rust fixture set.

	Required fixture categories:

	- positive fixture for each rule
	- negative fixture for each rule
	- mixed fixture where multiple Rust signals appear together
	- at least one example under `tests/fixtures/rust/` that demonstrates test-only suppression behavior

5. Define evidence quality requirements.

	Each Rust finding should include evidence that helps a reviewer understand why the rule fired.

	Examples:

	- macro call name and line number
	- method call evidence such as `.unwrap()` or `.expect()`
	- presence or absence of a nearby `SAFETY:` comment when unsafe-related rules fire

	Current nearby-comment policy for the first implementation pass:

	- accept `SAFETY:` on the same line as the unsafe usage or within the previous two lines
	- keep the policy simple and explainable rather than trying to infer broader comment scope

## Acceptance Criteria

- The first Rust rule pack is explicitly named and implemented.
- Every Rust rule has both positive and negative coverage.
- Test-only code suppression behavior is documented and tested.
- Rust findings produce clear rule IDs, messages, and evidence.
- The Rust rule pack remains small enough to review quickly and extend safely later.

## Verification

- Review `src/analysis/rust/mod.rs`, any Rust-specific heuristic code, and `tests/integration_scan/rust.rs` against the rule list in this phase.
- Add fixture-driven tests for every Rust rule.
- Run `cargo test --test integration_scan` and full `cargo test` after Rust heuristics land.

## Extended Rust Rule Packs

The Rust analyzer now also emits conservative findings for four additional rule families backed by parser evidence and fixture-driven tests.

- Performance and async runtime checks:
	- `rust_blocking_io_in_async`
	- `rust_unbuffered_file_writes`
	- `rust_lines_allocate_per_line`
	- `rust_hashmap_default_hasher`
	- `rust_lock_across_await`
	- `rust_tokio_mutex_unnecessary`
	- `rust_blocking_drop`
	- `rust_pointer_chasing_vec_box`
	- `rust_aos_hot_path`
	- `rust_large_future_stack`
	- `rust_utf8_validate_hot_path`
	- `rust_path_join_absolute`

- Domain modeling and invariants:
	- `rust_domain_raw_primitive`
	- `rust_domain_float_for_money`
	- `rust_domain_impossible_combination`
	- `rust_domain_default_produces_invalid`
	- `rust_debug_secret`
	- `rust_serde_sensitive_deserialize`
	- `rust_serde_sensitive_serialize`

- Async and concurrency pitfalls:
	- `rust_async_std_mutex_await`
	- `rust_async_hold_permit_across_await`
	- `rust_async_spawn_cancel_at_await`
	- `rust_async_missing_fuse_pin`
	- `rust_async_recreate_future_in_select`
	- `rust_async_monopolize_executor`
	- `rust_async_blocking_drop`
	- `rust_async_invariant_broken_at_await`
	- `rust_async_lock_order_cycle`

- Unsafe soundness checks:
	- `rust_unsafe_get_unchecked`
	- `rust_unsafe_from_raw_parts`
	- `rust_unsafe_set_len`
	- `rust_unsafe_assume_init`
	- `rust_unsafe_transmute`
	- `rust_unsafe_raw_pointer_cast`
	- `rust_unsafe_aliasing_assumption`

These rule packs are intentionally syntactic and conservative. They are designed to produce reviewable findings with clear evidence rather than Clippy-style semantic proofs.

## Document Update Obligations

- Update this file whenever the Rust rule pack changes.
- Update `guides/features-and-detections.md` when Rust rules become user-visible.
- Update `guides/implementation-guide.md` when the architecture description needs to mention Rust heuristics explicitly.

## Risks And Open Questions

- `unwrap` and `expect` are common and not always bad; test suppression and message wording must keep the detector from sounding absolute.
- `unsafe_without_safety_comment` depends on a nearby-comment policy that must be simple enough to explain and test.
- Rust users may expect Clippy-level sophistication quickly. The roadmap should keep the initial rule pack intentionally smaller and more explainable than that.