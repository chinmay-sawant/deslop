# AI is flooding your Go, Python, and Rust code with slop. Deslop finds it in seconds.

deslop is a Rust-based static analyzer for Go, Python, and Rust repositories that focuses on signals commonly associated with low-context AI-generated code. It currently scans a repository, parses Go, Python, and Rust files with tree-sitter, extracts structural fingerprints for each function, builds a lightweight local package index, runs early heuristic checks, and can benchmark the pipeline against real repositories.


## Commands

Run a scan against a target path:

```bash
cargo run -- scan /path/to/repo
```

deslop auto-detects supported source files under that path. The same command works for Go-only repositories, Python-only repositories, Rust-only repositories, and mixed-language repositories.

By default, scan output prints the scan summary plus the standard finding set. Pass `--details` when you want the per-function fingerprint breakdown alongside the normal findings.

Repository-local scan behavior can be tuned with a `.deslop.toml` file at the scan root:

```toml
go_semantic_experimental = false
rust_async_experimental = true
disabled_rules = ["panic_macro_leftover"]
suppressed_paths = ["tests/fixtures"]

[severity_overrides]
expect_in_non_test_code = "error"
```

`go_semantic_experimental = true` enables the opt-in deeper semantic Go heuristics such as nested-loop allocation/string-build checks and stronger nested N+1 escalation. 
`rust_async_experimental = false` disables the Rust async rule pack for that repository. 
`disabled_rules` removes matching rule ids entirely, 
`suppressed_paths` filters findings under matching relative path prefixes after analysis, and 
`severity_overrides` rewrites the emitted severity after analysis.

You can also ignore rule ids for a single scan invocation without changing repository config:

```bash
cargo run -- scan --ignore hallucinated_import_call,hallucinated_local_call /path/to/repo
```

`--ignore` applies after analysis and only affects the emitted findings for that command.

Run the same scan with JSON output:

```bash
cargo run -- scan --json /path/to/repo
```

Enable the opt-in deeper semantic Go checks for a single run:

```bash
cargo run -- scan --enable-semantic /path/to/go-repo
```

Show full per-function fingerprint details in either text or JSON output:

```bash
cargo run -- scan --details /path/to/repo
cargo run -- scan --json --details /path/to/repo
```

Write scan output directly to a file:

```bash
cargo run -- scan /path/to/repo > results.txt
cargo run -- scan /home/chinmay/ChinmayPersonalProjects/gopdfsuit > gopdfsuit_results.txt
cargo run -- scan /home/chinmay/ChinmayPersonalProjects/SnapBack > snapback_results.txt
cargo run -- scan --json /path/to/repo > results.txt
```

Run a scan without `.gitignore` filtering:

```bash
cargo run -- scan --no-ignore /path/to/repo
```

Benchmark the current pipeline against a real local repository:

```bash
cargo run -- bench /path/to/repo
```

Benchmark with explicit repeats and warmups:

```bash
cargo run -- bench --warmups 2 --repeats 5 /path/to/repo
```

Benchmark with JSON output:

```bash
cargo run -- bench --json /path/to/repo
```

List rules from the central registry:

```bash
cargo run -- rules
cargo run -- rules --language go
cargo run -- rules --status experimental --json
```

## Rule Inventory

<!-- GENERATED_RULE_SUMMARY_START -->
deslop now publishes a central rule registry that drives the CLI and the synced docs surfaces.

| Language | Stable | Experimental | Research | Total |
| --- | ---: | ---: | ---: | ---: |
| common | 11 | 0 | 0 | 11 |
| go | 312 | 2 | 0 | 314 |
| python | 212 | 0 | 0 | 212 |
| rust | 88 | 12 | 0 | 100 |
| total | 623 | 14 | 0 | 637 |

The totals above are language-scoped rule entries, so a shared rule ID implemented in more than one backend appears in each relevant language bucket.
The registry is now the source of truth for `deslop rules`, the frontend rule catalog, and the generated detection inventory guide.
<!-- GENERATED_RULE_SUMMARY_END -->

## GitHub Action

Use deslop directly in GitHub Actions without installing Rust. The action downloads the matching release binary for the current runner, adds it to the PATH, and runs either `deslop scan` or `deslop bench`.

Scan the checked out repository with the defaults:

<!-- GENERATED_ACTION_SCAN_EXAMPLE_START -->
```yaml
name: Deslop

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: chinmay-sawant/deslop@v0.1.0
        with:
          path: .
```
<!-- GENERATED_ACTION_SCAN_EXAMPLE_END -->

Emit JSON and include per-function fingerprint details:

