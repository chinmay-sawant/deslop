# Runtime & Security Checks — Rust Checklist Plan

**Date:** 2026-03-27

**Purpose:** Actionable checklist to detect, prevent, and remediate runtime and security issues Clippy may miss.

## High-level goals

- [x] Prevent silent, security-relevant runtime behaviors from going unreported by adding repo scanning/reporting.
- [x] Enforce safe idioms via lints, CI, and tests.
- [x] Create small, reviewable remediation PRs grouped by category.
- [x] Add automated detection scripts and continuous enforcement in CI.

## Implementation Status (2026-03-28)

- [x] Added `scripts/check-rust-security.sh` to generate `reports/rust-security-baseline/latest.txt`.
- [x] Added `.github/workflows/rust-security.yml` with strict Clippy, tests, security grep reporting, and optional `cargo audit` / `cargo geiger` steps.
- [x] Added `reports/rust-security-baseline/.gitkeep` so the report path exists in-repo.
- [x] Confirmed no live `&String` / `&Vec<_>` production API signatures in `src/` during the ownership/API audit branch.
- [ ] The security report is currently informational by default; it does not fail CI unless `STRICT=1` is set.
- [ ] The generated report still shows candidate follow-up items such as slice indexing in `src/index/mod.rs` and one numeric cast in `src/io.rs`.

## Per-issue checklist

### 1) Integer overflow in release mode

- [ ] Grep for `checked_add|checked_sub|saturating_add|wrapping_add|overflowing_add` and arithmetic patterns
- [ ] Replace unintended arithmetic with `checked_*` or `saturating_*` and properly handle `None`/errors
- [ ] Add unit and property tests for numeric boundary cases
- [ ] Decide whether to enable `overflow-checks = true` in `[profile.release]` and document the decision

### 2) `as` casts for numeric narrowing — silent truncation

- [ ] Grep for numeric `as` casts across `**/*.rs`
- [ ] Replace narrowing `as` with `TryFrom`/`try_into()` and handle conversion errors
- [ ] Add CI/script rule to flag new narrowing `as` uses or require explicit review/comments for intentional casts

### 3) `split_at` / unchecked indexing

- [ ] Grep for `.split_at(` and `[..]` slice-indexing patterns on slices derived from external input
- [ ] Replace with `slice.get(..)` or add explicit index/bounds guards before slicing
- [ ] Add tests for panics/boundary conditions

### 4) TOCTOU filesystem races

- [ ] Grep for `exists(`, `metadata(`, `symlink_metadata`, `read_link`, and `open(` used in check-then-open patterns
- [ ] Use `canonicalize()` and assert canonical paths begin with expected root when accepting user paths
- [ ] On Unix, use `OpenOptionsExt::custom_flags(libc::O_NOFOLLOW)` where appropriate to avoid symlink races
- [ ] Add integration tests or targeted checks simulating symlink attacks where practical

### 5) Non-constant-time comparison of secrets

- [ ] Grep for `password|token|secret|api_key` and inspect equality comparisons
- [ ] Replace `==` comparisons for secrets with constant-time compare (e.g., `subtle::ConstantTimeEq` / `ct_eq`)
- [ ] Add tests asserting expected comparison behavior for secrets

### 6) `Rc` cycles / `Weak` usage

- [ ] Grep for `Rc<` and `RefCell` ownership patterns and inspect possible cycles
- [ ] Convert parent/owner pointers to `Weak` where cycles may occur
- [ ] Add code review checklist item to flag potential cycles

### 7) `static mut` / `lazy_static!` without proper `Sync`

- [ ] Grep for `static mut` and `lazy_static!`
- [ ] Replace with `once_cell::sync::Lazy` + `Mutex`/`RwLock` or use atomic types as appropriate
- [ ] Add concurrency tests or document invariants and sync requirements

### 8) Accepting absolute paths in `Path::join`

- [ ] Grep for `Path` / `PathBuf` `.join()` usage that accepts external input
- [ ] Reject absolute inputs before joining or strip leading root components
- [ ] `canonicalize()` and assert `starts_with(expected_base)` before using results

