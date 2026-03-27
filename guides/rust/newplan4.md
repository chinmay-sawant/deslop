# Error Handling & API Surface — Detailed Plan

## Summary / quick verdict

- Findings: the codebase currently uses `anyhow` widely (application-style errors), has explicit detections for `panic!` / `.unwrap()` in analysis rules, and contains live occurrences of `panic!`, `.unwrap()` and unbounded `read_to_string()` reads.
- Missing: I did not find `thiserror` usage or a consistent typed library error strategy. There are also direct `read_to_string` calls without size checks.

Key example spots observed in the repo:

- [src/main.rs](src/main.rs#L1) — uses `anyhow::Result`.
- [src/cli/report.rs](src/cli/report.rs#L4) — uses `anyhow::Result`.
- [src/scan/mod.rs](src/scan/mod.rs#L118) — `fs::read_to_string(path)` (unbounded file read).
- [src/index/mod.rs](src/index/mod.rs#L111) — `target_segments.last().unwrap()`.
- [src/index/mod.rs](src/index/mod.rs#L594) — `panic!("expected resolved import, got {other:?}")` (and similar panics at L662, L696, L704, L715).
- [src/analysis/rust/parser.rs](src/analysis/rust/parser.rs#L1089) — `value.unwrap()`.
- [src/analysis/rust/mod.rs](src/analysis/rust/mod.rs#L67) — rule text referencing `panic!` / `unwrap()` detection.

Conclusion: detection exists, but remediation and API ergonomics are not implemented. We should adopt a library-vs-application error strategy, migrate library code to typed errors (thiserror), stop panicking/unwraping in library code, and introduce size-bounded IO helpers to avoid DoS via large inputs.

---

## Goals

1. Library code (anything exported from `lib.rs` or meant for reuse) must return concrete, typed error types (no `String` or `Box<dyn std::error::Error>` as public error types).
2. Application binaries (`main.rs`, CLI) should continue to use `anyhow::Result` and attach context for user-facing messages.
3. Eliminate `panic!` and `.unwrap()` from non-test library code; return errors instead.
4. Prevent unbounded memory use when reading inputs (files, network, archives) by enforcing size limits and streaming where possible.
5. Add CI enforcement (lint/tests) that detects regressions (panic/unwrap/unbounded reads) and ensures ergonomic API patterns are followed.

---

## Strategy & rationale

- Use `thiserror` to define typed `Error` enums per module (and a top-level `deslop::Error`) so callers can pattern-match or convert errors without losing type information.
- Keep `anyhow` at the binary edge only. Application code can convert typed errors into `anyhow::Error` (via `anyhow::Error::from`) or use `Context` for human-friendly messages.
- Replace ad-hoc `panic!` / `.unwrap()` with `?`-propagation into typed error variants. Reserve `panic!` for truly unrecoverable, internal-logic-bug cases if ever — but prefer `debug_assert!` in such cases and return errors otherwise.
- For file / input reading, prefer pre-checking size via `metadata()` where possible, or use streaming readers with `take(max)` to cap bytes read.

---

## Concrete migration plan (step-by-step)

1) Prep: add dependencies

   - Add to `Cargo.toml` (workspace or crate where library code lives):

```toml
[dependencies]
thiserror = "1.0"
anyhow = { version = "1", optional = true }
```

   - Rationale: `thiserror` for typed library errors; `anyhow` already used for application code.

2) Create a central `Error` and module pattern

   - Add `src/error.rs` (or `src/lib.rs` export) with a top-level `pub enum Error` that aggregates module errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse: {0}")]
    Parse(#[from] crate::analysis::parser::Error),

    #[error("input too large: {0} > {1}")]
    InputTooLarge(u64, u64),

    #[error("missing module name")]
    MissingModuleName,

    // module-specific variants (or use `#[from]` for module error enums)
}

pub type Result<T> = std::result::Result<T, Error>;
```

   - Export `pub use crate::error::{Error, Result};` from `lib.rs`.

3) Per-module typed errors

   - For larger submodules (e.g., `analysis::parser`, `index`, `scan`) add a small `Error` enum in their module namespace:

```rust
// src/analysis/parser/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("syntax: {0}")]
    Syntax(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
```

   - Let the top-level `Error` use `#[from]` to convert module errors into the public `Error`.

4) Replace `anyhow::Result` in library crates with the typed `Result` alias

   - Grep for `use anyhow::Result` / `-> Result<` in crates intended as libraries and incrementally replace them with `use crate::error::Result` and `-> Result<T>`.

   - Keep `anyhow` in binaries (`main.rs`, cli entrypoints). Example pattern for `main.rs`:

```rust
fn main() -> anyhow::Result<()> {
    if let Err(e) = deslop::run() {
        eprintln!("error: {:#}", anyhow::Error::new(e).context("running deslop"));
        std::process::exit(1);
    }
    Ok(())
}
```

5) Replace panics/unwraps with error returns

   - Replace `target_segments.last().unwrap()` (see [src/index/mod.rs](src/index/mod.rs#L111)) with:

```rust
let module_name = target_segments
    .last()
    .ok_or(Error::MissingModuleName)?;
```

   - Replace `panic!("expected ...")` branches with returning a domain error:

```rust
match thing {
    Expected => { /* ... */ }
    other => return Err(Error::UnexpectedImportKind(format!("{:?}", other))),
}
```

   - Replace `value.unwrap()` with `let v = value.ok_or(Error::MissingValue)?;` or `value.context("...")?` where appropriate.

6) Input-size limits and streaming IO

   - Implement a utility in e.g. `src/io.rs`:

```rust
use std::fs;
use std::io::Read;
use std::path::Path;

pub const DEFAULT_MAX_BYTES: u64 = 10 * 1024 * 1024; // 10 MB (tunable)

pub fn read_to_string_limited(path: &Path, max_bytes: u64) -> Result<String, Error> {
    let meta = fs::metadata(path)?;
    if meta.len() > max_bytes {
        return Err(Error::InputTooLarge(meta.len(), max_bytes));
    }
    let mut s = String::new();
    fs::File::open(path)?.read_to_string(&mut s)?;
    Ok(s)
}
```

   - Replace `fs::read_to_string(path)` calls (e.g., [src/scan/mod.rs](src/scan/mod.rs#L118)) with `read_to_string_limited(path, DEFAULT_MAX_BYTES)` or a configurable limit.

   - For streaming sources (network, stdin, archive entries) use `Read::take(max_bytes)` and fail if the reader produces more than allowed.

7) Linting / CI enforcement

   - Add a CI step (or augment existing CI) to fail on new occurrences of `panic!` / `.unwrap()` / `.expect()` in non-test code. Options:
     - Use `cargo clippy --all-targets -- -D clippy::unwrap_used -D clippy::expect_used` in CI (note: `clippy::panic` doesn't exist; use grep as backup).
     - Add a small grep-based test that scans `src/**` for `panic!` / `.unwrap(` / `.expect(` and fails if found outside `tests/` or `#[cfg(test)]` blocks. (A targeted script that ignores allowed instances is pragmatic.)

   - Add a test to assert that there are no `fs::read_to_string` calls in the codebase (optional until migration completes).

8) Migration sequencing (safe, incremental)

   - Phase A (low-risk, 1–2 days):
     - Add `thiserror` and `src/error.rs` top-level.
     - Implement `read_to_string_limited` and replace easy file reads (scan module).
     - Fix obvious `unwrap()` occurrences that are local and have a clear error case (`index`, `parser` checks).

   - Phase B (medium-risk, 2–4 days):
     - Introduce per-module error enums for `analysis::parser`, `index`, `scan`.
     - Convert public functions in those modules to return typed `Result<T>`.
     - Update binaries (`main.rs`, CLI) to `map`/`context` errors into `anyhow` for user messages.

   - Phase C (finish, 1–2 days):
     - Run full compilation and tests, fix fallout.
     - Add CI linting to prevent regressions.
     - Perform code review and adjust error message texts for ergonomics.

Estimate: 1-2 engineer-weeks depending on how many modules are large and how many call sites need careful error-handling.

9) Developer ergonomics

   - Provide `pub type Result<T> = std::result::Result<T, Error>` centrally so module code uses `Result<T>`.
   - Provide small adapters/utilities to convert IO/parse errors with context: e.g., `fn io_err<E: Into<std::io::Error>>(e: E) -> Error { Error::Io(e.into()) }` or prefer `#[from]` where possible.
   - Add examples in `README`/guides documenting the library vs app boundary and showing the new patterns.

10) Tests & benchmarks

   - Add unit tests that exercise error paths (missing files, too-large files, parse errors) to ensure error variants are constructed.
   - Add a small benchmark that loads large inputs to confirm the `read_to_string_limited` behavior and memory footprint.

---

## Checklist (developer tasks)

- [ ] Add `thiserror` to `Cargo.toml`.
- [ ] Add `src/error.rs` and export `Error` and `Result` from `lib.rs`.
- [ ] Implement `read_to_string_limited` and replace `fs::read_to_string` calls.
- [ ] Replace `unwrap()` / `panic!` occurrences in library code with typed errors (start with `index` and `analysis::parser`).
- [ ] Add per-module error enums where needed and `#[from]` conversions.
- [ ] Update binaries to use `anyhow` for user-facing messages.
- [ ] Add CI linting and grep-based checks to detect regressions.
- [ ] Write tests for limits and error variants.

---

## Notes & caveats

- Some `.unwrap()` occurrences can be trivially converted to `?` / `ok_or(...)` but others require thought (the correct error variant and message). Expect iterative refinement and PR reviews.
- For very performance-sensitive hot paths, consider error construction cost; reuse light-weight error variants where appropriate and document trade-offs.

---

If you want, I can now:

- Implement the `src/error.rs` top-level and add `thiserror` to `Cargo.toml` (small, low-risk PR), or
- Generate a first PR converting `src/scan/mod.rs`'s `read_to_string` to a bounded reader and replace the few obvious `unwrap()`/`panic!` usages.

Tell me which of the two you'd like me to start on (or I can open both PRs sequentially).
**Domain Modeling & Invariants — Rust support plan**

**Overview**
- **Goal**: Add Rust-specific static checks to detect and remediate domain-modeling anti-patterns so invalid states become harder to represent and easier to review.
- **Scope**: Detect raw-primitive usage for business values, structs that allow impossible combinations (boolean + Option fields), unsafe use of `#[derive(Default)]`, `#[derive(Debug)]` on secret-bearing types, and `Serialize/Deserialize` usage on sensitive fields without validation — limited to Rust source analysis in this repository.

**Current status (quick scan evidence)**
- **Shared model uses primitives**: many report structures use primitives (see [src/model/mod.rs](src/model/mod.rs)).
- **Rust analyzer parses symbols but not struct fields**: `src/analysis/rust/parser.rs` collects `struct_item` names into `DeclaredSymbol` but does not capture field names/types for structs; see [src/analysis/rust/parser.rs](src/analysis/rust/parser.rs#L200-L260).
- **Heuristics exist for secrets**: there are secret-detection heuristics in `src/heuristics/security.rs` but no domain-modeling rules for Rust yet.

**High-level approach**
1. Extend the Rust parser to extract struct/type definitions with field-level metadata (field name, type text, visibility, and whether the type is an `Option<>` or primitive).
2. Add new heuristics functions (Rust-scoped) that inspect the collected struct metadata and emit Findings for the rules below.
3. Integrate the heuristics into the Rust analyzer (`evaluate_rust_findings` in `src/analysis/rust/mod.rs`).
4. Add tests & fixtures demonstrating the anti-patterns and the recommended fixes.
5. Iterate on rule quality (tuning false positives, severity levels, and exceptions).

**Detailed detection rules and implementation notes**

**1) Raw primitives used for business values**
- **Rule id**: `rust_domain_raw_primitive`
- **What to detect**: Struct fields whose type is a raw primitive (e.g., `i32`, `i64`, `u32`, `f64`, `String`) where the field name indicates a business semantic (name matches a curated list/regex such as `amount|price|balance|money|cost|distance|username|user_name|email|url|path|port|ip`). Also detect `f32`/`f64` used for money.
- **Algorithm**:
  - Require the parser to produce for each `struct_item`: a `StructSummary { name, line, fields: Vec<FieldSummary> }` where `FieldSummary` has `name: String`, `type_text: String`, `is_pub: bool`, `is_option: bool`, `is_primitive: bool`.
  - For each `field`: if `is_primitive && field_name.matches(BUSINESS_REGEX)` then emit finding.
  - If `type_text` is `f32`/`f64` and field_name matches money keywords, emit `rust_domain_float_for_money` with recommendation to use `rust_decimal::Decimal` or an integer-cents newtype.
- **Recommended fix**: Replace primitive with a validated newtype: `struct Money(Decimal); impl TryFrom<i64> for Money { ... }` or `struct Username(String); impl TryFrom<String> for Username { validate non-empty }`.
- **Example**:
```rust
// BAD
pub struct Order { pub price: f64 }

// GOOD
pub struct Price(rust_decimal::Decimal);
impl Price { pub fn try_from_cents(cents: i64) -> Result<Self, Error> { ... } }
```
- **False-positive mitigation**: scope to public structs or to structs within application-level crates (configurable). Add allow-list comments like `// deslop-ignore:rust_domain_raw_primitive` when appropriate.

**2) Structs that allow impossible combinations (boolean flag + Option field)**
- **Rule id**: `rust_domain_impossible_combination`
- **What to detect**: A struct that contains a boolean toggle field (`ssl`, `tls`, `enabled`, `use_...`) and also an `Option` field representing a certificate/token/credential (fields named `cert`, `certificate`, `key`, `token`, `auth`) where presence/absence together can create impossible or ambiguous states.
- **Algorithm**:
  - For each `StructSummary` find boolean fields whose name matches `ENABLED_REGEX` and `Option` fields whose name matches `CRED_REGEX`.
  - If both exist in the same struct, flag it and recommend an enum-based design such as:
```rust
pub enum SslConfig { Disabled, EnabledWithCert(Cert) }
```
- **Severity**: Info/Warning depending on visibility (public types get higher severity).

**3) Auto-`#[derive(Default)]` producing unsafe defaults**
- **Rule id**: `rust_domain_default_produces_invalid`
- **What to detect**: Types that derive or implement `Default` and which produce likely-invalid defaults: `port` fields default to `0`, `token`/`key` fields default to empty string, boolean flags default to `false` where `false` may be unsafe.
- **Algorithm**:
  - Detect a `derive(Default)` attribute on a `struct_item` or `impl Default` blocks returning constants. If the struct has fields named `port`, `token`, `password`, `api_key` or similar, emit a finding.
  - If `impl Default` is custom code, try to detect explicit `0`/`String::new()` or `""` or `None` being returned for critical fields.
- **Fix**: Require either no `derive(Default)` or provide an explicit `new(...) -> Self` constructor and implement `TryFrom`/`TryInto` constructors for validated creation.

**4) `Debug` derive on types that include secrets**
- **Rule id**: `rust_debug_secret`
- **What to detect**: `#[derive(Debug)]` (or blanket `derive(Clone, Debug, ..)`) on structs where field names match `password|secret|api_key|token|access_token|private_key` OR heuristics detect secret-like values or annotations.
- **Algorithm**:
  - Use parsed `StructSummary`; if `derive(Debug)` exists on the type, and it contains secret fields (by name tokens), flag it.
  - Also look for `dbg!()`/`println!` usages with variables having secret-like names (there are already `non_test_macro_findings` for `dbg!` in `src/analysis/rust/mod.rs`; extend that to warn if the dbg target name is secret-like).
- **Recommendation**: Use the `secrecy` crate `Secret<T>` or implement a custom `Debug` that redacts secrets, or use attributes like `#[derive(Derivative)]` with field skip.
- **Example fix**:
```rust
#[derive(Clone)]
pub struct Creds { token: SecretString }
impl Debug for Creds { fn fmt(...) { write!(f, "Creds{ token: <redacted> }") } }
```

**5) `Serialize`/`Deserialize` on sensitive fields without validation**
- **Rule id**: `rust_serde_sensitive_deserialize`
- **What to detect**: `#[derive(Serialize, Deserialize)]` on structs with secret fields or on types where fields require validation (port ranges, non-empty tokens etc.) but no custom `Deserialize` implementation or no `validate()` function exists.
- **Algorithm**:
  - Detect derive(Serialize|Deserialize) presence on a struct with secret-like field names. Flag missing `#[serde(serialize_with = "...")]` or `#[serde(skip)]` or a `validate()` usage in source (heuristic search for `validate()` methods on the same `impl` block).
