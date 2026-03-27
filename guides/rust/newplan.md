## Runtime & Security Checks — Rust support (Clippy-blind items)

**Date:** 2026-03-27

### Purpose

This document records findings from a quick repository scan and provides a very detailed, actionable plan to detect, prevent, and remediate a set of runtime and security issues that Clippy may miss. Follow the implementation steps and CI snippets below to get these checks enforced automatically.

### Repository scan (quick findings)

- **Integer-overflow checks:** PARTIAL — repository uses `saturating_*` in several places (e.g. `src/index/mod.rs`, parser files). No global release overflow-checks profile found.
- **`as` numeric narrowing casts:** MISSING — no clear instances of numeric `as` narrowing found by targeted search in Rust sources; still audit source for non-obvious uses.
- **`split_at` / unchecked indexing:** PARTIAL — no `split_at` calls found in quick scan; many uses of `get()` and ordinary slice manipulation exist. Manual review required for `[]` indexing sites.
- **TOCTOU (check-then-use filesystem races):** MISSING — no canonicalization or O_NOFOLLOW-style defensive code discovered; filesystem handling appears limited to directory walking.
- **Constant-time secret comparison:** MISSING — no `subtle` / `constant_time_eq`-style usage found; compare-sites for secrets (passwords, tokens) appear mainly in tests.
- **`Rc` cycles / Weak usage:** MISSING — no `Rc<`/`Weak<` usages discovered during focused scan.
- **`static mut` / `lazy_static!` checks:** MISSING — no `static mut` or `lazy_static!` occurrences found; prefer `once_cell` + synchronization primitives when needed.
- **`Path::join` with absolute inputs:** PARTIAL — `Path`/`PathBuf` usage is common (e.g. `src/index/mod.rs`, `src/scan/walker.rs`), but there is no systematic validation to reject attacker-supplied absolute paths.
- **`#[derive(Default)]` misuse:** MISSING — no obvious `derive(Default)` hotspots found in quick scan; still search for types where a constructed Default would represent an invalid state.
- **`cargo-geiger` / unsafe-deps audit:** MISSING — no CI usage detected; recommend adding `cargo-geiger` and `cargo-audit` to CI.
- **`std::thread::spawn` with un-awaited futures:** MISSING — no obvious patterns found, but search for `spawn(async` should be included in checks.
- **Large `async fn` state machines / Box::pin:** MISSING — no `Box::pin`/`Pin` usage found in a quick search; identify very large `async fn`s during code review.
- **Path canonicalization usage:** MISSING — `Path::canonicalize()` calls not found in scan; add canonicalization where user-provided paths are accepted.

This scan was a targeted automated sweep to establish a baseline. It is not exhaustive — a human review will be necessary for high-risk modules.

---

## High-level goals

1. Prevent silent, security-relevant runtime behaviors (overflows, truncation, TOCTOU, path traversal).
2. Enforce safe idioms via lints, CI, and tests (Clippy + custom checks + cargo-audit/geiger).
3. Create an actionable remediation backlog with small, reviewable PRs.
4. Add automated detection (grep / scripts) and continuous enforcement in CI.

---

## Detailed plan (per issue)

### 1) Integer overflow in release mode (wrapping instead of panic)

- **Risk:** arithmetic that silently wraps in release builds can produce incorrect results and security bugs.
- **Detection:**
  - Search for arithmetic operators and for uses of `checked_*`, `saturating_*`, `wrapping_*`.
  - Grep examples: `checked_add|checked_sub|saturating_add|wrapping_add|overflowing_add`.
- **Remediation options:**
  - Prefer checked APIs (`checked_add`, `checked_sub`) and explicitly handle `None`.
  - Use `saturating_*` where saturation is desired (repo already uses this in places).
  - Use explicit `wrapping_*` where wrapping is intended and document intent.
  - Optionally enable overflow checks in release builds (team decision) by adding to `Cargo.toml`:

```toml
[profile.release]
overflow-checks = true
```

- **Tests / Validation:** unit tests for numeric boundary cases; property-based tests for math helpers.

### 2) `as` casts for numeric narrowing — silent truncation

- **Risk:** `value as u8` silently truncates bits and hides errors.
- **Detection:** grep for numeric `as` cast patterns across `**/*.rs`.
- **Remediation:** replace narrowing `as` with checked conversions:

```rust
use std::convert::TryFrom;
let n = u8::try_from(value).map_err(|_| anyhow::anyhow!("narrowing overflow"))?;
```

- **Policy:** disallow unreviewed narrowing `as` casts; require `TryFrom` or explicit comment + test for intentional truncation.

### 3) `split_at` / indexing that can panic at runtime

- **Risk:** `slice[a..b]` or `split_at` panics when indexes are out-of-range.
- **Detection:** `\.split_at\(` and bracket-index patterns for slices from external input.
- **Remediation:** prefer `slice.get(..)` and handle `None`, or guard with explicit `if idx <= s.len()` checks before `split_at`.

### 4) TOCTOU races (check-then-use on filesystem)

- **Risk:** attacker swaps a file between `exists()` and `open()`, or uses symlinks to cause access to unexpected files.
- **Detection:** grep for `exists\(|metadata\(|symlink_metadata|read_link|open\(` patterns used in check-then-open sequences.
- **Remediation:**
  - Use `canonicalize()` and ensure canonicalized path prefix containment.
  - On Unix, prefer `OpenOptionsExt::custom_flags(libc::O_NOFOLLOW)` to avoid symlink races when appropriate.

Example (Unix):

