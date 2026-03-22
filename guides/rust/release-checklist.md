# Rust Release Checklist

## Purpose

This checklist is the concrete Phase 4 release artifact for deslop's Rust backend. It is meant to be used when Rust support changes in observable ways, especially when parser evidence, local index behavior, or Rust-specific findings are added.

## Validation Commands

- Run `cargo test --test integration_scan`.
- Run full `cargo test`.
- Run `cargo build --release`.

## Required Verification Coverage

- Rust-only scan succeeds on fixture repositories.
- Mixed Go/Rust scan succeeds without a language flag.
- Malformed Rust fixtures remain recoverable and stay in the report with `syntax_error=true`.
- Rust positive and negative rule-pack fixtures cover every shipped Rust rule.
- Rust local imported-call hallucination checks cover `crate::`, `self::`, and `super::` imports.
- Rust direct-call hallucination checks cover imported function aliases and same-module direct calls.
- Mixed-language index separation still prevents Go and Rust symbols from merging in the same directory.
- Existing Go integration tests still pass.

## Documentation Sync

- Update `README.md` when the observable Rust feature set changes.
- Update `guides/features-and-detections.md` when new Rust rule IDs become user-visible.
- Update `guides/implementation-guide.md` when Rust parser or index behavior changes materially.
- Update `guides/rust/phase-4.md` when rollout criteria or benchmark conventions change.

## Benchmark Note Requirements

- Keep one repeatable Rust-only benchmark target in addition to the existing Go baseline.
- Use `cargo run -- bench --warmups 2 --repeats 5 <path>` for recorded notes.
- Record discovered files, analyzed files, functions, findings, parse failures, and stage timings together.
- Do not treat benchmark differences as hard release blockers until the benchmark target set is stable.

## Deferred Backlog To Recheck

- Cargo workspace and crate-graph awareness.
- Trait and impl resolution for stronger local-context checks.
- Async-runtime-specific heuristics.
- Allocation and clone-pattern heuristics.
- Wildcard-import or visibility-discipline rules if the project wants them later.