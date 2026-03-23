use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::common::{identifier_token_count, is_generic_name, normalize_name};

pub(super) fn overlong_finding(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Option<Finding> {
    let token_count = identifier_token_count(&function.fingerprint.name);
    if function.fingerprint.name.len() < 24 || token_count < 4 {
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

pub(super) fn generic_finding(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Option<Finding> {
    let normalized = normalize_name(&function.fingerprint.name);
    if !is_generic_name(&normalized) {
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
