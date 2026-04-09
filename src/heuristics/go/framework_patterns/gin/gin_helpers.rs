fn request_path_read_file_lines(file: &ParsedFile, lines: &[BodyLine]) -> Vec<usize> {
    let mut read_file_aliases = import_aliases_for(file, "os")
        .into_iter()
        .map(|alias| format!("{alias}.ReadFile("))
        .collect::<Vec<_>>();
    read_file_aliases.extend(
        import_aliases_for(file, "io/ioutil")
            .into_iter()
            .map(|alias| format!("{alias}.ReadFile(")),
    );

    lines
        .iter()
        .filter(|body_line| {
            read_file_aliases
                .iter()
                .any(|marker| body_line.text.contains(marker))
        })
        .map(|body_line| body_line.line)
        .collect()
}

pub(crate) fn repeated_argument_group_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    markers: &[(String, String)],
    require_loop: bool,
    spec: RepeatedArgumentGroupSpec<'_>,
) -> Vec<Finding> {
    if markers.is_empty() {
        return Vec::new();
    }

    let mut groups = BTreeMap::<String, Vec<(usize, String)>>::new();
    for (line, label, argument) in
        collect_labeled_first_argument_calls(lines, markers, require_loop)
    {
        groups.entry(argument).or_default().push((line, label));
    }

    let mut findings = Vec::new();
    for (argument, calls) in groups {
        if calls.len() < 2 {
            continue;
        }

        let lines = calls.iter().map(|(line, _)| *line).collect::<Vec<_>>();
        let operations = calls
            .iter()
            .map(|(line, label)| format!("{label} at line {line}"))
            .collect::<Vec<_>>()
            .join(", ");
        let anchor_line = lines[1];
        findings.push(Finding {
            rule_id: spec.rule_id.to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: anchor_line,
            end_line: anchor_line,
            message: format!(
                "function {} {}",
                function.fingerprint.name, spec.message_tail
            ),
            evidence: vec![
                format!(
                    "argument reused by {} at lines {}",
                    spec.helper_label,
                    join_lines(&lines)
                ),
                format!("first argument: {argument}"),
                format!("operations: {operations}"),
            ],
        });
    }

    findings
}

pub(crate) fn collect_labeled_first_argument_calls(
    lines: &[BodyLine],
    markers: &[(String, String)],
    require_loop: bool,
) -> Vec<(usize, String, String)> {
    let mut calls = Vec::new();

    for body_line in lines {
        if require_loop && !body_line.in_loop {
            continue;
        }

        for (marker, label) in markers {
            if !body_line.text.contains(marker) {
                continue;
            }

            if let Some(argument) = first_argument_after_marker(&body_line.text, marker) {
                calls.push((body_line.line, label.clone(), argument));
                break;
            }
        }
    }

    calls
}

fn form_file_readall_flow(
    file: &ParsedFile,
    lines: &[BodyLine],
    call: &GinCallSummary,
) -> Option<(usize, String, Option<usize>)> {
    let form_file_binding = call.assigned_binding.as_deref()?;
    let mut candidate_bindings = BTreeSet::from([form_file_binding.to_string()]);
    let open_binding = open_form_file_binding(lines, form_file_binding, call.line);
    let open_line = open_binding.as_ref().map(|(_, line)| *line);
    if let Some((binding, _)) = open_binding {
        candidate_bindings.insert(binding);
    }

    first_readall_for_bindings(file, lines, &candidate_bindings, call.line)
        .map(|(readall_line, binding)| (readall_line, binding, open_line))
}

fn open_form_file_binding(
    lines: &[BodyLine],
    form_file_binding: &str,
    after_line: usize,
) -> Option<(String, usize)> {
    let open_marker = format!("{form_file_binding}.Open(");

    lines
        .iter()
        .filter(|body_line| body_line.line > after_line)
        .find_map(|body_line| {
            binding_for_patterns(&body_line.text, &[open_marker.as_str()])
                .map(|(binding, _)| (binding, body_line.line))
        })
}

fn first_readall_for_bindings(
    file: &ParsedFile,
    lines: &[BodyLine],
    bindings: &BTreeSet<String>,
    after_line: usize,
) -> Option<(usize, String)> {
    let mut readall_markers = import_aliases_for(file, "io")
        .into_iter()
        .map(|alias| format!("{alias}.ReadAll("))
        .collect::<Vec<_>>();
    readall_markers.extend(
        import_aliases_for(file, "io/ioutil")
            .into_iter()
            .map(|alias| format!("{alias}.ReadAll(")),
    );

    lines
        .iter()
        .filter(|body_line| body_line.line > after_line)
        .find_map(|body_line| {
            readall_markers.iter().find_map(|marker| {
                let argument_text = first_argument_after_marker(&body_line.text, marker)?;
                let binding = simple_reference_binding(&argument_text)?;
                bindings
                    .contains(&binding)
                    .then_some((body_line.line, binding))
            })
        })
}

fn gin_call_threshold_bytes(call: &GinCallSummary) -> Option<u64> {
    call.argument_texts
        .first()
        .and_then(|argument_text| parse_go_size_expr_bytes(argument_text))
}

