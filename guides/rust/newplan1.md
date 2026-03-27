# Rust Performance-Checks: Implementation Plan

Last updated: 2026-03-27

Purpose
- Verify which of the requested Rust performance checks are already implemented in the codebase and, where missing, provide a detailed implementation plan (parser changes, heuristics, tests, examples, estimates).

Scope
- The checks below target the Rust language analyzer/backend in `src/analysis/rust` and the heuristics framework under `src/heuristics`.

High-level summary of repository findings
- The repository contains a `RustAnalyzer` at `src/analysis/rust/mod.rs` and a Rust parser at `src/analysis/rust/parser.rs` which already extracts call sites, imports, signatures, unsafe blocks, and other structural information.
- There is no dedicated Rust performance heuristics module under `src/heuristics` (the existing `src/heuristics` folder contains Go- and Python-targeted rules). Therefore the requested Rust-specific performance checks are not implemented and must be added.

Goals for this project
- Add a Rust-targeted heuristics module that detects the listed performance anti-patterns.
- Extend the Rust parser minimally to surface the required signals for robust detection.
- Add focused unit/integration tests and small example fixtures demonstrating both positive and negative cases.
- Integrate new rules with existing findings/reporting and include clear `rule_id`/`severity`/`evidence` for each check.

Implementation plan (per item)

IMPORTANT: general approach
- Parser changes: extend `src/analysis/rust/parser.rs` to collect specific syntactic patterns (extra fields on `ParsedFunction` and/or new small helper structures). Prefer incremental additions — only add fields we need.
- Heuristics code: create a new directory `src/heuristics/rust/` and a file `performance.rs` containing Rust-targeted performance rules, following the pattern used by `src/heuristics/python/performance.rs`.
- Integration: add `mod rust;` and wire the new evaluators in `src/heuristics/mod.rs` and the top-level evaluation paths that dispatch on `Language::Rust` (see existing `evaluate_rust_findings` in `src/analysis/rust/mod.rs`).
- Tests: add fixtures under `tests/fixtures/rust/` and add integration tests that run the analyzer on fixtures and assert presence/absence of `rule_id`s.
- Profiling: for some checks (hot-path AoS/SoA) prefer a runtime profiling step to verify actual hot spots — the linter will still provide heuristic suggestions.

Per-item detailed tasks

1) Array-of-structs (AoS) vs Struct-of-Arrays (SoA) in hot loops
- Why: iterating over Vec<Struct> and repeatedly accessing different fields causes poor spatial locality for some large datasets; SoA can be faster for numeric/analytic workloads.
- Detection heuristics (static approximation):
  - Detect loops over vectors/arrays of structs: tree-sitter node types `for_expression`, `while_expression` or `for..in` style where the iterated value is `IDENT` or `&IDENT` originating from a collection variable whose type or naming indicates a container (e.g., `Vec<...>` or `_vec`).
  - Inside loop body, detect multiple independent field accesses on the loop element (e.g., `item.a`, `item.b`, `item.c`) across the loop. If 2+ distinct fields are accessed repeatedly, flag as potential AoS hot path.
  - Optionally, detect indexed loops with `arr[i].field` patterns.
- Parser changes:
  - Add `vec_struct_field_access_loops: Vec<StructFieldLoop>` to `ParsedFunction` (define `StructFieldLoop { line: usize, container: String, fields: Vec<String> }`). Populate by scanning loop bodies and tracking `field_expression`/`field_access` patterns with the receiver pointing to a loop variable.
- Heuristic implementation (`rule_id`: `rust.aos_hot_path`, severity `Info`/`Warning`):
  - If a `StructFieldLoop` has >= 2 fields accessed and container's type looks like a `Vec<...>` or `&[T]`, emit finding with evidence listing the fields and loop location.
- Tests/fixtures:
  - Positive: loop over `Vec<Point>` accessing `.x` and `.y` branches.
  - Negative: loop using only a single field or operating on copies/refs explicitly optimized.
- Estimated complexity: Medium (2-4 days). Edge cases and false positives need careful examples.

2) `Vec<Box<T>>` or linked lists (pointer chasing)
- Why: `Vec<Box<T>>` or linked `Box`-based lists result in pointer-chasing and poor cache locality.
- Detection approach:
  - Scan type usage and `let` bindings for `Vec<` followed by `Box<` or `LinkedList` occurrences.
  - Detect iteration patterns over such containers in loops and flag.
- Parser changes:
  - Add `boxed_container_lines: Vec<usize>` or `box_container_usages: Vec<ContainerUsage>` where `ContainerUsage { line, container, inner_type }`.
  - Use tree-sitter to read generic type text in `type_identifier` or `call_expression` sites that construct `Vec::<Box<_>>::new()` or `Vec::new()` where assignment type is `Vec<Box<...>>`.
- Rule (`rust.pointer_chasing_vec_box`, severity `Info`):
  - Suggest `Vec<T>` (non-boxed) if ownership/size allows, or recommend slab/arena allocation or `Vec<T>` of primitives.
