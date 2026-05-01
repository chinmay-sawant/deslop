# Python Rule Fixture Coverage Plan

Generated on 2026-04-30 from a repo scan plus 4 parallel subagent mapping passes.

## Execution Update (2026-05-01)

This plan has now been executed as a full manual rewrite pass.

- Total subagents used: `10` (run in two waves because of active-agent limits).
- Rewrite scope: all `1382` Python fixture files under `tests/fixtures/python/rule_coverage/`.
- Ownership model: deterministic manifest slices from `reports/python_fixture_manifest.tsv`, non-overlapping by line range.
- Authoring constraint followed: no generator script and no mass template script for fixture creation.
- Verification:
  - Fixture pair presence unchanged: `691` positive + `691` negative.
  - Syntax compile check across all fixture files: `1382` files, `0` compile errors.
  - Legacy metadata tuple pattern removed from fixture corpus.

Execution completed phase-wise using the same sequencing principles in this document, but delivered as a single coordinated rewrite sprint across ten workers.

## Executive Summary

- Python catalog total: `691` rules.
- Existing Python rule-coverage fixtures on disk: `1382` files.
- Existing fixture pair coverage: `691` positive + `691` negative = `691` complete rule pairs.
- Existing behavioral integration coverage:
  - positive assertions: `125` rules
  - negative assertions: `82` rules
  - both positive and negative assertions: `79` rules
  - missing at least one behavioral side: `612` rules

## Important Clarification

We do **not** have a missing-file problem for Python anymore.

We already have:

- `691` `_positive.txt` files
- `691` `_negative.txt` files
- `1382` total files under `tests/fixtures/python/rule_coverage/`

The real problem is quality:

- the current Python generator emits metadata-only placeholder functions
- positive and negative files are almost identical
- the files do not model the rule trigger shape
- the current Python coverage test only verifies file existence, so placeholders pass

In short: the Python rule-coverage corpus is numerically complete but **semantically invalid**.

## Go Baseline To Copy

The Go path is now the right model for Python:

- real code, not metadata tuples
- concrete imports and small domain scaffolds
- one localized risky pattern in the positive file
- one same-context safe rewrite in the negative file
- fixture text that looks like believable source, not generated placeholder prose

Representative Go style:

- `tests/fixtures/go/rule_coverage/performance/sync_once_do_inside_loop_positive.txt`

## Family Map

| Family | Rules | Missing Both Behavioral Sides | Planning Notes |
|---|---:|---:|---|
| `performance` | 129 | 120 | Largest uncovered slice; needs reusable batching and hot-path scaffolds. |
| `discipline` | 70 | 61 | Best left late after shared test/error/type motifs exist. |
| `observability` | 67 | 56 | Broad and heterogeneous; easier after handler/service scaffolds exist. |
| `boundaries` | 64 | 57 | Concrete sink/guard rules; good once realistic inputs/sinks are available. |
| `architecture` | 54 | 46 | Needs believable layered modules, async/task ownership, repo/service/view roles. |
| `framework` | 51 | 51 | Zero real behavioral coverage; easy early win with real FastAPI/Flask/Django/Celery snippets. |
| `mlops` | 45 | 45 | Zero real behavioral coverage; useful early because its scaffolds help adjacent families. |
| `hot_path_ext` | 41 | 41 | Zero both-side coverage; good once loop/cache/serialization motifs exist. |
| `quality` | 39 | 22 | Mid-complexity; shares motifs with async, config, parsing, and side-effect rules. |
| `maintainability` | 38 | 32 | Good mid-rollout family after we establish concrete Python helper/module scaffolds. |
| `hot_path` | 37 | 32 | Early candidate; mostly tight function-local patterns. |
| `structure` | 27 | 20 | Good early/mid family; mostly threshold and file-shape motifs. |
| `packaging` | 16 | 16 | Must use mini-package trees, not single-file placeholders. |
| `duplication` | 8 | 8 | Small and straightforward; useful early smoke test for multi-file setup. |
| `ai_smells` | 5 | 5 | Fastest smoke test for real code instead of placeholders. |

