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

- [ ] Expand the detection surface (next-development tasks).
  - [x] Alias handling tests: ensure detection works when `context` is imported with an alias (e.g. `c "context"`).
  - [ ] Method receiver wrappers: test methods like `func (s *Srv) Fetch(ctx context.Context, ...)` that call contextless APIs through fields (e.g. `s.client.Get`).
  - [ ] Nested wrappers: detect when wrapper A accepts context and calls wrapper B without passing context (B might call context-free APIs).
  - [ ] Database APIs: expand to detect `Query` vs `QueryContext` style mismatches (leverage `is_db_query()` in `common.rs`).
  - [ ] Asynchronous work: warn if a function accepts a `ctx` but starts goroutines that ignore shutdown signals (already partially handled by `unmanaged_goroutines` heuristics; add cross-check).

- [ ] Reduce false positives / strengthen evidence.
  - [x] Exclude test files and known generator files (parser already tracks `is_test_file`).
  - [ ] Consider requiring both: function has context parameter AND caller of the function is in a public API surface (optional).
  - [ ] Avoid flagging code that intentionally uses `context.Background()` for top-level producers (detect common patterns: comments, named function docstrings).
  - [ ] Add heuristics to recognise explicit wrappers that intentionally decouple context (e.g., small adapters with documented reason strings).

- [ ] Tests: increase coverage with text fixtures and expected findings mapping.
  - For each fixture add an entry describing the expected findings (rule_id, severity, line):
    - context_wrapper_slop.txt
      - expected: `missing_context_propagation` on `http.Get` call line(s);
      - expected: `context_background_used` where `context.Background()` is used to construct request.
    - context_wrapper_clean.txt
      - expected: no findings for propagation rules.
  - [ ] Add additional fixtures for nested wrappers, methods, DB calls.
  - How to author fixtures: place Go code snippets as text files under `tests/fixtures/go/` (no compiled test code).

- [ ] Integration and CI
  - [x] Add an integration entry (if not already present) that runs the scanner against the `tests/fixtures/go/` set and asserts expected findings. Use the repo's existing test harness pattern (see `tests/integration_scan.rs`) to map fixture files to expected findings (the repo currently uses textual fixtures for other heuristics).
  - [x] Run full scan and record baseline findings count and performance metrics.

- [ ] Documentation and developer guidance
  - [x] Update `docs/` or `guides/go/` with a short section explaining the new rule(s), their rationale, and examples (move this high-level content into `guides/go/planned_improvements/` once stabilized).
  - [ ] Add a short note in `SECURITY.md` or `RELEASE.md` if the rule impacts API stability or security posture.

- [ ] Rollout and tuning
  - [x] Start with `Severity::Warning` and monitor false positive rate on several repositories.
  - [ ] Optionally provide an opt-in severity override to promote to `Error` for stricter projects.

- [ ] Acceptance criteria
  - [x] Detects the simple wrapper mistakes (Background/TODO and common stdlib calls) with high precision on the fixtures.
  - [x] No new failures in existing unit tests beyond expected new findings in the newly-added fixtures.
  - [x] Performance impact on heuristic evaluation remains negligible (measure via existing benchmarking harness).

**Implementation artifacts created during this pass**

- [src/heuristics/context.rs](src/heuristics/context.rs) — added `propagate_findings`.
- [src/heuristics/mod.rs](src/heuristics/mod.rs) — registered the new heuristic.
- [tests/fixtures/go/context_wrapper_slop.txt](tests/fixtures/go/context_wrapper_slop.txt) — failing examples.
- [tests/fixtures/go/context_wrapper_clean.txt](tests/fixtures/go/context_wrapper_clean.txt) — passing examples.
- [tests/fixtures/go/context_wrapper_alias_slop.txt](tests/fixtures/go/context_wrapper_alias_slop.txt) — alias-import propagation example.

---

Next immediate steps (short):

- [ ] Add nested-wrapper and receiver-wrapper fixtures and iterate on false-positive rules.
- [ ] Expand wrapper propagation into DB `Query` versus `QueryContext` mismatches.
- [ ] Decide whether stricter projects want a severity override or opt-in gate for wrapper propagation rules.
