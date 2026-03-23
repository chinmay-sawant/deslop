use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

const GOD_FUNCTION_LINE_THRESHOLD: usize = 45;
const GOD_FUNCTION_COMPLEXITY_THRESHOLD: usize = 6;
const GOD_FUNCTION_CALL_THRESHOLD: usize = 8;
const MONOLITHIC_INIT_BYTE_THRESHOLD: usize = 1200;
const MONOLITHIC_INIT_IMPORT_THRESHOLD: usize = 6;
const MONOLITHIC_INIT_FUNCTION_THRESHOLD: usize = 4;
const INSTANCE_ATTRIBUTE_THRESHOLD: usize = 10;
const INSTANCE_ATTRIBUTE_METHOD_THRESHOLD: usize = 3;

pub(super) fn god_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let fingerprint = &function.fingerprint;
    if fingerprint.line_count < GOD_FUNCTION_LINE_THRESHOLD
        || fingerprint.complexity_score < GOD_FUNCTION_COMPLEXITY_THRESHOLD
        || fingerprint.call_count < GOD_FUNCTION_CALL_THRESHOLD
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "god_function".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(fingerprint.name.clone()),
        start_line: fingerprint.start_line,
        end_line: fingerprint.end_line,
        message: format!(
            "function {} concentrates too much control flow and behavior",
            fingerprint.name
        ),
        evidence: vec![
            format!("line_count={}", fingerprint.line_count),
            format!("complexity_score={}", fingerprint.complexity_score),
            format!("call_count={}", fingerprint.call_count),
        ],
    }]
}

pub(super) fn monolithic_init_module_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file
        || file
            .path
            .file_name()
            .and_then(|name| name.to_str())
            != Some("__init__.py")
    {
        return Vec::new();
    }

    if file.functions.len() < MONOLITHIC_INIT_FUNCTION_THRESHOLD
        || (file.imports.len() < MONOLITHIC_INIT_IMPORT_THRESHOLD
            && file.byte_size < MONOLITHIC_INIT_BYTE_THRESHOLD)
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "monolithic_init_module".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: 1,
        end_line: 1,
        message: "__init__.py is carrying enough imports and behavior to look monolithic"
            .to_string(),
        evidence: vec![
            format!("functions={}", file.functions.len()),
            format!("imports={}", file.imports.len()),
            format!("byte_size={}", file.byte_size),
        ],
    }]
}

pub(super) fn too_many_instance_attributes_findings(file: &ParsedFile) -> Vec<Finding> {
    file.class_summaries
        .iter()
        .filter(|summary| {
            summary.instance_attribute_count >= INSTANCE_ATTRIBUTE_THRESHOLD
                && summary.method_count >= INSTANCE_ATTRIBUTE_METHOD_THRESHOLD
        })
        .map(|summary| Finding {
            rule_id: "too_many_instance_attributes".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: summary.line,
            end_line: summary.line,
            message: format!(
                "class {} assigns an unusually large number of instance attributes",
                summary.name
            ),
            evidence: vec![
                format!("instance_attribute_count={}", summary.instance_attribute_count),
                format!("method_count={}", summary.method_count),
            ],
        })
        .collect()
}