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

fn slice_membership_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let markers = import_aliases_for(file, "slices")
        .into_iter()
        .flat_map(|alias| {
            [
                (format!("{alias}.Contains("), "Contains".to_string()),
                (format!("{alias}.Index("), "Index".to_string()),
            ]
        })
        .collect::<Vec<_>>();
    if markers.is_empty() {
        return Vec::new();
    }

    collect_labeled_first_argument_calls(lines, &markers, true)
        .into_iter()
        .filter(|(_, _, argument)| simple_local_binding(argument).is_some())
        .map(|(line, label, argument)| Finding {
            rule_id: "slice_membership_in_loop_map_candidate".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "function {} checks slice membership inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("slices.{label}(...) observed inside a loop at line {line}"),
                format!("searched slice binding: {argument}"),
                "a one-time set or map index is often cheaper than repeated linear membership checks"
                    .to_string(),
            ],
        })
        .collect()
}

fn url_parse_in_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let markers = import_aliases_for(file, "net/url")
        .into_iter()
        .flat_map(|alias| {
            [
                (format!("{alias}.Parse("), "Parse".to_string()),
                (
                    format!("{alias}.ParseRequestURI("),
                    "ParseRequestURI".to_string(),
                ),
            ]
        })
        .collect::<Vec<_>>();
    if markers.is_empty() {
        return Vec::new();
    }

    collect_labeled_first_argument_calls(lines, &markers, true)
        .into_iter()
        .filter(|(_, _, argument)| url_parse_argument_looks_stable(argument))
        .map(|(line, label, argument)| Finding {
            rule_id: "url_parse_in_loop_on_invariant_base".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "function {} parses the same base-like URL input inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("url.{label}(...) observed inside a loop at line {line}"),
                format!("first argument: {argument}"),
            ],
        })
        .collect()
}

fn time_parse_in_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let markers = import_aliases_for(file, "time")
        .into_iter()
        .flat_map(|alias| {
            [
                (format!("{alias}.Parse("), "Parse".to_string()),
                (
                    format!("{alias}.ParseInLocation("),
                    "ParseInLocation".to_string(),
                ),
            ]
        })
        .collect::<Vec<_>>();
    if markers.is_empty() {
        return Vec::new();
    }

    collect_labeled_first_argument_calls(lines, &markers, true)
        .into_iter()
        .filter(|(_, _, layout)| layout_argument_looks_stable(layout))
        .map(|(line, label, layout)| Finding {
            rule_id: "time_parse_layout_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "function {} parses time values with a stable layout inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("time.{label}(...) observed inside a loop at line {line}"),
                format!("layout argument: {layout}"),
            ],
        })
        .collect()
}

fn repeated_strings_split_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let markers = import_aliases_for(file, "strings")
        .into_iter()
        .flat_map(|alias| {
            [
                (format!("{alias}.Split("), "Split".to_string()),
                (format!("{alias}.SplitN("), "SplitN".to_string()),
                (format!("{alias}.SplitAfter("), "SplitAfter".to_string()),
                (format!("{alias}.SplitAfterN("), "SplitAfterN".to_string()),
                (format!("{alias}.Fields("), "Fields".to_string()),
                (format!("{alias}.FieldsFunc("), "FieldsFunc".to_string()),
            ]
        })
        .collect::<Vec<_>>();

    repeated_argument_group_findings(
        file,
        function,
        lines,
        &markers,
        false,
        RepeatedArgumentGroupSpec {
            rule_id: "strings_split_same_input_multiple_times",
            message_tail: "splits the same string input multiple times",
            helper_label: "string split helpers",
        },
    )
}

fn repeated_bytes_split_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let markers = import_aliases_for(file, "bytes")
        .into_iter()
        .flat_map(|alias| {
            [
                (format!("{alias}.Split("), "Split".to_string()),
                (format!("{alias}.SplitN("), "SplitN".to_string()),
                (format!("{alias}.SplitAfter("), "SplitAfter".to_string()),
                (format!("{alias}.SplitAfterN("), "SplitAfterN".to_string()),
                (format!("{alias}.Fields("), "Fields".to_string()),
                (format!("{alias}.FieldsFunc("), "FieldsFunc".to_string()),
            ]
        })
        .collect::<Vec<_>>();

    repeated_argument_group_findings(
        file,
        function,
        lines,
        &markers,
        false,
        RepeatedArgumentGroupSpec {
            rule_id: "bytes_split_same_input_multiple_times",
            message_tail: "splits the same byte slice input multiple times",
            helper_label: "byte split helpers",
        },
    )
}

