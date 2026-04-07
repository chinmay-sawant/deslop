# Plan 4 — Optional deeper semantic analysis (Go) — Checklist

Date: 2026-03-29

Summary
-------
This checklist captures a conservative, opt-in roadmap to add "deeper semantic" detections for Go. The immediate focus (Phase 1) is to implement parser-driven, low-risk heuristics that can be implemented purely inside the Rust heuristics layer (no Go files added). Later phases describe index- and type-powered expansions.

Scope & constraints
-------------------
- Only modify Rust files under `src/heuristics/*` and related wiring in `src/heuristics/mod.rs` and `src/index/*` if strictly required.
- Do NOT add or change any `.go` source files — tests use text fixtures in `tests/fixtures/go/`.
- Tests must be text fixtures (`.txt`) in `tests/fixtures/go/`. Integration test harness lives in `tests/integration_scan.rs` and should continue to exercise fixtures by writing them to a temp workspace.
- Keep Phase 1 conservative and explainable: findings must include clear evidence and an explicit statement of uncertainty where appropriate.

Checklist (Phase 0 — plan file)
--------------------------------
- [x] Create this plan file at `guides/go/planned_improvements/plan4.md`.

Checklist (Phase 1 — parser-driven conservative heuristics)
---------------------------------------------------------
- [x] Search repository for existing semantic or nested-loop heuristics and collect findings.
  - Result: existing loop-local signals already lived in `src/heuristics/performance.rs` (`allocation_churn_in_loop`, `string_concat_in_loop`, `n_plus_one_query`) and the Go parser already exposed `alloc_loops`, `concat_loops`, `db_query_calls`, and `body_text`, so phase 1 could stay parser-driven and repository-local.
- [x] Add `likely_n_squared_allocation` heuristic in `src/heuristics/performance.rs`
  - Detection strategy (parser-driven, conservative):
    - Use `ParsedFunction.alloc_loops` (lines where allocations were detected inside loops) and `ParsedFunction.body_text` to infer loop nesting.
    - Consider a function "likely O(n²)" when:
      - There exist allocation(s) inside a loop (alloc_loops non-empty), AND
      - The function contains nested loop structure detected by a conservative body-text scan.
  - Severity: `Warning` (but evidence must describe uncertainty and mitigation suggestions).
  - Evidence: line numbers for inner-loop allocation, small snippet of surrounding lines, and a short note why this is uncertain (e.g., "append may be preallocated; verify capacity or use preallocation").
- [x] Add `likely_n_squared_string_concat` heuristic in `src/heuristics/performance.rs`
  - Detection strategy:
    - Use `ParsedFunction.concat_loops` (lines where `+` or `fmt`-style concatenation was found inside loops) and `body_text` to detect nested loops similarly to allocation detection.
    - Mark `Info` severity when inner-loop concatenation present without evidence of `strings.Builder` or pre-sized builder usage (scan `body_text` for `strings.Builder`, `bytes.Buffer`, or `Builder` allocations keyed to the same variable scope).
  - Evidence: lines, snippet, and suggestion to use `strings.Builder`, `bytes.Buffer`, or pre-allocated buffer.
- [x] Expand the existing `n_plus_one_query` family by correlating `db_query_calls` with nested-loop signals.
  - Detection strategy:
    - If `db_query_calls` contains a call marked `in_loop == true` and the function also shows nested-loop signal, raise severity and add evidence that N+1 is more likely (but still conservative; require both signals).
  - Acceptance: positive fixtures show higher-severity finding when query + nested loops present.

Checklist (Wiring)
------------------
- [x] Export new heuristics from `src/heuristics/performance.rs` and add `n_squared_findings`.
- [x] Wire the new findings into `src/heuristics/mod.rs` so `evaluate_go_file` invokes them for each function.
- [x] Add unit-level doc comments on each new heuristic function describing inputs consumed from `ParsedFunction` and limits of the detection.

