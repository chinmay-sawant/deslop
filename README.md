# AI is flooding your Go, Python, and Rust code with slop. Deslop finds it in seconds.

deslop is a Rust-based static analyzer for Go, Python, and Rust repositories that focuses on signals commonly associated with low-context AI-generated code. It currently scans a repository, parses Go, Python, and Rust files with tree-sitter, extracts structural fingerprints for each function, builds a lightweight local package index, runs early heuristic checks, and can benchmark the pipeline against real repositories.


## Commands

Run a scan against a target path:

```bash
cargo run -- scan /path/to/repo
```

deslop auto-detects supported source files under that path. The same command works for Go-only repositories, Python-only repositories, Rust-only repositories, and mixed-language repositories.

By default, scan output prints the scan summary plus the standard finding set. Detail-only diagnostics such as `full_dataset_load` are held back unless you pass `--details`.

Repository-local scan behavior can be tuned with a `.deslop.toml` file at the scan root:

```toml
rust_async_experimental = true
disabled_rules = ["panic_macro_leftover"]

[severity_overrides]
expect_in_non_test_code = "error"
```

`rust_async_experimental = false` disables the Rust async rule pack for that repository. `disabled_rules` removes matching rule ids entirely, and `severity_overrides` rewrites the emitted severity after analysis.

Run the same scan with JSON output:

```bash
cargo run -- scan --json /path/to/repo
```

Show full per-function fingerprint details and detail-only findings in either text or JSON output:

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

## GitHub Action

Use deslop directly in GitHub Actions without installing Rust. The action downloads the matching release binary for the current runner, adds it to the PATH, and runs either `deslop scan` or `deslop bench`.

Scan the checked out repository with the defaults:

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

Emit JSON and include detail-only findings:

```yaml
- uses: actions/checkout@v4
- uses: chinmay-sawant/deslop@v0.1.0
	with:
		path: .
		json: 'true'
		details: 'true'
```

Run a benchmark job instead of a scan:

```yaml
- uses: actions/checkout@v4
- uses: chinmay-sawant/deslop@v0.1.0
	with:
		command: bench
		path: .
		repeats: '10'
		warmups: '2'
```

Inputs:

- `version`: Release tag to install, for example `v0.1.0`. When omitted, deslop uses the action ref if it is a full release tag such as `v0.1.0`; otherwise it downloads the latest release binary.
- `command`: `scan` or `bench`. Defaults to `scan`.
- `path`: Path to the repository you want to analyze. Defaults to `.`.
- `json`: Set to `true` to emit JSON output.
- `details`: Set to `true` to include detail-only findings for `scan`.
- `no-ignore`: Set to `true` to ignore `.gitignore` filtering.
- `repeats`: Benchmark repeat count for `bench`. Defaults to `5`.
- `warmups`: Benchmark warmup count for `bench`. Defaults to `1`.

## Development

Run the test suite:

```bash
cargo test
```

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
For a detector-oriented overview, see `guides/features-and-detections.md`.

Library code uses typed errors internally and keeps `anyhow` at the CLI edge. The scanner also uses bounded file reads by default so repository scans do not rely on unbounded `read_to_string` calls.

Rust scan hardening now also canonicalizes the scan root, rejects symlinked file reads, and compares the generated Rust security baseline report against the committed baseline in CI.
