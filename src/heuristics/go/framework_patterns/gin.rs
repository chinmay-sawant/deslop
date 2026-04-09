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

include!("gin/gin_helpers.rs");
include!("gin/gin_request_path_rules.rs");