fn repeated_strconv_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let markers = import_aliases_for(file, "strconv")
        .into_iter()
        .flat_map(|alias| {
            [
                (format!("{alias}.Atoi("), "Atoi".to_string()),
                (format!("{alias}.ParseBool("), "ParseBool".to_string()),
                (format!("{alias}.ParseFloat("), "ParseFloat".to_string()),
                (format!("{alias}.ParseInt("), "ParseInt".to_string()),
                (format!("{alias}.ParseUint("), "ParseUint".to_string()),
            ]
        })
        .collect::<Vec<_>>();

    repeated_argument_group_findings(
        file,
        function,
        lines,
        &markers,
        false,
        RepeatedArgumentGroupSpec {
            rule_id: "strconv_repeat_on_same_binding",
            message_tail: "converts the same string input with strconv multiple times",
            helper_label: "strconv helpers",
        },
    )
}

fn read_then_decode_duplicate_materialization_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut readall_markers = import_aliases_for(file, "io")
        .into_iter()
        .map(|alias| format!("{alias}.ReadAll("))
        .collect::<Vec<_>>();
    readall_markers.extend(
        import_aliases_for(file, "io/ioutil")
            .into_iter()
            .map(|alias| format!("{alias}.ReadAll(")),
    );
    if readall_markers.is_empty() {
        return Vec::new();
    }

    let marker_refs = readall_markers
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let mut findings = Vec::new();

    for (binding, readall_line, _) in binding_matches(lines, &marker_refs) {
        let decode_calls = go
            .parse_input_calls
            .iter()
            .filter(|call| {
                call.line > readall_line && call.input_binding.as_deref() == Some(binding.as_str())
            })
            .collect::<Vec<_>>();
        if decode_calls.is_empty() {
            continue;
        }

        let anchor_line = decode_calls[0].line;
        let parser_families = decode_calls
            .iter()
            .map(|call| call.parser_family.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ");
        let decode_lines = decode_calls
            .iter()
            .map(|call| call.line)
            .collect::<Vec<_>>();
        findings.push(Finding {
            rule_id: "read_then_decode_duplicate_materialization".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: anchor_line,
            end_line: anchor_line,
            message: format!(
                "function {} reads a payload fully and then decodes the same binding",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("io.ReadAll(...) assigned binding {binding} at line {readall_line}"),
                format!(
                    "decode calls observed at lines {}",
                    join_lines(&decode_lines)
                ),
                format!("decoder families: {parser_families}"),
            ],
        });
    }

    findings
}

fn slice_append_without_prealloc_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        if body_line.text.contains("append(")
            && !body_line.text.contains("make(")
            && !body_line.text.contains("cap(")
            && let Some(target) = append_target(&body_line.text)
        {
            let has_prealloc = lines.iter().any(|prior| {
                prior.line < body_line.line
                    && prior.text.contains("make([]")
                    && prior.text.contains(&target)
            });
            if !has_prealloc {
                let has_range_bound = lines.iter().any(|prior| {
                    prior.line < body_line.line
                        && prior.text.contains("range ")
                        && prior.text.contains("for ")
                });
                if has_range_bound {
                    findings.push(Finding {
                            rule_id: "slice_append_without_prealloc_known_bound".to_string(),
                            severity: Severity::Info,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: body_line.line,
                            end_line: body_line.line,
                            message: format!(
                                "function {} appends to a slice inside a range loop without visible preallocation",
                                function.fingerprint.name
                            ),
                            evidence: vec![
                                format!("append to {target} observed inside a loop at line {}", body_line.line),
                                "preallocating with make([]T, 0, len(source)) usually reduces growth-related copying".to_string(),
                            ],
                        });
                }
            }
        }
    }

    findings
}

fn nested_append_without_outer_capacity_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut loop_depth = 0usize;

    for body_line in lines {
        if body_line.text.contains("for ") && body_line.text.contains("{") {
            loop_depth += 1;
        }
        if body_line.text.contains("}") && loop_depth > 0 {
            loop_depth = loop_depth.saturating_sub(body_line.text.matches('}').count());
        }

        if loop_depth >= 2
            && body_line.text.contains("append(")
            && !function.body_text.contains("make([]")
        {
            findings.push(Finding {
                rule_id: "nested_append_without_outer_capacity".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} appends inside nested loops without visible preallocation",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("nested-loop append observed at line {}", body_line.line),
                    "preallocating the output slice before nested loops usually avoids repeated growth copies".to_string(),
                ],
            });
            break;
        }
    }

    findings
}

