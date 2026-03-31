use std::collections::BTreeMap;

use crate::analysis::{GormChainStep, GormQueryChain, ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::gin::prepare_like_call_lines;
use super::{
    body_lines, has_import_path, has_sql_like_import, import_aliases_for, is_request_path_function,
    join_lines,
};

pub(super) fn data_access_performance_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
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
