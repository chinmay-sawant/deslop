# Plan 4 — Framework-specific rule packs: Django & FastAPI

## Checklist-Based Plan

This checklist maps the full plan into actionable items. All original details remain below unchanged.

### High-level deliverables

- [ ] Review `src/analysis/python` and confirm integration points.
- [ ] Define canonical rule manifest schema and filesystem layout.
- [ ] Create rule pack folders: `rules/python/django/` and `rules/python/fastapi/`.
- [ ] Author sample rule manifests (e.g., `PY-DJ-001`, `PY-FA-001`).
- [ ] Add good/bad example code under each pack's `examples/`.
- [ ] Add unit/integration tests under `rules/.../tests/` and `tests/fixtures/python/{django,fastapi}`.
- [ ] Implement `RulePackLoader` and integrate with existing engine.
- [ ] Add feature flag `--enable-framework-packs` and configuration for opt-in scanning.
- [ ] Run scanner against fixtures; iterate on false-positives and tune confidence.
- [ ] Update CI to run rule-pack tests and add nightly trial runs.
- [ ] Finalize docs and open PR.

### Implementation plan (detailed)

#### A — Rule pack layout and manifest

- [ ] Create `rules/python/<framework>/rules/` for YAML/JSON rule files.
- [ ] Add `examples/` and `tests/` directories for each pack.
- [ ] Define per-rule schema fields: `id`, `title`, `description`, `frameworks`, `severity`, `detector`, `patterns`, `tests`, `remediation`.
- [ ] Provide at least one example manifest snippet per pack.

#### B — Integration with existing Python logic

- [ ] Locate rule registration, detector interfaces, AST helpers, and test loader in `src/analysis/python`.
- [ ] Implement `RulePackLoader` to discover and register rules from `rules/python/*`.
- [ ] Ensure backward-compatibility and add a config/flag to enable packs.

#### C — Detection strategies

- [ ] Implement AST-based detectors for structural checks.
- [ ] Implement regex/text detectors for simple heuristics.
- [ ] Plan semi-semantic checks for imports/usage (decorators, middleware).
- [ ] Defer deep taint analysis to a future scope, mark as `semantic` detector type.

#### D — False-positive mitigation

- [ ] Add `confidence` and `explain` fields to rule manifests.
- [ ] Add project-level suppression mechanism (filename/glob or comment annotation).
- [ ] Provide clear good/bad examples for unit tests.

### Worst practices — Django (author one rule per item)

- [ ] PY-DJ-001 Hard-coded `SECRET_KEY` in settings: create manifest + examples + tests + remediation.
- [ ] PY-DJ-002 `DEBUG = True` in production: create manifest + examples + tests + remediation.
- [ ] PY-DJ-003 `ALLOWED_HOSTS = ['*']` or empty: create manifest + examples + tests + remediation.
- [ ] PY-DJ-004 Missing/disabled CSRF (`CsrfViewMiddleware` / `@csrf_exempt`): create manifest + examples + tests + remediation.
- [ ] PY-DJ-005 Raw SQL with string formatting: create manifest + examples + tests + remediation.
- [ ] PY-DJ-006 Insecure file uploads / unsafe path joins: create manifest + examples + tests + remediation.
- [ ] PY-DJ-007 Weak password hashers in `PASSWORD_HASHERS`: create manifest + examples + tests + remediation.
- [ ] PY-DJ-008 Debug/exception info in logs/responses: create manifest + examples + tests + remediation.
- [ ] PY-DJ-009 Missing security middleware (SecurityMiddleware, XFrameOptions): create manifest + examples + tests + remediation.
- [ ] PY-DJ-010 Unsafe templating / `mark_safe`: create manifest + examples + tests + remediation.

### Worst practices — FastAPI (author one rule per item)

- [ ] PY-FA-001 Missing request validation (no Pydantic models): create manifest + examples + tests + remediation.
- [ ] PY-FA-002 Unrestricted CORS (`allow_origins=['*']`) in prod: create manifest + examples + tests + remediation.
- [ ] PY-FA-003 Debug server or exception exposure (`uvicorn.reload`, `debug=True`): create manifest + examples + tests + remediation.
- [ ] PY-FA-004 Unsafe JWT handling (missing alg/signature checks): create manifest + examples + tests + remediation.
- [ ] PY-FA-005 Use of `eval()`/`exec()` on request data: create manifest + examples + tests + remediation.
- [ ] PY-FA-006 Serving static/media from unvalidated paths: create manifest + examples + tests + remediation.
- [ ] PY-FA-007 Unbounded file uploads / missing validation: create manifest + examples + tests + remediation.
- [ ] PY-FA-008 Insecure dependency injection / arbitrary callables: create manifest + examples + tests + remediation.