## Why The Current Python Fixtures Are Invalid

Every sampled Python rule-coverage file follows the same pattern:

```python
def positive_some_rule():
    rule_id = "some_rule"
    family = "..."
    severity = "..."
    status = "..."
    intent = "..."
    description = "..."
    return rule_id, family, severity, status, intent, description
```

That fails as real coverage because:

1. It does not contain the syntax or APIs the Rust heuristics inspect.
2. Positive and negative variants do not differ in behavioral shape.
3. Families like `framework`, `packaging`, `mlops`, `boundaries`, and `architecture` need imports, call sites, sinks, package trees, or multi-file context that the current files never provide.
4. Some rules are structurally impossible to test with the current placeholders:
   - repo/layout rules in `packaging`
   - repeated/cross-file rules in `duplication`
   - size/complexity rules in `structure`
   - framework-gated rules in `framework`
   - async/perf/runtime motifs in `hot_path`, `hot_path_ext`, `performance`, and `mlops`

## Foundation Work Before Phase 1

This should be treated as a required pre-phase.

### Phase 0: Generator And Validation Foundation

1. Replace `scripts/generate_python_rule_fixtures.py` with a scenario-based generator modeled on `scripts/generate_go_rule_fixtures.py`.
2. Introduce reusable Python scaffolds instead of one universal metadata template:
   - FastAPI/Flask handler
   - Django/SQLAlchemy repository and ORM query
   - service/repository/domain layered mini-slice
   - pandas/numpy transform
   - ML inference/training loop
   - async worker/background task
   - package tree / `pyproject.toml` / `__init__.py`
   - observability/logging/metrics handler
3. Add polarity switches so positive and negative files are meaningfully different.
4. Strengthen Python fixture validation so it fails on placeholder stubs:
   - must parse as Python
   - must not match the old metadata-only template
   - positive and negative must differ materially
   - packaging fixtures may span multiple coordinated files
5. Keep `scripts/audit_python_rule_test_coverage.py` and the integration guard, but extend them to verify semantic fixture validity, not just file existence.

## Scenario Catalog By Slice

### Early Scaffolds

- `ai_smells`
  - overlong docstrings
  - narration comments
  - mixed naming conventions
  - heavy unused imports

- `framework`
  - FastAPI/Flask sync handlers doing blocking I/O
  - Django queryset misuse
  - SQLAlchemy session/query misuse
  - Celery/Click/Typer task and command anti-patterns

- `packaging`
  - import-time side effects
  - bad `__init__.py` exports
  - `pyproject.toml` issues
  - cross-package internal imports

- `mlops`
  - model/tokenizer built per request
  - `iterrows`, repeated concat/copy, non-vectorized pandas patterns
  - prompt/API calls in loops
  - inference without `eval()` / `no_grad()`

### Mid Scaffolds

- `hot_path`
  - regex compiled in loops
  - repeated env/config lookups
  - list materialization and append+sort patterns
  - local imports in hot helpers

- `hot_path_ext`
  - repeated same-arg calls
  - nested rescans
  - parse/serialize loops
  - copy and invariant recomputation inside loops

- `duplication`
  - repeated validation pipelines
  - repeated SQL fragments
  - repeated error/log wrappers
  - cross-file copy-paste helpers

- `structure`
  - god functions/classes
  - monolithic modules
  - heavy constructors
  - vague manager/processor abstractions

- `quality`
  - untracked tasks
  - lock held across unrelated `await`
  - unsafe loaders and missing timeouts
  - import-time side effects

- `maintainability`
  - helper sinks
  - debug leftovers
  - passthrough wrappers
  - side-effect comprehensions
  - hardcoded repeated policy values

### Late Scaffolds