Checklist (Testing — fixtures only)
----------------------------------
Note: test harnesses in this repository expect fixture Go code as `.txt` files under `tests/fixtures/go/`. Phase 1 tests should add fixtures only; minimal Rust test harness changes are allowed only to reference new fixtures where necessary.
- [x] Add positive fixtures (text files) under `tests/fixtures/go/`:
  - `n_squared_alloc_slop.txt` — nested loops with `append` or `make`+`append` in inner loop, no preallocation.
  - `n_squared_concat_slop.txt` — nested loops with string `+=` concatenation in inner loop and no `strings.Builder` used.
  - `n_squared_query_slop.txt` — nested loops that perform `db` calls inside inner loop.
- [x] Add negative fixtures (text files) under `tests/fixtures/go/`:
  - `n_squared_alloc_clean.txt` — equivalent logic but with proper preallocation (`make(..., cap)`) or single-pass accumulation with precomputed total capacity.
  - `n_squared_concat_clean.txt` — uses `strings.Builder` or `bytes.Buffer` for concatenation.
  - `n_squared_query_clean.txt` — uses preloading (bulk query) or moves the query outside the inner loop.
- [x] Add short textual guidance at the top of each fixture describing why it should trigger or not trigger the rule (a single comment line is fine).

Checklist (Testing — Rust harness)
---------------------------------
- [x] Update or add integration test assertions (Rust) to exercise the fixtures through `scan_repository`.
  - Keep fixture content as `.txt` files; test code should remain minimal and follow existing patterns in `tests/integration_scan.rs`.
  - Acceptance: test asserts `finding.rule_id` presence for slop fixtures and absence for clean fixtures. Keep the rust test logic small and re-use existing helpers.

Checklist (Acceptance criteria)
------------------------------
- [x] Positive fixtures produce at least one `Finding` with the expected `rule_id` for the new heuristics.
- [x] Negative fixtures do not produce findings for the new heuristics.
- [x] Findings include clear evidence: rule id, function name, start/end line, short message, and evidence bullet lines explaining why the rule fired and possible mitigations.
- [x] Each new heuristic function has clear doc comments and a short unit-level test for the internal line-to-loop heuristics.

Checklist (Rollout & opt-in)
---------------------------
- [x] Add a CLI flag / config toggle to enable deeper semantic checks (`--enable-semantic` and `go_semantic_experimental = true` in `.deslop.toml`).
- [x] Default behaviour remains unchanged (semantic checks are disabled by default).
- [x] Add explicit docs in `guides/go/planned_improvements/plan4.md` (this file) explaining opt-in and recommended rollout sequence.

Documentation completed in this pass:

- [x] Add a short changelog-style entry in `README.md` describing the new heuristics and the opt-in flag.
- [x] Add a short entry in `guides/features-and-detections.md` and the frontend docs describing the rationale and how to enable semantic checks.
- Example validation captured in-repo: semantic fixtures now show `likely_n_squared_allocation`, `likely_n_squared_string_concat`, and stronger `n_plus_one_query` severity only when the opt-in toggle is enabled.

Future phases (kept as deferred notes rather than active checklist items):

- Phase 2: index-assisted rules that use `RepositoryIndex` to disambiguate expensive library calls from cheap helpers.
- Phase 3: full type-aware dataflow and inter-procedural analysis through `go/packages`, LSP data, or a dedicated type index.

Risks & mitigations
-------------------
- Risk: false positives where inner-loop allocation is intentionally amortized or preallocated.
  - Mitigation: require multiple signals (allocation + nested loops OR repeated len traversal) and provide conservative severities.
- Risk: DB index heuristics may be wrong without schema knowledge.
  - Mitigation: keep DB index findings informational unless additional schema or repository metadata confirms index absence.

Owner & timeline (suggested)
---------------------------
- Owner: `@owner` (replace with team owner) — implement Phase 1 in a single PR.
- Suggested ETA: 2–4 business days for Phase 1 (heuristics + fixtures + wiring + tests), longer if Phase 2 is attempted.

Completed verification for this pass:

- Focused repository search completed against the existing heuristics and parser evidence.
- Fixture list implemented and covered by Rust integration tests.
- Opt-in semantic configuration documented in README, guides, frontend docs, and the GitHub Action interface.
