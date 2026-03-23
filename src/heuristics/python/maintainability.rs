use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(super) fn exception_swallowed_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .exception_handlers
        .iter()
        .filter(|handler| handler.is_broad && handler.suppresses)
        .map(|handler| Finding {
            rule_id: "exception_swallowed".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: handler.line,
            end_line: handler.line,
            message: format!(
                "function {} swallows a broad exception handler",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("handler clause: {}", handler.clause),
                format!(
                    "handler action: {}",
                    handler.action.as_deref().unwrap_or("<unknown>")
                ),
            ],
        })
        .collect()
}

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

pub(super) fn none_comparison_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .none_comparison_lines
        .iter()
        .map(|line| Finding {
            rule_id: "none_comparison".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} compares against None with == or != instead of identity checks",
                function.fingerprint.name
            ),
            evidence: vec!["prefer is None or is not None for None checks".to_string()],
        })
        .collect()
}

pub(super) fn side_effect_comprehension_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .side_effect_comprehension_lines
        .iter()
        .map(|line| Finding {
            rule_id: "side_effect_comprehension".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} uses a comprehension only for side effects",
                function.fingerprint.name
            ),
            evidence: vec!["prefer an explicit loop when the result is discarded".to_string()],
        })
        .collect()
}

pub(super) fn redundant_return_none_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .redundant_return_none_lines
        .iter()
        .map(|line| Finding {
            rule_id: "redundant_return_none".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} returns None explicitly where falling through would be clearer",
                function.fingerprint.name
            ),
            evidence: vec!["explicit return None can add noise in simple Python control flow"
                .to_string()],
        })
        .collect()
}

pub(super) fn hardcoded_path_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .local_strings
        .iter()
        .filter(|literal| looks_like_hardcoded_path(&literal.value))
        .map(|literal| Finding {
            rule_id: "hardcoded_path_string".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: literal.line,
            end_line: literal.line,
            message: format!(
                "function {} hardcodes a filesystem path string",
                function.fingerprint.name
            ),
            evidence: vec![format!("binding {} = {}", literal.name, literal.value)],
        })
        .collect()
}

pub(super) fn variadic_public_api_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function
        || function.fingerprint.receiver_type.is_some()
        || function.fingerprint.name.starts_with('_')
        || (!function.has_varargs && !function.has_kwargs)
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "variadic_public_api".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.start_line,
        message: format!(
            "public function {} relies on *args or **kwargs instead of a clearer interface",
            function.fingerprint.name
        ),
        evidence: vec![format!(
            "has_varargs={} has_kwargs={}",
            function.has_varargs, function.has_kwargs
        )],
    }]
}

fn should_skip_print_rule(file: &ParsedFile, function: &ParsedFunction) -> bool {
    function.fingerprint.name == "main"
        || file
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "__main__.py")
}

fn looks_like_hardcoded_path(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || value
            .chars()
            .nth(1)
            .is_some_and(|character| character == ':')
            && value.contains('\\')
        || (value.contains('/') && has_path_like_suffix(value))
}

fn has_path_like_suffix(value: &str) -> bool {
    [
        ".json", ".yaml", ".yml", ".txt", ".csv", ".db", ".sqlite", ".ini", ".cfg",
        ".conf", ".pem", ".log",
    ]
    .iter()
    .any(|suffix| value.ends_with(suffix))
}
