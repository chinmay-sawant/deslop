use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{GinCallSummary, ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::{
    BodyLine, LARGE_MULTIPART_FORM_BYTES, binding_for_patterns, body_lines, import_aliases_for,
    is_identifier_name, is_request_path_function, join_lines, json_aliases, strip_line_comment,
};

pub(crate) fn gin_request_performance_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !is_request_path_function(file, function) {
        return Vec::new();
    }

    let go = function.go_evidence();
    let mut findings = Vec::new();
    let lines = body_lines(function);
    let body_bind_calls = go
        .gin_calls
        .iter()
        .filter(|call| is_gin_body_bind_operation(&call.operation))
        .collect::<Vec<_>>();
    let json_bind_calls = go
        .gin_calls
        .iter()
        .filter(|call| is_gin_json_bind_operation(&call.operation))
        .collect::<Vec<_>>();
    let raw_data_line = go
        .gin_calls
        .iter()
        .find(|call| call.operation == "get_raw_data")
        .map(|call| call.line);

    if let (Some(raw_data_line), Some(bind_line)) =
        (raw_data_line, json_bind_calls.first().map(|call| call.line))
    {
        findings.push(Finding {
            rule_id: "get_raw_data_then_should_bindjson_duplicate_body".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: bind_line,
            end_line: bind_line,
            message: format!(
                "function {} reads the Gin request body and then binds JSON again",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("GetRawData() observed at line {raw_data_line}"),
                format!("JSON bind observed later at line {bind_line}"),
                "Gin request bodies are single-pass unless copied, so this usually duplicates buffering or decoding"
                    .to_string(),
            ],
        });
    }

    let readall_body_line = go
        .gin_calls
        .iter()
        .find(|call| call.operation == "read_request_body")
        .map(|call| call.line);
    if let (Some(readall_body_line), Some(bind_line)) = (
        readall_body_line,
        body_bind_calls.first().map(|call| call.line),
    ) {
        findings.push(Finding {
            rule_id: "readall_body_then_bind_duplicate_deserialize".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: bind_line,
            end_line: bind_line,
            message: format!(
                "function {} reads the entire request body and then binds it again",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("request body read observed at line {readall_body_line}"),
                format!("body bind observed later at line {bind_line}"),
                "this usually materializes and deserializes the same request payload more than once"
                    .to_string(),
            ],
        });
    }

    let body_bind_lines = body_bind_calls
        .iter()
        .map(|call| call.line)
        .collect::<Vec<_>>();
    if body_bind_lines.len() >= 2 {
        findings.push(Finding {
            rule_id: "multiple_shouldbind_calls_same_handler".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: body_bind_lines[1],
            end_line: body_bind_lines[1],
            message: format!(
                "function {} binds the request body multiple times",
                function.fingerprint.name
            ),
            evidence: vec![format!(
                "body bind calls observed at lines {}",
                join_lines(&body_bind_lines)
            )],
        });
    }

    if let Some(bind_body_with_call) = go
        .gin_calls
        .iter()
        .find(|call| call.operation == "should_bind_body_with")
        && body_bind_calls.len() == 1
        && raw_data_line.is_none()
        && readall_body_line.is_none()
    {
        findings.push(Finding {
            rule_id: "shouldbindbodywith_when_single_bind_is_enough".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: bind_body_with_call.line,
            end_line: bind_body_with_call.line,
            message: format!(
                "function {} copies the Gin request body even though only one bind is observed",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "ShouldBindBodyWith(...) observed at line {}",
                    bind_body_with_call.line
                ),
                "no second body bind or explicit body reread was observed in the same handler"
                    .to_string(),
            ],
        });
    }

    for call in go
        .gin_calls
        .iter()
        .filter(|call| matches!(call.operation.as_str(), "bind_json" | "should_bind_json"))
    {
        if let Some(binding_name) = gin_call_binding_name(call)
            && is_dynamic_map_binding(function, &binding_name)
        {
            findings.push(Finding {
                rule_id: "bindjson_into_map_any_hot_endpoint".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} binds JSON into a dynamic map on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("{} targets binding {binding_name} at line {}", call.operation, call.line),
                    format!("binding {binding_name} is declared as map[string]any or map[string]interface{{}}"),
                ],
            });
        }
    }

    for call in go
        .gin_calls
        .iter()
        .filter(|call| matches!(call.operation.as_str(), "bind_query" | "should_bind_query"))
    {
        if let Some(binding_name) = gin_call_binding_name(call)
            && is_dynamic_map_binding(function, &binding_name)
        {
            findings.push(Finding {
                rule_id: "bindquery_into_map_any_hot_endpoint".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} binds query parameters into a dynamic map on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("{} targets binding {binding_name} at line {}", call.operation, call.line),
                    format!("binding {binding_name} is declared as map[string]any or map[string]interface{{}}"),
                ],
            });
        }
    }

    for call in go
        .gin_calls
        .iter()
        .filter(|call| call.operation == "parse_multipart_form")
    {
        if let Some(threshold_bytes) = gin_call_threshold_bytes(call)
            && threshold_bytes >= LARGE_MULTIPART_FORM_BYTES
        {
            findings.push(Finding {
                rule_id: "parsemultipartform_large_default_memory".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} parses multipart uploads with a large in-memory threshold on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "ParseMultipartForm(...) observed at line {} with threshold {}",
                        call.line,
                        format_size_bytes(threshold_bytes)
                    ),
                    format!(
                        "threshold expression: {}",
                        call.argument_texts.first().cloned().unwrap_or_default()
                    ),
                ],
            });
        }
    }

    for call in go
        .gin_calls
        .iter()
        .filter(|call| call.operation == "form_file")
    {
        if let Some((readall_line, readall_binding, open_line)) =
            form_file_readall_flow(file, &lines, call)
        {
            let mut evidence = vec![format!("FormFile(...) observed at line {}", call.line)];
            if let Some(open_line) = open_line {
                evidence.push(format!(
                    "uploaded file handle opened later at line {}",
                    open_line
                ));
            }
            evidence.push(format!(
                "io.ReadAll(...) consumes upload binding {readall_binding} at line {readall_line}"
            ));

            findings.push(Finding {
                rule_id: "formfile_open_readall_whole_upload".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: readall_line,
                end_line: readall_line,
                message: format!(
                    "function {} reads an uploaded form file fully into memory",
                    function.fingerprint.name
                ),
                evidence,
            });
        }
    }

    for call in go.gin_calls {
        if call.operation == "indented_json" {
            findings.push(Finding {
                rule_id: "indentedjson_in_hot_path".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} uses IndentedJSON on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("IndentedJSON(...) observed at line {}", call.line),
                    "pretty-printed JSON responses are usually more expensive than compact JSON"
                        .to_string(),
                ],
            });
        }

        if call.in_loop && matches!(call.operation.as_str(), "json" | "pure_json") {
            findings.push(Finding {
                rule_id: "repeated_c_json_inside_stream_loop".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} writes Gin JSON responses from inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("{} observed inside a loop at line {}", call.operation, call.line),
                    "looped JSON rendering often wants an encoder or streaming response shape instead"
                        .to_string(),
                ],
            });
        }

        if call.operation == "copy"
            && call.in_loop
            && (!go.goroutines.is_empty() || !go.loop_goroutines.is_empty())
        {
            findings.push(Finding {
                rule_id: "gin_context_copy_for_each_item_fanout".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} copies Gin contexts inside a fanout loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("Copy() observed inside a loop at line {}", call.line),
                    "copying the request context once per item adds avoidable request-scope allocation churn"
                        .to_string(),
                ],
            });
        }
    }

    let json_aliases = json_aliases(file);
    let marshal_line = function
        .calls
        .iter()
        .filter(|call| {
            call.receiver
                .as_deref()
                .is_some_and(|receiver| json_aliases.iter().any(|alias| alias == receiver))
        })
        .find(|call| matches!(call.name.as_str(), "Marshal" | "MarshalIndent"))
        .map(|call| call.line);
    let data_line = go
        .gin_calls
        .iter()
        .find(|call| call.operation == "data")
        .map(|call| call.line);
    if let (Some(marshal_line), Some(data_line)) = (marshal_line, data_line)
        && data_line > marshal_line
    {
        findings.push(Finding {
            rule_id: "json_marshaled_manually_then_c_data".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: data_line,
            end_line: data_line,
            message: format!(
                "function {} marshals JSON manually before writing through gin.Context.Data",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("json.Marshal observed at line {marshal_line}"),
                format!("gin Context.Data(...) observed later at line {data_line}"),
                "direct Gin JSON render helpers are usually simpler than manual marshal-plus-data paths"
                    .to_string(),
            ],
        });
    }

    let read_file_lines = request_path_read_file_lines(file, &lines);
    if let (Some(read_file_line), Some(data_line)) = (read_file_lines.first().copied(), data_line)
        && data_line > read_file_line
    {
        findings.push(Finding {
            rule_id: "servefile_via_readfile_then_c_data".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: data_line,
            end_line: data_line,
            message: format!(
                "function {} reads a file into memory and then writes it through gin.Context.Data",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("file read observed at line {read_file_line}"),
                format!("gin Context.Data(...) observed later at line {data_line}"),
                "Gin file helpers or streaming paths are usually cheaper than read-all-plus-data handlers"
                    .to_string(),
            ],
        });
    }

    for alias in import_aliases_for(file, "net/http/httputil") {
        for body_line in &lines {
            if body_line.text.contains(&format!("{alias}.DumpRequest("))
                || body_line.text.contains(&format!("{alias}.DumpRequestOut("))
                || body_line.text.contains(&format!("{alias}.DumpResponse("))
            {
                findings.push(Finding {
                    rule_id: "dumprequest_or_dumpresponse_in_hot_path".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} dumps full HTTP payloads on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("httputil dump helper observed at line {}", body_line.line),
                        "request and response dumps can add avoidable allocation and logging cost on hot paths"
                            .to_string(),
                    ],
                });
            }
        }
    }

    for read_file_line in read_file_lines {
        findings.push(Finding {
            rule_id: "file_or_template_read_per_request".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: read_file_line,
            end_line: read_file_line,
            message: format!(
                "function {} reads files directly on a request path",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("file read observed at line {read_file_line}"),
                "request-time file reads usually want startup caching or a dedicated static-file path"
                    .to_string(),
            ],
        });
    }

    findings.extend(template_parse_in_handler_findings(file, function, &lines));
    findings.extend(loadhtml_in_request_path_findings(file, function, &lines));
    findings.extend(middleware_allocation_findings(file, function, &lines));
    findings.extend(env_config_per_request_findings(file, function, &lines));
    findings.extend(upstream_fanout_findings(file, function, &lines));
    findings.extend(export_buffering_findings(file, function, &lines));
    findings.extend(repeated_body_rewind_findings(file, function, &lines));
    findings.extend(middleware_rebinds_body_findings(file, function, &lines));
    findings.extend(no_streaming_for_large_export_findings(
        file, function, &lines,
    ));
    findings.extend(large_map_response_findings(file, function, &lines));
    findings.extend(gin_logger_debug_body_findings(file, function, &lines));
    findings.extend(upstream_json_decode_same_response_findings(
        file, function, &lines,
    ));
    findings.extend(no_batching_db_write_loop_findings(file, function, &lines));

    findings
}

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

