mod api_design;
mod async_patterns;
pub(crate) mod bad_practices;
pub(crate) mod boundary;
mod domain_modeling;
mod evaluate;
pub(crate) mod module_surface;
mod performance;
mod runtime_boundary;
pub(crate) mod runtime_ownership;
pub(crate) mod security_footguns;
mod unsafe_soundness;

pub(crate) const BOUNDARY_BINDING_LOCATION: &str = boundary::BINDING_LOCATION;
pub(crate) const EVALUATE_BINDING_LOCATION: &str = evaluate::BINDING_LOCATION;
pub(crate) const API_DESIGN_BINDING_LOCATION: &str = api_design::BINDING_LOCATION;
pub(crate) const ASYNC_PATTERNS_BINDING_LOCATION: &str = async_patterns::BINDING_LOCATION;
pub(crate) const DOMAIN_MODELING_BINDING_LOCATION: &str = domain_modeling::BINDING_LOCATION;
pub(crate) const MODULE_SURFACE_BINDING_LOCATION: &str = module_surface::BINDING_LOCATION;
pub(crate) const PERFORMANCE_BINDING_LOCATION: &str = performance::BINDING_LOCATION;
pub(crate) const RUNTIME_OWNERSHIP_BINDING_LOCATION: &str = runtime_ownership::BINDING_LOCATION;
pub(crate) const SECURITY_FOOTGUNS_BINDING_LOCATION: &str = security_footguns::BINDING_LOCATION;
pub(crate) const RUNTIME_BOUNDARY_BINDING_LOCATION: &str = runtime_boundary::BINDING_LOCATION;
pub(crate) const UNSAFE_SOUNDNESS_BINDING_LOCATION: &str = unsafe_soundness::BINDING_LOCATION;
pub(crate) const BAD_PRACTICES_BINDING_LOCATION: &str = bad_practices::BINDING_LOCATION;

use crate::analysis::{FieldSummary, ParsedFile, ParsedFunction, StructSummary};
use crate::model::{Finding, Severity};

pub(crate) use api_design::{api_design_file_findings, api_design_function_findings};
pub(crate) use async_patterns::{async_file_findings, async_function_findings};
pub(crate) use boundary::{boundary_file_findings, boundary_function_findings};
pub(crate) use domain_modeling::domain_findings;
#[cfg(test)]
pub(crate) use evaluate::{alias_lookup, call_matches_import, import_matches_item};
pub(crate) use evaluate::{
    evaluate_rust_file_hygiene_findings, rust_api_design_file_findings,
    rust_api_design_function_findings, rust_async_file_findings, rust_async_function_findings,
    rust_bad_practices_file_findings, rust_bad_practices_function_findings,
    rust_bad_practices_indexed_repo_findings, rust_boundary_file_findings,
    rust_boundary_function_findings, rust_domain_file_findings, rust_import_resolution_findings,
    rust_local_call_findings, rust_module_surface_file_findings, rust_performance_file_findings,
    rust_performance_function_findings, rust_runtime_file_findings, rust_runtime_function_findings,
    rust_runtime_ownership_function_findings, rust_security_file_findings,
    rust_security_function_findings, rust_unsafe_soundness_findings,
};
pub(crate) use module_surface::module_surface_file_findings;
pub(crate) use performance::{performance_file_findings, performance_function_findings};
pub(crate) use runtime_boundary::{runtime_file_findings, runtime_function_findings};
pub(crate) use runtime_ownership::runtime_ownership_function_findings;
pub(crate) use security_footguns::{
    security_footguns_file_findings, security_footguns_function_findings,
};
pub(crate) use unsafe_soundness::unsafe_soundness_findings;

const RUST_GUIDE_REFERENCE: &str =
    "see guides/rust/heuristics-and-findings.md for remediation examples";

pub(crate) fn is_scanner_infra_file(file: &ParsedFile) -> bool {
    let path = file.path.to_string_lossy().to_ascii_lowercase();
    path.contains("/src/analysis/")
        || path.contains("/src/heuristics/")
        || path.contains("/src/rules/")
        || path.contains("/src/index/")
        || path.contains("/src/model/")
        || path.contains("/src/scan/")
}

pub(crate) fn file_attributes(file: &ParsedFile) -> &[crate::analysis::RustAttributeSummary] {
    file.rust_attributes()
}

pub(crate) fn is_test_like(file: &ParsedFile, function: Option<&ParsedFunction>) -> bool {
    let path = file.path.to_string_lossy().to_ascii_lowercase();
    function.is_some_and(|function| function.is_test_function)
        || file.is_test_file
        || path.ends_with("/tests.rs")
        || path.ends_with("/test_support.rs")
        || path.contains("/test_support/")
}

pub(crate) fn is_main_like_file(file: &ParsedFile) -> bool {
    let path = file.path.to_string_lossy().to_ascii_lowercase();
    path.ends_with("/main.rs") || path.ends_with("main.rs") || path.contains("/bin/")
}

pub(crate) fn contains_any(text: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| text.contains(marker))
}

pub(crate) fn first_line_with_any(body: &str, base_line: usize, markers: &[&str]) -> Option<usize> {
    body.lines()
        .enumerate()
        .find(|(_, line)| contains_any(line, markers))
        .map(|(offset, _)| base_line + offset)
}

pub(crate) fn has_secret_like_text(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    [
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
    ]
    .iter()
    .any(|token| normalized.contains(token))
}

pub(crate) fn has_numeric_narrowing_cast(line: &str) -> bool {
    [
        " as u8",
        " as u16",
        " as u32",
        " as u64",
        " as usize",
        " as i8",
        " as i16",
        " as i32",
        " as i64",
        " as isize",
    ]
    .iter()
    .any(|marker| line.contains(marker))
}

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
