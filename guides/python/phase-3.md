# Phase 3: Verification, Mixed-Language Hardening, And Rollout

## Overview

This phase makes Python support release-ready. By the time it starts, deslop should already be able to scan Python repositories and emit the first Python rule pack. The remaining work is to prove that the new backend behaves well in mixed-language repositories, does not weaken the existing Go and Rust paths, and has documentation and verification coverage strong enough for future extension.

This phase is also where deferred decisions become explicit. If some Python ideas from `python.md` remain intentionally unshipped, that should be captured as backlog rather than left ambiguous.

## In Scope

- [x] Complete end-to-end verification for Python-only and mixed-language repositories
- [x] Add benchmark sanity checks for Python support
- [x] Synchronize documentation across the main guides and README when Python support becomes user-visible
- [x] Add regression coverage for language routing and language-scoped indexing
- [x] Capture deferred Python detections and future module-resolution work explicitly

## Out Of Scope

- Expanding the Python rule pack just because rollout work reveals more ideas
- Deep semantic Python import resolution unless it was already planned and validated in earlier phases
- Reworking the entire symbol index purely for speculative future Python features
- Chasing micro-optimizations without measured evidence of regression

## Dependencies

- Depends on Phases 1 and 2 because rollout must validate the actual backend and rule pack that ship.

## Primary Code Areas

- `tests/integration_scan.rs`
- `tests/integration_scan/python.rs`
- `tests/fixtures/python/`
- `src/index/mod.rs`
- `src/benchmark/mod.rs`
- `README.md`
- `guides/implementation-guide.md`
- `guides/features-and-detections.md`
- `guides/python/index.md`
- `guides/python/phase-1.md`
- `guides/python/phase-2.md`

## Implementation Checkpoints

1. Define the baseline verification matrix.

	Required coverage:

	- [x] Python-only repository scan
	- [x] mixed Python and Go repository scan
	- [x] mixed Python and Rust repository scan
	- [x] mixed Python, Go, and Rust repository scan if a compact fixture can represent it cleanly
	- [x] malformed Python fixture behavior through recoverable parse failures or syntax-error reporting
	- [x] positive and negative fixture coverage for all shipped Python rules
	- [x] regression checks proving Python file discovery does not disturb Go or Rust routing

2. Verify language-scoped repository behavior.

	Required outcomes:

	- [x] Python symbols do not bleed into Go or Rust package entries in the local index.
	- [x] Mixed-language same-directory layouts remain stable.
	- [x] Any future Python-local symbol resolution stays language-scoped from the start.

	This matters because the repository already carries a language-scoped index policy to prevent Go and Rust symbol bleed. Python must preserve that guarantee.

3. Define the benchmark sanity-check policy for Python.

	Required outcomes:

	- [x] Add at least one repeatable Python-heavy benchmark target or fixture convention.
	- [x] Record discovered files, analyzed files, function count, finding count, parse failures, and stage timings together.
	- [x] Treat the benchmark as a sanity check rather than a release blocker unless the repository set becomes stable enough for stronger gating.

	Recommended benchmark convention:

	- [x] Use the existing `cargo run -- bench --warmups 2 --repeats 5 <path>` shape.
	- [x] Keep one Python-only baseline and preserve at least one mixed-language verification workspace.

4. Synchronize the documentation set.

	Required updates once Python support is user-visible:

	- [x] `guides/implementation-guide.md` describes Python as a supported backend.
	- [x] `guides/features-and-detections.md` documents the shipped Python rule pack.
	- [x] `README.md` mentions Python support and important limitations.
	- [x] `guides/python/index.md` remains aligned with the real phase status.

5. Capture deferred Python work explicitly.

	Examples of backlog items that may remain deferred after the first release:

	- [x] module-aware hallucinated import and local-call checks for Python
	- [x] stronger `asyncio`-specific coordination and blocking analysis
	- [x] monolithic `__init__.py` detection
	- [x] god function and oversized class heuristics with defensible thresholds
	- [x] duplication detection
	- [x] lower-noise AI-smell detection if a precise evidence policy is later defined

	Do not leave these as implied ideas. Record them explicitly so later work starts from a known baseline.

6. Define the release-ready completion rule.

	Python support is ready for initial release only when:

	- [x] `.py` files are routed through the backend and tested
	- [x] the first Python rule pack is documented and covered by fixtures
	- [x] mixed-language scans remain stable
	- [x] Go and Rust tests do not regress
	- [x] the standard validation commands pass

## Acceptance Criteria

- [x] deslop can scan Python-only and mixed-language repositories without destabilizing existing language support.
- [x] The shipped Python rule pack is documented in user-facing docs.
- [x] The verification matrix covers routing, parse-failure behavior, and all shipped Python rules.
- [x] Benchmark conventions for Python support are documented.
- [x] Deferred Python ideas from `python.md` are captured as explicit backlog instead of vague future intent.

## Verification

- [x] Run `cargo test --test integration_scan`.
- [x] Run full `cargo test`.
- [x] Run `cargo build --release`.
- [x] Run at least one Python-only benchmark and one mixed-language verification scan before calling the phase complete.
- [x] Review `README.md`, `guides/implementation-guide.md`, and `guides/features-and-detections.md` for documentation sync.

## Current Rollout Snapshot

- [x] `cargo test --test integration_scan`: passed with `65` integration tests and `1` ignored real-repository benchmark test.
- [x] `cargo test`: passed.
- [x] `cargo build --release`: passed.
- [x] Python-only benchmark snapshot: `2` files, `3` functions, `11` findings, `0` parse failures. See `guides/python/benchmark-note.md`.
- [x] Mixed-language verification scan snapshot: `3` discovered files, `3` analyzed files, `6` functions, `1` finding, `0` parse failures, `packages=3`, `symbols=8`, `imports=2`.

## Document Update Obligations

- [x] Keep this file updated whenever the rollout matrix or backlog changes materially.
- [x] Keep `guides/python/index.md` phase links and status language accurate.
- [x] Update `guides/implementation-guide.md`, `guides/features-and-detections.md`, and `README.md` whenever shipped Python support changes observable behavior.

## Risks And Open Questions

- Python import behavior is flexible enough that future local-symbol heuristics may be much trickier than backend registration suggests.
- Mixed-language repositories are the fastest way to expose hidden assumptions in file routing and indexing. They must be part of the release contract.
- Python rule noise can rise quickly if the first release expands beyond conservative syntactic signals.
- Benchmark behavior may differ substantially between script-heavy and framework-heavy repositories, so one baseline will not describe every workload.