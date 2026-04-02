# deslop Evaluation And Promotion Policy

## Purpose

This guide turns the roadmap's Phase 1 work into an operational contract.

Use it when:

- adding a new rule
- changing the severity or rollout status of an existing rule
- validating a parser or index expansion against real repositories
- deciding whether a finding family is ready for `stable`

## Stability Tiers

Every rule in the central registry must declare one of these statuses:

- `stable`: default-on, low-noise rules suitable for normal scans
- `experimental`: shipped but gated by config or rollout flags while the team collects more evidence
- `research`: documented backlog or exploratory metadata that is not yet ready for user-visible default output

Promotion is intentionally one-way in spirit but not irreversible. If corpus review reveals high-noise behavior, a rule can move back from `stable` to `experimental` or be deferred to `research`.

## Corpus Harness

deslop now keeps a dedicated corpus manifest in `corpus/manifest.json` and a harness in `scripts/corpus_harness.py`.

Current workflow goals:

- keep `gopdfsuit` and `SnapBack` as active comparison targets
- reserve planned slots for:
  - one additional Go HTTP-style service
  - one additional Go gRPC-style service
  - one Python web or API application
  - one Python ML or LLM application
  - one Rust application
- make scan and benchmark conventions explicit and repeatable

## Harness Commands

Validate the manifest:

```bash
python3 scripts/corpus_harness.py validate
```

List active targets:

```bash
python3 scripts/corpus_harness.py list
```

Run scan comparison for one active target:

```bash
python3 scripts/corpus_harness.py run --target gopdfsuit --scan
```

Run benchmark capture for one active target:

```bash
python3 scripts/corpus_harness.py run --target gopdfsuit --bench
```

Dry-run command resolution without executing:

```bash
python3 scripts/corpus_harness.py run --target gopdfsuit --scan --bench --dry-run
```

## Promotion Evidence Requirements

Before promoting a rule to `stable`, capture:

- fixture coverage:
  - one positive fixture
  - one negative fixture
  - suppression or config coverage when the rule participates in suppression or rollout gating
- parser or index regression coverage:
  - one mixed-language regression if parser or index behavior changed
- corpus evidence:
  - at least one real-repo note for a relevant corpus target
  - per-rule false-positive observations, even if the answer is "none observed"

## False-Positive Notes

Each corpus target owns a promotion note path under `reports/corpus/<target>/promotion-notes.md`.

The harness bootstraps that note the first time a target is executed.

Promotion notes should answer:

- which rule IDs were reviewed
- whether any candidate findings were false positives
- whether the rule should stay `experimental`, move to `stable`, or be deferred
- what follow-up work is still needed

Use the committed template in `reports/corpus/promotion-note-template.md` when notes need to be created manually.

## Per-Rule Merge Contract

Every new detector or materially changed detector should satisfy this contract:

1. Add or update the rule metadata entry in the registry.
2. Ship one positive fixture.
3. Ship one negative fixture.
4. Add suppression or config coverage if the rule supports suppression or staged rollout.
5. Add a mixed-language regression if parser or index behavior changed.
6. Run `cargo test --locked`.
7. Run `python3 scripts/sync_docs.py --check`.
8. Capture corpus evidence or explain why real-repo validation is intentionally deferred.

The repository PR template mirrors this checklist so the merge contract is visible during review.

## Current Defaults

- Do not promote a rule to `stable` on fixture evidence alone when it touches hot paths, framework routing, or repository-level duplication.
- Prefer keeping new families `experimental` if corpus coverage is still thin.
- If a new rule adds obvious value but the corpus is incomplete, ship it as `experimental` and record the gap explicitly instead of silently treating it as done.

## Relationship To Other Guides

- `guides/master-roadmap.md` defines the multi-phase roadmap.
- `guides/implementation-guide.md` tells readers which guides are active.
- `guides/features-and-detections.md` is the generated user-facing inventory of shipped rules.
- `guides/release-checklist.md` remains the release-time verification checklist.
