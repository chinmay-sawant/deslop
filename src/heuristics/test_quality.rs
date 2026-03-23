use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(super) fn test_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !file.is_test_file {
        return Vec::new();
    }

    let Some(summary) = &function.test_summary else {
        return Vec::new();
    };

    let mut findings = Vec::new();

    if summary.assertion_like_calls == 0 && summary.skip_calls == 0 && summary.production_calls > 0
    {
        findings.push(Finding {
            rule_id: "test_without_assertion_signal".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.end_line,
            message: format!(
                "test {} exercises code without an obvious assertion signal",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("production calls observed: {}", summary.production_calls),
                "no t.Fatal/assert/require-style assertion calls were observed".to_string(),
            ],
        });
    }

    if summary.assertion_like_calls > 0
        && summary.error_assertion_calls == 0
        && summary.production_calls > 0
        && summary.skip_calls == 0
    {
        findings.push(Finding {
            rule_id: "happy_path_only_test".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.end_line,
            message: format!(
                "test {} appears to cover only success expectations",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "assertion-like calls observed: {}",
                    summary.assertion_like_calls
                ),
                "no negative-path or error assertion signals were observed".to_string(),
            ],
        });
    }

    if summary.has_todo_marker
        || summary.skip_calls > 0 && summary.production_calls == 0
        || summary.assertion_like_calls == 0
            && summary.production_calls == 0
            && function.fingerprint.line_count <= 6
    {
        findings.push(Finding {
            rule_id: "placeholder_test_body".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.end_line,
            message: format!(
                "test {} looks like a placeholder rather than a validating test",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("skip calls observed: {}", summary.skip_calls),
                format!("production calls observed: {}", summary.production_calls),
            ],
        });
    }

    findings
}
