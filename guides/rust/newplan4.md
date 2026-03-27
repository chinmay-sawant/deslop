# Error Handling & API Surface — Checklist Plan

## Summary / Quick Verdict

- [ ] **Quick verdict**: repo uses `anyhow` widely; has `panic!/unwrap()` detections and unbounded `read_to_string()` uses.
- [ ] **Remediation goal**: migrate library APIs to typed `Error` types, keep `anyhow` at binaries, remove panics/unwraps from library code, and add bounded IO + CI checks.

## Goals

- [ ] **Library errors**: Convert library/public APIs to typed `Error` enums using `thiserror`.
- [ ] **Binary ergonomics**: Keep `anyhow` at application edges (`main.rs`, CLI).
- [ ] **No panics/unwraps**: Eliminate `panic!` and `.unwrap()` in non-test library code.
- [ ] **Bounded IO**: Replace unbounded `fs::read_to_string` with size-limited readers/streaming.
- [ ] **CI enforcement**: Add lint/tests to detect regressions.

## Prep / Dependencies

- [ ] **Add `thiserror`** to `Cargo.toml` (`thiserror = "1.0"`).
- [ ] **Ensure `anyhow`** remains available for binaries (optional feature or workspace dep).

## Top-level API

- [ ] **Create `src/error.rs`**: define `pub enum Error` with `#[derive(thiserror::Error, Debug)]` and `pub type Result<T> = std::result::Result<T, Error>`.
- [ ] **Export** from `lib.rs`: `pub use crate::error::{Error, Result};`.

## Per-module Errors

- [ ] **Add module-level `Error` enums** for large modules (`analysis::parser`, `index`, `scan`).
- [ ] **Use `#[from]`** conversions to aggregate into the top-level `Error`.

## Replace `anyhow` in libraries

- [ ] **Replace library `anyhow::Result`** with `crate::error::Result<T>` where appropriate.

## Eliminate panics & unwraps

- [ ] **Replace `unwrap()`** sites with `ok_or(Error::...)` or `?` and the correct error variants.
- [ ] **Replace `panic!()`** in library code with domain errors; reserve `panic!` for unrecoverable internal bugs or use `debug_assert!`.
- [ ] **Audit and fix examples** such as `target_segments.last().unwrap()` and explicit `panic!` sites in `src/index/mod.rs`.

## Bounded IO helpers

- [ ] **Add `src/io.rs`** with `read_to_string_limited(path, max_bytes)` and `DEFAULT_MAX_BYTES` (e.g., 10 MiB).
- [ ] **Replace `fs::read_to_string`** usages with the bounded reader or `Read::take(max)` where streaming sources exist.

## Linting & CI

- [ ] **Clippy/CI policy**: run `cargo clippy --all-targets` and consider `-D clippy::unwrap_used -D clippy::expect_used` in CI.
- [ ] **Grep-based check**: add a CI step/script to fail on `panic!`, `.unwrap(`, `.expect(`, and `fs::read_to_string` in non-test code.
- [ ] **Add test**: a unit/integration test that scans the repo for forbidden patterns (with allow-list support).

## Concrete Migration Checklist (developer tasks)

- [ ] Add `thiserror` to `Cargo.toml`.
- [ ] Add `src/error.rs` and export `Error` and `Result` from `lib.rs`.
- [ ] Implement `read_to_string_limited` and replace easy `fs::read_to_string` calls.
- [ ] Fix obvious `unwrap()` occurrences in `index`, `analysis::parser`, and similar modules.
- [ ] Add per-module error enums and `#[from]` conversions.
- [ ] Convert public functions in library modules to return `Result<T>`.
- [ ] Update binaries (`main.rs`, CLI) to convert typed errors into `anyhow` with context for user messages.
- [ ] Add CI linting and grep-based checks.
- [ ] Write tests for error variants, size limits, and regressions.

## Migration Phases

- [ ] **Phase A (low-risk)**: add `thiserror`, top-level `Error`, `read_to_string_limited`, and fix trivial `unwrap()`/panic sites.
- [ ] **Phase B (medium-risk)**: add per-module error enums, convert public APIs, and update binaries.
- [ ] **Phase C (finish)**: run full compile/tests, add CI enforcement, finalize messages and docs.

## Developer Ergonomics

- [ ] **Provide `pub type Result<T>`** centrally for ease of use.
- [ ] **Provide small adapters/utilities** for converting IO/parse errors with context where helpful.
- [ ] **Add README examples** documenting library vs app boundary and patterns.

## Tests & Benchmarks

- [ ] **Unit tests** for error paths (missing files, parse errors, input-too-large).
- [ ] **Benchmark/test** to confirm bounded read behavior and memory footprint.

## PR Checklist

- [ ] Add `thiserror` to `Cargo.toml`.
- [ ] Create `src/error.rs` and export it.
- [ ] Implement `src/io.rs` bounded reader and replace call sites.
- [ ] Replace `unwrap()`/`panic!` in library code.
- [ ] Add per-module error enums and `#[from]` conversions.
- [ ] Update binaries to map typed errors to `anyhow` for user-facing messages.
- [ ] Add CI checks and tests.
- [ ] Run the full test suite and fix fallout.

## Acceptance Criteria

- [ ] **Typed public errors**: Library/public APIs return typed `Error` (no `Box`/`anyhow` in public signatures).
- [ ] **No unbounded reads**: No lingering `fs::read_to_string` without limits in production code.
- [ ] **No unwrap/panic**: No `unwrap()`/`panic!()` in non-test library code.
- [ ] **CI enforcement**: CI fails on re-introduction of banned patterns.

## Next Steps / Options

- [ ] I can implement Phase A (add `thiserror`, `src/error.rs`, and bounded reader) and open a PR.
- [ ] Or I can generate a focused PR converting `src/scan/mod.rs` read to bounded reader and fix a few `unwrap()` occurrences.

---

If you want me to proceed with an implementation, tell me which option to start with.
