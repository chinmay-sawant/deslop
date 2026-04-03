use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::{file_finding, first_await_after, function_finding, is_tokio_mutex};

const HASHMAP_PREFIX: &str = "HashMap::";
const HASHMAP_NEW_SUFFIX: &str = "::new";
const HASHMAP_DEFAULT_SUFFIX: &str = "::default";

pub(crate) fn performance_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let rust = function.rust_evidence();
    let mut findings = Vec::new();

    if (rust.is_async || !rust.await_points.is_empty()) && !rust.blocking_calls.is_empty() {
        for call in rust.blocking_calls {
            findings.push(function_finding(
                file,
                function,
                "rust_blocking_io_in_async",
                Severity::Warning,
                call.line,
                format!(
                    "function {} performs blocking I/O or blocking work in async code",
                    function.fingerprint.name
                ),
                vec![
                    format!("blocking call: {}", call.name),
                    "prefer async filesystem/runtime APIs or move the work to spawn_blocking"
                        .to_string(),
                ],
            ));
        }
    }

    for line in rust.write_loops {
        findings.push(function_finding(
            file,
            function,
            "rust_unbuffered_file_writes",
            Severity::Info,
            *line,
            format!(
                "function {} writes to a file-like sink inside a loop",
                function.fingerprint.name
            ),
            vec![
                "write! / writeln! / File::write appears in a loop".to_string(),
                "buffering or batching writes usually reduces syscall overhead".to_string(),
            ],
        ));
    }

    for line in rust.line_iteration_loops {
        findings.push(function_finding(
            file,
            function,
            "rust_lines_allocate_per_line",
            Severity::Info,
            *line,
            format!(
                "function {} uses .lines() inside a loop",
                function.fingerprint.name
            ),
            vec![
                "line iteration can allocate per item in hot paths".to_string(),
                "consider reusing a buffer with read_line when this is performance-sensitive"
                    .to_string(),
            ],
        ));
    }

    for line in rust.default_hasher_lines {
        findings.push(function_finding(
            file,
            function,
            "rust_hashmap_default_hasher",
            Severity::Info,
            *line,
            format!(
                "function {} creates a HashMap with the default hasher in a likely hot path",
                function.fingerprint.name
            ),
            vec![
                "a default hash map constructor appears in a loop or other repeated path"
                    .to_string(),
                "for profiled hot paths consider a faster hasher or a different map type"
                    .to_string(),
            ],
        ));
    }

    for (offset, line) in function.body_text.split('\n').enumerate() {
        if contains_default_hashmap_ctor(line) {
            findings.push(function_finding(
                file,
                function,
                "rust_hashmap_default_hasher",
                Severity::Info,
                function.fingerprint.start_line + offset,
                format!(
                    "function {} creates a HashMap with the default hasher in a likely hot path",
                    function.fingerprint.name
                ),
                vec![
                    "a default hash map constructor appears in textual function body analysis"
                        .to_string(),
                    "for profiled hot paths consider a faster hasher or a different map type"
                        .to_string(),
                ],
            ));
        }
    }

    for lock in rust.lock_calls {
        if let Some(await_line) = first_await_after(function, lock.line) {
            findings.push(function_finding(
                file,
                function,
                "rust_lock_across_await",
                Severity::Warning,
                lock.line,
                format!(
                    "function {} appears to hold a lock across .await",
                    function.fingerprint.name
                ),
                vec![
                    format!("lock acquisition line: {}", lock.line),
                    format!("later await line: {await_line}"),
                ],
            ));
        }
    }

    if is_tokio_mutex(file, function) && rust.await_points.is_empty() && !rust.lock_calls.is_empty()
    {
        findings.push(function_finding(
            file,
            function,
            "rust_tokio_mutex_unnecessary",
            Severity::Info,
            rust.lock_calls[0].line,
            format!(
                "function {} uses tokio::sync::Mutex without any await in the critical path",
                function.fingerprint.name
            ),
            vec![
                "the function locks a Tokio mutex but does not await afterward".to_string(),
                "a std::sync::Mutex may be simpler if the guarded section is fully synchronous"
                    .to_string(),
            ],
        ));
    }

    if rust.drop_impl && !rust.blocking_calls.is_empty() {
        findings.push(function_finding(
            file,
            function,
            "rust_blocking_drop",
            Severity::Warning,
            rust.blocking_calls[0].line,
            format!(
                "Drop implementation {} performs blocking work",
                function.fingerprint.name
            ),
            vec![
                "blocking work inside Drop can stall executors or hide shutdown latency"
                    .to_string(),
            ],
        ));
    }

    for line in rust.boxed_container_lines {
        findings.push(function_finding(
            file,
            function,
            "rust_pointer_chasing_vec_box",
            Severity::Info,
            *line,
            format!(
                "function {} uses boxed vector-style storage",
                function.fingerprint.name
            ),
            vec!["pointer-heavy container layouts can harm cache locality".to_string()],
        ));
    }

    for (offset, line) in function.body_text.split('\n').enumerate() {
        let absolute = function.fingerprint.start_line + offset;
        if contains_absolute_join(line) {
            findings.push(function_finding(
                file,
                function,
                "rust_path_join_absolute",
                Severity::Warning,
                absolute,
                format!(
                    "function {} joins an absolute path segment",
                    function.fingerprint.name
                ),
                vec![
                    "Path::join on an absolute segment discards the existing base path".to_string(),
                ],
            ));
        }
        if contains_utf8_validation(line) && function.fingerprint.complexity_score > 2 {
            findings.push(function_finding(
                file,
                function,
                "rust_utf8_validate_hot_path",
                Severity::Info,
                absolute,
                format!(
                    "function {} validates UTF-8 in a likely hot path",
                    function.fingerprint.name
                ),
                vec![
                    "UTF-8 validation is correct by default, but worth profiling in hot loops"
                        .to_string(),
                ],
            ));
        }
        if rust.is_async
            && (line.contains("vec![")
                || line.contains("Vec::with_capacity")
                || line.contains("String::with_capacity"))
            && !rust.await_points.is_empty()
        {
            findings.push(function_finding(
                file,
                function,
                "rust_large_future_stack",
                Severity::Info,
                absolute,
                format!(
                    "function {} may capture a large allocation in an async future",
                    function.fingerprint.name
                ),
                vec!["large locals captured across await points can bloat future size".to_string()],
            ));
        }
    }

    if looks_like_aos_hot_path(file, function) {
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

    for summary in file.structs() {
        for field in &summary.fields {
            if contains_boxed_vec_type(field.type_text.as_str()) {
                findings.push(file_finding(
                    file,
                    "rust_pointer_chasing_vec_box",
                    Severity::Info,
                    field.line,
                    format!("struct {} stores boxed vector-style data", summary.name),
                    vec![format!(
                        "field {} uses type {}",
                        field.name, field.type_text
                    )],
                ));
            }
        }
    }

    findings
}

