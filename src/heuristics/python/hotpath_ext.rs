use crate::analysis::{ParsedFile, ParsedFunction};

fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

// ── Rules using extended repeated_call_same_arg evidence ──────────────────────

pub(super) fn yaml_repeated_load_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    python
        .repeated_call_same_arg
        .iter()
        .filter(|(key, _)| key.starts_with("yaml.safe_load(") || key.starts_with("yaml.load("))
        .map(|(key, line)| Finding {
            rule_id: "yaml_load_same_payload_multiple_times".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} calls {key} multiple times; cache the result",
                function.fingerprint.name
            ),
            evidence: vec![format!("repeated_call={key}")],
        })
        .collect()
}

pub(super) fn xml_repeated_parse_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    python
        .repeated_call_same_arg
        .iter()
        .filter(|(key, _)| {
            key.starts_with("ET.fromstring(")
                || key.starts_with("ET.parse(")
                || key.starts_with("minidom.parseString(")
        })
        .map(|(key, line)| Finding {
            rule_id: "xml_parse_same_payload_multiple_times".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} calls {key} multiple times; cache the parsed result",
                function.fingerprint.name
            ),
            evidence: vec![format!("repeated_call={key}")],
        })
        .collect()
}

pub(super) fn datetime_strptime_repeated_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    python
        .repeated_call_same_arg
        .iter()
        .filter(|(key, _)| key.starts_with("datetime.strptime("))
        .map(|(key, line)| Finding {
            rule_id: "repeated_datetime_strptime_same_format".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} calls {key} multiple times with the same format; cache the compiled format",
                function.fingerprint.name
            ),
            evidence: vec![format!("repeated_call={key}")],
        })
        .collect()
}

pub(super) fn hashlib_repeated_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    python
        .repeated_call_same_arg
        .iter()
        .filter(|(key, _)| key.starts_with("hashlib."))
        .map(|(key, line)| Finding {
            rule_id: "repeated_hashlib_new_same_algorithm".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} recreates {key} per iteration; reuse or .copy() the digest",
                function.fingerprint.name
            ),
            evidence: vec![format!("repeated_call={key}")],
        })
        .collect()
}

// ── Rules using new loop-body evidence ────────────────────────────────────────

pub(super) fn copy_in_loop_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    python
        .copy_in_loop_lines
        .iter()
        .map(|line| {
            let body = &function.body_text;
            let rule_id = if body.contains("dict(") || body.contains("{**") {
                "dict_copy_in_loop_same_source"
            } else if body.contains("set(") {
                "set_created_per_iteration_same_elements"
            } else {
                "list_copy_in_loop_same_source"
            };
            Finding {
                rule_id: rule_id.to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: *line,
                end_line: *line,
                message: format!(
                    "function {} copies a collection inside a loop; consider hoisting above the loop",
                    function.fingerprint.name
                ),
                evidence: vec!["pattern=collection_copy_in_loop".to_string()],
            }
        })
        .collect()
}

pub(super) fn invariant_call_in_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    python
        .invariant_call_in_loop_lines
        .iter()
        .map(|(callee, line)| {
            let rule_id = if callee.contains("urlparse") || callee.contains("urlsplit") {
                "urlparse_in_loop_on_invariant_base"
            } else if callee.contains("resolve")
                || callee.contains("expanduser")
                || callee.contains("abspath")
                || callee.contains("realpath")
            {
                "path_resolve_or_expanduser_in_loop"
            } else {
                "repeated_locale_or_codec_lookup_in_loop"
            };
            Finding {
                rule_id: rule_id.to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: *line,
                end_line: *line,
                message: format!(
                    "function {} calls {callee} inside a loop on possibly invariant input; hoist above the loop",
                    function.fingerprint.name
                ),
                evidence: vec![format!("invariant_call_in_loop={callee}")],
            }
        })
        .collect()
}

pub(super) fn index_in_loop_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    python
        .index_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "repeated_list_index_lookup".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} calls .index() inside a loop; build a reverse-lookup dict instead",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=list_index_in_loop".to_string()],
        })
        .collect()
}

pub(super) fn append_sort_in_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    python
        .append_sort_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "append_then_sort_each_iteration".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} appends and sorts inside a loop; sort once after the loop or use bisect.insort",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=append_then_sort_in_loop".to_string()],
        })
        .collect()
}

pub(super) fn join_list_comp_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    python
        .join_list_comp_lines
        .iter()
        .map(|line| Finding {
            rule_id: "string_join_without_generator".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} passes a list comprehension to .join(); use a generator expression to avoid an intermediate list",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=join_list_comprehension".to_string()],
        })
        .collect()
}

