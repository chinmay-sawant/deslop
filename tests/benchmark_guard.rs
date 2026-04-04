/// Benchmark regression guard.
///
/// This test is `#[ignore]` by default and does not run during `cargo test`.
/// Run it explicitly with:
///
///   cargo test --release -- --ignored benchmark_go_gin_example_within_baseline
///
/// The test scans `real-repos/go-gin-example` and asserts that the total scan
/// wall-clock time stays within the recorded baseline of **10 seconds** at 1×
/// warmup / 1× measurement pass.  The threshold is intentionally generous (2×
/// the expected sub-second wall time on modern hardware) to avoid flaky CI
/// failures while still catching catastrophic regressions.
///
/// To update the baseline:
/// 1. Run the test in release mode and note the reported duration.
/// 2. Update `BASELINE_MILLIS` below to the observed value.
/// 3. Commit the change with a note explaining the hardware / workload change.
use std::path::PathBuf;
use std::time::Instant;

use deslop::{BenchmarkOptions, benchmark_repository};

/// Maximum allowed wall-clock time in milliseconds for one scan pass.
/// Set to 10 000 ms (10 s) — roughly 10× the expected duration on a
/// modern developer workstation — to catch catastrophic regressions only.
const CEILING_MILLIS: u128 = 10_000;

#[test]
#[ignore = "benchmark: run explicitly with `cargo test --release -- --ignored`"]
fn benchmark_go_gin_example_within_baseline() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("real-repos")
        .join("go-gin-example");

    assert!(
        repo_root.exists(),
        "benchmark fixture missing: {}",
        repo_root.display()
    );

    let options = BenchmarkOptions {
        root: repo_root.clone(),
        repeats: 1,
        warmups: 1,
        respect_ignore: false,
    };

    let start = Instant::now();
    let report = benchmark_repository(&options)
        .expect("benchmark_repository should succeed on go-gin-example");
    let elapsed_ms = start.elapsed().as_millis();

    println!(
        "benchmark: {} files, {} functions, {} findings, {}ms (ceiling {}ms)",
        report.files_analyzed,
        report.functions_found,
        report.findings_found,
        elapsed_ms,
        CEILING_MILLIS,
    );

    assert!(
        elapsed_ms <= CEILING_MILLIS,
        "scan of go-gin-example took {}ms, which exceeds the {}ms ceiling — \
         investigate for a performance regression",
        elapsed_ms,
        CEILING_MILLIS,
    );
}
