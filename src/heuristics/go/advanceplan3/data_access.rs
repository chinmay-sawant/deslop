use std::collections::BTreeMap;

use crate::analysis::{GormChainStep, GormQueryChain, ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::gin::prepare_like_call_lines;
use super::{
    body_lines, has_import_path, has_sql_like_import, import_aliases_for, is_request_path_function,
    join_lines, BodyLine,
};

pub(crate) fn data_access_performance_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();
    let lines = body_lines(function);
    let request_path = is_request_path_function(file, function);

    findings.extend(gorm_chain_findings(file, function, request_path));
    findings.extend(select_or_get_inside_loop_findings(file, function));
    findings.extend(repeated_same_query_template_findings(file, function));
    findings.extend(exists_via_count_star_findings(file, function));
    findings.extend(duplicate_find_then_preload_findings(file, function));
    findings.extend(gorm_query_shape_findings(file, function, request_path));
    findings.extend(sql_query_shape_findings(file, function, &lines, request_path));
    findings.extend(unbounded_in_clause_findings(file, function, &lines));
    findings.extend(scan_into_map_findings(file, function, &lines));
    findings.extend(rows_to_struct_per_row_findings(file, function, &lines));
    findings.extend(sqlx_select_unbounded_findings(file, function, &lines, request_path));
    findings.extend(orm_tuning_findings(file, function, &lines, request_path));
    findings.extend(find_all_then_manual_paginate_findings(file, function, &lines));
    findings.extend(row_by_row_upsert_findings(file, function, &lines));

    if request_path {
        findings.extend(nested_transaction_findings(file, function, &lines));
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
                && (body_line.text.contains(".Prepare(")
                    || body_line.text.contains(".PrepareContext("))
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
            for query_call in go.db_query_calls.iter().filter(|query_call| {
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

        for query_call in go.db_query_calls.iter().filter(|query_call| {
            query_call.in_loop
                && matches!(
                    query_call.method_name.as_str(),
                    "QueryRow" | "QueryRowContext"
                )
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

        for query_call in go.db_query_calls.iter().filter(|query_call| {
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

        for chain in go
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

        for chain in go
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

        for chain in go
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

        for chain in go.gorm_query_chains.iter().filter(|chain| {
            chain.in_loop && chain.terminal_method == "Scan" && gorm_chain_has_step(chain, "Raw")
        }) {
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

        for chain in go.gorm_query_chains.iter().filter(|chain| {
            chain.in_loop
                && chain.terminal_method == "Find"
                && gorm_chain_has_step(chain, "Association")
        }) {
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

        for chain in go
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

        for chain in go
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

        for chain in go.gorm_query_chains.iter().filter(|chain| {
            chain.in_loop
                && matches!(
                    chain.terminal_method.as_str(),
                    "Update" | "UpdateColumn" | "Updates"
                )
        }) {
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

        for chain in go
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

fn gorm_chain_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    request_path: bool,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    if !has_import_path(file, "gorm.io/gorm") {
        return findings;
    }

    let go = function.go_evidence();

    if request_path {
        for chain in go.gorm_query_chains {
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

    let has_batch_create = go
        .gorm_query_chains
        .iter()
        .any(|chain| chain.terminal_method == "CreateInBatches");
    if !has_batch_create {
        for chain in go
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

fn count_then_find_same_filter_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let count_chains = go
        .gorm_query_chains
        .iter()
        .filter(|chain| !chain.in_loop && chain.terminal_method == "Count")
        .collect::<Vec<_>>();
    let find_chains = go
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
                format!(
                    "Count chain at line {}: {}",
                    count_chain.line,
                    gorm_chain_shape(count_chain)
                ),
                format!(
                    "Find chain at line {}: {}",
                    find_chain.line,
                    gorm_chain_shape(find_chain)
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
    chain
        .steps
        .iter()
        .any(|step| step.method_name == method_name)
}

fn gorm_chain_step<'a>(chain: &'a GormQueryChain, method_name: &str) -> Option<&'a GormChainStep> {
    chain
        .steps
        .iter()
        .find(|step| step.method_name == method_name)
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

fn nested_transaction_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut begin_count = 0usize;

    for body_line in lines {
        if body_line.text.contains(".Begin(") || body_line.text.contains(".BeginTx(") {
            begin_count += 1;
            if begin_count >= 2 {
                findings.push(Finding {
                    rule_id: "nested_transaction_in_request_path".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} starts multiple transactions on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("second Begin-like call observed at line {}", body_line.line),
                        "nested or repeated transactional scopes often indicate missing batching intent".to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn select_or_get_inside_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    for chain in go.gorm_query_chains.iter().filter(|chain| {
        chain.in_loop
            && matches!(chain.terminal_method.as_str(), "Find" | "First" | "Take")
            && !gorm_chain_has_step(chain, "Preload")
    }) {
        findings.push(Finding {
            rule_id: "select_or_get_inside_loop_lookup".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: chain.line,
            end_line: chain.line,
            message: format!(
                "function {} runs ORM lookups inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("chain shape: {}", gorm_chain_shape(chain)),
                "bulk prefetch with IN or Preload is usually cheaper than per-item lookups".to_string(),
            ],
        });
    }

    for query_call in go.db_query_calls.iter().filter(|qc| {
        qc.in_loop
            && matches!(qc.method_name.as_str(), "Query" | "QueryContext" | "Get" | "Select")
    }) {
        findings.push(Finding {
            rule_id: "select_or_get_inside_loop_lookup".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: query_call.line,
            end_line: query_call.line,
            message: format!(
                "function {} runs SQL lookups inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("{} observed inside a loop at line {}", query_call.method_name, query_call.line),
                "bulk prefetch with IN clause is usually cheaper than per-item lookups".to_string(),
            ],
        });
    }

    findings
}

fn row_by_row_upsert_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let go = function.go_evidence();

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        if body_line.text.contains("ON CONFLICT") || body_line.text.contains("ON DUPLICATE") {
            findings.push(Finding {
                rule_id: "row_by_row_upsert_loop".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} executes upsert-style writes row by row",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("upsert query observed inside a loop at line {}", body_line.line),
                    "bulk conflict-handling inserts are usually cheaper than row-by-row upserts".to_string(),
                ],
            });
            break;
        }
    }

    for query_call in go.db_query_calls.iter().filter(|qc| {
        qc.in_loop
            && matches!(qc.method_name.as_str(), "Exec" | "ExecContext")
            && qc.query_text.as_deref().is_some_and(|qt| {
                let upper = qt.to_ascii_uppercase();
                upper.contains("ON CONFLICT") || upper.contains("ON DUPLICATE")
            })
    }) {
        findings.push(Finding {
            rule_id: "row_by_row_upsert_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: query_call.line,
            end_line: query_call.line,
            message: format!(
                "function {} executes upsert queries row by row in a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("{} observed inside a loop at line {}", query_call.method_name, query_call.line),
                "bulk conflict-handling is usually cheaper than per-row upserts".to_string(),
            ],
        });
    }

    findings
}

fn repeated_same_query_template_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut query_groups = BTreeMap::<String, Vec<usize>>::new();

    for query_call in go.db_query_calls.iter().filter(|qc| !qc.in_loop) {
        if let Some(query_text) = &query_call.query_text {
            let normalized = query_text.trim().to_ascii_uppercase();
            if !normalized.is_empty() {
                query_groups.entry(normalized).or_default().push(query_call.line);
            }
        }
    }

    let mut findings = Vec::new();
    for (_, query_lines) in query_groups {
        if query_lines.len() < 2 {
            continue;
        }
        findings.push(Finding {
            rule_id: "repeated_same_query_template_same_function".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: query_lines[1],
            end_line: query_lines[1],
            message: format!(
                "function {} executes the same query template multiple times",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("same query template observed at lines {}", join_lines(&query_lines)),
                "caching or reusing query results is often cheaper than re-executing the same query".to_string(),
            ],
        });
    }

    findings
}

fn exists_via_count_star_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    for query_call in go.db_query_calls.iter().filter(|qc| {
        qc.query_text.as_deref().is_some_and(db_query_text_looks_count)
    }) {
        let count_line = query_call.line;
        let lines_after: Vec<_> = function.body_text.lines().enumerate()
            .filter(|(offset, _)| function.body_start_line + offset > count_line)
            .take(5)
            .collect();

        let only_boolean_use = lines_after.iter().any(|(_, line)| {
            let trimmed = line.trim();
            (trimmed.contains("> 0") || trimmed.contains("== 0") || trimmed.contains("!= 0"))
                && !trimmed.contains("range ")
        });

        if only_boolean_use {
            findings.push(Finding {
                rule_id: "exists_via_count_star".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: count_line,
                end_line: count_line,
                message: format!(
                    "function {} uses COUNT(*) for an existence check",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("COUNT query at line {} compared to zero afterwards", count_line),
                    "EXISTS or LIMIT 1 is usually cheaper when only a boolean answer is needed".to_string(),
                ],
            });
        }
    }

    for chain in go.gorm_query_chains.iter().filter(|c| c.terminal_method == "Count") {
        let count_line = chain.line;
        let lines_after: Vec<_> = function.body_text.lines().enumerate()
            .filter(|(offset, _)| function.body_start_line + offset > count_line)
            .take(5)
            .collect();

        let only_boolean_use = lines_after.iter().any(|(_, line)| {
            let trimmed = line.trim();
            (trimmed.contains("> 0") || trimmed.contains("== 0") || trimmed.contains("!= 0"))
                && !trimmed.contains("range ")
        });

        if only_boolean_use {
            findings.push(Finding {
                rule_id: "exists_via_count_star".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: count_line,
                end_line: count_line,
                message: format!(
                    "function {} uses GORM Count for an existence check",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "EXISTS or LIMIT 1 with First is usually cheaper when only a boolean answer is needed".to_string(),
                ],
            });
        }
    }

    findings
}

fn find_all_then_manual_paginate_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    for chain in go.gorm_query_chains.iter().filter(|c| {
        c.terminal_method == "Find"
            && !gorm_chain_has_step(c, "Limit")
            && !gorm_chain_has_step(c, "Offset")
    }) {
        let has_manual_slice = lines.iter().any(|bl| {
            bl.line > chain.line
                && (bl.text.contains("[:") || bl.text.contains("[offset:") || bl.text.contains("[start:"))
        });

        if has_manual_slice {
            findings.push(Finding {
                rule_id: "find_all_then_manual_paginate_in_go".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} fetches all rows and then slices them in Go",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "database-level Limit/Offset or keyset pagination is usually cheaper than fetching all rows and slicing in Go".to_string(),
                ],
            });
        }
    }

    findings
}

fn duplicate_find_then_preload_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    let plain_finds: Vec<_> = go.gorm_query_chains.iter().filter(|c| {
        c.terminal_method == "Find" && !gorm_chain_has_step(c, "Preload")
    }).collect();

    let preload_finds: Vec<_> = go.gorm_query_chains.iter().filter(|c| {
        (c.terminal_method == "Find" || c.terminal_method == "First") && gorm_chain_has_step(c, "Preload")
    }).collect();

    for plain in &plain_finds {
        for preload in &preload_finds {
            if preload.line > plain.line
                && preload.root_text == plain.root_text
            {
                findings.push(Finding {
                    rule_id: "duplicate_find_then_preload_followup".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: preload.line,
                    end_line: preload.line,
                    message: format!(
                        "function {} fetches rows and then follows up with a separate preload query",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("initial Find at line {}: {}", plain.line, gorm_chain_shape(plain)),
                        format!("follow-up Preload+Find at line {}: {}", preload.line, gorm_chain_shape(preload)),
                        "folding the preload into the initial query usually reduces round trips".to_string(),
                    ],
                });
            }
        }
    }

    findings
}

fn gorm_query_shape_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    request_path: bool,
) -> Vec<Finding> {
    if !has_import_path(file, "gorm.io/gorm") || !request_path {
        return Vec::new();
    }

    let go = function.go_evidence();
    let mut findings = Vec::new();

    for chain in go.gorm_query_chains.iter() {
        if chain.terminal_method == "Find"
            && !gorm_chain_has_step(chain, "Select")
            && !gorm_chain_has_step(chain, "Omit")
            && gorm_chain_has_step(chain, "Where")
        {
            let has_many_fields = chain.steps.len() >= 3;
            if has_many_fields {
                findings.push(Finding {
                    rule_id: "gorm_select_missing_projection_on_wide_model".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: chain.line,
                    end_line: chain.line,
                    message: format!(
                        "function {} fetches wide models without Select or Omit projection",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                        "adding Select(...) or Omit(...) to narrow the result set can reduce transfer and allocation cost".to_string(),
                    ],
                });
            }
        }

        if chain.terminal_method == "Find"
            && gorm_chain_has_step(chain, "Joins")
            && gorm_chain_has_step(chain, "Preload")
            && !gorm_chain_has_step(chain, "Limit")
        {
            findings.push(Finding {
                rule_id: "gorm_joins_plus_preload_plus_find_without_limit".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} combines Joins, Preload, and unbounded Find on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "Joins + Preload + Find without Limit can produce very large result sets and round trips".to_string(),
                ],
            });
        }

        if gorm_chain_has_step(chain, "Order") && !gorm_chain_has_step(chain, "Limit")
            && matches!(chain.terminal_method.as_str(), "Find" | "Scan")
        {
            findings.push(Finding {
                rule_id: "order_by_without_limit_orm_chain".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} orders results without a Limit on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "ordering without a limit can cause expensive full-table sorts on large datasets".to_string(),
                ],
            });
        }

        if let Some(order_step) = gorm_chain_step(chain, "Order") {
            let order_text = order_step.argument_texts.first().cloned().unwrap_or_default().to_ascii_uppercase();
            if order_text.contains("RAND()") || order_text.contains("RANDOM()") {
                findings.push(Finding {
                    rule_id: "order_by_random_request_path".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: order_step.line,
                    end_line: order_step.line,
                    message: format!(
                        "function {} orders by random on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("ORDER BY RAND()/RANDOM() observed at line {}", order_step.line),
                        "random ordering requires a full table scan and sort, which is expensive on large tables".to_string(),
                    ],
                });
            }
        }

        if gorm_chain_has_step(chain, "Distinct")
            && !gorm_chain_has_step(chain, "Select")
            && matches!(chain.terminal_method.as_str(), "Find" | "Scan")
        {
            findings.push(Finding {
                rule_id: "distinct_wide_row_request_path".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} uses Distinct on wide rows without Select projection",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "Distinct on wide rows is expensive; a key-only subquery is usually cheaper".to_string(),
                ],
            });
        }
    }

    findings
}

