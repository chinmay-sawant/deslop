use crate::analysis::{ParsedFile, ParsedFunction, UnsafePatternKind};
use crate::model::{Finding, Severity};

use super::function_finding;

pub(crate) fn unsafe_soundness_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for pattern in &function.unsafe_soundness {
        let (rule_id, severity, hint) = match pattern.kind {
            UnsafePatternKind::GetUnchecked => (
                "rust_unsafe_get_unchecked",
                Severity::Warning,
                "document and prove bounds on every path before using get_unchecked",
            ),
            UnsafePatternKind::RawParts => (
                "rust_unsafe_from_raw_parts",
                Severity::Warning,
                "raw slice construction requires strong lifetime and length invariants",
            ),
            UnsafePatternKind::SetLen => (
                "rust_unsafe_set_len",
                Severity::Warning,
                "Vec::set_len requires initialized elements and correct capacity invariants",
            ),
            UnsafePatternKind::AssumeInit => (
                "rust_unsafe_assume_init",
                Severity::Warning,
                "MaybeUninit::assume_init must only run after full initialization",
            ),
            UnsafePatternKind::Transmute => (
                "rust_unsafe_transmute",
                Severity::Error,
                "transmute needs layout, validity, and thread-safety proof",
            ),
            UnsafePatternKind::RawPointerCast => (
                "rust_unsafe_raw_pointer_cast",
                Severity::Warning,
                "raw pointer casts require aliasing and lifetime guarantees",
            ),
        };

        findings.push(function_finding(
            file,
            function,
            rule_id,
            severity,
            pattern.line,
            format!("function {} uses an unsafe soundness-sensitive operation", function.fingerprint.name),
            vec![format!("unsafe detail: {}", pattern.detail), hint.to_string()],
        ));
    }

    if !function.unsafe_lines.is_empty()
        && function.body_text.contains("UnsafeCell")
        && function.body_text.contains("&mut")
    {
        findings.push(function_finding(
            file,
            function,
            "rust_unsafe_aliasing_assumption",
            Severity::Warning,
            function.unsafe_lines[0],
            format!("function {} mixes unsafe code with interior mutability and mutable references", function.fingerprint.name),
            vec!["review aliasing guarantees carefully when UnsafeCell or similar types are involved".to_string()],
        ));
    }

    findings
}