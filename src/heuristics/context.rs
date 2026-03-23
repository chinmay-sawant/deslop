use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::common::import_alias_lookup;

const HTTP_CONTEXTLESS_CALLS: &[&str] = &["Get", "Head", "Post", "PostForm", "NewRequest"];
const EXEC_CONTEXTLESS_CALLS: &[&str] = &["Command"];
const NET_CONTEXTLESS_CALLS: &[&str] = &["Dial", "DialTimeout"];

pub(super) fn ctx_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.has_context_parameter {
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

pub(super) fn cancel_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    function
        .context_factory_calls
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

pub(super) fn sleep_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    function
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

pub(super) fn busy_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    function
        .busy_wait_lines
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