fn sql_query_shape_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    request_path: bool,
) -> Vec<Finding> {
    if !request_path || !has_sql_like_import(file) {
        return Vec::new();
    }

    let go = function.go_evidence();
    let mut findings = Vec::new();

    for query_call in go.db_query_calls.iter() {
        if let Some(query_text) = &query_call.query_text {
            let upper: String = query_text.to_ascii_uppercase();

            if upper.contains("LOWER(") || upper.contains("UPPER(") || upper.contains("COALESCE(") {
                if upper.contains("WHERE") {
                    findings.push(Finding {
                        rule_id: "lower_or_func_wrapped_indexed_column".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: query_call.line,
                        end_line: query_call.line,
                        message: format!(
                            "function {} uses function-wrapped columns in WHERE clauses",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("query at line {}: {}", query_call.line, query_text),
                            "LOWER/UPPER/COALESCE on indexed columns often defeats index usage".to_string(),
                        ],
                    });
                }
            }

            if upper.contains("DATE(") || upper.contains("CAST(") {
                if upper.contains("WHERE") {
                    findings.push(Finding {
                        rule_id: "date_or_cast_wrapped_indexed_column".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: query_call.line,
                        end_line: query_call.line,
                        message: format!(
                            "function {} uses DATE/CAST on columns in WHERE clauses",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("query at line {}: {}", query_call.line, query_text),
                            "wrapping indexed columns in DATE/CAST often prevents index usage".to_string(),
                        ],
                    });
                }
            }

            if upper.contains("LIKE") && upper.contains("WHERE") {
                if let Some(like_pos) = upper.find("LIKE") {
                    let after_like = &upper[like_pos + 4..];
                    if after_like.trim_start().starts_with("'%") || after_like.trim_start().starts_with("\"%") {
                        findings.push(Finding {
                            rule_id: "leading_wildcard_builder_chain".to_string(),
                            severity: Severity::Info,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: query_call.line,
                            end_line: query_call.line,
                            message: format!(
                                "function {} uses leading wildcard LIKE patterns",
                                function.fingerprint.name
                            ),
                            evidence: vec![
                                format!("query at line {}: {}", query_call.line, query_text),
                                "leading wildcards prevent index usage; consider full-text search or trigram indexes".to_string(),
                            ],
                        });
                    }
                }
            }

            if upper.contains("ORDER BY") && (upper.contains("RAND()") || upper.contains("RANDOM()")) {
                findings.push(Finding {
                    rule_id: "order_by_random_request_path".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: query_call.line,
                    end_line: query_call.line,
                    message: format!(
                        "function {} orders by random in a SQL query on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("query at line {}: {}", query_call.line, query_text),
                        "random ordering requires a full table scan and sort".to_string(),
                    ],
                });
            }
        }
    }

    for body_line in lines {
        if body_line.text.contains("LIKE") && body_line.text.contains("\"%") {
            let already_found = findings.iter().any(|f| f.rule_id == "leading_wildcard_builder_chain");
            if !already_found {
                let upper_text = body_line.text.to_ascii_uppercase();
                if upper_text.contains("WHERE") || upper_text.contains("LIKE") {
                    let has_where_context = go.gorm_query_chains.iter().any(|c| gorm_chain_has_step(c, "Where"));
                    if has_where_context {
                        findings.push(Finding {
                            rule_id: "leading_wildcard_builder_chain".to_string(),
                            severity: Severity::Info,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: body_line.line,
                            end_line: body_line.line,
                            message: format!(
                                "function {} uses leading wildcard search patterns",
                                function.fingerprint.name
                            ),
                            evidence: vec![
                                format!("leading wildcard observed at line {}", body_line.line),
                                "leading wildcards prevent index usage; consider full-text search or trigram indexes".to_string(),
                            ],
                        });
                    }
                }
            }
        }
    }

    for chain in go.gorm_query_chains.iter() {
        if let Some(where_step) = gorm_chain_step(chain, "Where") {
            let or_count = chain.steps.iter().filter(|s| s.method_name == "Or").count();
            if or_count >= 3 {
                findings.push(Finding {
                    rule_id: "many_column_or_filter_chain".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: where_step.line,
                    end_line: where_step.line,
                    message: format!(
                        "function {} builds a large OR filter chain on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                        format!("{} Or() clauses observed", or_count),
                        "large OR chains can prevent index usage; consider a different query strategy".to_string(),
                    ],
                });
            }
        }
    }

    findings
}

