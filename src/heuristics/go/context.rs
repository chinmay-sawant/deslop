use crate::analysis::Language;
use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::{Finding, Severity};

use super::super::common::import_alias_lookup;

const CONTEXT_FACTORY_ESCAPES: &[&str] = &["Background", "TODO"];
const HTTP_CONTEXTLESS_CALLS: &[&str] = &["Get", "Head", "Post", "PostForm", "NewRequest"];
const EXEC_CONTEXTLESS_CALLS: &[&str] = &["Command"];
const NET_CONTEXTLESS_CALLS: &[&str] = &["Dial", "DialTimeout"];
const DB_CONTEXTLESS_CALLS: &[&str] = &["Query", "QueryRow", "Exec", "Get", "Select"];

pub(crate) fn ctx_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.go_evidence().has_context_parameter {
        return Vec::new();
    }

    let import_aliases = import_alias_lookup(&file.imports);

    function
        .calls
        .iter()
        .filter_map(|call| {
            let receiver = call.receiver.as_ref()?;
            let import_path = import_aliases.get(receiver)?;

            let is_context_aware_api = matches!(import_path.as_str(), "net/http")
                && HTTP_CONTEXTLESS_CALLS.contains(&call.name.as_str())
                || matches!(import_path.as_str(), "os/exec")
                    && EXEC_CONTEXTLESS_CALLS.contains(&call.name.as_str())
                || matches!(import_path.as_str(), "net")
                    && NET_CONTEXTLESS_CALLS.contains(&call.name.as_str());

            if !is_context_aware_api {
                return None;
            }

            Some(Finding {
                rule_id: "missing_context".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} performs context-aware work without accepting context.Context",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "context-free API call: {receiver}.{} from {import_path}",
                        call.name
                    ),
                    "function signature does not accept context.Context".to_string(),
                ],
            })
        })
        .collect()
}

pub(crate) fn cancel_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.context_factory_calls
        .iter()
        .filter(|factory_call| {
            !function
                .calls
                .iter()
                .any(|call| call.receiver.is_none() && call.name == factory_call.cancel_name)
        })
        .map(|factory_call| Finding {
            rule_id: "missing_cancel_call".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: factory_call.line,
            end_line: factory_call.line,
            message: format!(
                "function {} creates a derived context without an observed cancel call",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "context.{} assigns cancel function {}",
                    factory_call.factory_name, factory_call.cancel_name
                ),
                "no local cancel() or defer cancel() call was observed".to_string(),
            ],
        })
        .collect()
}

pub(crate) fn sleep_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    function
        .go_evidence()
        .sleep_loops
        .iter()
        .map(|line| Finding {
            rule_id: "sleep_polling".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} uses time.Sleep inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "time.Sleep appears inside a loop, which often indicates polling".to_string(),
            ],
        })
        .collect()
}

pub(crate) fn busy_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.busy_wait_lines
        .iter()
        .map(|line| Finding {
            rule_id: "busy_waiting".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} spins on a select default branch inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "select { default: ... } appears inside a loop".to_string(),
                "default branches in looped selects often indicate busy-waiting".to_string(),
            ],
        })
        .collect()
}