fn template_parse_in_handler_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let mut template_aliases = import_aliases_for(file, "html/template");
    template_aliases.extend(import_aliases_for(file, "text/template"));

    for alias in &template_aliases {
        for body_line in lines {
            if body_line.text.contains(&format!("{alias}.New("))
                || body_line.text.contains(&format!("{alias}.Must("))
            {
                findings.push(Finding {
                    rule_id: "template_parse_in_handler".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} creates or parses templates inside a request handler",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("template construction observed at line {}", body_line.line),
                        "templates should be parsed at startup and cached, not created per request"
                            .to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn loadhtml_in_request_path_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines {
        if body_line.text.contains(".LoadHTMLGlob(") || body_line.text.contains(".LoadHTMLFiles(") {
            findings.push(Finding {
                rule_id: "loadhtmlglob_or_loadhtmlfiles_in_request_path".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} loads HTML templates on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("LoadHTML* observed at line {}", body_line.line),
                    "Gin HTML template loading should happen at startup via router.LoadHTMLGlob, not per request".to_string(),
                ],
            });
        }
    }

    findings
}

fn middleware_allocation_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for alias in import_aliases_for(file, "net/http") {
        for body_line in lines {
            if body_line.text.contains(&format!("{alias}.Client{{"))
                || body_line.text.contains(&format!("{alias}.Client {{"))
                || body_line.text.contains(&format!("&{alias}.Client"))
            {
                findings.push(Finding {
                    rule_id: "middleware_allocates_http_client_per_request".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} allocates an HTTP client inside a request handler",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("http.Client allocation observed at line {}", body_line.line),
                        "HTTP clients should be reused across requests; allocate once and share"
                            .to_string(),
                    ],
                });
                break;
            }
        }
    }

    for alias in import_aliases_for(file, "database/sql") {
        for body_line in lines {
            if body_line.text.contains(&format!("{alias}.Open(")) {
                findings.push(Finding {
                    rule_id: "middleware_allocates_db_or_gorm_handle_per_request".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} opens database connections inside a handler or middleware",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("sql.Open observed at line {}", body_line.line),
                        "database handles should be shared across requests, not created per handler call".to_string(),
                    ],
                });
                break;
            }
        }
    }

    for alias in import_aliases_for(file, "gorm.io/gorm") {
        for body_line in lines {
            if body_line.text.contains(&format!("{alias}.Open(")) {
                findings.push(Finding {
                    rule_id: "middleware_allocates_db_or_gorm_handle_per_request".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} opens GORM connections inside a handler or middleware",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("gorm.Open observed at line {}", body_line.line),
                        "database handles should be shared across requests, not created per handler call".to_string(),
                    ],
                });
                break;
            }
        }
    }

    for alias in import_aliases_for(file, "regexp") {
        for body_line in lines {
            if body_line.text.contains(&format!("{alias}.Compile("))
                || body_line.text.contains(&format!("{alias}.MustCompile("))
            {
                findings.push(Finding {
                    rule_id: "middleware_allocates_regex_or_template_per_request".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} compiles regexps inside a request handler",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("regexp compile observed at line {}", body_line.line),
                        "compiled regexps should be cached as package-level variables".to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn env_config_per_request_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let config_markers: Vec<String> = import_aliases_for(file, "os")
        .into_iter()
        .flat_map(|alias| vec![format!("{alias}.Getenv("), format!("{alias}.LookupEnv(")])
        .collect();

    if config_markers.is_empty() {
        return findings;
    }

    let env_reads: Vec<_> = lines
        .iter()
        .filter(|bl| config_markers.iter().any(|m| bl.text.contains(m)))
        .collect();

    if env_reads.len() >= 3 {
        let anchor = env_reads[0].line;
        findings.push(Finding {
            rule_id: "env_or_config_lookup_per_request".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: anchor,
            end_line: anchor,
            message: format!(
                "function {} reads environment variables on a request path",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("{} environment reads observed in handler", env_reads.len()),
                "config values should be read at startup and cached, not per request".to_string(),
            ],
        });
    }

    findings
}

fn upstream_fanout_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    for alias in import_aliases_for(file, "net/http") {
        if function.body_text.contains("SetLimit(") {
            break;
        }
        for body_line in lines.iter().filter(|bl| bl.in_loop) {
            if body_line.text.contains(&format!("{alias}.Get("))
                || body_line.text.contains(&format!("{alias}.Post("))
                || body_line.text.contains(&format!("{alias}.Do("))
                || body_line.text.contains(".Do(")
            {
                findings.push(Finding {
                    rule_id: "upstream_http_call_per_item_in_handler_loop".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} makes upstream HTTP calls per item in a handler loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("HTTP call observed inside a loop at line {}", body_line.line),
                        "batch upstream calls or use concurrent fanout with limits instead of sequential per-item calls".to_string(),
                    ],
                });
                break;
            }
        }
    }

    let mut http_call_urls = BTreeMap::<String, Vec<usize>>::new();
    for body_line in lines {
        for alias in import_aliases_for(file, "net/http") {
            if (body_line.text.contains(&format!("{alias}.Get("))
                || body_line.text.contains(&format!("{alias}.Post(")))
                && let Some(url_arg) = first_argument_after_marker(&body_line.text, ".Get(")
                    .or_else(|| first_argument_after_marker(&body_line.text, ".Post("))
            {
                http_call_urls
                    .entry(url_arg)
                    .or_default()
                    .push(body_line.line);
            }
        }
    }

    for (url, call_lines) in &http_call_urls {
        if call_lines.len() >= 2 {
            findings.push(Finding {
                rule_id: "duplicate_upstream_calls_same_url_same_handler".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call_lines[1],
                end_line: call_lines[1],
                message: format!(
                    "function {} calls the same upstream URL multiple times",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("same URL called at lines {}", join_lines(call_lines)),
                    format!("URL: {url}"),
                    "caching or reusing the response is usually cheaper than duplicate upstream calls".to_string(),
                ],
            });
        }
    }

    if !go.goroutines.is_empty()
        || !go.loop_goroutines.is_empty()
        || function.body_text.contains(".Go(func")
    {
        let has_errgroup =
            function.body_text.contains("errgroup") || function.body_text.contains("Group{");
        let has_concurrency_limit = function.body_text.contains("SetLimit(")
            || function.body_text.contains("make(chan")
            || function.body_text.contains("semaphore");

        if has_errgroup && !has_concurrency_limit {
            let anchor = go
                .goroutines
                .first()
                .copied()
                .unwrap_or(function.body_start_line);
            findings.push(Finding {
                rule_id: "errgroup_fanout_without_limit_in_handler".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: anchor,
                end_line: anchor,
                message: format!(
                    "function {} uses errgroup fanout without a visible concurrency limit",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("goroutine fanout observed near line {anchor}"),
                    "request-driven fanout without limits can overwhelm upstream services; use SetLimit or a semaphore".to_string(),
                ],
            });
        }
    }

    findings
}

