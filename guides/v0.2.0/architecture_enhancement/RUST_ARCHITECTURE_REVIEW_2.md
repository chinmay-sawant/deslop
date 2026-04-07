# Rust Architecture Review

## Scope

This review focuses on the Rust-facing structure of `src/` and `tests/`, with emphasis on maintainability, navigability, and how well the current layout will scale as the rule set grows.

## Rating

- Overall Rust project structure: **9/10**
- `src/`: **9.2/10**
- `tests/`: **8.6/10**

## Executive Summary

This is a strong Rust project structure.

The architecture already has the traits expected in a mature Rust codebase:

- clean library/CLI separation via `src/lib.rs` and `src/main.rs`
- a clear scan pipeline in `src/scan/`
- sensible backend separation in `src/analysis/`
- good domain boundaries across parsing, indexing, heuristics, reporting, benchmarking, and config
- broad automated coverage with dedicated support code and fixture-driven integration tests

The reason this is not a 10/10 is not that the structure is weak. It is that a few parts are beginning to outgrow their current boundaries:

- some Rust-specific source files are too large and carry too many responsibilities
- the test harness is only partially centralized
- integration-test topology is functional but harder to navigate than it needs to be
- rule metadata and executable rule wiring still live in separate places

## What Is Excellent Already

### `src/`

- `src/lib.rs` presents a disciplined public API instead of exposing the whole crate internals.
- `src/main.rs` stays thin and behaves like a CLI shell over library functionality rather than a second application core.
- `src/scan/` is laid out well for the actual execution flow: discovery, parsing, indexing, evaluation, suppression, and reporting are separate concerns.
- `src/analysis/` is structurally strong. The backend abstraction is clear, and the typed evidence model is a good Rust-native way to support multiple languages without collapsing into stringly-typed plumbing.
- `src/index/`, `src/model/`, and `src/benchmark/` are placed where a Rust reader would expect them.
- the facade pattern introduced in `mod.rs` files is a good choice. It reduces root-level noise while still letting the subsystem boundaries read clearly.

### `tests/`

- the test portfolio is strong: CLI coverage, integration scans, corpus regressions, parser coverage, and feature-flag coverage are all represented.
- `tests/support/mod.rs` is the right direction. A shared test workspace abstraction is exactly what this project needs.
- fixture-driven tests are the correct fit for a static-analysis tool. They make rule intent visible and keep regressions cheap to add.
- `tests/parser_corpus_regression.rs` is especially strong from an architecture perspective because it protects parser correctness without over-coupling tests to implementation details.

## What Prevents a 10/10

### 1. Rust-specific modules are starting to exceed healthy size limits

The biggest structural issue in `src/` is not the top-level layout. It is that some Rust-specific files are becoming mini-subsystems on their own.

Current hotspots:

- `src/analysis/rust/parser/mod.rs`
- `src/analysis/rust/evaluate.rs`
- `src/heuristics/rust/api_design.rs`

These files are large enough that readers have to hold too much context in their head at once. That increases review cost, makes onboarding slower, and raises the risk of unrelated concerns changing together.

### 2. The test harness is centralized in principle, but not in practice

`tests/support/mod.rs` already provides the right abstraction, but a large portion of the integration suite still uses custom temp workspace creation plus explicit `fs::remove_dir_all(...)` cleanup.

That means the project currently has two testing styles:

- RAII-style shared workspace support
- ad hoc temp-dir lifecycle management inside individual test modules

That split is the main reason the `tests/` rating is below the `src/` rating.

### 3. Integration test topology is harder to navigate than it needs to be

`tests/integration_scan.rs` works, but it has become a large entrypoint with many `#[path = ...]` inclusions and shared helper indirection.

This pattern is valid, but it has three drawbacks:

- test module discovery is less obvious than first-class integration crates
- helper ownership is less obvious
- repeated fixture/write/cleanup patterns are easier to hide and harder to standardize

### 4. Rule metadata and executable rule wiring are still split

The project has improved here, but the structure still has two closely related registries:

- `src/rules.rs` for metadata and rollout policy
- `src/heuristics/registry.rs` for executable rule wiring

That separation is manageable today, but it still leaves room for drift as the rule catalog grows.