### 9) `#[derive(Default)]` misuse

- [ ] Grep for `derive(Default)` and review types where a default value could represent an invalid state
- [ ] Replace `derive(Default)` with custom `impl Default` or remove it; prefer constructor functions that return `Result` when applicable

### 10) `cargo-geiger` / unsafe-deps audit

- [ ] Add `cargo-geiger` and `cargo-audit` to CI
- [ ] Run locally and in CI; fail on regressions or unacceptable unsafe counts
- [ ] Triage unsafe-dependency results and document mitigation options

### 11) `std::thread::spawn` with un-awaited futures

- [ ] Grep for `spawn(async` and for calls that invoke async fns without `.await`
- [ ] Use a runtime (`tokio::spawn`) or create a `Runtime` + `block_on` in the thread to run futures
- [ ] Add tests ensuring spawned async work executes as intended

### 12) Large `async fn` state machines / `Box::pin`

- [ ] Identify very large `async fn` bodies and large state machines
- [ ] Refactor into smaller `async` functions or return boxed futures: `Pin<Box<dyn Future<Output = _> + Send>>`
- [ ] Add code-size or CI checks if necessary (optional)

## Cross-cutting actions (implementation checklist)

- [ ] Create `scripts/check-rust-security.sh` (grep-based) that produces `reports/rust-security-baseline/`
  - [ ] Search for numeric `as` casts
  - [ ] Search for `.split_at(` and `[..]` indexing
  - [ ] Search for `exists()`/`metadata()` followed by `open()` patterns
  - [ ] Search for `password|token|secret` and equality comparisons
  - [ ] Search for `static mut`, `lazy_static!`, `derive(Default)`
  - [ ] Search for `spawn(async` and `thread::spawn` patterns
- [ ] Commit baseline outputs under `reports/rust-security-baseline/` (CI baseline acceptance)
- [ ] Add `.github/workflows/rust-security.yml` with steps:
  - [ ] `cargo clippy --all-targets --all-features -- -D warnings`
  - [ ] `cargo test --all-features --all-targets`
  - [ ] `cargo audit`
  - [ ] `cargo geiger`
  - [ ] `scripts/check-rust-security.sh` (fail on new critical hits)
- [ ] Configure CI to accept baseline initially and fail on new issues thereafter
- [ ] Run `cargo audit` / `cargo geiger` locally and triage results
- [ ] Create remediation PRs grouped by category (one category per PR when possible)
- [ ] Add owners and triage responsibilities to `MAINTAINERS.md`
- [ ] Document policy decisions in `CONTRIBUTING.md` or `SECURITY.md` (overflow policy, `as` casts, secret handling)

## CI snippets & developer commands

```bash
rustup toolchain install stable
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --all-targets
cargo install cargo-audit cargo-geiger || true
cargo audit && cargo geiger || true
./scripts/check-rust-security.sh
```

Minimal GitHub Actions job example:

```yaml
name: Rust security checks
on: [push, pull_request]
jobs:
  rust-security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: cargo clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      - name: cargo test
        run: cargo test --all-features --all-targets
      - name: cargo audit
        run: |
          cargo install cargo-audit || true
          cargo audit
      - name: cargo geiger
        run: |
          cargo install cargo-geiger || true
          cargo geiger || true
      - name: grep-based security checks
        run: scripts/check-rust-security.sh
```

## Quick checklist & next actions

- [ ] Create `scripts/check-rust-security.sh` and commit baseline
- [ ] Add draft `.github/workflows/rust-security.yml`
- [ ] Decide `overflow-checks = true` for `[profile.release]`
- [ ] Run `cargo audit` / `cargo geiger` and triage findings
- [ ] Start PRs fixing highest-priority hits (`as` casts, secret comparisons, TOCTOU)

---

If you want, I can now:

- create the grep script and baseline report
- add the draft GitHub Actions CI workflow
- start the first remediation PR fixing top-priority hits