- **Recommendations**: Add `skip_serializing` for secrets in logs, `serialize_with` to mask values, and implement `TryFrom`/custom `Deserialize` where validation is required.

**Parser & data model changes (concrete edits)**
- **Files to change**:
  - **Add** `StructSummary` and `FieldSummary` in [src/analysis/types.rs](src/analysis/types.rs), e.g.:
```rust
pub struct FieldSummary { pub line: usize, pub name: String, pub type_text: String, pub is_pub: bool, pub is_option: bool, pub is_primitive: bool }
pub struct StructSummary { pub line: usize, pub name: String, pub fields: Vec<FieldSummary> }
```
  - **Update** `ParsedFile` to include `pub structs: Vec<StructSummary>`.
  - **Update** [src/analysis/rust/parser.rs](src/analysis/rust/parser.rs): extend `visit_for_symbols` (or add a dedicated `visit_structs`) to extract a struct's fields and their type textual representation using `node.child_by_field_name("name")` and the type child. Mark `is_option` by detecting `Option<` or `option` type nodes, and `is_primitive` by matching known primitive type identifiers.
  - **Add** new heuristics file `src/heuristics/rust_domain_modeling.rs` (or `src/heuristics/domain_modeling.rs`) implementing the rule functions listed above. Each function should accept `&ParsedFile` (and if needed `&ParsedFunction` or `&RepositoryIndex`) and return `Vec<Finding>`.
  - **Call** these functions from `evaluate_rust_findings` in [src/analysis/rust/mod.rs](src/analysis/rust/mod.rs) after index resolution and before returning findings.