### Best-practice checks to add

- [ ] Parameterized DB queries / ORM usage detection (flag formatted SQL in `execute()`).
- [ ] Secrets-from-env detection (flag literal assignments to known secret keys).
- [ ] Debug-flag detection (flag `DEBUG = True`, `reload=True`, `app.debug = True`).
- [ ] CORS config validation (flag `allow_origins=['*']` in non-dev contexts).
- [ ] CSRF presence check in Django `MIDDLEWARE`.
- [ ] Pydantic schema enforcement for FastAPI endpoints.
- [ ] Insecure password-hasher detection in Django `PASSWORD_HASHERS`.
- [ ] Templating-escape bypass detection (`mark_safe`, `safe` filters).

### Rule development workflow (checklist)

- [ ] Author rule manifest under `rules/python/<framework>/rules/<id>.yaml`.
- [ ] Add `examples/bad/*` and `examples/good/*` sample files.
- [ ] Reference examples in `tests` entries within the manifest.
- [ ] Add unit tests under `rules/.../tests/` and `tests/fixtures/python/...`.
- [ ] Run local scanner and iterate until false positives acceptable.

### Tests & CI

- [ ] Add fixtures in `tests/fixtures/python/django/` and `tests/fixtures/python/fastapi/`.
- [ ] Add unit tests that assert `bad` examples are flagged and `good` are not.
- [ ] Update CI config to include rule pack discovery and test execution.
- [ ] Add nightly CI job to run framework packs behind feature flag for one-week trial.

### Rollout

- [ ] Release behind `--enable-framework-packs` feature flag.
- [ ] Run trial for one week; collect telemetry on rule firings.
- [ ] Tune rules and false-positive rates based on telemetry.
- [ ] Enable by default after successful trial.

### Appendix & next steps

- [ ] Add example AST detectors for SECRET_KEY, DEBUG, ALLOWED_HOSTS, raw SQL patterns.
- [ ] Draft canonical YAML/JSON manifest schema file.
- [ ] Implement the two sample rules (Django + FastAPI) and tests.

---

## Overview

Purpose: add framework-specific rule packs for Django and FastAPI while preserving and integrating with the repository's existing Python analysis logic. The new rule packs must be: (1) easy to maintain, (2) expressive enough for framework-specific patterns, (3) backward-compatible with existing detectors.

Scope: design, documentation, one example rule for each framework, test fixtures, CI checks, and a rollout plan to integrate into the current scanner pipeline (see `src/analysis/python`).

## Goals

- Provide a canonical rule pack layout for `django` and `fastapi` checks.
- Define a rule manifest/schema that supports metadata, severity, framework tags, detector types, and tests.
- Enumerate the worst practices (concrete insecure patterns) to avoid for each framework and specify concrete detectors for them.
- Provide best-practice checks that the repository does not yet enforce and how to detect them.
- Implement one sample rule for Django and one for FastAPI, with unit/integration tests.

## Assumptions & constraints

- The project already contains Python analysis code at `src/analysis/python` (review required). New rule packs should integrate with that loader/engine.
- Rule detection capabilities available: AST-based detectors, regex/text detectors, and configuration checks. If more advanced data-flow/taint analyses are required, they will be scoped separately.
- Tests should reuse the existing `tests/fixtures` pattern and CI job(s).

## Deliverables

- `guides/python/plan4.md` (this document)
- Rule pack folders: `rules/python/django/` and `rules/python/fastapi/` with rule manifests, examples, and tests
- Implementation: two sample rules (one Django, one FastAPI) wired into the loader
- Test fixtures and unit/integration tests under `tests/fixtures/python/{django,fastapi}`
- Updated docs and a short rollout checklist

## High-level timeline & milestones

- Day 0: Review `src/analysis/python` and confirm integration points.
- Day 1: Define rule manifest schema and filesystem layout.
- Day 2–3: Enumerate worst practices & best-practice checks; author rule YAMLs.
- Day 4–6: Implement two sample rules and tests; run CI and iterate.
- Day 7: Finalize docs and open PR.

## Implementation plan (detailed)

### A. Rule pack layout and manifest

Recommended filesystem layout (under repository root):

- `rules/python/django/` — rule pack directory
  - `rules/` — YAML/JSON definitions per rule
  - `examples/` — good/bad code samples
  - `tests/` — unit test cases referencing fixtures

- `rules/python/fastapi/` — same layout as above

Rule manifest (per-rule, concise schema):