fn unbounded_in_clause_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines {
        let text_upper = body_line.text.to_ascii_uppercase();
        if text_upper.contains(" IN (") || text_upper.contains(" IN(") {
            if body_line.text.contains("...") || body_line.text.contains("ids)") || body_line.text.contains("items)") {
                findings.push(Finding {
                    rule_id: "unbounded_in_clause_expansion".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} builds an IN clause from a request-driven collection",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("IN clause with collection expansion observed at line {}", body_line.line),
                        "unbounded IN clauses can cause query plan instability; consider batching or temp tables".to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn scan_into_map_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines {
        if body_line.text.contains(".Scan(")
            && (body_line.text.contains("map[string]any")
                || body_line.text.contains("map[string]interface{}"))
        {
            findings.push(Finding {
                rule_id: "scan_into_map_string_any_hot_path".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} scans rows into map[string]any",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("scan into dynamic map at line {}", body_line.line),
                    "typed struct destinations are usually faster and safer than dynamic map scanning".to_string(),
                ],
            });
        }
    }

    let has_dynamic_scan_binding = lines.iter().any(|bl| {
        bl.text.contains("map[string]any{}")
            || bl.text.contains("map[string]interface{}{}")
            || bl.text.contains("make(map[string]any")
            || bl.text.contains("make(map[string]interface{}")
    });

    if has_dynamic_scan_binding {
        for body_line in lines.iter().filter(|bl| bl.in_loop) {
            if body_line.text.contains(".Scan(") || body_line.text.contains(".MapScan(") {
                let already_found = findings.iter().any(|f| f.rule_id == "scan_into_map_string_any_hot_path" && f.start_line == body_line.line);
                if !already_found {
                    findings.push(Finding {
                        rule_id: "scan_into_map_string_any_hot_path".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} scans rows into dynamic maps in a loop",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("dynamic map scan observed inside a loop at line {}", body_line.line),
                            "typed struct destinations are usually faster and safer than dynamic map scanning".to_string(),
                        ],
                    });
                }
            }
        }
    }

    findings
}

