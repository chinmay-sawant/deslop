# Plan 3 - Import-Time Side Effects And Global State (Python)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verification passed with `cargo test python_advanceplan2`.
- [x] Parser evidence regression passed with `cargo test test_python_advanceplan2_parser_evidence`.

## Objective

Add file-level Python heuristics for import-time side effects and mutable module state that often make generated code brittle, slow to import, and difficult to test.

## Existing Coverage Explicitly Excluded

This plan does not duplicate:

- `environment_boundary_without_fallback`
- `unrelated_heavy_import`
- `mixed_sync_async_module`
- `commented_out_code`
- `obvious_commentary`

## Shipped Rules

- [x] `import_time_network_call`
- [x] `import_time_file_io`
- [x] `import_time_subprocess`
- [x] `module_singleton_client_side_effect`
- [x] `mutable_module_global_state`
- [x] `import_time_config_load`

## Implementation Notes

- [x] Kept the scope file-local and parser-driven.
- [x] Reused new module-scope call and binding summaries instead of re-parsing whole files in the heuristic layer.
- [x] Required multiple mutating functions before flagging `mutable_module_global_state`.

## Parser And Evidence Work

- [x] Added `module_scope_calls` and `top_level_bindings` to shared parsed Python file evidence.
- [x] Kept module-scope classification separate from function and class bodies.
- [x] Reused import alias resolution for network, subprocess, file I/O, and config-load classification.

## Fixtures And Tests

- [x] Added parser coverage in `src/analysis/python/parser/tests.rs`.
- [x] Added grouped positive and clean fixtures under `tests/fixtures/python/integration/advanceplan2/`.
- [x] Added grouped integration coverage in `tests/integration_scan/python/advanceplan2.rs`.

## Acceptance

- [x] Findings distinguish import-time behavior from ordinary runtime setup.
- [x] Top-level constants and inert metadata stay quiet on representative clean fixtures.
- [x] The rule family adds signal beyond existing monolithic-module checks.

## Non-Goals

- [x] Full import graph or circular import analysis in the first pass.
- [x] Framework-specific application boot conventions.
- [x] Enforcing a no-globals policy across all Python code.