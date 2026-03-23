use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(super) fn error_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();

    for line in &function.dropped_errors {
        findings.push(Finding {
            rule_id: "dropped_error".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} discards an error-like value with the blank identifier",
                function.fingerprint.name
            ),
            evidence: vec!["blank identifier assignment drops an err value".to_string()],
        });
    }

    for line in &function.panic_errors {
        findings.push(Finding {
            rule_id: "panic_on_error".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} escalates ordinary error handling through panic or fatal logging",
                function.fingerprint.name
            ),
            evidence: vec![
                "err != nil branch contains panic or log.Fatal style handling".to_string(),
            ],
        });
    }

    for call in &function.errorf_calls {
        if !call.mentions_err || call.uses_percent_w {
            continue;
        }

        let mut evidence = Vec::new();
        if let Some(format_string) = &call.format_string {
            evidence.push(format!("fmt.Errorf format string: {format_string}"));
        }
        evidence.push("call mentions err but does not use %w wrapping".to_string());

        findings.push(Finding {
            rule_id: "error_wrapping_misuse".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} uses fmt.Errorf without %w while referencing err",
                function.fingerprint.name
            ),
            evidence,
        });
    }

    findings
}