**Tests & fixtures**
- **Fixtures**: Add representative Rust fixture files under `tests/fixtures/rust/domain_modeling/` covering:
  - primitive `pub struct Order { pub price: f64 }`
  - `pub struct Server { pub ssl: bool, pub cert: Option<PathBuf> }`
  - `#[derive(Default)] pub struct Config { pub port: u16, pub token: String }`
  - `#[derive(Debug, Serialize)] pub struct Creds { pub api_key: String }`
  - `#[derive(Serialize, Deserialize)] pub struct User { pub password: String }
`
- **Tests**: Extend `tests/integration_scan.rs` (or add new test file) that runs the scanner on these fixtures and asserts presence of rule ids above. Use the existing test harness patterns already in the repo.

**Prioritization & rollout plan**
- Phase 1 (parser + basic rules): implement struct extraction + `rust_domain_raw_primitive` and `rust_domain_impossible_combination`. (~1–2 dev days)
- Phase 2 (secrets & serde): implement `rust_debug_secret`, `rust_serde_sensitive_deserialize`, extend `dbg!` heuristics. (~1 day)
- Phase 3 (tuning & tests): expand fixtures, tune false positives, document exceptions and add config flags. (~1 day)

**Performance considerations**
- Only inspect structs in files that are part of the crate (skip vendored code).
- Limit expensive checks (regex matching large lists) to public/exported types or to types in `src/` (not `tests/`/`examples/`) by default.