fn export_buffering_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_bufio =
        !import_aliases_for(file, "bufio").is_empty() || function.body_text.contains("bufio.");

    let csv_aliases = import_aliases_for(file, "encoding/csv");
    let json_aliases = import_aliases_for(file, "encoding/json");

    if !has_bufio {
        let has_csv_loop_write = !csv_aliases.is_empty()
            && lines
                .iter()
                .any(|bl| bl.in_loop && bl.text.contains(".Write("));
        let has_json_loop_encode = !json_aliases.is_empty()
            && lines
                .iter()
                .any(|bl| bl.in_loop && bl.text.contains(".Encode("));

        if has_csv_loop_write || has_json_loop_encode {
            let writes_to_response = is_request_path_function(file, function)
                || function.signature_text.contains("ResponseWriter")
                || function.body_text.contains("c.Writer")
                || function.body_text.contains("gin.Context");

            if writes_to_response {
                let anchor = lines
                    .iter()
                    .find(|bl| {
                        bl.in_loop && (bl.text.contains(".Write(") || bl.text.contains(".Encode("))
                    })
                    .map(|bl| bl.line)
                    .unwrap_or(function.body_start_line);

                findings.push(Finding {
                    rule_id: "large_csv_or_json_export_without_bufio".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: anchor,
                    end_line: anchor,
                    message: format!(
                        "function {} writes export data in a loop without visible buffering",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("looped write observed at line {anchor}"),
                        "wrapping the response writer in bufio.NewWriter usually improves export throughput".to_string(),
                    ],
                });
            }
        }
    }

    let gzip_aliases = import_aliases_for(file, "compress/gzip");
    if !gzip_aliases.is_empty() {
        for body_line in lines.iter().filter(|bl| bl.in_loop) {
            let creates_gzip = gzip_aliases.iter().any(|alias| {
                body_line.text.contains(&format!("{alias}.NewWriter("))
                    || body_line.text.contains(&format!("{alias}.NewReader("))
            });
            if creates_gzip {
                findings.push(Finding {
                    rule_id: "gzip_or_zip_writer_created_per_chunk".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} recreates gzip writers per chunk in a handler",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("gzip constructor inside a loop at line {}", body_line.line),
                        "reusing one gzip writer per response or using sync.Pool is usually cheaper than per-chunk creation".to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn repeated_body_rewind_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut body_read_count = 0usize;
    let mut rewind_lines = Vec::new();

    for body_line in lines {
        if body_line.text.contains("c.Request.Body")
            && (body_line.text.contains("ReadAll(")
                || body_line.text.contains("NewDecoder(")
                || body_line.text.contains(".Read("))
        {
            body_read_count += 1;
        }
        if body_line.text.contains("NopCloser(")
            || (body_line.text.contains("NewReader(") && body_line.text.contains("c.Request.Body"))
            || body_line.text.contains("c.Request.Body =")
        {
            rewind_lines.push(body_line.line);
        }
    }

    if body_read_count >= 2 && !rewind_lines.is_empty() {
        let anchor = rewind_lines[0];
        findings.push(Finding {
            rule_id: "repeated_body_rewind_for_multiple_decoders".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: anchor,
            end_line: anchor,
            message: format!(
                "function {} reads, rewinds, and decodes the request body multiple times",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("body rewind observed at line {anchor}"),
                format!("{body_read_count} body read operations observed in the same handler"),
                "reading the body once and decoding from the buffered copy avoids repeated rewind overhead".to_string(),
            ],
        });
    }

    findings
}

fn middleware_rebinds_body_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    let handler_bind_calls: Vec<_> = go
        .gin_calls
        .iter()
        .filter(|call| is_gin_body_bind_operation(&call.operation))
        .collect();

    if handler_bind_calls.is_empty() {
        return findings;
    }

    let next_line = lines.iter().find(|bl| bl.text.contains("c.Next()"));

    if let Some(next_bl) = next_line {
        let bind_after_next = handler_bind_calls
            .iter()
            .find(|call| call.line > next_bl.line);

        if let Some(rebind_call) = bind_after_next {
            findings.push(Finding {
                rule_id: "middleware_rebinds_body_after_handler_bind".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: rebind_call.line,
                end_line: rebind_call.line,
                message: format!(
                    "function {} binds the request body after c.Next() when the handler may have already consumed it",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("body bind observed after c.Next() at line {}", rebind_call.line),
                    "the request body is single-pass; binding after the handler has already consumed it will fail or produce empty results".to_string(),
                ],
            });
        }
    }

    findings
}

fn no_streaming_for_large_export_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let ja = json_aliases(file);
    let has_marshal_before_write = ja
        .iter()
        .any(|alias| function.body_text.contains(&format!("{alias}.Marshal(")));

    let has_collection_marshal = has_marshal_before_write
        && lines
            .iter()
            .any(|bl| bl.text.contains("append(") || bl.text.contains("range "));

    let response_write_line = response_write_line(function, lines);

    if has_collection_marshal && let Some(response_write_line) = response_write_line {
        let marshal_line = lines
            .iter()
            .find(|bl| {
                ja.iter()
                    .any(|alias| bl.text.contains(&format!("{alias}.Marshal(")))
            })
            .map(|bl| bl.line);

        if let Some(marshal_line) = marshal_line
            && response_write_line > marshal_line
        {
            findings.push(Finding {
                    rule_id: "no_streaming_for_large_export_handler".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: response_write_line,
                    end_line: response_write_line,
                    message: format!(
                        "function {} materializes a collection into memory before writing the response",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("json.Marshal observed at line {marshal_line}"),
                        format!("response write observed at line {response_write_line}"),
                        "streaming with json.NewEncoder or chunked writes avoids materializing the full payload in memory".to_string(),
                    ],
                });
        }
    }

    findings
}

fn large_map_response_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    let map_literal_lines: Vec<_> = lines
        .iter()
        .filter(|bl| {
            (bl.text.contains("map[string]any{")
                || bl.text.contains("map[string]interface{}{")
                || bl.text.contains("gin.H{"))
                && !bl.in_loop
        })
        .collect();

    if map_literal_lines.is_empty() {
        return findings;
    }

    let json_render_call = go.gin_calls.iter().find(|call| {
        matches!(
            call.operation.as_str(),
            "json" | "pure_json" | "indented_json"
        )
    });

    let ja = json_aliases(file);
    let response_write_line = response_write_line(function, lines);
    let has_json_marshal = ja
        .iter()
        .any(|alias| function.body_text.contains(&format!("{alias}.Marshal(")));

    let mut map_key_count = 0usize;
    let mut counting = false;
    for body_line in lines {
        if map_literal_lines.iter().any(|ml| ml.line == body_line.line) {
            counting = true;
            map_key_count = 0;
        }
        if counting {
            if body_line.text.contains(":") && !body_line.text.starts_with("//") {
                map_key_count += 1;
            }
            if body_line.text.contains("}") {
                counting = false;
            }
        }
    }

    let large_map = map_key_count >= 5 || map_literal_lines.len() >= 3;

    if large_map {
        if let Some(json_call) = json_render_call {
            let map_line = map_literal_lines[0].line;
            findings.push(Finding {
                rule_id: "large_h_payload_built_only_for_json_response".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: json_call.line,
                end_line: json_call.line,
                message: format!(
                    "function {} builds a large dynamic map payload for JSON rendering",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("large map literal observed near line {map_line}"),
                    format!("JSON render at line {}", json_call.line),
                    "a typed response struct avoids transient map allocation and provides compile-time field safety".to_string(),
                ],
            });
        }

        if has_json_marshal && response_write_line.is_some() && json_render_call.is_none() {
            let map_line = map_literal_lines[0].line;
            let anchor = response_write_line.unwrap_or(map_line);
            findings.push(Finding {
                rule_id: "repeated_large_map_literal_response_construction".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: anchor,
                end_line: anchor,
                message: format!(
                    "function {} builds large dynamic map responses for manual marshaling",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("large map literal near line {map_line}"),
                    "a typed response struct avoids repeated transient map allocation".to_string(),
                ],
            });
        }
    }

    findings
}

