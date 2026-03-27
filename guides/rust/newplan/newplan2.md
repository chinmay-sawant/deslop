# Error Handling & API Surface — Checklist Plan

## Summary / Quick Verdict

- [x] Findings captured and remediated in core library paths: typed crate-level `Error`/`Result` added, bounded file reads implemented, and the library parse/scan backends no longer expose `anyhow::Result`.
- [x] Key implementation points: `src/error.rs`, `src/io.rs`, `src/lib.rs`, `src/scan/mod.rs`, `src/scan/walker.rs`, `src/analysis/{mod,go,python,rust}/**`.
- [x] Binary edge remains on `anyhow::Result` with added context in `src/main.rs`.
- [ ] Test-only `expect`/`unwrap` calls remain in unit/integration tests by design.

## Implementation Status (2026-03-28)

- [x] Added `thiserror = "1.0"` to `Cargo.toml`.
- [x] Added `src/error.rs` with a top-level `Error` enum and central `Result<T>` alias.
- [x] Exported `Error`/`Result` and the bounded-reader helpers from `src/lib.rs`.
- [x] Added `src/io.rs` with `read_to_string_limited` and `DEFAULT_MAX_BYTES`.
- [x] Replaced the production `fs::read_to_string` path in the scanner with bounded reads.
- [x] Replaced the production `target_segments.last().unwrap()` site and removed other easy non-test `expect`/`unwrap` uses in library code.
- [x] Added CI enforcement via Clippy unwrap/expect denial and the grep-based hygiene script.
- [x] Added unit tests for bounded read success and oversized-input failure.
- [x] Added domain-modeling parser support and heuristics from the second half of this plan.
- [ ] Per-module `Error` enums with `#[from]` aggregation were not split out; this pass kept one crate-level `Error` type instead.
- [ ] Repo-level allowlist/config toggles were not added in this pass.

---

## Goals (checklist)

- [ ] Library code returns concrete typed errors (`thiserror` enums).
- [ ] Binaries (`main.rs`, CLI) keep `anyhow::Result` and attach human-facing context.
- [ ] Eliminate `panic!` / `.unwrap()` in non-test library code.
- [ ] Prevent unbounded reads; use size limits or streaming readers.
- [ ] Add CI enforcement to prevent regressions (panic/unwrap/unbounded reads).

---

## Strategy & Rationale (checklist)

- [ ] Adopt `thiserror` for library error enums; keep `anyhow` at the app edge.
- [ ] Provide a central `Error` and `Result<T>` alias exported from `lib.rs`.
- [ ] Replace `panic!`/`.unwrap()` with `?` / `ok_or(...)` returning typed variants.
- [ ] Add bounded IO helpers (metadata pre-checks, `Read::take(max)`); replace `fs::read_to_string` uses.
- [ ] Add CI/clippy/grep checks to detect regressions and enforce policy.

---

## Concrete Migration Checklist

### Prep

- [ ] Add `thiserror = "1.0"` to `Cargo.toml` for library crates.
- [ ] Keep `anyhow` as an optional dependency for binaries that need it.

### Top-level error & exports

- [ ] Add `src/error.rs` with a `pub enum Error` aggregating module errors.
- [ ] Add `pub type Result<T> = std::result::Result<T, Error>` in `src/error.rs`.
- [ ] Export `pub use crate::error::{Error, Result};` from `lib.rs`.

### Per-module errors

- [ ] Add `Error` enums for large submodules (`analysis::parser`, `index`, `scan`) with `#[from]` conversions.

### Replace `anyhow` in libraries

- [ ] Find and replace `use anyhow::Result` / `anyhow::Result` in library crates with `crate::error::Result`.

### Eliminate panics/unwraps

