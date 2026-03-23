# deslop Features and Detections

## Purpose

deslop is a static analyzer for Go, Python, and Rust repositories that looks for signals commonly associated with low-context or AI-assisted code. The goal is not to prove correctness. The goal is to surface suspicious patterns quickly, explain why they were flagged, and let a reviewer decide whether the code is actually a problem.

## Current feature set

### Scan modes

- `cargo run -- scan <path>` prints a compact summary plus findings.
- `cargo run -- scan --details <path>` prints the full per-file and per-function breakdown.
- `cargo run -- scan --json <path>` prints structured JSON.
- `cargo run -- bench <path>` benchmarks the end-to-end pipeline.

### Repository handling

- Walks a repository with `.gitignore` awareness by default.
- Skips `vendor/` and generated Go files.
- Parses Go syntax with `tree-sitter-go`, Python syntax with `tree-sitter-python`, and Rust syntax with `tree-sitter-rust`.
- Continues scanning even when some files contain syntax errors.

### Analysis pipeline

- Extracts package names, imports, declared symbols, call sites, and function fingerprints.
- Builds a repository-local index keyed by package plus directory.
- Runs explainable heuristics that emit rule IDs, messages, and evidence.
- Produces compact text output by default, or detailed text/JSON with `--details`.

## What deslop detects today

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
- `hardcoded_secret`: secret-like identifiers assigned direct string literals instead of environment or secret-manager lookups.
- `sql_string_concat`: query execution calls where SQL is constructed dynamically with concatenation or `fmt.Sprintf`.

### Rust-specific signals

- `todo_macro_leftover`: `todo!()` left in non-test Rust code.
- `unimplemented_macro_leftover`: `unimplemented!()` left in non-test Rust code.
- `dbg_macro_leftover`: `dbg!()` left in non-test Rust code.
- `panic_macro_leftover`: `panic!()` left in non-test Rust code.
- `unreachable_macro_leftover`: `unreachable!()` left in non-test Rust code.
- `unwrap_in_non_test_code`: `.unwrap()` used in non-test Rust code.
- `expect_in_non_test_code`: `.expect(...)` used in non-test Rust code.
- `unsafe_without_safety_comment`: `unsafe fn` or `unsafe` block without a nearby `SAFETY:` comment. The current nearby-comment policy accepts a `SAFETY:` comment on the same line or within the previous two lines.

### Python-specific signals

- `blocking_sync_io_in_async`: obvious synchronous network, subprocess, sleep, or file I/O calls made from `async def` functions.
- `eval_exec_usage`: direct `eval()` or `exec()` usage in non-test Python code.
- `print_debugging_leftover`: `print()` calls left in non-test Python functions when they do not look like obvious `main`-entrypoint output.

Python also reuses shared signals when the parser evidence supports them, including `hardcoded_secret`, comment-style findings based on docstrings, `full_dataset_load`, and `string_concat_in_loop` when those patterns are syntactically clear.

### Consistency and tag signals

- `mixed_receiver_kinds`: methods on the same receiver type mix pointer and value receivers.
- `malformed_struct_tag`: struct field tags that do not parse as valid Go tag key/value pairs.
- `duplicate_struct_tag_key`: struct field tags that repeat the same key more than once.

### Context and blocking signals

- `missing_context`: obvious standard-library context-aware calls such as `http.Get`, `http.NewRequest`, `exec.Command`, or `net.Dial` made from functions that do not accept `context.Context`.
- `missing_cancel_call`: derived contexts created with `context.WithCancel`, `context.WithTimeout`, or `context.WithDeadline` where deslop cannot find a local `cancel()` or `defer cancel()` call.
- `sleep_polling`: `time.Sleep` inside loops, which often indicates polling or busy-wait style code.
- `busy_waiting`: `select { default: ... }` inside loops, which often spins instead of blocking on a channel, timer, or context.

### Performance signals

- `string_concat_in_loop`: repeated string concatenation inside loops when the function is clearly building a string value incrementally.
- `repeated_json_marshaling`: `encoding/json.Marshal` or `MarshalIndent` inside loops, which can turn iterative paths into repeated allocation and serialization hot spots.
- `allocation_churn_in_loop`: obvious `make`, `new`, or buffer-construction calls inside loops.
- `fmt_hot_path`: `fmt` formatting calls such as `Sprintf` inside loops.
- `reflection_hot_path`: `reflect` package calls inside loops.
- `full_dataset_load`: calls such as `io.ReadAll`, `ioutil.ReadAll`, or `os.ReadFile` that load an entire payload into memory instead of streaming it.

### Concurrency signals

