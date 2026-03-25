# Phase 5: Profiling-Sensitive Follow-Up And Deferred Gap Closure

## Overview

This phase exists for the Python backlog items that Phase 4 intentionally did not over-promise. The current advanced baseline already ships a conservative set of performance, structure, duplication, maintainability, and AI-smell findings. What remains is a narrower but harder class of work:

- items that are only partially covered by a smaller shipped rule
- items that need repository-level comparison instead of file-local pattern checks
- items whose original wording implies profiling or workload knowledge that static analysis cannot claim directly
- items that would become style noise unless the evidence boundary becomes more explicit first

The goal of this phase is not to force every deferred item into a finding. The goal is to turn the remaining backlog into a merge-quality implementation contract with explicit promotion rules:

- what should expand an existing rule
- what should become a new rule family
- what needs parser or repository-index enrichment first
- what should remain a permanent non-goal unless stronger evidence becomes available

This document should be read together with [Phase 4](advanced-rule-families-and-backlog.md).

- Phase 4 remains the status map for what is shipped, partial, deferred, or intentionally out of scope.
- Phase 5 is the execution contract for the still-open items that deserve a deliberate next pass.

## In Scope

- Tighten the evidence policy for profiling-sensitive Python smells before any new rule ships.
- Expand the current partial duplication support into explicit repository-level clone and repeated-pipeline checkpoints.
- Close the major gap between the original backlog wording and the narrower Phase 4 implementations for architecture and naming smells.
- Separate "needs more evidence" items from true non-goals so later work is not forced to rediscover the same triage decisions.
- Define test, benchmark, and documentation expectations for any promoted Phase 5 rule.
- Keep every proposed detector explainable enough for fixture-driven verification.

## Out Of Scope

- Runtime profiling, instrumentation, or workload replay.
- Framework-specific ORM internals, query planners, or database index reasoning unless explicitly designed behind a documented adapter boundary.
- Style-only variable naming review that cannot be reduced to objective structural evidence.
- Personality, tone, or "feels AI-generated" judgments that do not collapse into a measurable code smell.
- Semantic linting that would require import execution, type inference, or dynamic code evaluation.

## Dependencies

- Depends on Phase 4 because this phase only exists to promote or reject the backlog that Phase 4 already classified.
- Depends on the current Python parser evidence and repository index staying language-scoped and explainable.
- Depends on fixture coverage remaining the default merge gate for any new Python rule.
- May require small parser and index extensions, but those extensions must be driven by concrete rule consumers rather than speculative future modeling.

## Primary Code Areas

- `src/analysis/python/mod.rs`
- `src/analysis/python/parser/mod.rs`
- `src/analysis/python/parser/`
- `src/heuristics/python/duplication.rs`
- `src/heuristics/python/performance.rs`
- `src/heuristics/python/structure.rs`
- `src/heuristics/python/maintainability.rs`
- `src/heuristics/python/ai_smells.rs`
- `src/index/mod.rs`
- `src/benchmark/mod.rs`
- `tests/integration_scan/python.rs`
- `tests/fixtures/python/`
- `guides/python/advanced-rule-families-and-backlog.md`
- `guides/features-and-detections.md`
- `guides/implementation-guide.md`

## Implementation Checkpoints

## Current Phase 5 Status

- [x] Query-fragment duplication is now shipped as `duplicate_query_fragment`.
- [x] `too_many_instance_attributes` now uses a tiered policy with a stronger 20-plus escalation checkpoint.
- [x] Cross-file copy-paste function detection for non-test code is now shipped as `cross_file_copy_paste_function`.
- [x] Duplicate transformation-pipeline detection is now shipped as `duplicate_transformation_pipeline`.
- [x] Broader non-`__init__.py` monolithic-module detection is now shipped as `monolithic_module`.
- [ ] Over-abstraction expansion beyond the current thin-wrapper rule is not shipped yet.
- [ ] Misleading-name expansion beyond the current responsibility-mismatch proxy is not shipped yet.
- [ ] Hardcoded business-logic, magic-value, and reinvented-utility follow-up rules are not shipped yet.
- [ ] Boundary-aware robustness checks for missing retries, logging, environment handling, or input hardening are not shipped yet.
- [ ] The full promotion matrix for every still-open item is not yet captured as a single checked-off implementation table in this file.

