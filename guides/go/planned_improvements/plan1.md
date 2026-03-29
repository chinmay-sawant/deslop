# Plan 1 — Stronger repo-wide style checks (Go)

This document is a detailed, checklist-driven plan to add repo-wide style checks for Go code. The work will be Rust-only (modify or add Rust files under `src/heuristics`) and will not create any Go source files; tests use plain-text Go fixtures under `tests/fixtures/go`.

---

## Goals

- Surface repository-level style problems that are easy to fix centrally.
- Provide actionable findings with minimal false positives.
- Keep performance impact negligible compared to parsing and indexing.

---

## Scope (Phase 1)

- Add checks that operate across files in a directory and per-file import-order checks.
- Initial rules:
  - [x] `inconsistent_package_name`: detect directories where multiple package names are used (ignoring `_test` suffix differences).
  - [x] `misgrouped_imports`: detect files that mix stdlib and third-party imports out-of-order (stdlib imports should not appear after third-party imports in the same import list).

Excluded for Phase 1: license/header checks, enforcing `gofmt` output (Phase 2).

---

## High-level design

- Use the existing parser output (`ParsedFile`) and repository index.
  - `ParsedFile.package_name` — package name per file
  - `ParsedFile.path` — file path (anchor findings)
  - `ParsedFile.imports` — import records in source order
- Implement checks in Rust under `src/heuristics` and wire them into the existing `evaluate_go_file` and `evaluate_go_repo` entry points.

Rule decision details:
- Package comparison: strip a trailing `_test` when normalizing package names for directory-level aggregation.
- Stdlib detection: treat any import path that does **not** contain a `.` as stdlib (practical and conventional heuristic).

Severity defaults:
- `inconsistent_package_name` → `Warning` (repo-level, likely accidental)
- `misgrouped_imports` → `Info` (style/readability)

Rule ids must be snake_case and match repository config usage: `inconsistent_package_name`, `misgrouped_imports`.

---

## Implementation checklist (developer tasks)

Repository paths mentioned below are relative to the project root.

- [x] Confirm no existing implementation already covers these rules (search):
  - [x] Inspect `src/heuristics/mod.rs` to see how repo/file heuristics are registered.
  - [x] Inspect `src/heuristics/consistency.rs`, `src/heuristics/naming.rs`, and other heuristics for any overlapping logic.
  - [x] Inspect parser `ParsedFile` usage: `src/analysis/types.rs`, `src/analysis/go/parser/mod.rs` to confirm `package_name` and `imports` are available.

- [x] Add new Rust source file: `src/heuristics/style.rs` implementing:
  - [x] `pub(super) fn package_name_consistency(files: &[&ParsedFile]) -> Vec<Finding>`
    - Behavior:
      - Group `files` by parent directory.
      - For each directory, normalize package names by stripping a trailing `_test` suffix.
      - If >1 distinct base package name found, create one `Finding` per directory anchored to a representative file (prefer the earliest file path lexicographically).
      - Evidence: list package names and example file paths.
      - Rule id: `inconsistent_package_name`.
  - [x] `pub(super) fn import_grouping_findings(file: &ParsedFile) -> Vec<Finding>`
    - Behavior:
      - Walk imports in declared order, map each import to `is_stdlib: bool` by checking for `.` in the path.
      - If a stdlib import appears after a third-party import, emit a `Finding` with rule id `misgrouped_imports` and evidence describing the offending import order and a short suggestion (e.g., "group stdlib imports before third-party imports").

- [x] Wire into heuristics pipeline:
  - [x] Add `mod style;` to `src/heuristics/mod.rs`.
  - [x] In `evaluate_go_file`, call `import_grouping_findings(file)` and extend the file-level findings.
  - [x] In `evaluate_go_repo`, call `package_name_consistency(files)` in addition to existing repo-level checks (e.g., `receiver_findings`).

- [x] Documentation and rule metadata
  - [x] Add brief descriptions for each rule to the guides or a rules registry if one exists (brief note in this plan suffices for Phase 1).
  - [x] Ensure rule ids are usable by `RepoConfig.disabled_rules` and `severity_overrides`.

---

## Tests and fixtures

Testing style follows the repository conventions: plain-text Go fixtures live under `tests/fixtures/go` and Rust integration tests in `tests/integration_scan` drive the scanner using those fixtures.

Fixtures to add (plain text files):
- [x] `tests/fixtures/go/package_conflict_a.txt` — package name `sample`
- [x] `tests/fixtures/go/package_conflict_b.txt` — package name `sample_test` (or a different base name) placed in same directory during test to trigger `inconsistent_package_name`
- [x] `tests/fixtures/go/import_misgrouped.txt` — an import block that contains a third-party import followed by a stdlib import (to trigger `misgrouped_imports`)
- [x] `tests/fixtures/go/import_grouped.txt` — correctly grouped imports (control)

Integration tests (Rust) — checklist:
- [x] Add test module file `tests/integration_scan/style.rs` with tests that:
  - [x] Write fixtures into a temp workspace using the existing `write_fixture` helper.
  - [x] Call `scan_repository(&ScanOptions { root: temp_dir.clone(), respect_ignore: true })`.
  - [x] Assert that the expected rule_ids appear (or do not appear) in `report.findings`.

Test file content style note (per repo convention):
- The Go code for test cases must be in plain text fixture files (as above). The Rust integration test file itself will contain only harness/assert logic, not Go source code.

---

## Performance/Benchmark notes

- The checks are linear in the number of files and imports; they should be cheap compared to parsing and indexing.
- Validate via existing benchmark harness: `cargo run -- bench --warmups 2 --repeats 5 <path>` and ensure added checks do not regress wall-clock significantly.

---

## Acceptance criteria

- [x] New rules produce expected findings on canonical fixtures.
- [x] No unrelated false positives on representative real repositories (sample quick-run on `gopdfsuit` and `SnapBack` produced no `inconsistent_package_name` or `misgrouped_imports` findings).
- [x] Rules can be toggled via repository config (`disabled_rules`) and severity overrides work (covered by scan config tests plus Go style integration coverage).
- [x] Performance impact is minor (<= ~5-10% extra heuristics time for large repos).

---

## Rollout plan

- [x] Implement with conservative severities and default enabled.
- [x] Add docs to `guides/go/planned_improvements/plan1.md` (this file).
- [x] Run CI-equivalent local validation (`cargo test`) and preserve repository-level escape hatches through `disabled_rules` and `severity_overrides`.

Later style backlog notes:

- `gofmt` detection and license/header checks were intentionally left out of this phase. Subsequent Go plans in this repository focused on context propagation and opt-in semantic heuristics instead.

---

## Tasks checklist (concrete steps)

- [x] Search repository for `package_name` and `imports` usage and any existing style checks.
- [x] Implement `src/heuristics/style.rs` with `package_name_consistency` and `import_grouping_findings`.
- [x] Update `src/heuristics/mod.rs` to register and call new checks.
- [x] Add test fixtures under `tests/fixtures/go` (text files only).
- [x] Add `tests/integration_scan/style.rs` with test harness to write fixtures and assert findings.
- [x] Run unit and integration tests, adjust heuristics for noise.
- [x] Add benchmark run and document performance.

---

Implementation status: phase 1 shipped with heuristic wiring, fixtures, integration coverage, markdown docs, frontend docs, benchmark sanity checks, config override verification, and representative-repository quick-run validation.