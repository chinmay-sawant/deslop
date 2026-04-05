use std::path::Path;

use toml::Value;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::io::{DEFAULT_MAX_BYTES, read_to_string_limited};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::{function_finding, is_scanner_infra_file};

#[derive(Debug, Clone)]
struct BodyLine {
    line: usize,
    text: String,
    in_loop: bool,
}

pub(crate) fn runtime_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if is_scanner_infra_file(file) {
        return Vec::new();
    }

    if function.is_test_function {
        return Vec::new();
    }

    let lines = body_lines(function);
    let body = function.body_text.as_str();
    let request_path = is_axum_handler(file, function)
        || is_actix_handler(file, function)
        || is_tonic_handler(file, function);
    let mut findings = Vec::new();

    if let Some(line) = first_line_with_any(
        body,
        function.fingerprint.start_line,
        &[
            "Runtime::new(",
            "Builder::new_multi_thread(",
            "Builder::new_current_thread(",
        ],
    ) && !main_like(function)
    {
        findings.push(function_finding(
            file,
            function,
            "rust_tokio_runtime_built_per_call",
            Severity::Warning,
            line,
            format!(
                "function {} builds a Tokio runtime in the call path",
                function.fingerprint.name
            ),
            vec![
                "Tokio runtimes are usually process-level infrastructure instead of per-call state"
                    .to_string(),
            ],
        ));
    }

    if request_path {
        if let Some(line) = first_line_with_any(
            body,
            function.fingerprint.start_line,
            &[
                "std::env::var(",
                "env::var(",
                "std::env::var_os(",
                "env::var_os(",
            ],
        ) {
            findings.push(function_finding(
                file,
                function,
                "rust_env_var_read_in_request_path",
                Severity::Warning,
                line,
                format!(
                    "function {} reads environment configuration on a request path",
                    function.fingerprint.name
                ),
                vec![
                    "request handlers usually read already-initialized config rather than hitting std::env per request"
                        .to_string(),
                ],
            ));
        }

        if has_import_fragment(file, "axum")
            && let Some(line) =
                first_line_with_any(body, function.fingerprint.start_line, &["Router::new("])
        {
            findings.push(function_finding(
                file,
                function,
                "rust_axum_router_built_in_handler",
                Severity::Warning,
                line,
                format!(
                    "function {} builds an Axum router inside a handler path",
                    function.fingerprint.name
                ),
                vec![
                    "routers are typically assembled at startup rather than rebuilt per request"
                        .to_string(),
                ],
            ));
        }

        if has_import_fragment(file, "tonic")
            && let Some(line) = first_line_with_any(
                body,
                function.fingerprint.start_line,
                &[
                    ".connect().await",
                    "Endpoint::from_static(",
                    "Endpoint::from_shared(",
                    "Channel::from_static(",
                    "Channel::from_shared(",
                ],
            )
        {
            findings.push(function_finding(
                file,
                function,
                "rust_tonic_channel_connect_per_request",
                Severity::Warning,
                line,
                format!(
                    "function {} establishes a tonic transport channel on a request path",
                    function.fingerprint.name
                ),
                vec![
                    "reuse a client/channel when possible instead of dialing a transport on each request"
                        .to_string(),
                ],
            ));
        }
    }

    if let Some(line) = lines.iter().find(|line| {
        line.in_loop && line.text.contains(".clone()") && looks_like_heavy_clone(line.text.as_str())
    }) {
        findings.push(function_finding(
            file,
            function,
            "rust_clone_heavy_state_in_loop",
            Severity::Info,
            line.line,
            format!(
                "function {} clones likely heavy state inside a loop",
                function.fingerprint.name
            ),
            vec![
                format!("clone expression={}", line.text.trim()),
                "consider borrowing, moving the clone outside the loop, or sharing cheaper state handles"
                    .to_string(),
            ],
        ));
    }

    findings
}

pub(crate) fn runtime_file_findings(file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
    let manifest_path = index.root().join("Cargo.toml");
    if !should_emit_workspace_manifest_finding(file, index.root()) || !manifest_path.exists() {
        return Vec::new();
    }

    let Ok(source) = read_to_string_limited(&manifest_path, DEFAULT_MAX_BYTES) else {
        return Vec::new();
    };
    let Ok(parsed) = source.parse::<Value>() else {
        return Vec::new();
    };
    let Some(workspace) = parsed.get("workspace").and_then(Value::as_table) else {
        return Vec::new();
    };
    let member_count = workspace
        .get("members")
        .and_then(Value::as_array)
        .map(|members| members.len())
        .unwrap_or(0);
    if member_count < 2 || workspace.get("resolver").is_some() {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "rust_workspace_missing_resolver".to_string(),
        severity: Severity::Info,
        path: manifest_path,
        function_name: None,
        start_line: 1,
        end_line: 1,
        message: "workspace Cargo.toml does not declare an explicit dependency resolver"
            .to_string(),
        evidence: vec![format!("workspace_members={member_count}")],
    }]
}