pub(super) fn repeated_subscript_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    python
        .repeated_subscript_lines
        .iter()
        .map(|line| Finding {
            rule_id: "repeated_dict_get_same_key_no_cache".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} calls .get() with the same key multiple times; assign to a local variable",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=repeated_dict_get_same_key".to_string()],
        })
        .collect()
}

// ── Body-text based rules ────────────────────────────────────────────────────

pub(super) fn nested_list_search_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Detect nested for loops with `if ... in ...` or `if ... == ...` inner lookup
    let mut findings = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut outer_for_line = None;
    let mut inner_for_depth = 0;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            if outer_for_line.is_some() {
                inner_for_depth += 1;
                if inner_for_depth == 1 && i + 1 < lines.len() {
                    let next = lines[i + 1].trim();
                    if next.starts_with("if ") && (next.contains(" in ") || next.contains(" == ")) {
                        findings.push(Finding {
                            rule_id: "nested_list_search_map_candidate".to_string(),
                            severity: Severity::Info,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: function.fingerprint.start_line + i,
                            end_line: function.fingerprint.start_line + i,
                            message: format!(
                                "function {} uses nested loops for lookup; consider a dict/set for O(1) access",
                                function.fingerprint.name
                            ),
                            evidence: vec!["pattern=nested_loop_lookup_join".to_string()],
                        });
                    }
                }
            } else {
                outer_for_line = Some(i);
            }
        }
        if trimmed.is_empty()
            && !trimmed.starts_with("for ")
            && !trimmed.starts_with(' ')
            && !trimmed.starts_with('\t')
        {
            outer_for_line = None;
            inner_for_depth = 0;
        }
    }
    findings
}

pub(super) fn sort_then_first_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Detect .sort() followed by [0] or [-1] usage on the same list
        if trimmed.contains(".sort(") {
            // Check next few lines for [0] or [-1] access
            for next in lines.iter().skip(i + 1).take(3) {
                let next = next.trim();
                if next.contains("[0]") || next.contains("[-1]") {
                    findings.push(Finding {
                        rule_id: "sort_then_first_or_membership_only".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: function.fingerprint.start_line + i,
                        end_line: function.fingerprint.start_line + i,
                        message: format!(
                            "function {} sorts a list then only takes the first/last element; use min()/max()",
                            function.fingerprint.name
                        ),
                        evidence: vec!["pattern=sort_then_subscript_0".to_string()],
                    });
                    break;
                }
            }
        }
    }
    findings
}

pub(super) fn filter_count_iterate_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Detect repeated full traversals: multiple comprehensions/filter() calls
    // over the same iterable name
    let mut findings = Vec::new();
    let mut comprehension_targets: std::collections::BTreeMap<String, Vec<usize>> =
        std::collections::BTreeMap::new();

    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        // Match patterns like `[x for x in items if ...]` or `filter(lambda x: ..., items)`
        if let Some(in_idx) = trimmed.find(" for ")
            && let Some(source_start) = trimmed[in_idx + 5..].find(" in ")
        {
            let after_in = &trimmed[in_idx + 5 + source_start + 4..];
            let target = after_in
                .split([' ', ')', ']', ':'])
                .next()
                .unwrap_or("")
                .trim();
            if !target.is_empty() && target.len() < 40 {
                comprehension_targets
                    .entry(target.to_string())
                    .or_default()
                    .push(function.fingerprint.start_line + i);
            }
        }
    }

    for (target, lines) in &comprehension_targets {
        if lines.len() >= 3 {
            findings.push(Finding {
                rule_id: "filter_then_count_then_iterate".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: lines[0],
                end_line: lines[0],
                message: format!(
                    "function {} traverses '{}' {} times with comprehensions; combine into a single pass",
                    function.fingerprint.name, target, lines.len()
                ),
                evidence: vec![format!("repeated_traversals={}", lines.len())],
            });
        }
    }
    findings
}

// ── Plan 1 Wave 5 remaining rules ────────────────────────────────────────────

pub(super) fn repeated_format_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some()
            && !trimmed.is_empty()
            && (trimmed.contains(".format(") || trimmed.contains("f\"") || trimmed.contains("f'"))
        {
            findings.push(Finding {
                rule_id: "repeated_string_format_invariant_template".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line + i,
                end_line: function.fingerprint.start_line + i,
                message: format!(
                    "function {} formats a string inside a loop; consider building the template once",
                    function.fingerprint.name
                ),
                evidence: vec!["pattern=format_in_loop".to_string()],
            });
            break;
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
    }
    findings
}

