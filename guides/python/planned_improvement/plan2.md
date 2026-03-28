# Python Type Annotation Inference — Plan 2

Date: 2026-03-29

## Checklist-based plan

Use this checklist as the primary entry point for work; all items expand into the detailed plan below. Nothing from the original plan has been removed — the full original content follows after the checklist.

- [ ] Discovery & inventory
  - [ ] Run quick inventory commands (count python files, annotated functions, `Any`, `type: ignore`)
  - [ ] Create inventory CSV: module, functions, annotated?, tests covering?
  - [ ] Identify dynamic hotspots (`exec`, `eval`, `setattr`, metaclasses)
  - [ ] Baseline CI: run `pytest` and the project's test suite

- [ ] Define inference strategy
  - [ ] Evaluate static-only (AST) trade-offs
  - [ ] Evaluate dynamic-only (tracing) trade-offs
  - [ ] Choose hybrid approach (recommended) and document rules

- [ ] Prototype inference engine (static prototype)
  - [ ] Implement AST visitor to collect signatures and literal assignment types
  - [ ] Infer simple return types from `return` statements
  - [ ] Emit `.pyi` for a single target module and run `mypy --pyi`

- [ ] Build runtime tracing harness
  - [ ] Implement tracer harness (pytest plugin or wrapper)
  - [ ] Record observed argument and return types during tests
  - [ ] Merge runtime evidence with static analysis

- [ ] Merge results & generator
  - [ ] Define confidence rules for merging static/runtime results
  - [ ] Emit `.pyi` stubs to `build/inferred-stubs/`
  - [ ] Produce human-readable `report/` describing decisions and confidence

- [ ] Validate with mypy & tests
  - [ ] Run `mypy` and `pyright` checks with generated stubs
  - [ ] Run full test suite and compare outputs with baseline
  - [ ] Flag and hold low-confidence/ambiguous annotations for manual review

- [ ] Incremental apply & rollout
  - [ ] Apply inferred annotations to low-risk modules first
  - [ ] Create small PRs per module with `report/` attached
  - [ ] Require at least one human reviewer per PR

- [ ] CI integration & enforcement
  - [ ] Add advisory mypy/pyright checks to CI (non-blocking initially)
  - [ ] Gradually make checks blocking as confidence grows

- [ ] Documentation, best-practices, and automation
  - [ ] Publish reviewer checklist and automation scripts
  - [ ] Create `tools/infer_types.py` CLI prototype (dry-run, stubs-only, apply)
  - [ ] Add `scripts/collect-typing-metrics.sh` for baseline metrics

- [ ] Deliverables and follow-up
  - [ ] `guides/python/plan2.md` (this file)
  - [ ] `tools/infer_types.py` prototype
  - [ ] `build/inferred-stubs/` and `report/` artifacts

<!-- Detailed original plan (unchanged) follows -->

## Executive summary

This document defines a detailed plan to implement *type annotation inference* for the repository's Python code. The objective is to support existing Python logic unchanged while adding inferred type annotations (as stubs or inline hints), integrate checks into CI, and provide a safe, reviewable rollout path.

This plan prioritizes safety and reproducibility: inference produces recommendations (stubs by default), CI validation runs before any annotations are applied, and runtime behavior must remain identical.

## Scope

- Target code areas (initial): `src/analysis/python/`, `heuristics/python/`, `tests/fixtures/python/` and other files under `src/` that are Python implementations or bindings.
- Tests and fixtures used by the project (so inferred types reflect real runtime usage).
- A first-phase rollout will target low-risk modules (pure-Python utility modules), then progress to higher-risk modules.

If a module uses heavy metaprogramming (eval/exec/dynamic attribute creation), it will be marked as "manual review required" and skipped by automated inference.

## Goals

- Produce safe, mypy/pyright-compatible annotations (stubs `.pyi` initially).
- Preserve runtime behavior; all existing tests must pass unchanged.
- Provide a reproducible inference pipeline: `--dry-run` output with generated stubs, `--apply` to commit changes.
- Integrate automated checks into CI (mypy/pyright, tests).
- Provide a best-practice checklist and automatic detectors for common anti-patterns.

## Constraints & assumptions

- Repo uses Python 3.10+ semantics (verify during discovery). Use generic typing compatible with project's minimum supported Python.
- Existing tests cover the runtime behaviors sufficiently for tracing-based inference; if coverage is poor, add targeted tests/fixtures.
- We will not introduce runtime type checks or change execution semantics in any automatic pass.

## Discovery & inventory (first week)

1. Run a quick inventory of Python code and current typing signals.

    ```sh
    # Count python files
    find src -name "*.py" | wc -l

    # Count annotated functions / occurrences of '->'
    grep -R "def .*->" src | wc -l

    # Count 'Any' and 'type: ignore' uses
    grep -R "\bAny\b" src | wc -l
    grep -R "type: ignore" src | wc -l
    ```

2. Create an inventory CSV: module, functions, annotated? (yes/no), tests covering? (yes/no). This will guide phased rollout.

