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
    // Emit at most one finding per function to avoid noisy per-line duplication.
    if let Some(&first_line) = python.repeated_subscript_lines.first() {
        return vec![Finding {
            rule_id: "repeated_dict_get_same_key_no_cache".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: first_line,
            end_line: first_line,
            message: format!(
                "function {} calls .get() with the same key multiple times; assign to a local variable",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=repeated_dict_get_same_key".to_string()],
        }];
    }
    Vec::new()
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
    let mut loop_stack: Vec<usize> = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let indent = indent_level(line);
        while loop_stack.last().is_some_and(|depth| indent <= *depth) {
            loop_stack.pop();
        }

        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            if !loop_stack.is_empty()
                && nested_lookup_condition_after(&lines, index, indent)
            {
                findings.push(Finding {
                    rule_id: "nested_list_search_map_candidate".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: function.fingerprint.start_line + index,
                    end_line: function.fingerprint.start_line + index,
                    message: format!(
                        "function {} uses nested loops for lookup; consider a dict/set for O(1) access",
                        function.fingerprint.name
                    ),
                    evidence: vec!["pattern=nested_loop_lookup_join".to_string()],
                });
                break;
            }
            loop_stack.push(indent);
        }
    }

    findings
}

fn nested_lookup_condition_after(lines: &[&str], index: usize, loop_indent: usize) -> bool {
    for next_line in lines.iter().skip(index + 1) {
        let trimmed = next_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let indent = indent_level(next_line);
        if indent <= loop_indent {
            break;
        }

        if trimmed.starts_with("if ") {
            return nested_lookup_condition(trimmed);
        }

        if !trimmed.starts_with('#') {
            break;
        }
    }

    false
}

fn nested_lookup_condition(trimmed_if: &str) -> bool {
    let condition = trimmed_if
        .trim_start_matches("if ")
        .trim_end_matches(':')
        .trim();

    if condition.contains(" == ") {
        return true;
    }

    let Some((_, rhs)) = condition.split_once(" in ") else {
        return false;
    };

    !looks_string_like_membership_target(rhs)
}

fn looks_string_like_membership_target(expression: &str) -> bool {
    let lower = expression.trim().to_ascii_lowercase();
    lower.contains(".lower(")
        || lower.contains(".casefold(")
        || lower.contains(".strip(")
        || lower.contains("text")
        || lower.contains("string")
        || lower.contains("message")
        || lower.contains("content")
        || lower.ends_with("_str")
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
    let template_bindings = body
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let (name, value) = trimmed.split_once('=')?;
            let value = value.trim();
            ((value.starts_with('"') || value.starts_with('\''))
                && value.contains("{}"))
            .then(|| name.trim().to_string())
        })
        .collect::<std::collections::BTreeSet<_>>();
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some()
            && !trimmed.is_empty()
            && trimmed.contains(".format(")
            && template_bindings
                .iter()
                .any(|name| trimmed.contains(&format!("{name}.format(")))
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

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

