# Roadmap Issue Backlog

This document turns the master roadmap into concrete backlog issues with enough scope, validation, and acceptance detail to execute without re-planning the work each time.

## Backlog Usage

- Open issues from this guide in order unless a blocker changes the sequence.
- Keep titles stable so issue history matches roadmap reporting.
- When an issue ships, mark it complete here and move any residual work back into the relevant remaining issue instead of creating overlapping plans.

## `master-roadmap-guide`

Phase: `0`

Purpose: keep [`master-roadmap.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/master-roadmap.md) as the single current execution roadmap.

Scope:
- keep roadmap phase ordering current
- record milestone progress snapshots
- document what is intentionally deferred

Acceptance criteria:
- the roadmap reflects shipped governance work and active next milestones
- phase descriptions use exact file and workflow names that exist in the repository
- deferred work is labeled instead of silently disappearing

Validation:
- roadmap links open correctly
- implementation guide and backlog guide point at the same active roadmap

## `missing-implementation-guide-resolution`

Phase: `0`

Purpose: preserve the implementation guide as the status map and guide taxonomy entry point.

Scope:
- keep [`implementation-guide.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/implementation-guide.md) accurate
- maintain the guide taxonomy for active, backlog, and completed material
- explicitly identify imported historical plans

Acceptance criteria:
- active, backlog, and historical buckets are visible from one place
- new contributors can find the current roadmap and backlog within one hop
- stale guides are not presented as current execution contracts

