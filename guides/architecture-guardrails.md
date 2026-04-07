# Architecture Guardrails

Date: 2026-04-07

This guide captures the ownership model, large-file watchlist, and validation loop that should keep `src/` and `tests/` from drifting back toward monolithic modules.

## Ownership Expectations

### `src/analysis`

- owns parsing, evidence extraction, and language-specific syntax interpretation
- should not own rule execution orchestration
- module facades should stay thin and prefer submodules for imports, symbols, model summaries, and function-body evidence

### `src/heuristics`

- owns finding generation and family-level execution contracts
- each language family should have a small facade with helper or concern-specific submodules when the family grows beyond one obvious concern
- runtime dispatch should go through shared execution specs rather than backend-specific evaluator seams

### `src/index`

- owns repository-wide resolution, cross-file lookup, and import/module resolution support
- `mod.rs` should stay a facade over build and resolution details
- Rust- or language-specific resolution helpers should live in leaf modules unless they are truly shared API

### `src/rules`

- owns public metadata, inventory assembly, and validation
- catalog assembly should remain mechanical
- family metadata should be grouped by family and language so adding a rule is not a multi-file scavenger hunt

### `tests`

- owns behavioral contracts, regression coverage, and characterization checks
- `tests/support/mod.rs` is the default home for workspace setup, fixtures, scan helpers, and shared assertions
- `tests/integration_scan/*` should mirror production ownership by language and family where practical
- large mixed-language suites should be the exception, not the default

## Advisory Size Thresholds

These are review triggers, not hard failure thresholds.

- `> 250` lines: ask whether the file is now mixing more than one concern
- `> 400` lines: require an explicit “why this stays together” check during review
- `> 700` lines: default toward decomposition into a facade plus submodules
- `> 1000` lines: treat as architectural debt unless the file is generated or data-like

The point is to trigger decomposition conversations early, not to force arbitrary micro-files.

## Current Large-File Snapshot

Post-refactor `src` hotspots:

- `1768` lines: `src/heuristics/go/framework_patterns/gin.rs`
- `1592` lines: `src/heuristics/go/framework_patterns/hot_path.rs`
- `1123` lines: `src/heuristics/go/idioms.rs`
- `1052` lines: `src/heuristics/go/library_misuse/library.rs`
- `883` lines: `src/heuristics/go/framework_patterns/data_access/sql.rs`
- `868` lines: `src/heuristics/go/framework_patterns/data_access/gorm.rs`
- `860` lines: `src/rules/catalog/go/performance.rs`
- `830` lines: `src/rules/catalog/go/data_access.rs`
- `760` lines: `src/rules/catalog/go/security.rs`
- `754` lines: `src/heuristics/python/hotpath_ext.rs`

Post-refactor `tests` hotspots:

- `455` lines: `tests/integration_scan/rust/advanced.rs`
- `411` lines: `tests/integration_scan/go/framework_patterns.rs`
- `292` lines: `tests/integration_scan/context.rs`
- `252` lines: `tests/integration_scan/rust/core.rs`
- `249` lines: `tests/integration_scan/python/phase5_rules/structure.rs`
- `245` lines: `tests/integration_scan/go/library_misuse.rs`
- `240` lines: `tests/integration_scan/python/phase5_rules/duplication.rs`
- `229` lines: `tests/integration_scan/performance.rs`
- `224` lines: `tests/integration_scan/python/baseline/repo.rs`
- `220` lines: `tests/integration_scan/python/baseline/rules.rs`

## Review Rules

- When touching a file already above `400` lines, prefer extracting one real concern instead of adding more branches.
- When adding new tests for a specific family, place them under that language/family subtree before extending a generic bucket file.
- If a new rule requires edits in more than one heuristics file and more than one catalog file, stop and simplify the ownership path first.
- When a facade file starts carrying implementation-heavy helpers, split the helpers before adding new behavior.

## Normal Validation Loop

Run these together for architecture-heavy changes:

- `cargo fmt --check`
- `cargo test --quiet`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --quiet -- scan . --json --no-fail`

## Performance Visibility

Keep self-scan timing visible in architecture changes. A structural cleanup is not complete if it quietly adds noticeable scan overhead or increases finding drift without explanation.

Record at least:

- files discovered
- files analyzed
- functions found
- findings
- total scan time

Update the architecture plan or this guide when the shape of the hotspot list changes meaningfully.
