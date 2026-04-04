use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::super::common::import_alias_lookup;

pub(crate) fn alloc_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.alloc_loops
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

pub(crate) fn fmt_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.fmt_loops
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

pub(crate) fn reflect_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.reflect_loops
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

pub(crate) fn concat_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.concat_loops
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

pub(crate) fn json_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.json_loops
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

pub(crate) fn db_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    enable_go_semantic: bool,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();
    let nested_loop_signal = enable_go_semantic && has_nested_loop_signal(&function.body_text);

    for query_call in go.db_query_calls {
        if query_call.in_loop && query_call.method_name != "Preload" {
            let receiver = query_call.receiver.as_deref().unwrap_or("<unknown>");
            findings.push(Finding {
                rule_id: "n_plus_one_query".to_string(),
                severity: if nested_loop_signal {
                    Severity::Error
                } else {
                    Severity::Warning
                },
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
                    "query execution inside loops often turns into N+1 access patterns".to_string(),
                    if nested_loop_signal {
                        "semantic opt-in correlated this query with nested loop structure, which makes multiplicative query growth more likely"
                            .to_string()
                    } else {
                        "no nested-loop correlation was required for the baseline warning".to_string()
                    },
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

/// Returns conservative nested-loop findings when Go semantic analysis is enabled.
pub(crate) fn n_squared_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    enable_go_semantic: bool,
) -> Vec<Finding> {
    if !enable_go_semantic || !has_nested_loop_signal(&function.body_text) {
        return Vec::new();
    }

    let mut findings = nested_loop_alloc_findings(file, function);
    findings.extend(nested_loop_concat_findings(file, function));
    findings
}

/// Flags nested-loop allocation churn when allocation evidence already exists.
fn nested_loop_alloc_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.alloc_loops
        .iter()
        .map(|line| Finding {
            rule_id: "likely_n_squared_allocation".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} appears to allocate inside a nested loop",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("allocation observed inside a loop at line {line}"),
                "nested loop structure was also detected in the same function body".to_string(),
                "this is conservative: preallocation or small bounded inputs may still make the code acceptable"
                    .to_string(),
                "consider preallocating outside the inner loop or flattening repeated traversal"
                    .to_string(),
            ],
        })
        .collect()
}

/// Flags nested-loop string concatenation when no builder-style buffer is visible.
fn nested_loop_concat_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if has_builder_usage(&function.body_text) {
        return Vec::new();
    }

    let go = function.go_evidence();

    go.concat_loops
        .iter()
        .map(|line| Finding {
            rule_id: "likely_n_squared_string_concat".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} appears to concatenate strings inside a nested loop",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("string concatenation observed inside a loop at line {line}"),
                "nested loop structure was also detected in the same function body".to_string(),
                "no obvious strings.Builder or bytes.Buffer usage was detected nearby".to_string(),
                "consider strings.Builder, bytes.Buffer, or a pre-sized byte slice for repeated inner-loop string assembly"
                    .to_string(),
            ],
        })
        .collect()
}

fn has_nested_loop_signal(body_text: &str) -> bool {
    let mut brace_depth = 0usize;
    let mut active_loop_exit_depths = Vec::new();

    for raw_line in body_text.split('\n') {
        let line = raw_line.split("//").next().unwrap_or("");
        let closing_braces = line.chars().filter(|character| *character == '}').count();
        for _ in 0..closing_braces {
            brace_depth = brace_depth.saturating_sub(1);
            while active_loop_exit_depths
                .last()
                .is_some_and(|exit_depth| *exit_depth > brace_depth)
            {
                active_loop_exit_depths.pop();
            }
        }

        if contains_keyword(line, "for") {
            if !active_loop_exit_depths.is_empty() {
                return true;
            }

            let opening_braces = line.chars().filter(|character| *character == '{').count();
            active_loop_exit_depths.push(brace_depth + opening_braces.max(1));
        }

        brace_depth += line.chars().filter(|character| *character == '{').count();
    }

    false
}

fn has_builder_usage(body_text: &str) -> bool {
    let normalized = body_text.replace(char::is_whitespace, "");
    normalized.contains("strings.Builder")
        || normalized.contains("bytes.Buffer")
        || normalized.contains("Builder{}")
        || normalized.contains("Buffer{}")
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

        let left_ok =
            start == 0 || !bytes[start - 1].is_ascii_alphanumeric() && bytes[start - 1] != b'_';
        let right_index = start + keyword_bytes.len();
        let right_ok = right_index == bytes.len()
            || !bytes[right_index].is_ascii_alphanumeric() && bytes[right_index] != b'_';

        if left_ok && right_ok {
            return true;
        }
    }

    false
}

pub(crate) fn load_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let import_aliases = import_alias_lookup(&file.imports);

    function
        .calls
        .iter()
        .filter_map(|call| {
            let receiver = call.receiver.as_deref()?;
            let import_path = import_aliases.get(receiver)?;
            let evidence = match (import_path.as_str(), call.name.as_str()) {
                ("io", "ReadAll") | ("io/ioutil", "ReadAll") => Some(format!(
                    "{receiver}.{} reads the full stream into memory",
                    call.name
                )),
                ("os", "ReadFile") => Some(format!(
                    "{receiver}.ReadFile loads the whole file before processing"
                )),
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

#[cfg(test)]
mod tests {
    use super::{contains_keyword, has_builder_usage, has_nested_loop_signal};

    #[test]
    fn detects_nested_loop_signal() {
        let body = "{\nfor _, row := range rows {\n    for _, cell := range row {\n        _ = cell\n    }\n}\n}";
        assert!(has_nested_loop_signal(body));
    }

    #[test]
    fn ignores_sequential_loops() {
        let body = "{\nfor _, row := range rows {\n    _ = row\n}\nfor _, cell := range cells {\n    _ = cell\n}\n}";
        assert!(!has_nested_loop_signal(body));
    }

    #[test]
    fn detects_builder_usage_markers() {
        assert!(has_builder_usage("{\nvar builder strings.Builder\n}\n"));
        assert!(has_builder_usage("{\nvar buf bytes.Buffer\n}\n"));
    }

    #[test]
    fn matches_keyword_boundaries() {
        assert!(contains_keyword("for _, item := range items {", "for"));
        assert!(!contains_keyword("formatValue()", "for"));
    }
}
