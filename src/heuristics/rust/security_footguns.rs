use std::path::Path;

use toml::Value;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::io::{DEFAULT_MAX_BYTES, read_to_string_limited};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::{contains_any, first_line_with_any, is_scanner_infra_file, is_test_like};
use super::{file_finding, function_finding};

pub(crate) const RULE_DEFINITIONS: &[crate::rules::catalog::RuleDefinition] = &[
    crate::rules::catalog::RuleDefinition {
        id: "rust_split_at_unchecked_external_input",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "security_footguns",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Slice splitting and range indexing on externally-derived offsets without obvious bounds guards.",
        binding_location: crate::rules::catalog::bindings::RUST_SECURITY_FOOTGUNS,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_from_utf8_unchecked_boundary",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "security_footguns",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Unchecked UTF-8 conversion at a repository or service boundary.",
        binding_location: crate::rules::catalog::bindings::RUST_SECURITY_FOOTGUNS,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_thread_spawn_async_without_runtime",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "security_footguns",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Raw std::thread::spawn blocks that call async work without an explicit runtime handoff.",
        binding_location: crate::rules::catalog::bindings::RUST_SECURITY_FOOTGUNS,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rc_cycle_parent_link",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "security_footguns",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Rc-based parent/back-reference shapes that likely need Weak on the reverse edge.",
        binding_location: crate::rules::catalog::bindings::RUST_SECURITY_FOOTGUNS,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_static_mut_global",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "security_footguns",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "static mut global state that bypasses the safer shared-state models already in the scanner.",
        binding_location: crate::rules::catalog::bindings::RUST_SECURITY_FOOTGUNS,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_release_profile_missing_overflow_checks",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "security_footguns",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Release profiles that omit overflow-checks = true in Cargo.toml.",
        binding_location: crate::rules::catalog::bindings::RUST_SECURITY_FOOTGUNS,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_release_profile_panic_unwind",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "security_footguns",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Release profiles that still explicitly use panic = \"unwind\".",
        binding_location: crate::rules::catalog::bindings::RUST_SECURITY_FOOTGUNS,
    },
];

pub(crate) fn security_footguns_file_findings(
    file: &ParsedFile,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    if is_test_like(file, None) {
        return Vec::new();
    }

    let mut findings = Vec::new();
    findings.extend(static_mut_findings(file));
    findings.extend(rc_cycle_findings(file));
    findings.extend(release_profile_findings(file, index));
    findings
}

pub(crate) fn security_footguns_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if is_test_like(file, Some(function)) || is_scanner_infra_file(file) {
        return Vec::new();
    }

    let mut findings = Vec::new();
    findings.extend(split_at_findings(file, function));
    findings.extend(from_utf8_findings(file, function));
    findings.extend(thread_spawn_findings(file, function));
    findings
}

fn static_mut_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for static_summary in file.rust_statics() {
        if !static_summary.is_mut {
            continue;
        }

        findings.push(file_finding(
            file,
            "rust_static_mut_global",
            Severity::Warning,
            static_summary.line,
            format!(
                "file {} declares mutable global static state",
                file.path.display()
            ),
            vec![
                format!(
                    "static {}: {}",
                    static_summary.name, static_summary.type_text
                ),
                "prefer interior mutability behind a narrower ownership boundary".to_string(),
            ],
        ));
    }

    findings
}

fn rc_cycle_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for summary in file.structs() {
        let Some(field) = summary.fields.iter().find(|field| {
            let name = field.name.to_ascii_lowercase();
            let type_text = field.type_text.as_str();
            name.contains("parent") && type_text.contains("Rc<") && !type_text.contains("Weak<")
        }) else {
            continue;
        };

        findings.push(file_finding(
            file,
            "rust_rc_cycle_parent_link",
            Severity::Warning,
            field.line,
            format!(
                "struct {} keeps a parent link in Rc instead of Weak",
                summary.name
            ),
            vec![
                format!("field {}: {}", field.name, field.type_text),
                "parent back-references are usually better modeled with Weak".to_string(),
            ],
        ));
    }

    findings
}

fn release_profile_findings(file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
    let manifest_path = index.root().join("Cargo.toml");
    if !should_emit_manifest_finding(file, index.root()) || !manifest_path.exists() {
        return Vec::new();
    }

    let Ok(source) = read_to_string_limited(&manifest_path, DEFAULT_MAX_BYTES) else {
        return Vec::new();
    };
    let Ok(parsed) = source.parse::<Value>() else {
        return Vec::new();
    };

    let Some(profile) = parsed.get("profile").and_then(Value::as_table) else {
        return Vec::new();
    };
    let Some(release) = profile.get("release").and_then(Value::as_table) else {
        return Vec::new();
    };
    let Some((section_line, section_end)) = release_profile_span(&source) else {
        return Vec::new();
    };

    let mut findings = Vec::new();

    match release.get("overflow-checks").and_then(Value::as_bool) {
        Some(true) => {}
        Some(false) => {
            if let Some(line) =
                find_line_in_span(&source, section_line, section_end, "overflow-checks")
            {
                findings.push(file_finding(
                    file,
                    "rust_release_profile_missing_overflow_checks",
                    Severity::Warning,
                    line,
                    "release profile disables overflow checks".to_string(),
                    vec!["overflow-checks = false".to_string()],
                ));
            }
        }
        None => {
            findings.push(file_finding(
                file,
                "rust_release_profile_missing_overflow_checks",
                Severity::Warning,
                section_line,
                "release profile omits overflow-checks = true".to_string(),
                vec![format!("manifest={}", manifest_path.display())],
            ));
        }
    }

    if let Some(value) = release.get("panic").and_then(Value::as_str)
        && value == "unwind"
        && let Some(line) =
            find_line_in_span(&source, section_line, section_end, "panic = \"unwind\"")
    {
        findings.push(file_finding(
            file,
            "rust_release_profile_panic_unwind",
            Severity::Warning,
            line,
            "release profile explicitly keeps panic = \"unwind\"".to_string(),
            vec!["panic = \"unwind\"".to_string()],
        ));
    }

    findings
}

