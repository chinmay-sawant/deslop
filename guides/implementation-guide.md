# deslop Implementation Guide

## Overview

deslop is structured as a multi-stage Rust analysis pipeline for Go, Python, and Rust repositories. The current implementation focuses on fast full-repository ingestion, lightweight structural fingerprints, a repository-local symbol index, explainable heuristic findings, and repeatable benchmark measurements.

The code is intentionally split so later phases such as copypasta detection, control-flow analysis, taint tracking, and auto-fix generation can consume stable intermediate results instead of reparsing source files or embedding analysis logic inside the CLI.

## Current architecture

### CLI layer

`src/main.rs` exposes two subcommands:

- `scan <path>` runs the analyzer and prints either text or JSON
- `bench <path>` runs the pipeline repeatedly and reports timing statistics

Both commands accept `--no-ignore`, and both support `--json` for structured output.
The `scan` command defaults to a compact per-function summary and accepts `--details` to include full fingerprint metrics.

### Repository discovery

`src/scan/walker.rs` uses the `ignore` crate to walk the target directory. The current policy:

- respects `.gitignore` by default
- works even when the scanned path is not itself a Git repository
- skips unsupported files and keeps language detection extension-driven through the registered backends
- skips `vendor/` paths

This stage is responsible only for file selection. It does not parse or classify file contents.

### Parse stage

`src/analysis/mod.rs` exposes the language-agnostic analyzer boundary. `src/analysis/go/parser.rs` wraps `tree-sitter` and `tree-sitter-go`, `src/analysis/python/parser/mod.rs` wraps `tree-sitter-python`, and `src/analysis/rust/parser.rs` wraps `tree-sitter-rust` for the current backends.

For each Go file, the parser extracts:

- package name
- imports and effective aliases
- declared symbols such as functions, methods, structs, interfaces, and types
- per-function call sites
- context parameter presence for imported `context` aliases
- raw goroutine launch sites
- `time.Sleep` calls inside loops
- string concatenation sites inside loops when the target variable is clearly string-like
- per-function fingerprints

Parsing remains syntax-tolerant. Files with syntax errors still participate in the report when tree-sitter can recover enough structure.

For Python files, the current parser extracts:

- conservative module names derived from file paths
- import statements and aliases from `import` and `from ... import ...` forms
- module-level functions, class methods, and class symbols
- per-function call sites using direct and attribute-style call targets
- async function classification through function-kind metadata
- module-level and local string literals that can support shared secret-style heuristics
- function docstrings, local binding names, conservative exception-handler evidence, and conservative test-file and test-function classification
- loop-local string concatenation evidence for the shared performance rule
- syntax error state

Python support remains syntax-oriented. It does not execute imports, resolve installed packages, or attempt mypy-style semantic understanding.

For Rust files, the current parser extracts:

- a conservative module name derived from the file path
- `use` declarations and conservative aliases
- declared symbols such as functions, methods, structs, traits, enums, and type aliases
- per-function call sites, including macro invocations such as `dbg!` and `todo!`
- test-only function classification derived from `tests/`, `#[test]`, and `#[cfg(test)]`
- package-level and local named string literals
- unsafe usage lines plus nearby `SAFETY:` comment lines
- syntax error state
- lightweight function fingerprints and call counts

Rust still avoids type-aware semantic modeling, macro expansion, and cargo-graph reasoning. The parser stays syntax-oriented so the scan pipeline can handle `.rs` files without special casing while keeping the evidence model reviewable.

### Fingerprint stage

`src/analysis/go/fingerprint.rs` computes lightweight function-level signals:

- line span
- comment and code line counts
- comment-to-code ratio
- a simple control-structure complexity proxy
- a symmetry score derived from repeated sibling statement kinds
- boilerplate `if err != nil { return err }` suppression
- signature-level weak-typing markers such as `any` and `interface{}`
- type assertion count
- call count

These fingerprints are intended as low-cost primitives for later heuristic and ranking layers.

### Symbol index

`src/index/mod.rs` builds a repository-local package index from parsed files. The current version tracks:

- package-level functions keyed by package name plus repository-relative directory
- methods grouped by receiver type
- declared symbols for counting and future evidence output
- per-package import counts
- local directory origins for more precise import-call resolution

The index is now language-scoped as well, so mixed Go, Python, and Rust repositories do not merge same-directory symbols into one local package entry.

This index is deliberately lightweight. It is useful for local-context checks, but it is not an authoritative substitute for `go/types`. Import-call resolution is now package-plus-directory aware, which reduces ambiguity when multiple local packages share the same package name.

### Heuristic layer

`src/heuristics/mod.rs` currently implements several shared and Go-specific rule families, `src/heuristics/python/` now hosts the first Python-specific rule pack, and `src/analysis/rust/mod.rs` hosts the first Rust-specific rule pack:

1. `generic_name`
   Flags functions whose names are unusually generic, but only when the function also lacks stronger contextual signals such as specific typing or commentary.

2. `overlong_name`
   Flags unusually long identifiers with many tokens, which is a common low-context AI smell when names describe every step instead of the domain concept.

