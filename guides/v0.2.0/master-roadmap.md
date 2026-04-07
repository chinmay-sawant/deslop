# Master Roadmap For deslop Next Phase

## Summary

deslop already has a broad shipped rule surface across Go, Python, and Rust. The next phase should optimize for governance and high-signal growth instead of raw rule-count expansion alone.

This roadmap sets the default order of work:

1. establish one source of truth for guides and rule inventory
2. centralize rule metadata in code
3. add an additive machine-readable `rules` surface
4. automate docs sync and drift detection
5. formalize evaluation and promotion policy
6. deepen Go and Python with application-oriented rule families
7. keep Rust narrower and more selective
8. only then begin work on a fourth backend

## Current State

- The repository already supports Go, Python, and Rust scanning.
- The Rust test baseline is green: `cargo test --locked` passed on April 2, 2026.
- Historical planning documents are numerous and useful, but many of them are now completed rather than active.
- The detector surface has grown faster than the governance surface:
  - rule inventory is distributed across code, README, guides, and frontend content
  - `features-and-detections.md` under-reports the real implemented rule count
  - the repo references `implementation-guide.md`, but that file did not previously exist
- The medium-term `500 rules` goal should be treated as a structured inventory target across `stable`, `experimental`, and `research`, not as the next default-on milestone.

## Phase 0 - Governance And Source Of Truth

### Goals

- replace fragmented planning with one current roadmap
- make rule metadata authoritative and machine-readable
- stop manually maintaining rule inventories across code, docs, and frontend surfaces

### Deliverables

- this roadmap as the active planning document
- [implementation-guide.md](implementation-guide.md) as the guide taxonomy and status map
- a central rule registry with:
  - `id`
  - `language`
  - `family`
  - `default_severity`
  - `status`
  - `configurability`
  - `description`
- additive `deslop rules` CLI support with JSON and filtering
- docs-sync automation and CI validation

### Exit Criteria

- active versus completed guides are explicit
- rule inventory is generated from the registry rather than copied into multiple places
- README, feature guide, frontend docs, and action input docs all have a reproducible sync path

## Phase 1 - Evaluation, Stability Tiers, And Promotion Policy

### Goals

- formalize when a rule is `stable`, `experimental`, or `research`
- make real-repo validation a first-class release input
- set a stricter merge contract for new rules

### Stability Tiers

- `stable`: default-on, low-noise rules suitable for normal scans
- `experimental`: shipped but gated by rollout flags, staged config, or opt-in enablement
- `research`: documented backlog items that are not ready for user-visible default output

### Merge Contract For New Rules

Every new detector should ship with:

- one positive fixture
- one negative fixture
- suppression or config coverage when the rule participates in suppression or rollout gating
- one mixed-language regression if parser or index behavior changes
- one real-repo validation note before promotion to `stable`

### Evaluation Harness Direction

Turn the existing report-diff workflow into a reusable corpus harness.

Initial representative corpus targets:

- Go services:
  - `gopdfsuit`
  - `SnapBack`
  - at least one `net/http` or `chi` service
  - at least one gRPC-heavy service
- Python web or API applications:
  - one Django or DRF app
  - one FastAPI or Flask app
- Python AI/ML applications:
  - one training or ETL-style repo
  - one LLM application repo
- Rust applications:
  - one `tokio` or `axum` service
  - one CLI or library-focused application repo

### Exit Criteria

- the corpus list is documented
- drift reports are reproducible
- severity promotion requires real-repo notes, not just fixture success

## Phase 2 - Go Growth

### Direction

Go remains the broadest and most application-oriented backend. The next Go wave should prioritize service and production patterns that still have meaningful coverage gaps.

### Priority Families

- framework expansion:
  - `net/http`
  - `chi`
  - `echo`
  - `fiber`
  - `grpc`
- data and infrastructure expansion:
  - `sqlc`
  - `ent`
  - `bun`
  - `pgxpool`
  - `redis`
- request-boundary and runtime reliability:
  - retries and backoff
  - upstream batching gaps
  - request-path client or config setup churn
  - streaming and export inefficiencies
  - config and environment misuse

### Non-Goals For This Phase

- style-only rule growth
- taste-based naming expansion
- generic linter overlap that does not improve application review

### Milestone Target

Reach roughly `220 stable Go rules` before approving work on a fourth backend.

## Phase 3 - Python Growth

### Direction

Python should grow in two deliberate directions:

1. framework, packaging, and repo-shape awareness
2. ML, data, and LLM ecosystem depth that remains import-gated

### Priority Families

- web and framework support:
  - Django REST Framework
  - Celery
  - asyncio + `httpx` + `aiohttp`
  - SQLModel
  - Pydantic v2
  - click and typer CLI boundaries
- packaging and repository awareness:
  - local module graph resolution
  - `pyproject.toml` awareness
  - better public API contract detection beyond `Any`
- AI/ML and LLM ecosystem work:
  - LangChain and LlamaIndex orchestration
  - vector database client misuse
  - embedding batch and cache patterns
  - tokenizer or model lifecycle reuse
  - prompt and response boundary handling

### Python Explicit Epics

- import-time lifecycle and bootstrap control
- environment and config loading boundaries
- timeout and retry hygiene
- external schema validation
- stronger but still conservative repo-level duplication refinement
- framework-specific N+1 or over-fetching only when evidence is strong enough

## Phase 4 - Rust Growth

### Direction

Rust should continue to grow, but with narrower scope than Go and Python. Focus on application review, local correctness review, and high-signal runtime or API risks rather than trying to maximize rule count.

### Priority Families

- cargo workspace awareness
- crate-graph and trait or impl-local resolution
- macro-aware escapes and local expansion-aware heuristics
- async runtime families:
  - `tokio`
  - `axum`
  - `actix`
  - `tonic`
- allocation and clone waste
- config, builder, and state-model misuse
- application-boundary rules

### Milestone Target

Aim for roughly `60` to `90` high-signal Rust rules.

## Phase 5 - New Backend Order

Do not start a fourth backend until rule registry, docs sync, and corpus validation are routine.

### Preferred Backend Order

1. TypeScript or JavaScript
2. Java
3. C# when demand is clear

### Deferred Backends

- Kotlin: defer until Java exists
- PHP: defer until a real corpus justifies it
- Ruby: defer until a real corpus justifies it
- C and C++: defer unless the project intentionally moves toward systems-level security review

## Immediate Backlog

Create or track these issue families immediately:

- `master-roadmap-guide`
- `missing-implementation-guide-resolution`
- `rule-registry-and-rules-cli`
- `docs-sync-ci`
- `real-repo-corpus-harness`
- `rule-stability-tiers`
- `go-web-framework-expansion`
- `go-data-layer-expansion`
- `python-framework-expansion`
- `python-module-graph-and-packaging-awareness`
- `python-ml-llm-ecosystem-pack`
- `rust-workspace-runtime-pack`
- `typescript-backend-research-spike`

## Working Defaults

- prefer one master guide instead of many concurrent phase-plan trees
- treat historical `advanceplan*` folders as completed records unless reactivated
- keep scan and bench output backward-compatible while adding new metadata and rule-listing surfaces
- favor explainable, corpus-validated rules over quick rule-count inflation