- Tests & estimates: Small-medium (1-2 days).

3) HashMap poor key distribution or default hasher
- Why: `std::collections::HashMap` uses a secure but slower hasher; for internal hot maps `FxHashMap` or `BTreeMap` can be faster depending on access pattern.
- Detection approach:
  - Detect occurrences of `HashMap<` (and `HashMap::new()`), especially in hot loops or in structures used heavily (evidence: used in loop or called frequently within functions with high call counts).
  - Look at `use` lines to see if `FxHashMap` or `hashbrown::HashMap` already present; detect `HashMap::with_hasher` usage.
- Parser changes:
  - Add `call_site_usages` detection already exists in `calls`; extend to capture constructor patterns and `use` import alias mapping (parser already extracts imports).
- Rule (`rust.hashmap_default_hasher`, severity `Info`):
  - If a `HashMap` is allocated in a hot function or loop and there's no custom hasher, emit guidance listing alternatives (FxHashMap, BTreeMap) and a short rationale.
- Tests & estimate: Small (1-2 days), but guidance must be conservative.

4) `.lines()` iterator on large files
- Why: `.lines()` on `BufRead` yields `String` per line; `read_line` with a reused buffer avoids repeated allocations.
- Detection approach:
  - Detect invocation patterns `.lines()` on `File`/`BufReader`/`std::io::Read` objects within functions; if called inside a loop or large-file processing function, flag.
- Parser changes:
  - Populate `calls` already exists; implement a check in heuristics to find `receiver` names that resolve to `File`/`BufReader` imports and call name `lines`.
- Rule (`rust.lines_allocate_per_line`, severity `Info`):
  - Evidence: call line, receiver import resolution.
  - Suggest: `let mut line = String::new(); while reader.read_line(&mut line)? { ...; line.clear(); }`.
- Tests & estimate: Small (1 day).

5) Unbuffered `std::fs::File` or `writeln!` in tight loops
- Why: unbuffered writes cause syscalls often; `BufWriter` amortizes syscalls.
- Detection approach:
  - Detect `writeln!` or `write!` macro invocations where the writer expression is a `File` (no `BufWriter`) within loop bodies.
  - Detect direct `File::write_all` or `File::write` calls inside loops.
- Parser changes:
  - Use `calls` (macro invocations are available) and import alias resolution to map receiver to `std::fs::File`.
  - Add `writeln_in_loop_lines: Vec<usize>` or similar entries.
- Rule (`rust.unbuffered_file_writes`, severity `Info`):
  - Recommend wrapping `File` with `BufWriter::new(file)` and using that in the loop.
- Tests & estimate: Small (1 day).

6) UTF-8 validation on known-valid byte data
- Why: `std::str::from_utf8` validates bytes; `from_utf8_unchecked` avoids checks but is unsafe — only recommend when caller can guarantee data validity.
- Detection approach:
  - Detect `from_utf8` calls in hot functions and flag with caution that `from_utf8_unchecked` is faster but unsafe.
  - Only recommend use when source is e.g., data coming from known UTF-8 sources (`String` conversion, known literals, protocol guarantees) — this will be a conservative hint.
- Parser changes: none or minimal — calls are in `calls`.
- Rule (`rust.utf8_validate_hot_path`, severity `Info`):
  - Evidence: call site location; message contains trade-offs.
- Tests & estimate: Small (1 day).

7) `Path::join` with absolute paths
- Why: joining with an absolute path discards base path and is often a logic bug.
- Detection approach:
  - Detect `Path::join` or `path.join(...)` where the argument is a string literal starting with `/` or `PathBuf::from("/")`.
  - Use tree-sitter to check the join argument node if it's a string literal and inspect its text.
- Parser changes:
  - No global schema changes needed; detect this in heuristics using the raw `body_text` or finer-grained AST navigation available via tree-sitter.
- Rule (`rust.path_join_absolute`, severity `Warning`):
  - Evidence: literal text and join location; explain that `join` discards the base when the RHS is absolute.
- Tests & estimate: Small (1 day).

8) Holding locks or RAII guards across `.await`
- Why: holding sync locks across await points can deadlock/serialize the async executor.
- Detection approach:
  - Detect calls to `.lock()` or `tokio::sync::Mutex::lock().await` and then search the same function body for subsequent `.await` occurrences while the guard variable is still in lexical scope.
  - Conservative heuristic: if `lock().await` occurs and there is another `.await` later in the function before an obvious early drop (e.g., explicit `drop(guard)`), flag.
- Parser changes:
  - Add `await_lines: Vec<usize>` to `ParsedFunction` and track `macro`/`call_expression` nodes that include `.await` or parse `await_expression` nodes if tree-sitter exposes them.
  - Add `lock_call_lines: Vec<LockCall>` with `LockCall {line, receiver}`.
- Rule (`rust.lock_across_await`, severity `Warning`):
  - Evidence: lock site and later await site lines; message explains deadlock/starvation risk.