fn response_write_line(function: &ParsedFunction, lines: &[BodyLine]) -> Option<usize> {
    if let Some(line) = function
        .go_evidence()
        .gin_calls
        .iter()
        .find(|call| call.operation == "data")
        .map(|call| call.line)
    {
        return Some(line);
    }

    lines.iter().find_map(|body_line| {
        (body_line.text.contains(".Write(")
            || body_line.text.contains(".Send(")
            || body_line.text.contains(".Blob(")
            || body_line.text.contains(".SendString("))
        .then_some(body_line.line)
    })
}

fn gin_logger_debug_body_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    let log_markers = [
        "log.Print",
        "log.Info",
        "log.Debug",
        "log.Warn",
        "logger.Info",
        "logger.Debug",
        "logger.Warn",
        "fmt.Print",
        "fmt.Sprint",
        "logrus.",
        "slog.",
        "zap.String(",
        "zap.Any(",
    ];

    let body_dump_in_log = lines.iter().find(|bl| {
        let has_log = log_markers.iter().any(|m| bl.text.contains(m));
        let has_body_ref = bl.text.contains("body")
            || bl.text.contains("payload")
            || bl.text.contains("raw")
            || bl.text.contains("c.Request.Body");
        has_log && has_body_ref
    });

    let raw_data_read = go.gin_calls.iter().any(|c| c.operation == "get_raw_data")
        || go
            .gin_calls
            .iter()
            .any(|c| c.operation == "read_request_body");

    if let Some(log_line) = body_dump_in_log
        && raw_data_read
    {
        findings.push(Finding {
            rule_id: "gin_logger_debug_body_logging_on_hot_routes".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: log_line.line,
            end_line: log_line.line,
            message: format!(
                "function {} logs request body content on a request path",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("body logging observed at line {}", log_line.line),
                "logging full request bodies on hot routes adds allocation and I/O cost; consider conditional debug logging".to_string(),
            ],
        });
    }

    findings
}

