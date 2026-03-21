use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::common::import_alias_lookup;

pub(super) fn allocation_churn_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    function
        .allocation_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "allocation_churn_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} allocates new objects inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "make/new or buffer construction appears inside a loop".to_string(),
                "repeated per-iteration allocation can create avoidable heap churn".to_string(),
            ],
        })
        .collect()
}

pub(super) fn fmt_hot_path_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    function
        .fmt_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "fmt_hot_path".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} formats strings with fmt inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "fmt formatting call appears inside a loop".to_string(),
                "fmt-heavy formatting in iterative paths can be expensive".to_string(),
            ],
        })
        .collect()
}

pub(super) fn reflection_hot_path_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    function
        .reflection_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "reflection_hot_path".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} uses reflection inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "reflect package call appears inside a loop".to_string(),
                "reflection in hot paths often adds avoidable overhead".to_string(),
            ],
        })
        .collect()
}

pub(super) fn string_concat_in_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    function
        .string_concat_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "string_concat_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} concatenates strings inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "loop-local string concatenation can create repeated allocations".to_string(),
            ],
        })
        .collect()
}

pub(super) fn repeated_json_marshaling_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    function
        .json_marshal_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "repeated_json_marshaling".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} marshals JSON inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "encoding/json marshal call appears inside a loop".to_string(),
                "repeated JSON serialization in iterative paths can become a hot allocation site"
                    .to_string(),
            ],
        })
        .collect()
}

pub(super) fn database_query_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for query_call in &function.db_query_calls {
        if query_call.in_loop && query_call.method_name != "Preload" {
            let receiver = query_call.receiver.as_deref().unwrap_or("<unknown>");
            findings.push(Finding {
                rule_id: "n_plus_one_query".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: query_call.line,
                end_line: query_call.line,
                message: format!(
                    "function {} issues a database-style query inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("looped query method: {receiver}.{}", query_call.method_name),
                    "query execution inside loops often turns into N+1 access patterns"
                        .to_string(),
                ],
            });
        }

        let Some(query_text) = &query_call.query_text else {
            continue;
        };
        let normalized = query_text.to_ascii_uppercase();

        if normalized.starts_with("SELECT *") {
            findings.push(Finding {
                rule_id: "wide_select_query".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: query_call.line,
                end_line: query_call.line,
                message: format!(
                    "function {} issues a wide SELECT * query",
                    function.fingerprint.name
                ),
                evidence: vec![format!("query text: {query_text}")],
            });
        }

        if normalized.contains("LIKE '%")
            || normalized.contains(" ORDER BY ") && !normalized.contains(" LIMIT ")
        {
            findings.push(Finding {
                rule_id: "likely_unindexed_query".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: query_call.line,
                end_line: query_call.line,
                message: format!(
                    "function {} uses a query shape that may bypass effective indexing",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("query text: {query_text}"),
                    "leading wildcard filters or ORDER BY without LIMIT often scale poorly"
                        .to_string(),
                ],
            });
        }
    }

    findings
}

pub(super) fn full_dataset_load_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let import_aliases = import_alias_lookup(&file.imports);

    function
        .calls
        .iter()
        .filter_map(|call| {
            let receiver = call.receiver.as_deref()?;
            let import_path = import_aliases.get(receiver)?;
            let evidence = match (import_path.as_str(), call.name.as_str()) {
                ("io", "ReadAll") | ("io/ioutil", "ReadAll") => {
                    Some(format!("{receiver}.{} reads the full stream into memory", call.name))
                }
                ("os", "ReadFile") => {
                    Some(format!("{receiver}.ReadFile loads the whole file before processing"))
                }
                _ => None,
            }?;

            Some(Finding {
                rule_id: "full_dataset_load".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} loads an entire payload into memory",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("import alias {receiver} resolves to {import_path}"),
                    evidence,
                ],
            })
        })
        .collect()
}