- `boundaries`
  - f-string SQL
  - SSRF/open redirect/path traversal
  - pickle/YAML/XML/JWT/Jinja hazards
  - secret/config hygiene
  - mutable defaults and contract-boundary issues

- `architecture`
  - request/response objects leaking into services
  - repositories returning raw ORM query objects
  - handlers building ORM or raw SQL directly
  - domain objects doing transport or external I/O
  - task/concurrency lifecycle ownership issues

- `observability`
  - f-string logging
  - missing `exc_info`
  - slow health checks
  - metrics cardinality issues
  - trace/log correlation drift

- `discipline`
  - broad exception handling
  - `raise` without chaining
  - boolean flag APIs
  - test smells
  - return-shape drift
  - type misuse and setup leakage

## Recommended Delivery Plan

Rule-pair cap per content phase: `100` rules = `200` files (`100` positive + `100` negative).

That means the Python rollout is:

- `6` full phases of `100` rule pairs
- `1` final phase of `91` rule pairs

### Phase 1: `100` Rule Pairs / `200` Files

- `framework` = 51
- `packaging` = 16
- `ai_smells` = 5
- `duplication` = 8
- `structure` = 20

Why first:

- fastest path away from fake placeholders
- real imports, comments, docstrings, decorators, and package trees immediately prove the new generator is producing actual code

### Phase 2: `100` Rule Pairs / `200` Files

- remaining `structure` = 7
- `mlops` = 45
- `hot_path` = 37
- first `quality` = 11

Why second:

- establishes realistic Python execution-path scaffolds
- reuses data/loop/import/config patterns that will also help `performance` and `observability`

### Phase 3: `100` Rule Pairs / `200` Files

- remaining `quality` = 28
- `hot_path_ext` = 41
- `maintainability` = 31

Why third:

- converts the function-local and module-local families once the core scaffolds are stable

### Phase 4: `100` Rule Pairs / `200` Files

- remaining `maintainability` = 7
- `boundaries` = 64
- first `architecture` = 29

Why fourth:

- security/resource/contract edge cases benefit from the service/repo/handler scaffolds already established in earlier phases

### Phase 5: `100` Rule Pairs / `200` Files

- remaining `architecture` = 25
- first `performance` = 75

Why fifth:

- performance is the largest family and should start only after hot-path and ML/data-path motifs are already proven

### Phase 6: `100` Rule Pairs / `200` Files

- remaining `performance` = 54
- first `observability` = 46

Why sixth:

- performance and observability share handler, loop, batching, and logging scaffolds

### Phase 7: `91` Rule Pairs / `182` Files

- remaining `observability` = 21
- `discipline` = 70

Why last:

- `discipline` is broad, context-heavy, and benefits the most from all prior scaffolds already existing

## Success Criteria Per Phase

Each phase should only be marked complete when all of the following are true:

1. All files in the batch are real Python scenario fixtures, not metadata stubs.
2. Positive and negative variants differ by a concrete risky-vs-safe implementation change.
3. Fixtures parse as Python.
4. Family-specific imports or package-tree context are present where required.
5. `scripts/audit_python_rule_test_coverage.py` still reports full fixture-pair presence.
6. The stronger Python integration guard rejects the old placeholder template.

## Final Recommendation

Treat this as a **fixture regeneration** project, not a fixture creation project.

The file count is already done.

The actual remaining work is:

1. replace all `691` placeholder rule pairs with concrete scenario pairs
2. upgrade the generator to a Go-style scenario builder
3. strengthen validation so placeholder files can never pass again
4. deliver the rewrite in `7` content phases after one foundation phase

## Manual Rewrite Status

The Python fixture tree has now been manually rewritten in the workspace.

- Existing `.txt` files under `tests/fixtures/python/rule_coverage/` were replaced with hand-written Python scenario text.
- The current fixture audit still reports `691` positive files and `691` negative files.
- Representative families now contain real source-shaped content rather than metadata tuples.
- Worker verification reported successful syntax compilation across the fixture tree.
