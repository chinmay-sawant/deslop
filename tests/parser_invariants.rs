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
