mod library;
mod performance;
mod security;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::Finding;

use super::framework_patterns::body_lines;

pub(crate) fn go_library_misuse_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = security::file_security_findings(file);

    for function in &file.functions {
        findings.extend(go_library_misuse_function_findings(file, function));
    }

    findings
}

fn go_library_misuse_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if file.is_test_file || function.is_test_function {
        return Vec::new();
    }

    let lines = body_lines(function);

    let mut findings = performance::performance_findings(file, function, &lines);
    findings.extend(security::security_findings(file, function, &lines));
    findings.extend(library::library_findings(file, function, &lines));
    findings
}
