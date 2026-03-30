# Plan 4 - Boundary Safety And Serialization Slop (Python)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verification passed with `cargo test python_advanceplan2`.
- [x] Parser evidence regression passed with `cargo test test_python_advanceplan2_parser_evidence`.

## Objective

Add a conservative Python boundary-safety pack for dangerous serialization and subprocess helpers that frequently appear in example-driven or AI-authored code.

## Existing Coverage Explicitly Excluded

This plan does not duplicate:

- `eval_exec_usage`
- `external_input_without_validation`
- `missing_context_manager`
- `hardcoded_secret`

## Shipped Rules

- [x] `unsafe_yaml_loader`
- [x] `pickle_deserialization_boundary`
- [x] `subprocess_shell_true`
- [x] `tar_extractall_unfiltered`
- [x] `tempfile_without_cleanup`

## Implementation Notes

- [x] Kept the rule family local to call classification plus nearby line inspection.
- [x] Reused Python import alias resolution for `yaml`, `pickle`, `subprocess`, `tarfile`, and `tempfile` helpers.
- [x] Added conservative suppressions for safe loader markers, context-manager-owned temp resources, and explicit tar filters.

## Parser And Evidence Work

- [x] Reused existing Python call-site evidence and import metadata for serializer and subprocess classification.
- [x] Used source-line inspection for `shell=True`, loader markers, tar filters, and cleanup follow-up checks.
- [x] Kept temp-resource ownership reasoning local to context-manager and cleanup markers.

## Fixtures And Tests

- [x] Added grouped positive and clean fixtures under `tests/fixtures/python/integration/advanceplan2/`.
- [x] Added grouped integration coverage in `tests/integration_scan/python/advanceplan2.rs`.
- [x] Kept severity and evidence text factual and boundary-focused.

## Acceptance

- [x] Each rule is backed by grouped positive and clean fixtures.
- [x] Alias imports and keyword spelling do not break detection on representative cases.
- [x] The rule family remains conservative enough for default-on use.

## Non-Goals

- [x] Full taint tracking from external inputs to dangerous APIs.
- [x] Broad security auditing across third-party frameworks.
- [x] Replacing dedicated security tooling.