Validation:
- [`guides/README.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/README.md) and the implementation guide agree on taxonomy

## `rule-registry-and-rules-cli`

Phase: `0`

Purpose: keep the rule registry and `deslop rules` CLI authoritative for inventory reporting.

Scope:
- preserve `rules/registry.json` as the source of truth
- keep language, family, severity, status, configurability, and description fields complete
- maintain CLI filtering behavior for `--language`, `--status`, and `--json`

Acceptance criteria:
- every shipped rule appears in the registry
- docs sync does not require handwritten rule inventories
- additive CLI output remains backward-compatible with `scan`

Validation:
- `cargo run --quiet -- rules --json`
- `cargo run --quiet -- rules --language go --status experimental`

## `docs-sync-ci`

Phase: `0`

Purpose: prevent drift between code, README, guides, frontend docs, and action-facing examples.

Scope:
- keep [`scripts/sync_docs.py`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/scripts/sync_docs.py) current
- keep CI drift checks active
- ensure action examples and docs are regenerated from the registry flow

Acceptance criteria:
- docs drift is caught in CI
- rule metadata changes can be propagated by one sync path
- synced files are documented and reproducible

Validation:
- `python3 scripts/sync_docs.py --check`
- CI runs the same check

## `real-repo-corpus-harness`

Phase: `1`

Purpose: make real-repository scan and benchmark validation part of routine detector development.

Scope:
- maintain [`corpus/manifest.json`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/corpus/manifest.json)
- keep [`scripts/corpus_harness.py`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/scripts/corpus_harness.py) operational
- populate remaining active corpus slots for Go, Python web, Python ML, and Rust

Acceptance criteria:
- active corpus targets can run scan and bench workflows reproducibly
- promotion note scaffolding exists for every active target
- at least one Python web, one Python ML, and one Rust application target are populated

Validation:
- `python3 scripts/corpus_harness.py validate`
- `python3 scripts/corpus_harness.py list --include-planned`
- at least one active target run per language family

## `rule-stability-tiers`

Phase: `1`

Purpose: keep `stable`, `experimental`, and `research` status meaningful.

Scope:
- require status fields in the registry
- document promotion and downgrade policy
- keep rollout flags and config gates aligned with experimental rule families

Acceptance criteria:
- every rule entry has a valid tier
- promotion decisions reference corpus notes, not fixtures alone
- experimental families do not silently become default-on

Validation:
- registry review
- evaluation guide review
- release checklist references tier expectations

## `go-web-framework-expansion`

Phase: `2`

Purpose: broaden Go request-path coverage beyond Gin and make framework-aware heuristics work for the dominant service shapes.

Scope:
- support `net/http`, `chi`, `echo`, and `fiber` handler recognition
- keep request-path export, fanout, body, and setup-churn heuristics reusable across frameworks
- normalize versioned Go module import aliases so `.../v2` and `.../v4` imports resolve to real package names

Acceptance criteria:
- request-path heuristics run on `net/http`, Echo, and Fiber handlers
- parser tests cover versioned import alias resolution
- integration fixtures prove positive and negative coverage for non-Gin request handlers

Validation:
- [`src/analysis/go/parser/tests.rs`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/src/analysis/go/parser/tests.rs)
- [`tests/integration_scan/go_advanceplan3.rs`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/tests/integration_scan/go_advanceplan3.rs)
- `cargo test --locked`

## `go-data-layer-expansion`

Phase: `2`

Purpose: deepen Go data-access coverage around service-oriented storage patterns.

Scope:
- expand `sqlc`, `ent`, `bun`, `pgxpool`, and `redis`-shaped rules
- add request-path pool, client, and transaction setup churn rules
- bias toward batching, projection, pagination, and operational correctness

Acceptance criteria:
- at least one fixture-backed rule family lands for each of `pgxpool` and `redis`
- new rules include positive and negative fixtures
- any new parser or index support ships with regression coverage

Validation:
- targeted integration tests
- corpus comparison notes for at least one Go service target

## `python-framework-expansion`

Phase: `3`

Purpose: broaden Python framework coverage with production-facing rules instead of generic lint overlap.

Scope:
- add DRF, Celery, asyncio + `httpx`/`aiohttp`, SQLModel, and Pydantic v2 families
- keep handlers, middleware, workers, and request-boundary detectors import-gated
- preserve low-noise default behavior

Acceptance criteria:
- each new family has positive and negative fixtures
- framework imports gate the rules cleanly
- at least one async-boundary rule family lands for `httpx` or `aiohttp`

Validation:
- Python integration tests
- corpus notes for at least one Python web repo

## `python-module-graph-and-packaging-awareness`

Phase: `3`

Purpose: make Python analysis more aware of repository-local structure and packaging metadata.

Scope:
- incorporate `pyproject.toml` and local module graph signals where they improve rule precision
- refine public contract checks beyond `Any`
- improve repo-level duplication and coupling context with packaging awareness

Acceptance criteria:
- new repo-level heuristics use repository metadata conservatively
- public API rules become more precise without broad false-positive growth
- packaging-aware logic is covered by dedicated tests

Validation:
- targeted parser or repository tests
- corpus notes on one typed Python application

## `python-ml-llm-ecosystem-pack`

Phase: `3`

Purpose: expand ML and LLM coverage while keeping non-ML repositories quiet.

Scope:
- add LangChain or orchestration misuse checks
- add vector database, embedding batching, tokenizer lifecycle, and prompt-boundary rules
- keep import gating strict

Acceptance criteria:
- ML and LLM rules do not fire in plain web-service fixtures
- at least one fixture-backed batching or caching rule lands
- corpus evidence exists for one ML or LLM repo target

Validation:
- Python ML integration tests
- corpus harness promotion notes

## `rust-workspace-runtime-pack`

Phase: `4`

Purpose: grow Rust in a narrow, application-oriented way.

Scope:
- add cargo workspace awareness where feasible
- deepen `tokio`, `axum`, `actix`, or `tonic` runtime checks
- focus on allocation, clone, builder, and boundary correctness rules

Acceptance criteria:
- new Rust rules remain application-biased and high signal
- async runtime heuristics are covered by positive and negative fixtures
- workspace-aware logic does not require a full new backend architecture

Validation:
- Rust integration tests
- one Rust corpus target run

## `typescript-backend-research-spike`

Phase: `5`

Purpose: document the conditions required before a fourth backend begins.

Scope:
- define architecture fit for a TypeScript or JavaScript backend
- record parser, index, registry, and rollout expectations
- keep the work explicitly deferred until Go, Python, Rust governance is stable

Acceptance criteria:
- the spike is research-only until prerequisites are complete
- parser and evaluation prerequisites are written down
- no implementation branch begins before roadmap prerequisites are met

Validation:
- roadmap review
- backlog review
- no active backend scaffold without an explicit roadmap update

Supporting guide:
- [`typescript-backend-research-spike.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/backlog/typescript-backend-research-spike.md)