### 1. Freeze The Promotion Matrix For Every Still-Open Item

Before new implementation starts, convert the open Phase 4 backlog into a single working matrix with one row per item and one allowed disposition:

- expand an existing rule conservatively
- create a new rule family
- add parser evidence first
- add repository-level evidence first
- keep deferred as a documented non-goal

This matrix must explicitly include the current gap list called out by the user:

- cross-file copy-paste detection
- query-fragment duplication
- duplicate transformation pipelines
- 20-plus instance-variable classes versus the current 10-attribute threshold
- monolithic modules beyond `__init__.py`
- classes that should really be functions or dataclasses
- misleading names beyond the current responsibility-mismatch proxy
- hardcoded business logic
- verb and subject reversal
- over-fetching
- magic numbers and strings
- reinventing the wheel
- boilerplate exception wrappers
- missing real-world context such as retries and environment handling
- toy-problem code that assumes perfect input

Phase 5 should not start implementation until each row has a target disposition and a short explanation for why that disposition is honest.

Status:

- [ ] Promotion matrix completed in this document
- [x] Query-fragment duplication classified and implemented
- [x] 20-plus instance-variable escalation classified and implemented
- [x] Cross-file copy-paste detection classified and implemented
- [x] Duplicate transformation pipelines classified and implemented
- [x] Monolithic modules beyond `__init__.py` classified and implemented

### 2. Add Repository-Level Cross-File Copy-Paste Detection

The current duplication support is intentionally narrower than full clone detection. Phase 5 should add a repository-level function-clone checkpoint with strict boundaries.

Required implementation shape:

- derive a normalized fingerprint for non-trivial Python function bodies across files
- ignore trivial helpers, tiny wrappers, and generated-looking glue code
- exclude test-only duplication from the default production rule path unless the finding is specifically about test-versus-production overlap
- anchor each finding on one representative location while still reporting the number of matching files and functions

Required evidence policy:

- minimum line-count or token-count floor high enough to avoid accidental matches
- normalization that removes harmless identifier churn but preserves operation shape
- evidence payload that names the clone-group size and file spread without echoing full bodies

Target rule outcome:

- either a new `cross_file_copy_paste_function` rule or a clearly named equivalent

Status:

- [x] Repository-level non-test function clone rule shipped as `cross_file_copy_paste_function`
- [x] Minimum body-size and structure thresholds documented in code and tests
- [x] Evidence payload reports clone-group size and file spread

### 3. Separate Query-Fragment Duplication From Generic Literal Repetition

Repeated string literals are already shipped, but the backlog asked for repeated query fragments more specifically. Phase 5 should treat those as a separate repository-level smell rather than an extension of raw string equality.

Required implementation shape:

- normalize SQL-like or query-like string fragments before clustering
- allow repeated fragments across helpers, services, and endpoint modules
- distinguish shared query constants from scattered inline duplication

Required evidence policy:

- the fragment must appear in multiple functions or files, not only multiple lines of one query builder
- the normalization should preserve enough tokens to explain the duplication cluster
- negative fixtures must cover centralized constants, migrations, and intentionally shared query templates

Target rule outcome:

- a dedicated query-fragment duplication rule or an explicitly documented decision to keep this deferred if normalization proves too noisy

Status:

- [x] Dedicated repo-level query-fragment rule shipped as `duplicate_query_fragment`
- [x] Query-like literals are normalized before clustering
- [x] Query-like literals are excluded from the generic `cross_file_repeated_literal` rule
- [ ] Negative fixtures added for centralized query constants, migrations, and intentionally shared templates

### 4. Add Duplicate Transformation-Pipeline Detection

Phase 4 already recognizes validation-pipeline repetition, but the broader data-transformation backlog remains open. This checkpoint should add a structured pipeline signature instead of relying on loose clone detection.

Required implementation shape:

- model ordered stages such as parse, validate, map, filter, enrich, aggregate, and serialize when those stages are syntactically visible
- compare those stage sequences across functions and files
- require enough pipeline length and operator overlap to avoid matching normal short helpers

