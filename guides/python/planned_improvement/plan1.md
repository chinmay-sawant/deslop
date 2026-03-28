# Plan 1 — Stronger asyncio-specific reasoning

Overview

This plan describes actionable work to add stronger, audit-ready asyncio reasoning to the Python scanning support while preserving the existing synchronous Python logic and rule behavior. The goal is a careful, incremental migration that reduces false positives and false negatives for async-related findings (for example: `blocking_sync_io_in_async`, `mixed_sync_async_module`) and adds a set of explicit checks, fix patterns, and rollout steps.

Checklist (actionable)

Use this checklist as the primary, actionable plan. Each checkbox maps to sections below that contain the full details; nothing in the original document is removed — this is a checklist overlay to drive implementation and verification.

- [ ] Preserve existing Python logic and exported API/outputs used by other parts of the system. (See `Scope & Goals`)
- [ ] Improve static detection of async-related anti-patterns and runtime misuse signals. (See `Scope & Goals`)
- [ ] Add automated checks and test coverage so new rules are verifiable and low-noise. (See `Scope & Goals`)
- [ ] Provide a migration path for repositories that mix sync and async code. (See `Scope & Goals`)

- Deliverables:
  - [ ] `guides/python/plan1.md` (this document) — plan + checklists
  - [ ] A repo-scan script or set of grep patterns to detect blocking patterns in async contexts
  - [ ] Concrete rule updates, unit tests, and integration fixtures covering async misuse
  - [ ] CI checks (grep/static analysis) that flag the most dangerous regressions
  - [ ] Migration notes and one or more example PRs showing safe fixes

- High-level approach checklist:
  - [ ] Codebase scan — identify current uses of `async`/`await`, `time.sleep`, `requests`, `subprocess.run`, synchronous DB drivers, `threading`-based code, and any existing `blocking_sync_io_in_async` signals.
  - [ ] Define concrete evidence patterns in the parser for async contexts (e.g., `async def` scopes, `await` presence, top-level `async for` / `async with`).
  - [ ] Expand the rule `blocking_sync_io_in_async` to categorize severity (hard-blocking, potential-blocking) and provide auto-fix suggestions.
  - [ ] Add checks for common anti-patterns (detailed in `Worst practices to avoid`) and provide prioritized fixes.
  - [ ] Add tests: unit (pytest-asyncio), fixture-backed integration, and quick benchmarks.
  - [ ] Add CI grep-lint and static checks; roll out incrementally behind a feature flag.


Scope & Goals

- Preserve existing Python logic and exported API/outputs used by other parts of the system.
- Improve static detection of async-related anti-patterns and runtime misuse signals.
- Add automated checks and test coverage so new rules are verifiable and low-noise.
- Provide a migration path for repositories that mix sync and async code.

Assumptions

- We must remain backward-compatible with current Python rule outputs unless there is an explicit, documented change.
- Parser evidence already extracts call sites, imports, and context (see `guides/python/index.md`).
- Target implementation should support Python 3.8+; Python 3.11 features (e.g., `asyncio.TaskGroup`) are preferred where available but fallback patterns must exist.

Deliverables

- `guides/python/plan1.md` (this document) — plan + checklists
- A repo-scan script or set of grep patterns to detect blocking patterns in async contexts
- Concrete rule updates, unit tests, and integration fixtures covering async misuse
- CI checks (grep/static analysis) that flag the most dangerous regressions
- Migration notes and one or more example PRs showing safe fixes

Checklist: Detailed tasks & checkpoints

1) Repository scan & evidence matrix
- [ ] Run automated scans (commands in checklist) to list candidate files.
- [ ] Produce a matrix mapping file → evidence types: {sync-blocking-call, async-def, mixed-imports} and save as `analysis/python-async-scan.json`.
- [ ] Checkpoint: commit `analysis/python-async-scan.json` with the matrix.