3. Identify dynamic hotspots: modules using `exec`, `eval`, `setattr`, heavy meta-class usage. Mark these as "manual review".

4. Baseline CI: run `pytest` and the project's test suite; ensure green before proceeding.

## Proposed inference approaches (trade-offs)

- Static-only (AST-based) inference
  - Pros: no runtime execution required; deterministic.
  - Cons: incomplete for dynamic features; harder for inter-procedural inference.

- Dynamic-only (runtime tracing)
  - Pros: accurate types for observed runs; easier to map to real behavior.
  - Cons: only covers executed paths; needs good test coverage.

- Hybrid (recommended)
  - Static pass to gather structure + conservative propagation.
  - Runtime tracing during tests to disambiguate ambiguous cases and confirm unions.
  - Fallback rules: where uncertain, emit `Any` in stubs and flag for review.

Recommendation: implement a hybrid pipeline. Generate `.pyi` stubs by default, allow review, then optionally apply inline annotations.

## High-level design

- `tools/infer_types.py` CLI (or `scripts/`): discover modules, run static analysis, optionally run tests with tracer, merge results, emit stubs or patch files.
- Modes: `--dry-run`, `--stubs-only`, `--apply`, `--level safe|medium|aggressive`.
- Output: `build/inferred-stubs/` and a `report/` with diffs and confidence scores per symbol.
- Safety net: all `--apply` patches must pass a pre-commit validation (full test suite, mypy in chosen config).

## Detailed implementation roadmap

Phase 0 — Preparation (1–3 days)

- Create a feature branch and ensure tests run locally.
- Add basic linter and type checkers to CI in non-blocking mode (example: `mypy --ignore-missing-imports` as advisory).
- Add `scripts/collect-typing-metrics.sh` that runs the inventory commands above and produces a baseline report.

Phase 1 — Static prototype (3–5 days)

- Implement an AST visitor using `ast` + `typed_ast` if needed. Key tasks:
  - Collect function signatures (names, args, defaults).
  - Collect assignment types for literals (int/str/float/bool/None), container literals (list/dict/tuple) with element types.
  - Infer simple return types from `return` statements that return literals or variable names with known types.
  - Generate conservative `typing` annotations: `int`, `str`, `list[int]`, `dict[str, int]`, `Optional[T]`, `Union[...]`.

- Produce `.pyi` for a single target module and run `mypy --pyi` against it to verify basic consistency.

Phase 2 — Runtime tracing (4–7 days)

- Implement a tracer harness (pytest plugin or wrapper) that records observed types at function entry/exit and for important assignments.
- Minimal tracer approach (example): write a small wrapper that decorates functions under test to record argument types and return types using `type()` and `typing.get_origin`/`get_args` on annotated containers.

  - Example tracer sketch (for tests only):

    ```py
    import inspect
    from collections import defaultdict

    observed = defaultdict(lambda: {"args": [], "returns": []})

    def record_call(func):
        def wrapped(*a, **kw):
            args_types = tuple(type(x) for x in a)
            kw_types = {k: type(v) for k, v in kw.items()}
            r = func(*a, **kw)
            observed[func.__qualname__]["args"].append((args_types, kw_types))
            observed[func.__qualname__]["returns"].append(type(r))
            return r
        return wrapped
    ```

- Use the tracer during test runs to collect representative runtime types. Merge runtime evidence with static analysis results.

- For union/optional detection: if a parameter sometimes receives `None`, annotate `Optional[T]`.

Phase 3 — Merge & generator (3–5 days)

- Merge static and runtime results with confidence rules (e.g., runtime evidence > 1 occurrence => higher confidence).
- Emit `.pyi` stubs under `build/inferred-stubs/` and a human-readable `report/` describing decisions and confidence levels.

Phase 4 — Validation and safety (2–4 days)

- Run `mypy` and `pyright` checks with generated stubs.
- Run the full test suite; compare outputs with baseline. If any test changes or errors occur, abort and mark affected symbols for manual review.

Phase 5 — Incremental apply & rollout (ongoing)

- First apply to low-risk modules. Create smaller PRs per module.
- Use review checklist (below) and require at least one human reviewer to accept inferred changes.

Phase 6 — CI enforcement (2–3 days)

- Add blocking checks gradually: first block regressions in tests, then block new `Any` increases, then block certain type errors.

## Best-practices checklist (automatable)

For each module/PR, run checks and require passing or documented justification.

- **Public API typed**: public functions/classes in `module.__all__` or not prefixed with `_` should have signatures and return types.
  - How to check: `grep -R "def \|class " src | xargs -n1 python -c '...'` (script that flags unannotated public defs).

- **Minimal use of `Any`**: avoid `Any` for public APIs.
  - How to check: `grep -R "\bAny\b" src | wc -l` and fail if count increases.

- **Limit `type: ignore`**: every `type: ignore` occurrence must have a comment explaining why and reference a ticket.

- **Use `TypedDict` / `dataclass`** when dictionaries represent structured records.