fn should_emit_workspace_manifest_finding(file: &ParsedFile, root: &Path) -> bool {
    let candidates = [
        root.join("src/lib.rs"),
        root.join("src/main.rs"),
        root.join("lib.rs"),
        root.join("main.rs"),
    ];
    candidates.iter().any(|candidate| candidate == &file.path)
}

fn body_lines(function: &ParsedFunction) -> Vec<BodyLine> {
    let mut brace_depth = 0usize;
    let mut loop_exit_depths = Vec::new();
    let mut lines = Vec::new();

    for (offset, raw_line) in function.body_text.lines().enumerate() {
        let absolute_line = function.body_start_line + offset;
        let stripped = raw_line.split("//").next().unwrap_or("").trim().to_string();
        let closing_braces = stripped
            .chars()
            .filter(|character| *character == '}')
            .count();
        for _ in 0..closing_braces {
            brace_depth = brace_depth.saturating_sub(1);
            while loop_exit_depths
                .last()
                .is_some_and(|exit_depth| *exit_depth > brace_depth)
            {
                loop_exit_depths.pop();
            }
        }

        let starts_loop = contains_keyword(&stripped, "for")
            || contains_keyword(&stripped, "while")
            || contains_keyword(&stripped, "loop");
        let in_loop = !loop_exit_depths.is_empty() || starts_loop;
        let opening_braces = stripped
            .chars()
            .filter(|character| *character == '{')
            .count();
        if starts_loop {
            loop_exit_depths.push(brace_depth + opening_braces.max(1));
        }
        brace_depth += opening_braces;

        lines.push(BodyLine {
            line: absolute_line,
            text: stripped,
            in_loop,
        });
    }

    lines
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
            start == 0 || (!bytes[start - 1].is_ascii_alphanumeric() && bytes[start - 1] != b'_');
        let right_index = start + keyword_bytes.len();
        let right_ok = right_index == bytes.len()
            || (!bytes[right_index].is_ascii_alphanumeric() && bytes[right_index] != b'_');
        if left_ok && right_ok {
            return true;
        }
    }

    false
}

fn first_line_with_any(body: &str, base_line: usize, markers: &[&str]) -> Option<usize> {
    body.lines()
        .enumerate()
        .find(|(_, line)| markers.iter().any(|marker| line.contains(marker)))
        .map(|(offset, _)| base_line + offset)
}

fn has_import_fragment(file: &ParsedFile, fragment: &str) -> bool {
    file.imports
        .iter()
        .any(|import| import.path.contains(fragment) || import.alias.contains(fragment))
}

fn is_axum_handler(file: &ParsedFile, function: &ParsedFunction) -> bool {
    has_import_fragment(file, "axum")
        && ["State<", "Json<", "Path<", "Query<", "Form<"]
            .iter()
            .any(|marker| function.signature_text.contains(marker))
}

fn is_actix_handler(file: &ParsedFile, function: &ParsedFunction) -> bool {
    has_import_fragment(file, "actix_web")
        && ["HttpRequest", "web::Json<", "web::Data<", "Responder"]
            .iter()
            .any(|marker| function.signature_text.contains(marker))
}

fn is_tonic_handler(file: &ParsedFile, function: &ParsedFunction) -> bool {
    has_import_fragment(file, "tonic")
        && ["Request<", "tonic::Request<", "Streaming<"]
            .iter()
            .any(|marker| function.signature_text.contains(marker))
}

fn main_like(function: &ParsedFunction) -> bool {
    matches!(function.fingerprint.name.as_str(), "main" | "build_runtime")
}

fn looks_like_heavy_clone(line: &str) -> bool {
    [
        "state.clone()",
        "config.clone()",
        "payload.clone()",
        "request.clone()",
        "response.clone()",
        "body.clone()",
        "bytes.clone()",
        "client.clone()",
        "channel.clone()",
    ]
    .iter()
    .any(|pattern| line.contains(pattern))
}
