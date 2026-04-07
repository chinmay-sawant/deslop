# Plan 3 — Better context propagation through wrappers

Purpose: reduce incorrect use of global or context-free APIs inside Go wrappers that accept a `context.Context` but fail to pass it through to downstream calls. This checklist captures analysis, implementation, tests, rollout, and follow-ups.

- [x] Repository analysis: confirm existing context heuristics and parser evidence exist.
  - Notes: the repo contains Go context heuristics in [src/heuristics/context.rs](src/heuristics/context.rs) and parser helpers in [src/analysis/go/parser/context.rs](src/analysis/go/parser/context.rs).

- [x] Minimal heuristic implemented in Rust to catch obvious wrapper issues.
  - Implementation summary: new `propagate_findings` in [src/heuristics/context.rs](src/heuristics/context.rs) that:
    - requires `ParsedFunction.has_context_parameter == true`;
    - flags uses of `context.Background()` or `context.TODO()` inside functions that accept a context (`context_background_used`);
    - flags calls to well-known context-free stdlib APIs (e.g., `http.Get`, `net.Dial`, `os/exec.Command`) from functions that accept a context (`missing_context_propagation`);
    - uses `import_alias_lookup` and `function.calls` + `function.body_text` as parser evidence.
  - Registration: heuristic was wired into the Go pipeline in [src/heuristics/mod.rs](src/heuristics/mod.rs).

- [x] Add minimal textual test fixtures verifying behaviour.
  - Fixtures added:
    - [tests/fixtures/go/context_wrapper_slop.txt](tests/fixtures/go/context_wrapper_slop.txt) — contains wrappers that accept `ctx` but call `http.Get` and `context.Background()`.
    - [tests/fixtures/go/context_wrapper_clean.txt](tests/fixtures/go/context_wrapper_clean.txt) — shows correct propagation (`NewRequestWithContext(ctx, ...)` and forwarding `ctx`).
    - [tests/fixtures/go/context_wrapper_alias_slop.txt](tests/fixtures/go/context_wrapper_alias_slop.txt) — verifies alias imports still trigger propagation findings.
    - [tests/fixtures/go/context_receiver_wrapper_slop.txt](tests/fixtures/go/context_receiver_wrapper_slop.txt) — verifies receiver-field wrappers such as `s.client.Get(...)`.
    - [tests/fixtures/go/context_nested_wrapper_slop.txt](tests/fixtures/go/context_nested_wrapper_slop.txt) — verifies local wrapper-chain propagation.
    - [tests/fixtures/go/context_db_query_slop.txt](tests/fixtures/go/context_db_query_slop.txt) — verifies `Query` versus `QueryContext` style mismatches.
    - [tests/fixtures/go/context_documented_detach_clean.txt](tests/fixtures/go/context_documented_detach_clean.txt) — verifies documented detached-context adapters stay quiet.

- [x] Expand the detection surface (next-development tasks).
  - [x] Alias handling tests: ensure detection works when `context` is imported with an alias (e.g. `c "context"`).
  - [x] Method receiver wrappers: test methods like `func (s *Srv) Fetch(ctx context.Context, ...)` that call contextless APIs through fields (e.g. `s.client.Get`).
  - [x] Nested wrappers: detect when wrapper A accepts context and calls wrapper B without passing context through the wrapper chain.
  - [x] Database APIs: expand to detect `Query` vs `QueryContext` style mismatches (using `db_query_calls` and query-method classification).
  - [x] Asynchronous work: strengthen `goroutine_without_shutdown_path` evidence when the parent function already accepts `context.Context`.

- [x] Reduce false positives / strengthen evidence.
  - [x] Exclude test files and known generator files (parser already tracks `is_test_file`).
  - [x] Avoid flagging code that intentionally uses `context.Background()` for top-level producers when the function documents an intentionally detached context boundary.
  - [x] Add heuristics to recognise explicit wrappers that intentionally decouple context (small adapters with documented reason strings).
  - Public-API gating remains deferred. The current pass stays repository-local and conservative rather than trying to infer API surface exposure.

