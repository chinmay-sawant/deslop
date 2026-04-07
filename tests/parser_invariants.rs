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
#[path = "support/mod.rs"]
mod support;

use std::path::Path;

use proptest::prelude::*;

use support::scan_fixture;

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

macro_rules! positive_fixture_invariant {
    ($test_name:ident, $fixture_path:literal, $target_path:literal) => {
        #[test]
        fn $test_name() {
            let report = scan_fixture($fixture_path, $target_path);
            assert!(
                !report.findings.is_empty(),
                "positive fixture '{}' should produce at least one finding",
                $fixture_path
            );
        }
    };
}

positive_fixture_invariant!(
    go_error_handling_slop_has_findings,
    "go/error_handling_slop.txt",
    "error_handling_slop.go"
);
positive_fixture_invariant!(
    go_context_cancel_slop_has_findings,
    "go/context_cancel_slop.txt",
    "context_cancel_slop.go"
);
positive_fixture_invariant!(
    go_concurrency_slop_has_findings,
    "go/concurrency_slop.txt",
    "concurrency_slop.go"
);
positive_fixture_invariant!(
    go_busy_waiting_slop_has_findings,
    "go/busy_waiting_slop.txt",
    "busy_waiting_slop.go"
);
positive_fixture_invariant!(
    go_db_query_slop_has_findings,
    "go/db_query_slop.txt",
    "db_query_slop.go"
);
positive_fixture_invariant!(
    python_rule_pack_positive_has_findings,
    "python/rule_pack_positive.txt",
    "rule_pack_positive.py"
);
positive_fixture_invariant!(
    python_phase4_positive_has_findings,
    "python/phase4_positive.txt",
    "phase4_positive.py"
);
positive_fixture_invariant!(
    rust_rule_pack_positive_has_findings,
    "rust/rule_pack_positive.txt",
    "rule_pack_positive.rs"
);
positive_fixture_invariant!(
    rust_direct_call_hallucination_positive_has_findings,
    "rust/direct_call_hallucination_positive.txt",
    "direct_call_hallucination_positive.rs"
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
    let findings = scan_fixture("go/zero_findings_baseline.txt", "zero_findings_baseline.go")
        .findings
        .into_iter()
        .map(|finding| finding.rule_id)
        .collect::<Vec<_>>();
    assert!(
        findings.is_empty(),
        "zero-findings Go baseline produced unexpected findings: {findings:?}"
    );
}

#[test]
fn python_zero_findings_baseline_produces_no_findings() {
    let findings = scan_fixture(
        "python/zero_findings_baseline.txt",
        "zero_findings_baseline.py",
    )
    .findings
    .into_iter()
    .map(|finding| finding.rule_id)
    .collect::<Vec<_>>();
    assert!(
        findings.is_empty(),
        "zero-findings Python baseline produced unexpected findings: {findings:?}"
    );
}

#[test]
fn rust_zero_findings_baseline_produces_no_findings() {
    let findings = scan_fixture(
        "rust/zero_findings_baseline.txt",
        "zero_findings_baseline.rs",
    )
    .findings
    .into_iter()
    .map(|finding| finding.rule_id)
    .collect::<Vec<_>>();
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
    let report = scan_fixture("go/error_handling_slop.txt", "dedup_check.go");

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