pub(super) fn project_agnostic_hotpath_ext_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let body = &function.body_text;
    let lower_body = body.to_ascii_lowercase();
    let line = function.fingerprint.start_line;

    let push = |rule_id: &str, message: String| Finding {
        rule_id: rule_id.to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message,
        evidence: vec![format!("function={}", function.fingerprint.name)],
    };

    if contains_any(
        body,
        &["requests.get(", "httpx.get(", "open(", "subprocess.run("],
    ) && lower_body.matches("for ").count() >= 1
    {
        findings.push(push(
            "blocking_io_call_executed_per_item_without_batching",
            format!(
                "function {} performs blocking I/O per item without an obvious batching strategy",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("os.listdir(").count() + lower_body.matches("glob(").count() >= 2
        && lower_body.matches("for ").count() >= 2
    {
        findings.push(push(
            "repeated_directory_scan_inside_nested_loop",
            format!(
                "function {} rescans directories inside nested iteration",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("for ").count() >= 3
        && contains_any(&lower_body, &["len(", "sorted(", ".lower()", "path("])
        && function.fingerprint.line_count >= 15
    {
        findings.push(push(
            "invariant_computation_not_hoisted_out_of_nested_loop",
            format!(
                "function {} appears to recompute invariant work inside nested loops",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(body, &["any([", "all(["]) {
        findings.push(push(
            "any_or_all_wraps_list_comprehension_instead_of_generator",
            format!(
                "function {} materializes a list before any/all instead of using a generator",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(body, &["sum([", "max([", "min(["]) {
        findings.push(push(
            "sum_max_min_wrap_list_comprehension_instead_of_generator",
            format!(
                "function {} materializes a list before a reduction that accepts generators",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["copy.copy(", "copy.deepcopy(", ".copy("])
        && lower_body.matches("for ").count() >= 1
    {
        findings.push(push(
            "per_item_copy_of_large_config_or_context_object",
            format!(
                "function {} copies context-like objects per item on an iteration path",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("for ").count() >= 2
        && lower_body.matches("sum(").count()
            + lower_body.matches("max(").count()
            + lower_body.matches("min(").count()
            >= 2
    {
        findings.push(push(
            "same_sequence_scanned_multiple_times_for_related_aggregates",
            format!(
                "function {} scans the same sequence multiple times for related aggregates",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("list(").count() >= 2
        && contains_any(&lower_body, &["map(", "filter(", "sorted("])
    {
        findings.push(push(
            "generator_pipeline_materialized_between_each_transformation_stage",
            format!(
                "function {} repeatedly materializes intermediate pipeline stages",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("for ").count() >= 2
        && contains_any(&lower_body, &["index(", "find(", "lookup("])
    {
        findings.push(push(
            "linear_search_helper_called_from_nested_loops",
            format!(
                "function {} calls linear-search helpers from nested loops",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(body, &["os.path.exists(", ".exists()"])
        && contains_any(body, &["open(", "replace(", "unlink(", "write_text("])
    {
        findings.push(push(
            "repeated_path_exists_check_before_open_or_replace_in_loop",
            format!(
                "function {} checks path existence before repeated file operations",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(
        body,
        &["json.dumps(", "json.loads(", ".encode()", ".decode()"],
    ) && lower_body.matches("helper").count() >= 1
    {
        findings.push(push(
            "serialization_then_deserialization_between_adjacent_helpers",
            format!(
                "function {} bounces data through serialization between adjacent helpers",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("[i:i+").count() >= 1 || lower_body.matches("window = data[").count() >= 1
    {
        findings.push(push(
            "large_slice_copy_created_each_iteration_for_sliding_window",
            format!(
                "function {} creates slice copies on each sliding-window step",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("append(").count() >= 1
        && contains_any(
            &lower_body,
            &[" not in seen_list", " not in items", " not in values"],
        )
    {
        findings.push(push(
            "per_item_deduplication_uses_list_instead_of_hash_index",
            format!(
                "function {} performs per-item deduplication with linear membership checks",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("sorted(").count() >= 1
        && contains_any(&lower_body, &["key=lambda", "key = lambda", "expensive"])
    {
        findings.push(push(
            "expensive_sort_key_recomputed_without_preprojection",
            format!(
                "function {} computes expensive sort keys inline instead of preprojecting once",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches(".lower(").count() + lower_body.matches(".casefold(").count() >= 2 {
        findings.push(push(
            "repeated_casefold_or_lower_calls_before_multiple_comparisons",
            format!(
                "function {} repeatedly lowercases or casefolds before comparisons",
                function.fingerprint.name
            ),
        ));
    }

    if has_expensive_per_item_logging(body) {
        findings.push(push(
            "formatted_log_or_debug_payload_built_for_each_item_without_guard",
            format!(
                "function {} constructs per-item log payloads on a loop path without a visible guard",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("open(").count() >= 2
        && contains_any(&lower_body, &["read(", "read_text(", "read_bytes("])
    {
        findings.push(push(
            "repeated_open_read_close_of_same_small_file_in_single_workflow",
            format!(
                "function {} reopens and rereads files repeatedly within one workflow",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(body, &["sleep(0.01", "sleep(0.001", "sleep(0.1"])
        && contains_any(body, &["while ", "for "])
    {
        findings.push(push(
            "polling_loop_uses_tiny_sleep_instead_of_blocking_primitive",
            format!(
                "function {} uses a tiny-sleep polling loop instead of a blocking primitive",
                function.fingerprint.name
            ),
        ));
    }

    if has_invariant_template_reformatting(body)
        && function.fingerprint.line_count >= 12
    {
        findings.push(push(
            "invariant_template_or_prefix_string_reformatted_inside_loop",
            format!(
                "function {} repeatedly reformats template-like strings inside a loop",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(body, &["{\"", "{'"])
        && lower_body.matches("for ").count() >= 1
        && (lower_body.matches("{\"").count() + lower_body.matches("dict(").count() >= 3)
    {
        findings.push(push(
            "lookup_table_derived_from_constants_rebuilt_per_invocation",
            format!(
                "function {} rebuilds lookup tables from constants per invocation",
                function.fingerprint.name
            ),
        ));
    }

    findings
}

fn has_expensive_per_item_logging(body: &str) -> bool {
    let lines: Vec<&str> = body.lines().collect();
    let mut loop_stack: Vec<usize> = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let indent = indent_level(line);
        while loop_stack.last().is_some_and(|depth| indent <= *depth) {
            loop_stack.pop();
        }

        if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
            loop_stack.push(indent);
            continue;
        }

        if !loop_stack.is_empty() && is_expensive_log_line(trimmed) {
            return true;
        }
    }

    false
}

fn has_invariant_template_reformatting(body: &str) -> bool {
    let template_bindings = body
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let (name, value) = trimmed.split_once('=')?;
            let value = value.trim();
            ((value.starts_with('"') || value.starts_with('\''))
                && (value.contains("{}") || value.len() >= 12))
            .then(|| name.trim().to_string())
        })
        .collect::<std::collections::BTreeSet<_>>();

    let lines: Vec<&str> = body.lines().collect();
    let mut loop_stack: Vec<usize> = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let indent = indent_level(line);
        while loop_stack.last().is_some_and(|depth| indent <= *depth) {
            loop_stack.pop();
        }

        if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
            loop_stack.push(indent);
            continue;
        }

        if loop_stack.is_empty() {
            continue;
        }

        if template_bindings
            .iter()
            .any(|name| trimmed.contains(&format!("{name}.format(")))
        {
            return true;
        }

        if template_bindings.iter().any(|name| {
            trimmed.contains(&format!("{name} +")) || trimmed.contains(&format!("+ {name}"))
        }) {
            return true;
        }
    }

    false
}

fn is_expensive_log_line(line: &str) -> bool {
    if !(line.contains("logger.") || line.contains("logging.")) {
        return false;
    }

    contains_any(
        &line.to_ascii_lowercase(),
        &[
            "json.dumps(",
            ".format(",
            "join(",
            "pformat(",
            "traceback.",
            "serialize(",
            "to_dict(",
        ],
    )
}