fn contains_default_hashmap_ctor(line: &str) -> bool {
    let normalized = strip_double_quoted_strings(line).replace(char::is_whitespace, "");

    normalized.contains("HashMap::new(")
        || normalized.contains("HashMap::default(")
        || (normalized.contains(HASHMAP_PREFIX)
            && (normalized.contains(&format!("<{HASHMAP_NEW_SUFFIX}("))
                || normalized.contains(&format!("<{HASHMAP_DEFAULT_SUFFIX}("))))
}

fn strip_double_quoted_strings(line: &str) -> String {
    let mut stripped = String::with_capacity(line.len());
    let mut in_string = false;
    let mut escaped = false;

    for character in line.chars() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }

            match character {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }

            continue;
        }

        if character == '"' {
            in_string = true;
            continue;
        }

        stripped.push(character);
    }

    stripped
}

fn contains_absolute_join(line: &str) -> bool {
    has_absolute_join_after(line, ".join(\"") || has_absolute_join_after(line, ".join('")
}

fn contains_utf8_validation(line: &str) -> bool {
    line.contains("from_utf8") && line.contains('(')
}

fn contains_boxed_vec_type(text: &str) -> bool {
    let Some(vec_start) = text.find("Vec<") else {
        return false;
    };
    text[vec_start + 4..].contains("Box<")
}

fn has_absolute_join_after(line: &str, marker: &str) -> bool {
    let Some(start) = line.find(marker) else {
        return false;
    };
    line[start + marker.len()..].starts_with('/')
}

fn looks_like_aos_hot_path(file: &ParsedFile, function: &ParsedFunction) -> bool {
    function.body_text.contains("for ")
        && has_numeric_update(function.body_text.as_str())
        && repeated_receiver_field_accesses(function.body_text.as_str()) >= 4
        && file.structs().iter().any(|summary| summary.fields.len() >= 3)
}

fn has_numeric_update(body: &str) -> bool {
    ["+=", "-=", "*=", "/="]
        .iter()
        .any(|marker| body.contains(marker))
}

fn repeated_receiver_field_accesses(body: &str) -> usize {
    let mut counts = std::collections::BTreeMap::<&str, usize>::new();

    for line in body.split('\n') {
        let bytes = line.as_bytes();
        let mut index = 0usize;

        while index < bytes.len() {
            if !(bytes[index].is_ascii_alphabetic() || bytes[index] == b'_') {
                index += 1;
                continue;
            }

            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }

            if index < bytes.len() && bytes[index] == b'.' {
                let receiver = &line[start..index];
                *counts.entry(receiver).or_default() += 1;
            }
        }
    }

    counts.into_values().max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{contains_default_hashmap_ctor, strip_double_quoted_strings};

    #[test]
    fn strips_double_quoted_strings_before_text_matching() {
        assert_eq!(
            strip_double_quoted_strings("let msg = \"HashMap::new(\";"),
            "let msg = ;"
        );
    }

    #[test]
    fn ignores_hashmap_markers_inside_strings() {
        assert!(!contains_default_hashmap_ctor(
            "let msg = \"HashMap::new(\";"
        ));
        assert!(contains_default_hashmap_ctor("let map = HashMap::new();"));
    }
}