- `id` — canonical id (e.g., `PY-DJ-001`)
- `title` — short title
- `description` — what it detects and why it's insecure
- `frameworks` — [`django`] or [`fastapi`]
- `severity` — `LOW|MEDIUM|HIGH|CRITICAL`
- `detector` — type: `ast`, `regex`, `config`, `semantic` with a detector payload
- `patterns` — AST patterns, regexes or pseudo-code for detection
- `tests` — list of example files marked `bad` / `good`
- `remediation` — brief remediation steps

Example manifest snippet (pseudo-YAML):

```yaml
id: PY-DJ-001
title: Hard-coded SECRET_KEY in Django settings
frameworks: [django]
severity: CRITICAL
detector:
  type: ast
  match:
    - file_name: settings.py
    - assign_target: SECRET_KEY
    - value_is_literal: true
remediation: Move SECRET_KEY to environment variables and validate length.
tests:
  - examples/bad/settings_secret_literal.py
  - examples/good/settings_env_secret.py
```

### B. Integration with existing Python logic

- Review `src/analysis/python` to locate: rule registration, detector interfaces, AST helpers, and test loader.
- Add a `RulePackLoader` that can discover `rules/python/*` packs and register rules with the existing engine.
- Ensure backward compatibility by making pack discovery opt-in via config (e.g., `--enable-framework-packs`), defaulting on for deep scans.

### C. Detection strategies (practical guidance)

- AST detectors: preferred for structural checks (assignments to SECRET_KEY, DEBUG flag, decorator usage, presence/absence of middleware).
- Regex/text detectors: acceptable for config or small heuristics (check for ALLOWED_HOSTS = ['*']).
- Semi-semantic checks: require scanning imports/usage (e.g., presence of `csrf_exempt` decorators or missing `CsrfViewMiddleware`).
- Behavioral/taint checks: marking as future improvement; initial set should avoid deep interprocedural TAINT unless already supported.

### D. False-positive mitigation

- Provide `confidence` and `explain` fields on rules.
- Allow project-level suppression with explicit filename/glob and comment annotations.
- Supply good/bad examples for each rule to validate test coverage.

## Worst practices to avoid — Django (detection heuristics + remediation)

1. Hard-coded `SECRET_KEY` in settings
   - Detection: AST assignment to `SECRET_KEY` where value is a literal string in `settings*.py`.
   - Severity: CRITICAL
   - Remediation: use environment variable `DJANGO_SECRET_KEY` and `django.core.management.utils.get_random_secret_key()` for generation.

2. `DEBUG = True` in production
   - Detection: AST assignment `DEBUG = True` and absence of an env-driven override or check for `if DEBUG` gating.
   - Severity: CRITICAL
   - Remediation: set `DEBUG=False` in production and gate on environment variables.

3. `ALLOWED_HOSTS = ['*']` or empty ALLOWED_HOSTS
   - Detection: literal `'*'` in list assigned to `ALLOWED_HOSTS` or empty list/tuple.
   - Severity: HIGH
   - Remediation: enumerate expected hosts or configure via env.

4. CSRF disabled / missing CsrfViewMiddleware
   - Detection: absence of `'django.middleware.csrf.CsrfViewMiddleware'` in `MIDDLEWARE` or use of `@csrf_exempt` decorators on views.
   - Severity: HIGH
   - Remediation: ensure CSRF middleware is included and only selectively exempt safe endpoints.

5. Using raw SQL with string formatting
   - Detection: calls like `cursor.execute(f"... {var} ...")` or string concatenation passed to DB execute methods.
   - Severity: CRITICAL
   - Remediation: use parameterized queries or Django ORM query APIs.

6. Insecure file uploads and path joins
   - Detection: use of `os.path.join(MEDIA_ROOT, user_input)` without sanitization, or unsanitized `open()` using request data.
   - Severity: HIGH
   - Remediation: validate filenames, normalize paths, check content types, store with safe names.

7. Using weak password hashers
   - Detection: settings referencing `MD5PasswordHasher` or `Unsalted` hashers.
   - Severity: HIGH
   - Remediation: prefer `Argon2PasswordHasher`/`PBKDF2PasswordHasher`.

8. Debug/exception info in logs or responses
   - Detection: presence of `DEBUG=True` combined with handlers that render exceptions, or logging of request bodies.
   - Severity: HIGH
   - Remediation: sanitize logs and disable debug output in production.

9. Missing security middleware (SecurityMiddleware, XFrameOptions)
   - Detection: absence of `django.middleware.security.SecurityMiddleware` or `X_FRAME_OPTIONS` not set.
   - Severity: MEDIUM