2) Parser evidence improvements
- [ ] Ensure parser tags `async_scope: true` for `async def` and nested `async` constructs.
- [ ] Extract call node metadata: callee name, module name, whether it is awaited, and lexical scope.
- [ ] Tag known blocking call names (table included in this plan).
- [ ] Checkpoint: parser tests add examples for each evidence type.

3) Rule enhancement & severity model
- [ ] Split `blocking_sync_io_in_async` into two severities (`critical`, `warning`).
- [ ] Provide automated suggestion text and example replacement snippets.
- [ ] Checkpoint: new rule tests pass and integration fixtures demonstrate reduced false positives.

4) Library-specific guidance & migration helpers
- [ ] Provide adapters and examples for common libraries (requests→aiohttp/httpx, psycopg2→asyncpg, boto3→aiobotocore, blocking file I/O→asyncio.to_thread).
- [ ] Add a small helper module in the repo (optional) with a `safe_run_in_executor` pattern used in automated fixes.

5) Testing and CI
- [ ] Add `pytest-asyncio` to test dependencies; write unit tests that exercise cancellation, timeouts, and task cleanup.
- [ ] Fixture-based integration tests: sample repo fixtures that mix sync/async to verify rule quality.
- [ ] Add CI grep-lint job to fail builds if new regressions (critical blocking calls inside async defs) are introduced.

6) Rollout & monitoring
- [ ] Roll out rule in three phases: `detect-only` → `suggest-fix` → `enforce`.
- [ ] Instrument rule with simple telemetry fields: file path, rule id, severity, sampler for high-volume repos.
- [ ] Create dashboard queries for changes in detection rates after rollout.

High-level approach

1. Codebase scan — identify current uses of `async`/`await`, `time.sleep`, `requests`, `subprocess.run`, synchronous DB drivers, `threading`-based code, and any existing `blocking_sync_io_in_async` signals.
2. Define concrete evidence patterns in the parser for async contexts (e.g., `async def` scopes, `await` presence, top-level `async for` / `async with`).
3. Expand the rule `blocking_sync_io_in_async` to categorize severity (hard-blocking, potential-blocking) and provide auto-fix suggestions.
4. Add checks for common anti-patterns (detailed below) and provide prioritized fixes.
5. Add tests: unit (pytest-asyncio), fixture-backed integration, and quick benchmarks.
6. Add CI grep-lint and static checks; roll out incrementally behind a feature flag.

Detailed tasks & checkpoints

1) Repository scan & evidence matrix
- Run automated scans (commands in checklist) to list candidate files.
- Produce a matrix mapping file → evidence types: {sync-blocking-call, async-def, mixed-imports}.
- Checkpoint: commit `analysis/python-async-scan.json` with the matrix.

2) Parser evidence improvements
- Ensure parser tags `async_scope: true` for `async def` and nested `async` constructs.
- Extract call node metadata: callee name, module name, whether it is awaited, and lexical scope.
- Tag known blocking call names (table included in this plan).
- Checkpoint: parser tests add examples for each evidence type.

3) Rule enhancement & severity model
- Split `blocking_sync_io_in_async` into two severities:
  - `critical`: blocking call inside an `async def` and on the hot path (no executor usage / not wrapped in `to_thread`).
  - `warning`: calls on code paths that might execute in worker threads or only on rare codepaths.
- Provide automated suggestion text and example replacement snippets.
- Checkpoint: new rule tests pass and integration fixtures demonstrate reduced false positives.

4) Library-specific guidance & migration helpers
- Provide adapters and examples for common libraries:
  - `requests` → `aiohttp` or `httpx` async client
  - `psycopg2` → `asyncpg` or wrapped executor usage
  - `boto3` (sync) → `aiobotocore` or executor wrapper
  - Blocking file I/O: use `asyncio.to_thread` or run in executor
- Add a small helper module in the repo (optional) with a `safe_run_in_executor` pattern used in automated fixes.

