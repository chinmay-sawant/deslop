# Plan 3 - Runtime Ownership, Task Lifecycle, And Hot-Path Setup (Rust)

Date: 2026-04-05

## Status

- [x] Implemented in the Rust scan pipeline, heuristics tree, and catalog registry.
- [x] Backlog derived from shipped async/runtime rules plus additional runtime-boundary scenarios still uncovered today.
- [x] Required async-heavy fixtures and false-positive tuning are now in place.

## Objective

Extend Rust runtime support beyond the current request-path and lock-misuse rules so deslop can also flag detached task lifecycles, per-request coordination primitives, and obviously repeated runtime setup work.

## Source Inputs

- [x] `src/heuristics/rust/async_patterns.rs`
- [x] `src/heuristics/rust/runtime_boundary.rs`
- [x] `tests/integration_scan/rust_advanced.rs`
- [x] `tests/integration_scan/rust.rs`
- [x] `guides/architecture_enhancement/RUST_ARCHITECTURE_REVIEW_1.md`
- [x] `guides/architecture_enhancement/RUST_ARCHITECTURE_REVIEW_6.md`
- [x] `guides/rust/advanceplan1/newplan3.md`
- [x] `guides/rust/benchmark-note.md`

## Existing Coverage Explicitly Excluded

- [x] `rust_tokio_runtime_built_per_call`
- [x] `rust_axum_router_built_in_handler`
- [x] `rust_tonic_channel_connect_per_request`
- [x] `rust_async_spawn_cancel_at_await`
- [x] `rust_async_monopolize_executor`

## Proposed Rules

- [x] `rust_detached_spawn_without_handle`
  - Flag spawned background tasks whose handles are discarded without any obvious shutdown, join, or supervision path.
- [x] `rust_channel_created_per_request`
  - Flag channels, broadcast senders, or watch state repeatedly created on handler/request paths instead of startup ownership boundaries.
- [x] `rust_block_in_place_request_path`
  - Flag `block_in_place`, `block_on`, or similar sync bridging inside request-handling or hot-loop paths.
- [x] `rust_runtime_builder_in_loop`
  - Flag repeated `Builder::new_*` runtime setup inside loops, retries, or test harness helpers.
- [x] `rust_notify_without_shutdown_contract`
  - Flag wait/notify coordination where the waiting side has no obvious cancellation or shutdown branch nearby.
- [x] `rust_process_global_env_toggle`
  - Flag `std::env::set_var`/`remove_var` style process-global environment mutation used as a runtime feature toggle or control path.

## Parser And Evidence Work

- [x] Reused existing async evidence, macro-call summaries, and request-path markers before adding new state.
- [x] Added lightweight spawn-handle evidence only where detached-task reporting needed a clearer explanation.
- [x] Kept handler-path detection explicit and framework-biased instead of claiming deep interprocedural knowledge.

## Fixtures And Tests

- [x] Added async runtime fixtures under `tests/fixtures/rust/advanceplan3/`.
- [x] Added clean coverage for supervised tasks, startup-owned channels, and explicit shutdown paths.
- [x] Added at least one mixed fixture that intentionally triggers both a shipped runtime rule and one new backlog rule.

## Acceptance

- [x] Findings clearly show the repeated setup or unsupervised task shape.
- [x] The family complements the existing async pack instead of duplicating it.
- [x] False positives remain low for tests, small examples, and obviously one-shot scripts.
- [x] Process-global env mutation findings stay focused on runtime control flow, not benign one-time bootstrap setup.

## Non-Goals

- [ ] Proving task leaks or executor starvation.
- [ ] Modeling framework lifecycles end to end.
- [ ] Replacing runtime tracing or production observability.