fn map_growth_without_size_hint_findings(
    _file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        if (body_line.text.contains("[") && body_line.text.contains("] ="))
            || (body_line.text.contains("[") && body_line.text.contains("] +="))
        {
            let has_map_decl_without_hint = lines.iter().any(|prior| {
                prior.line < body_line.line
                    && prior.text.contains("make(map[")
                    && !prior.text.contains(',')
            });
            let has_bare_map_literal = lines.iter().any(|prior| {
                prior.line < body_line.line
                    && prior.text.contains("map[")
                    && prior.text.contains("{}")
                    && !prior.text.contains("make(")
            });
            if has_map_decl_without_hint || has_bare_map_literal {
                findings.push(Finding {
                    rule_id: "map_growth_without_size_hint".to_string(),
                    severity: Severity::Info,
                    path: _file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} inserts into a map in a loop without a visible size hint",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("map insertion observed inside a loop at line {}", body_line.line),
                        "make(map[K]V, expectedSize) usually reduces rehashing during hot-path growth".to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn builder_without_grow_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for alias in import_aliases_for(file, "strings") {
        let builder_marker = format!("{alias}.Builder");
        for body_line in lines {
            if body_line.text.contains(&builder_marker) {
                let has_grow = lines.iter().any(|other| other.text.contains(".Grow("));
                if !has_grow
                    && lines
                        .iter()
                        .any(|other| other.in_loop && other.text.contains(".WriteString("))
                {
                    findings.push(Finding {
                        rule_id: "strings_builder_without_grow_known_bound".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} uses strings.Builder without Grow when approximate size is locally visible",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("strings.Builder declared at line {} without Grow", body_line.line),
                            "calling Grow(estimatedSize) before looped writes avoids repeated buffer expansion".to_string(),
                        ],
                    });
                    break;
                }
            }
        }
    }

    for alias in import_aliases_for(file, "bytes") {
        let buffer_marker = format!("{alias}.Buffer");
        for body_line in lines {
            if body_line.text.contains(&buffer_marker)
                && !body_line.text.contains(&format!("{alias}.NewBuffer("))
            {
                let has_grow = lines.iter().any(|other| other.text.contains(".Grow("));
                if !has_grow
                    && lines
                        .iter()
                        .any(|other| other.in_loop && other.text.contains(".Write"))
                {
                    findings.push(Finding {
                        rule_id: "bytes_buffer_without_grow_known_bound".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} uses bytes.Buffer without Grow when approximate size is locally visible",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("bytes.Buffer declared at line {} without Grow", body_line.line),
                            "calling Grow(estimatedSize) before looped writes avoids repeated buffer expansion".to_string(),
                        ],
                    });
                    break;
                }
            }
        }
    }

    findings
}

fn repeated_map_clone_findings(
    _file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        if body_line.text.contains("for ")
            && body_line.text.contains("range ")
            && body_line.text.contains("{")
        {
            continue;
        }

        let is_map_copy_loop = body_line.text.contains("make(map[")
            && lines.iter().any(|other| {
                other.in_loop && other.line > body_line.line && other.text.contains("] =")
            });

        if is_map_copy_loop {
            findings.push(Finding {
                rule_id: "repeated_map_clone_in_loop".to_string(),
                severity: Severity::Info,
                path: _file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} clones maps inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("map construction inside a loop at line {}", body_line.line),
                    "cloning a read-only map on every iteration is usually avoidable".to_string(),
                ],
            });
            break;
        }
    }

    findings
}

fn append_then_trim_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        if body_line.text.contains("append(")
            && let Some(target) = append_target(&body_line.text)
        {
            let has_trim = lines.iter().any(|other| {
                other.in_loop
                    && other.line > body_line.line
                    && other.text.contains(&format!("{target}["))
                    && other.text.contains(":]")
            });
            let has_reslice = lines.iter().any(|other| {
                other.in_loop
                    && other.line > body_line.line
                    && other.text.contains(&format!("{target} = {target}[:"))
            });
            if has_trim || has_reslice {
                findings.push(Finding {
                        rule_id: "append_then_trim_each_iteration".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} appends and then reslices in a loop",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("append to {target} followed by reslice inside a loop at line {}", body_line.line),
                            "a reusable scratch buffer with reset is usually cheaper than grow-then-trim per iteration".to_string(),
                        ],
                    });
                break;
            }
        }
    }

    findings
}