**False positives & configuration**
- Provide an allowlist mechanism via code comment `// deslop-ignore:<rule_id>` and/or repo-level config to disable rules per-repo.
- Start rules at `Info` severity by default; escalate to `Warning` for public types.

**Acceptance criteria**
- Parser provides accurate `StructSummary` & `FieldSummary` for at least 90% of real-world Rust struct shapes (named structs + tuple structs optional).
- Each rule has unit/integration tests that assert both positive and negative cases.
- New rules are included in `evaluate_rust_findings` and appear in scan outputs for fixtures.

**PR checklist**
- [ ] Add `StructSummary` & `FieldSummary` types and update `ParsedFile`.
- [ ] Update `src/analysis/rust/parser.rs` to populate struct/field summaries.
- [ ] Add `src/heuristics/rust_domain_modeling.rs` with rule implementations.
- [ ] Integrate rules into `evaluate_rust_findings`.
- [ ] Add fixtures under `tests/fixtures/rust/domain_modeling/`.
- [ ] Add tests asserting findings in `tests/integration_scan.rs` or new test module.
- [ ] Update `README.md` or `guides/rust/` with a short note about the new rules.

**Next steps (proposal)**
- I can implement Phase 1 (parser + `rust_domain_raw_primitive` and `rust_domain_impossible_combination`) as a focused PR. Shall I proceed to implement and add tests now?