3. `weak_typing`
   Flags functions whose signatures use `any` or `interface{}`. The severity is reduced when the function includes type assertions that indicate at least some local narrowing.

4. `dropped_error`, `panic_on_error`, and `error_wrapping_misuse`
   Uses parser-level evidence to flag blank-identifier error drops, panic-or-fatal handling in `err != nil` branches, and `fmt.Errorf` calls that reference `err` without `%w` wrapping.

5. `comment_style_title_case` and `comment_style_tutorial`
   Flags doc comments that read like headings or tutorial narration instead of concise Go documentation.

6. `weak_crypto`
   Flags calls into weak standard-library crypto packages such as `crypto/md5`, `crypto/sha1`, `crypto/des`, and `crypto/rc4`.

7. `missing_context`, `sleep_polling`, `string_concat_in_loop`, and `goroutine_without_coordination`
   Flags a small conservative set of context-aware standard-library calls made from functions that do not accept `context.Context`, flags `time.Sleep` inside loops as a polling-style signal, flags obvious looped string concatenation, and flags raw goroutine launches when no obvious coordination signal is present.

8. `hallucinated_import_call` and `hallucinated_local_call`
   Uses the local package index to flag calls that appear to reference symbols not present in the scanned repository context. Import calls are matched against local package-plus-directory candidates derived from import paths, and ambiguous matches are handled conservatively. This is intentionally local-only and should be described as a heuristic, not as proof of broken code.

9. `todo_macro_leftover`, `unimplemented_macro_leftover`, `dbg_macro_leftover`, `panic_macro_leftover`, `unreachable_macro_leftover`, `todo_doc_comment_leftover`, `fixme_doc_comment_leftover`, `unwrap_in_non_test_code`, `expect_in_non_test_code`, `unsafe_without_safety_comment`, Rust-local `hallucinated_import_call`, and Rust-local `hallucinated_local_call`
   The Rust backend uses parser-level call, local-binding, doc-comment, test-classification, import-alias, and unsafe-comment evidence to flag obvious leftover macros, leftover TODO or FIXME doc comments, `.unwrap()` and `.expect(...)` in non-test Rust code, local imported calls that do not match indexed Rust modules, direct same-module calls that do not match indexed Rust symbols, and `unsafe` usage that lacks a nearby `SAFETY:` comment.

10. `blocking_sync_io_in_async`, `exception_swallowed`, `eval_exec_usage`, `print_debugging_leftover`, Python `full_dataset_load`, and Python `string_concat_in_loop`
   The Python heuristic layer uses parser-level async function kind, import aliases, call sites, exception-handler evidence, and loop-concatenation evidence to flag blocking sync I/O inside `async def`, broad exception handlers that immediately suppress failures, direct `eval()` and `exec()` usage, debug-style `print()` leftovers, full-file materialization patterns such as `open(...).read()` or `Path(...).read_text()`, and repeated string concatenation inside loops.

### Benchmark layer

`src/benchmark/mod.rs` runs repeated full scans and computes stage statistics:

- minimum
- maximum
- mean
- median

The benchmark command is meant to be run against a real local repository so the sub-2s target can be measured on realistic input rather than fixture-scale samples.

## Output model

The scan report currently includes:

- root path
- discovered and analyzed file counts
- function count
- findings by default, or per-file function fingerprints with `--details`
- structured findings
- index summary counts
- parse failures
- per-stage timing breakdown

The CLI continues to write reports to stdout. The intended workflow for saving results is shell redirection, for example `cargo run -- scan /absolute/path/to/repo > results.txt`.

The benchmark report includes:

- target path
- warmup count
- repeat count
- final file, function, and finding counts
- timing statistics for discover, parse, index, heuristics, and total stages
- raw per-run measurements

## Commands

### Scan a repository

```bash
cargo run -- scan /absolute/path/to/repo
```

The scan command auto-detects supported languages from the files present under the target root, so the same command works for Go-only repositories, Python-only repositories, Rust-only repositories, and mixed repositories.

This default output prints the summary plus findings only.

### Scan with JSON output

```bash
cargo run -- scan --json /absolute/path/to/repo
```

### Include full per-function details

```bash
cargo run -- scan --details /absolute/path/to/repo
cargo run -- scan --json --details /absolute/path/to/repo
```

### Write findings to a file

```bash
cargo run -- scan /absolute/path/to/repo > results.txt
cargo run -- scan --json /absolute/path/to/repo > results.txt
```

### Ignore `.gitignore` rules

```bash
cargo run -- scan --no-ignore /absolute/path/to/repo
```

### Benchmark a repository

```bash
cargo run -- bench /absolute/path/to/repo
```

### Benchmark with explicit run counts

```bash
cargo run -- bench --warmups 2 --repeats 5 /absolute/path/to/repo
```

### Benchmark with JSON output

```bash
cargo run -- bench --json /absolute/path/to/repo
```

## Benchmark baseline

Preferred local benchmark target: `gopdfsuit`

Measurement convention:

- command: `cargo run -- bench --warmups 2 --repeats 5 <local-gopdfsuit-path>`
- scan counts: discovered=89 analyzed=89 functions=702 findings=5 parse_failures=0
- index summary on the baseline scan: packages=28 symbols=853 imports=448
- total ms: min=174 max=194 mean=180.80 median=177.00
- parse ms: min=115 max=132 mean=122.40 median=120.00
- index ms: min=0 max=0 mean=0.00 median=0.00
- heuristics ms: min=45 max=50 mean=48.20 median=49.00

Interpretation notes:

- these numbers measure full-repository static analysis latency, not request latency inside the target application
- use the same warmup and repeat convention when comparing future runs
- if the target repository changes materially, refresh both the counts and timings together

Rust rollout convention:

- keep one repeatable Rust-only benchmark target in addition to the Go baseline, using the same `cargo run -- bench --warmups 2 --repeats 5 <path>` command shape
- record discovered files, analyzed files, functions, findings, parse failures, and stage timings together when refreshing a Rust benchmark note
- keep at least one mixed-language verification workspace in the integration suite so benchmark or index changes do not silently reintroduce Go/Rust symbol bleed

## Detailed plan for the next extension phase

### 1. Strengthen the heuristic layer

The current heuristic layer is intentionally shallow and explainable. The next pass should preserve that property while making the signals more context-aware.

Work items:

- add a scored rule result model so individual findings can contribute to an aggregate slop score without hiding the underlying reasons
- separate generic-name token matching from evidence thresholds so false-positive tuning is easier
- capture argument shapes and local declarations from the parser so weak typing can distinguish boundary interfaces from vague internal plumbing
- add negative fixtures for common legitimate names such as server handlers, formatters, and adapters with strong concrete typing
- keep standard boilerplate suppression centralized so future rules do not over-penalize canonical Go error handling

### 2. Build a stronger package and symbol index

The current index is keyed primarily by package name and receiver type. That is enough for early local-context checks, but it needs more structure before hallucination detection can become reliable.

Work items:

- group packages by both package name and directory to avoid conflating unrelated packages with identical names
- record import aliases, import paths, and directory origins so selector-call resolution can be traced more honestly
- index exported and unexported symbols separately so findings can express whether a missing symbol would be visible cross-package even if it exists elsewhere
- capture method sets by receiver type and normalize pointer/value receivers consistently
- add a cheap unresolved-reference staging layer so suspicious call sites can be recorded first and classified second

### 3. Expand the hallucination check using local project context

The next hallucination pass should remain explicit about being local and syntactic, not full semantic truth.

Work items:

- verify unresolved same-package direct calls against package-plus-directory symbol tables
- verify selector calls against locally indexed imported packages when the import path can be mapped back into the scanned repository
- add better evidence payloads that include the import alias, candidate package, and missing symbol name
- distinguish hard local misses from ambiguous cases where multiple packages share the same name
- defer `go.mod`, `go list`, and `go/types` integration until the local index behavior and performance are stable

### 4. Add a more rigorous benchmark baseline

The current benchmark command reports repeated end-to-end timings. The next step is to make those numbers useful for regression tracking.

Work items:

- add optional CSV or line-delimited JSON output for benchmark automation
- separate cold-cache and warm-cache conventions explicitly in the benchmark docs and command help
- record repository scale metadata such as discovered files, analyzed files, function count, and findings count for each run
- benchmark at least one real medium-size Go repository and capture the baseline in documentation or repository memory
- add a simple regression threshold check in CI later, but only after the benchmark output is stable and noise is understood

### 5. Keep documentation aligned with the executable surface

The README should stay short and operational. The guide should remain the authoritative place for architecture, heuristics, limitations, and roadmap detail.

Work items:

- update the guide whenever the report schema changes materially
- document known false-positive cases for each heuristic family
- add a benchmark interpretation section once real baseline numbers exist
- add a local-context versus authoritative-type-checking comparison section before introducing `go/types`

## Limitations

- The symbol index is local and tree-sitter-derived. It cannot replace authoritative Go type checking.
- Import resolution is currently local and suffix-based against repository directories. It does not resolve module paths authoritatively through `go.mod`.
- Hallucination findings are conservative heuristics for repository-local context, not a proof that a program fails to compile.
- The benchmark command measures the current Rust pipeline only. It does not run `go test`, `go vet`, or `go/types`.
- The weak-typing rule currently focuses on signature-level vagueness rather than full dataflow-level type misuse.

## Testing strategy

The repository uses small fixture-driven tests under `tests/fixtures/` and integration coverage in `tests/integration_scan.rs`. The current suite verifies:

- `.gitignore` handling
- generated-file filtering
- syntax-error tolerance
- heuristic triggering for generic naming and weak typing
- negative cases for legitimate handler and adapter naming
- local import-call hallucination detection
- package-plus-directory import resolution when multiple local packages share the same package name
- benchmark command behavior on a temporary Go workspace

As the heuristic set expands, each new rule should land with at least one positive fixture and one negative fixture to keep false-positive drift visible.