- **Prefer `list[T]` / `dict[K, V]` (PEP 585)** when the minimum Python version allows it.

- **Use `Optional[T]` for `None`-able values**, not `Union[T, NoneType]` or raw `Any`.

- **Annotations must be mypy/pyright-friendly**: run both with repo settings and ensure no new errors at chosen strictness level.

- **Document inferred annotations in PR description**: include `report/` snippet and confidence for reviewers.

## Worst practices to avoid (detailed) — DO NOT do these

1. Blindly annotating with `Any` everywhere
   - Why bad: defeats the purpose of typing; hides errors.
   - How to detect: large spikes in `Any` occurrences after inference; grep for `Any`.
   - Remediation: mark as `Any` only with explicit reasons and add a follow-up task to refine.

2. Replacing runtime checks or adding type-enforcement code
   - Why bad: changes runtime behavior and can break subtle logic.
   - Remediation: inference must be non-invasive. Do not add `assert isinstance(...)` or runtime validators automatically.

3. Over-eager aggressive inference without trace evidence
   - Why bad: static guesses can be wrong for polymorphic code.
   - Remediation: require higher confidence for inline annotations; low-confidence cases should remain as stubs or marked for review.

4. Applying mass automatic edits in a single large commit
   - Why bad: hard to review and revert.
   - Remediation: produce small, per-module PRs with generated `report/` and test runs attached.

5. Ignoring dynamic code and forcing annotations there
   - Why bad: dynamic code may never match static annotations.
   - Remediation: detect `exec/eval/setattr` usage and skip automatic annotation; add manual tasks.

6. Relying solely on ML inference without deterministic checks
   - Why bad: ML may hallucinate types and generate incorrect annotations.
   - Remediation: ML suggestions only as human-reviewed hints; require test validation.

7. Using `type: ignore` as a shortcut for mismatches
   - Why bad: hides real typing errors; proliferates technical debt.
   - Remediation: require comment and ticket for each `type: ignore`.

## Detection snippets / quick checks

Use these to find problematic patterns before applying annotations.

```sh
# Find dynamic hotspots
grep -R "exec\|eval\|setattr\|__getattr__" src || true

# Count 'Any' occurrences
grep -R "\bAny\b" src | wc -l

# List files with many unannotated defs (heuristic)
python - <<'PY'
import ast, sys, pathlib
for p in pathlib.Path('src').rglob('*.py'):
    src = p.read_text()
    tree = ast.parse(src)
    unann = 0
    for n in ast.walk(tree):
        if isinstance(n, ast.FunctionDef):
            if n.returns is None:
                unann += 1
    if unann>0:
        print(p, unann)
PY
```

## Example inference pseudocode (AST + simple propagation)

1. Parse file with `ast.parse()`.
2. First pass: collect symbol table of literals and simple assignments (e.g., `x = 1` -> `int`).
3. Second pass: for each `FunctionDef`, collect candidate argument types from default values and global usage.
4. Optional: merge runtime types if tracer data exists.
5. Emit `typing` annotation string for each symbol with confidence level.

## Validation criteria (Definition of Done)

- Generated stubs exist under `build/inferred-stubs/` with a human-readable `report/` for every changed module.
- No change in test results after applying stubs and re-running the suite (baseline == post-inference).
- `mypy` run at repository configured level shows no *new* errors for annotated modules (initially use non-blocking strictness, gradually tighten).
- PRs are created per module with diff, report, and at least one approving reviewer.

## Risks & mitigations

- Risk: insufficient test coverage => incomplete runtime inference.
  - Mitigation: add targeted tests/fixtures before tracing; annotate only with static inference and mark for review.

- Risk: incorrect annotations cause churn.
  - Mitigation: generate `.pyi` stubs first and require manual acceptance.

- Risk: community friction over large automated PRs.
  - Mitigation: small PRs, clear reports, and opt-in aggressive modes.

## Deliverables

- `guides/python/plan2.md` (this document).
- `tools/infer_types.py` CLI prototype (dry-run + stubs output).
- `build/inferred-stubs/` with initial module stubs.
- `report/` artifacts showing decisions and confidence.

## Timeline (rough)

- Week 0: Discovery & baseline (inventory, CI baseline).
- Week 1: Static prototype + single-module demo.
- Week 2: Runtime tracer + merge logic.
- Week 3: Validation, small rollout, CI additions.

## Next steps (immediate)

1. Confirm target directories: if you want a different target than the defaults, list them.
2. I will create a small static prototype that emits a `.pyi` for one low-risk module and attach the `report/`.
3. After your review, enable the tracer on CI in `--dry-run` mode to collect runtime evidence.

---

Appendix: Quick reviewer checklist (for PRs containing inferred types)

- **Does the PR include a `report/` that explains each inferred symbol?**
- **Do tests pass with the stubs applied?**
- **Are any uses of `Any` justified and tracked?**
- **Were dynamic hotspots skipped or manually reviewed?**
- **Is the scope of change small and reviewable (one module per PR)?**
