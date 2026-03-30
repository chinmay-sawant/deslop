# Plan 4 - Package State And Abstraction Shape (Go)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verified by the Go advanceplan2 integration suite.

## Objective

Add a repo-aware Go rule pack for package-level mutable state and low-signal abstraction patterns that frequently show up in generated service code.

## Shipped Rules

- [x] `mutable_package_global`
- [x] `init_side_effect`
- [x] `single_impl_interface`
- [x] `passthrough_wrapper_interface`
- [x] `public_bool_parameter_api`

## Implementation Notes

- [x] Kept the scope separate from naming, style, hallucination, and existing receiver-consistency rules.
- [x] Extended Go parser output with package vars, interface summaries, struct field summaries, and function signature text.
- [x] Added repo-level interface and wrapper scoring without forcing new cross-language abstractions.
- [x] Kept `public_bool_parameter_api` conservative and informational unless multiple flags appear.
- [x] Kept `single_impl_interface` and `passthrough_wrapper_interface` repo-local.

## Fixtures And Tests

- [x] Added positive fixtures:
  - [x] `mutable_package_global_positive.txt`
  - [x] `init_side_effect_positive.txt`
  - [x] `single_impl_interface_positive_a.txt`
  - [x] `single_impl_interface_positive_b.txt`
  - [x] `passthrough_wrapper_positive.txt`
  - [x] `public_bool_parameter_api_positive.txt`
- [x] Added clean coverage in `architecture_clean_a.txt`.
- [x] Added parser regression coverage for package vars, interfaces, structs, and function signatures.
- [x] Added grouped integration coverage in `tests/integration_scan/go_advanceplan2.rs`.

## Acceptance

- [x] Repo-level rules explain the abstraction signal instead of only naming the symbol.
- [x] File-level rules distinguish mutable globals from read-only package variables.
- [x] The implementation stays conservative about legitimate interfaces and wrappers.

## Non-Goals

- [x] Enforcing one architectural style for all Go packages.
- [x] Full dependency-graph or cycle analysis in the first iteration.
- [x] Penalizing legitimate interfaces used for testing, plug-in boundaries, or public SDK surfaces.