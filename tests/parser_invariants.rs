/// Parser invariant tests.
///
/// These tests assert properties that must hold unconditionally regardless of
/// which rule is under development:
///
///  1. Parser no-panic: `parse_source_file` must never panic on arbitrary input
///     for any registered language.  It may return an `Err` for invalid syntax;
///     it must not unwind.
///
///  2. Positive-fixture guarantee: every `_slop`, `_positive`, and
///     `_phase4_positive` fixture must produce at least one finding.
///     Spot-checked against a representative subset.
///
/// NOTE — Why there is no global "clean fixture → zero findings" invariant:
///   Fixtures named `_clean` or `_negative` are clean only for the specific
///   rule family under test.  Other unrelated rules may fire on the same
///   source.  Rule-specific cleanliness is verified by the `assert_rules_absent`
///   calls inside `tests/integration_scan/`.  A global scan assertion here would
///   produce false failures whenever a new unrelated rule is added.
use std::path::Path;

use deslop::{ScanOptions, scan_repository};
use proptest::prelude::*;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// 1. Parser no-panic via proptest
// ---------------------------------------------------------------------------

proptest! {
    /// Given any string, parsing it as Go source must not panic.
    /// The function may return Ok (with syntax_error=true) or Err, but must not unwind.
    #[test]
    fn go_parser_never_panics(source in ".*") {
        let _ = deslop::validate_source(Path::new("arb.go"), &source);
    }

    /// Same guarantee for Python.
    #[test]
    fn python_parser_never_panics(source in ".*") {
        let _ = deslop::validate_source(Path::new("arb.py"), &source);
    }

    /// Same guarantee for Rust.
    #[test]
    fn rust_parser_never_panics(source in ".*") {
        let _ = deslop::validate_source(Path::new("arb.rs"), &source);
    }
}

// ---------------------------------------------------------------------------
// Helper: write a single file into a temp dir and scan it.
// ---------------------------------------------------------------------------

fn scan_single(filename: &str, source: &str) -> Vec<String> {
    let dir = TempDir::new().expect("temp dir should be created");
    let file_path = dir.path().join(filename);
    std::fs::write(&file_path, source).expect("fixture write should succeed");

    let report = scan_repository(&ScanOptions {
        root: dir.path().to_path_buf(),
        respect_ignore: false,
    })
    .expect("scan should succeed");

    report
        .findings
        .iter()
        .map(|f| f.rule_id.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// 2. Positive-fixture invariant: Go
// ---------------------------------------------------------------------------

macro_rules! go_positive_invariant {
    ($test_name:ident, $fixture_file:literal) => {
        #[test]
        fn $test_name() {
            let source = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/go/",
                $fixture_file
            ));
            let findings = scan_single(concat!($fixture_file, ".go"), source);
            assert!(
                !findings.is_empty(),
                "positive fixture '{}' should produce at least one finding",
                $fixture_file
            );
        }
    };
}

go_positive_invariant!(go_error_handling_slop_has_findings, "error_handling_slop.txt");
go_positive_invariant!(go_context_cancel_slop_has_findings, "context_cancel_slop.txt");
go_positive_invariant!(go_concurrency_slop_has_findings, "concurrency_slop.txt");
go_positive_invariant!(go_busy_waiting_slop_has_findings, "busy_waiting_slop.txt");
go_positive_invariant!(go_db_query_slop_has_findings, "db_query_slop.txt");

// ---------------------------------------------------------------------------
// 2. Positive-fixture invariant: Python
// ---------------------------------------------------------------------------

macro_rules! python_positive_invariant {
    ($test_name:ident, $fixture_file:literal) => {
        #[test]
        fn $test_name() {
            let source = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/python/",
                $fixture_file
            ));
            let findings = scan_single(concat!($fixture_file, ".py"), source);
            assert!(
                !findings.is_empty(),
                "positive fixture '{}' should produce at least one finding",
                $fixture_file
            );
        }
    };
}

python_positive_invariant!(python_rule_pack_positive_has_findings, "rule_pack_positive.txt");
python_positive_invariant!(python_phase4_positive_has_findings, "phase4_positive.txt");

// ---------------------------------------------------------------------------
// 2. Positive-fixture invariant: Rust
// ---------------------------------------------------------------------------

macro_rules! rust_positive_invariant {
    ($test_name:ident, $fixture_file:literal) => {
        #[test]
        fn $test_name() {
            let source = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/rust/",
                $fixture_file
            ));
            let findings = scan_single(concat!($fixture_file, ".rs"), source);
            assert!(
                !findings.is_empty(),
                "positive fixture '{}' should produce at least one finding",
                $fixture_file
            );
        }
    };
}

rust_positive_invariant!(rust_rule_pack_positive_has_findings, "rule_pack_positive.txt");
rust_positive_invariant!(
    rust_direct_call_hallucination_positive_has_findings,
    "direct_call_hallucination_positive.txt"
);

// ---------------------------------------------------------------------------
// 3. Zero-findings baseline invariant: every language
//
// NOTE — These fixtures are deliberately minimal code samples that are
// globally finding-free.  They are distinct from the per-rule `_clean`
// fixtures, which are clean only for a specific rule family; other rules may
// fire on them.  The baselines here must produce zero findings from any rule.
// ---------------------------------------------------------------------------

#[test]
fn go_zero_findings_baseline_produces_no_findings() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/go/zero_findings_baseline.txt"
    ));
    let findings = scan_single("zero_findings_baseline.go", source);
    assert!(
        findings.is_empty(),
        "zero-findings Go baseline produced unexpected findings: {findings:?}"
    );
}

#[test]
fn python_zero_findings_baseline_produces_no_findings() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/python/zero_findings_baseline.txt"
    ));
    let findings = scan_single("zero_findings_baseline.py", source);
    assert!(
        findings.is_empty(),
        "zero-findings Python baseline produced unexpected findings: {findings:?}"
    );
}

#[test]
fn rust_zero_findings_baseline_produces_no_findings() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/rust/zero_findings_baseline.txt"
    ));
    let findings = scan_single("zero_findings_baseline.rs", source);
    assert!(
        findings.is_empty(),
        "zero-findings Rust baseline produced unexpected findings: {findings:?}"
    );
}

// ---------------------------------------------------------------------------
// 4. Deduplication invariant: scan results must never contain two findings
//    with the same (path, start_line, rule_id) triple.
// ---------------------------------------------------------------------------

#[test]
fn scan_results_contain_no_duplicate_findings() {
    // Use a known-noisy Go fixture so multiple rules fire, giving the
    // deduplication guard a realistic workout.
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/go/error_handling_slop.txt"
    ));
    // scan_single returns Vec<String> of rule_ids; use the full ScanReport
    // to access (start_line, rule_id) pairs for the duplicate check.
    let dir = tempfile::TempDir::new().expect("temp dir");
    let file_path = dir.path().join("dedup_check.go");
    std::fs::write(&file_path, source).expect("write fixture");

    let report = deslop::scan_repository(&deslop::ScanOptions {
        root: dir.path().to_path_buf(),
        respect_ignore: false,
    })
    .expect("scan should succeed");

    let mut seen = std::collections::BTreeSet::new();
    for f in &report.findings {
        let key = (f.start_line, f.rule_id.clone());
        assert!(
            seen.insert(key.clone()),
            "duplicate finding detected: rule='{}' at line {}",
            key.1,
            key.0
        );
    }
}
