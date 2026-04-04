use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::{
    contains_any, first_line_with_any, has_numeric_narrowing_cast, has_secret_like_text,
    is_main_like_file, is_test_like,
};

pub(crate) const RULE_DEFINITIONS: &[crate::rules::catalog::RuleDefinition] = &[
    crate::rules::catalog::RuleDefinition {
        id: "rust_internal_anyhow_result",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "boundary",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Internal library functions that return anyhow-style error surfaces instead of crate-local errors.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN1,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_unbounded_read_to_string",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "boundary",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Production code that reads an entire file into a string without a size bound.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN1,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_check_then_open_path",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "boundary",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Filesystem code that checks metadata or existence before opening a path.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN1,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_secret_equality_compare",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "boundary",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Direct equality or inequality comparisons on secret-like values.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN1,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_narrowing_numeric_cast",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "boundary",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Numeric narrowing casts that may silently truncate or change precision.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN1,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_manual_tempdir_lifecycle",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "boundary",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Manual temp-directory setup and cleanup that should usually use RAII helpers.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN1,
    },
];

pub(crate) fn file_findings(file: &ParsedFile) -> Vec<Finding> {
    let _ = file;
    Vec::new()
}

pub(crate) fn function_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if is_test_like(file, Some(function)) {
        return Vec::new();
    }

    let mut findings = Vec::new();
    findings.extend(internal_anyhow_findings(file, function));
    findings.extend(unbounded_read_findings(file, function));
    findings.extend(check_then_open_findings(file, function));
    findings.extend(secret_comparison_findings(file, function));
    findings.extend(narrowing_cast_findings(file, function));
    findings.extend(manual_tempdir_function_findings(file, function));
    findings
}

fn internal_anyhow_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let signature = function.signature_text.trim_start();
    if signature.starts_with("pub ") || is_main_like_file(file) {
        return Vec::new();
    }

    let Some(line) = first_line_with_any(
        &function.signature_text,
        function.fingerprint.start_line,
        &[
            "anyhow::Result",
            "anyhow::Error",
            "eyre::Result",
            "eyre::Report",
            "color_eyre::Result",
        ],
    ) else {
        return Vec::new();
    };

    vec![Finding {
        rule_id: "rust_internal_anyhow_result".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!(
            "function {} returns an anyhow-style error surface in internal code",
            function.fingerprint.name
        ),
        evidence: vec![
            function.signature_text.trim().to_string(),
            "prefer a crate-local error enum for internal library code".to_string(),
        ],
    }]
}

fn unbounded_read_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let Some(line) = first_line_with_any(
        &function.body_text,
        function.fingerprint.start_line,
        &[".read_to_string(", "read_to_string("],
    ) else {
        return Vec::new();
    };

    vec![Finding {
        rule_id: "rust_unbounded_read_to_string".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!(
            "function {} reads an entire file into memory without an obvious bound",
            function.fingerprint.name
        ),
        evidence: vec![
            "fs::read_to_string or a similar full-file read was detected".to_string(),
            "prefer bounded readers or streaming for production paths".to_string(),
        ],
    }]
}

fn check_then_open_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let Some(line) = first_line_with_any(
        &function.body_text,
        function.fingerprint.start_line,
        &["exists(", "metadata(", "symlink_metadata("],
    ) else {
        return Vec::new();
    };

    if !contains_any(
        &function.body_text,
        &["open(", "read_to_string(", "read(", "File::open("],
    ) {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "rust_check_then_open_path".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!(
            "function {} checks filesystem state before opening a path",
            function.fingerprint.name
        ),
        evidence: vec![
            "check-then-open flow may race on mutable filesystems".to_string(),
            "canonicalize or open first with the desired flags when appropriate".to_string(),
        ],
    }]
}

fn secret_comparison_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    for (offset, line) in function.body_text.lines().enumerate() {
        if !(line.contains("==") || line.contains("!=")) {
            continue;
        }

        if !has_secret_like_text(line)
            && !function
                .local_binding_names
                .iter()
                .any(|name| has_secret_like_text(name))
        {
            continue;
        }

        return vec![Finding {
            rule_id: "rust_secret_equality_compare".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line + offset,
            end_line: function.fingerprint.start_line + offset,
            message: format!(
                "function {} compares a secret-like value with == or !=",
                function.fingerprint.name
            ),
            evidence: vec![line.trim().to_string()],
        }];
    }

    Vec::new()
}

fn narrowing_cast_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    for (offset, line) in function.body_text.lines().enumerate() {
        if !has_numeric_narrowing_cast(line) {
            continue;
        }

        return vec![Finding {
            rule_id: "rust_narrowing_numeric_cast".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line + offset,
            end_line: function.fingerprint.start_line + offset,
            message: format!(
                "function {} uses a numeric narrowing cast",
                function.fingerprint.name
            ),
            evidence: vec![line.trim().to_string()],
        }];
    }

    Vec::new()
}

fn manual_tempdir_function_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !(contains_any(
        &function.body_text,
        &["tempfile::Builder::new()", "tempfile::Builder::new"],
    ) || contains_any(&function.body_text, &["std::env::temp_dir()", "temp_dir()"]))
    {
        return Vec::new();
    }

    if !contains_any(
        &function.body_text,
        &["remove_dir_all(", "fs::remove_dir_all(", ".remove_dir_all("],
    ) {
        return Vec::new();
    }

    let Some(line) = first_line_with_any(
        &function.body_text,
        function.fingerprint.start_line,
        &[
            "tempfile::Builder::new()",
            "std::env::temp_dir()",
            "temp_dir()",
        ],
    ) else {
        return Vec::new();
    };

    vec![Finding {
        rule_id: "rust_manual_tempdir_lifecycle".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!(
            "function {} manually creates and cleans up a temp directory",
            function.fingerprint.name
        ),
        evidence: vec![
            "prefer tempfile::TempDir or another RAII helper when possible".to_string(),
            "manual cleanup is easy to get wrong in error paths".to_string(),
        ],
    }]
}