fn stable_value_normalization_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let normalization_markers: Vec<(String, &str)> = import_aliases_for(file, "strings")
        .into_iter()
        .flat_map(|alias| {
            vec![
                (format!("{alias}.ToLower("), "ToLower"),
                (format!("{alias}.ToUpper("), "ToUpper"),
                (format!("{alias}.TrimSpace("), "TrimSpace"),
                (format!("{alias}.ReplaceAll("), "ReplaceAll"),
            ]
        })
        .chain(
            import_aliases_for(file, "path")
                .into_iter()
                .flat_map(|alias| vec![(format!("{alias}.Clean("), "Clean")]),
        )
        .chain(
            import_aliases_for(file, "path/filepath")
                .into_iter()
                .flat_map(|alias| vec![(format!("{alias}.Clean("), "Clean")]),
        )
        .collect();

    if normalization_markers.is_empty() {
        return findings;
    }

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        for (marker, method) in &normalization_markers {
            if body_line.text.contains(marker)
                && let Some(argument) = first_argument_after_marker_simple(&body_line.text, marker)
                && simple_local_binding(&argument).is_some()
                && !lines
                    .iter()
                    .any(|other| other.in_loop && other.text.contains(&format!("{argument} =")))
            {
                findings.push(Finding {
                    rule_id: "stable_value_normalization_in_inner_loop".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} normalizes a stable value inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!(
                            "{method}({argument}) observed inside a loop at line {}",
                            body_line.line
                        ),
                        "normalizing an invariant value before the loop avoids repeated work"
                            .to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn bufio_missing_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_bufio =
        !import_aliases_for(file, "bufio").is_empty() || function.body_text.contains("bufio.");

    if has_bufio {
        return findings;
    }

    let write_in_loop = lines.iter().any(|bl| {
        bl.in_loop
            && (bl.text.contains(".Write(") || bl.text.contains(".WriteString("))
            && !bl.text.contains("Builder")
            && !bl.text.contains("Buffer")
    });

    let has_os_file_or_socket = import_aliases_for(file, "os").iter().any(|alias| {
        function.body_text.contains(&format!("{alias}.Create("))
            || function.body_text.contains(&format!("{alias}.File"))
            || function.signature_text.contains(&format!("{alias}.File"))
    }) || import_aliases_for(file, "os")
        .iter()
        .any(|alias| function.body_text.contains(&format!("{alias}.OpenFile(")))
        || function.body_text.contains("net.Conn")
        || function.body_text.contains("net.Dial");

    if write_in_loop && has_os_file_or_socket {
        let anchor_line = lines
            .iter()
            .find(|bl| {
                bl.in_loop && (bl.text.contains(".Write(") || bl.text.contains(".WriteString("))
            })
            .map(|bl| bl.line)
            .unwrap_or(function.body_start_line);

        findings.push(Finding {
            rule_id: "bufio_writer_missing_in_bulk_export".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: anchor_line,
            end_line: anchor_line,
            message: format!(
                "function {} writes to a file or socket in a loop without visible buffering",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("direct .Write(...) observed in a loop at line {anchor_line}"),
                "wrapping the writer in bufio.NewWriter usually improves throughput for looped writes".to_string(),
            ],
        });
    }

    let read_in_loop = lines.iter().any(|bl| {
        bl.in_loop
            && (bl.text.contains(".Read(") || bl.text.contains(".ReadByte("))
            && !bl.text.contains("ReadAll(")
            && !bl.text.contains("ReadFile(")
    });

    if read_in_loop && has_os_file_or_socket {
        let anchor_line = lines
            .iter()
            .find(|bl| bl.in_loop && (bl.text.contains(".Read(") || bl.text.contains(".ReadByte(")))
            .map(|bl| bl.line)
            .unwrap_or(function.body_start_line);

        findings.push(Finding {
            rule_id: "bufio_reader_missing_for_small_read_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: anchor_line,
            end_line: anchor_line,
            message: format!(
                "function {} reads from a file or socket in a loop without visible buffering",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("direct .Read(...) observed in a loop at line {anchor_line}"),
                "wrapping the reader in bufio.NewReader usually improves throughput for looped reads".to_string(),
            ],
        });
    }

    findings
}

fn nested_linear_join_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut outer_loop_depth = 0usize;
    let mut inner_loop_starts = Vec::new();

    for body_line in lines {
        if contains_for_keyword(&body_line.text) {
            outer_loop_depth += 1;
            if outer_loop_depth >= 2 {
                inner_loop_starts.push(body_line.line);
            }
        }
    }

    if inner_loop_starts.is_empty() {
        return findings;
    }

    let has_lookup_in_inner = lines.iter().any(|bl| {
        bl.in_loop
            && (bl.text.contains("== ") || bl.text.contains("!= "))
            && !bl.text.contains("err ")
            && !bl.text.contains("nil")
    });

    if has_lookup_in_inner && !inner_loop_starts.is_empty() {
        let slices_import = import_aliases_for(file, "slices");
        let has_linear_search = lines.iter().any(|bl| {
            bl.in_loop
                && slices_import.iter().any(|alias| {
                    bl.text.contains(&format!("{alias}.Contains("))
                        || bl.text.contains(&format!("{alias}.Index("))
                })
        });

        if has_linear_search {
            let anchor = inner_loop_starts[0];
            findings.push(Finding {
                rule_id: "nested_linear_join_map_candidate".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: anchor,
                end_line: anchor,
                message: format!(
                    "function {} performs nested-loop lookups that could use a map index",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("nested loop with linear search observed near line {anchor}"),
                    "indexing one collection into a map before the join loop usually avoids O(n*m) scans".to_string(),
                ],
            });
        }
    }

    findings
}

fn append_then_sort_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let sort_aliases = import_aliases_for(file, "sort");
    let slices_aliases = import_aliases_for(file, "slices");
    if sort_aliases.is_empty() && slices_aliases.is_empty() {
        return Vec::new();
    }

    let mut findings = Vec::new();

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        let has_sort = sort_aliases.iter().any(|alias| {
            body_line.text.contains(&format!("{alias}.Sort("))
                || body_line.text.contains(&format!("{alias}.Slice("))
                || body_line.text.contains(&format!("{alias}.Strings("))
                || body_line.text.contains(&format!("{alias}.Ints("))
        }) || slices_aliases.iter().any(|alias| {
            body_line.text.contains(&format!("{alias}.Sort("))
                || body_line.text.contains(&format!("{alias}.SortFunc("))
        });

        if has_sort {
            findings.push(Finding {
                rule_id: "append_then_sort_each_iteration".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} sorts inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("sort call observed inside a loop at line {}", body_line.line),
                    "sorting once after all insertions is usually cheaper than re-sorting each iteration".to_string(),
                ],
            });
            break;
        }
    }

    findings
}

