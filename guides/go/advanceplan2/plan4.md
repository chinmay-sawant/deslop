# Plan 4 - Package State And Abstraction Shape (Go)

Date: 2026-03-30

## Objective

Add a repo-aware Go rule pack for package-level mutable state and low-signal abstraction patterns that frequently show up in generated service code.

## Existing Coverage Explicitly Excluded

This plan must not duplicate:

- `generic_name`
- `overlong_name`
- `weak_typing`
- `inconsistent_package_name`
- `misgrouped_imports`
- `mixed_receiver_kinds`
- `hallucinated_import_call`
- `hallucinated_local_call`

The goal is API and abstraction quality, not naming, package style, or hallucination detection.

## Candidate Rule Inventory

- [ ] `mutable_package_global`
  - Detect exported or production-looking package globals that are reassigned or mutated outside obvious constant/config usage.
- [ ] `init_side_effect`
  - Detect `init()` functions that perform filesystem, network, subprocess, or database side effects instead of lightweight registration.
- [ ] `single_impl_interface`
  - Detect repository-local interfaces with one obvious implementation and one consumer, where the interface appears ceremonial rather than contractual.
- [ ] `passthrough_wrapper_interface`
  - Detect interfaces or service wrappers that forward one-to-one to an underlying dependency with little added policy or isolation value.
- [ ] `public_bool_parameter_api`
  - Detect exported functions or methods that expose raw boolean mode switches instead of named options or separate entry points.

## Why These Rules Belong In Advance Plan 2

- [ ] These signals are common in AI-assisted codebases that optimize for passing interfaces around rather than preserving clear package boundaries.
- [ ] The current Go pack is stronger on runtime and performance patterns than on package-architecture slop.
- [ ] Several candidate rules naturally require repository-local evidence, which fits an advanced second planning pass.

## Parser And Evidence Work

- [ ] Extend parser output with package-level declarations:
  - [ ] mutable globals and their assignment sites
  - [ ] `init()` functions and their call summaries
  - [ ] interface summaries and method sets
  - [ ] exported function parameter lists including raw boolean parameters
- [ ] Extend the repository index with lightweight implementation counts for local interfaces.
- [ ] Record wrapper-style one-to-one forwarding shapes for service methods so thin abstractions can be scored conservatively.

## Implementation Checklist

- [ ] Add parser tests covering interface declarations, init functions, and mutable globals.
- [ ] Add repo-level integration fixtures for single-implementation interface scenarios.
- [ ] Start `public_bool_parameter_api` at `Info`; escalate only if API surface or call count suggests real public misuse.
- [ ] Keep `single_impl_interface` and `passthrough_wrapper_interface` repo-local and conservative.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `mutable_package_global_positive.txt`
  - [ ] `init_side_effect_positive.txt`
  - [ ] `single_impl_interface_positive_a.txt`
  - [ ] `single_impl_interface_positive_b.txt`
  - [ ] `passthrough_wrapper_positive.txt`
  - [ ] `public_bool_parameter_api_positive.txt`
- [ ] Negative fixtures:
  - [ ] package constants and immutable registries
  - [ ] lightweight `init()` registration with no external I/O
  - [ ] interfaces with multiple concrete implementations
  - [ ] wrappers that add caching, validation, retry, or tracing policy

## Acceptance Criteria

- [ ] Repo-level rules explain why the abstraction looks ceremonial instead of valuable.
- [ ] File-level rules clearly distinguish mutable globals from constants and registries.
- [ ] The plan preserves Go's normal interface style and only flags low-context over-abstraction.

## Non-Goals

- [ ] Enforcing one architectural style for all Go packages.
- [ ] Full dependency-graph or cycle analysis in the first iteration.
- [ ] Penalizing legitimate interfaces used for testing, plug-in boundaries, or public SDK surfaces.