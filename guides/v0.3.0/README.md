# deslop v0.3.0 Guides

This folder contains the draft guide set for deslop `v0.3.0`.

## Current Contents

- [go/architecture-and-governance-rule-backlog.md](go/architecture-and-governance-rule-backlog.md): checklist-driven backlog of 210 new Go rule candidates focused on architecture, layering, Gin boundaries, service and model ownership, error contracts, GORM/SQL governance, transactions, bootstrap, testing, and operational consistency.
- [go/go-architecture-review-gap-checklist.md](go/go-architecture-review-gap-checklist.md): follow-up checklist that maps an external Go architecture review against the shipped registry, marks what is already covered, and queues only the remaining net-new rule candidates to avoid duplication.
- [go/go-project-agnostic-optimization-50-point-checklist.md](go/go-project-agnostic-optimization-50-point-checklist.md): 50-point audit of a Go architecture and optimization wishlist against the live registry, now fully resolved across new shipped rules, existing coverage, and explicitly non-promoted review guidance.

## Scope Notes

- This draft intentionally avoids re-listing the already shipped Go performance, hot-path, and request-path DB misuse inventory from `v0.2.0`.
- This draft also avoids generic repo-layout enforcement except where layered Gin/GORM applications already show clear package ownership patterns such as `service`, `repository`, `model`, `dto`, `router`, `middleware`, or `transport`.
- Treat the architecture backlog docs as planning material; the 50-point Go optimization checklist documents what is now shipped versus what remains intentionally review-only.

## Expected Implementation Fit

- Heuristics should continue to live under `src/heuristics/go`.
- Rule catalog bindings should continue to live under `src/rules/catalog/go`.
- Positive and clean Go fixtures should continue to live under `tests/fixtures/go`.
- Integration coverage should continue to live under `tests/integration_scan/go`.
