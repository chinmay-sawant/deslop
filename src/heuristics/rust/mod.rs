mod api_design;
mod async_patterns;
mod domain_modeling;
mod performance;
mod runtime_boundary;
mod unsafe_soundness;

pub(crate) const API_DESIGN_BINDING_LOCATION: &str = api_design::BINDING_LOCATION;
pub(crate) const ASYNC_PATTERNS_BINDING_LOCATION: &str = async_patterns::BINDING_LOCATION;
pub(crate) const DOMAIN_MODELING_BINDING_LOCATION: &str = domain_modeling::BINDING_LOCATION;
pub(crate) const PERFORMANCE_BINDING_LOCATION: &str = performance::BINDING_LOCATION;
pub(crate) const RUNTIME_BOUNDARY_BINDING_LOCATION: &str = runtime_boundary::BINDING_LOCATION;
pub(crate) const UNSAFE_SOUNDNESS_BINDING_LOCATION: &str = unsafe_soundness::BINDING_LOCATION;

use crate::analysis::{FieldSummary, ParsedFile, ParsedFunction, StructSummary};
use crate::model::{Finding, Severity};

pub(crate) use api_design::{api_design_file_findings, api_design_function_findings};
pub(crate) use async_patterns::{async_file_findings, async_function_findings};
pub(crate) use domain_modeling::domain_findings;
pub(crate) use performance::{performance_file_findings, performance_function_findings};
pub(crate) use runtime_boundary::{runtime_file_findings, runtime_function_findings};
pub(crate) use unsafe_soundness::unsafe_soundness_findings;

const RUST_GUIDE_REFERENCE: &str =
    "see guides/rust/heuristics-and-findings.md for remediation examples";

fn function_finding(
    file: &ParsedFile,
    function: &ParsedFunction,
    rule_id: &str,
    severity: Severity,
    line: usize,
    message: String,
    evidence: Vec<String>,
) -> Finding {
    let mut evidence = evidence;
    evidence.push(RUST_GUIDE_REFERENCE.to_string());

    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message,
        evidence,
    }
}

fn file_finding(
    file: &ParsedFile,
    rule_id: &str,
    severity: Severity,
    line: usize,
    message: String,
    evidence: Vec<String>,
) -> Finding {
    let mut evidence = evidence;
    evidence.push(RUST_GUIDE_REFERENCE.to_string());

    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: None,
        start_line: line,
        end_line: line,
        message,
        evidence,
    }
}

fn struct_severity(summary: &StructSummary) -> Severity {
    if summary.visibility_pub || summary.fields.iter().any(|field| field.is_pub) {
        Severity::Warning
    } else {
        Severity::Info
    }
}

fn first_await_after(function: &ParsedFunction, line: usize) -> Option<usize> {
    function
        .rust_evidence()
        .await_points
        .iter()
        .copied()
        .find(|await_line| *await_line > line)
}

fn secret_like(name: &str) -> bool {
    matches_token(
        name,
        &[
            "password",
            "secret",
            "token",
            "api_key",
            "apikey",
            "access_token",
            "private_key",
            "cert",
            "certificate",
            "auth",
            "key",
        ],
    )
}

fn credential_like(name: &str) -> bool {
    matches_token(
        name,
        &[
            "cert",
            "certificate",
            "key",
            "token",
            "auth",
            "password",
            "secret",
        ],
    )
}

fn enabled_like(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    normalized == "enabled"
        || normalized.ends_with("_enabled")
        || normalized.starts_with("use_")
        || normalized.starts_with("has_")
        || matches_token(name, &["ssl", "tls", "enabled"])
}

fn business_value_like(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    if normalized.ends_with("_ms") || normalized.starts_with("uses_") {
        return false;
    }

    matches_token(
        name,
        &[
            "amount", "price", "cost", "total", "balance", "rate", "money", "username", "email",
            "percent",
        ],
    )
}

fn sensitive_default_like(name: &str) -> bool {
    matches_token(
        name,
        &[
            "port", "token", "password", "api_key", "secret", "key", "enabled",
        ],
    )
}

fn matches_token(name: &str, tokens: &[&str]) -> bool {
    let normalized = name.to_ascii_lowercase();
    tokens
        .iter()
        .any(|token| normalized == *token || normalized.contains(token))
}

fn is_std_mutex(file: &ParsedFile, function: &ParsedFunction) -> bool {
    file.imports
        .iter()
        .any(|import| import.path.ends_with("std::sync::Mutex"))
        || function.body_text.contains("std::sync::Mutex")
}

fn is_tokio_mutex(file: &ParsedFile, function: &ParsedFunction) -> bool {
    file.imports
        .iter()
        .any(|import| import.path.ends_with("tokio::sync::Mutex"))
        || function.body_text.contains("tokio::sync::Mutex")
}

fn has_cancellation_pattern(function: &ParsedFunction) -> bool {
    let rust = function.rust_evidence();

    function.body_text.contains("CancellationToken")
        || function.body_text.contains("cancelled()")
        || function.body_text.contains("shutdown")
        || rust.select_macro_lines.len() > 1
}

fn field_type_mentions(field: &FieldSummary, text: &str) -> bool {
    field
        .type_text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .contains(text)
}
