use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};
use std::path::Path;

pub(crate) const BINDING_LOCATION: &str = file!();

use super::common::{identifier_token_count, is_generic_name};
use super::python::should_skip_python_weak_typing;

pub(super) fn overlong_finding(file: &ParsedFile, function: &ParsedFunction) -> Option<Finding> {
    if function.is_test_function {
        return None;
    }

    let token_count = identifier_token_count(&function.fingerprint.name);
    if function.fingerprint.name.len() < 28 || token_count < 5 {
        return None;
    }

    Some(Finding {
        rule_id: "overlong_name".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} uses an overly descriptive name",
            function.fingerprint.name
        ),
        evidence: vec![
            format!(
                "identifier length: {} characters",
                function.fingerprint.name.len()
            ),
            format!("identifier token count: {token_count}"),
        ],
    })
}

pub(super) fn generic_finding(file: &ParsedFile, function: &ParsedFunction) -> Option<Finding> {
    if function.is_test_function {
        return None;
    }

    if !is_generic_name(&function.fingerprint.name) {
        return None;
    }

    // Skip when the function name is contextualised by its parent module.
    // e.g. `parse_file` inside a `parser/` module is descriptive, not generic.
    if function_name_matches_module_context(&file.path, &function.fingerprint.name) {
        return None;
    }

    let mut evidence = Vec::new();
    let has_weak_typing =
        function.fingerprint.contains_any_type || function.fingerprint.contains_empty_interface;
    let has_high_symmetry = function.fingerprint.symmetry_score >= 0.5;
    let has_low_comment_specificity =
        function.fingerprint.comment_to_code_ratio <= 0.15 && function.fingerprint.line_count >= 5;

    if has_weak_typing {
        evidence.push("uses vague signature types".to_string());
    }
    if has_high_symmetry {
        evidence.push(format!(
            "high structural symmetry ({:.2})",
            function.fingerprint.symmetry_score
        ));
    }
    if has_low_comment_specificity {
        evidence.push(format!(
            "low comment specificity ({:.2})",
            function.fingerprint.comment_to_code_ratio
        ));
    }
    if function.fingerprint.type_assertion_count == 0 && has_weak_typing {
        evidence.push("no narrowing type assertions found".to_string());
    }

    if !has_weak_typing && !has_high_symmetry {
        return None;
    }

    if evidence.is_empty() {
        return None;
    }

    Some(Finding {
        rule_id: "generic_name".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} uses a generic name without strong domain-specific signals",
            function.fingerprint.name
        ),
        evidence,
    })
}

pub(super) fn weak_finding(file: &ParsedFile, function: &ParsedFunction) -> Option<Finding> {
    if !function.fingerprint.contains_any_type && !function.fingerprint.contains_empty_interface {
        return None;
    }
    if should_skip_python_weak_typing(file, function) {
        return None;
    }

    let severity = if function.fingerprint.type_assertion_count == 0 {
        Severity::Warning
    } else {
        Severity::Info
    };

    let mut evidence = Vec::new();
    if function.fingerprint.contains_any_type {
        evidence.push("signature contains any".to_string());
    }
    if function.fingerprint.contains_empty_interface {
        evidence.push("signature contains interface{}".to_string());
    }
    evidence.push(format!(
        "type assertions observed: {}",
        function.fingerprint.type_assertion_count
    ));

    Some(Finding {
        rule_id: "weak_typing".to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} relies on weakly typed inputs or outputs",
            function.fingerprint.name
        ),
        evidence,
    })
}

/// Returns `true` when any token in `function_name` also appears (possibly as
/// a stem) in the parent directory or file stem of `path`.  For example,
/// `parse_file` in `src/parser/mod.rs` shares the "parse" stem with "parser".
fn function_name_matches_module_context(path: &Path, function_name: &str) -> bool {
    let fn_tokens: Vec<String> = super::common::identifier_tokens(function_name)
        .into_iter()
        .map(|t| t.to_ascii_lowercase())
        .collect();
    if fn_tokens.is_empty() {
        return false;
    }

    // Collect context tokens from the parent directory name and the file stem.
    let mut context_tokens: Vec<String> = Vec::new();
    if let Some(parent) = path.parent().and_then(|p| p.file_name()) {
        context_tokens.extend(
            super::common::identifier_tokens(&parent.to_string_lossy())
                .into_iter()
                .map(|t| t.to_ascii_lowercase()),
        );
    }
    if let Some(stem) = path.file_stem() {
        let stem_str = stem.to_string_lossy();
        if stem_str != "mod" && stem_str != "lib" && stem_str != "main" {
            context_tokens.extend(
                super::common::identifier_tokens(&stem_str)
                    .into_iter()
                    .map(|t| t.to_ascii_lowercase()),
            );
        }
    }

    if context_tokens.is_empty() {
        return false;
    }

    // If any function-name token shares a common stem with a context token,
    // the name is contextualised by its module.
    fn stem_matches(a: &str, b: &str) -> bool {
        let short = a.len().min(b.len());
        if short < 3 {
            return a == b;
        }
        // Allow stem-level matching: "parse" matches "parser", "scan" matches "scanner"
        a.starts_with(b) || b.starts_with(a)
    }

    fn_tokens
        .iter()
        .any(|ft| context_tokens.iter().any(|ct| stem_matches(ft, ct)))
}
