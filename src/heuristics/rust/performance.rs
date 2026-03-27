use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::{file_finding, first_await_after, function_finding, is_tokio_mutex};

pub(crate) fn performance_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    if (function.is_async || !function.await_points.is_empty()) && !function.blocking_calls.is_empty() {
        for call in &function.blocking_calls {
            findings.push(function_finding(
                file,
                function,
                "rust_blocking_io_in_async",
                Severity::Warning,
                call.line,
                format!("function {} performs blocking I/O or blocking work in async code", function.fingerprint.name),
                vec![
                    format!("blocking call: {}", call.name),
                    "prefer async filesystem/runtime APIs or move the work to spawn_blocking".to_string(),
                ],
            ));
        }
    }

    for line in &function.write_loops {
        findings.push(function_finding(
            file,
            function,
            "rust_unbuffered_file_writes",
            Severity::Info,
            *line,
            format!("function {} writes to a file-like sink inside a loop", function.fingerprint.name),
            vec![
                "write! / writeln! / File::write appears in a loop".to_string(),
                "buffering or batching writes usually reduces syscall overhead".to_string(),
            ],
        ));
    }

    for line in &function.line_iteration_loops {
        findings.push(function_finding(
            file,
            function,
            "rust_lines_allocate_per_line",
            Severity::Info,
            *line,
            format!("function {} uses .lines() inside a loop", function.fingerprint.name),
            vec![
                "line iteration can allocate per item in hot paths".to_string(),
                "consider reusing a buffer with read_line when this is performance-sensitive".to_string(),
            ],
        ));
    }

    for line in &function.default_hasher_lines {
        findings.push(function_finding(
            file,
            function,
            "rust_hashmap_default_hasher",
            Severity::Info,
            *line,
            format!("function {} creates a HashMap with the default hasher in a likely hot path", function.fingerprint.name),
            vec![
                "HashMap::new/default appears in a loop or other repeated path".to_string(),
                "for profiled hot paths consider a faster hasher or a different map type".to_string(),
            ],
        ));
    }

    for (offset, line) in function.body_text.lines().enumerate() {
        if line.contains("HashMap::new()") || line.contains("HashMap::default()") {
            findings.push(function_finding(
                file,
                function,
                "rust_hashmap_default_hasher",
                Severity::Info,
                function.fingerprint.start_line + offset,
                format!("function {} creates a HashMap with the default hasher in a likely hot path", function.fingerprint.name),
                vec![
                    "HashMap::new/default appears in textual function body analysis".to_string(),
                    "for profiled hot paths consider a faster hasher or a different map type".to_string(),
                ],
            ));
        }
    }

    for lock in &function.lock_calls {
        if let Some(await_line) = first_await_after(function, lock.line) {
            findings.push(function_finding(
                file,
                function,
                "rust_lock_across_await",
                Severity::Warning,
                lock.line,
                format!("function {} appears to hold a lock across .await", function.fingerprint.name),
                vec![
                    format!("lock acquisition line: {}", lock.line),
                    format!("later await line: {await_line}"),
                ],
            ));
        }
    }

    if is_tokio_mutex(file, function) && function.await_points.is_empty() && !function.lock_calls.is_empty() {
        findings.push(function_finding(
            file,
            function,
            "rust_tokio_mutex_unnecessary",
            Severity::Info,
            function.lock_calls[0].line,
            format!("function {} uses tokio::sync::Mutex without any await in the critical path", function.fingerprint.name),
            vec![
                "the function locks a Tokio mutex but does not await afterward".to_string(),
                "a std::sync::Mutex may be simpler if the guarded section is fully synchronous".to_string(),
            ],
        ));
    }

    if function.drop_impl && !function.blocking_calls.is_empty() {
        findings.push(function_finding(
            file,
            function,
            "rust_blocking_drop",
            Severity::Warning,
            function.blocking_calls[0].line,
            format!("Drop implementation {} performs blocking work", function.fingerprint.name),
            vec!["blocking work inside Drop can stall executors or hide shutdown latency".to_string()],
        ));
    }

    for line in &function.boxed_container_lines {
        findings.push(function_finding(
            file,
            function,
            "rust_pointer_chasing_vec_box",
            Severity::Info,
            *line,
            format!("function {} uses Vec<Box<T>> style storage", function.fingerprint.name),
            vec!["pointer-heavy container layouts can harm cache locality".to_string()],
        ));
    }

    for (offset, line) in function.body_text.lines().enumerate() {
        let absolute = function.fingerprint.start_line + offset;
        if line.contains(".join(\"/") || line.contains(".join('/") {
            findings.push(function_finding(
                file,
                function,
                "rust_path_join_absolute",
                Severity::Warning,
                absolute,
                format!("function {} joins an absolute path segment", function.fingerprint.name),
                vec!["Path::join on an absolute segment discards the existing base path".to_string()],
            ));
        }
        if line.contains("from_utf8(") && function.fingerprint.complexity_score > 2 {
            findings.push(function_finding(
                file,
                function,
                "rust_utf8_validate_hot_path",
                Severity::Info,
                absolute,
                format!("function {} validates UTF-8 in a likely hot path", function.fingerprint.name),
                vec!["UTF-8 validation is correct by default, but worth profiling in hot loops".to_string()],
            ));
        }
        if function.is_async
            && (line.contains("vec![") || line.contains("Vec::with_capacity") || line.contains("String::with_capacity"))
            && !function.await_points.is_empty()
        {
            findings.push(function_finding(
                file,
                function,
                "rust_large_future_stack",
                Severity::Info,
                absolute,
                format!("function {} may capture a large allocation in an async future", function.fingerprint.name),
                vec!["large locals captured across await points can bloat future size".to_string()],
            ));
        }
    }

    if function.body_text.contains("for ")
        && function.body_text.matches('.').count() > 8
        && file.structs.iter().any(|summary| summary.fields.len() >= 3)
    {
        findings.push(function_finding(
            file,
            function,
            "rust_aos_hot_path",
            Severity::Info,
            function.fingerprint.start_line,
            format!("function {} repeatedly dereferences struct fields inside a loop", function.fingerprint.name),
            vec!["consider validating with profiling if an array-of-structs layout becomes a hot path".to_string()],
        ));
    }

    findings
}

pub(crate) fn performance_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for summary in &file.structs {
        for field in &summary.fields {
            if field.type_text.contains("Vec<Box<") {
                findings.push(file_finding(
                    file,
                    "rust_pointer_chasing_vec_box",
                    Severity::Info,
                    field.line,
                    format!("struct {} stores Vec<Box<T>> style data", summary.name),
                    vec![format!("field {} uses type {}", field.name, field.type_text)],
                ));
            }
        }
    }

    findings
}