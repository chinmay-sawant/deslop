# Phase 3: Verification, Mixed-Language Hardening, And Rollout

## Overview

This phase makes Python support release-ready. By the time it starts, deslop should already be able to scan Python repositories and emit the first Python rule pack. The remaining work is to prove that the new backend behaves well in mixed-language repositories, does not weaken the existing Go and Rust paths, and has documentation and verification coverage strong enough for future extension.

This phase is also where deferred decisions become explicit. If some Python ideas from `python.md` remain intentionally unshipped, that should be captured as backlog rather than left ambiguous.

## In Scope

- End-to-end verification for Python-only and mixed-language repositories
- Benchmark sanity checks for Python support
- Documentation sync across the main guides and README when Python support becomes user-visible
- Regression coverage for language routing and language-scoped indexing
- Explicit backlog capture for deferred Python detections and future module-resolution work

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

	- Python-only repository scan
	- mixed Python and Go repository scan
	- mixed Python and Rust repository scan
	- mixed Python, Go, and Rust repository scan if a compact fixture can represent it cleanly
	- malformed Python fixture behavior through recoverable parse failures or syntax-error reporting
	- positive and negative fixture coverage for all shipped Python rules
	- regression checks proving Python file discovery does not disturb Go or Rust routing

2. Verify language-scoped repository behavior.

	Required outcomes:

	- Python symbols must not bleed into Go or Rust package entries in the local index.
	- Mixed-language same-directory layouts must remain stable.
	- Any future Python-local symbol resolution must stay language-scoped from the start.

	This matters because the repository already carries a language-scoped index policy to prevent Go and Rust symbol bleed. Python must preserve that guarantee.

3. Define the benchmark sanity-check policy for Python.

	Required outcomes:

	- Add at least one repeatable Python-heavy benchmark target or fixture convention.
	- Record discovered files, analyzed files, function count, finding count, parse failures, and stage timings together.
	- Treat the benchmark as a sanity check rather than a release blocker unless the repository set becomes stable enough for stronger gating.

	Recommended benchmark convention:

	- use the existing `cargo run -- bench --warmups 2 --repeats 5 <path>` shape
	- keep one Python-only baseline and continue preserving at least one mixed-language verification workspace

4. Synchronize the documentation set.

	Required updates once Python support is user-visible:

	- `guides/implementation-guide.md` must describe Python as a supported backend
	- `guides/features-and-detections.md` must document the shipped Python rule pack
	- `README.md` must mention Python support and any important limitations
	- `guides/python/index.md` must remain aligned with the real phase status

5. Capture deferred Python work explicitly.

	Examples of backlog items that may remain deferred after the first release:

	- module-aware hallucinated import and local-call checks for Python
	- stronger `asyncio`-specific coordination and blocking analysis
	- monolithic `__init__.py` detection
	- god function and oversized class heuristics with defensible thresholds
	- duplication detection
	- lower-noise AI-smell detection if a precise evidence policy is later defined

	Do not leave these as implied ideas. Record them explicitly so later work starts from a known baseline.

6. Define the release-ready completion rule.

	Python support is ready for initial release only when:

	- `.py` files are routed through the backend and tested
	- the first Python rule pack is documented and covered by fixtures
	- mixed-language scans remain stable
	- Go and Rust tests do not regress
	- the standard validation commands pass

## Acceptance Criteria

- deslop can scan Python-only and mixed-language repositories without destabilizing existing language support.
- The shipped Python rule pack is documented in user-facing docs.
- The verification matrix covers routing, parse-failure behavior, and all shipped Python rules.
- Benchmark conventions for Python support are documented.
- Deferred Python ideas from `python.md` are captured as explicit backlog instead of vague future intent.

## Verification

- Run `cargo test --test integration_scan`.
- Run full `cargo test`.
- Run `cargo build --release`.
- Run at least one Python-only benchmark and one mixed-language verification scan before calling the phase complete.
- Review `README.md`, `guides/implementation-guide.md`, and `guides/features-and-detections.md` for documentation sync.

## Document Update Obligations

- Update this file whenever the rollout matrix or backlog changes materially.
- Keep `guides/python/index.md` phase links and status language accurate.
- Update `guides/implementation-guide.md`, `guides/features-and-detections.md`, and `README.md` whenever shipped Python support changes observable behavior.

## Risks And Open Questions

- Python import behavior is flexible enough that future local-symbol heuristics may be much trickier than backend registration suggests.
- Mixed-language repositories are the fastest way to expose hidden assumptions in file routing and indexing. They must be part of the release contract.
- Python rule noise can rise quickly if the first release expands beyond conservative syntactic signals.
- Benchmark behavior may differ substantially between script-heavy and framework-heavy repositories, so one baseline will not describe every workload.