- Tests & estimate: Medium (2-3 days). This requires careful scope analysis to reduce false positives.

9) `tokio::sync::Mutex` when `std::sync::Mutex` suffices
- Why: `tokio::sync::Mutex` is async-aware but has extra overhead; for short critical sections without `.await`, a `std::sync::Mutex` may be better.
- Detection approach:
  - If `tokio::sync::Mutex` is used and analysis of the guarded section shows no `.await` or `await`-free critical section, recommend `std::sync::Mutex`.
- Parser changes: same data from #8 (await_lines, lock_call_lines) plus import resolution to detect `tokio::sync::Mutex` usage.
- Rule (`rust.tokio_mutex_unnecessary`, severity `Info`):
  - Evidence: import alias and lack of `.await` while guard in scope.
- Tests & estimate: Medium (2-3 days).

10) Large futures on stack (complex async blocks)
- Why: large local variables captured by `async` blocks/fns increase the future size and can cause stack pressure or performance issues.
- Detection approach:
  - Detect `async fn` or `async` blocks and check for large-looking local bindings (large arrays, heap-allocated buffers, or many captured variables). Use heuristics: variables of type `[T; N]` or arrays > threshold, or many `Vec` declarations.
  - Suggest boxing the future (`Box::pin`) or refactoring to move large allocations out of the future scope.
- Parser changes:
  - Add `async_functions: Vec<AsyncFunctionSummary>` capturing `start_line`, `local_bindings_summary` (counts, types seen as text).
- Rule (`rust.large_future_stack` severity `Info`):
  - Evidence: listing of suspicious local bindings.
- Tests & estimate: Medium (3 days). This is conservative and may require refinement to reduce false positives.

11) Blocking I/O or `std::fs` inside async runtime
- Why: calling blocking I/O primitives in an async runtime worker thread blocks the executor.
- Detection approach:
  - Detect `std::fs::read_to_string`, `std::fs::File::open`, `File::read_to_end`, etc. inside `async fn` or function bodies containing `.await` (i.e., likely in async context).
  - Suggest `tokio::fs` equivalents or `spawn_blocking`/`blocking` wrappers.
- Parser changes:
  - Add `is_async_function` to `ParsedFunction` (populate by detecting `async` keyword near function signature using tree-sitter).
  - Use existing `calls` to detect `std::fs` invocations.
- Rule (`rust.blocking_io_in_async`, severity `Warning`):
  - Evidence: call site and enclosing function marked `async`.
- Tests & estimate: Small-medium (2 days).

12) `Drop` impls that do blocking cleanup
- Why: `Drop` runs on stack-unwind or when the guard goes out of scope; blocking operations here can block runtime threads.
- Detection approach:
  - Find `impl Drop for` blocks and inspect function body for calls to blocking APIs (`std::fs::`, `std::thread::sleep`, `std::net::TcpStream::shutdown`, etc.).
  - Emit a warning if blocking ops are present in `drop`.
- Parser changes:
  - No big structural changes; parser already exposes function bodies and call sites. Heuristic code will search for `impl` blocks with trait `Drop`.
- Rule (`rust.blocking_drop`, severity `Warning`):
  - Evidence: text of drop method and blocking call sites.
- Tests & estimate: Small (1-2 days).

Cross-cutting work (integration, tests, doc)
- Create new module: `src/heuristics/rust/performance.rs` (create) and `src/heuristics/rust/mod.rs` (if needed).
- Update `src/heuristics/mod.rs` to import the new Rust heuristics and call them from a new `evaluate_rust_file()` function or integrate with existing `evaluate_*` flows.
- Add unit tests under `tests/` and fixtures under `tests/fixtures/rust/` with small source files exercising each rule (both positive and negative cases). Mirror the style used by other language fixtures in repo.
- Add integration test(s) to `tests/integration_scan.rs` or a new rust-specific integration test that runs the analyzer and asserts on returned `rule_id`s.
- Add short documentation to `guides/rust/` with examples and mitigation guidance (this file is the starting point).

Acceptance criteria
- Each of the rules above is implemented as a heuristic with:
  - `rule_id` and `severity` defined and used consistently with other heuristics
  - test fixtures covering true positives and false positives
  - integration test ensuring the rule triggers for the fixture during repo scan
- Parser changes are minimal, well-documented, and used only for the necessary heuristics.
- The heuristics are conservative (prefer false negatives over noisy false positives) and the messages provide remediation steps.

Prioritization and estimated effort
- Phase A (high priority, 3-7 days):
  - Blocking I/O in async runtime (`rust.blocking_io_in_async`) — 1-2 days
  - Unbuffered file writes / `writeln!` in loops (`rust.unbuffered_file_writes`) — 1 day
  - `.lines()` allocation issue (`rust.lines_allocate_per_line`) — 1 day
  - HashMap default hasher in hot paths (`rust.hashmap_default_hasher`) — 1-2 days
- Phase B (medium priority, 4-9 days):
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