5) Testing and CI
- Add `pytest-asyncio` to test dependencies; write unit tests that exercise cancellation, timeouts, and task cleanup.
- Fixture-based integration tests: sample repo fixtures that mix sync/async to verify rule quality.
- Add CI grep-lint job to fail builds if new regressions (critical blocking calls inside async defs) are introduced.

6) Rollout & monitoring
- Roll out rule in three phases: `detect-only` → `suggest-fix` → `enforce`.
- Instrument rule with simple telemetry fields: file path, rule id, severity, sampler for high-volume repos.
- Create dashboard queries for changes in detection rates after rollout.

Worst practices to avoid (explicit anti-patterns)

- Calling blocking APIs directly inside `async def` without offloading:
  - Examples: `time.sleep`, `requests.get`, `open(...).read()` (blocking large-file read), `subprocess.run(..., check=True)`.
  - Why bad: blocks the event loop; all other coroutines stall.
  - Detection: grep patterns; parser tag for call inside `async_scope` and no `await` of an async client.

- Using `asyncio.run()` from inside an already-running event loop (re-entrancy):
  - Common in libraries accidentally used by frameworks (e.g., invoked by a running FastAPI worker).
  - Avoids nested event loops; leads to RuntimeError or surprising behavior.
  - Use: top-level script only; libraries must expose async functions.

- Swallowing `CancelledError` or broad exceptions in async tasks
  - Catching `Exception` or `BaseException` and ignoring cancellation prevents graceful shutdown.
  - Detection: catch blocks without re-raising when `asyncio.CancelledError` is present.

- Creating unlimited concurrent tasks without flow-control
  - e.g., `for url in urls: create_task(fetch(url))` with millions of URLs.
  - Use semaphores / bounded queues / TaskGroup limits.

- Using synchronous libraries for network/database in async code
  - Requests, syncronous DB drivers, some cloud SDKs.
  - Prefer async-native drivers or move blocking calls to `asyncio.to_thread` / executor.

- Blocking the loop with CPU-heavy work
  - Heavy computation should run in separate processes or via `loop.run_in_executor`.

- Relying on `asyncio.get_event_loop()` in library code
  - Use `asyncio.get_running_loop()` inside coroutines; only the application bootstrap code should manage event loop creation.

- Using `ensure_future` / `create_task` without tracking or cancelling tasks
  - Tasks must be tracked for shutdown and exceptions read (or they may get swallowed).

Checklist: Worst practices to detect and remediate

- [ ] Calling blocking APIs directly inside `async def` without offloading (time.sleep, requests.get, open().read(), subprocess.run).
  - [ ] Detection: grep patterns; parser tag for call inside `async_scope` and no `await` of an async client.
  - [ ] Remediation: replace with async equivalent or `await asyncio.to_thread(...)`.

- [ ] Using `asyncio.run()` from inside an already-running event loop.
  - [ ] Detection: grep for `asyncio.run(` in library code.
  - [ ] Remediation: expose async interface; only call `asyncio.run` at top-level scripts.

- [ ] Swallowing `CancelledError` or broad exceptions in async tasks.
  - [ ] Detection: catch blocks missing `CancelledError` re-raise.
  - [ ] Remediation: re-raise `CancelledError` and handle cleanup in `finally`.

- [ ] Creating unlimited concurrent tasks without flow-control.
  - [ ] Detection: unbounded `create_task` loops.
  - [ ] Remediation: use semaphores, bounded queues, or TaskGroup limits.

- [ ] Using synchronous libraries for network/database in async code.
  - [ ] Detection: `requests`, `psycopg2`, `pymysql`, `mysqlclient` usages in async contexts.
  - [ ] Remediation: adopt async drivers or run in executor with timeouts.

- [ ] Blocking the loop with CPU-heavy work.
  - [ ] Detection: heavy CPU loops or large CPU-bound functions invoked in coroutines.
  - [ ] Remediation: move to processes or `run_in_executor`.