fn sort_before_first_or_membership_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let sort_aliases = import_aliases_for(file, "sort");
    let slices_aliases = import_aliases_for(file, "slices");
    if sort_aliases.is_empty() && slices_aliases.is_empty() {
        return Vec::new();
    }

    let mut findings = Vec::new();

    let sort_line = lines.iter().find(|bl| {
        sort_aliases.iter().any(|alias| {
            bl.text.contains(&format!("{alias}.Sort("))
                || bl.text.contains(&format!("{alias}.Slice("))
                || bl.text.contains(&format!("{alias}.Strings("))
                || bl.text.contains(&format!("{alias}.Ints("))
                || bl.text.contains(&format!("{alias}.Float64s("))
        }) || slices_aliases.iter().any(|alias| {
            bl.text.contains(&format!("{alias}.Sort("))
                || bl.text.contains(&format!("{alias}.SortFunc("))
        })
    });

    if let Some(sort_bl) = sort_line {
        let only_uses_first = lines.iter().any(|bl| {
            bl.line > sort_bl.line && (bl.text.contains("[0]") || bl.text.contains("[:1]"))
        });
        let only_uses_min_max = lines.iter().any(|bl| {
            bl.line > sort_bl.line
                && (bl.text.contains("[0]") || bl.text.contains("[len("))
                && !bl.text.contains("range ")
        });

        if only_uses_first || only_uses_min_max {
            let no_range_after = !lines.iter().any(|bl| {
                bl.line > sort_bl.line && bl.text.contains("range ") && bl.text.contains("for ")
            });

            if no_range_after {
                findings.push(Finding {
                    rule_id: "sort_before_first_or_membership_only".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: sort_bl.line,
                    end_line: sort_bl.line,
                    message: format!(
                        "function {} sorts a collection but appears to only use the first element or min/max",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("sort observed at line {}", sort_bl.line),
                        "using slices.Min, slices.Max, or a single-pass scan is cheaper when only one element is needed".to_string(),
                    ],
                });
            }
        }
    }

    findings
}

