use std::collections::BTreeSet;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::gin::{
    RepeatedArgumentGroupSpec, collect_labeled_first_argument_calls, layout_argument_looks_stable,
    repeated_argument_group_findings, simple_local_binding, url_parse_argument_looks_stable,
};
use super::{
    BodyLine, binding_matches, body_lines, first_line_with_any, has_prior_loop_line,
    import_aliases_for, is_request_path_function, join_lines, repeated_parse_findings,
};

pub(super) fn core_hot_path_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines = body_lines(function);

    findings.extend(repeated_parse_findings(file, function));
    findings.extend(core_repeated_work_findings(file, function, &lines));

    for alias in import_aliases_for(file, "regexp") {
        for body_line in &lines {
            if body_line.in_loop
                && (body_line.text.contains(&format!("{alias}.Compile("))
                    || body_line.text.contains(&format!("{alias}.MustCompile(")))
            {
                findings.push(Finding {
                    rule_id: "regexp_compile_in_hot_path".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} compiles regular expressions inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!(
                            "{}.Compile(...) or {}.MustCompile(...) observed inside a loop at line {}",
                            alias, alias, body_line.line
                        ),
                        "reusing a compiled regexp is usually cheaper than compiling per iteration"
                            .to_string(),
                    ],
                });
            }
        }
    }

    if is_request_path_function(file, function) {
        let mut template_aliases = import_aliases_for(file, "html/template");
        template_aliases.extend(import_aliases_for(file, "text/template"));
        for alias in template_aliases {
            for body_line in &lines {
                if body_line.text.contains(&format!("{alias}.ParseFiles("))
                    || body_line.text.contains(&format!("{alias}.ParseGlob("))
                    || body_line.text.contains(&format!("{alias}.ParseFS("))
                {
                    findings.push(Finding {
                        rule_id: "template_parse_in_hot_path".to_string(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} parses templates on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "{}.Parse* call observed at line {} in a handler-like function",
                                alias, body_line.line
                            ),
                            "template parsing is usually better cached during startup than repeated on request paths"
                                .to_string(),
                        ],
                    });
                }
            }
        }
    }

    for alias in import_aliases_for(file, "encoding/json") {
        for body_line in &lines {
            if body_line.in_loop && body_line.text.contains(&format!("{alias}.NewEncoder(")) {
                findings.push(Finding {
                    rule_id: "json_encoder_recreated_per_item".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} recreates a JSON encoder inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.NewEncoder(...) observed inside a loop at line {}", alias, body_line.line),
                        "reusing a stable encoder or stream writer usually avoids repeated setup work"
                            .to_string(),
                    ],
                });
            }

            if body_line.in_loop && body_line.text.contains(&format!("{alias}.NewDecoder(")) {
                findings.push(Finding {
                    rule_id: "json_decoder_recreated_per_item".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} recreates a JSON decoder inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.NewDecoder(...) observed inside a loop at line {}", alias, body_line.line),
                        "reusing a decoder or restructuring the loop often avoids repeated decode setup work"
                            .to_string(),
                    ],
                });
            }
        }
    }

    let mut gzip_markers = import_aliases_for(file, "compress/gzip")
        .into_iter()
        .flat_map(|alias| {
            vec![
                format!("{alias}.NewWriter("),
                format!("{alias}.NewWriterLevel("),
                format!("{alias}.NewReader("),
            ]
        })
        .collect::<Vec<_>>();
    gzip_markers.extend([
        "gzip.NewWriter(".to_string(),
        "gzip.NewWriterLevel(".to_string(),
        "gzip.NewReader(".to_string(),
    ]);

    for body_line in &lines {
        if body_line.in_loop
            && gzip_markers
                .iter()
                .any(|marker| body_line.text.contains(marker))
        {
            findings.push(Finding {
                rule_id: "gzip_reader_writer_recreated_per_item".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} recreates gzip readers or writers inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "gzip constructor observed inside a loop at line {}",
                        body_line.line
                    ),
                    "reusing compression state per stream is usually cheaper than rebuilding it per item"
                        .to_string(),
                ],
            });
        }
    }

    if !findings
        .iter()
        .any(|finding| finding.rule_id == "gzip_reader_writer_recreated_per_item")
    {
        let fallback_line = first_line_with_any(
            function,
            &["gzip.NewWriter(", "gzip.NewWriterLevel(", "gzip.NewReader("],
        );
        if let Some(fallback_line) = fallback_line
            && has_prior_loop_line(function, fallback_line)
        {
            findings.push(Finding {
                rule_id: "gzip_reader_writer_recreated_per_item".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: fallback_line,
                end_line: fallback_line,
                message: format!(
                    "function {} recreates gzip readers or writers inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("gzip constructor observed in loop-bearing function at line {fallback_line}"),
                    "reusing compression state per stream is usually cheaper than rebuilding it per item"
                        .to_string(),
                ],
            });
        }
    }

    let csv_aliases = import_aliases_for(file, "encoding/csv");
    if !csv_aliases.is_empty() {
        let writer_patterns = csv_aliases
            .iter()
            .map(|alias| format!("{alias}.NewWriter("))
            .collect::<Vec<_>>();
        let writer_pattern_refs = writer_patterns
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        for (name, line, _) in binding_matches(&lines, &writer_pattern_refs) {
            let flush_line = lines
                .iter()
                .find(|body_line| {
                    body_line.in_loop && body_line.text.contains(&format!("{name}.Flush()"))
                })
                .map(|body_line| body_line.line);

            if let Some(flush_line) = flush_line {
                findings.push(Finding {
                    rule_id: "csv_writer_flush_per_row".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: flush_line,
                    end_line: flush_line,
                    message: format!(
                        "function {} flushes a csv.Writer inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("csv writer {name} created at line {line}"),
                        format!(
                            "{}.Flush() observed inside a loop at line {}",
                            name, flush_line
                        ),
                        "flushing once per row usually reduces buffering effectiveness".to_string(),
                    ],
                });
            }
        }
    }

    findings
}

fn core_repeated_work_findings(
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
