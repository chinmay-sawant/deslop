# deslop v0.3.0 Guides

This folder contains the draft guide set for deslop `v0.3.0`.

## Current Contents

- [python/python-worst-practices-and-performance-checklist.md](python/python-worst-practices-and-performance-checklist.md): checklist-driven backlog of 200 net-new Python rule candidates covering architecture and layer boundaries, async and concurrency correctness, error handling discipline, type system contracts, testing anti-patterns, security boundaries, memory and resource management, configuration and secrets hygiene, logging and observability, module and package design, data structure and algorithm choices, and web API design anti-patterns.
- [python/python-project-agnostic-performance-and-structure-checklist.md](python/python-project-agnostic-performance-and-structure-checklist.md): additional 200-rule Python backlog aligned to the existing category vocabulary, focused on repo-agnostic architecture, boundaries, hot-path efficiency, maintainability, observability, packaging, performance, quality, and structural clarity.
- [go/architecture-and-governance-rule-backlog.md](go/architecture-and-governance-rule-backlog.md): checklist-driven backlog of 210 new Go rule candidates focused on architecture, layering, Gin boundaries, service and model ownership, error contracts, GORM/SQL governance, transactions, bootstrap, testing, and operational consistency.
- [go/go-architecture-review-gap-checklist.md](go/go-architecture-review-gap-checklist.md): follow-up checklist that maps an external Go architecture review against the shipped registry, marks what is already covered, and queues only the remaining net-new rule candidates to avoid duplication.
- [go/go-project-agnostic-optimization-50-point-checklist.md](go/go-project-agnostic-optimization-50-point-checklist.md): 50-point audit of a Go architecture and optimization wishlist against the live registry, now fully resolved across new shipped rules, existing coverage, and explicitly non-promoted review guidance.
- [go/go-data-access-gap-plan-checklist.md](go/go-data-access-gap-plan-checklist.md): checklist for the final eight unreferenced Go `data_access` rules, including the fixture and integration coverage needed to close the remaining test gap.
- [rust/rust-bad-practices-and-performance-plan-checklist.md](rust/rust-bad-practices-and-performance-plan-checklist.md): unchecked Rust v0.3.0 planning backlog for repo-agnostic bad-practice and performance rules, prioritized around the current direct crate stack: `tree-sitter`, `rayon`, `ignore`, `clap`, `serde`, `serde_json`, `toml`, `anyhow`, `thiserror`, `libc`, `proptest`, `tempfile`, and `libfuzzer-sys`.

## Scope Notes

- This draft intentionally avoids re-listing the already shipped Go performance, hot-path, and request-path DB misuse inventory from `v0.2.0`.
- This draft also avoids generic repo-layout enforcement except where layered Gin/GORM applications already show clear package ownership patterns such as `service`, `repository`, `model`, `dto`, `router`, `middleware`, or `transport`.
- Treat the architecture backlog docs as planning material; the 50-point Go optimization checklist documents what is now shipped versus what remains intentionally review-only.
- The Rust plan intentionally starts with dependency-gated rules from the current project before moving into broader Rust allocation, concurrency, I/O, manifest, and build-system guidance.

## Expected Implementation Fit

- Heuristics should continue to live under `src/heuristics/go`.
- Rule catalog bindings should continue to live under `src/rules/catalog/go`.
- Positive and clean Go fixtures should continue to live under `tests/fixtures/go`.
- Integration coverage should continue to live under `tests/integration_scan/go`.
- Rust heuristics should continue to live under `src/heuristics/rust`.
- Rust rule catalog bindings should continue to live under `src/rules/catalog/rust`.
- Positive and clean Rust fixtures should continue to live under `tests/fixtures/rust`.
- Rust integration coverage should continue to live under `tests/integration_scan/rust`.
