# Phase 4: Verification, Performance, And Rollout

## Overview

This phase makes Rust support shippable. By the time this phase starts, deslop should already be able to scan Rust files and emit the first Rust rule pack. The remaining work is to prove the backend is stable in mixed-language repositories, verify the output contract, update the docs, and keep the new support from regressing the existing Go path.

## In Scope

- End-to-end verification for Rust-only and mixed-language repositories
- Benchmark sanity checks for Rust support
- Documentation sync for architecture and supported detections
- Release checklist updates for Rust support
- Backlog capture for deferred Rust enhancements that are intentionally not part of the first implementation pass

## Out Of Scope

- Expanding the Rust rule pack just because verification reveals possible future ideas
- Benchmark-driven micro-optimizations without evidence of real regression
- Cargo workspace graph analysis, dependency audit rules, or semantic linting that were intentionally deferred from earlier phases

## Dependencies

- Depends on Phases 1 through 3 because rollout work must validate the actual shipped backend and rule set.

## Primary Code Areas

- `tests/integration_scan.rs`
- `tests/integration_scan/rust.rs`
- `tests/fixtures/rust/`
- `src/benchmark/mod.rs`
- `guides/implementation-guide.md`
- `guides/features-and-detections.md`
- `README.md`
- `guides/rust/index.md`
- `guides/rust/release-checklist.md`

## Implementation Checkpoints

1. Define the baseline verification matrix.

	Required coverage:

	- Rust-only repository scan
	- mixed Rust and Go repository scan
	- recoverable parse-failure behavior for malformed Rust files
	- rule-positive and rule-negative Rust fixtures
	- Rust-local imported-call hallucination checks for `crate::`, `self::`, and `super::`
	- Rust direct-call hallucination checks for imported function aliases and same-module calls
	- mixed-language same-directory package separation in the local index
	- no regression in existing Go integration tests

2. Define the benchmark sanity-check policy.

	Required outcomes:

	- Add at least one benchmark target or repeatable local convention for a real Rust repository.
	- Record whether Rust support materially changes parse-stage or total runtime.
	- Treat this as a sanity check, not a hard release gate, unless the team later establishes stable benchmark environments.

	Current rollout convention:

	- keep the existing Go baseline and add one Rust-only baseline using the same warmup and repeat counts
	- keep the benchmark note attached to file, function, finding, package, symbol, and import counts so timing shifts can be interpreted
	- do not treat benchmark differences as release blockers until the repository set and machine-noise expectations are stable

3. Update the public documentation set.

	Required outcomes:

	- `guides/implementation-guide.md` must state that deslop supports Go and Rust backends.
	- `guides/features-and-detections.md` must document the first Rust rule pack.
	- `README.md` must reflect Rust support once the feature is user-visible.
	- `guides/rust/release-checklist.md` must stay aligned with the shipped verification matrix.

4. Capture deferred backlog explicitly.

	Examples of deferred Rust-specific work that should be recorded rather than silently forgotten:

	- Cargo workspace and crate graph awareness
	- trait and impl resolution for stronger local-context checks
	- async-runtime specific heuristics
	- allocation and clone-pattern heuristics
	- wildcard-import or visibility-discipline rules if the team later wants them

5. Define the release-ready completion rule.

	Rust support is ready for initial release only when:

	- the Rust backend is wired and tested
	- the first rule pack is documented and covered
	- mixed-language scans behave predictably
	- Rust-local imported-call checks and mixed-language index separation are covered by regression tests
	- the standard validation commands pass

## Acceptance Criteria

- deslop can scan Rust-only repositories and mixed Rust-and-Go repositories.
- The first Rust rule pack is documented in user-facing docs.
- Rust fixture coverage exists for all shipped Rust rules.
- The release checklist explicitly includes Rust support validation.
- Deferred Rust features are recorded as backlog rather than left implicit.

## Verification

- Run `cargo test --test integration_scan`, full `cargo test`, and `cargo build --release`.
- Re-check Rust-only and mixed-language fixture repositories after any backend or rule-pack changes.
- Review `guides/implementation-guide.md`, `guides/features-and-detections.md`, and `README.md` for documentation sync before calling the phase complete.

Current matrix notes:

- Rust-only scan coverage lives in `tests/integration_scan/rust.rs` with positive, negative, and malformed fixtures.
- Mixed-language scan coverage includes both shared repository discovery and same-directory Go/Rust index separation regressions.
- Rust-local imported-call coverage currently validates `crate::`, `self::`, and `super::` imports against locally indexed Rust modules.
- Rust direct-call coverage currently validates imported function aliases, same-module calls, local closures, and constructor-like symbols.

## Document Update Obligations

- Update this file whenever the verification matrix or rollout criteria change.
- Keep the phase cross-links in `guides/rust/index.md` accurate.
- Update `guides/implementation-guide.md`, `guides/features-and-detections.md`, and `README.md` whenever shipped Rust support changes observable behavior.

## Risks And Open Questions

- Mixed-language repositories can expose assumptions that are invisible in language-isolated tests.
- Rust support may tempt the codebase toward per-language duplication; rollout should keep that tradeoff visible rather than hiding it.
- Benchmark differences between Go-heavy and Rust-heavy repositories may be large enough that one baseline is not representative.
- Rust module resolution is still intentionally local and path-based; crate-graph-aware rollout work remains deferred.