<!-- GENERATED_ACTION_JSON_EXAMPLE_START -->
```yaml
- uses: actions/checkout@v4
- uses: chinmay-sawant/deslop@v0.1.0
  with:
    path: .
    json: 'true'
    details: 'true'
    fail-on-findings: 'false'
```
<!-- GENERATED_ACTION_JSON_EXAMPLE_END -->

Run a benchmark job instead of a scan:

<!-- GENERATED_ACTION_BENCH_EXAMPLE_START -->
```yaml
- uses: actions/checkout@v4
- uses: chinmay-sawant/deslop@v0.1.0
  with:
    command: bench
    path: .
    repeats: '10'
    warmups: '2'
```
<!-- GENERATED_ACTION_BENCH_EXAMPLE_END -->

Inputs:

<!-- GENERATED_ACTION_INPUTS_START -->
- `version`: Release tag to install, for example v0.1.0. Defaults to the current action ref when it is a full release tag, otherwise latest. Optional.
- `command`: Subcommand to run. Supported values are scan and bench. Defaults to `scan`. Optional.
- `path`: Path to the repository to analyze. Defaults to `.`. Optional.
- `json`: Emit JSON output. Defaults to `false`. Optional.
- `details`: Include full per-function fingerprint details in scan output. Applies only to the scan command. Defaults to `false`. Optional.
- `no-ignore`: Scan without respecting .gitignore. Defaults to `false`. Optional.
- `enable-semantic`: Enable the opt-in deeper semantic Go heuristics. Defaults to `false`. Optional.
- `fail-on-findings`: Exit with a non-zero status code when scan findings are present. Applies only to the scan command. Defaults to `true`. Optional.
- `repeats`: Benchmark repeat count. Applies only to the bench command. Defaults to `5`. Optional.
- `warmups`: Benchmark warmup count. Applies only to the bench command. Defaults to `1`. Optional.
<!-- GENERATED_ACTION_INPUTS_END -->

## Recent Go additions

- Wrapper propagation now covers receiver-field clients, local wrapper chains, and `Query` versus `QueryContext`-style mismatches when a function already accepts `context.Context`.
- Functions that intentionally detach from request context can document that boundary and avoid the propagation warning.
- The opt-in semantic Go pack adds `likely_n_squared_allocation`, `likely_n_squared_string_concat`, and stronger nested-loop correlation for `n_plus_one_query`.

## Development

Run the test suite:

```bash
cargo test
```

Validate the documentation sync and corpus manifest:

```bash
python3 scripts/sync_docs.py --check
python3 scripts/corpus_harness.py validate
```

List or run the real-repository evaluation corpus:

```bash
python3 scripts/corpus_harness.py list
python3 scripts/corpus_harness.py run --target gopdfsuit --scan
python3 scripts/corpus_harness.py run --target gopdfsuit --bench
```

Expand a saved findings report into review-ready code context:

```bash
python3 scripts/extract_finding_context.py temp_gopdfsuit.txt
```

That command reads the `path:line` entries from `temp_gopdfsuit.txt`, extracts the requested code context, and rewrites `scripts/temp.txt` with one consolidated block per finding. By default each block only includes:

- `Source`
- `Rule description`
- `Auto triage note`
- `Code`

If you want the full metadata-rich output again, pass `--details`:

```bash
python3 scripts/extract_finding_context.py temp_gopdfsuit.txt --details
```

Run the repo-local scripts through one shared entrypoint:

```bash
make run-scripts
```

`run-scripts` executes the normal repo-local utility scripts and validates installer scripts in a safe non-installing mode.

Build release executables for your current platform or cross-compile for other supported platforms:

```bash
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu
```

If you are cross-compiling, make sure the matching Rust target is installed first. Adjust the target triple to match the architecture you want to ship:

```bash
rustup target add x86_64-pc-windows-gnu x86_64-apple-darwin x86_64-unknown-linux-gnu
```

The native release binary is written to `target/release/`. Cross-compiled binaries are written under `target/<target-triple>/release/` and are named `deslop` on Unix-like systems and `deslop.exe` on Windows.

For a detailed architecture and roadmap guide, see `guides/implementation-guide.md`.
For the corpus workflow and promotion contract, see `guides/evaluation-and-promotion-policy.md`.
For a detector-oriented overview, see `guides/features-and-detections.md`.

Library code uses typed errors internally and keeps `anyhow` at the CLI edge. The scanner also uses bounded file reads by default so repository scans do not rely on unbounded `read_to_string` calls.

Rust scan hardening now also canonicalizes the scan root, rejects symlinked file reads, and compares the generated Rust security baseline report against the committed baseline in CI.
