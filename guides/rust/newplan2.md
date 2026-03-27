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
