# Phase 2: Python Heuristics And First Rule Pack

## Overview

This phase turns Python parsing into actual user-visible findings. The goal is not to recreate Ruff, Flake8, Pylint, or mypy. The goal is to surface a small first set of high-signal bad patterns from the attached Python notes using evidence that is explainable, testable, and compatible with deslop's current output model.

This phase must use the abstractions already present in the repository:

- shared heuristics stay shared when the rule meaning is still honest for Python
- Python-only rules live together under `src/heuristics/python/`
- the Python backend calls into those rule entry points the same way Go and Rust already plug findings into the scan pipeline

## In Scope

- [x] Create `src/heuristics/python/mod.rs`
- [x] Create Python-specific heuristics modules under `src/heuristics/python/`
- [x] Enable any existing shared heuristics that the Phase 1 parser evidence can support cleanly
- [x] Define and implement the first Python-specific rule pack
- [x] Add fixture-driven coverage for every shipped Python rule
- [x] Update user-facing documentation when Python rules become observable

## Out Of Scope

- Reproducing Python linter rule sets wholesale
- Profiling-only claims that static analysis cannot justify
- Duplication detection
- Repository-wide architectural grading or AI-style scoring
- Deep data-flow, type-aware, or framework-specific reasoning
- Large refactors of existing Go or Rust heuristic code just to make the Python path look symmetrical

## Dependencies

- Depends on Phase 1 because Python heuristics must consume stable parser evidence.
- Blocks Phase 3 because rollout verification must validate the actual shipped Python rule set.

## Primary Code Areas

- `src/analysis/python/mod.rs`
- `src/analysis/python/parser/mod.rs`
- `src/heuristics/mod.rs`
- `src/heuristics/common.rs`
- `src/heuristics/python/mod.rs`
- `src/heuristics/python/performance.rs`
- `src/heuristics/python/maintainability.rs`
- `src/heuristics/python/structure.rs`
- `tests/integration_scan/python.rs`
- `tests/fixtures/python/`
- `guides/features-and-detections.md`
- `guides/implementation-guide.md`

## Implementation Checkpoints

1. Decide which shared heuristics Python should inherit immediately.

	The first explicit decision in this phase should be whether Python parser evidence is strong enough to enable these existing shared rule families:

	- [x] `generic_name`
	- [x] `overlong_name`
	- [x] `hardcoded_secret`
	- [x] comment-style findings if normalized docstring handling makes them meaningful
	- [x] test-quality findings if Python test summary extraction is reliable

	Rule:

	- [x] Reuse shared rules when the semantics are truly shared.
	- [x] Do not clone shared rules into `src/heuristics/python/` just to keep language code visually separate.

2. Create a Python heuristic entrypoint.

	Required outcomes:

	- [x] `src/heuristics/python/mod.rs` exports the Python-language evaluation surface.
	- [x] `src/analysis/python/mod.rs` remains thin and delegates rule execution cleanly.
	- [x] Python-specific rules do not depend on Go-specific helper assumptions.

3. Define the first Python-specific rule pack explicitly.

	Current shipped first-release rule pack:

	- [x] `string_concat_in_loop`: repeated `+=` or equivalent concatenation on clearly string-like values inside loops
	- [x] `blocking_sync_io_in_async`: obvious synchronous I/O or blocking calls inside `async def`
	- [x] `exception_swallowed`: `except:` or very broad exception handlers that immediately `pass`, `continue`, or otherwise suppress failure without meaningful handling
	- [x] `eval_exec_usage`: direct use of `eval()` or `exec()`
	- [x] `print_debugging_leftover`: `print()` calls that look like debug leftovers in non-test code
	- [x] `full_dataset_load`: obvious full-file or full-payload read patterns that load everything into memory when streaming would be more appropriate

	Current implementation note:

	- [x] `full_dataset_load` and `string_concat_in_loop` reuse shared rule IDs because their user-facing meaning remains the same across languages.
	- [x] `exception_swallowed`, `blocking_sync_io_in_async`, `eval_exec_usage`, and `print_debugging_leftover` remain Python-specific rule IDs.

	Naming rule:

	- [x] Reuse an existing rule ID only when the user-facing meaning is still the same across languages.
	- [x] Use a new Python-specific rule ID when the pattern is language-specific or the evidence policy differs materially.

