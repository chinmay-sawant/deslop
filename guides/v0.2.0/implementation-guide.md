# deslop Implementation Guide

## Purpose

This guide is the top-level navigation and status map for implementation planning in deslop.

Use it to answer three questions quickly:

1. Which guides are current and actionable?
2. Which guides are historical records of shipped work?
3. Where should new roadmap and implementation planning live?

## Guide Taxonomy

deslop now treats planning material in three buckets:

- `active`: documents that describe the current product direction or active implementation contracts
- `completed`: historical phase plans that were already shipped and now exist mainly for traceability
- `backlog`: future-oriented expansion plans that are not current execution contracts yet

## Active Guides

- [README.md](README.md): guide index and taxonomy entry point
- [master-roadmap.md](master-roadmap.md): current multi-phase roadmap for rule governance, docs sync, evaluation, and language expansion
- [evaluation-and-promotion-policy.md](evaluation-and-promotion-policy.md): operational contract for corpus evaluation, rule stability tiers, and promotion evidence
- [features-and-detections.md](features-and-detections.md): user-facing inventory of shipped detections and current limitations
- [release-checklist.md](release-checklist.md): rollout and release validation checklist

## Completed Historical Plans

These documents are retained because they explain how major rule families were shipped, but they should not be treated as the current roadmap unless a new guide explicitly revives them.

- [go/advanceplan1/](go/advanceplan1/)
- [go/advanceplan2/](go/advanceplan2/)
- [go/advanceplan3/](go/advanceplan3/)
- [go/advanceplan4/](go/advanceplan4/)
- [python/advanceplan1/](python/advanceplan1/)
- [python/advanceplan2/](python/advanceplan2/)
- [python/advanceplan3/](python/advanceplan3/)
- [python/already_implemented/](python/already_implemented/)
- [rust/advanceplan1/](rust/advanceplan1/)
- [rust/advanceplan2/](rust/advanceplan2/)
- [rust/advanceplan3/](rust/advanceplan3/)
- [rust/index.md](rust/index.md)
- [rust/backend-scaffold-and-routing.md](rust/backend-scaffold-and-routing.md)
- [rust/parser-and-evidence-extraction.md](rust/parser-and-evidence-extraction.md)
- [rust/heuristics-and-findings.md](rust/heuristics-and-findings.md)
- [verification-performance-and-rollout.md](verification-performance-and-rollout.md)

## Backlog And Future Work

Backlog planning now belongs in the master roadmap first.

Canonical backlog issue definitions live in [backlog/roadmap-issue-backlog.md](backlog/roadmap-issue-backlog.md).
The deferred fourth-backend spike is tracked in [backlog/typescript-backend-research-spike.md](backlog/typescript-backend-research-spike.md).

If a new language, rule family, or evaluation workflow needs a dedicated guide:

1. Add the high-level direction to [master-roadmap.md](master-roadmap.md).
2. Mark whether the work is `stable-now`, `experimental-next`, or `research/deferred`.
3. Only create a separate phase guide after the work is specific enough to execute.

## Planning Rules

- Do not create a new `advanceplan*` folder for ongoing work by default.
- Prefer one active roadmap and a small number of focused supporting guides.
- When a guide is fully shipped, leave it in place but treat it as `completed` unless explicitly reactivated.
- Keep README links pointing at active guides, not historical phase files.

## Current Defaults

- Near-term product work should prioritize Go and Python depth, rule governance, docs sync, and evaluation quality.
- Rust should remain narrower and more application-focused than Go and Python.
- A fourth backend should not start until the registry, docs-sync, and evaluation workflow are settled.
