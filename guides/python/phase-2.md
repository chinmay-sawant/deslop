# Phase 2: Python Heuristics And First Rule Pack

## Overview

This phase turns Python parsing into actual user-visible findings. The goal is not to recreate Ruff, Flake8, Pylint, or mypy. The goal is to surface a small first set of high-signal bad patterns from the attached Python notes using evidence that is explainable, testable, and compatible with deslop's current output model.

This phase must use the abstractions already present in the repository:

- shared heuristics stay shared when the rule meaning is still honest for Python
- Python-only rules live together under `src/heuristics/python/`
- the Python backend calls into those rule entry points the same way Go and Rust already plug findings into the scan pipeline

## In Scope

- Creating `src/heuristics/python/mod.rs`
- Creating Python-specific heuristics modules under `src/heuristics/python/`
- Enabling any existing shared heuristics that the Phase 1 parser evidence can support cleanly
- Defining and implementing the first Python-specific rule pack
- Fixture-driven coverage for every shipped Python rule
- Updating user-facing documentation when Python rules become observable

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

	- `generic_name`
	- `overlong_name`
	- `hardcoded_secret`
	- comment-style findings if normalized docstring handling makes them meaningful
	- test-quality findings if Python test summary extraction is reliable

	Rule:

	- Reuse shared rules when the semantics are truly shared.
	- Do not clone shared rules into `src/heuristics/python/` just to keep language code visually separate.

2. Create a Python heuristic entrypoint.

	Required outcomes:

	- `src/heuristics/python/mod.rs` exports the Python-language evaluation surface.
	- `src/analysis/python/mod.rs` remains thin and delegates rule execution cleanly.
	- Python-specific rules do not depend on Go-specific helper assumptions.

3. Define the first Python-specific rule pack explicitly.

	Current shipped first-release rule pack:

	- `string_concat_in_loop`: repeated `+=` or equivalent concatenation on clearly string-like values inside loops
	- `blocking_sync_io_in_async`: obvious synchronous I/O or blocking calls inside `async def`
	- `exception_swallowed`: `except:` or very broad exception handlers that immediately `pass`, `continue`, or otherwise suppress failure without meaningful handling
	- `eval_exec_usage`: direct use of `eval()` or `exec()`
	- `print_debugging_leftover`: `print()` calls that look like debug leftovers in non-test code
	- `full_dataset_load`: obvious full-file or full-payload read patterns that load everything into memory when streaming would be more appropriate

	Current implementation note:

	- `full_dataset_load` and `string_concat_in_loop` reuse shared rule IDs because their user-facing meaning remains the same across languages.
	- `exception_swallowed`, `blocking_sync_io_in_async`, `eval_exec_usage`, and `print_debugging_leftover` remain Python-specific rule IDs.

	Naming rule:

	- Reuse an existing rule ID only when the user-facing meaning is still the same across languages.
	- Use a new Python-specific rule ID when the pattern is language-specific or the evidence policy differs materially.

4. Tie the attached `python.md` ideas to explicit implementation status.

	The phase deliverable should classify the attached ideas into three groups:

	- implemented now
	- intentionally deferred but compatible with the current parser contract
	- deferred because they need profiling, duplication analysis, or deeper semantics

	This classification matters because the attached list is broad and the implementation must stay reviewable.

5. Define evidence quality requirements for each Python rule.

	Each rule should emit evidence that lets a reviewer understand why it fired.

	Examples:

	- the loop line and target variable for `string_concat_in_loop`
	- the async function name and blocking call target for `blocking_sync_io_in_async`
	- the caught exception shape and handler body action for `exception_swallowed`
	- the callee name for `eval_exec_usage`
	- the print call site and argument summary for `print_debugging_leftover`
	- the I/O call shape for `full_dataset_load`

	Message rule:

	- findings should describe the risky pattern, not claim a proven runtime bug.

6. Keep Python heuristics foldered and cohesive.

	Recommended structure:

	- `performance.rs`: concatenation in loops, full-dataset loads, sync I/O in async
	- `maintainability.rs`: swallowed exceptions, `eval` and `exec`, debug leftovers
	- `structure.rs`: later home for monolithic `__init__.py`, god functions, or other file-shape rules when their evidence policy becomes clear

	Do not create empty future-looking files unless they are immediately useful.

7. Add fixture coverage that proves the rule pack is conservative.

	Required fixture categories:

	- positive fixture for every shipped rule
	- negative fixture for every shipped rule
	- mixed fixture where multiple Python findings appear together
	- test-only fixture proving that noisy rules are suppressed or downgraded in test code where appropriate

	Examples that deserve explicit negatives:

	- legitimate `print()` use in a CLI entrypoint if the rule is intended to ignore obvious user-facing output
	- async functions that call real async clients instead of sync clients
	- exception handlers that log, re-raise, or convert errors with real context

8. Decide what stays out of the first Python release.

	The attached notes mention many ideas that should not ship in the first rule pack without stronger evidence, including:

	- list-versus-set performance claims when the value flow is unclear
	- deep architectural smells like tight coupling or over-abstraction
	- code-duplication detection
	- "looks AI-generated" signals without a precise false-positive policy

	Those ideas should be recorded as backlog, not smuggled into the initial rule pack.

	Current deferred backlog after implementation:

	- module-aware hallucinated import and local-call checks for Python
	- monolithic `__init__.py` detection
	- god function and oversized class heuristics
	- duplication detection
	- lower-noise AI-smell detection with a defensible evidence policy

## Acceptance Criteria

- The first Python rule pack is explicitly named and implemented.
- Python-specific heuristics live under `src/heuristics/python/` and plug into the existing pipeline cleanly.
- Shared heuristics are reused where appropriate instead of duplicated.
- Every shipped Python rule has positive and negative fixture coverage.
- Rule messages and evidence stay conservative and reviewable.
- The attached Python idea list is explicitly triaged into shipped and deferred buckets.

## Verification

- Review `src/analysis/python/mod.rs`, `src/heuristics/python/`, and `tests/integration_scan/python.rs` against the rule list defined here.
- Run `cargo test --test integration_scan` after Python fixtures and heuristics land.
- Run full `cargo test` after any shared heuristic or shared analysis-model changes.
- Re-scan a mixed-language fixture set to ensure Python rules do not interfere with Go or Rust findings.

## Document Update Obligations

- Update this file whenever the Python rule pack changes.
- Update `guides/features-and-detections.md` when Python rules become user-visible.
- Update `guides/implementation-guide.md` when the architecture description must mention Python-specific heuristic routing.

## Risks And Open Questions

- Some Python bad patterns are very context-sensitive, especially `print()`, full-dataset loading, and broad exception handling. The first release must stay conservative.
- The attached notes mix static and dynamic concerns. Phase 2 must not turn profiling advice into false certainty.
- Enabling shared rules for Python is valuable, but only if the parser evidence supports them honestly.
- Python framework diversity means a rule that feels obvious in one stack may be normal in another. Keep the initial rule set generic and explainable.