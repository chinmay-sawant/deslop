# deslop Guides Index

This index is the entry point for roadmap, implementation, and backlog material in deslop.

## Guide Taxonomy

deslop uses three planning buckets:

- `active`: the guides that define current implementation direction or release policy
- `backlog`: detailed future work that is approved direction but not yet fully shipped
- `completed`: historical phase guides kept for traceability

## Active

- [`master-roadmap.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/master-roadmap.md): the current multi-phase roadmap
- [`implementation-guide.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/implementation-guide.md): status map and planning rules
- [`evaluation-and-promotion-policy.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/evaluation-and-promotion-policy.md): corpus, promotion, and stability contract
- [`features-and-detections.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/features-and-detections.md): synced rule inventory and user-facing detection guide
- [`inventory-regression-guards.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/inventory-regression-guards.md): intentional inventory-count guards and informational scan workflow notes
- [`release-checklist.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/release-checklist.md): release validation checklist

## Backlog

- [`backlog/roadmap-issue-backlog.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/backlog/roadmap-issue-backlog.md): canonical issue backlog derived from the master roadmap
- [`backlog/rust-code-structure-improvement-plan.md`](/home/chinmay/ChinmayPersonalProjects/deslop/guides/backlog/rust-code-structure-improvement-plan.md): detailed plan for tightening Rust code structure across `src/` and `tests/`
- [`backlog/rust-code-structure-followup-plan.md`](/home/chinmay/ChinmayPersonalProjects/deslop/guides/backlog/rust-code-structure-followup-plan.md): post-refactor follow-up plan after the first Rust structure implementation pass
- [`backlog/typescript-backend-research-spike.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/backlog/typescript-backend-research-spike.md): deferred research contract for the first new backend after Go, Python, and Rust governance work settles

## Completed Or Historical

- [`newplan02042026.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/newplan02042026.md): imported source snapshot that fed the master roadmap
- [`go/advanceplan1/`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/go/advanceplan1)
- [`go/advanceplan2/`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/go/advanceplan2)
- [`go/advanceplan3/`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/go/advanceplan3)
- [`python/advanceplan1/`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/python/advanceplan1)
- [`python/advanceplan2/`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/python/advanceplan2)
- [`python/advanceplan3/`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/python/advanceplan3)
- [`python/already_implemented/`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/python/already_implemented)
- [`rust/advanceplan1/`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/rust/advanceplan1)
- [`rust/advanceplan2/`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/rust/advanceplan2)
- [`verification-performance-and-rollout.md`](/home/chinmay/ChinmayPersonalProjects/deslop-codex/guides/verification-performance-and-rollout.md)

## Usage Rules

- Put new execution contracts in the active guides first, not in a new `advanceplan*` folder.
- Put future tickets, epics, and candidate expansions in the backlog guide before opening parallel planning files.
- Treat historical phase docs as reference material unless the master roadmap explicitly revives them.