fn split_at_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !has_index_like_context(function) {
        return Vec::new();
    }

    let guard_markers = [
        "assert!",
        "debug_assert!",
        "checked_",
        "saturating_",
        "clamp(",
        "min(",
        "max(",
    ];
    let split_markers = ["split_at(", "split_at_mut("];

    for (offset, line) in function.body_text.lines().enumerate() {
        if !contains_any(line, &split_markers) {
            continue;
        }

        if contains_any(&function.body_text, &guard_markers) {
            continue;
        }

        return vec![function_finding(
            file,
            function,
            "rust_split_at_unchecked_external_input",
            Severity::Warning,
            function.fingerprint.start_line + offset,
            format!(
                "function {} splits a slice on a likely external offset without a visible guard",
                function.fingerprint.name
            ),
            vec![line.trim().to_string()],
        )];
    }

    Vec::new()
}

fn from_utf8_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let Some(line) = first_line_with_any(
        &function.body_text,
        function.fingerprint.start_line,
        &["from_utf8_unchecked(", "std::str::from_utf8_unchecked("],
    ) else {
        return Vec::new();
    };

    vec![function_finding(
        file,
        function,
        "rust_from_utf8_unchecked_boundary",
        Severity::Warning,
        line,
        format!(
            "function {} uses unchecked UTF-8 conversion",
            function.fingerprint.name
        ),
        vec!["unchecked conversion should only appear with a very explicit invariant".to_string()],
    )]
}

fn thread_spawn_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !contains_any(
        &function.body_text,
        &["thread::spawn(", "std::thread::spawn("],
    ) {
        return Vec::new();
    }

    if contains_any(
        &function.body_text,
        &[
            "Runtime::new(",
            "Handle::current()",
            "block_on(",
            "spawn_blocking(",
        ],
    ) {
        return Vec::new();
    }

    if !contains_any(&function.body_text, &[".await", "async move", "async {"]) {
        return Vec::new();
    }

    let Some(line) = first_line_with_any(
        &function.body_text,
        function.fingerprint.start_line,
        &["thread::spawn(", "std::thread::spawn("],
    ) else {
        return Vec::new();
    };

    vec![function_finding(
        file,
        function,
        "rust_thread_spawn_async_without_runtime",
        Severity::Warning,
        line,
        format!(
            "function {} spawns a thread that still performs async work without a clear runtime handoff",
            function.fingerprint.name
        ),
        vec![
            "raw thread spawning and async work usually need an explicit runtime boundary"
                .to_string(),
        ],
    )]
}

fn should_emit_manifest_finding(file: &ParsedFile, root: &Path) -> bool {
    let candidates = [
        root.join("src/lib.rs"),
        root.join("src/main.rs"),
        root.join("lib.rs"),
        root.join("main.rs"),
    ];
    candidates.iter().any(|candidate| candidate == &file.path)
}

fn release_profile_span(source: &str) -> Option<(usize, usize)> {
    let lines = source.lines().collect::<Vec<_>>();
    let section_idx = lines
        .iter()
        .position(|line| line.trim() == "[profile.release]")?;
    let start_line = section_idx + 1;
    let mut end_line = lines.len() + 1;

    for (offset, line) in lines.iter().enumerate().skip(section_idx + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            end_line = offset + 1;
            break;
        }
    }

    Some((start_line, end_line))
}

fn find_line_in_span(
    source: &str,
    start_line: usize,
    end_line: usize,
    needle: &str,
) -> Option<usize> {
    source
        .lines()
        .enumerate()
        .filter_map(|(offset, line)| {
            let line_number = offset + 1;
            (line_number >= start_line && line_number < end_line).then_some((line_number, line))
        })
        .find(|(_, line)| line.contains(needle))
        .map(|(line_number, _)| line_number)
}

fn has_index_like_context(function: &ParsedFunction) -> bool {
    let suspicious_terms = [
        "idx", "index", "offset", "start", "end", "pos", "cursor", "len", "span",
    ];

    suspicious_terms.iter().any(|term| {
        function.signature_text.to_ascii_lowercase().contains(term)
            || function
                .local_binding_names
                .iter()
                .any(|name| name.to_ascii_lowercase().contains(term))
            || function.body_text.to_ascii_lowercase().contains(term)
    })
}
