use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{GormChainStep, GormQueryChain, ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

#[derive(Debug, Clone)]
struct BodyLine {
    line: usize,
    text: String,
    in_loop: bool,
}

pub(super) fn go_advanceplan3_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for function in &file.functions {
        findings.extend(go_advanceplan3_function_findings(file, function));
    }

    findings
}

fn go_advanceplan3_function_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if file.is_test_file || function.is_test_function {
        return Vec::new();
    }

    let mut findings = core_hot_path_findings(file, function);
    findings.extend(data_access_performance_findings(file, function));
    findings.extend(gin_request_performance_findings(file, function));
    findings
}

fn core_hot_path_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines = body_lines(function);

    findings.extend(repeated_parse_findings(file, function));

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
        .map(|alias| vec![
            format!("{alias}.NewWriter("),
            format!("{alias}.NewWriterLevel("),
            format!("{alias}.NewReader("),
        ])
        .flatten()
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
            &[
                "gzip.NewWriter(",
                "gzip.NewWriterLevel(",
                "gzip.NewReader(",
            ],
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
        let writer_pattern_refs = writer_patterns.iter().map(String::as_str).collect::<Vec<_>>();
        for (name, line, _) in binding_matches(&lines, &writer_pattern_refs) {
            let flush_line = lines
                .iter()
                .find(|body_line| body_line.in_loop && body_line.text.contains(&format!("{name}.Flush()")))
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
                        format!("{}.Flush() observed inside a loop at line {}", name, flush_line),
                        "flushing once per row usually reduces buffering effectiveness".to_string(),
                    ],
                });
            }
        }
    }

    findings
}

