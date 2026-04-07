use crate::analysis::ParsedFile;
use crate::model::{Finding, Severity};

use super::helpers::is_commented_code;

pub(crate) fn commented_out_code_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let suspicious_comments = file
        .comments
        .iter()
        .filter(|comment| is_commented_code(&comment.text))
        .collect::<Vec<_>>();
    if suspicious_comments.is_empty() {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "commented_out_code".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: suspicious_comments[0].line,
        end_line: suspicious_comments[0].line,
        message: "file contains comments that look like disabled code".to_string(),
        evidence: suspicious_comments
            .iter()
            .take(3)
            .map(|comment| format!("line {}: {}", comment.line, comment.text))
            .collect(),
    }]
}

pub(crate) fn sync_async_module_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file
        || !file
            .imports
            .iter()
            .any(|import| import.path.starts_with("asyncio"))
    {
        return Vec::new();
    }

    let async_public = file
        .functions
        .iter()
        .filter(|function| {
            function.fingerprint.kind.starts_with("async")
                && !function.is_test_function
                && !function.fingerprint.name.starts_with('_')
        })
        .count();
    let sync_public = file
        .functions
        .iter()
        .filter(|function| {
            !function.fingerprint.kind.starts_with("async")
                && !function.is_test_function
                && !function.fingerprint.name.starts_with('_')
        })
        .count();
    if async_public == 0 || sync_public == 0 {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "mixed_sync_async_module".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: file.functions[0].fingerprint.start_line,
        end_line: file.functions[0].fingerprint.start_line,
        message: "module mixes public sync and async entry points".to_string(),
        evidence: vec![
            format!("async_public_functions={async_public}"),
            format!("sync_public_functions={sync_public}"),
        ],
    }]
}