```rust
use std::os::unix::fs::OpenOptionsExt;
use std::fs::OpenOptions;
let mut opts = OpenOptions::new();
opts.read(true).custom_flags(libc::O_NOFOLLOW);
let f = opts.open(path)?; // fails if path is a symlink
```

### 5) Non-constant-time comparison of secrets

- **Risk:** using `==` for passwords/tokens leaks timing information.
- **Detection:** grep for `password|token|secret|api_key` and inspect equality comparisons.
- **Remediation:** use `subtle` / `ring` constant-time compare functions, e.g. `ct_eq` from `subtle`.

### 6) `Rc` cycles without `Weak`

- **Risk:** cycles between `Rc` values leak memory in single-threaded code.
- **Detection:** grep for `Rc<` and `RefCell` patterns and manually inspect tree/parent pointer implementations.
- **Remediation:**
  - Introduce `Weak` references for parent pointers and only hold strong `Rc` references for owning children.

### 7) `static mut` or `lazy_static!` without proper `Sync`

- **Risk:** unsynchronized mutable statics cause UB in multi-threaded contexts.
- **Detection:** grep for `static mut` and `lazy_static!`.
- **Remediation:**
  - Use `once_cell::sync::Lazy` + `Mutex`/`RwLock`, or `static` with `Atomic*` types when appropriate:

```rust
use once_cell::sync::Lazy;
static GLOBAL: Lazy<std::sync::Mutex<MyType>> = Lazy::new(|| Mutex::new(MyType::new()));
```

### 8) Accepting absolute paths in `Path::join`

- **Risk:** `base.join("/etc/passwd")` ignores base on unix and yields the absolute path passed by an attacker.
- **Detection:** grep for `Path`/`PathBuf` `join` usage that may accept user input.
- **Remediation:** explicitly reject absolute inputs or strip a leading root component before joining. Always `canonicalize()` and check that the result begins with the expected root.

Example:

```rust
let candidate = Path::new(user_input);
if candidate.is_absolute() { return Err(...); }
let joined = base.join(candidate);
let canon = joined.canonicalize()?;
if !canon.starts_with(base.canonicalize()?) { return Err(...); }
```

### 9) Deriving `Default` on types that should never have a useful default

- **Risk:** `#[derive(Default)]` may produce invalid object graphs or silent corruption.
- **Detection:** grep for `derive(Default)` and review types manually.
- **Remediation:** replace `derive(Default)` with a custom `impl Default` or remove it; prefer constructor functions that return `Result` when default state may be invalid.

### 10) No `cargo-geiger` audit on unsafe dependencies

- **Risk:** unsafe code in dependencies can introduce UB and hidden vulnerabilities.
- **Detection:** use `cargo geiger` and `cargo audit`.
- **Remediation:** add a CI job that runs `cargo geiger` and `cargo audit` and fails on regressed counts.

### 11) `std::thread::spawn` with un-awaited futures

- **Risk:** creating a `Future` but not polling it (calling an async fn without `.await`) leads to no work being done.
- **Detection:** grep for `spawn\(async` and for calls like `thread::spawn(|| some_async_fn())` (async fn invoked but not awaited).
- **Remediation:**
  - Execute futures on a runtime: `tokio::spawn` or create a `Runtime` and `block_on` the future in the thread.

### 12) Big `async fn` -> huge state machines without `Box::pin`

- **Risk:** very large async functions lead to very large state machines; boxing or refactoring reduces stack pressure and binary size.
- **Detection:** locate very large `async fn` bodies.
- **Remediation:**
  - Refactor large `async fn` into smaller functions or return boxed futures:

```rust
fn handler(...) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
    Box::pin(async move { /* large body */ })
}
```

---

## Cross-cutting actions (implementation plan)

1. Create `scripts/check-rust-security.sh` (grep-based) that produces `reports/rust-security-baseline/` (baseline output). This script should at minimum search for:
   - numeric `as` casts
   - `.split_at(` and `[..]` indexing on slices
   - `exists()`/`metadata()` followed by `open()` patterns
   - `password|token|secret` names
   - `static mut`, `lazy_static!`, `derive(Default)`
   - `spawn(async` and `thread::spawn` patterns

2. Add CI workflow `.github/workflows/rust-security.yml` with steps:
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test --all-features --all-targets`
   - `cargo audit` and `cargo geiger`
   - `scripts/check-rust-security.sh` -> produce failure if new critical hits appear (baseline mode allowed for initial onboarding)

3. Create a baseline report directory `reports/rust-security-baseline/` committed with the initial findings so CI can accept the baseline and fail on new issues only.

4. Iterate on remediation PRs in small batches by category (one category per PR where possible).

5. Track owners and assign triage of baseline findings (small table in `MAINTAINERS.md`).

---

## CI snippets & developer commands

Minimal commands to run locally:

```bash
rustup toolchain install stable
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --all-targets
cargo install cargo-audit cargo-geiger || true
cargo audit && cargo geiger || true
./scripts/check-rust-security.sh
```

Minimal GitHub Actions job (example):

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

---

## Checklist & next steps

- [ ] Add `scripts/check-rust-security.sh` and the baseline report.
- [ ] Add `.github/workflows/rust-security.yml` (draft) to CI.
- [ ] Decide whether to enable `overflow-checks = true` in `[profile.release]`.
- [ ] Run `cargo-audit` / `cargo-geiger` and triage results.
- [ ] Start remediation PRs, prioritizing `as` casts, secret comparisons, and TOCTOU-sensitive file handling.

If you want, I can now:

- create the grep script and baseline report,
- add the draft GitHub Actions CI workflow,
- or start the first small PR fixing the highest-priority grep hits.

Tell me which of these to do next.