fn upstream_json_decode_same_response_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let ja = json_aliases(file);
    if ja.is_empty() {
        return findings;
    }

    let http_response_bindings: Vec<(String, usize)> = lines
        .iter()
        .filter_map(|bl| {
            if bl.text.contains("http.Get(")
                || bl.text.contains("http.Post(")
                || bl.text.contains(".Do(")
            {
                let assignment = super::split_assignment(&bl.text)?;
                let binding = assignment.0.trim().split(',').next()?.trim().to_string();
                if super::is_identifier_name(&binding) {
                    return Some((binding, bl.line));
                }
            }
            None
        })
        .collect();

    for (resp_binding, resp_line) in &http_response_bindings {
        let body_binding_text = format!("{resp_binding}.Body");
        let decode_lines: Vec<usize> = lines
            .iter()
            .filter(|bl| {
                bl.line > *resp_line
                    && (bl.text.contains(&body_binding_text) || bl.text.contains("body"))
                    && ja.iter().any(|alias| {
                        bl.text.contains(&format!("{alias}.Unmarshal("))
                            || bl.text.contains(&format!("{alias}.NewDecoder("))
                    })
            })
            .map(|bl| bl.line)
            .collect();

        if decode_lines.len() >= 2 {
            findings.push(Finding {
                rule_id: "upstream_json_decode_same_response_multiple_times".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: decode_lines[1],
                end_line: decode_lines[1],
                message: format!(
                    "function {} decodes the same upstream response body multiple times",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("upstream response from line {resp_line}"),
                    format!("decode operations at lines {}", join_lines(&decode_lines)),
                    "decoding once into a shared intermediate representation avoids duplicate deserialization".to_string(),
                ],
            });
        }
    }

    findings
}