Required evidence policy:

- findings must explain the repeated stage sequence, not just say the bodies look similar
- fixtures must cover both genuine repeated ETL-style flows and false-positive candidates such as short mapping helpers

Target rule outcome:

- a dedicated transformation-pipeline duplication rule or a documented repository-level prerequisite if the current parser cannot expose the needed stages cleanly

Status:

- [x] Ordered transformation-stage signature implemented
- [x] Repository-level transformation-pipeline clustering shipped as `duplicate_transformation_pipeline`
- [x] Positive fixtures added
- [ ] Negative fixtures added

### 5. Close The Class-Size And Module-Monolith Gaps

The current structure rules cover part of the original backlog, but the exact wording is still only partially satisfied.

Required implementation shape:

- extend `too_many_instance_attributes` into a tiered threshold model so the current 10-attribute smell can coexist with a stronger 20-plus escalation checkpoint
- generalize `monolithic_init_module` into a broader monolithic-module heuristic when a module combines size, imports, declarations, and orchestration behavior
- keep the existing `god_class` and `mixed_concerns_function` rules separate so the new thresholds do not duplicate their purpose

Required evidence policy:

- size alone cannot be enough for a monolithic-module finding
- high attribute counts should be contextualized by method count, public surface, or collaborator count
- negative fixtures must include intentionally broad but structured modules such as registries, schemas, or constant catalogs

Target rule outcome:

- either an expanded `too_many_instance_attributes` policy with tiered messaging and a new `monolithic_module` rule, or a documented reason why the broader module rule remains deferred

Status:

- [x] `too_many_instance_attributes` tiered messaging shipped with 10-plus and 20-plus tiers
- [x] Broader `monolithic_module` rule shipped for non-`__init__.py` files with a 1500-line floor and stronger orchestration evidence
- [x] Negative fixtures added for broad-but-structured modules such as registries, schemas, and API-surface modules

### 6. Expand Over-Abstraction Coverage Beyond Thin Wrappers

The current `over_abstracted_wrapper` rule catches only the narrowest wrapper shapes. Phase 5 should decide whether the broader backlog item "classes where a function or dataclass would suffice" can be promoted safely.

Required implementation shape:

- identify tiny classes that mostly store constructor arguments and forward one or two calls
- distinguish between stateless wrappers, lightweight data containers, and meaningful protocol or lifecycle abstractions
- look for class shapes that could plausibly collapse into a function, dataclass, or module helper without semantic loss

Required evidence policy:

- no promotion if the class participates in inheritance, protocol conformance, or visible lifecycle management
- no promotion if the class owns meaningful mutable state across methods
- findings should explain why the current class shape looks ceremonial rather than behavioral

Target rule outcome:

- broadened `over_abstracted_wrapper` coverage with more explicit messaging about function-or-dataclass alternatives

### 7. Strengthen Misleading-Name Detection Without Drifting Into Style Policing

The current name/responsibility mismatch rule covers only part of the user's wording. Phase 5 should add stronger anchors but keep style-only naming review out of scope.

Required implementation shape:

- compare function or class names against the dominant side effects visible in calls and imports
- focus on mismatches such as "parse" functions that perform writes, "client" classes that orchestrate business workflows, or "helper" modules that own persistence and HTTP logic
- keep verb and subject reversal as a separate checkpoint instead of silently absorbing it into the existing rule

Required evidence policy:

- a finding needs both name evidence and behavior evidence
- imported subsystems or call categories should support the message when available
- fixtures must include counterexamples where names are broad but still honest for the implementation

Target rule outcome:

- expanded `name_responsibility_mismatch` plus an explicit keep-deferred decision for verb-and-subject inversion unless a better proxy emerges

### 8. Evaluate Hardcoded Business Logic, Magic Values, And Reinvented Utilities As Separate Families

These items are currently grouped together in backlog language, but they should not be implemented as one catch-all smell.

Required implementation shape:

- prototype business-threshold detection for embedded ratios, status maps, or eligibility cutoffs in endpoint or service logic
- prototype magic-number and magic-string detection only when the value is repeated, branch-shaping, or externally meaningful
- prototype reinvented-utility detection only for obvious overlap with the standard library or already-imported dependencies

