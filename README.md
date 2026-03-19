# goslop

goslop is a Rust-based static analyzer for Go repositories that focuses on signals commonly associated with low-context AI-generated code. It currently scans a repository, parses Go files with tree-sitter-go, extracts structural fingerprints for each function, builds a lightweight local package index, runs early heuristic checks, and can benchmark the pipeline against real Go repositories.

## Overview

The current implementation is optimized around a fast full-repository pass:

- walk the target tree with `.gitignore` awareness
- skip common generated-code inputs and `vendor/` paths
- parse Go files with tree-sitter-go
- fingerprint functions and methods with lightweight structural metrics
- flag generic naming, overlong naming, weak typing, comment-style slop, weak crypto usage, early error-handling anti-patterns, conservative missing-context cases, looped string concatenation, and raw goroutine coordination gaps
- use a local package index to catch some unresolved repository-local calls
- benchmark discovery, parse, index, heuristic, and total runtime stages

## Commands

Run a scan against a target path:

```bash
cargo run -- scan /path/to/go-repo
```

By default, scan output prints the scan summary plus findings only.

Run the same scan with JSON output:

```bash
cargo run -- scan --json /path/to/go-repo
```

Show full per-function fingerprint details in either text or JSON output:

```bash
cargo run -- scan --details /path/to/go-repo
cargo run -- scan --json --details /path/to/go-repo
```

Write scan output directly to a file:

```bash
cargo run -- scan /path/to/go-repo > results.txt
cargo run -- scan /home/chinmay/ChinmayPersonalProjects/gopdfsuit > results.txt
cargo run -- scan --json /path/to/go-repo > results.txt
```

Run a scan without `.gitignore` filtering:

```bash
cargo run -- scan --no-ignore /path/to/go-repo
```

Benchmark the current pipeline against a real local Go repository:

```bash
cargo run -- bench /path/to/go-repo
```

Benchmark with explicit repeats and warmups:

```bash
cargo run -- bench --warmups 2 --repeats 5 /path/to/go-repo
```

Benchmark with JSON output:

```bash
cargo run -- bench --json /path/to/go-repo
```

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

The native release binary is written to `target/release/`. Cross-compiled binaries are written under `target/<target-triple>/release/` and are named `goslop` on Unix-like systems and `goslop.exe` on Windows.

For a detailed architecture and roadmap guide, see `guides/implementation-guide.md`.
For a detector-oriented overview, see `guides/features-and-detections.md`.