pub(super) fn json_encoder_recreated_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some()
            && !trimmed.is_empty()
            && (trimmed.contains("JSONEncoder(") || trimmed.contains("JSONDecoder("))
        {
            findings.push(Finding {
                rule_id: "json_encoder_recreated_per_item".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line + i,
                end_line: function.fingerprint.start_line + i,
                message: format!(
                    "function {} creates JSONEncoder/Decoder inside a loop; reuse one instance",
                    function.fingerprint.name
                ),
                evidence: vec!["pattern=encoder_per_iteration".to_string()],
            });
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
    }
    findings
}

pub(super) fn gzip_open_per_chunk_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some()
            && !trimmed.is_empty()
            && (trimmed.contains("gzip.open(") || trimmed.contains("GzipFile("))
        {
            findings.push(Finding {
                rule_id: "gzip_open_per_chunk".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line + i,
                end_line: function.fingerprint.start_line + i,
                message: format!(
                    "function {} opens gzip inside a loop; use one streaming writer",
                    function.fingerprint.name
                ),
                evidence: vec!["pattern=gzip_per_chunk".to_string()],
            });
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
    }
    findings
}

pub(super) fn pickle_in_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some() && !trimmed.is_empty() && trimmed.contains("pickle.dumps(") {
            findings.push(Finding {
                rule_id: "pickle_dumps_in_loop_same_structure".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line + i,
                end_line: function.fingerprint.start_line + i,
                message: format!(
                    "function {} calls pickle.dumps() inside a loop; consider a single serialization pass",
                    function.fingerprint.name
                ),
                evidence: vec!["pattern=pickle_per_iteration".to_string()],
            });
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
    }
    findings
}

pub(super) fn isinstance_chain_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let isinstance_count = body.matches("isinstance(").count();
    if isinstance_count >= 4
        && let Some(first_line) = body.lines().enumerate().find_map(|(i, line)| {
            if line.contains("isinstance(") {
                Some(function.fingerprint.start_line + i)
            } else {
                None
            }
        })
    {
        return vec![Finding {
            rule_id: "repeated_isinstance_chain_same_object".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: first_line,
            end_line: first_line,
            message: format!(
                "function {} has {} isinstance() checks; use isinstance(obj, (T1, T2, ...)) or dispatch",
                function.fingerprint.name, isinstance_count
            ),
            evidence: vec![format!("isinstance_count={isinstance_count}")],
        }];
    }
    Vec::new()
}

pub(super) fn concat_in_comprehension_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if (trimmed.starts_with('[') || trimmed.starts_with('{'))
            && trimmed.contains(" for ")
            && trimmed.contains(" in ")
            && let Some(for_idx) = trimmed.find(" for ")
        {
            let expr = &trimmed[1..for_idx];
            if expr.contains(" + ") && (expr.contains('"') || expr.contains('\'')) {
                findings.push(Finding {
                        rule_id: "concatenation_in_comprehension_body".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: function.fingerprint.start_line + i,
                        end_line: function.fingerprint.start_line + i,
                        message: format!(
                            "function {} concatenates strings inside a comprehension; use f-strings or .join()",
                            function.fingerprint.name
                        ),
                        evidence: vec!["pattern=string_concat_in_comprehension".to_string()],
                    });
            }
        }
    }
    findings
}

/// tuple_unpacking_in_tight_loop – unpacking tuples in numeric-heavy loops
/// adds overhead vs indexed access.
pub(super) fn tuple_unpacking_in_tight_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for (") || trimmed.starts_with("for [") {
            let comma_count = trimmed.matches(',').count();
            if comma_count >= 3 && trimmed.ends_with(':') {
                let next_lines = &lines[i + 1..std::cmp::min(i + 10, lines.len())];
                let is_numeric = next_lines.iter().any(|l| {
                    let t = l.trim();
                    t.contains("np.")
                        || t.contains("math.")
                        || t.contains("sum(")
                        || t.contains("float(")
                        || t.contains("int(")
                });
                if is_numeric {
                    findings.push(Finding {
                        rule_id: "tuple_unpacking_in_tight_loop".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: function.fingerprint.start_line + i,
                        end_line: function.fingerprint.start_line + i,
                        message: format!(
                            "function {} unpacks many tuple elements in a tight numeric loop; consider indexed access or dataclass",
                            function.fingerprint.name
                        ),
                        evidence: vec!["pattern=tuple_unpack_numeric_loop".to_string()],
                    });
                }
            }
        }
    }
    findings
}