- [ ] Relying on `asyncio.get_event_loop()` in library code.
  - [ ] Detection: grep for `get_event_loop()` in library modules.
  - [ ] Remediation: use `asyncio.get_running_loop()` inside coroutines.

- [ ] Using `ensure_future` / `create_task` without tracking or cancelling tasks.
  - [ ] Detection: `create_task(` sites with no task tracking or awaiting.
  - [ ] Remediation: track tasks and ensure proper cancellation and exception handling.

Best-practices checklist — what to check in existing code (and how to detect/fix)

For each check below, the right-hand `How to detect` shows quick grep/static checks and `How to fix` gives the recommended remediation.

- Check: Blocking calls inside `async def`
  - How to detect: parser evidence mapping call node inside `async_scope` OR grep for `time.sleep`/`requests.`/`subprocess.run` inside files with `async def`.
    - Example commands:
      - `grep -R "async def" -n | cut -d: -f1 | xargs -I{} grep -n "time.sleep\|requests\.\|subprocess\.run\|open(" {} || true`
  - How to fix: replace with async equivalent or dispatch to thread via `await asyncio.to_thread(blocking_call, ...)`.

- Check: Synchronous HTTP client usage
  - How to detect: `grep -R "requests\.\|urllib\.request" -n`
  - How to fix: use `aiohttp` or `httpx` (async mode), or document `to_thread` usage.

- Check: Direct DB driver usage that is synchronous
  - How to detect: `grep -R "psycopg2\|pymysql\|mysqlclient" -n`
  - How to fix: adopt `asyncpg` / `databases` or run DB calls in an executor with clear timeouts.

- Check: Missing timeouts and incorrect `gather` usage
  - How to detect: `grep -R "asyncio.gather(" -n` and inspect for `return_exceptions` and surrounding timeouts.
  - How to fix: use `asyncio.wait_for` for per-task timeouts, limit concurrency.

- Check: Using `asyncio.run` in library code
  - How to detect: `grep -R "asyncio.run(" -n`
  - How to fix: expose an async interface; provide a synchronous wrapper only in CLI main scripts.

- Check: Dropped task exceptions
  - How to detect: search for `create_task(` and check if the resulting Task is awaited or tracked; run tests with `asyncio` debug mode.
  - How to fix: gather tasks and `await` them, or attach exception handlers.

- Check: Cancellation safety
  - How to detect: `grep -R "except Exception" -n` inside async functions; look for missing handling of `CancelledError`.
  - How to fix: re-raise `CancelledError` and handle cleanup in `finally` blocks.

- Check: Using `time.sleep` in async code
  - How to detect: `grep -R "time.sleep" -n`
  - How to fix: `await asyncio.sleep()`.

Quick actionable detection commands (run these to populate the evidence matrix):

```sh
# Find `async def` files that reference blocking calls
grep -R "async def" -n | cut -d: -f1 | sort -u | xargs -I{} sh -c "grep -n \"time.sleep\|requests\.\|subprocess\.run\|open(\" {} || true"

# Find direct uses of `requests`/sync DB drivers
grep -R --line-number "\(requests\|psycopg2\|pymysql\|mysqlclient\)" || true

# Find suspicious uses of `asyncio.run` in library files
grep -R --line-number "asyncio.run(" || true
```

Concrete detection rules and sample grep commands (copyable)

