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
- [ ] Search repository for existing semantic or nested-loop heuristics and collect findings
  - Acceptance: annotated grep output saved to PR or issue describing hits and gaps.
- [ ] Add `likely_n_squared_allocation` heuristic in `src/heuristics/performance.rs`
  - Detection strategy (parser-driven, conservative):
    - Use `ParsedFunction.alloc_loops` (lines where allocations were detected inside loops) and `ParsedFunction.body_text` to infer loop nesting.
    - Consider a function "likely O(n²)" when:
      - There exist allocation(s) inside a loop (alloc_loops non-empty), AND
      - The function contains at least two loop constructs where the allocation line is inside an inner loop region (heuristic string-scan or coarse AST-path matching), OR
      - There is an allocation inside a loop and there also exists another loop-iteration site referencing the same collection (e.g., repeated traversal of the same slice variable name) in the same function body.
  - Severity: `Warning` (but evidence must describe uncertainty and mitigation suggestions).
  - Evidence: line numbers for inner-loop allocation, small snippet of surrounding lines, and a short note why this is uncertain (e.g., "append may be preallocated; verify capacity or use preallocation").
- [ ] Add `likely_n_squared_string_concat` heuristic in `src/heuristics/performance.rs`
  - Detection strategy:
    - Use `ParsedFunction.concat_loops` (lines where `+` or `fmt`-style concatenation was found inside loops) and `body_text` to detect nested loops similarly to allocation detection.
    - Mark `Info` severity when inner-loop concatenation present without evidence of `strings.Builder` or pre-sized builder usage (scan `body_text` for `strings.Builder`, `bytes.Buffer`, or `Builder` allocations keyed to the same variable scope).
  - Evidence: lines, snippet, and suggestion to use `strings.Builder`, `bytes.Buffer`, or pre-allocated buffer.
- [ ] Expand the existing `n_plus_one_query` family by correlating `db_query_calls` with nested-loop signals
  - Detection strategy:
    - If `db_query_calls` contains a call marked `in_loop == true` and the function also shows nested-loop signal, raise severity and add evidence that N+1 is more likely (but still conservative; require both signals).
  - Acceptance: positive fixtures show higher-severity finding when query + nested loops present.

Checklist (Wiring)
------------------
- [ ] Export new heuristics from `src/heuristics/performance.rs` and add `n_squared_findings` (or similar) function.
- [ ] Wire the new findings into `src/heuristics/mod.rs` so `evaluate_go_file` invokes them for each function.
- [ ] Add unit-level doc comments on each new heuristic function describing inputs consumed from `ParsedFunction` and limits of the detection.

Checklist (Testing — fixtures only)
----------------------------------
Note: test harnesses in this repository expect fixture Go code as `.txt` files under `tests/fixtures/go/`. Phase 1 tests should add fixtures only; minimal Rust test harness changes are allowed only to reference new fixtures where necessary.
- [ ] Add positive fixtures (text files) under `tests/fixtures/go/`:
  - `n_squared_alloc_slop.txt` — nested loops with `append` or `make`+`append` in inner loop, no preallocation.
  - `n_squared_concat_slop.txt` — nested loops with string `+=` concatenation in inner loop and no `strings.Builder` used.
  - `n_squared_query_slop.txt` — nested loops that perform `db` calls inside inner loop.
- [ ] Add negative fixtures (text files) under `tests/fixtures/go/`:
  - `n_squared_alloc_clean.txt` — equivalent logic but with proper preallocation (`make(..., cap)`) or single-pass accumulation with precomputed total capacity.
  - `n_squared_concat_clean.txt` — uses `strings.Builder` or `bytes.Buffer` for concatenation.
  - `n_squared_query_clean.txt` — uses preloading (bulk query) or moves the query outside the inner loop.
- [ ] Add short textual guidance at the top of each fixture describing why it should trigger or not trigger the rule (a single comment line is fine).

Checklist (Testing — Rust harness)
---------------------------------
- [ ] Update or add integration test assertions (Rust) to exercise the fixtures through `scan_repository`.
  - Keep fixture content as `.txt` files; test code should remain minimal and follow existing patterns in `tests/integration_scan.rs`.
  - Acceptance: test asserts `finding.rule_id` presence for slop fixtures and absence for clean fixtures. Keep the rust test logic small and re-use existing helpers.

Checklist (Acceptance criteria)
------------------------------
- [ ] Positive fixtures produce at least one `Finding` with the expected `rule_id` for the new heuristics.
- [ ] Negative fixtures do not produce findings for the new heuristics.
- [ ] Findings include clear evidence: rule id, function name, start/end line, short message, and evidence bullet lines explaining why the rule fired and possible mitigations.
- [ ] Each new heuristic function has clear doc comments and a short unit-level test (if appropriate) for the internal line-to-loop heuristics.

Checklist (Rollout & opt-in)
---------------------------
- [ ] Add a CLI flag / config toggle to enable deeper semantic checks (e.g., `--enable-semantic` or `semantic.enabled: true` in config).
- [ ] Default behaviour remains unchanged (semantic checks are disabled by default).
- [ ] Add explicit docs in `guides/go/planned_improvements/plan4.md` (this file) explaining opt-in and recommended rollout sequence.

Checklist (Documentation & PR)
-----------------------------
- [ ] Add a short changelog entry describing the new heuristics and the opt-in flag.
- [ ] In the PR description, include example before/after findings produced for positive fixtures and note any borderline false-positive sources.
- [ ] Add a short entry in `docs/` or `README.md` describing the rationale and how to enable semantic checks.

Future phases (Phase 2 & Phase 3 — index & type powered)
-------------------------------------------------------
- [ ] Phase 2: Index-assisted rules
  - Use `RepositoryIndex` to resolve symbols and disambiguate expensive library calls vs cheap helpers.
  - Example: confirm that `db.Query` resolves to a known DB adapter package before raising severity.
- [ ] Phase 3: Full type-aware dataflow and inter-procedural analysis (deferred)
  - Integrate with `go/packages` or LSP-based type index to get real types and method resolution.
  - Add heavier data-flow analyses for precise O(n²) proofs, taint flows, and layout-aware indexing checks.

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

Next immediate step
-------------------
- [ ] Run a focused repository search for existing related heuristics and prepare the implementation PR template with the fixture list above.