- `goroutine_without_coordination`: raw `go` statements where deslop cannot find an obvious context or WaitGroup-like coordination signal in the same function.
- `goroutine_spawn_in_loop`: raw `go` statements launched from inside loops without an obvious context or WaitGroup-like coordination signal.
- `goroutine_without_shutdown_path`: looping goroutine literals that do not show an obvious `ctx.Done()` or done-channel shutdown path.
- `mutex_in_loop`: repeated `Lock` or `RLock` acquisition inside loops.
- `blocking_call_while_locked`: potentially blocking calls observed between `Lock` and `Unlock`.

### Data-access signals

- `n_plus_one_query`: database-style query calls issued inside loops.
- `wide_select_query`: literal `SELECT *` query shapes.
- `likely_unindexed_query`: query shapes such as leading-wildcard `LIKE` or `ORDER BY` without `LIMIT` that often scale poorly.

### Local hallucination signals

- `hallucinated_import_call`: package-qualified calls that do not match locally indexed symbols for the imported package.
- `hallucinated_local_call`: same-package calls to symbols that are not present in the scanned local package context.

For Rust, `hallucinated_import_call` currently covers conservative local-module imports built from `crate::`, `self::`, and `super::` paths when deslop can map them back to locally indexed Rust modules, plus direct calls through locally imported Rust function aliases.

For Rust, `hallucinated_local_call` now also covers direct same-module calls when the callee name is not locally bound and does not exist in the indexed Rust module.

### Rust-specific comment-leftover signals

- `todo_doc_comment_leftover`: Rust doc comments that still contain a `TODO` marker in non-test code.
- `fixme_doc_comment_leftover`: Rust doc comments that still contain a `FIXME` marker in non-test code.

### Test-quality signals

- `test_without_assertion_signal`: tests that call production code without any obvious assertion or failure signal.
- `happy_path_only_test`: tests that assert success expectations without any obvious negative-path signal.
- `placeholder_test_body`: tests that look skipped, TODO-shaped, or otherwise placeholder-like rather than validating behavior.

## Detection philosophy

- Findings are heuristics, not compile-time proof.
- The analyzer is intentionally conservative where full type information is missing.
- Rules are designed to produce readable evidence so humans can validate them quickly.
- Local repository context is used where possible, but deslop does not replace `go/types`.

## Current limitations

- No authoritative Go or Rust type checking yet.
- No interprocedural context propagation.
- No proof of goroutine leaks, N+1 queries, or runtime performance regressions.
- Package-method and local-symbol checks are repository-local and now language-scoped for mixed-language repositories.
- No Rust trait resolution, cargo workspace modeling, or macro expansion yet.

## Phase status

### Implemented so far

- Phase 1 rule pack: naming, weak typing, comment style, weak crypto, early error-handling checks, and local hallucination checks.
- Phase 2 parser enrichment: context-parameter detection, derived-context factory tracking, raw goroutine launch tracking, goroutine-in-loop tracking, goroutine shutdown-path tracking, looped `time.Sleep` detection, looped `select default` detection, looped JSON marshal detection, mutex lock-in-loop tracking, allocation tracking, fmt and reflect hot-path tracking, looped database query extraction, and string-concatenation-in-loop tracking.
- Phase 2 heuristic additions: broader `missing_context`, `missing_cancel_call`, `sleep_polling`, `busy_waiting`, `repeated_json_marshaling`, `string_concat_in_loop`, `goroutine_spawn_in_loop`, `goroutine_without_shutdown_path`, `mutex_in_loop`, `blocking_call_while_locked`, `allocation_churn_in_loop`, `fmt_hot_path`, `reflection_hot_path`, `full_dataset_load`, `n_plus_one_query`, `wide_select_query`, `likely_unindexed_query`, and the first conservative goroutine-coordination pass.
- Phase 3 heuristic additions: `hardcoded_secret`, `sql_string_concat`, `mixed_receiver_kinds`, `malformed_struct_tag`, `duplicate_struct_tag_key`, `test_without_assertion_signal`, `happy_path_only_test`, and `placeholder_test_body`.
- Rust heuristic additions so far: `todo_macro_leftover`, `unimplemented_macro_leftover`, `dbg_macro_leftover`, `panic_macro_leftover`, `unreachable_macro_leftover`, `todo_doc_comment_leftover`, `fixme_doc_comment_leftover`, `unwrap_in_non_test_code`, `expect_in_non_test_code`, `unsafe_without_safety_comment`, and Rust-local `hallucinated_import_call` coverage for `crate::`, `self::`, and `super::` module imports.

### Still pending

- Stronger repo-wide style checks.
- Deeper goroutine lifetime analysis beyond local shutdown-path heuristics.
- Better context propagation through wrappers and helper functions.
- Optional deeper semantic analysis for harder cases such as true index awareness, struct layout analysis, and O(n²) detection.