fn rows_to_struct_per_row_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        if (body_line.text.contains("new(") || body_line.text.contains("&"))
            && (body_line.text.contains(".Scan(") || body_line.text.contains("rows.Next"))
        {
            let is_struct_alloc = body_line.text.contains("new(")
                || (body_line.text.contains("&") && body_line.text.contains("{}"));

            if is_struct_alloc {
                findings.push(Finding {
                    rule_id: "rows_to_struct_allocation_per_row_without_reuse".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} allocates a new struct for each row scan",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("per-row struct allocation observed at line {}", body_line.line),
                        "reusing a destination struct and copying per-row reduces allocation pressure".to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn sqlx_select_unbounded_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    request_path: bool,
) -> Vec<Finding> {
    if !request_path {
        return Vec::new();
    }

    let mut findings = Vec::new();

    let has_sqlx = has_import_path(file, "github.com/jmoiron/sqlx");
    if has_sqlx {
        for body_line in lines {
            if (body_line.text.contains(".Select(") || body_line.text.contains("sqlx.Select("))
                && !function.body_text.contains("LIMIT")
                && !function.body_text.contains("limit")
            {
                findings.push(Finding {
                    rule_id: "sqlx_select_large_slice_without_limit".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} uses sqlx.Select without a visible LIMIT on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("sqlx.Select observed at line {} without LIMIT in query", body_line.line),
                        "unbounded selects can materialize large slices; add LIMIT or pagination".to_string(),
                    ],
                });
                break;
            }
        }
    }

    let has_pgx = has_import_path(file, "github.com/jackc/pgx/v5")
        || has_import_path(file, "github.com/jackc/pgx/v4");
    if has_pgx {
        for body_line in lines {
            if body_line.text.contains("CollectRows(")
                && !function.body_text.contains("LIMIT")
                && !function.body_text.contains("limit")
            {
                findings.push(Finding {
                    rule_id: "pgx_collectrows_unbounded_materialization".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} uses pgx.CollectRows without a visible LIMIT on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("pgx.CollectRows observed at line {} without LIMIT in query", body_line.line),
                        "unbounded row collection can materialize very large slices; add LIMIT or cursor iteration".to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn orm_tuning_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    request_path: bool,
) -> Vec<Finding> {
    if !has_import_path(file, "gorm.io/gorm") {
        return Vec::new();
    }

    let go = function.go_evidence();
    let mut findings = Vec::new();

    let has_bulk_create = go.gorm_query_chains.iter().any(|c| {
        c.terminal_method == "Create"
            && c.steps.iter().any(|s| {
                s.argument_texts.first().is_some_and(|a| a.contains("[]") || a.contains("..."))
            })
    });
    if has_bulk_create {
        let uses_default_tx = !function.body_text.contains("SkipDefaultTransaction")
            && !function.body_text.contains("Session(");
        if uses_default_tx {
            let anchor = go.gorm_query_chains.iter()
                .find(|c| c.terminal_method == "Create")
                .map(|c| c.line)
                .unwrap_or(function.body_start_line);

            findings.push(Finding {
                rule_id: "default_transaction_enabled_for_bulk_create".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: anchor,
                end_line: anchor,
                message: format!(
                    "function {} performs bulk creates with default transaction enabled",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("bulk Create observed at line {anchor}"),
                    "disabling the default transaction via Session(&gorm.Session{{SkipDefaultTransaction: true}}) can improve bulk insert throughput".to_string(),
                ],
            });
        }
    }

    for chain in go.gorm_query_chains.iter().filter(|c| c.terminal_method == "Save") {
        let update_count = function.body_text.lines().filter(|line| {
            let trimmed = line.trim();
            trimmed.contains(". =") || trimmed.contains(" = ")
        }).count();

        if update_count <= 2 && update_count > 0 {
            findings.push(Finding {
                rule_id: "save_for_single_column_change".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} uses Save for a single-column update",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "Update or Updates for specific columns is cheaper than Save which writes the full model".to_string(),
                ],
            });
        }
    }

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        if body_line.text.contains(".Updates(map[string]") {
            findings.push(Finding {
                rule_id: "updates_map_allocated_per_row".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} allocates update maps inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("Updates(map[string]...) inside a loop at line {}", body_line.line),
                    "reusing an update map or using struct updates reduces per-row allocation churn".to_string(),
                ],
            });
            break;
        }
    }

    if request_path {
        let has_findinbatches = go.gorm_query_chains.iter().any(|c| c.terminal_method == "FindInBatches");
        if !has_findinbatches {
            for chain in go.gorm_query_chains.iter().filter(|c| {
                c.terminal_method == "Find"
                    && !gorm_chain_has_step(c, "Limit")
                    && gorm_chain_has_step(c, "Where")
            }) {
                let step_count = chain.steps.len();
                if step_count >= 2 {
                    findings.push(Finding {
                        rule_id: "findinbatches_candidate_for_large_scan".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: chain.line,
                        end_line: chain.line,
                        message: format!(
                            "function {} fetches unbounded results that could use FindInBatches",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("chain shape: {}", gorm_chain_shape(chain)),
                            "FindInBatches or cursor iteration avoids materializing the full result set at once".to_string(),
                        ],
                    });
                }
            }
        }
    }

    if request_path {
        for body_line in lines {
            if body_line.text.contains(".AutoMigrate(") || body_line.text.contains(".Migrator()") {
                findings.push(Finding {
                    rule_id: "automigrate_or_schema_probe_in_request_path".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} runs AutoMigrate or schema probes on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("migration/schema probe observed at line {}", body_line.line),
                        "schema migrations and introspection belong at startup, not on request paths".to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}