Required evidence policy:

- one-off numeric constants are not enough by themselves
- findings must explain why the literal looks policy-bearing, not just present
- library-choice judgments must stay conservative and local to visible code usage

Target rule outcome:

- either one or more narrowly defined rules with strong fixtures, or an explicit decision that the family remains deferred because the evidence line is still too subjective

### 9. Treat Context And Robustness Gaps As Boundary-Aware Checks

The backlog items about missing logging, retries, environment handling, and perfect-input assumptions are easy to overstate. Phase 5 should only promote them when the code is clearly at a risk boundary.

Required implementation shape:

- focus on request handlers, CLI entry points, scheduled jobs, file and network boundaries, and environment-dependent startup code
- look for visible risky operations with no surrounding validation, fallback, timeout, retry, or error-context handling
- keep pure local helpers and obviously internal transformations out of scope

Required evidence policy:

- the finding must point to a concrete external boundary or failure surface
- absence-of-context claims must stay narrow and avoid framework-specific assumptions unless explicitly supported
- negative fixtures must include simple internal helpers where adding retries or logging would be gratuitous

Target rule outcome:

- a small boundary-aware robustness family, or a reaffirmed decision to keep these items deferred if the false-positive rate stays too high

### 10. Keep Profiling-Sensitive Work Honest With Benchmarks And Messaging

Several deferred items were originally framed as performance problems. Static analysis can only approximate those claims, so this phase needs an explicit benchmark and messaging policy.

Required implementation shape:

- any new performance-oriented rule must state the static proxy it actually detects
- benchmark notes should record whether a parser or repository-index expansion materially changed scan cost
- messages should avoid claiming measured slowness and instead describe the detected structural risk

Required evidence policy:

- no rule should say code is "slow" unless the product has runtime evidence, which this phase does not add
- messages should prefer wording like "repeated work," "materializes intermediate data," or "uses an expensive shape in a hot-looking path"

Target rule outcome:

- performance findings that remain honest about their static-analysis limits

## Acceptance Criteria

- Every deferred or partial item promoted by this phase has a documented evidence boundary before code changes begin.
- Cross-file duplication work is either implemented with repository-level evidence or explicitly left deferred with a technical reason.
- Structural gaps from the original backlog wording are either closed conservatively or documented as narrower intentional support.
- Profiling-sensitive items use messages that describe structural risk rather than pretending to prove runtime measurements.
- New rules do not collapse style preference into user-visible findings.

## Verification

- Review this file against [Phase 4](advanced-rule-families-and-backlog.md) and confirm every still-open item from that document has a concrete Phase 5 checkpoint or an explicit non-goal decision.
- Spot-check the current implementation baseline in `src/heuristics/python/duplication.rs` and `src/heuristics/python/structure.rs` so the plan reflects what is actually partial today.
- Do not mark any Phase 5 detector complete until it has fixture-backed positive and negative coverage in `tests/fixtures/python/` and `tests/integration_scan/python.rs`.
- Update the benchmark note if parser or repository-level evidence expansion changes scan cost meaningfully.

## Document Update Obligations

- Update this file whenever a Phase 5 checkpoint changes status, is narrowed, or is promoted into shipped detection work.
- Keep [Phase 4](advanced-rule-families-and-backlog.md) synchronized so Phase 4 remains the canonical shipped-versus-deferred status map.
- Update `guides/features-and-detections.md` when any Phase 5 rule becomes user-visible.
- Update `guides/implementation-guide.md` when Phase 5 introduces new parser evidence or repository-index abstractions.

## Risks And Open Questions

- Cross-file duplication can become expensive quickly if normalization grows toward full clone detection instead of targeted evidence-rich groups.
- Magic-value and business-logic rules can drift into code-review taste unless the policy-bearing cases are defined narrowly.
- Context and robustness detectors can overfit framework expectations if the boundary model is vague.
- Broader module and class-shape rules risk overlapping existing `god_class`, `god_function`, and `mixed_concerns_function` findings unless thresholds and messages stay distinct.
- Some backlog items may still deserve a permanent non-goal outcome, and this phase should preserve that option instead of treating every deferred item as implementation debt.