fn no_batching_db_write_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let has_gorm = super::has_import_path(file, "gorm.io/gorm");
    let has_sql = super::has_sql_like_import(file);

    if !has_gorm && !has_sql {
        return findings;
    }

    let has_bulk_escape = function.body_text.contains("CreateInBatches(")
        || function.body_text.contains("CopyFrom(")
        || function.body_text.contains("FindInBatches(")
        || function.body_text.contains("COPY ")
        || function.body_text.contains("bulk");

    if has_bulk_escape {
        return findings;
    }

    let write_markers = [
        ".Create(",
        ".Save(",
        ".Update(",
        ".Updates(",
        ".Delete(",
        ".Exec(",
    ];

    let loop_writes: Vec<usize> = lines
        .iter()
        .filter(|bl| bl.in_loop && write_markers.iter().any(|m| bl.text.contains(m)))
        .map(|bl| bl.line)
        .collect();

    if loop_writes.len() >= 2 {
        let anchor = loop_writes[0];
        findings.push(Finding {
            rule_id: "no_batching_on_handler_driven_db_write_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: anchor,
            end_line: anchor,
            message: format!(
                "function {} performs row-by-row database writes in a handler loop without batching",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("looped write operations at lines {}", join_lines(&loop_writes)),
                "batch writes via CreateInBatches, bulk inserts, or set-based operations are usually faster".to_string(),
            ],
        });
    }

    findings
}
