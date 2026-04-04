use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::common::{is_title_doc, is_tutorial_doc};

pub(super) fn comment_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    let Some(doc_comment) = &function.doc_comment else {
        return findings;
    };

    let first_line = doc_comment
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");

    if is_title_doc(first_line) {
        findings.push(Finding {
            rule_id: "comment_style_title_case".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "function {} uses Title Case documentation that reads more like a heading",
                function.fingerprint.name
            ),
            evidence: vec![format!("doc comment line: {first_line}")],
        });
    }

    if is_tutorial_doc(doc_comment) {
        findings.push(Finding {
            rule_id: "comment_style_tutorial".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "function {} has a verbose tutorial-style doc comment",
                function.fingerprint.name
            ),
            evidence: vec![format!(
                "doc comment spans {} lines",
                doc_comment.lines().count()
            )],
        });
    }

    findings
}