- [x] Tests: increase coverage with text fixtures and expected findings mapping.
  - For each fixture add an entry describing the expected findings (rule_id, severity, line):
    - context_wrapper_slop.txt
      - expected: `missing_context_propagation` on `http.Get` call line(s);
      - expected: `context_background_used` where `context.Background()` is used to construct request.
    - context_wrapper_clean.txt
      - expected: no findings for propagation rules.
    - context_receiver_wrapper_slop.txt
      - expected: `missing_context_propagation` on the receiver-field call line.
    - context_nested_wrapper_slop.txt
      - expected: `missing_context_propagation` on the local wrapper-chain call line.
    - context_db_query_slop.txt
      - expected: `missing_context_propagation` on the `Query(...)` call line.
    - context_documented_detach_clean.txt
      - expected: no `missing_context_propagation` or `context_background_used` findings because the detach is documented.
  - [x] Add additional fixtures for nested wrappers, methods, DB calls, and documented detach cases.
  - How to author fixtures: place Go code snippets as text files under `tests/fixtures/go/` (no compiled test code).

- [x] Integration and CI
  - [x] Add an integration entry (if not already present) that runs the scanner against the `tests/fixtures/go/` set and asserts expected findings. Use the repo's existing test harness pattern (see `tests/integration_scan.rs`) to map fixture files to expected findings (the repo currently uses textual fixtures for other heuristics).
  - [x] Run full local validation (`cargo test`) and representative quick-runs against `gopdfsuit` and `SnapBack` to confirm the new Go wrapper rules do not appear as unrelated noise there.

- [x] Documentation and developer guidance
  - [x] Update `docs/` or `guides/go/` with a short section explaining the new rule(s), their rationale, and examples (move this high-level content into `guides/go/planned_improvements/` once stabilized).
  - [x] Add a short note in `SECURITY.md` explaining how repositories can promote `missing_context_propagation` when dropped context is treated as a stricter boundary.

- [x] Rollout and tuning
  - [x] Start with `Severity::Warning` and monitor false positive rate on several repositories.
  - [x] Provide a stricter-project path through generic `severity_overrides`; integration coverage now verifies `missing_context_propagation = "error"` works as expected.

- [x] Acceptance criteria
  - [x] Detects the simple wrapper mistakes (Background/TODO and common stdlib calls) with high precision on the fixtures.
  - [x] No new failures in existing unit tests beyond expected new findings in the newly-added fixtures.
  - [x] Performance impact on heuristic evaluation remains negligible (measure via existing benchmarking harness).

**Implementation artifacts created during this pass**

- [src/heuristics/context.rs](src/heuristics/context.rs) — added `propagate_findings`.
- [src/heuristics/mod.rs](src/heuristics/mod.rs) — registered the new heuristic.
- [tests/fixtures/go/context_wrapper_slop.txt](tests/fixtures/go/context_wrapper_slop.txt) — failing examples.
- [tests/fixtures/go/context_wrapper_clean.txt](tests/fixtures/go/context_wrapper_clean.txt) — passing examples.
- [tests/fixtures/go/context_wrapper_alias_slop.txt](tests/fixtures/go/context_wrapper_alias_slop.txt) — alias-import propagation example.
- [tests/fixtures/go/context_receiver_wrapper_slop.txt](tests/fixtures/go/context_receiver_wrapper_slop.txt) — receiver-field wrapper propagation example.
- [tests/fixtures/go/context_nested_wrapper_slop.txt](tests/fixtures/go/context_nested_wrapper_slop.txt) — local wrapper-chain propagation example.
- [tests/fixtures/go/context_db_query_slop.txt](tests/fixtures/go/context_db_query_slop.txt) — DB context propagation example.
- [tests/fixtures/go/context_documented_detach_clean.txt](tests/fixtures/go/context_documented_detach_clean.txt) — documented detach exemption example.

---

Future follow-ups (kept outside the shipped checklist):

- Public-API-aware propagation remains a future refinement if repository-local conservative coverage proves too noisy or too weak.
- Cross-package wrapper reasoning beyond the current local package index can be layered on later if needed.
