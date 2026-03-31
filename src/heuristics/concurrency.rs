use std::collections::BTreeSet;

use crate::analysis::{ImportSpec, ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::common::{import_alias_lookup, is_blocking_call};

const COORDINATION_METHODS: &[&str] = &["Add", "Done", "Wait", "Go"];

pub(super) fn shutdown_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.unmanaged_goroutines
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
                if function.has_context_parameter {
                    "the parent function already accepts context.Context, so the goroutine likely skipped an available shutdown signal"
                        .to_string()
                } else {
                    "no parent context parameter was available for the goroutine to observe".to_string()
                },
            ],
        })
        .collect()
}

pub(super) fn mutex_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    imports: &[ImportSpec],
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = go
        .mutex_loops
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

        if !active_locks.is_empty() && is_blocking_call(&call, &import_aliases) {
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

pub(super) fn coordination_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    if go.goroutines.is_empty() || has_coordination(function) {
        return Vec::new();
    }

    let mut findings = go
        .goroutines
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

    findings.extend(go.loop_goroutines.iter().map(|line| Finding {
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

pub(super) fn deeper_goroutine_lifetime_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();

    if go.context_factory_calls.is_empty()
        || (go.loop_goroutines.is_empty() && go.unmanaged_goroutines.is_empty())
    {
        return Vec::new();
    }

    let loop_goroutines = go.loop_goroutines.iter().copied().collect::<BTreeSet<_>>();
    let unmanaged_goroutines = go
        .unmanaged_goroutines
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let candidate_lines = loop_goroutines
        .union(&unmanaged_goroutines)
        .copied()
        .collect::<Vec<_>>();

    let mut findings = Vec::new();

    for goroutine_line in candidate_lines {
        let Some(factory_call) = go
            .context_factory_calls
            .iter()
            .filter(|factory_call| factory_call.line < goroutine_line)
            .max_by_key(|factory_call| factory_call.line)
        else {
            continue;
        };

        let earliest_cancel = function
            .calls
            .iter()
            .filter(|call| {
                call.receiver.is_none()
                    && call.name == factory_call.cancel_name
                    && call.line > factory_call.line
            })
            .map(|call| call.line)
            .min();

        if earliest_cancel.is_some_and(|cancel_line| goroutine_line >= cancel_line) {
            continue;
        }

        let severity = if unmanaged_goroutines.contains(&goroutine_line) {
            Severity::Warning
        } else {
            Severity::Info
        };
        let goroutine_shape = if unmanaged_goroutines.contains(&goroutine_line) {
            "goroutine literal loops without an obvious shutdown path"
        } else {
            "goroutine launch occurs inside a loop"
        };
        let cancel_evidence = match earliest_cancel {
            Some(cancel_line) => format!(
                "earliest {}() observed at line {} after the goroutine launch",
                factory_call.cancel_name, cancel_line
            ),
            None => format!(
                "no local {}() call was observed after the derived context was created",
                factory_call.cancel_name
            ),
        };

        findings.push(Finding {
            rule_id: "goroutine_derived_context_unmanaged".to_string(),
            severity,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: goroutine_line,
            end_line: goroutine_line,
            message: format!(
                "function {} launches a likely long-lived goroutine after deriving context but before {}()",
                function.fingerprint.name, factory_call.cancel_name
            ),
            evidence: vec![
                format!(
                    "context.{} assigns cancel function {} at line {}",
                    factory_call.factory_name, factory_call.cancel_name, factory_call.line
                ),
                format!("goroutine launch observed at line {goroutine_line}"),
                goroutine_shape.to_string(),
                cancel_evidence,
            ],
        });
    }

    findings
}

fn has_coordination(function: &ParsedFunction) -> bool {
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
