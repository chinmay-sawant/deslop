use std::collections::BTreeMap;

use crate::analysis::ParsedFile;
use crate::analysis::ParsedFunction;
use crate::model::{Finding, Severity};

use super::{
    file_finding, first_await_after, function_finding, has_cancellation_pattern, is_std_mutex,
};

pub(crate) fn async_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let rust = function.rust_evidence();
    let mut findings = Vec::new();

    if (rust.is_async || !rust.await_points.is_empty()) && is_std_mutex(file, function) {
        for lock in rust.lock_calls {
            if let Some(await_line) = first_await_after(function, lock.line) {
                findings.push(function_finding(
                    file,
                    function,
                    "rust_async_std_mutex_await",
                    Severity::Error,
                    lock.line,
                    format!(
                        "function {} appears to hold std::sync::Mutex across .await",
                        function.fingerprint.name
                    ),
                    vec![
                        format!("lock line: {}", lock.line),
                        format!("await line: {await_line}"),
                    ],
                ));
            }
        }
    }

    for permit in rust.permit_acquires {
        if let Some(await_line) = first_await_after(function, permit.line) {
            findings.push(function_finding(
                file,
                function,
                "rust_async_hold_permit_across_await",
                Severity::Warning,
                permit.line,
                format!(
                    "function {} may hold a permit or pooled resource across .await",
                    function.fingerprint.name
                ),
                vec![
                    format!("permit/resource acquisition line: {}", permit.line),
                    format!("await line: {await_line}"),
                ],
            ));
        }
    }

    if !rust.spawn_calls.is_empty()
        && !rust.await_points.is_empty()
        && !has_cancellation_pattern(function)
    {
        findings.push(function_finding(
            file,
            function,
            "rust_async_spawn_cancel_at_await",
            Severity::Warning,
            rust.spawn_calls[0].line,
            format!(
                "function {} spawns async work without an obvious cancellation path",
                function.fingerprint.name
            ),
            vec!["no CancellationToken or select!-based shutdown branch detected".to_string()],
        ));
    }

    if !rust.select_macro_lines.is_empty() && function.body_text.contains("loop") {
        if !function.body_text.contains("pin_mut!") && !function.body_text.contains(".fuse()") {
            findings.push(function_finding(
                file,
                function,
                "rust_async_missing_fuse_pin",
                Severity::Warning,
                rust.select_macro_lines[0],
                format!(
                    "function {} reuses select! without fuse/pin markers",
                    function.fingerprint.name
                ),
                vec![
                    format!("macro calls observed={}", rust.macro_calls.len()),
                    "consider pinning and fusing reused futures before select! loops".to_string(),
                ],
            ));
        }

        if function.body_text.contains("select!") && function.body_text.contains("(") {
            findings.push(function_finding(
                file,
                function,
                "rust_async_recreate_future_in_select",
                Severity::Info,
                rust.select_macro_lines[0],
                format!(
                    "function {} may recreate futures inside a select! loop",
                    function.fingerprint.name
                ),
                vec![
                    format!(
                        "tracked future-like bindings={}",
                        rust.futures_created.len()
                    ),
                    "move long-lived futures outside the loop when they are polled repeatedly"
                        .to_string(),
                ],
            ));
        }
    }

    if rust.is_async
        && rust.await_points.is_empty()
        && (function.body_text.contains("loop {") || function.body_text.contains("while "))
        && !rust.blocking_calls.is_empty()
    {
        findings.push(function_finding(
            file,
            function,
            "rust_async_monopolize_executor",
            Severity::Warning,
            rust.blocking_calls[0].line,
            format!("async function {} may monopolize the executor", function.fingerprint.name),
            vec!["long-running loops with blocking work and no await can starve cooperative scheduling".to_string()],
        ));
    }

    if rust.drop_impl && !rust.blocking_calls.is_empty() {
        findings.push(function_finding(
            file,
            function,
            "rust_async_blocking_drop",
            Severity::Warning,
            rust.blocking_calls[0].line,
            format!(
                "Drop implementation {} does blocking work that may surface in async code",
                function.fingerprint.name
            ),
            vec![
                "prefer explicit async shutdown paths instead of blocking cleanup in Drop"
                    .to_string(),
            ],
        ));
    }

    if !rust.await_points.is_empty() {
        let lines = function.body_text.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            if line.contains(".await")
                && index > 0
                && index + 1 < lines.len()
                && (lines[index - 1].contains("self.") || lines[index - 1].contains("state."))
                && (lines[index + 1].contains("self.") || lines[index + 1].contains("state."))
            {
                findings.push(function_finding(
                    file,
                    function,
                    "rust_async_invariant_broken_at_await",
                    Severity::Warning,
                    function.fingerprint.start_line + index,
                    format!("function {} mutates related state around an await boundary", function.fingerprint.name),
                    vec!["consider keeping invariant-maintaining state updates contiguous or protected".to_string()],
                ));
                break;
            }
        }
    }

    findings
}

pub(crate) fn async_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut order_edges = BTreeMap::<(String, String), usize>::new();

    for function in &file.functions {
        let rust = function.rust_evidence();
        let receivers = rust
            .lock_calls
            .iter()
            .filter(|call| is_lock_order_call(call))
            .filter_map(|call| call.receiver.clone())
            .collect::<Vec<_>>();

        for pair in receivers.windows(2) {
            let edge = (pair[0].clone(), pair[1].clone());
            order_edges
                .entry(edge)
                .or_insert(function.fingerprint.start_line);
        }
    }

    for ((left, right), line) in &order_edges {
        if let Some(reverse_line) = order_edges.get(&(right.clone(), left.clone())) {
            findings.push(file_finding(
                file,
                "rust_async_lock_order_cycle",
                Severity::Error,
                *line,
                format!(
                    "file {} contains conflicting lock acquisition order",
                    file.path.display()
                ),
                vec![
                    format!("observed lock order {left} -> {right} at line {line}"),
                    format!("observed reverse order {right} -> {left} at line {reverse_line}"),
                ],
            ));
        }
    }

    findings
}

fn is_lock_order_call(call: &crate::analysis::RuntimeCall) -> bool {
    if matches!(call.name.as_str(), "lock" | "lock_owned") {
        return true;
    }

    let receiver = call
        .receiver
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(call.name.as_str(), "read" | "write")
        && (receiver.contains("lock") || receiver.contains("mutex") || receiver.contains("rwlock"))
}