## Focused Assessment By Folder

### `src/`

This folder is architecturally sound.

The top-level split is good and the dependency direction is mostly intuitive:

- `analysis` parses and extracts language evidence
- `index` builds repository-level resolution data
- `heuristics` turns parsed evidence into findings
- `scan` orchestrates end-to-end execution
- `model` owns output types
- `cli` formats user-facing output

That is exactly the kind of shape you want in a Rust application crate.

The only reason `src/` is not already a 10 is that Rust-specific implementation detail has accumulated inside a small number of oversized files instead of being further decomposed by concern.

### `tests/`

This folder is good, but not yet architecturally clean.

The underlying strategy is right:

- high-value fixture coverage
- shared support helpers
- domain-oriented integration files
- dedicated regression coverage for parsers

The problem is consistency. The suite still contains a lot of manual workspace lifecycle code, which creates noise and makes the test architecture look flatter and less intentional than the production code.

## Plan To Reach 10/10

### Phase 1: Finish test harness consolidation

Target: make `tests/support/mod.rs` the single way tests create workspaces, write fixtures, and run scans.

Actions:

- replace `create_temp_workspace()` usage in `tests/integration_scan.rs` and child modules with `FixtureWorkspace`
- eliminate manual `fs::remove_dir_all(...)` calls from integration tests
- add small helper methods for common test flows, such as writing multiple files and scanning with custom options
- consider `assert_cmd` plus `predicates` for CLI tests so process assertions become cleaner and more idiomatic

Success criteria:

- no integration test manually manages temp-dir cleanup
- no duplicate temp workspace helper remains outside `tests/support/mod.rs`

### Phase 2: Split Rust parser internals by concern

Target: make the Rust parser navigable at the same level as the Go and Python parsers.

Actions:

- keep `src/analysis/rust/parser/mod.rs` as a facade and coordinator only
- split parser logic into focused files such as function extraction, unsafe-pattern collection, call extraction, and comment/doc extraction
- preserve `imports.rs` and `items.rs`, but continue the decomposition until each file has a clear single responsibility

Success criteria:

- `src/analysis/rust/parser/mod.rs` becomes primarily orchestration
- the parser can be understood by reading a small set of focused modules instead of one large file

### Phase 3: Split Rust evaluation hotspots

Target: keep evaluation orchestration thin and move rule-specific logic closer to the rule families that own it.

Actions:

- reduce the responsibility of `src/analysis/rust/evaluate.rs`
- break out import-resolution findings, leftover macro/call findings, unsafe findings, and other family-specific logic into smaller focused modules
- split `src/heuristics/rust/api_design.rs` into narrower files, for example API surface, ownership/state modeling, defaults/builders, and error/result patterns

Success criteria:

- no Rust evaluation file behaves like a catch-all
- Rust rule families are easy to locate from the filesystem alone

### Phase 4: Collapse rule metadata and rule binding into one source of truth

Target: define a rule once and derive all outputs from that single definition.

Actions:

- move toward a Rust-owned static rule catalog that carries id, language, severity, status, configurability, description, and binding location together
- generate `rules/registry.json` and any downstream documentation artifacts from that Rust catalog instead of maintaining parallel structure by hand

Success criteria:

- adding a rule requires touching one logical registry, not two
- CLI, docs, and runtime rule execution stay aligned by construction

### Phase 5: Add a compact architecture map for future contributors

Target: make the intended module boundaries explicit.

Actions:

- add crate-level docs in `src/lib.rs` describing the dependency flow across `analysis`, `index`, `heuristics`, `scan`, and `cli`
- document expected size and responsibility boundaries for subsystem facades versus implementation modules

Success criteria:

- new contributors can understand the intended structure before reading implementation details
- future refactors have an explicit target shape to preserve

## Bottom Line

This project is already above the usual standard for Rust application structure.

The top-level architecture is strong enough to scale. The gap between **9/10** and **10/10** is mostly cleanup and decomposition work, not a redesign.

If the team completes the four substantive items below, the structure reaches 10/10 territory:

1. fully centralize the test harness
2. split oversized Rust parser and evaluation files
3. reduce integration-test entrypoint complexity
4. unify rule metadata and executable rule registration