pub(crate) fn propagate_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    if !function.go_evidence().has_context_parameter
        || file.is_test_file
        || has_documented_context_decoupling(function)
    {
        return Vec::new();
    }

    let import_aliases = import_alias_lookup(&file.imports);
    let go = function.go_evidence();
    let mut findings = Vec::new();

    for call in &function.calls {
        let Some(receiver) = call.receiver.as_ref() else {
            continue;
        };
        let Some(import_path) = import_aliases.get(receiver) else {
            continue;
        };

        if import_path == "context" && CONTEXT_FACTORY_ESCAPES.contains(&call.name.as_str()) {
            findings.push(Finding {
                rule_id: "context_background_used".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} accepts context.Context but creates context.{}() locally",
                    function.fingerprint.name, call.name
                ),
                evidence: vec![
                    "function signature already accepts context.Context".to_string(),
                    format!("observed call: {receiver}.{}()", call.name),
                    "prefer propagating the incoming context instead of starting from Background or TODO"
                        .to_string(),
                ],
            });
            continue;
        }

        if !is_contextless_wrapper_call(import_path, &call.name) {
            continue;
        }

        findings.push(Finding {
            rule_id: "missing_context_propagation".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} accepts context.Context but still calls {}.{} without propagating ctx",
                function.fingerprint.name, receiver, call.name
            ),
            evidence: vec![
                "function signature already accepts context.Context".to_string(),
                format!(
                    "context-free API call: {receiver}.{} from {import_path}",
                    call.name
                ),
                "prefer a context-aware variant or request construction that forwards ctx"
                    .to_string(),
            ],
        });
    }

    for call in &function.calls {
        let Some(receiver) = call.receiver.as_deref() else {
            continue;
        };

        if receiver.contains('.') && is_contextless_method_name(&call.name) {
            findings.push(Finding {
                rule_id: "missing_context_propagation".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} accepts context.Context but calls {}.{} without forwarding ctx",
                    function.fingerprint.name, receiver, call.name
                ),
                evidence: vec![
                    "function signature already accepts context.Context".to_string(),
                    format!("receiver-field or wrapper call observed: {receiver}.{}", call.name),
                    "field-backed clients should prefer context-aware request, network, or exec variants"
                        .to_string(),
                ],
            });
        }
    }

    for query_call in go.db_query_calls {
        if !DB_CONTEXTLESS_CALLS.contains(&query_call.method_name.as_str()) {
            continue;
        }

        let receiver = query_call.receiver.as_deref().unwrap_or("<unknown>");
        findings.push(Finding {
            rule_id: "missing_context_propagation".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: query_call.line,
            end_line: query_call.line,
            message: format!(
                "function {} accepts context.Context but still calls {}.{} without a context-aware DB variant",
                function.fingerprint.name, receiver, query_call.method_name
            ),
            evidence: vec![
                "function signature already accepts context.Context".to_string(),
                format!("observed database-style call: {receiver}.{}", query_call.method_name),
                "prefer QueryContext, QueryRowContext, ExecContext, or another ctx-aware wrapper"
                    .to_string(),
            ],
        });
    }

    let Some(package_name) = file.package_name.as_deref() else {
        return findings;
    };
    let Some(current_package) = index.package_for_file(Language::Go, &file.path, package_name)
    else {
        return findings;
    };

    for call in &function.calls {
        if call.receiver.is_some() || !current_package.has_contextless_wrapper_function(&call.name)
        {
            continue;
        }

        findings.push(Finding {
            rule_id: "missing_context_propagation".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} accepts context.Context but calls local wrapper {} without propagating ctx through the wrapper chain",
                function.fingerprint.name, call.name
            ),
            evidence: vec![
                "function signature already accepts context.Context".to_string(),
                format!("local package call observed: {}(...)", call.name),
                "the local callee also performs context-aware work without accepting context.Context"
                    .to_string(),
            ],
        });
    }

    findings
}

fn is_contextless_wrapper_call(import_path: &str, call_name: &str) -> bool {
    matches!(import_path, "net/http") && HTTP_CONTEXTLESS_CALLS.contains(&call_name)
        || matches!(import_path, "os/exec") && EXEC_CONTEXTLESS_CALLS.contains(&call_name)
        || matches!(import_path, "net") && NET_CONTEXTLESS_CALLS.contains(&call_name)
}

fn is_contextless_method_name(call_name: &str) -> bool {
    HTTP_CONTEXTLESS_CALLS.contains(&call_name)
        || EXEC_CONTEXTLESS_CALLS.contains(&call_name)
        || NET_CONTEXTLESS_CALLS.contains(&call_name)
}

fn has_documented_context_decoupling(function: &ParsedFunction) -> bool {
    let mut combined = function
        .doc_comment
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    combined.push('\n');
    combined.push_str(&function.body_text.to_ascii_lowercase());

    [
        "intentionally detached",
        "intentional detached",
        "intentionally decouple",
        "intentionally decoupled",
        "detached context",
        "background worker",
        "top-level producer",
        "independent of request context",
        "survive request cancellation",
    ]
    .iter()
    .any(|marker| combined.contains(marker))
}
