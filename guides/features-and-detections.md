# goslop Features and Detections

## Purpose

goslop is a static analyzer for Go repositories that looks for signals commonly associated with low-context or AI-assisted code. The goal is not to prove correctness. The goal is to surface suspicious patterns quickly, explain why they were flagged, and let a reviewer decide whether the code is actually a problem.

## Current feature set

### Scan modes

- `cargo run -- scan <path>` prints a compact summary plus findings.
- `cargo run -- scan --details <path>` prints the full per-file and per-function breakdown.
- `cargo run -- scan --json <path>` prints structured JSON.
- `cargo run -- bench <path>` benchmarks the end-to-end pipeline.

### Repository handling

- Walks a repository with `.gitignore` awareness by default.
- Skips `vendor/` and generated Go files.
- Parses Go syntax with `tree-sitter-go`.
- Continues scanning even when some files contain syntax errors.

### Analysis pipeline

- Extracts package names, imports, declared symbols, call sites, and function fingerprints.
- Builds a repository-local index keyed by package plus directory.
- Runs explainable heuristics that emit rule IDs, messages, and evidence.
- Produces compact text output by default, or detailed text/JSON with `--details`.

## What goslop detects today

### Naming and abstraction signals

- `generic_name`: function names that are overly generic without stronger contextual signals.
- `overlong_name`: very long identifiers with too many descriptive tokens.

### Typing signals

- `weak_typing`: signatures that rely on `any` or `interface{}`.

### Error-handling signals

- `dropped_error`: blank identifier assignments that discard an `err`-like value.
- `panic_on_error`: `err != nil` branches that jump straight to `panic` or `log.Fatal` style exits.
- `error_wrapping_misuse`: `fmt.Errorf` calls that reference `err` without `%w`.

### Comment-style signals

- `comment_style_title_case`: heading-like Title Case doc comments.
- `comment_style_tutorial`: tutorial-style comments that narrate obvious implementation steps.

### Security signals

- `weak_crypto`: direct use of weak standard-library crypto packages such as `crypto/md5`, `crypto/sha1`, `crypto/des`, and `crypto/rc4`.

### Context and blocking signals

- `missing_context`: obvious standard-library context-aware calls such as `http.Get`, `http.NewRequest`, `exec.Command`, or `net.Dial` made from functions that do not accept `context.Context`.
- `sleep_polling`: `time.Sleep` inside loops, which often indicates polling or busy-wait style code.

### Performance signals

- `string_concat_in_loop`: repeated string concatenation inside loops when the function is clearly building a string value incrementally.

### Concurrency signals

- `goroutine_without_coordination`: raw `go` statements where goslop cannot find an obvious context or WaitGroup-like coordination signal in the same function.

### Local hallucination signals

- `hallucinated_import_call`: package-qualified calls that do not match locally indexed symbols for the imported package.
- `hallucinated_local_call`: same-package calls to symbols that are not present in the scanned local package context.

## Detection philosophy

- Findings are heuristics, not compile-time proof.
- The analyzer is intentionally conservative where full type information is missing.
- Rules are designed to produce readable evidence so humans can validate them quickly.
- Local repository context is used where possible, but goslop does not replace `go/types`.

## Current limitations

- No authoritative Go type checking yet.
- No interprocedural context propagation.
- No proof of goroutine leaks, N+1 queries, or runtime performance regressions.
- Package-method and local-symbol checks are repository-local only.

## Phase status

### Implemented so far

- Phase 1 rule pack: naming, weak typing, comment style, weak crypto, early error-handling checks, and local hallucination checks.
- Phase 2 parser enrichment: context-parameter detection, raw goroutine launch tracking, looped `time.Sleep` detection, and string-concatenation-in-loop tracking.
- Phase 2 heuristic additions: broader `missing_context`, `sleep_polling`, `string_concat_in_loop`, and the first conservative goroutine-coordination pass.

### Still pending

- Stronger repo-wide style checks.
- More reliable concurrency signals.
- Better context propagation through wrappers and helper functions.
- Optional deeper semantic analysis for harder cases.