fn parse_go_size_expr_bytes(expression: &str) -> Option<u64> {
    let trimmed = trim_wrapping_parens(expression.trim());
    if trimmed.is_empty() {
        return None;
    }

    for prefix in ["int(", "int64(", "uint(", "uint64("] {
        if let Some(inner) = trimmed
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix(')'))
        {
            return parse_go_size_expr_bytes(inner);
        }
    }

    if let Some((left, right)) = trimmed.split_once("<<") {
        let left_value = parse_go_size_expr_bytes(left)?;
        let shift_value = parse_go_size_expr_bytes(right)?;
        let shift = u32::try_from(shift_value).ok()?;
        return left_value.checked_shl(shift);
    }

    if trimmed.contains('*') {
        let mut product = 1u64;
        for part in trimmed.split('*') {
            product = product.checked_mul(parse_go_size_expr_bytes(part)?)?;
        }
        return Some(product);
    }

    trimmed.replace('_', "").parse::<u64>().ok()
}

fn trim_wrapping_parens(mut text: &str) -> &str {
    while text.starts_with('(') && text.ends_with(')') {
        let mut depth = 0usize;
        let mut wraps = true;

        for (index, character) in text.char_indices() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 && index != text.len() - 1 {
                        wraps = false;
                        break;
                    }
                }
                _ => {}
            }
        }

        if !wraps {
            break;
        }

        text = text[1..text.len() - 1].trim();
    }

    text
}

fn format_size_bytes(bytes: u64) -> String {
    let mib = 1024 * 1024;
    if bytes.is_multiple_of(mib) {
        format!("{} MiB", bytes / mib)
    } else {
        format!("{bytes} bytes")
    }
}

pub(crate) fn prepare_like_call_lines(lines: &[BodyLine]) -> Vec<(usize, String)> {
    let mut prepare_calls = Vec::new();

    for body_line in lines {
        for marker in [".PrepareContext(", ".Prepare("] {
            if body_line.text.contains(marker)
                && let Some(query_text) = first_string_literal_after_marker(&body_line.text, marker)
            {
                prepare_calls.push((body_line.line, query_text));
                break;
            }
        }
    }

    prepare_calls
}

fn gin_call_binding_name(call: &crate::analysis::GinCallSummary) -> Option<String> {
    let argument_text = call.argument_texts.first()?.trim();
    let binding_name = argument_text
        .trim_start_matches('&')
        .trim_start_matches('*')
        .trim();
    is_identifier_name(binding_name).then(|| binding_name.to_string())
}

fn is_dynamic_map_binding(function: &ParsedFunction, binding_name: &str) -> bool {
    let compact_binding = binding_name.replace(char::is_whitespace, "");

    function.body_text.lines().any(|line| {
        let compact_line = compact_code_text(strip_line_comment(line));
        compact_line.contains(&format!("var{compact_binding}map[string]any"))
            || compact_line.contains(&format!("var{compact_binding}map[string]interface{{}}"))
            || compact_line.contains(&format!("{compact_binding}:=map[string]any{{"))
            || compact_line.contains(&format!("{compact_binding}:=map[string]interface{{}}{{"))
            || compact_line.contains(&format!("{compact_binding}:=make(map[string]any"))
            || compact_line.contains(&format!("{compact_binding}:=make(map[string]interface{{}}"))
    })
}

fn compact_code_text(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn first_string_literal_after_marker(text: &str, marker: &str) -> Option<String> {
    let suffix = text.split_once(marker)?.1;
    for (index, character) in suffix.char_indices() {
        if character != '"' && character != '`' {
            continue;
        }

        let quote = character;
        let mut escaped = false;
        for (end_index, current) in suffix[index + character.len_utf8()..].char_indices() {
            if quote == '"' && current == '\\' && !escaped {
                escaped = true;
                continue;
            }

            if current == quote && (!escaped || quote == '`') {
                let start = index + character.len_utf8();
                let end = index + character.len_utf8() + end_index;
                return Some(suffix[start..end].to_string());
            }

            escaped = false;
        }
    }

    None
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RepeatedArgumentGroupSpec<'a> {
    pub rule_id: &'a str,
    pub message_tail: &'a str,
    pub helper_label: &'a str,
}

fn first_argument_after_marker(text: &str, marker: &str) -> Option<String> {
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

pub(crate) fn url_parse_argument_looks_stable(text: &str) -> bool {
    simple_local_binding(text).is_some() || looks_like_string_literal(text)
}

pub(crate) fn layout_argument_looks_stable(text: &str) -> bool {
    simple_reference_binding(text).is_some() || looks_like_string_literal(text)
}

pub(crate) fn simple_local_binding(text: &str) -> Option<String> {
    let trimmed = text
        .trim()
        .trim_start_matches('&')
        .trim_start_matches('*')
        .trim();
    is_identifier_name(trimmed).then(|| trimmed.to_string())
}

fn looks_like_string_literal(text: &str) -> bool {
    let trimmed = text.trim();
    (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('`') && trimmed.ends_with('`'))
}

fn simple_reference_binding(text: &str) -> Option<String> {
    let trimmed = text
        .trim()
        .trim_start_matches('&')
        .trim_start_matches('*')
        .trim();

    (!trimmed.is_empty()
        && trimmed
            .split('.')
            .all(|segment| is_identifier_name(segment.trim())))
    .then(|| trimmed.to_string())
}

fn is_gin_body_bind_operation(operation: &str) -> bool {
    matches!(
        operation,
        "bind"
            | "bind_json"
            | "bind_toml"
            | "bind_xml"
            | "bind_yaml"
            | "should_bind"
            | "should_bind_body_with"
            | "should_bind_json"
            | "should_bind_toml"
            | "should_bind_xml"
            | "should_bind_yaml"
    )
}

fn is_gin_json_bind_operation(operation: &str) -> bool {
    matches!(
        operation,
        "bind" | "bind_json" | "should_bind" | "should_bind_body_with" | "should_bind_json"
    )
}
