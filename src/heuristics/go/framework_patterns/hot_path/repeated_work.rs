use std::collections::BTreeSet;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::super::gin::{
    RepeatedArgumentGroupSpec, collect_labeled_first_argument_calls, layout_argument_looks_stable,
    repeated_argument_group_findings, simple_local_binding, url_parse_argument_looks_stable,
};
use super::super::{BodyLine, binding_matches, import_aliases_for, join_lines};

pub(super) fn core_repeated_work_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(builder_buffer_recreated_findings(file, function, lines));
    findings.extend(scratch_container_churn_findings(file, function, lines));
    findings.extend(repeated_slice_clone_findings(file, function, lines));
    findings.extend(byte_string_conversion_findings(file, function, lines));
    findings.extend(slice_membership_findings(file, function, lines));
    findings.extend(url_parse_in_loop_findings(file, function, lines));
    findings.extend(time_parse_in_loop_findings(file, function, lines));
    findings.extend(repeated_strings_split_findings(file, function, lines));
    findings.extend(repeated_bytes_split_findings(file, function, lines));
    findings.extend(repeated_strconv_findings(file, function, lines));
    findings.extend(read_then_decode_duplicate_materialization_findings(
        file, function, lines,
    ));
    findings.extend(slice_append_without_prealloc_findings(
        file, function, lines,
    ));
    findings.extend(nested_append_without_outer_capacity_findings(
        file, function, lines,
    ));
    findings.extend(map_growth_without_size_hint_findings(file, function, lines));
    findings.extend(builder_without_grow_findings(file, function, lines));
    findings.extend(repeated_map_clone_findings(file, function, lines));
    findings.extend(append_then_trim_findings(file, function, lines));
    findings.extend(stable_value_normalization_findings(file, function, lines));
    findings.extend(bufio_missing_findings(file, function, lines));
    findings.extend(nested_linear_join_findings(file, function, lines));
    findings.extend(append_then_sort_findings(file, function, lines));
    findings.extend(sort_before_first_or_membership_findings(
        file, function, lines,
    ));
    findings.extend(filter_count_iterate_findings(file, function, lines));
    findings.extend(uuid_hash_formatting_only_for_logs_findings(
        file, function, lines,
    ));
    findings
}

fn builder_buffer_recreated_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines.iter().filter(|body_line| body_line.in_loop) {
        for alias in import_aliases_for(file, "strings") {
            if body_line.text.contains(&format!("{alias}.Builder")) {
                findings.push(Finding {
                    rule_id: "builder_or_buffer_recreated_per_iteration".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} recreates a strings.Builder inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{} observed inside a loop at line {}", body_line.text, body_line.line),
                        "reusing one builder or resetting it between iterations is usually cheaper than constructing a new builder per item"
                            .to_string(),
                    ],
                });
                break;
            }
        }

        for alias in import_aliases_for(file, "bytes") {
            if body_line.text.contains(&format!("{alias}.Buffer"))
                || body_line.text.contains(&format!("{alias}.NewBuffer("))
                || body_line
                    .text
                    .contains(&format!("{alias}.NewBufferString("))
            {
                findings.push(Finding {
                    rule_id: "builder_or_buffer_recreated_per_iteration".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} recreates a bytes.Buffer inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{} observed inside a loop at line {}", body_line.text, body_line.line),
                        "reusing one buffer or resetting it between iterations is usually cheaper than constructing a new buffer per item"
                            .to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn scratch_container_churn_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines.iter().filter(|body_line| body_line.in_loop) {
        if body_line.text.contains("make([]") {
            findings.push(Finding {
                rule_id: "make_slice_inside_hot_loop_same_shape".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} recreates scratch slices inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("{} observed inside a loop at line {}", body_line.text, body_line.line),
                    "a reusable scratch slice is often cheaper than recreating the same shape every iteration"
                        .to_string(),
                ],
            });
        }

        if body_line.text.contains("make(map[") {
            findings.push(Finding {
                rule_id: "make_map_inside_hot_loop_same_shape".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} recreates scratch maps inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("{} observed inside a loop at line {}", body_line.text, body_line.line),
                    "a reusable map or prebuilt index is often cheaper than recreating the same map shape every iteration"
                        .to_string(),
                ],
            });
        }
    }

    findings
}

fn repeated_slice_clone_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    lines
        .iter()
        .filter(|body_line| {
            body_line.in_loop
                && (body_line.text.contains("slices.Clone(")
                    || (body_line.text.contains("append([]") && body_line.text.contains("...)")))
        })
        .map(|body_line| Finding {
            rule_id: "repeated_slice_clone_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: body_line.line,
            end_line: body_line.line,
            message: format!(
                "function {} clones slices inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "{} observed inside a loop at line {}",
                    body_line.text, body_line.line
                ),
                "reusing or reshaping one slice is often cheaper than cloning on every iteration"
                    .to_string(),
            ],
        })
        .collect()
}

fn byte_string_conversion_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    lines
        .iter()
        .filter(|body_line| {
            body_line.in_loop
                && (body_line.text.contains("string([]byte(")
                    || body_line.text.contains("[]byte(string(")
                    || (body_line.text.contains("[string(") && body_line.text.contains("[]byte("))
                    || (body_line.text.contains("append(")
                        && (body_line.text.contains("string(") || body_line.text.contains("[]byte("))))
        })
        .map(|body_line| Finding {
            rule_id: "byte_string_conversion_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: body_line.line,
            end_line: body_line.line,
            message: format!(
                "function {} converts between bytes and strings inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("{} observed inside a loop at line {}", body_line.text, body_line.line),
                "repeated byte-string conversion can add avoidable allocation churn in iterative paths"
                    .to_string(),
            ],
        })
        .collect()
}

include!("repeated_work/repeated_work_primary.rs");
include!("repeated_work/repeated_work_secondary.rs");
