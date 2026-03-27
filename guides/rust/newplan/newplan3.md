# Error Handling & Rust Heuristics — Checklist Plan

This file is the checklist-focused conversion of the detailed plan. Use the checkboxes to track progress.

## Summary

- [x] Adopt typed library errors (`thiserror`) and keep `anyhow` at binary edges.
- [x] Eliminate the targeted non-test `panic!` / `.unwrap()` library cases and add bounded IO helpers.
- [x] Add CI checks to prevent regressions (unwrap/panic/read_to_string).

## Implementation Status (2026-03-28)

- [x] `src/error.rs` and `src/io.rs` landed, and the scanner now uses bounded reads.
- [x] Rust parser/type-model changes landed: `StructSummary`, `FieldSummary`, async metadata, macro/runtime call summaries, and unsafe-pattern summaries.
- [x] Domain-modeling heuristics landed with fixture-backed tests.
- [x] Async/concurrency heuristics landed with fixture-backed tests.
- [x] Strict verification completed with `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and the repo hygiene script.
- [x] Repo-level suppression comments landed via `deslop-ignore:<rule_id>` in the scan pipeline.
- [x] Feature-flagged rollout for async rules landed via `.deslop.toml` with `rust_async_experimental = false`.
- [x] Repo-level severity/config toggles landed via `.deslop.toml` using `disabled_rules` and `severity_overrides`.

## Goals (high level)

- [x] Library APIs return concrete `Error` types (no opaque `Box<dyn Error>` in public API).
- [x] Binaries use `anyhow::Result` and attach context for user-facing messages.
- [x] No `panic!`/`.unwrap()` in non-test library code.
- [x] Prevent unbounded reads (use size limits / streaming readers).
- [x] CI enforces these patterns.

## Prep: Dependencies & small changes

- [x] Add `thiserror = "1.0"` to `Cargo.toml` where library code lives.
- [x] Ensure `anyhow` remains available for binaries (optional in libraries).

## Core implementation (priority tasks)

- [x] Create `src/error.rs` with a top-level `pub enum Error` and `pub type Result<T> = Result<T, Error>`.
   - [x] Include `#[from]` conversions for common module errors (I/O, parser/config fan-in).
   - [x] Export `pub use crate::error::{Error, Result};` from `lib.rs`.
- [x] Add per-module `Error` enums (e.g., `analysis::parser::Error`) and wire them into the top-level `Error` via `#[from]`.

## Replace application vs library error types

- [x] Grep and replace `use anyhow::Result` in library crates with `use crate::error::Result`.
- [x] Keep `main.rs` / CLI return types as `anyhow::Result` and map library errors to `anyhow::Error` with context.

## Remove panics/unwraps and add typed errors

- [x] Replace trivial `unwrap()` occurrences with `ok_or(Error::...) ?` or `?` propagation.
   - [x] Example: replace `target_segments.last().unwrap()` with a non-panicking path.
- [x] Replace `panic!(...)` in library code with domain `Error` variants.

## Bounded IO / streaming helpers

- [x] Add `src/io.rs` with `read_to_string_limited(path: &Path, max_bytes: u64) -> Result<String, Error>`.
- [x] Replace `fs::read_to_string(path)` usages with the limited reader or streaming `Read::take(max)`.

## CI / Lint enforcement

- [x] Add CI step(s) to detect new `panic!`/`.unwrap()`/`.expect()` in non-test code.
   - [x] Option A: enforce `clippy` lints (e.g., `-D clippy::unwrap_used -D clippy::expect_used`).
   - [x] Option B: add a small grep-based check that excludes `tests/` and `#[cfg(test)]` blocks.
- [ ] Optionally add a test that fails on `fs::read_to_string` occurrences.

## Migration sequencing (recommended phases)

- [x] Phase A — Low risk (1–2 days)
   - [x] Add `thiserror` and `src/error.rs`.
   - [x] Implement `read_to_string_limited` and migrate simple reads (e.g., `src/scan/mod.rs`).
   - [x] Fix local `unwrap()`/`panic!` instances with clear error cases.
- [x] Phase B — Medium risk (2–4 days)
   - [x] Add per-module errors and convert public APIs to `crate::error::Result`.
   - [x] Update binaries to map errors into `anyhow` with context.
- [x] Phase C — Finish (1–2 days)
   - [x] Run full build and tests; fix regressions.
   - [x] Add CI linting; finalize error messages and docs.

## Developer ergonomics & utilities

- [x] Provide `pub type Result<T> = std::result::Result<T, Error>` centrally.
- [x] Add small adapter helpers for error conversion/context where convenient.
- [x] Document the library vs app boundary in `README`/guides.

## Tests & benchmarks

- [x] Add unit tests for error paths (missing files, oversized inputs, parse errors).
- [ ] Add a small benchmark that validates `read_to_string_limited` memory behavior.

## Domain Modeling & Invariants — checklist (parser + heuristics)

