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
    file: &ParsedFile,
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
                    path: file.path.clone(),
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
