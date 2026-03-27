---
# Rust Performance & Unsafe-Soundness — Checklist Plan

Last updated: 2026-03-27

Purpose
- Convert the implementation plan into an actionable, checklist-style Markdown plan. Each checklist item is a concrete task or deliverable that can be tracked and executed.

Scope
- Heuristics and parser changes in `src/analysis/rust` and `src/heuristics/rust`; tests and CI updates under `tests/` and CI configuration files.

How to use this file
- Check items as you complete them. Each checked item should correspond to a small, reviewable PR and/or a test run.

## Implementation Status (2026-03-28)

- [x] Added `src/heuristics/rust/` with `performance.rs`, `async_patterns.rs`, `domain_modeling.rs`, `unsafe_soundness.rs`, and `mod.rs`.
- [x] Extended `ParsedFile` / `ParsedFunction` and `src/analysis/rust/parser.rs` with Rust-specific async/runtime, struct, and unsafe metadata.
- [x] Implemented the planned Rust performance rules: blocking I/O in async, unbuffered writes, `.lines()` hot-path allocation, default hasher use, lock-across-await, tokio mutex guidance, blocking drop, pointer-chasing containers, AoS hot-path, large-future, UTF-8 hot-path, and absolute-path join checks.
- [x] Implemented unsafe soundness detectors for `get_unchecked`, raw-parts creation, `set_len`, `assume_init`, `transmute`, and raw-pointer casts.
- [x] Added positive and negative Rust fixtures plus integration coverage in `tests/integration_scan/rust_advanced.rs`.
- [x] Added CI/workflow support with `.github/workflows/ci.yml`, `.github/workflows/rust-security.yml`, `scripts/check_rust_hygiene.sh`, and `scripts/check-rust-security.sh`.
- [x] Verified with `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` on 2026-03-28.
- [x] Audited the ownership/API branch: no live `&String` or `&Vec<_>` production API signatures were found in `src/` during implementation; enforcement/reporting was added instead of forced API churn.
- [x] Added repo-level `deslop-ignore:<rule_id>` suppression support in the scan pipeline for same-line and next-code-line directives, with Rust integration coverage.
- [ ] `cargo miri`/strict `cargo geiger` gating remains report-only and is not enforced as a blocking CI step.

---

**Phase: Setup & Scaffolding**

- [x] Create `src/heuristics/rust/` and scaffold `performance.rs` and `mod.rs`.
  - [x] Add rule functions for the initial rules.
  - [x] Add the module structure used by the Rust rule packs.
- [x] Add minimal parser enhancements in `src/analysis/rust/parser.rs`:
  - [x] `is_async` flag on `ParsedFunction`.
  - [x] `await_lines: Vec<usize>` and simple `call_site` metadata.
  - [x] `boxed_container_usages` and related hot-path placeholders.
- [x] Wire new Rust heuristics into `src/heuristics/mod.rs` and the evaluation flow.

**Phase: Phase A — High Priority Rules (deliver fast wins)**

- [x] `rust.blocking_io_in_async` (Warning)
  - [ ] Detect `std::fs::` calls inside `async fn` or functions containing `.await`.
  - [ ] Add tests/fixtures under `tests/fixtures/rust/blocking_io_in_async/` (positive & negative).
  - [ ] Add integration test to confirm rule triggers.
- [x] `rust.unbuffered_file_writes` (Info)
  - [ ] Detect `writeln!`/`write!` or `File::write_*` called on `File` within loops.
  - [ ] Suggest `BufWriter` in message.
  - [ ] Add tests/fixtures.
- [x] `rust.lines_allocate_per_line` (Info)
  - [ ] Detect `.lines()` use on `BufRead` in hot contexts; add mitigation example using `read_line` + `clear()`.
  - [ ] Add tests/fixtures.
- [x] `rust.hashmap_default_hasher` (Info)
  - [ ] Detect `HashMap::new()` in hot functions/loops and missing custom hasher.
  - [ ] Add guidance in evidence suggesting `FxHashMap`/`hashbrown`/`BTreeMap`.

**Phase: Phase B — Medium Priority Rules**

- [x] `rust.lock_across_await` (Warning)
  - [ ] Track lock calls, guard variables and later `.await` occurrences in same function scope.
  - [ ] Add tests showing common false-positive patterns and an explicit `drop(guard)` example.
- [x] `rust.tokio_mutex_unnecessary` (Info)
  - [ ] Detect `tokio::sync::Mutex` usage without `.await` inside critical section and recommend `std::sync::Mutex`.
  - [ ] Add tests/fixtures.
- [x] `rust.blocking_drop` (Warning)
  - [ ] Inspect `impl Drop for` bodies for blocking APIs and add rule tests.
- [x] `rust.pointer_chasing_vec_box` (Info)
  - [ ] Detect `Vec<Box<T>>` or linked-list patterns; add template mitigation suggestions.

**Phase: Phase C — Research / Complex Rules**

- [x] `rust.aos_hot_path` (Info/Warning)
  - [ ] Detect loops over `Vec<Struct>` where 2+ fields are accessed repeatedly.
  - [ ] Create `StructFieldLoop` parser summary and run against fixtures; consider profiler-guided validation before promoting severity.
- [x] `rust.large_future_stack` (Info)
  - [ ] Heuristic detection of large local bindings captured by `async fn`/blocks; add guidance to box futures or refactor.
- [x] `rust.utf8_validate_hot_path` (Info)
  - [ ] Flag `from_utf8` in hot paths; provide conservative guidance about `from_utf8_unchecked`.
- [x] `rust.path_join_absolute` (Warning)
  - [ ] Detect `path.join("/abs/...")` string literal cases and add tests.

**Unsafe Soundness Rule Pack (separate module)**

