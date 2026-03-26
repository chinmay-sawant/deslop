use std::collections::BTreeMap;

use crate::analysis::{CallSite, ImportSpec, ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

const REQUEST_METHODS: &[&str] = &[
    "get", "post", "put", "patch", "delete", "head", "options", "request",
];
const SUBPROCESS_CALLS: &[&str] = &["run", "call", "check_call", "check_output", "Popen"];
const PATH_READ_CALLS: &[&str] = &["read_text", "read_bytes"];
const PATH_WRITE_CALLS: &[&str] = &["write_text", "write_bytes"];

pub(super) fn string_concat_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .concat_loops
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

pub(super) fn blocking_sync_io_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !function.fingerprint.kind.starts_with("async") {
        return Vec::new();
    }

    let alias_lookup = import_alias_lookup(&file.imports);
    let mut findings = Vec::new();

    for call in &function.calls {
        let Some(evidence) = blocking_sync_io_evidence(call, &alias_lookup) else {
            continue;
        };

        findings.push(Finding {
            rule_id: "blocking_sync_io_in_async".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "async function {} calls likely blocking sync I/O",
                function.fingerprint.name
            ),
            evidence: vec![evidence],
        });
    }

    findings
}

pub(super) fn full_dataset_load_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter_map(|call| {
            let evidence = if call
                .receiver
                .as_deref()
                .is_some_and(|receiver| receiver.starts_with("open("))
                && matches!(call.name.as_str(), "read" | "readlines")
            {
                Some(format!(
                    "{}.{}() reads the full file into memory",
                    call.receiver.as_deref().unwrap_or("open(...)"),
                    call.name
                ))
            } else if call
                .receiver
                .as_deref()
                .is_some_and(|receiver| receiver.starts_with("Path("))
                && PATH_READ_CALLS.contains(&call.name.as_str())
            {
                Some(format!(
                    "{}.{}() materializes the full file payload",
                    call.receiver.as_deref().unwrap_or("Path(...)"),
                    call.name
                ))
            } else {
                None
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
                evidence: vec![evidence],
            })
        })
        .collect()
}

pub(super) fn list_materialization_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .list_materialization_lines
        .iter()
        .map(|line| Finding {
            rule_id: "list_materialization_first_element".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} materializes a list just to read the first element",
                function.fingerprint.name
            ),
            evidence: vec!["prefer next(iter(...)) when only the first item is needed"
                .to_string()],
        })
        .collect()
}

pub(super) fn deque_candidate_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .deque_operation_lines
        .iter()
        .map(|line| Finding {
            rule_id: "deque_candidate_queue".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} performs queue-style list operations that may want collections.deque",
                function.fingerprint.name
            ),
            evidence: vec!["list pop(0) and insert(0, ...) can create avoidable shifting work"
                .to_string()],
        })
        .collect()
}

pub(super) fn temp_collection_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .temp_collection_lines
        .iter()
        .map(|line| Finding {
            rule_id: "temporary_collection_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} allocates a temporary collection inside a loop",
                function.fingerprint.name
            ),
            evidence: vec!["loop-local list or dict construction can add avoidable allocation churn"
                .to_string()],
        })
        .collect()
}

pub(super) fn recursive_traversal_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function
        || function.recursive_call_lines.is_empty()
        || function.fingerprint.line_count < 12
    {
        return Vec::new();
    }

    function
        .recursive_call_lines
        .iter()
        .map(|line| Finding {
            rule_id: "recursive_traversal_risk".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} uses direct recursion and may need an iterative traversal for deep inputs",
                function.fingerprint.name
            ),
            evidence: vec![format!(
                "line_count={} recursive_calls={}",
                function.fingerprint.line_count,
                function.recursive_call_lines.len()
            )],
        })
        .collect()
}

pub(super) fn list_membership_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .list_membership_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "list_membership_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} performs list-style membership checks inside a loop",
                function.fingerprint.name
            ),
            evidence: vec!["repeated membership checks may want a set when order is irrelevant"
                .to_string()],
        })
        .collect()
}

pub(super) fn repeated_len_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .repeated_len_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "repeated_len_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} repeats len(...) checks inside a loop",
                function.fingerprint.name
            ),
            evidence: vec!["cache the length locally when the container is unchanged in the loop"
                .to_string()],
        })
        .collect()
}

fn blocking_sync_io_evidence(
    call: &CallSite,
    alias_lookup: &BTreeMap<String, String>,
) -> Option<String> {
    if call.receiver.is_none() && call.name == "open" {
        return Some("open() performs blocking file I/O inside async code".to_string());
    }

    if let Some(receiver) = &call.receiver {
        if receiver.starts_with("open(")
            && matches!(call.name.as_str(), "read" | "readlines" | "write")
        {
            return Some(format!(
                "{receiver}.{}() performs blocking file I/O",
                call.name
            ));
        }

        if receiver.starts_with("Path(")
            && (PATH_READ_CALLS.contains(&call.name.as_str())
                || PATH_WRITE_CALLS.contains(&call.name.as_str()))
        {
            return Some(format!(
                "{receiver}.{}() performs blocking file I/O",
                call.name
            ));
        }

        if let Some(import_path) = alias_lookup.get(receiver) {
            if import_path == "requests" && REQUEST_METHODS.contains(&call.name.as_str()) {
                return Some(format!("{receiver}.{} resolves to requests", call.name));
            }
            if import_path == "subprocess" && SUBPROCESS_CALLS.contains(&call.name.as_str()) {
                return Some(format!("{receiver}.{} resolves to subprocess", call.name));
            }
            if import_path == "time" && call.name == "sleep" {
                return Some(format!("{receiver}.sleep resolves to time.sleep"));
            }
        }
    }

    if let Some(import_path) = alias_lookup.get(&call.name) {
        if import_path.starts_with("requests.") && REQUEST_METHODS.contains(&call.name.as_str()) {
            return Some(format!("{}() was imported from {import_path}", call.name));
        }
        if import_path.starts_with("subprocess.") && SUBPROCESS_CALLS.contains(&call.name.as_str())
        {
            return Some(format!("{}() was imported from {import_path}", call.name));
        }
        if import_path == "time.sleep" {
            return Some("sleep() was imported from time.sleep".to_string());
        }
    }

    None
}

fn import_alias_lookup(imports: &[ImportSpec]) -> BTreeMap<String, String> {
    imports
        .iter()
        .map(|import| (import.alias.clone(), import.path.clone()))
        .collect()
}
