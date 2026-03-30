# Plan 4 - Implementation Roadmap, Parser Work, And Validation (Go)

Date: 2026-03-31

## Status

- [ ] Drafted only; not implemented yet.
- [ ] This file is the execution roadmap for `advanceplan3` and ties the scenario backlog to the current Rust parser and heuristics architecture.

## Backlog Summary

- [x] `advanceplan3/plan1.md` defines 37 generic hot-path and allocation scenarios.
- [x] `advanceplan3/plan2.md` defines 50 SQL, `database/sql`, `sqlx`, `pgx`, and `gorm` scenarios.
- [x] `advanceplan3/plan3.md` defines 34 Gin and HTTP request-path scenarios.
- [x] Total backlog in `advanceplan3`: 121 candidate scenarios.

## Shared Parser And Evidence Work

- [ ] Add loop summaries that capture loop depth, loop kind, iterated binding, and whether a local bound such as `len(x)` is visible.
- [ ] Add append and allocation summaries that record target binding, source binding, visible capacity hint, and the line where growth starts.
- [ ] Add builder and buffer summaries for `strings.Builder`, `bytes.Buffer`, `bufio.Reader`, `bufio.Writer`, `csv.Writer`, `gzip.Writer`, and `gzip.Reader` including `Grow`, `Reset`, `Flush`, and `Close` calls.
- [ ] Add repeated-call grouping keyed by receiver, method, normalized callee, and stable argument binding so "same input parsed twice" rules do not need raw string scans.
- [ ] Add import-aware framework classification for `database/sql`, `sqlx`, `pgx`, `gorm`, `gin`, `html/template`, `text/template`, `regexp`, `encoding/json`, `encoding/xml`, `bufio`, `compress/gzip`, and `encoding/csv`.
- [ ] Add ordered GORM chain summaries that preserve steps such as `Model`, `Table`, `Where`, `Joins`, `Preload`, `Scopes`, `Select`, `Omit`, `Limit`, `Offset`, `Order`, `Distinct`, `Group`, `Count`, `Find`, `First`, `Take`, `Scan`, `Rows`, `Create`, `Save`, `Updates`, `Delete`, `Raw`, and `Exec`.
- [ ] Add Gin handler summaries using `*gin.Context` parameters, router-registration cues, and render/body helper calls.
- [ ] Add request-body summaries for `GetRawData`, `ShouldBind*`, `Bind*`, `ParseMultipartForm`, `FormFile`, `ReadAll(c.Request.Body)`, and manual decoder creation.
- [ ] Add response/export summaries for `JSON`, `PureJSON`, `IndentedJSON`, `Data`, file-serving helpers, streaming helpers, and row-wise response writes.

## Heuristic Wave Plan

- [ ] Wave 1: ship the highest-confidence generic hot-path rules from `plan1.md` that only require function-local evidence.
- [ ] Wave 2: ship import-aware lower-level SQL rules for `database/sql`, `sqlx`, and `pgx`.
- [ ] Wave 3: ship GORM chain-shape rules once ordered query-step summaries exist.
- [ ] Wave 4: ship Gin request-body duplication and export-path rules once handler summaries are stable.
- [ ] Wave 5: add repo-aware correlation such as duplicate query templates, repeated upstream calls, and cross-handler hot-route regressions only after the function-local rules have settled.

## False-Positive Controls

- [ ] Gate handler-only findings on `*gin.Context`, `http.Handler`, or clear router-registration evidence instead of simple package imports.
- [ ] Keep startup-only setup code quiet unless the expensive operation clearly appears inside request or loop paths.
- [ ] Default "candidate optimization" rules to `Info` and escalate to `Warning` only when at least two supporting signals agree.
- [ ] Suppress batch findings when `CreateInBatches`, `FindInBatches`, cursor iteration, copy-style bulk APIs, or explicit chunking helpers are already present.
- [ ] Suppress missing-limit style findings for migrations, CLI commands, offline exports, and background jobs when package or symbol names clearly indicate non-request workloads.
- [ ] Avoid raw method-name matching when import resolution cannot disambiguate DB, Gin, or helper-library symbols.
- [ ] Keep messages explicit about uncertainty when the rule infers hot-path intent from naming, handler shape, or repeated-loop evidence rather than a guaranteed profiler trace.

## Fixtures, Integration Tests, And Benchmarks

- [ ] Add dedicated fixture slices for `tests/fixtures/go/performance_advanceplan3_core`, `tests/fixtures/go/performance_advanceplan3_sql`, and `tests/fixtures/go/performance_advanceplan3_gin`.
- [ ] Create one positive and one clean fixture for every scenario family before enabling any family by default.
- [ ] Add parser regression tests for new summaries before wiring the heuristics that consume them.
- [ ] Group integration tests by family instead of by single rule so the Rust harness stays compact and reviewable.
- [ ] Add benchmark snapshots on at least one generic Go repository, one Gin-heavy service, and one GORM-heavy service.
- [ ] Re-run representative external validation on repositories similar to the existing `gopdfsuit` and `SnapBack` quick-runs before promoting severities.

## Suggested Priority Order

- [ ] First ship the 15 to 20 highest-confidence rules that need only local call, loop, and allocation evidence.
- [ ] Next ship import-aware SQL rules before deeper GORM chain heuristics because they have clearer anchors and easier clean fixtures.
- [ ] Then ship Gin body-duplication and export/buffering rules before lower-confidence response-shape or logging-cost rules.
- [ ] Hold speculative rules until at least two real-repo examples justify the signal and the clean fixtures prove the false-positive controls.

## Acceptance Criteria

- [ ] Every shipped rule points at a concrete expensive site and explains why it matters in throughput, allocation, or round-trip terms.
- [ ] Every rule family has clean fixtures that demonstrate the intended escape hatch or optimized pattern.
- [ ] The new parser summaries do not materially regress end-to-end Go scan time.
- [ ] Real repositories with startup code, migrations, tests, and CLI commands do not light up with broad false positives.

## Non-Goals

- [ ] Do not claim exact Big-O proofs, precise query plans, or index truth without schema/runtime evidence.
- [ ] Do not turn `advanceplan3` into a generic style pack; keep the focus on measurable or plausibly measurable performance waste.
- [ ] Do not couple the first implementation wave to SSA, type checking, or full Go package loading unless a rule demonstrably needs it.