- Find `async def` files that reference blocking calls:

  grep -R "async def" -n | cut -d: -f1 | sort -u | xargs -I{} sh -c "grep -n "time.sleep\|requests\.\|subprocess\.run\|open(" {} || true"

- Find direct uses of `requests`/sync DB drivers:

  grep -R --line-number "\(requests\|psycopg2\|pymysql\|mysqlclient\)" || true

- Find suspicious uses of `asyncio.run` in library files:

  grep -R --line-number "asyncio.run(" || true

Implementation examples & safe patterns

- Offload blocking call using `asyncio.to_thread` (Python 3.9+):

```py
async def read_big_file(path):
    data = await asyncio.to_thread(lambda: open(path, 'rb').read())
    return data
```

- Concurrency limit pattern (semaphore + gather):

```py
async def _wrap(sem, coro):
    async with sem:
        return await coro

async def gather_with_limit(coros, limit=100):
    sem = asyncio.Semaphore(limit)
    return await asyncio.gather(*[_wrap(sem, c) for c in coros])
```

- Cancellation-safe worker pattern:

```py
async def worker(q):
    try:
        while True:
            item = await q.get()
            try:
                await process(item)
            finally:
                q.task_done()
    except asyncio.CancelledError:
        # cleanup if needed
        raise
```

- Proper top-level bootstrap (only top-level uses `asyncio.run`):

```py
async def main():
    await run_app()

if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
```

Testing & verification

- Unit tests: `pytest-asyncio` for `async def` unit tests. Cover cancellation and timeout behavior.
- Integration tests: fixture repos under `tests/fixtures/python/` that intentionally mix sync/async patterns to validate detection.
- Fuzz/bench tests: lightweight throughput tests that measure task creation and event-loop latency in critical code paths.
- CI checks: grep-based scan to block PRs that introduce new `critical` blocking patterns.

Testing checklist (apply and mark when done):

- [ ] Add `pytest-asyncio` to `tests` dependencies.
- [ ] Add unit tests for `async_scope` evidence extraction.
- [ ] Add unit tests for `blocking_sync_io_in_async` critical and warning cases.
- [ ] Add fixture-based integration tests under `tests/fixtures/python/`.
- [ ] Add CI job for grep-lint blocking `critical` regressions (initially detect-only).

Observability and runtime checks

- Enable `asyncio` debug mode during test runs: `asyncio.get_event_loop().set_debug(True)` to detect slow callbacks and long-running tasks.
- Add instrumentation points to the scanner: how many `async def` functions seen, how many blocking calls found, top 10 offending modules.
- Create dashboards for rule volume and per-repo signal.

Migration & rollout strategy

1. Phase `detect-only` — add new parser evidence and run rules in detect-only mode for a week. Collect FP/FN cases.
2. Phase `suggest-fix` — include automated suggested code snippets in findings; show guidance in the UI.
3. Phase `enforce` — optionally escalate to a higher severity after manual review of the collected data.

Acceptance criteria

- Parser tags `async_scope` present and verified by unit tests for at least one representative file per pattern.
- `blocking_sync_io_in_async` has test coverage for critical and warning cases and demonstrates lower false-positive rate on fixture set.
- CI scan warns on new critical patterns in PRs (detect-only initially).
- Documentation updated in `guides/python` summarizing the rules and remediation steps.

Risks and mitigations

- Risk: Rule too aggressive and noisy → Mitigation: run detect-only phase and gather FP/FN, provide easy opt-out per-repo.
- Risk: Breaking backwards compatibility → Mitigation: keep prior rule behavior until confirmed by tests and rollout.
- Risk: Large migration effort for many repos using sync libs → Mitigation: provide executor wrappers, example PRs, and small-step migration guides.

Next steps (immediate)

- Run the quick grep commands in this plan to produce a list of candidate files.
- Create parser tests covering a short list of async patterns and blocking calls.
- Draft a small example PR that replaces `requests.get` in one fixture with `aiohttp` or `asyncio.to_thread` wrapped call.

Immediate checklist

- [ ] Run the quick grep commands in this plan to produce a list of candidate files and save results to `analysis/python-async-scan.json`.
- [ ] Create parser test templates for `async_scope` and blocking-call evidence.
- [ ] Draft an example PR replacing one `requests.get` in a fixture with `asyncio.to_thread` or `aiohttp`.

---

Notes

- This checklist overlay preserves every section and detail from the original document; it simply provides a compact, actionable set of boxes to drive implementation, tests, and rollout.


If you want, I can now:
- run the recommended grep scans and produce the evidence matrix, or
- create parser test templates and example PR snippets for one sample fixture.