---

*Notes*: this plan assumes the Rust parser uses tree-sitter node kinds like `struct_item` and that you are willing to extend `ParsedFile` with struct-level metadata. If you prefer a lighter approach we can implement heuristics purely on textual parsing (regex) as a stop-gap, but extracting structured field/type info will yield far more accurate results and cleaner rules.

## Async & Concurrency Pitfalls (runtime behavior only)

### Purpose

Add a conservative, parser-driven rule pack that detects common async/runtime concurrency anti-patterns in Rust source code. These rules focus on runtime-behavior mistakes that are visible from syntax and small local flow patterns (cancellation at `.await`, holding blocking guards across `.await`, misuse of futures in `select!`, use of blocking primitives inside async code, lock ordering anomalies, etc.).

This plan documents detection heuristics, parser and type-model changes, heuristic implementation, tests/fixtures, rollout strategy, and acceptance criteria.

### Scope & constraints

- In-scope: runtime-behavior anti-patterns discoverable by tree-sitter parsing and reasonable local control-flow heuristics.
- Out-of-scope: fully type- or macro-expanded semantic analyses, borrow-checker-level reasoning, or dynamic instrumentation.

### Target rules (short summary)

1. `rust_async_spawn_cancel_at_await` — tasks spawned without cancellation protection (cancellable at `.await`).
2. `rust_async_recreate_future_in_select` — futures recreated inside a `select!` loop.
3. `rust_async_missing_fuse_pin` — reused futures not fused/pinned across polls.
4. `rust_async_hold_permit_across_await` — RAII permit/guard held across `.await`.
5. `rust_async_monopolize_executor` — long-running/CPU-bound async tasks with insufficient awaits.
6. `rust_async_blocking_drop` — `Drop` impls doing blocking operations used in async contexts.
7. `rust_async_invariant_broken_at_await` — invariants split across `.await` points.
8. `rust_async_std_mutex_await` — `std::sync::Mutex` used across `.await` (deadlock risk).
9. `rust_async_lock_order_cycle` — inconsistent lock acquisition ordering leading to cycles.

