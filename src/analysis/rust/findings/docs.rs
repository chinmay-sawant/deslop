use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(super) fn doc_marker_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let Some(doc_comment) = &function.doc_comment else {
        return Vec::new();
    };
    let normalized = doc_comment.to_ascii_uppercase();
    let mut findings = Vec::new();

    if normalized.contains("TODO") {
        findings.push(Finding {
            rule_id: "todo_doc_comment_leftover".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "function {} has a TODO marker in its Rust doc comment",
                function.fingerprint.name
            ),
            evidence: vec![first_doc_comment_line(doc_comment)],
        });
    }

    if normalized.contains("FIXME") {
        findings.push(Finding {
            rule_id: "fixme_doc_comment_leftover".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "function {} has a FIXME marker in its Rust doc comment",
                function.fingerprint.name
            ),
            evidence: vec![first_doc_comment_line(doc_comment)],
        });
    }

    if normalized.contains("HACK") {
        findings.push(Finding {
            rule_id: "hack_doc_comment_leftover".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "function {} has a HACK marker in its Rust doc comment",
                function.fingerprint.name
            ),
            evidence: vec![first_doc_comment_line(doc_comment)],
        });
    }

    findings
}

fn first_doc_comment_line(doc_comment: &str) -> String {
    let line = doc_comment
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(doc_comment.trim());

    format!("doc comment line: {line}")
}