fn data_access_performance_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines = body_lines(function);
    let request_path = is_request_path_function(file, function);

    findings.extend(gorm_chain_findings(file, function, request_path));

    if request_path {
        for alias in import_aliases_for(file, "database/sql") {
            for body_line in &lines {
                if body_line.text.contains(&format!("{alias}.Open("))
                    || body_line.text.contains(&format!("{alias}.OpenDB("))
                {
                    findings.push(Finding {
                        rule_id: "sql_open_per_request".to_string(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} opens a database/sql handle on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("{}.Open(...) observed at line {}", alias, body_line.line),
                            "database pools are usually initialized once and reused across requests"
                                .to_string(),
                        ],
                    });
                }
            }
        }

        let mut prepare_groups = BTreeMap::<String, Vec<usize>>::new();
        for (line, query_text) in prepare_like_call_lines(&lines) {
            prepare_groups.entry(query_text).or_default().push(line);
        }

        for (query_text, prepare_lines) in prepare_groups {
            if prepare_lines.len() < 2 {
                continue;
            }

            findings.push(Finding {
                rule_id: "prepare_on_every_request_same_sql".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: prepare_lines[1],
                end_line: prepare_lines[1],
                message: format!(
                    "function {} prepares the same SQL multiple times on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "Prepare-like calls for the same query were observed at lines {}",
                        join_lines(&prepare_lines)
                    ),
                    format!("query text: {query_text}"),
                ],
            });
        }

        if has_sql_like_import(file) {
            for call in &function.calls {
                if matches!(call.name.as_str(), "Ping" | "PingContext") {
                    findings.push(Finding {
                        rule_id: "db_ping_per_request".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: call.line,
                        end_line: call.line,
                        message: format!(
                            "function {} pings a database handle on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "{}{} observed at line {}",
                                call.receiver
                                    .as_deref()
                                    .map(|receiver| format!("{receiver}."))
                                    .unwrap_or_default(),
                                call.name,
                                call.line
                            ),
                            "database connectivity checks are usually better handled during startup or explicit health checks"
                                .to_string(),
                        ],
                    });
                }

                if matches!(
                    call.name.as_str(),
                    "SetMaxOpenConns"
                        | "SetMaxIdleConns"
                        | "SetConnMaxLifetime"
                        | "SetConnMaxIdleTime"
                ) {
                    findings.push(Finding {
                        rule_id: "connection_pool_reconfigured_per_request".to_string(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: call.line,
                        end_line: call.line,
                        message: format!(
                            "function {} reconfigures a DB pool on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "{}{} observed at line {}",
                                call.receiver
                                    .as_deref()
                                    .map(|receiver| format!("{receiver}."))
                                    .unwrap_or_default(),
                                call.name,
                                call.line
                            ),
                            "connection-pool sizing and lifetime settings are usually process-level configuration"
                                .to_string(),
                        ],
                    });
                }
            }
        }

        for alias in import_aliases_for(file, "gorm.io/gorm") {
            for body_line in &lines {
                if body_line.text.contains(&format!("{alias}.Open(")) {
                    findings.push(Finding {
                        rule_id: "gorm_open_per_request".to_string(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} opens a GORM handle on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("{}.Open(...) observed at line {}", alias, body_line.line),
                            "ORM handles and underlying pools are usually reused instead of created per request"
                                .to_string(),
                        ],
                    });
                }
            }
        }
    }

    if has_sql_like_import(file) {
        for body_line in &lines {
            if body_line.in_loop
                && (body_line.text.contains(".Prepare(") || body_line.text.contains(".PrepareContext("))
            {
                findings.push(Finding {
                    rule_id: "prepare_inside_loop".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} prepares statements inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("Prepare-like call observed inside a loop at line {}", body_line.line),
                        "reusing a prepared statement outside the loop is usually cheaper than preparing per iteration"
                            .to_string(),
                    ],
                });
            }

            if body_line.in_loop
                && (body_line.text.contains(".Begin(") || body_line.text.contains(".BeginTx("))
            {
                findings.push(Finding {
                    rule_id: "tx_begin_per_item_loop".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} starts transactions inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("Begin-like call observed inside a loop at line {}", body_line.line),
                        "wrapping the whole batch in one transaction is usually cheaper than beginning one per item"
                            .to_string(),
                    ],
                });
            }
        }

        let has_bulk_write_escape = function.body_text.contains("CreateInBatches(")
            || function.body_text.contains("CopyFrom(")
            || function.body_text.contains("FindInBatches(");
        if !has_bulk_write_escape {
            for query_call in function.db_query_calls.iter().filter(|query_call| {
                query_call.in_loop
                    && matches!(query_call.method_name.as_str(), "Exec" | "ExecContext")
                    && query_call
                        .query_text
                        .as_deref()
                        .is_some_and(db_query_text_looks_write)
            }) {
                findings.push(Finding {
                    rule_id: "exec_inside_loop_without_batch".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: query_call.line,
                    end_line: query_call.line,
                    message: format!(
                        "function {} executes per-row SQL writes inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!(
                            "{} observed inside a loop at line {}",
                            query_call.method_name, query_call.line
                        ),
                        format!(
                            "query text: {}",
                            query_call.query_text.as_deref().unwrap_or_default()
                        ),
                        "a set-based write or batch path is usually cheaper than issuing one Exec per item"
                            .to_string(),
                    ],
                });
            }
        }

        for query_call in function.db_query_calls.iter().filter(|query_call| {
            query_call.in_loop
                && matches!(query_call.method_name.as_str(), "QueryRow" | "QueryRowContext")
        }) {
            findings.push(Finding {
                rule_id: "queryrow_inside_loop_existence_check".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: query_call.line,
                end_line: query_call.line,
                message: format!(
                    "function {} performs QueryRow-style lookups inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "{} observed inside a loop at line {}",
                        query_call.method_name, query_call.line
                    ),
                    query_call
                        .query_text
                        .as_ref()
                        .map(|query_text| format!("query text: {query_text}"))
                        .unwrap_or_else(|| {
                            "loop-local point lookups often want a bulk prefetch path instead"
                                .to_string()
                        }),
                ],
            });
        }

        for query_call in function.db_query_calls.iter().filter(|query_call| {
            query_call.in_loop
                && query_call
                    .query_text
                    .as_deref()
                    .is_some_and(db_query_text_looks_count)
        }) {
            findings.push(Finding {
                rule_id: "count_inside_loop".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: query_call.line,
                end_line: query_call.line,
                message: format!(
                    "function {} executes COUNT-style SQL inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "{} observed inside a loop at line {}",
                        query_call.method_name, query_call.line
                    ),
                    format!(
                        "query text: {}",
                        query_call.query_text.as_deref().unwrap_or_default()
                    ),
                ],
            });
        }

        for chain in function
            .gorm_query_chains
            .iter()
            .filter(|chain| chain.in_loop && chain.terminal_method == "Count")
        {
            findings.push(Finding {
                rule_id: "count_inside_loop".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} runs a GORM Count chain inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
            });
        }

        for chain in function
            .gorm_query_chains
            .iter()
            .filter(|chain| chain.in_loop && gorm_chain_has_step(chain, "Session"))
        {
            findings.push(Finding {
                rule_id: "gorm_session_allocated_per_item".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} allocates GORM sessions inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
            });
        }

        for chain in function
            .gorm_query_chains
            .iter()
            .filter(|chain| chain.in_loop && gorm_chain_has_step(chain, "Preload"))
        {
            findings.push(Finding {
                rule_id: "preload_inside_loop".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} issues preloaded GORM queries inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
            });
        }

        for chain in function
            .gorm_query_chains
            .iter()
            .filter(|chain| chain.in_loop && chain.terminal_method == "Scan" && gorm_chain_has_step(chain, "Raw"))
        {
            findings.push(Finding {
                rule_id: "raw_scan_inside_loop".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} runs raw SQL scans inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
            });
        }

        for chain in function
            .gorm_query_chains
            .iter()
            .filter(|chain| chain.in_loop && chain.terminal_method == "Find" && gorm_chain_has_step(chain, "Association"))
        {
            findings.push(Finding {
                rule_id: "association_find_inside_loop".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} loads GORM associations inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
            });
        }

        for chain in function
            .gorm_query_chains
            .iter()
            .filter(|chain| chain.in_loop && chain.terminal_method == "FirstOrCreate")
        {
            findings.push(Finding {
                rule_id: "first_or_create_in_loop".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} runs FirstOrCreate inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
            });
        }

        for chain in function
            .gorm_query_chains
            .iter()
            .filter(|chain| chain.in_loop && chain.terminal_method == "Save")
        {
            findings.push(Finding {
                rule_id: "save_in_loop_full_model".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} saves full GORM models inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
            });
        }

        for chain in function
            .gorm_query_chains
            .iter()
            .filter(|chain| {
                chain.in_loop
                    && matches!(
                        chain.terminal_method.as_str(),
                        "Update" | "UpdateColumn" | "Updates"
                    )
            })
        {
            findings.push(Finding {
                rule_id: "update_single_row_in_loop_without_batch".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} updates rows one at a time inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
            });
        }

        for chain in function
            .gorm_query_chains
            .iter()
            .filter(|chain| chain.in_loop && chain.terminal_method == "Delete")
        {
            findings.push(Finding {
                rule_id: "delete_single_row_in_loop_without_batch".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} deletes rows one at a time inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
            });
        }
    }

    findings
}