For each rule below we describe detection heuristics, evidence, false-positive mitigations, remediation hints, tests, and estimated severity.

---

### 1) `rust_async_spawn_cancel_at_await` — spawned tasks without cancellation handling

- Detection heuristic:
   - Identify spawn-like calls: `spawn`, `tokio::spawn`, `async_std::task::spawn`, or `spawn_local` via import resolution (check `imports` for common async crates).
   - For the spawned async body (an `async move` block or `async fn` passed to spawn), collect `.await` points and resource acquisitions: recorded `lock`/`acquire` calls or counters (pattern-match method names like `lock`, `acquire`, `acquire_owned`, `open`, `insert` on counters/global structures).
   - If resource acquisition occurs before an `.await` and no cancellation/cooperative-shutdown pattern is detected (no `tokio::select!` checking a cancellation channel, no `futures::select!` with a cancellation branch, no `CancellationToken` use), emit a finding.

- Evidence recorded: spawn site line, resource acquisition line(s), `.await` line(s), absent cancellation pattern marker.

- Severity: Warning

- Mitigation: Suggest `tokio::select!` with a cancellation channel, use `CancellationToken`, or restructure to keep critical sections short and use RAII correctly.

- Tests: fixtures showing a `tokio::spawn` with `let guard = sem.acquire().await;` and another `await` while `guard` still in scope (positive), and a safe example using `select!` or acquiring the guard after awaits (negative).

---

### 2) `rust_async_recreate_future_in_select` — creating futures inside a `select!` loop

- Detection heuristic:
   - Find macro invocation nodes for `select!` inside loop contexts.
   - For each `select!` arm, detect expressions that create futures on each iteration (calls to non-`async`-returning constructors that produce `impl Future`, inline `async {}` blocks, or function calls to `async fn` with immediate use instead of pre-creating/pinning a future).
   - If futures are created each iteration and not pinned/fused, emit Info/Performance finding.

- Evidence: loop line, `select!` line, arm expression lines.

- Severity: Info (performance)

- Mitigation: Move future creation outside the loop and use `.fuse()` + `pin_mut!()` or restructure logic.

---

### 3) `rust_async_missing_fuse_pin` — reused futures without `.fuse()`/`pin_mut!`

- Detection heuristic:
   - Identify futures reused across poll sites (declared before loop/use site) and used with `select!` or multi-poll patterns.
   - If `.fuse()` and `pin_mut!()` calls are not present on the future variable, emit a finding.

- Evidence: future declaration line and usage lines.

- Severity: Warning

- Mitigation: `use futures::FutureExt; let mut fut = foo().fuse(); pin_mut!(fut);` and then use in `select!`.

---

### 4) `rust_async_hold_permit_across_await` — permit or RAII guard held across `.await`

