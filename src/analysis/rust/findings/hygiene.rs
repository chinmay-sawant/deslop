use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(super) fn unsafe_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let rust = function.rust_evidence();

    rust.unsafe_lines
        .iter()
        .filter(|unsafe_line| !has_safety_comment(**unsafe_line, rust.safety_comment_lines))
        .map(|unsafe_line| Finding {
            rule_id: "unsafe_without_safety_comment".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *unsafe_line,
            end_line: *unsafe_line,
            message: format!(
                "function {} uses unsafe without a nearby SAFETY comment",
                function.fingerprint.name
            ),
            evidence: vec![format!("unsafe usage line: {unsafe_line}")],
        })
        .collect()
}

pub(super) fn non_test_macro_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    macro_name: &str,
    rule_id: &str,
    message_suffix: &str,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter(|call| call.name == macro_name)
        .map(|call| Finding {
            rule_id: rule_id.to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!("function {} {message_suffix}", function.fingerprint.name),
            evidence: vec![format!("macro invocation: {macro_name}")],
        })
        .collect()
}

pub(super) fn non_test_call_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    call_name: &str,
    rule_id: &str,
    message_suffix: &str,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter(|call| call.name == call_name)
        .map(|call| Finding {
            rule_id: rule_id.to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!("function {} {message_suffix}", function.fingerprint.name),
            evidence: vec![match &call.receiver {
                Some(receiver) => format!("method call: {receiver}.{call_name}()"),
                None => format!("call: {call_name}()"),
            }],
        })
        .collect()
}

fn has_safety_comment(unsafe_line: usize, safety_comment_lines: &[usize]) -> bool {
    let min_line = unsafe_line.saturating_sub(2);
    safety_comment_lines
        .iter()
        .any(|comment_line| *comment_line >= min_line && *comment_line <= unsafe_line)
}
