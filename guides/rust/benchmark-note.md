# Rust Benchmark Note

## Purpose

This file records a repeatable Rust-only benchmark snapshot for the first deslop Rust rollout. It is a local sanity-check artifact, not a hard release gate.

## Target

- Benchmark target: `src`
- Benchmark command: `cargo run -- bench --json --warmups 2 --repeats 5 src`
- Matching scan snapshot: `cargo run -- scan --json src`

The benchmark target uses deslop's own Rust source tree so parser, index, and heuristic timings can be tracked against a stable in-repo baseline.

## Recorded Snapshot

### Scan counts

- Files discovered: 35
- Files analyzed: 35
- Functions found: 301
- Findings reported: 248
- Parse failures: 0
- Indexed packages: 35
- Indexed symbols: 341
- Indexed imports: 327

### Finding mix

- Warning findings: 135
- Info findings: 113
- Top rules in this snapshot:
  - `hallucinated_import_call`: 129
  - `overlong_name`: 113
  - `expect_in_non_test_code`: 5
  - `generic_name`: 1

### Benchmark timings

- Warmups: 2
- Repeats: 5
- Discover stage: min 0 ms, max 2 ms, mean 1.0 ms, median 1.0 ms
- Parse stage: min 90 ms, max 102 ms, mean 97.6 ms, median 99.0 ms
- Index stage: min 0 ms, max 0 ms, mean 0.0 ms, median 0.0 ms
- Heuristics stage: min 10 ms, max 11 ms, mean 10.6 ms, median 11.0 ms
- Total runtime: min 103 ms, max 114 ms, mean 110.4 ms, median 111.0 ms

Per-run totals from the recorded benchmark JSON:

| Run | Discover | Parse | Index | Heuristics | Total |
| --- | --- | --- | --- | --- | --- |
| 1 | 2 ms | 90 ms | 0 ms | 11 ms | 103 ms |
| 2 | 0 ms | 99 ms | 0 ms | 10 ms | 110 ms |
| 3 | 2 ms | 102 ms | 0 ms | 10 ms | 114 ms |
| 4 | 1 ms | 99 ms | 0 ms | 11 ms | 111 ms |
| 5 | 0 ms | 98 ms | 0 ms | 11 ms | 110 ms |

## Interpretation

- On this target, parse time dominates the run at roughly 88% of the mean total runtime.
- Heuristic evaluation is materially smaller than parse time on the current Rust rule pack.
- Zero parse failures on the in-repo Rust target means the current parser and file routing handle deslop's own source tree cleanly.
- The matching scan exits with status 1 because findings were reported, so the saved JSON should be treated as expected output rather than a failed benchmark run.

## Rollout Use

- Re-run this note when Rust parser behavior, Rust-local symbol resolution, or Rust heuristic coverage changes materially.
- Compare future runs against both counts and timings so performance changes are not interpreted without repository-shape context.
- Do not treat small timing shifts on this local target as release blockers until benchmark environments and additional Rust baselines are standardized.