fn gin_request_performance_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !is_gin_handler(file, function) {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let lines = body_lines(function);
    let body_bind_calls = function
        .gin_calls
        .iter()
        .filter(|call| is_gin_body_bind_operation(&call.operation))
        .collect::<Vec<_>>();
    let json_bind_calls = function
        .gin_calls
        .iter()
        .filter(|call| is_gin_json_bind_operation(&call.operation))
        .collect::<Vec<_>>();
    let raw_data_line = function
        .gin_calls
        .iter()
        .find(|call| call.operation == "get_raw_data")
        .map(|call| call.line);

    if let (Some(raw_data_line), Some(bind_line)) = (
        raw_data_line,
        json_bind_calls.first().map(|call| call.line),
    ) {
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

    let readall_body_line = function
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

    if let Some(bind_body_with_call) = function
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
                format!("ShouldBindBodyWith(...) observed at line {}", bind_body_with_call.line),
                "no second body bind or explicit body reread was observed in the same handler"
                    .to_string(),
            ],
        });
    }

    for call in function
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

    for call in function
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

    for call in &function.gin_calls {
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
            && (!function.goroutines.is_empty() || !function.loop_goroutines.is_empty())
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
    let data_line = function
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

fn prepare_like_call_lines(lines: &[BodyLine]) -> Vec<(usize, String)> {
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
    text.chars().filter(|character| !character.is_whitespace()).collect()
}

fn first_string_literal_after_marker(text: &str, marker: &str) -> Option<String> {
    let suffix = text.split_once(marker)?.1;
    let mut chars = suffix.char_indices();
    while let Some((index, character)) = chars.next() {
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

fn is_request_path_function(file: &ParsedFile, function: &ParsedFunction) -> bool {
    is_gin_handler(file, function) || is_http_handler(file, function)
}

fn is_gin_handler(file: &ParsedFile, function: &ParsedFunction) -> bool {
    import_aliases_for(file, "github.com/gin-gonic/gin")
        .into_iter()
        .any(|alias| function.signature_text.contains(&format!("*{alias}.Context")))
}

fn is_http_handler(file: &ParsedFile, function: &ParsedFunction) -> bool {
    import_aliases_for(file, "net/http").into_iter().any(|alias| {
        function
            .signature_text
            .contains(&format!("{alias}.ResponseWriter"))
            && function.signature_text.contains(&format!("*{alias}.Request"))
    })
}

fn has_sql_like_import(file: &ParsedFile) -> bool {
    [
        "database/sql",
        "github.com/jmoiron/sqlx",
        "github.com/jackc/pgx/v5",
        "github.com/jackc/pgx/v5/pgxpool",
        "github.com/jackc/pgx/v4",
        "github.com/jackc/pgx/v4/pgxpool",
    ]
    .iter()
    .any(|path| has_import_path(file, path))
}

fn has_import_path(file: &ParsedFile, path: &str) -> bool {
    file.imports.iter().any(|import| import.path == path)
}

fn binding_matches(lines: &[BodyLine], patterns: &[&str]) -> Vec<(String, usize, String)> {
    let mut matches = Vec::new();

    for body_line in lines {
        if let Some((name, target)) = binding_for_patterns(&body_line.text, patterns) {
            matches.push((name, body_line.line, target));
        }
    }

    matches
}

fn binding_for_patterns(text: &str, patterns: &[&str]) -> Option<(String, String)> {
    let (left, right) = split_assignment(text)?;
    let target = patterns
        .iter()
        .find(|pattern| right.contains(**pattern))?
        .to_string();
    let binding = left
        .trim()
        .trim_start_matches("var ")
        .split(',')
        .next()?
        .split_whitespace()
        .next()?
        .trim();
    is_identifier_name(binding).then(|| (binding.to_string(), target))
}

fn split_assignment(text: &str) -> Option<(&str, &str)> {
    if let Some((left, right)) = text.split_once(":=") {
        return Some((left, right));
    }

    if text.contains("==") || text.contains("!=") || text.contains("<=") || text.contains(">=") {
        return None;
    }

    text.split_once(" = ")
        .or_else(|| text.split_once('='))
        .filter(|(left, _)| !left.trim_start().starts_with("if "))
}

fn body_lines(function: &ParsedFunction) -> Vec<BodyLine> {
    let mut brace_depth = 0usize;
    let mut loop_exit_depths = Vec::new();
    let mut lines = Vec::new();

    for (offset, raw_line) in function.body_text.lines().enumerate() {
        let line_no = function.body_start_line + offset;
        let stripped = strip_line_comment(raw_line).trim().to_string();
        let closing_braces = stripped.chars().filter(|character| *character == '}').count();
        for _ in 0..closing_braces {
            brace_depth = brace_depth.saturating_sub(1);
            while loop_exit_depths
                .last()
                .is_some_and(|exit_depth| *exit_depth > brace_depth)
            {
                loop_exit_depths.pop();
            }
        }

        let starts_loop = contains_keyword(&stripped, "for");
        let in_loop = !loop_exit_depths.is_empty() || starts_loop;
        let opening_braces = stripped.chars().filter(|character| *character == '{').count();
        if starts_loop {
            loop_exit_depths.push(brace_depth + opening_braces.max(1));
        }

        brace_depth += opening_braces;
        lines.push(BodyLine {
            line: line_no,
            text: stripped,
            in_loop,
        });
    }

    lines
}

fn import_aliases_for(file: &ParsedFile, import_path: &str) -> Vec<String> {
    file.imports
        .iter()
        .filter(|import| import.path == import_path)
        .map(|import| import.alias.clone())
        .collect()
}

fn json_aliases(file: &ParsedFile) -> Vec<String> {
    import_aliases_for(file, "encoding/json")
}

fn first_line_with_any(function: &ParsedFunction, markers: &[&str]) -> Option<usize> {
    function
        .body_text
        .lines()
        .enumerate()
        .find(|(_, line)| markers.iter().any(|marker| line.contains(marker)))
        .map(|(offset, _)| function.body_start_line + offset)
}

fn has_prior_loop_line(function: &ParsedFunction, line_no: usize) -> bool {
    function
        .body_text
        .lines()
        .enumerate()
        .take_while(|(offset, _)| function.body_start_line + *offset < line_no)
        .any(|(_, line)| contains_keyword(strip_line_comment(line), "for"))
}

fn is_identifier_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && text
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
}

fn join_lines(lines: &[usize]) -> String {
    lines.iter().map(usize::to_string).collect::<Vec<_>>().join(", ")
}

fn contains_keyword(line: &str, keyword: &str) -> bool {
    let bytes = line.as_bytes();
    let keyword_bytes = keyword.as_bytes();

    if keyword_bytes.is_empty() || bytes.len() < keyword_bytes.len() {
        return false;
    }

    for start in 0..=bytes.len() - keyword_bytes.len() {
        if &bytes[start..start + keyword_bytes.len()] != keyword_bytes {
            continue;
        }

        let left_ok = start == 0
            || (!bytes[start - 1].is_ascii_alphanumeric() && bytes[start - 1] != b'_');
        let right_index = start + keyword_bytes.len();
        let right_ok = right_index == bytes.len()
            || (!bytes[right_index].is_ascii_alphanumeric() && bytes[right_index] != b'_');

        if left_ok && right_ok {
            return true;
        }
    }

    false
}

fn strip_line_comment(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

fn repeated_parse_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (parser_family, rule_id, payload_label) in [
        (
            "json_unmarshal",
            "json_unmarshal_same_payload_multiple_times",
            "JSON",
        ),
        (
            "xml_unmarshal",
            "xml_unmarshal_same_payload_multiple_times",
            "XML",
        ),
        (
            "yaml_unmarshal",
            "yaml_unmarshal_same_payload_multiple_times",
            "YAML",
        ),
        (
            "proto_unmarshal",
            "proto_unmarshal_same_payload_multiple_times",
            "protobuf",
        ),
    ] {
        findings.extend(repeated_parse_family_findings(
            file,
            function,
            parser_family,
            rule_id,
            payload_label,
        ));
    }

    findings
}

fn gorm_chain_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    request_path: bool,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    if !has_import_path(file, "gorm.io/gorm") {
        return findings;
    }

    if request_path {
        for chain in &function.gorm_query_chains {
            if let Some(debug_step) = gorm_chain_step(chain, "Debug") {
                findings.push(Finding {
                    rule_id: "gorm_debug_enabled_in_request_path".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: debug_step.line,
                    end_line: debug_step.line,
                    message: format!(
                        "function {} enables GORM debug logging on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("Debug() observed in GORM chain at line {}", debug_step.line),
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                    ],
                });
            }

            if chain.terminal_method == "Find" && !gorm_chain_has_step(chain, "Limit") {
                findings.push(Finding {
                    rule_id: "gorm_find_without_limit_on_handler_path".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: chain.line,
                    end_line: chain.line,
                    message: format!(
                        "function {} runs a GORM Find chain on a request path without an observed Limit",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                        "no Limit(...) step was observed before Find(...)".to_string(),
                    ],
                });
            }

            if chain.terminal_method == "Find"
                && let Some(offset_step) = gorm_chain_step(chain, "Offset")
                && offset_step
                    .argument_texts
                    .first()
                    .is_none_or(|argument| argument != "0")
            {
                findings.push(Finding {
                    rule_id: "offset_pagination_on_large_table".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: offset_step.line,
                    end_line: offset_step.line,
                    message: format!(
                        "function {} uses offset pagination in a GORM Find chain",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("Offset step observed at line {}", offset_step.line),
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                    ],
                });
            }

            if chain.terminal_method == "Find" && is_wide_preload_chain(chain) {
                findings.push(Finding {
                    rule_id: "gorm_preload_clause_associations_on_wide_graph".to_string(),
                    severity: if gorm_chain_has_step(chain, "Limit") {
                        Severity::Info
                    } else {
                        Severity::Warning
                    },
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: chain.line,
                    end_line: chain.line,
                    message: format!(
                        "function {} uses a broad GORM preload graph on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                        "broad Preload(...) chains can widen result sets and round trips significantly"
                            .to_string(),
                    ],
                });
            }
        }

        findings.extend(count_then_find_same_filter_findings(file, function));
    }

    let has_batch_create = function
        .gorm_query_chains
        .iter()
        .any(|chain| chain.terminal_method == "CreateInBatches");
    if !has_batch_create {
        for chain in function
            .gorm_query_chains
            .iter()
            .filter(|chain| chain.in_loop && chain.terminal_method == "Create")
        {
            findings.push(Finding {
                rule_id: "create_single_in_loop_instead_of_batches".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} inserts single GORM rows inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "no CreateInBatches(...) chain was observed in the same function".to_string(),
                ],
            });
        }
    }

    findings
}