- [x] Add `StructSummary` and `FieldSummary` types to `src/analysis/types.rs`.
   - [x] `FieldSummary { line, name, type_text, is_pub, is_option, is_primitive }`.
   - [x] `StructSummary { line, name, fields: Vec<FieldSummary> }`.
- [x] Extend `ParsedFile` to include `structs: Vec<StructSummary>`.
- [x] Update `src/analysis/rust/parser.rs` to extract struct fields and mark `is_option` / `is_primitive`.
- [x] Add `src/heuristics/rust/domain_modeling.rs` with rule implementations and wire into `evaluate_rust_findings`.

### Domain rules to add (each as an implementation task)

- [x] `rust_domain_raw_primitive` — detect business-value fields using raw primitives (money, price, username, etc.) and recommend newtypes.
- [x] `rust_domain_impossible_combination` — detect boolean + `Option` credential combos and recommend enum-based designs.
- [x] `rust_domain_default_produces_invalid` — flag `#[derive(Default)]` producing unsafe defaults for sensitive fields.
- [x] `rust_debug_secret` — detect `Debug` on secret-bearing types and recommend redaction/secrecy crate.
- [x] `rust_serde_sensitive_deserialize` — detect `Serialize`/`Deserialize` on sensitive fields without validation or masking.

## Async & Concurrency heuristics — checklist (parser + rules)

- [x] Extend `ParsedFunction` with async fields: `await_points`, `macro_calls`, `spawn_calls`, `lock_calls`, `permit_acquires`, `futures_created`.
- [x] Implement Rust collectors in `src/analysis/rust/parser.rs` to populate those fields.
- [x] Add Rust async heuristics and wire them into the heuristics pipeline.

### High-value async rules (implement and test each)

- [x] `rust_async_std_mutex_await` — detect `std::sync::Mutex` held across `.await` (Error).
- [x] `rust_async_hold_permit_across_await` — detect RAII permits held across `.await` (Warning).
- [x] `rust_async_spawn_cancel_at_await` — detect spawned tasks lacking cancellation handling (Warning).
- [x] `rust_async_missing_fuse_pin` & `rust_async_recreate_future_in_select` — detect select/future misuse (Info/Warning).
- [x] `rust_async_lock_order_cycle` — build lock-order graph and detect cycles (Error).

## Tests, fixtures, and rollout for heuristics

- [x] Add fixtures under `tests/fixtures/rust/domain_modeling/` and `tests/fixtures/rust/async/`.
- [x] Add integration tests in `tests/integration_scan.rs` (or new modules) asserting findings for positive and negative cases.
- [x] Stage rollout behind a feature flag (e.g., `rust_async_experimental`) for initial tuning.

## PR checklist (one combined list)

- [x] Add `thiserror` to `Cargo.toml`.
- [x] Add `src/error.rs` and export `Error`/`Result`.
- [x] Implement `read_to_string_limited` and migrate simple reads.
- [x] Replace trivial `unwrap()` / `panic!` cases with typed errors.
- [x] Add per-module `Error` enums and `#[from]` conversions.
- [x] Update binaries to use `anyhow` for user-facing messages.
- [x] Add CI linting and grep-based checks for regressions.
- [x] Add `StructSummary`/`FieldSummary` and parser extracts.
- [x] Implement `rust_domain_*` heuristics and tests.
- [x] Implement `rust_async_*` heuristics and tests (start with `std::sync::Mutex` across `.await`).
- [x] Add fixtures and integration tests; run and fix failures.
- [x] Update `README.md` with summary and developer guidance.

## Next actions (pick one)

- [x] Implemented `src/error.rs` and added `thiserror` to `Cargo.toml`.
- [x] Converted `src/scan/mod.rs` to use `read_to_string_limited` and fixed obvious `unwrap()`/`panic!` usages.
- [x] Implemented the domain-modeling parser changes (added `StructSummary`, extracted fields, and added tests).

---

If you pick one of the next actions above, I will start and update this checklist as I make changes.

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
   Status: comment-based suppression and repo-level config are both implemented.
- Start rules at `Info` severity by default; escalate to `Warning` for public types.

**Acceptance criteria**
- Parser provides accurate `StructSummary` & `FieldSummary` for at least 90% of real-world Rust struct shapes (named structs + tuple structs optional).
- Each rule has unit/integration tests that assert both positive and negative cases.
- New rules are included in `evaluate_rust_findings` and appear in scan outputs for fixtures.

**PR checklist**
- [x] Add `StructSummary` & `FieldSummary` types and update `ParsedFile`.
- [x] Update `src/analysis/rust/parser.rs` to populate struct/field summaries.
- [x] Add `src/heuristics/rust/domain_modeling.rs` with rule implementations.
- [x] Integrate rules into `evaluate_rust_findings`.
- [x] Add fixtures under `tests/fixtures/rust/domain_modeling/`.
- [x] Add tests asserting findings in `tests/integration_scan.rs` or new test module.
- [x] Update `README.md` with a short note about the new rules.

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

