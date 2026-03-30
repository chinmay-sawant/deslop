# Plan 3 - Import-Time Side Effects And Global State (Python)

Date: 2026-03-30

## Objective

Add file-level Python heuristics for import-time side effects and mutable module state that often make generated code brittle, slow to import, and difficult to test.

## Existing Coverage Explicitly Excluded

This plan must not duplicate:

- `environment_boundary_without_fallback`
- `unrelated_heavy_import`
- `mixed_sync_async_module`
- `commented_out_code`
- `obvious_commentary`

The focus here is executable side effects at module import time, not import selection quality or commentary style.

## Candidate Rule Inventory

- [ ] `import_time_network_call`
  - Detect module-level HTTP or socket calls executed while the module is imported.
- [ ] `import_time_file_io`
  - Detect file reads, writes, or directory scans performed at module scope outside clearly named constants or fixtures.
- [ ] `import_time_subprocess`
  - Detect subprocess launches triggered during import.
- [ ] `module_singleton_client_side_effect`
  - Detect eagerly constructed network, database, or cloud clients at module scope when they encapsulate live configuration or external state.
- [ ] `mutable_module_global_state`
  - Detect mutable module globals that are updated from multiple functions without an obvious ownership wrapper.
- [ ] `import_time_config_load`
  - Detect environment, file, or secret loading that happens at import time instead of at an explicit startup boundary.

## Why These Rules Belong In Advance Plan 2

- [ ] Import-time work is a classic AI-slop smell because examples often inline setup code directly at the top level.
- [ ] The current Python pack reasons about file shape and module size but not module initialization behavior.
- [ ] All candidate rules can stay file-local and parser-driven.

## Parser And Evidence Work

- [ ] Extend the Python parser to distinguish module-scope statements from function or class bodies.
- [ ] Capture module-scope assignments, calls, and mutation sites.
- [ ] Reuse existing call classification for network, subprocess, and file I/O where possible.
- [ ] Add a small model of module-level mutable containers and reassignment sites.

## Implementation Checklist

- [ ] Add parser tests for module-scope call extraction.
- [ ] Implement file-level heuristics in a dedicated Python module or structure-oriented pack.
- [ ] Add suppressions for:
  - [ ] test fixtures
  - [ ] constant registration tables
  - [ ] harmless metadata declarations
  - [ ] optional plugin registration that performs no I/O
- [ ] Keep `mutable_module_global_state` repo-local only if cross-function mutation evidence materially improves precision.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `import_time_network_positive.txt`
  - [ ] `import_time_file_io_positive.txt`
  - [ ] `import_time_subprocess_positive.txt`
  - [ ] `singleton_client_positive.txt`
  - [ ] `mutable_module_global_positive.txt`
  - [ ] `import_time_config_positive.txt`
- [ ] Negative fixtures:
  - [ ] startup function that performs the same work explicitly
  - [ ] constant and enum declarations only
  - [ ] lazy singleton wrapper or factory function
  - [ ] immutable module-level configuration objects

## Acceptance Criteria

- [ ] Findings clearly distinguish import-time behavior from normal runtime setup.
- [ ] Top-level constant declarations and metadata are not over-flagged.
- [ ] The rule family adds new signal beyond existing monolithic-module findings.

## Non-Goals

- [ ] Full import graph or circular import analysis in the first pass.
- [ ] Framework-specific application boot conventions.
- [ ] Enforcing a no-globals policy across all Python code.