- Detection heuristic:
   - Recognize semaphore/guard acquisition calls (`Semaphore::acquire`, `Semaphore::acquire_owned`, `permit`, `pool.get()` patterns) and record the guard variable name and line.
   - Search for subsequent `.await` within the lexical scope before an explicit `drop(guard)` or guard goes out-of-scope.
   - If guard is live across `.await`, emit Warning.

- Evidence: acquire line, guard variable name, await line.

- Severity: Warning

- Mitigation: drop guard before `.await`, acquire after `.await`, or redesign to avoid long-held guards.

---

### 5) `rust_async_monopolize_executor` — long-running tasks starving the executor

- Detection heuristic:
   - Detect async functions that contain loops with zero or very few `.await` points and call patterns that look CPU-bound or blocking (calls to functions recognized as blocking by `is_blocking_call` helper, `std::thread::sleep`, heavy compute loops with no `await`).
   - If such a task is spawned on the executor (e.g., `tokio::spawn`) and not offloaded to `spawn_blocking`, emit a finding.

- Evidence: function/loop lines, blocking-call lines.

- Severity: Warning / Performance

- Mitigation: offload to `spawn_blocking`, insert cooperative `yield_now().await`/await points, or restructure into smaller async tasks.

---

### 6) `rust_async_blocking_drop` — blocking operations inside `Drop`

- Detection heuristic:
   - Collect `impl Drop for` blocks and search for calls to blocking operations inside `drop` (e.g., `.join()` on threads, blocking locks on `std::sync::Mutex`, file I/O, `block_on`).
   - If the type is used within async contexts (detected by presence in functions with `.await`), emit a finding.

- Evidence: type name, `drop` impl line(s), blocking-call lines.

- Severity: Warning / Error (if high confidence)

- Mitigation: avoid blocking in `Drop` — provide explicit async cleanup (e.g., `async fn shutdown(self)`) or ensure drop occurs in blocking context.

---

### 7) `rust_async_invariant_broken_at_await` — invariants split by `.await`

- Detection heuristic:
   - Detect sequences where multiple related writes/updates happen to shared state with an `.await` between them, and where later code assumes a combined invariant without locking.
   - Heuristic signals: write to two or more fields or call `insert`/`remove` on map then `.await`, then read both fields expecting consistency.

- Evidence: pre-await mutation lines, await line, post-await assertion/use lines.

- Severity: Warning

- Mitigation: protect sequences with a lock or redesign to keep invariant-maintaining code contiguous without `.await`.

---

### 8) `rust_async_std_mutex_await` — `std::sync::Mutex` held across `.await`

- Detection heuristic:
   - Detect `.lock()` call expressions where the receiver type resolves to `std::sync::Mutex` (match import aliases or fully-qualified `std::sync::Mutex`).
   - If `.await` exists while the guard variable is still in scope, flag immediately as Error.

- Evidence: lock acquisition line, guard variable name, `.await` line.

- Severity: Error

- Mitigation: use `tokio::sync::Mutex` in async code or ensure guard is dropped before `.await`.

---

### 9) `rust_async_lock_order_cycle` — lock ordering cycles across functions

- Detection heuristic:
   - Build per-function lock acquisition sequences (ordered lists of lock identifiers derived from receiver names or simple alias resolution).
   - Build a global partial order graph where an edge `A -> B` indicates `A` acquired then `B` acquired while `A` held.
   - Detect cycles in the graph and emit an Error with cycle details.

- Evidence: acquisition lines for each edge, cycle trace.

- Severity: Error

- Mitigation: establish a consistent global lock ordering, reduce nested locks, or use non-blocking concurrency primitives.

---

## Parser & analysis changes (implementation details)

1. Extend `ParsedFunction` in `src/analysis/types.rs` with Rust-async-specific fields (default empty for other languages):
    - `await_points: Vec<usize>`
    - `macro_calls: Vec<(String, usize)>` (names like `select!`)
    - `spawn_calls: Vec<CallSite>`
    - `lock_calls: Vec<CallSite>` (captures `.lock()` and `.acquire()` sites)
    - `permit_acquires: Vec<CallSite>` (semaphore/permit acquisition calls)
    - `futures_created: Vec<usize>` (lines where `async {}` or `async fn()` future is created)
    - Optionally: `drop_impls: Vec<DropInfo>` at `ParsedFile` level.

2. Initialize these fields in all language parsers when constructing `ParsedFunction` so non-Rust parsers remain unaffected (empty vectors).