4. Tie the attached `python.md` ideas to explicit implementation status.

	The phase deliverable should classify the attached ideas into three groups:

	- [x] implemented now
	- [x] intentionally deferred but compatible with the current parser contract
	- [x] deferred because they need profiling, duplication analysis, or deeper semantics

	This classification matters because the attached list is broad and the implementation must stay reviewable.

5. Define evidence quality requirements for each Python rule.

	Each rule should emit evidence that lets a reviewer understand why it fired.

	Examples:

	- [x] the loop line and target variable for `string_concat_in_loop`
	- [x] the async function name and blocking call target for `blocking_sync_io_in_async`
	- [x] the caught exception shape and handler body action for `exception_swallowed`
	- [x] the callee name for `eval_exec_usage`
	- [x] the print call site and argument summary for `print_debugging_leftover`
	- [x] the I/O call shape for `full_dataset_load`

	Message rule:

	- [x] Findings describe the risky pattern instead of claiming a proven runtime bug.

6. Keep Python heuristics foldered and cohesive.

	Recommended structure:

	- [x] `performance.rs`: concatenation in loops, full-dataset loads, sync I/O in async
	- [x] `maintainability.rs`: swallowed exceptions, `eval` and `exec`, debug leftovers
	- [x] `structure.rs`: later home for monolithic `__init__.py`, god functions, or other file-shape rules when their evidence policy becomes clear

	Do not create empty future-looking files unless they are immediately useful.

7. Add fixture coverage that proves the rule pack is conservative.

	Required fixture categories:

	- [x] positive fixture for every shipped rule
	- [x] negative fixture for every shipped rule
	- [x] mixed fixture where multiple Python findings appear together
	- [x] test-only fixture proving that noisy rules are suppressed or downgraded in test code where appropriate

	Examples that deserve explicit negatives:

	- [x] legitimate `print()` use in a CLI entrypoint if the rule is intended to ignore obvious user-facing output
	- [x] async functions that call real async clients instead of sync clients
	- [x] exception handlers that log, re-raise, or convert errors with real context

8. Decide what stays out of the first Python release.

	The attached notes mention many ideas that should not ship in the first rule pack without stronger evidence, including:

	- [x] list-versus-set performance claims when the value flow is unclear
	- [x] deep architectural smells like tight coupling or over-abstraction
	- [x] code-duplication detection
	- [x] "looks AI-generated" signals without a precise false-positive policy

	Those ideas should be recorded as backlog, not smuggled into the initial rule pack.

	Current deferred backlog after implementation:

	- [ ] module-aware hallucinated import and local-call checks for Python
	- [ ] monolithic `__init__.py` detection
	- [ ] god function and oversized class heuristics
	- [ ] duplication detection
	- [ ] lower-noise AI-smell detection with a defensible evidence policy

## Acceptance Criteria

- [x] The first Python rule pack is explicitly named and implemented.
- [x] Python-specific heuristics live under `src/heuristics/python/` and plug into the existing pipeline cleanly.
- [x] Shared heuristics are reused where appropriate instead of duplicated.
- [x] Every shipped Python rule has positive and negative fixture coverage.
- [x] Rule messages and evidence stay conservative and reviewable.
- [x] The attached Python idea list is explicitly triaged into shipped and deferred buckets.

## Verification

- [x] Review `src/analysis/python/mod.rs`, `src/heuristics/python/`, and `tests/integration_scan/python.rs` against the rule list defined here.
- [x] Run `cargo test --test integration_scan` after Python fixtures and heuristics land.
- [x] Run full `cargo test` after any shared heuristic or shared analysis-model changes.
- [x] Re-scan a mixed-language fixture set to ensure Python rules do not interfere with Go or Rust findings.

## Document Update Obligations

- [x] Keep this file updated whenever the Python rule pack changes.
- [x] Update `guides/features-and-detections.md` when Python rules become user-visible.
- [x] Update `guides/implementation-guide.md` when the architecture description must mention Python-specific heuristic routing.

## Risks And Open Questions

- Some Python bad patterns are very context-sensitive, especially `print()`, full-dataset loading, and broad exception handling. The first release must stay conservative.
- The attached notes mix static and dynamic concerns. Phase 2 must not turn profiling advice into false certainty.
- Enabling shared rules for Python is valuable, but only if the parser evidence supports them honestly.
- Python framework diversity means a rule that feels obvious in one stack may be normal in another. Keep the initial rule set generic and explainable.