- [x] Create the Rust unsafe soundness rule pack and register it in the analyzer.
  - [ ] `get_unchecked` / bounds-less indexing detector.
    - [ ] Flag `get_unchecked`, `get_unchecked_mut`, `Vec::set_len`, `from_raw_parts`, etc. inside `unsafe` blocks.
    - [ ] Require guard detection or `SAFETY:` justification to suppress.
  - [ ] `transmute` & raw pointer cast detector.
    - [ ] Flag `mem::transmute`, `as *mut`, `as *const`, `MaybeUninit::assume_init` unless `SAFETY:` present.
  - [ ] Generic-invariant & aliasing detectors.
    - [ ] Identify unsafe blocks that assume properties of unconstrained `T`.
  - [ ] Public `unsafe` constructor / raw-parts API checks.
  - [ ] Send/Sync & thread-safety checks when raw pointers/transmutes cross threads.
  - [ ] Add fixtures and integration tests under `tests/fixtures/rust/unsafe_soundness/`.

**Tests, Fixtures & Examples**

- [x] Add fixtures under `tests/fixtures/rust/` grouped by rule id.
- [x] Add unit and integration tests validating true positives and representative negatives.
- [x] Update integration coverage in `tests/integration_scan.rs` / `tests/integration_scan/rust_advanced.rs` to assert expected `rule_id`s.
- [ ] Add short example snippets and remediation examples to `guides/rust/` and link them from rule findings.

**CI & Automation**

- [x] Add CI diagnostics covering strict `cargo clippy`, test execution, and grep-based security reporting.
- [ ] Add `rust-unsafe-soundness` job to run new detectors and small `cargo miri` test subset (opt-in due to slowness).

**Implementation Checklist (PR-sized steps)**

1. [ ] Scaffold heuristics module and add a minimal `performance.rs` with empty rule stubs.
2. [ ] Add parser flags (`is_async`, `await_lines`) and small summaries to `ParsedFunction`.
3. [ ] Implement the Phase A rules and their tests (one rule per PR recommended).
4. [ ] Implement Phase B rules; iterate on false positives with test cases.
5. [ ] Implement `unsafe_soundness` detectors and tests.
6. [ ] Wire rules into `src/heuristics/mod.rs` and add integration tests.
7. [ ] Add CI jobs and documentation; run the rules against a few representative Rust projects to tune.

**Acceptance Criteria (must be met before merge)**

- [x] Each rule exposes a stable `rule_id`, severity, and clear evidence message.
- [x] Tests/fixtures exist for both true positives and representative false-positive controls for each rule.
- [x] Integration tests assert the rule triggers on fixtures.
- [ ] Parser changes are minimal and documented in code comments.
- [x] CI includes diagnostics that run the heuristic test suite and report results.

**Quick Start / First PRs (recommended order)**

- [x] PR 1: Scaffold `src/heuristics/rust/performance.rs` and wire into `src/heuristics/mod.rs`.
- [x] PR 2: Parser changes — add `is_async` / `await_lines` and minor `ParsedFunction` fields.
- [x] PR 3: Implement `rust.blocking_io_in_async` + tests/fixtures.

**Notes / Risks**

- False positives are expected during early iterations — tune thresholds and add negative fixtures.
- Some detections (AoS hot-paths, large-future detection) are research-heavy; prefer conservative messaging.

---

If you'd like, I can now:

- [x] scaffolded the heuristics module and completed the initial Rust rule pack implementation,
- [x] implemented the parser change for `is_async`/`await` detection, and
- [x] implemented the first Phase A rule and its tests (blocking I/O in async).

Pick one and I will proceed.

---

## Appendix: Useful commands

```bash
rg "&'static" --glob "*.rs"
rg "struct .*<'[a-z]" --glob "*.rs"
rg "&String" --glob "*.rs"
rg "&\s*Vec<" --glob "*.rs"
rg "Rc<RefCell|Arc<Mutex" --glob "*.rs"
rg "\.clone\(\)" --glob "*.rs"
```

---

End of checklist plan.
  - Locks across `.await` and tokio vs std mutex guidance (`rust.lock_across_await`, `rust.tokio_mutex_unnecessary`) — 2-4 days
  - `Drop` doing blocking cleanup (`rust.blocking_drop`) — 1-2 days
  - `Vec<Box<T>>` pointer-chasing (`rust.pointer_chasing_vec_box`) — 1-2 days
- Phase C (lower-priority / research-intensive, 5-10 days):
  - AoS vs SoA hot-path detection (`rust.aos_hot_path`) — 3-5 days (requires profiler + careful heuristics)
  - Large futures on stack (`rust.large_future_stack`) — 2-4 days
  - UTF-8 validation guidance (`rust.utf8_validate_hot_path`) — 1 day
  - Path::join absolute path detection (`rust.path_join_absolute`) — 1 day

Implementation checklist (concrete steps)
1. Add new `src/heuristics/rust/` module and scaffold `performance.rs` with empty rule functions.
2. Add minimal parser enhancements in `src/analysis/rust/parser.rs` to surface `is_async` and `await` lines plus the few targeted collection fields needed.
3. Implement rules one-by-one in `src/heuristics/rust/performance.rs`, starting with the simple ones (blocking I/O in async, unbuffered writes, `.lines()` allocations).
4. Add fixtures in `tests/fixtures/rust/` and unit/integration tests in `tests/`.
5. Update `src/heuristics/mod.rs` to wire Rust rules into evaluation flow.
6. Run test suite and iterate on false positives.
7. Optionally: add a `profiling/` example and docs describing how to verify hot-paths with `pprof`, `flamegraph`, `perf`, or `tokio-console` to complement static heuristics.

Examples & test snippets (short)
```rust
// AoS example (positive)
struct Point { x: f64, y: f64 }
fn process(points: &Vec<Point>) {
    for p in points {
        let a = p.x;
        let b = p.y;
        // repeated multi-field access -> candidate for SoA
    }
}

// Unbuffered write example (positive)
use std::fs::File;
fn save(items: &[u8]) {
    let mut f = File::create("out.bin").unwrap();
    for b in items {
        writeln!(f, "{}", b).unwrap(); // flagged: recommend BufWriter
    }
}

// Blocking IO in async (positive)
async fn handler() {
    let s = std::fs::read_to_string("/tmp/a").unwrap(); // flagged
}
```

