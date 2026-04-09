fn repeated_map_clone_findings(
    file: &ParsedFile,
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
                path: file.path.clone(),
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