10. Unsafe templating / use of `mark_safe` on untrusted data
    - Detection: calls to `mark_safe()` with user-influenced strings, or `render_to_string()` without sanitization.
    - Severity: HIGH

## Worst practices to avoid — FastAPI (detection heuristics + remediation)

1. Missing request validation (no Pydantic models)
   - Detection: route handlers using raw `dict` / `Request` body parsing without typed `BaseModel` annotations.
   - Severity: HIGH
   - Remediation: use Pydantic models for request validation.

2. Unrestricted CORS (`allow_origins=['*']` in production)
   - Detection: `CORSMiddleware` configured with `allow_origins=['*']` AND environment not clearly marked as dev.
   - Severity: HIGH

3. Debug server or exception exposure in production
   - Detection: `uvicorn.run(..., reload=True)` or `debug=True` flags in app startup.
   - Severity: CRITICAL

4. Unsafe JWT handling
   - Detection: verifying JWTs without algorithm check, using `alg='none'` or missing signature verification steps.
   - Severity: CRITICAL

5. Using `eval()` or `exec()` on request data
   - Detection: direct calls to `eval()`/`exec()` with request-sourced data.
   - Severity: CRITICAL

6. Serving static/media files from unvalidated user-specified paths
   - Detection: `StaticFiles(directory=...)` where `directory` is influenced by request input.
   - Severity: HIGH

7. Unbounded file upload sizes and missing content validation
   - Detection: route accepts file uploads but no size/content checks or streaming safety logic.
   - Severity: HIGH

8. Insecure dependency injection or permitting arbitrary callables
   - Detection: constructing dependencies from untrusted data or trusting class names to import.
   - Severity: HIGH

## Best practices to check (concrete checks to implement)

Common checks that are often missing and should be added for both frameworks:

- Enforce use of parameterized DB queries / ORM usage. Detector: find `execute()` calls with formatted strings.
- Ensure secrets and keys come from environment/config stores, not literals. Detector: literal assignments to common key names.
- Detect debug flags and explicit exceptions/tracebacks in prod builds. Detector: `DEBUG = True`, `reload=True` or `app.debug = True`.
- Validate CORS settings do not use `*` in production. Detector: `allow_origins` with `*` and no dev marker.
- Check for presence of CSRF protections in Django stacks. Detector: look in `MIDDLEWARE` for CSRF middleware.
- Enforce Pydantic schemas for FastAPI request bodies. Detector: endpoint parameters typed to `BaseModel` vs raw types.
- Detect insecure password-hasher settings in Django `PASSWORD_HASHERS`.
- Detect `mark_safe`, `safe` template filters, and other templating escapes bypasses.

For each of the above, include: detection code (AST match), confidence level, and recommended remediation message.

## Rule development workflow

1. Author rule manifest in `rules/python/<framework>/rules/<id>.yaml` with metadata and detector payload.
2. Add good/bad examples to `rules/python/<framework>/examples/` and reference them from the manifest `tests`.
3. Add unit tests under `rules/python/<framework>/tests/` that the test runner will pick up (reuse patterns found in `tests/`).
4. Run local scanner against fixtures and iterate until false-positive rate acceptable.

## Tests and CI

- Add fixtures in `tests/fixtures/python/django/` and `tests/fixtures/python/fastapi/` containing minimal apps demonstrating both bad and good examples.
- Add unit tests that execute the detector and assert it flags `bad` examples and not `good` ones.
- Update CI config to include rule pack discovery and test execution for these fixtures.

## Rollout plan

1. Implement packs behind feature flag `--enable-framework-packs` and run in CI nightly.
2. Collect telemetry on rule firings (counts, source files) to prioritize tuning.
3. After one-week trial with tuned false-positives, enable by default for full scans.

## Appendix: Example detectors (patterns / heuristics)

- SECRET_KEY (Django): AST pattern `Assign(targets=[Name(id='SECRET_KEY', ...)], value=Constant(value=str))` AND file path matching `settings`.
- DEBUG (Django): `Assign(targets=[Name(id='DEBUG')], value=Constant(value=True))` OR `NameConstant(True)` depending on AST.
- ALLOWED_HOSTS star: `Assign(... value=List(elts=[Constant('*')]) )`.
- Raw SQL injection (both): `Call(func=Attribute(value=Name(id='cursor'), attr='execute'), args=[BinOp or JoinedStr])`.

## Next steps (actionable)

1. I will review `src/analysis/python` and confirm detector API and test harness. (next)
2. Draft the rule manifest schema as canonical YAML/JSON and create the pack folders.
3. Implement one representative rule for Django and one for FastAPI with tests and examples.

---
Last updated: plan4.md (author: security rule engineer) — ready to implement.