Risks and mitigation
- False positives: many heuristics are pattern-based; tune thresholds and restrict to functions with high complexity or loops to reduce noise.
- Parser limitations: tree-sitter gives AST nodes; some semantic info (e.g., type resolution) may require index lookup. Use import resolution in `RepositoryIndex` where available.
- Safety trade-offs (e.g., recommending `from_utf8_unchecked`) require conservative messages and explicit warnings.

Next steps (recommended immediate tasks)
1. Approve the plan or request changes to scope/priorities.
2. Create the heuristics scaffolding and implement 2-3 high-value, low-risk rules (blocking IO in async, unbuffered writes, `.lines()` allocations) as a first PR.
3. Run the rules against a few medium-sized Rust projects to tune false positives.

Appendix: Where to implement in repo
- Parser: `src/analysis/rust/parser.rs` and types in `src/analysis/types.rs`.
- New heuristics module: `src/heuristics/rust/performance.rs` (create) and `src/heuristics/rust/mod.rs` (if needed).
- Wire into heuristics entry points: update `src/heuristics/mod.rs` to call rust evaluators for Rust files.
- Tests & fixtures: `tests/fixtures/rust/` and `tests/` for integration tests.

Contact / owner
- Proposed owner: whoever maintains `src/analysis/rust/*` and `src/heuristics/*` (assign in PR).  

---

If you want, I can: (a) scaffold `src/heuristics/rust/performance.rs` and update `src/heuristics/mod.rs`, (b) make the minimal parser changes for `is_async` / `await` detection, or (c) implement the first three high-priority rules and tests now. Which should I do next?
# Rust: Ownership, Lifetime & API Design — Action Plan

Date: 2026-03-27

Purpose: audit the workspace for common ownership/lifetime API anti-patterns and provide a prioritized, actionable remediation and rollout plan. The checks below were run against `**/*.rs` files in the repository; results and recommendations follow.

---

## Quick scan summary

- Patterns searched (grep/snippets used):
  - `&'static`
  - `&'a`
  - `&String`, `&Vec<...>`
  - `Rc<RefCell`, `Arc<Mutex`
  - `Cow<`
  - `-> impl ` (returning `impl Trait`)
  - `fn ... self(...) -> Self` (builder-consume pattern)
  - `impl From<` / `impl TryFrom<`
  - `\.clone\(\)` (explicit clones)