- [ ] Replace `target_segments.last().unwrap()` with `ok_or(Error::MissingModuleName)?` in [src/index/mod.rs](src/index/mod.rs#L111).
- [ ] Replace explicit `panic!(...)` occurrences in library code with domain error returns.
- [ ] Replace other `value.unwrap()` instances with `ok_or(...)` or contextual errors.

### Bounded IO

- [ ] Add `src/io.rs` (or similar) with `read_to_string_limited(path, max_bytes)` and a `DEFAULT_MAX_BYTES` constant.
- [ ] Replace `fs::read_to_string(path)` occurrences (e.g., [src/scan/mod.rs](src/scan/mod.rs#L118)) with bounded helpers.
- [ ] For streaming sources, use `Read::take(max)` and fail if exceeded.

### Binaries & ergonomics

- [ ] Update `main.rs`/CLI to convert typed `Error` into `anyhow::Error` and attach `Context` for user messages.

### CI & tests

- [ ] Add CI step: `cargo clippy --all-targets -- -D clippy::unwrap_used -D clippy::expect_used` (or equivalent).
- [ ] Add a grep-based CI check for `panic!` / `.unwrap(` / `.expect(` in non-test code as a pragmatic guard.
- [ ] Add tests asserting no `fs::read_to_string` calls remain (optional until migration completes).
- [ ] Add unit tests for error variants and the bounded IO behavior (too-large inputs).

---

## Migration Phases (checklist)

- [ ] Phase A (low-risk): add `thiserror` + `src/error.rs`; implement bounded IO; replace trivial `unwrap()` sites.
- [ ] Phase B (medium-risk): add per-module error enums; convert public functions to typed `Result<T>`; update binaries.
- [ ] Phase C (finish): run full compile/tests; add CI linting; perform PR reviews and ergonomic tweaks.

Estimated effort: 1–2 engineer-weeks depending on module scope.

---

## Developer Ergonomics (checklist)

- [ ] Provide `pub type Result<T> = std::result::Result<T, Error>` centrally.
- [ ] Add small adapter helpers and `#[from]` conversions for common error types (IO, parse).
- [ ] Document library-vs-app error boundary in `README` / `guides/rust/` with examples.

---

## Tests & Benchmarks (checklist)

- [ ] Add unit tests for error paths (missing files, parse errors, input-too-large).
- [ ] Add a small benchmark to validate bounded IO memory usage on large inputs.

---

## Checklist (developer tasks — condensed)

- [ ] Add `thiserror` to `Cargo.toml`.
- [ ] Add `src/error.rs` and export `Error`/`Result` from `lib.rs`.
- [ ] Implement `read_to_string_limited` and replace `fs::read_to_string` calls.
- [ ] Replace `unwrap()` / `panic!` occurrences in library code (start with `index` and `analysis::parser`).
- [ ] Add per-module error enums and `#[from]` conversions.
- [ ] Update binaries to use `anyhow` for user-facing messages.
- [ ] Add CI linting and grep-based checks to detect regressions.
- [ ] Write tests for limits and error variants.

---

# Domain Modeling & Invariants — Checklist Plan

## Overview (checklist)

- [ ] Goal: Add Rust-specific static checks to detect domain-modeling anti-patterns.
- [ ] Scope: Detect raw-primitive business values, impossible struct combinations, unsafe `Default`, `Debug` with secrets, and unsafe `Serialize`/`Deserialize` on sensitive fields.

---

## Parser & Data Model Changes (checklist)

- [ ] Add `StructSummary` and `FieldSummary` types in `src/analysis/types.rs`.
- [ ] Update `ParsedFile` to include `pub structs: Vec<StructSummary>`.
- [ ] Extend `src/analysis/rust/parser.rs` to extract struct fields and types.
- [ ] Mark `is_option` when type is `Option<>`; detect `is_primitive` for known primitives.

---

## Rules & Heuristics (checklist)

- [ ] `rust_domain_raw_primitive`: flag business-named fields using raw primitives (e.g., `price: f64`, `amount: i64`).
- [ ] `rust_domain_float_for_money`: flag `f32`/`f64` for monetary fields; recommend `rust_decimal::Decimal` or integer-cents newtype.
- [ ] `rust_domain_impossible_combination`: flag structs with `enabled` boolean + credential `Option` fields; recommend enum-based design.
- [ ] `rust_domain_default_produces_invalid`: flag `derive(Default)` on types producing unsafe defaults for `port`, `token`, `password`, etc.
- [ ] `rust_debug_secret`: flag `derive(Debug)` on types containing secret-like fields; recommend redaction or `secrecy::Secret`.
- [ ] `rust_serde_sensitive_deserialize`: flag `Serialize`/`Deserialize` derives on structs with secret fields lacking validation or custom deserialize.

---

## Implementation & Integration (checklist)

- [ ] Add `src/heuristics/rust_domain_modeling.rs` implementing rule functions returning `Vec<Finding>`.
- [ ] Integrate rules into `evaluate_rust_findings` in `src/analysis/rust/mod.rs`.
- [ ] Add allowlist mechanism (e.g., `// deslop-ignore:<rule_id>`) and config to tune severity.

---

## Tests & Fixtures (checklist)

- [ ] Add fixtures under `tests/fixtures/rust/domain_modeling/` covering positive/negative cases.
- [ ] Add integration tests asserting findings in `tests/integration_scan.rs` or a new test module.

---

## Prioritization & Rollout (checklist)

- [ ] Phase 1: Parser + `rust_domain_raw_primitive` + `rust_domain_impossible_combination`.
- [ ] Phase 2: `rust_debug_secret` + `rust_serde_sensitive_deserialize` + dbg! checks.
- [ ] Phase 3: Tuning, tests expansion, docs, and config flags.

---

## PR Checklist (checklist)

- [ ] Add `StructSummary` & `FieldSummary` types and update `ParsedFile`.
- [ ] Update `src/analysis/rust/parser.rs` to populate struct/field summaries.
- [ ] Add `src/heuristics/rust_domain_modeling.rs` with rule implementations.
- [ ] Integrate rules into `evaluate_rust_findings`.
- [ ] Add fixtures under `tests/fixtures/rust/domain_modeling/`.
- [ ] Add tests asserting findings in `tests/integration_scan.rs` or new module.
- [ ] Update `README.md` or `guides/rust/` with a short note about the new rules.

---

## Notes & Caveats (checklist)

- [ ] Some `unwrap()` conversions need semantic decisions — expect iterative PRs.
- [ ] For hot-paths, consider error construction cost when designing variants.
- [ ] If structured parsing is infeasible quickly, a temporary regex-based heuristic may be used (lower accuracy).

---

## Next Steps (you choose)

- [ ] I will implement `src/error.rs` and add `thiserror` to `Cargo.toml` (low-risk).
- [ ] I will implement `read_to_string_limited` and update `src/scan/mod.rs` (bounded IO first).

Pick one and I'll open a focused PR implementing it and the tests.
