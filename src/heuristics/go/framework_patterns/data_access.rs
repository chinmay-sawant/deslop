mod clients;
mod gorm;
mod sql;

use std::collections::BTreeMap;

use crate::analysis::{GormChainStep, GormQueryChain, ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::gin::prepare_like_call_lines;
use super::{
    BodyLine, body_lines, has_import_path, has_sql_like_import, import_aliases_for,
    is_likely_non_request_workload, is_request_path_function, join_lines,
};

use self::clients::{pool_lifecycle_findings, service_client_lifecycle_findings};

use self::gorm::{
    duplicate_find_then_preload_findings, find_all_then_manual_paginate_findings,
    gorm_chain_findings, gorm_loop_findings, gorm_query_shape_findings, orm_tuning_findings,
};
use self::sql::{
    exists_via_count_star_findings, nested_transaction_findings,
    repeated_same_query_template_findings, row_by_row_upsert_findings,
    rows_to_struct_per_row_findings, scan_into_map_findings, select_or_get_inside_loop_findings,
    sql_loop_findings, sql_query_shape_findings, sqlx_select_unbounded_findings,
    unbounded_in_clause_findings,
};

pub(crate) fn data_access_performance_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines = body_lines(function);
    let request_path = is_request_path_function(file, function);

    findings.extend(gorm_chain_findings(file, function, request_path));
    findings.extend(gorm_loop_findings(file, function));
    findings.extend(select_or_get_inside_loop_findings(file, function));
    findings.extend(repeated_same_query_template_findings(file, function));
    findings.extend(exists_via_count_star_findings(file, function));
    findings.extend(duplicate_find_then_preload_findings(file, function));
    findings.extend(gorm_query_shape_findings(file, function, request_path));
    findings.extend(sql_query_shape_findings(
        file,
        function,
        &lines,
        request_path,
    ));
    findings.extend(unbounded_in_clause_findings(file, function, &lines));
    findings.extend(scan_into_map_findings(file, function, &lines));
    findings.extend(rows_to_struct_per_row_findings(file, function, &lines));
    findings.extend(sqlx_select_unbounded_findings(
        file,
        function,
        &lines,
        request_path,
    ));
    findings.extend(sql_loop_findings(file, function, &lines));
    findings.extend(pool_lifecycle_findings(
        file,
        function,
        &lines,
        request_path,
    ));
    findings.extend(service_client_lifecycle_findings(
        file,
        function,
        &lines,
        request_path,
    ));
    findings.extend(orm_tuning_findings(file, function, &lines, request_path));
    findings.extend(find_all_then_manual_paginate_findings(
        file, function, &lines,
    ));
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
            }
        }
    }

    findings
}
