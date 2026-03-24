# Python Release Checklist

## Purpose

This checklist is the concrete rollout artifact for deslop's first Python backend release. Use it when Python support changes in observable ways, especially when parser evidence, Python rule-pack behavior, or mixed-language verification changes.

## Validation Commands

- Run `cargo test --test integration_scan`.
- Run full `cargo test`.
- Run `cargo build --release`.

## Required Verification Coverage

- Python-only scan succeeds on the Python fixture workspace.
- Mixed Python and Rust scan succeeds.
- Mixed Python, Go, and Rust scan succeeds.
- Same-directory mixed Python, Go, and Rust scan succeeds without symbol bleed across languages.
- Malformed Python fixtures remain recoverable and stay in the report with `syntax_error=true`.
- Python positive and negative rule-pack fixtures cover every shipped Python rule.
- Test-only Python fixtures suppress noisy non-test rules.
- Existing Go and Rust integration tests still pass.

## Benchmark Sanity Check

- Re-run the Python fixture benchmark using the command shape documented in `guides/python/benchmark-note.md`.
- Confirm the file, function, finding, and parse-failure counts stay aligned with the recorded snapshot or update the note with a reason.

## Documentation Sync

- Update `README.md` when the observable Python feature set changes.
- Update `guides/features-and-detections.md` when new Python rule IDs become user-visible.
- Update `guides/implementation-guide.md` when Python parser or heuristic routing changes materially.
- Update `guides/python/index.md`, `guides/python/heuristics-first-rule-pack.md`, and `guides/python/rollout-hardening-and-verification.md` when rollout status or backlog changes.