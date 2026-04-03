use super::gorm::{gorm_chain_has_step, gorm_chain_shape, gorm_chain_step};
use super::*;

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

pub(super) fn sql_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    if !has_sql_like_import(file) {
        return Vec::new();
    }

    let go = function.go_evidence();
    let mut findings = Vec::new();

    for body_line in lines {
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
                    format!(
                        "Prepare-like call observed inside a loop at line {}",
                        body_line.line
                    ),
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

    findings
}

pub(super) fn nested_transaction_findings(
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

pub(super) fn select_or_get_inside_loop_findings(
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
                "bulk prefetch with IN or Preload is usually cheaper than per-item lookups"
                    .to_string(),
            ],
        });
    }

    for query_call in go.db_query_calls.iter().filter(|qc| {
        qc.in_loop
            && matches!(
                qc.method_name.as_str(),
                "Query" | "QueryContext" | "Get" | "Select"
            )
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
                format!(
                    "{} observed inside a loop at line {}",
                    query_call.method_name, query_call.line
                ),
                "bulk prefetch with IN clause is usually cheaper than per-item lookups".to_string(),
            ],
        });
    }

    findings
}

pub(super) fn row_by_row_upsert_findings(
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
                    format!(
                        "upsert query observed inside a loop at line {}",
                        body_line.line
                    ),
                    "bulk conflict-handling inserts are usually cheaper than row-by-row upserts"
                        .to_string(),
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
                format!(
                    "{} observed inside a loop at line {}",
                    query_call.method_name, query_call.line
                ),
                "bulk conflict-handling is usually cheaper than per-row upserts".to_string(),
            ],
        });
    }

    findings
}

pub(super) fn repeated_same_query_template_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut query_groups = BTreeMap::<String, Vec<usize>>::new();

    for query_call in go.db_query_calls.iter().filter(|qc| !qc.in_loop) {
        if let Some(query_text) = &query_call.query_text {
            let normalized = query_text.trim().to_ascii_uppercase();
            if !normalized.is_empty() {
                query_groups
                    .entry(normalized)
                    .or_default()
                    .push(query_call.line);
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

pub(super) fn exists_via_count_star_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    for query_call in go.db_query_calls.iter().filter(|qc| {
        qc.query_text
            .as_deref()
            .is_some_and(db_query_text_looks_count)
    }) {
        let count_line = query_call.line;
        let lines_after: Vec<_> = function
            .body_text
            .lines()
            .enumerate()
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
                    format!(
                        "COUNT query at line {} compared to zero afterwards",
                        count_line
                    ),
                    "EXISTS or LIMIT 1 is usually cheaper when only a boolean answer is needed"
                        .to_string(),
                ],
            });
        }
    }

    for chain in go
        .gorm_query_chains
        .iter()
        .filter(|c| c.terminal_method == "Count")
    {
        let count_line = chain.line;
        let lines_after: Vec<_> = function
            .body_text
            .lines()
            .enumerate()
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

pub(super) fn sql_query_shape_findings(
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

            if (upper.contains("LOWER(") || upper.contains("UPPER(") || upper.contains("COALESCE("))
                && upper.contains("WHERE")
            {
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
                        "LOWER/UPPER/COALESCE on indexed columns often defeats index usage"
                            .to_string(),
                    ],
                });
            }

            if (upper.contains("DATE(") || upper.contains("CAST(")) && upper.contains("WHERE") {
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
                        "wrapping indexed columns in DATE/CAST often prevents index usage"
                            .to_string(),
                    ],
                });
            }

            if upper.contains("LIKE")
                && upper.contains("WHERE")
                && let Some(like_pos) = upper.find("LIKE")
            {
                let after_like = &upper[like_pos + 4..];
                if after_like.trim_start().starts_with("'%")
                    || after_like.trim_start().starts_with("\"%")
                {
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

            if upper.contains("ORDER BY")
                && (upper.contains("RAND()") || upper.contains("RANDOM()"))
            {
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
            let already_found = findings
                .iter()
                .any(|f| f.rule_id == "leading_wildcard_builder_chain");
            if !already_found {
                let upper_text = body_line.text.to_ascii_uppercase();
                if upper_text.contains("WHERE") || upper_text.contains("LIKE") {
                    let has_where_context = go
                        .gorm_query_chains
                        .iter()
                        .any(|c| gorm_chain_has_step(c, "Where"));
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

pub(super) fn unbounded_in_clause_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines {
        let text_upper = body_line.text.to_ascii_uppercase();
        if (text_upper.contains(" IN (") || text_upper.contains(" IN("))
            && (body_line.text.contains("...")
                || body_line.text.contains("ids)")
                || body_line.text.contains("items)"))
        {
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

    findings
}

pub(super) fn scan_into_map_findings(
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
                let already_found = findings.iter().any(|f| {
                    f.rule_id == "scan_into_map_string_any_hot_path"
                        && f.start_line == body_line.line
                });
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

pub(super) fn rows_to_struct_per_row_findings(
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

pub(super) fn sqlx_select_unbounded_findings(
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
                        format!(
                            "sqlx.Select observed at line {} without LIMIT in query",
                            body_line.line
                        ),
                        "unbounded selects can materialize large slices; add LIMIT or pagination"
                            .to_string(),
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
