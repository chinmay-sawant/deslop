# deslop v0.2.0 Guides

This folder contains the versioned guide set for deslop `v0.2.0`.

Use this index to navigate the current roadmap, rule inventory, release policy, historical plan records, and the public release note for this version.

## Start Here

- [release-notes.md](release-notes.md): public-facing release note for `v0.2.0`
- [features-and-detections.md](features-and-detections.md): current generated rule inventory and detection guide
- [implementation-guide.md](implementation-guide.md): current status map and planning rules

## Guide Taxonomy

deslop uses three planning buckets:

- `active`: the guides that define current implementation direction or release policy
- `backlog`: detailed future work that is approved direction but not yet fully shipped
- `completed`: historical phase guides kept for traceability

## Active

- [master-roadmap.md](master-roadmap.md): the current multi-phase roadmap
- [implementation-guide.md](implementation-guide.md): status map and planning rules
- [evaluation-and-promotion-policy.md](evaluation-and-promotion-policy.md): corpus, promotion, and stability contract
- [features-and-detections.md](features-and-detections.md): synced rule inventory and user-facing detection guide
- [inventory-regression-guards.md](inventory-regression-guards.md): intentional inventory-count guards and informational scan workflow notes
- [release-checklist.md](release-checklist.md): release validation checklist
- [architecture-guardrails.md](architecture-guardrails.md): ownership expectations, size thresholds, and the standing refactor validation loop

## Backlog

- [backlog/roadmap-issue-backlog.md](backlog/roadmap-issue-backlog.md): canonical issue backlog derived from the master roadmap
- [backlog/rust-code-structure-improvement-plan.md](backlog/rust-code-structure-improvement-plan.md): detailed plan for tightening Rust code structure across `src/` and `tests/`
- [backlog/rust-code-structure-followup-plan.md](backlog/rust-code-structure-followup-plan.md): completed repository-wide architecture refactor record for `src/` and `tests/` across Go, Python, and Rust
- [backlog/typescript-backend-research-spike.md](backlog/typescript-backend-research-spike.md): deferred research contract for the first new backend after Go, Python, and Rust governance work settles

## Completed Or Historical

- [go/advanceplan1/](go/advanceplan1)
- [go/advanceplan2/](go/advanceplan2)
- [go/advanceplan3/](go/advanceplan3)
- [go/advanceplan4/](go/advanceplan4)
- [python/advanceplan1/](python/advanceplan1)
- [python/advanceplan2/](python/advanceplan2)
- [python/advanceplan3/](python/advanceplan3)
- [python/already_implemented/](python/already_implemented)
- [rust/advanceplan1/](rust/advanceplan1)
- [rust/advanceplan2/](rust/advanceplan2)
- [rust/advanceplan3/](rust/advanceplan3)
- [verification-performance-and-rollout.md](verification-performance-and-rollout.md)

## Older Version

- [../v0.1.0/README.md](../v0.1.0/README.md): archived `v0.1.0` guide snapshot

## Usage Rules

- Put new execution contracts in the active guides first, not in a new `advanceplan*` folder.
- Put future tickets, epics, and candidate expansions in the backlog guide before opening parallel planning files.
- Treat historical phase docs as reference material unless the master roadmap explicitly revives them.