- Notable findings (representative examples):
  - `&'static` is used intentionally in a few places (lists of supported extensions, backend registration):
    - [src/analysis/python/mod.rs](src/analysis/python/mod.rs#L20)
    - [src/analysis/mod.rs](src/analysis/mod.rs#L30)
    - [src/analysis/mod.rs](src/analysis/mod.rs#L45)
    - [src/analysis/mod.rs](src/analysis/mod.rs#L53)
    - [src/analysis/rust/parser.rs](src/analysis/rust/parser.rs#L401)

  - Short-lived `<'a>` borrows appear in summary/view types (these are used for zero-copy summaries):
    - [src/cli/report.rs](src/cli/report.rs#L172)
    - [src/cli/report.rs](src/cli/report.rs#L201)
    - [src/index/mod.rs](src/index/mod.rs#L27)

  - `impl Trait` return detected (iterator returning references):
    - [src/heuristics/python/duplication.rs](src/heuristics/python/duplication.rs#L401)

  - Clones appear in multiple places (some are in tests or one-off copies, some are likely production copies):
    - [src/scan/mod.rs](src/scan/mod.rs#L57)
    - [src/analysis/python/parser/general.rs](src/analysis/python/parser/general.rs#L88)
    - [src/index/mod.rs](src/index/mod.rs#L227)

  - Absent or rare/zero-usage indicators:
    - No obvious usages of `&String` found (good).
    - No usages of `Rc<RefCell<...>>` or `Arc<Mutex<...>>` were detected in a project-wide scan.
    - No `Cow<...>` usages were found (so opportunity for Cow in hot-paths exists).
    - `AsRef<str>` patterns were not detected in a quick scan (opportunity to use generics/APIs).

Notes: the scan was a surface pass (regex-based). Some patterns are legitimate (e.g., global static lists and deliberate short-lived borrow summaries). The plan below prescribes how to confirm intent and incrementally harden APIs.

---

## Objectives

1. Eliminate accidental long-lived borrows where owned data is required.
2. Avoid `&'static` and lifetime hacks at public API boundaries unless intentionally static.
3. Make public APIs ergonomic: accept `&str`/`&[T]` or `AsRef`/`Into`/generic traits rather than `&String`/`Vec<T>` where appropriate.
4. Remove unnecessary cloning on hot paths and provide zero-copy views where feasible.
5. Prefer ownership redesign over runtime-sharing primitives (`Rc<RefCell>` / `Arc<Mutex>`) where possible.
6. Add CI checks and Clippy rules to detect regressions.

---

## Detailed remediation plan (by anti-pattern)

Each section lists: problem, detection, concrete fixes, tests/benchmarks, and an incremental rollout strategy.

### 1) Structs holding `&'a T` where the reference should be owned

- Problem: A struct stores `&'a T` but logically needs ownership (e.g., stored across API boundaries or used after the caller scope ends). This forces callers to manage lifetimes and often leads to leaking data or complex code.

- Detection:
  - Grep for `&'a ` and `struct .*<'a>` declarations: `rg "struct .*<'a>|&'a " --glob "*.rs"`
  - Manually review public `pub struct` types with lifetimes; ask: is the struct ever stored beyond the caller's stack/frame or returned from functions?

- Fixes (choose the appropriate one):
  - If the type must own the data, switch to the owned type: `String` instead of `&'a str` and `Vec<T>` instead of `&'a [T]`.

    Bad:
    ```rust
    pub struct Foo<'a> { name: &'a str }
    ```

    Good (owning):
    ```rust
    pub struct Foo { name: String }
    ```

  - If you want to allow both borrow and own without copies, use `Cow<'a, T>`:
    ```rust
    use std::borrow::Cow;
    pub struct Foo<'a> { name: Cow<'a, str> }
    ```

  - If the struct must reference many short-lived items, consider an arena/bump allocator (e.g., `bumpalo`) and document lifetimes carefully.

- Tests & validation:
  - Add unit tests that exercise constructing the struct from both owned and borrowed sources and ensure no lifetime escapes.
  - Add a small runtime test that attempts to return previously-borrowed data across API boundary; compilation should fail if lifetimes were incorrect.

- Rollout strategy:
  1. Identify all public structs with lifetimes via `rg "struct .*<'[a-z]" --glob "*.rs"`.
  2. For each, open an issue/PR proposing `String` or `Cow` replacement with rationale and benchmarks where relevant.
  3. If changing a public struct, add a `From` impl or constructor to ease migration, and add a deprecation shim where necessary.

Estimated effort: small per-struct (1–3 hrs), medium at scale across many public types.

### 2) Overusing `&'static` to “make lifetimes compile”

- Problem: `&'static` used as a shortcut forces data to be static or causes clones to be forced into `'static` storage.

- Detection:
  - `rg "&'static" --glob "*.rs"` (already run). Review each occurrence; static arrays of extension strings or true singletons are OK.

- Guidance:
  - Allow `&'static` only for genuine compile-time constants (literal arrays, compiled-in backends, global tables).
  - Avoid returning `&'static` from functions that build values dynamically. Instead return `String`, `Arc<str>`, or `Cow<'static, str>` when storing into global caches.

- Migration:
  1. Audit every `&'static` occurrence and tag with a short justification comment `// static: ok - <reason>` or consider changing to owned type.
  2. For APIs: prefer owning return types when data is constructed at runtime.

Estimated effort: review-only often small (1–4 hrs); fixes larger if many APIs change.

### 3) Returning `impl Trait` that captures hidden lifetimes

- Problem: `-> impl Iterator<Item=&T>` or `async fn` returning futures that capture non-`'static` lifetimes may compile locally but fail in real usage (e.g., when spawning tasks).

- Detection:
  - `rg "-> impl " --glob "*.rs"`
  - For `async fn`, inspect whether the function captures `&self` or local refs and then is passed to `tokio::spawn` or stored.

- Fixes:
  - Prefer returning owned containers (`Vec<T>`) if the result must outlive the current scope.
  - If you need to return an iterator over borrowed data, document the lifetime and do not expose it as `'static` unless guarantees exist.
  - For `async` functions: if a future needs to be `Send + 'static` for spawning, box it explicitly: `Box::pin(async move { ... })` or return `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.

Example (iterator):
```rust
// risky: returns impl Iterator<Item=&T> that borrows local data
fn iter_vals<'a>(v: &'a Vec<T>) -> impl Iterator<Item=&'a T> { v.iter() }

// safer for API that must own output
fn iter_vals_owned<T: Clone>(v: &Vec<T>) -> Vec<T> { v.clone() }
```

- Rollout: document lifetime contracts for all `impl Trait` returns and prefer owned outputs for cross-thread usage.

Estimated effort: small per-function but requires careful API design.

### 4) API taking `&String` or `Vec<T>` instead of `&str` / `&[T]` / generic traits

- Problem: `&String` forces callers to have a `String` (not `&str`) and `Vec<T>` parameters force allocations for callers that have slices.

- Detection:
  - `rg "&String" --glob "*.rs"`
  - `rg "&\s*Vec<" --glob "*.rs"`

- Fixes & recommended signatures:
  - Prefer `&str` over `&String`:
    ```rust
    // bad
    pub fn foo(s: &String) -> usize { s.len() }

    // better
    pub fn foo(s: &str) -> usize { s.len() }

    // flexible/generic
    pub fn foo<S: AsRef<str>>(s: S) -> usize { s.as_ref().len() }
    ```

  - For slices, prefer `&[T]` or a generic `impl IntoIterator<Item = T>` as appropriate:
    ```rust
    // bad
    fn process(v: Vec<u8>) { /* forces allocation */ }

    // better
    fn process_slice(v: &[u8]) { /* borrow-only */ }

    // or generic
    fn process_iter<I: IntoIterator<Item = u8>>(i: I) { for b in i { ... } }
    ```

- Migration strategy:
  1. Identify public fn signatures with `&String` and change to `&str` (non-breaking for callers with `String` or `&str`).
  2. For `Vec<T>` parameters, evaluate whether callers commonly pass slices — if yes, change to `&[T]` or `impl IntoIterator`.
  3. Add deprecation attributes if both signatures must coexist briefly.

Estimated effort: small per-signature; high-value wins for public API functions first.

### 5) Public API forcing clones because ownership wasn’t designed upfront

- Problem: returning owned data or accepting owned inputs causes clones in callers or library internals.

- Detection:
  - `rg "\.clone\(\)" --glob "*.rs"` and then check whether clones happen in hot paths or API boundaries.

- Fixes:
  - Return references `&T` where the caller can borrow (non-owning accessor) and document lifetime semantics.
  - Provide `into_owned()`-style methods for callers that need ownership, e.g., `fn into_name(self) -> String`.
  - Use `Cow` on types that may be borrowed or owned.

- Benchmarks & tests:
  - Add microbenchmarks on hot paths before/after changing clones (use `cargo bench` or `criterion`) to measure impact.

Rollout: prioritize hot paths and public functions used by external crates.

### 6) Using `Rc<RefCell<T>>` or `Arc<Mutex<T>>` as the default sharing primitive

- Problem: these are often used as an easy shortcut; they add runtime overhead and can mask poor ownership design.

- Detection:
  - `rg "Rc<RefCell|Arc<Mutex" --glob "*.rs"`

- Guidance & alternatives:
  - Prefer immutable `Arc<T>` for shared read-only state.
  - If mutation is required, consider redesigning ownership: message-passing, actor-style, or splitting responsibilities so mutation happens in a single owner thread.
  - Use `RwLock` if majority-of-time reads vs writes.

- Rollout: audit instances; for each usage, decide if redesign (preferred) or a documented, localized use is acceptable.

### 7) `struct S { data: Vec<u8> }` where `Cow<'a, [u8]>` or arena would be better

- Problem: frequent copies of `Vec<u8>` or many small allocations are a hidden perf cost.

- Detection:
  - `rg "Vec<u8>" --glob "*.rs"`

- Fixes:
  - Use `Cow<'a, [u8]>` when callers sometimes pass borrowed slices and sometimes owned bytes.
  - Consider an arena (e.g., `bumpalo`) for many small, same-lifetime allocations.

- Benchmarks: add `cargo bench` comparisons; estimate memory profile differences.

### 8) Builder pattern consuming `self` on every method (instead of `&mut self`)

- Problem: `fn set_x(self, ...) -> Self` forces moves and may cause allocations/moves; `&mut self` forwarding returning `&mut Self` is usually more ergonomic and efficient.

- Detection:
  - `rg "fn\s+\w+\([^\)]*self[^\)]*\)\s*->\s*Self" --glob "*.rs"` (scan impl blocks for builder-like types)

- Fix & example:
  - Bad:
    ```rust
    impl Builder {
      pub fn set_x(self, x: String) -> Self { ... }
    }
    ```
  - Good:
    ```rust
    impl Builder {
      pub fn set_x(&mut self, x: impl Into<String>) -> &mut Self {
        self.x = x.into();
        self
      }
    }
    ```

- Rollout: change builder internals first (non-public), then update public builder APIs with `#[deprecated]` shims if necessary.

### 9) Newtypes without `From` / `TryFrom` validation

- Problem: `pub struct Username(pub String);` allows construction of invalid values.

- Fix pattern: make inner field private and expose validated constructors or `TryFrom`.

  ```rust
  pub struct Username(String);

  impl TryFrom<String> for Username {
      type Error = &'static str;
      fn try_from(s: String) -> Result<Self, Self::Error> {
          if s.trim().is_empty() { Err("empty username") } else { Ok(Username(s)) }
      }
  }

  impl AsRef<str> for Username { fn as_ref(&self) -> &str { &self.0 } }
  ```

- Rollout: identify newtypes (via `rg "struct \w+\(" --glob "*.rs"`), then make inner fields private + add TryFrom/From/AsRef as appropriate.

---

## Cross-cutting automation & CI

- Add CI steps that run on PRs:
  1. `cargo fmt -- --check`
  2. `cargo clippy --all-targets --all-features -- -D warnings` (address warnings progressively)
  3. A small script to grep for dangerous patterns and fail the build if found:
     - `rg --hidden "&String|&\s*Vec<|Cow<|Rc<RefCell|Arc<Mutex|\.clone\(\)" --glob "!.git" --glob "**/target/**" || true` — run this as a diagnostic and triage matches.

- Add unit tests and benchmarks to lock down regressions and quantify cost of fixes.

## Incremental rollout plan (practical steps)

1. Lock the baseline: run `cargo test` and `cargo clippy` to record current warnings.
2. Add the diagnostic grep script to CI as a non-failing job to collect a list of problem sites.
3. Triage and prioritize by impact: find hot paths (profiling), public API surface, and test-only occurrences.
4. Implement targeted changes in small PRs with tests and benchmarks:
   - PR 1: Convert `&String` to `&str` (should be safe, simple), run tests.
   - PR 2: Replace `Vec<T>` inputs with `&[T]` where callers commonly have slices.
   - PR 3: Replace public getters that clone with `&T` returns; add `into_*` accessors for ownership where needed.
   - PR 4: Audit and refactor `pub struct ...<'a>` where ownership is required; prefer `String` or `Cow`.
   - PR 5: Add `TryFrom`/smart constructors for selected newtypes.

5. After each PR: run benchmarks and update docs and changelog notes.

## Review checklist for PR reviewers

- Is the API change backward-compatible for callers? If not, is a deprecation path provided?
- Are unit tests added/updated to cover lifetime/ownership behavior?
- Are clones removed from hot paths or justified with benchmark data?
- Has `cargo clippy` been run and warnings resolved?
- Are public types documented with their lifetime/ownership contracts?

## Suggested priority (short list)

- High (do first): public `&String` → `&str`; `Vec<T>` → `&[T]` or `IntoIterator`; remove obvious clones on hot paths; add CI diagnostics.
- Medium: audit `&'a` structs and convert to owned types or `Cow` where appropriate; add TryFrom validation for newtypes.
- Low: replace `Vec<u8>` with `Cow`/arena where proven by benchmarks; adjust builder patterns where callers expect chaining without moves.

## Useful grep commands (copy-paste)

```bash
rg "&'static" --glob "*.rs"
rg "struct .*<'[a-z]" --glob "*.rs"
rg "&String" --glob "*.rs"
rg "&\s*Vec<" --glob "*.rs"
rg "Rc<RefCell|Arc<Mutex" --glob "*.rs"
rg "Cow<" --glob "*.rs"
rg "-> impl " --glob "*.rs"
rg "impl From<|impl TryFrom<" --glob "*.rs"
rg "\.clone\(\)" --glob "*.rs"
```

If `rg` (ripgrep) is unavailable, use `grep -R --line-number "PATTERN" --include "*.rs" .`.

---

## Next actions (recommended immediate steps)

1. Add the grep diagnostics to CI as a non-failing job to produce a short report on every PR (low friction).  — (owner: infra)
2. Triage the grep report and mark high-impact sites as issues (owner: code owner for module).  — (owner: maintainers)
3. Start small PRs: convert `&String` → `&str` and remove trivial `.clone()` sites in hot paths (owner: contributor).  — (owner: any contributor)

---

If you want, I can:

- open individual PR candidates for the highest-priority changes (e.g., `&String` → `&str`),
- create the CI diagnostic script and add it to the repo, or
- run a deeper static analysis that lists every public `pub fn` signature that mentions `String`/`Vec`/lifetimes.

---

End of plan.
# Plan: Unsafe Code Soundness (compiles, but UB possible)

## Quick findings

- There is already a small set of unsafe-related heuristics in the codebase that check for `unsafe` usage and nearby `SAFETY:` comments (see `src/analysis/rust/mod.rs` and the Rust guides such as `guides/rust/heuristics-and-findings.md` and `guides/rust/parser-and-evidence-extraction.md`).
- I did not find targeted checks for the specific *unsafe-soundness* patterns below (no matches for `get_unchecked`, `transmute`, `from_raw_parts`, `set_len`, `assume_init`, etc.).

## Goal

Design and implement a robust set of static-detection heuristics, tests, and CI checks to catch common *unsafe code soundness* issues that compile but can cause undefined behavior (UB) at runtime. The focus is on the following problems described by the team:

1. Violating struct invariants from safe code (e.g., manually tweaking `Vec` internals) — leading to `get_unchecked` UB.
2. Unsafe blocks that implicitly rely on *generic* safe code preserving state — broken when generics change or traits are implemented differently.
3. `get_unchecked` (and equivalents) used without demonstrating bounds are preserved on all paths.
4. Unsafe code spread across modules without privacy boundaries so safe code can break invariants.
5. Assuming `&mut T` uniqueness when generics or interior mutability are involved (aliasing violations).
6. Using `transmute` or raw pointers without ensuring `Send`/`Sync` or other cross-thread guarantees — potential data races.

## Scope

- Add static detection *heuristics* in the Rust analysis backend (non-invasive; heuristic-based like existing rules).
- Add unit and integration test fixtures showing both true positives and false-positive controls.
- Integrate with existing evidence extraction (SAFETY comments) so legitimate patterns can be accepted when properly documented.
- Optionally add Miri-based runtime tests for high-confidence UB detection in CI.

## Deliverables

- A new rule pack (or module) `unsafe_soundness` implemented under `src/analysis/rust/` (suggested filename `unsafe_soundness.rs`) that exports individual detectors.
- Tests and fixtures under `tests/integration_scan/unsafe_soundness.rs` and `tests/fixtures/rust/unsafe_soundness/*`.
- Guide and docs: this plan (saved at `guides/rust/newplan.md`) and a short developer doc `guides/rust/unsafe-soundness.md` describing accepted `SAFETY:` comment format for these rules.
- CI additions: a job that runs the new heuristics over fixtures and optionally runs `cargo miri test` on selected tests.

## Detection rules (detailed)

Each detector below includes: pattern to match, detection algorithm, heuristics to reduce false positives, suggested evidence extracted, and example test snippets.

### 1) Detect `get_unchecked` / bounds-less indexing

- Pattern: function/method call tokens named `get_unchecked`, `get_unchecked_mut`, `ptr::read`, `ptr::write`, `slice::from_raw_parts(_mut)`, `Vec::set_len`, `Vec::as_mut_ptr`, `slice::as_mut_ptr` used inside `unsafe` contexts.
- Detection algorithm:
  - Parse AST tokens for those function names inside `unsafe` blocks or `unsafe fn`.
  - For each occurrence, perform a local control-flow check: is there an explicit bound check along *all* paths reaching the call? Simple heuristics:
    - Look for `if index < len { ... } else { ... }` or `debug_assert!(index < len)` in same function prior to call.
    - Look for a previous call that calculates index using safe wrapper that is known to bound result (e.g., `.get()` result handled, checked `checked_sub` etc.).
    - If a guard exists in the same basic block or dominates the call, treat as OK; else flag.
- Heuristics to reduce FP:
  - Accept when a nearby `SAFETY:` comment documents the proof (see `SAFETY:` policy below).
  - If guard occurs in a different function but that function is private and previously analyzed to maintain invariant, treat with lower severity (or warn instead of error).
- Evidence: record code snippet, unsafe line, nearby `SAFETY:` lines (if any), and the variable names used in bounds.
- Example (should be flagged):

```rust
unsafe fn use_unchecked(v: &Vec<i32>, i: usize) -> i32 {
    // no bound check
    *v.get_unchecked(i)
}
```

Example (should be accepted if proof exists):

```rust
unsafe fn use_unchecked(v: &Vec<i32>, i: usize) -> i32 {
    // SAFETY: caller ensures `i < v.len()`
    *v.get_unchecked(i)
}
```

### 2) Detect `transmute`, `mem::transmute`, raw pointer casts used unsafely

- Pattern: `mem::transmute`, `std::ptr::cast`, manual `as *const`/`as *mut` conversions, `MaybeUninit::assume_init`
- Detection algorithm:
  - Flag every `transmute` + raw pointer casts and require either: (A) an explicit `SAFETY:` justification covering type layout/equivalence, or (B) type-level proof such as `#[repr(C)]` with documented layout and tests.
  - For transmutes that can change the `Send/Sync` properties (e.g., transmute from a non-Send type to a type that may be sent across threads), flag as high-severity unless a `SAFETY:` justification and explicit `unsafe impl Send`/`Sync` are present.
- Evidence and heuristics: when `transmute` is in a private module with narrow usage and SAFETY comment present, lower the severity.

### 3) Unsafe code relying on generic invariants

- Pattern: `unsafe` blocks that call generic functions, use generic types' internal fields or assume properties of `T` without trait bounds.
- Detection algorithm:
  - Identify `unsafe` blocks or `unsafe fn` that call functions/methods that are generic or accept `T` where `T` is unconstrained (no trait that enforces invariants).
  - If the unsafe block assumes layout or behavior of `T` (for example, transmuting `T` to bytes, or assuming `T: Copy`), require explicit bounds or SAFETY comment.
- Heuristic: only flag when the generic dependency is external/public (i.e., callers can instantiate `T` arbitrarily), or when interior mutability types (UnsafeCell/RefCell) are involved.

### 4) Unsafe code spanning modules with broken privacy barriers

- Pattern: `pub` functions or types that expose raw pointers or `unsafe` constructors that return types allowing callers to violate invariants (e.g., public function that takes raw parts and returns a `Vec<T>`-like wrapper without enforcing invariants)
- Detection algorithm:
  - Detect `pub` `unsafe fn` or `pub` APIs that convert raw parts/ptr into safe-looking types.
  - If type invariants rely on private fields that are accessible through other public helper functions (or if raw pointers can be created by callers), flag potential invariant leakage.
- Recommendation: prefer private constructors / `pub(crate)` constructors, or require `SAFETY:` comments explaining how public API preserves invariants.

### 5) Aliasing assumptions & interior mutability

- Pattern: code assumes `&mut T` uniqueness and then uses `as *mut T` across scopes, or casts `&mut T` into multiple raw pointers and uses them concurrently; also, taking references when `T` contains `UnsafeCell`/`RefCell`.
- Detection algorithm:
  - Flag `as *mut` or `as *const` flows that originate from `&mut` and escape function boundary or are stored in `static mut`.
  - If generic parameter `T` is present and could be an `UnsafeCell` (or not bounded by `Sync`/`Send`), flag with advisories.

### 6) Send/Sync guarantees and thread-safety

- Pattern: raw pointers or transmutes used in code that is `Send`/`Sync` or used with thread spawns.
- Detection algorithm:
  - Find uses of `std::thread::spawn`, `crossbeam`, `rayon`, or explicit `std::thread::spawn(move || ...)` that move raw pointers across threads.
  - When raw pointers are sent across thread boundaries or types are transmuted, require either `T: Send`/`Sync` bounds or explicit `unsafe impl Send/Sync` with SAFETY justification.
  - If `unsafe` code constructs a type assumed to be thread-safe, but that type contains `NonSend` subtypes or `!Sync` elements (based on obviously-known wrappers), flag it.

## SAFETY comment policy (extended)

- Extend the existing `unsafe_without_safety_comment` rule to accept structured justifications for soundness rules.
- Suggested mini-spec for `SAFETY:` lines to reduce FPs:

```
// SAFETY: <short tag>; required invariants: <list>; proof sketch: <short proof or reference to test>; assumptions: <caller responsibilities>
```

- Example: `// SAFETY: bounds-proof; requires `i < v.len()`; caller guarantees this.`

When a detector finds an explicit matching `SAFETY:` comment that names a relevant tag (e.g., `bounds-proof`, `layout-proof`, `send-sync-proof`), the detector lowers severity or suppresses the finding.

## Implementation plan and file map

1. Create module `src/analysis/rust/unsafe_soundness.rs` with the following sub-components:
   - `find_get_unchecked_and_related()` — AST matchers for `get_unchecked`, `set_len`, `from_raw_parts`, `assume_init`, etc.
   - `check_bounds_guards()` — simple control-flow dominance-based check for guards in the same function.
   - `find_transmute_and_pointer_casts()` — detect `transmute`, `as *mut`, `as *const` and `MaybeUninit::assume_init`.
   - `detect_generic_dependency_issues()` — identify unsafe blocks that call generic or trait-supplied functions without bounds.
   - `expose_violation_across_modules()` — identify public unsafe constructors and raw-part-taking APIs.

2. Integrate detectors with existing heuristics engine by registering a new rule pack name `unsafe_soundness` (so it can be enabled/disabled like other rules). Hook into evidence extraction to collect `SAFETY:` comments.

3. Add tests and fixtures:
   - Add positive fixtures (should flag) for each detector.
   - Add negative fixtures (should not flag) for legitimate usages with adequate `SAFETY:` comments.
   - Add small miri-target tests for 1–2 high-confidence UB patterns (e.g., use-after-free, out-of-bounds raw writes) and mark them as `cargo miri` tests in CI.

4. CI changes:
   - Add a job `rust-unsafe-soundness` to run the analysis and the new tests.
   - Optionally run `cargo miri test` on a smaller test subset (Miri can be slow).

5. Documentation:
   - Add `guides/rust/unsafe-soundness.md` summarizing the rule pack, accepted `SAFETY:` format, and developer guidance for fixing flagged code.

## Test examples (concrete fixtures)

- Case A (flagged): `get_unchecked` without guard

```rust
fn bad(v: &Vec<u8>, i: usize) -> u8 {
    unsafe { *v.get_unchecked(i) }
}
```

- Case B (allowed if SAFETY present):

```rust
// SAFETY: bounds-proof; caller guarantees `i < v.len()`.
unsafe fn ok(v: &Vec<u8>, i: usize) -> u8 {
    *v.get_unchecked(i)
}
```

- Case C (flagged): `transmute` used to remove non-Send property

```rust
let x: NonSendType = ...;
let y: usize = unsafe { std::mem::transmute(x) }; // flag unless documented
```

- Case D (flagged): public `unsafe` constructor exposing raw parts

```rust
pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> MyVec<T> { ... }
```

## Prioritization and timeline (suggested)

- Week 0 — design & discovery: finalize rule list and SAFETY comment schema (this doc).
- Week 1 — implement `get_unchecked`/bounds detector + tests and integrate with heuristics engine.
- Week 2 — implement `transmute`/pointer cast detector, add public API checks and tests.
- Week 3 — implement generic-dependency and aliasing detectors; add representative fixtures.
- Week 4 — CI integration, Miri test subset, documentation and developer guidance.

Estimated effort: 2–3 developer-weeks (1 engineer full-time) to reach a robust alpha with tests; additional time for hardening, FP reduction, and Miri coverage.

## Acceptance criteria

- The rule pack detects the example UB patterns in `tests/fixtures/rust/unsafe_soundness/*` and flags them in the integration test runner.
- False positives are acceptably low on a selected baseline of crates (define a baseline / sample set from the repo or from public crates).
- SAFETY comments suppress findings when they provide a clear short justification and proof sketch.

## Risks and mitigations

- Static heuristics will never be perfect; complex dataflow/aliasing will produce FPs and FNs. Mitigate with:
  - A structured `SAFETY:` comment policy to document acceptable usages.
  - Miri-based dynamic tests for high-confidence detection.
  - A process for whitelisting and tuning heuristics based on real-world false positives.

## Next concrete steps (what I can do now)

1. Open a branch and scaffold `src/analysis/rust/unsafe_soundness.rs` with a small detector for `get_unchecked` + test fixtures. (Estimated 1–2 days.)
2. Add `SAFETY:` structured-comment parser/normalizer to evidence extraction (existing `unsafe` comment extractor likely already useful). (half-day.)
3. Add initial CI job to run the new rule pack on fixtures and produce a report. (half-day.)

---

If you'd like, I can proceed to scaffold the implementation (create the new Rust analyzer file and at least one failing fixture + test). Next step would be: implement the `get_unchecked` detector and a minimal unit test harness that runs the repo's integration scanner against the fixture.

---

# Project & Crate Structure — Audit & Migration Plan

Date: 2026-03-27

Purpose
- Verify whether the repository already avoids common Rust project/crate structural anti-patterns and, where it does not, provide a precise, testable migration plan.

Findings (quick)
- `lib.rs` and `main.rs` exist and are properly separated: no mixing of heavy library logic into `main.rs`.
- No dangling `mod` declarations discovered (script reported no missing external mod files).
- Multiple `mod.rs` files are present (listed below) — the primary actionable item is migrating to the modern `foo.rs` + `foo/` layout if the team wants that convention.
- `Cargo.toml` does not contain explicit `[lib]` or `[[bin]]` sections; the project currently relies on defaults (this is harmless but an explicit `[lib]` entry is recommended for clarity).

`mod.rs` locations (detected)
- src/cli/mod.rs
- src/scan/mod.rs
- src/analysis/python/parser/mod.rs
- src/analysis/python/mod.rs
- src/analysis/mod.rs
- src/analysis/rust/mod.rs
- src/analysis/go/parser/mod.rs
- src/analysis/go/mod.rs
- src/model/mod.rs
- src/benchmark/mod.rs
- src/heuristics/mod.rs
- src/heuristics/python/mod.rs
- src/index/mod.rs

Automated checks we ran (evidence)
- root `Cargo.toml` inspected: no `[workspace]` and no explicit `[lib]` or `[[bin]]` sections. See [Cargo.toml](Cargo.toml).
- Verified `src/lib.rs` and `src/main.rs` exist and `main.rs` delegates to `lib.rs` (good separation).
- Ran a script to detect missing `mod` targets; result: "No missing external mod files found".

Recommended detailed migration plan

1) Create a branch and baseline

```bash
git checkout -b rust/structure-migration
cargo build --verbose
cargo test --verbose
```

2) Inventory & baseline measurements

```bash
# list mod.rs files
rg --hidden --line-number "mod\.rs$" -g "src/**" || true

# list the largest Rust files to surface >5k LOC files (if any)
find src -name '*.rs' -print0 | xargs -0 wc -l | sort -n | tail -n 30

# quick missing-mod check (script used earlier)
python - <<'PY'
import re,os
errs=0
for root,dirs,files in os.walk('src'):
  for f in files:
    if f.endswith('.rs'):
      p=os.path.join(root,f)
      text=open(p,encoding='utf-8',errors='ignore').read()
      for m in re.finditer(r"\bmod\s+([A-Za-z0-9_]+)\s*;", text):
        modname=m.group(1)
        d=os.path.dirname(p)
        if not (os.path.exists(os.path.join(d,modname+'.rs')) or os.path.exists(os.path.join(d,modname,'mod.rs'))):
          print('MISSING',p,modname)
          errs+=1
print('errors:',errs)
PY
```

3) Migrate `mod.rs` → `foo.rs` (incremental)

Rationale: prefer `foo.rs` + `foo/` layout for clearer editor navigation and fewer surprises in tooling. The migration is mechanical but should be done one module at a time with build/test verification per-step.

Safe iterative process (manual or scripted):

```bash
# interactive: print the files to migrate first
git ls-files 'src/**/mod.rs'

# incremental migration (review each move and run build/test):
for modfile in $(git ls-files 'src/**/mod.rs'); do
  moddir=$(dirname "$modfile")
  modname=$(basename "$moddir")
  parentdir=$(dirname "$moddir")
  dest="$parentdir/${modname}.rs"

  echo "Move: $modfile -> $dest"
  git mv "$modfile" "$dest"
  git commit -m "chore(rust): move $modfile -> $dest (mod.rs -> ${modname}.rs)"

  # verify immediately after each move
  cargo build || { echo "build failed after moving $modfile"; exit 1; }
  cargo test || { echo "tests failed after moving $modfile"; exit 1; }
done

# confirm no remaining mod.rs
rg "mod\.rs" -n src || true
```

Notes & caveats:
- If a destination file already exists, merge manually and run tests.
- If `include_str!` or other path-based macros rely on file-relative paths, update those paths.
- If a moved module had `pub(crate)`/`pub(super)` details, the module tree does not change; however, double-check `use` paths and module visibility.

4) Add explicit Cargo manifest metadata (recommended)

Add to `Cargo.toml` to make crate entrypoints explicit:

```toml
[lib]
name = "deslop"
path = "src/lib.rs"

[[bin]]
name = "deslop"
path = "src/main.rs"
```

Commit and run `cargo check`.

5) Audit public API surface

- Generate a list of `pub use` / `pub mod` / public `pub fn` signatures:

```bash
rg "^pub (use|mod) |^pub fn|^pub struct|^pub enum" src --hidden || true
```

- Decide per-symbol whether it is intended for external use. For internal helpers, change to `pub(crate)` or `pub(super)`.

6) CI & enforcement

- Add `cargo fmt -- --check` and `cargo clippy -- -D warnings` to CI.
- Add a non-failing diagnostic job that runs the grep/script used above and posts results to PRs for triage.

7) Optional: split to a workspace (only if multi-crate split is desired)

- If you intend to split library and CLI into separate crates, scaffold `crates/core` and `crates/cli`, move code, and create a root `Cargo.toml` with a `[workspace]` `members` list. Validate with `cargo build --workspace`.

8) Acceptance criteria

- `cargo build` & `cargo test` pass locally and in CI after the migration.
- No dangling `mod` mismatches and no unintended API surface expansions.
- Team agrees on the module layout convention (ban/allow `mod.rs`) and CI enforces it.

Rollback & safety
- Make single-purpose commits and verify build/test after each. If anything is broken, revert the last commit and fix the root cause before continuing.

Deliverables
- Branch `rust/structure-migration` with incremental commits.
- Updated `guides/rust/newplan.md` (this file) with the crate-structure plan appended.
- Optional: a small helper script under `scripts/` to standardize the migration.

Next steps I can take now
- Create the migration helper script under `scripts/migrate-modrs.sh` and open a draft branch with the first 1–3 module moves for review.
- Add the non-failing CI diagnostic job that lists `mod.rs`, missing `mod` targets, and public API items.

---

End of Project & Crate Structure plan