fn filter_count_iterate_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut range_blocks = Vec::new();
    for body_line in lines {
        if body_line.text.contains("for ") && body_line.text.contains("range ") {
            range_blocks.push(body_line.line);
        }
    }

    if range_blocks.len() < 3 {
        return Vec::new();
    }

    let has_filter_count_iterate_pattern = range_blocks.windows(3).any(|window| {
        let gap1 = window[1] - window[0];
        let gap2 = window[2] - window[1];
        gap1 < 15 && gap2 < 15
    });

    if has_filter_count_iterate_pattern {
        let anchor = range_blocks[2];
        vec![Finding {
            rule_id: "filter_then_count_then_iterate".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: anchor,
            end_line: anchor,
            message: format!(
                "function {} traverses the same collection multiple times for filter, count, and process",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("multiple range loops over collections near lines {}", join_lines(&range_blocks)),
                "combining filter, count, and processing into a single pass is usually more efficient".to_string(),
            ],
        }]
    } else {
        Vec::new()
    }
}

fn append_target(text: &str) -> Option<String> {
    let (left, right) = text.split_once('=')?;
    if right.trim_start().starts_with("append(") || right.contains("= append(") {
        let binding = left.split_whitespace().last()?.trim();
        if super::super::is_identifier_name(binding) {
            return Some(binding.to_string());
        }
    }
    None
}

fn first_argument_after_marker_simple(text: &str, marker: &str) -> Option<String> {
    let suffix = text.split_once(marker)?.1;
    let mut depth = 0usize;
    for (index, character) in suffix.char_indices() {
        match character {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                if depth == 0 {
                    return Some(suffix[..index].trim().to_string());
                }
                depth = depth.saturating_sub(1);
            }
            ',' if depth == 0 => return Some(suffix[..index].trim().to_string()),
            _ => {}
        }
    }
    let trimmed = suffix.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn uuid_hash_formatting_only_for_logs_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let format_markers = [
        ("uuid.String(", "uuid.String()"),
        (".String()", ".String()"),
        ("hex.EncodeToString(", "hex.EncodeToString"),
        (
            "base64.StdEncoding.EncodeToString(",
            "base64.EncodeToString",
        ),
        (
            "base64.URLEncoding.EncodeToString(",
            "base64.EncodeToString",
        ),
        (
            "base64.RawStdEncoding.EncodeToString(",
            "base64.EncodeToString",
        ),
    ];

    let log_markers = [
        "log.Print",
        "log.Info",
        "log.Debug",
        "log.Warn",
        "log.Error",
        "log.Fatal",
        "log.Trace",
        "fmt.Print",
        "fmt.Sprint",
        "logger.Info",
        "logger.Debug",
        "logger.Warn",
        "logger.Error",
        "zap.String(",
        "zap.Any(",
        "logrus.",
        "slog.",
    ];

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        let has_format = format_markers
            .iter()
            .any(|(marker, _)| body_line.text.contains(marker));
        if !has_format {
            continue;
        }
        let only_feeds_logging = log_markers
            .iter()
            .any(|marker| body_line.text.contains(marker))
            || lines.iter().any(|other| {
                other.line == body_line.line + 1
                    && log_markers.iter().any(|marker| other.text.contains(marker))
            });
        if only_feeds_logging {
            let label = format_markers
                .iter()
                .find(|(marker, _)| body_line.text.contains(marker))
                .map(|(_, label)| *label)
                .unwrap_or("formatting call");
            findings.push(Finding {
                rule_id: "uuid_hash_formatting_only_for_logs".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} formats identifiers inside a loop only for logging",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("{label} observed inside a loop at line {}", body_line.line),
                    "formatting UUIDs, hashes, or base64 values only for debug strings adds avoidable allocation churn in tight loops".to_string(),
                ],
            });
            break;
        }
    }

    findings
}

fn contains_for_keyword(line: &str) -> bool {
    line.contains("for ") && (line.contains("range ") || line.contains("{") || line.contains("; "))
}