fn count_then_find_same_filter_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let count_chains = function
        .gorm_query_chains
        .iter()
        .filter(|chain| !chain.in_loop && chain.terminal_method == "Count")
        .collect::<Vec<_>>();
    let find_chains = function
        .gorm_query_chains
        .iter()
        .filter(|chain| {
            !chain.in_loop
                && chain.terminal_method == "Find"
                && !gorm_chain_has_step(chain, "Limit")
                && !gorm_chain_has_step(chain, "Offset")
        })
        .collect::<Vec<_>>();

    let mut findings = Vec::new();
    for count_chain in count_chains {
        let count_shape = gorm_shape_key(count_chain);
        if count_shape.is_empty() {
            continue;
        }

        let Some(find_chain) = find_chains.iter().copied().find(|find_chain| {
            find_chain.line > count_chain.line
                && find_chain.root_text == count_chain.root_text
                && gorm_shape_key(find_chain) == count_shape
        }) else {
            continue;
        };

        findings.push(Finding {
            rule_id: "count_then_find_same_filter".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: find_chain.line,
            end_line: find_chain.line,
            message: format!(
                "function {} runs Count and Find with the same broad GORM filter chain",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("Count chain at line {}: {}", count_chain.line, gorm_chain_shape(count_chain)),
                format!("Find chain at line {}: {}", find_chain.line, gorm_chain_shape(find_chain)),
            ],
        });
    }

    findings
}