3. Implement Rust-specific collectors in `src/analysis/rust/parser.rs`:
    - Traverse tree-sitter nodes to detect `await` expressions (node kind `await_expression` or pattern matching `.await` field expressions) and record line numbers.
    - Detect macro invocations named `select!`, `futures::select!`, `tokio::select!` and record macro name + line.
    - Detect method calls with names `lock`, `acquire`, `acquire_owned`, `try_acquire`, and record `CallSite` with receiver + name.
    - Detect spawn-like calls and record them as `spawn_calls`.
    - Detect `impl Drop for` nodes and inspect body for blocking calls (calls to `join`, `block_on`, `std::thread::sleep`, `std::sync::Mutex::lock`) and record `DropInfo` for heuristics.

4. Add small helpers in `src/heuristics/common.rs` (or reuse existing `is_blocking_call`) to identify likely-blocking calls for Rust-specific names and to normalize import alias lookup.

## Heuristics implementation

1. Create `src/heuristics/rust_async.rs` with `pub(super) fn rust_async_findings(file: &ParsedFile, function: &ParsedFunction, imports: &[ImportSpec]) -> Vec<Finding>`.
2. Implement one helper per rule that analyzes `ParsedFunction` fields and emits `Finding` entries consistent with repo reporting (`rule_id`, `severity`, `message`, `evidence`).
3. Wire the function into `src/heuristics/mod.rs` so Rust files are evaluated by the async ruleset when the file `language` is `Language::Rust`.

## Tests & fixtures

- Add `tests/fixtures/rust/async/` with per-rule positive and negative examples (one file per rule with `_positive.rs` and `_negative.rs` suffixes).
- Add integration test `tests/integration_scan/rust_async.rs` that runs the analyzer on the fixtures and asserts expected rule ids and evidence.

Example fixture (bad) for `rust_async_std_mutex_await`:

```rust
use std::sync::Mutex;
static M: Mutex<i32> = Mutex::new(0);

async fn do_work() {
      let g = M.lock().unwrap();
      async_std::task::sleep(std::time::Duration::from_millis(10)).await; // .await while std::sync::Mutex is held
      drop(g);
}
```

Example fixture (good):

```rust
use tokio::sync::Mutex;
static M: tokio::sync::Mutex<i32> = tokio::sync::Mutex::const_new(0);

async fn do_work() {
      let mut guard = M.lock().await;
      // critical section without awaiting
      let val = *guard;
      drop(guard);
      some_other_future().await;
}
```

## Documentation & developer guidance

- Add a short section to `guides/rust/heuristics-and-findings.md` linking to this new async plan and summarizing each rule with quick remediations.
- In each rule's `Finding` evidence include short remediation text and a link to the guide if appropriate.

## Rollout & verification

1. Implement parser changes with defaults and run full test suite to ensure no regressions.
2. Add heuristics behind a feature flag (e.g., `rust_async_experimental`) to allow staged rollout and internal validation.
3. Add fixtures and integration tests; run on representative Rust repositories and measure noise.
4. Ramp severities: start `Info` or `Warning` for most rules; escalate `std::sync::Mutex` across `.await` and lock-order cycles to `Error` once tuned.

## Acceptance criteria

- Parser collects awaited points, spawn/macro calls, and lock/permit acquisitions for Rust functions.
- Each rule has positive and negative fixtures and passing integration tests.
- Findings include clear `rule_id`, `severity`, `message`, and `evidence` describing the risky pattern and fix.

## Estimated effort

- Parser field additions & extraction: 1–2 days
- Heuristic scaffolding + 3–4 initial rules: 2–4 days
- Fixtures & tests: 1–2 days
- Validation & performance checks: 1–2 days

Total (initial conservative pack): ~1–2 weeks.

## Next steps (immediate)

1. Add the new `ParsedFunction` fields and initialize them in all language parsers (mechanical change).
2. Implement `await` and macro collectors in `src/analysis/rust/parser.rs` and unit-test extraction.
3. Add `src/heuristics/rust_async.rs` skeleton and wire it into `src/heuristics/mod.rs`.
4. Add 2 high-value fixtures (e.g., `std::sync::Mutex` across `.await`, and RAII permit held across `.await`) and integration tests.

If you'd like, I can implement the parser changes and one high-confidence rule (e.g., `rust_async_std_mutex_await`) and add fixtures/tests now — shall I proceed?

