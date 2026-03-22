use std::collections::BTreeSet;

use crate::analysis::{ImportSpec, ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::common::{import_alias_lookup, is_potentially_blocking_call};

const COORDINATION_METHODS: &[&str] = &["Add", "Done", "Wait", "Go"];

pub(super) fn goroutine_shutdown_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    function
        .goroutine_without_shutdown_lines
        .iter()
        .map(|line| Finding {
            rule_id: "goroutine_without_shutdown_path".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} launches a looping goroutine without an obvious shutdown path",
                function.fingerprint.name
            ),
            evidence: vec![
                "go func literal contains a loop".to_string(),
                "no ctx.Done() or done-channel shutdown signal was observed in the goroutine body"
                    .to_string(),
            ],
        })
        .collect()
}

pub(super) fn mutex_contention_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    imports: &[ImportSpec],
) -> Vec<Finding> {
    let mut findings = function
        .mutex_lock_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "mutex_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} acquires a mutex inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "Lock or RLock appears inside a loop".to_string(),
                "repeated lock acquisition in iterative paths can create contention".to_string(),
            ],
        })
        .collect::<Vec<_>>();

    let import_aliases = import_alias_lookup(imports);
    let mut calls = function.calls.clone();
    calls.sort_by_key(|call| call.line);

    let mut active_locks = BTreeSet::new();
    let mut blocking_lines = BTreeSet::new();

    for call in calls {
        if let Some(receiver) = &call.receiver {
            if matches!(call.name.as_str(), "Lock" | "RLock") {
                active_locks.insert(receiver.clone());
                continue;
            }
            if matches!(call.name.as_str(), "Unlock" | "RUnlock") {
                active_locks.remove(receiver);
                continue;
            }
        }

        if !active_locks.is_empty() && is_potentially_blocking_call(&call, &import_aliases) {
            blocking_lines.insert(call.line);
        }
    }

    findings.extend(blocking_lines.into_iter().map(|line| Finding {
        rule_id: "blocking_call_while_locked".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!(
            "function {} performs a potentially blocking call while a mutex appears held",
            function.fingerprint.name
        ),
        evidence: vec![
            "a blocking or external call was observed between Lock and Unlock".to_string(),
            "holding a mutex across I/O or sleeps can amplify contention".to_string(),
        ],
    }));

    findings
}

pub(super) fn goroutine_coordination_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.goroutine_launch_lines.is_empty() || has_obvious_coordination_signal(function) {
        return Vec::new();
    }

    let mut findings = function
        .goroutine_launch_lines
        .iter()
        .map(|line| Finding {
            rule_id: "goroutine_without_coordination".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} launches a goroutine without an obvious coordination signal",
                function.fingerprint.name
            ),
            evidence: vec![
                "raw go statement observed".to_string(),
                "no context.Context parameter or WaitGroup-like coordination call found"
                    .to_string(),
            ],
        })
        .collect::<Vec<_>>();

    findings.extend(function.goroutine_in_loop_lines.iter().map(|line| Finding {
        rule_id: "goroutine_spawn_in_loop".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: *line,
        end_line: *line,
        message: format!(
            "function {} launches goroutines inside a loop without an obvious coordination signal",
            function.fingerprint.name
        ),
        evidence: vec![
            "raw go statement appears inside a loop".to_string(),
            "loop-local goroutine fan-out without context or WaitGroup-like coordination can grow unexpectedly"
                .to_string(),
        ],
    }));

    findings
}

fn has_obvious_coordination_signal(function: &ParsedFunction) -> bool {
    function.has_context_parameter
        || function.calls.iter().any(|call| {
            call.receiver.as_ref().is_some_and(|receiver| {
                COORDINATION_METHODS.contains(&call.name.as_str())
                    && matches!(
                        receiver.as_str(),
                        "wg" | "group" | "g" | "errGroup" | "errgroup"
                    )
            })
        })
}