fn repeated_parse_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    parser_family: &str,
    rule_id: &str,
    payload_label: &str,
) -> Vec<Finding> {
    let mut groups = BTreeMap::<String, Vec<_>>::new();

    for call in function.parse_input_calls.iter().filter(|call| {
        call.parser_family == parser_family && call.input_binding.is_some()
    }) {
        groups
            .entry(call.input_binding.clone().unwrap_or_default())
            .or_default()
            .push(call);
    }

    let mut findings = Vec::new();
    for (input_binding, calls) in groups {
        if calls.len() < 2 {
            continue;
        }

        let distinct_targets = calls
            .iter()
            .filter_map(|call| call.target_text.clone())
            .collect::<BTreeSet<_>>();
        if distinct_targets.len() < 2 {
            continue;
        }

        let repeated_lines = calls.iter().map(|call| call.line).collect::<Vec<_>>();
        let anchor_line = repeated_lines[1];
        findings.push(Finding {
            rule_id: rule_id.to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: anchor_line,
            end_line: anchor_line,
            message: format!(
                "function {} unmarshals the same {} payload binding multiple times",
                function.fingerprint.name, payload_label
            ),
            evidence: vec![
                format!(
                    "input binding {input_binding} was unmarshaled at lines {}",
                    join_lines(&repeated_lines)
                ),
                format!("normalized input text: {}", calls[0].input_text),
                format!(
                    "distinct targets observed: {}",
                    distinct_targets.into_iter().collect::<Vec<_>>().join(", ")
                ),
            ],
        });
    }

    findings
}

