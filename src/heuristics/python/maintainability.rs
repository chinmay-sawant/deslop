use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(super) fn eval_exec_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter(|call| call.receiver.is_none() && matches!(call.name.as_str(), "eval" | "exec"))
        .map(|call| Finding {
            rule_id: "eval_exec_usage".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} uses {} in non-test Python code",
                function.fingerprint.name, call.name
            ),
            evidence: vec![format!("builtin call: {}()", call.name)],
        })
        .collect()
}

pub(super) fn print_debugging_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || should_skip_print_rule(file, function) {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter(|call| call.receiver.is_none() && call.name == "print")
        .map(|call| Finding {
            rule_id: "print_debugging_leftover".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} leaves print-style debugging in Python code",
                function.fingerprint.name
            ),
            evidence: vec!["builtin call: print()".to_string()],
        })
        .collect()
}

fn should_skip_print_rule(file: &ParsedFile, function: &ParsedFunction) -> bool {
    function.fingerprint.name == "main"
        || file
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "__main__.py")
}