fn is_wide_preload_chain(chain: &GormQueryChain) -> bool {
    let preload_steps = chain
        .steps
        .iter()
        .filter(|step| step.method_name == "Preload")
        .collect::<Vec<_>>();

    preload_steps.len() >= 3
        || preload_steps.iter().any(|step| {
            step.first_string_arg.as_deref() == Some("clause.Associations")
                || step
                    .argument_texts
                    .first()
                    .is_some_and(|argument| argument.contains("clause.Associations"))
        })
}

fn gorm_shape_key(chain: &GormQueryChain) -> String {
    chain
        .steps
        .iter()
        .filter(|step| {
            !matches!(
                step.method_name.as_str(),
                "Count" | "Find" | "Limit" | "Offset" | "Order"
            )
        })
        .map(|step| {
            let first_argument = step.argument_texts.first().cloned().unwrap_or_default();
            format!("{}({})", step.method_name, first_argument)
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn gorm_chain_shape(chain: &GormQueryChain) -> String {
    chain
        .steps
        .iter()
        .map(|step| {
            if step.argument_texts.is_empty() {
                step.method_name.clone()
            } else {
                format!("{}({})", step.method_name, step.argument_texts.join(", "))
            }
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn gorm_chain_has_step(chain: &GormQueryChain, method_name: &str) -> bool {
    chain.steps.iter().any(|step| step.method_name == method_name)
}

fn gorm_chain_step<'a>(chain: &'a GormQueryChain, method_name: &str) -> Option<&'a GormChainStep> {
    chain.steps.iter().find(|step| step.method_name == method_name)
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

fn db_query_text_looks_write(query_text: &str) -> bool {
    let normalized = query_text.to_ascii_uppercase();
    normalized.contains("INSERT ")
        || normalized.contains("UPDATE ")
        || normalized.contains("DELETE ")
        || normalized.contains("REPLACE ")
}

fn db_query_text_looks_count(query_text: &str) -> bool {
    let normalized = query_text.to_ascii_uppercase();
    normalized.contains("COUNT(") || normalized.contains("COUNT (")
}