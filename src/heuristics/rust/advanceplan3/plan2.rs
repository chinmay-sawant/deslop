use crate::analysis::ParsedFile;
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::{file_attributes, is_main_like_file, is_test_like};

pub(crate) const RULE_DEFINITIONS: &[crate::rules::catalog::RuleDefinition] = &[
    crate::rules::catalog::RuleDefinition {
        id: "rust_oversized_module_file",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "module_surface",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Rust module files that grow too large and mix too many responsibilities.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN2,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_pub_use_glob_surface",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "module_surface",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Public glob re-exports that flatten the crate surface.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN2,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_root_reexport_wall",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "module_surface",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Crate roots that expose too many public re-exports at once.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN2,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_mod_rs_catchall",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "module_surface",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "mod.rs files that look like catch-all subsystem dumps.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN2,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_duplicate_bootstrap_sequence",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "module_surface",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Repeated startup or bootstrap wiring in multiple functions within the same file.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN2,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_redundant_path_attribute",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "module_surface",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Same-directory #[path = \"...\"] module attributes that standard resolution could replace.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN2,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_broad_allow_dead_code",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "module_surface",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Broad dead_code suppression that can hide real wiring or maintenance gaps.",
        binding_location: crate::rules::catalog::bindings::RUST_ADVANCEPLAN3_PLAN2,
    },
];

pub(crate) fn file_findings(file: &ParsedFile) -> Vec<Finding> {
    if is_test_like(file, None) {
        return Vec::new();
    }

    let mut findings = Vec::new();
    findings.extend(oversized_module_findings(file));
    findings.extend(public_surface_findings(file));
    findings.extend(mod_rs_findings(file));
    findings.extend(bootstrap_sequence_findings(file));
    findings.extend(path_attribute_findings(file));
    findings.extend(dead_code_allow_findings(file));
    findings
}

pub(crate) fn function_findings(
    _file: &ParsedFile,
    _function: &crate::analysis::ParsedFunction,
) -> Vec<Finding> {
    Vec::new()
}

fn oversized_module_findings(file: &ParsedFile) -> Vec<Finding> {
    let top_level_count = file.functions.len()
        + file.imports.len()
        + file
            .rust_data()
            .map(|data| data.structs.len() + data.rust_enums.len() + data.rust_statics.len())
            .unwrap_or(0);
    if top_level_count < 18 || file.line_count < 40 {
        return Vec::new();
    }

    if !(file.path.ends_with("lib.rs")
        || file.path.ends_with("main.rs")
        || file.path.file_name().and_then(|name| name.to_str()) == Some("mod.rs"))
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "rust_oversized_module_file".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: 1,
        end_line: 1,
        message: format!(
            "module file {} is large enough to justify a closer ownership split",
            file.path.display()
        ),
        evidence: vec![
            format!("top_level_items={top_level_count}"),
            format!("line_count={}", file.line_count),
        ],
    }]
}

fn public_surface_findings(file: &ParsedFile) -> Vec<Finding> {
    let public_globs = file
        .imports
        .iter()
        .filter(|import| import.is_public && import.alias == "*")
        .collect::<Vec<_>>();
    let public_reexports = file
        .imports
        .iter()
        .filter(|import| import.is_public && import.alias != "*")
        .collect::<Vec<_>>();

    let mut findings = Vec::new();
    if !public_globs.is_empty() {
        findings.push(Finding {
            rule_id: "rust_pub_use_glob_surface".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: public_globs[0].line,
            end_line: public_globs[0].line,
            message: format!("file {} uses a public glob re-export", file.path.display()),
            evidence: vec![public_globs[0].path.clone()],
        });
    }

    if (file.path.ends_with("lib.rs") || file.path.ends_with("mod.rs"))
        && public_reexports.len() >= 5
    {
        findings.push(Finding {
            rule_id: "rust_root_reexport_wall".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: public_reexports[0].line,
            end_line: public_reexports[0].line,
            message: format!(
                "file {} exposes many public re-exports from the root surface",
                file.path.display()
            ),
            evidence: vec![format!("public_reexports={}", public_reexports.len())],
        });
    }

    findings
}

fn mod_rs_findings(file: &ParsedFile) -> Vec<Finding> {
    let Some(file_name) = file.path.file_name().and_then(|name| name.to_str()) else {
        return Vec::new();
    };

    if file_name != "mod.rs" {
        return Vec::new();
    }

    let top_level_count = file.functions.len()
        + file.imports.len()
        + file
            .rust_data()
            .map(|data| data.structs.len() + data.rust_enums.len() + data.rust_statics.len())
            .unwrap_or(0);
    if top_level_count < 10 {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "rust_mod_rs_catchall".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: 1,
        end_line: 1,
        message: format!(
            "mod.rs file {} is acting as a catch-all module",
            file.path.display()
        ),
        evidence: vec![format!("top_level_items={top_level_count}")],
    }]
}

fn bootstrap_sequence_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut bootstrap_hits = Vec::new();

    for function in &file.functions {
        let body = function.body_text.as_str();
        let markers = [
            "Router::new(",
            "Runtime::new(",
            "Builder::new_multi_thread(",
            "Builder::new_current_thread(",
            "Client::new(",
            ".connect().await",
        ];
        if markers.iter().any(|marker| body.contains(marker)) {
            bootstrap_hits.push((
                function.fingerprint.name.clone(),
                function.fingerprint.start_line,
            ));
        }
    }

    if bootstrap_hits.len() < 2 {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "rust_duplicate_bootstrap_sequence".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(bootstrap_hits[0].0.clone()),
        start_line: bootstrap_hits[0].1,
        end_line: bootstrap_hits[0].1,
        message: format!(
            "file {} contains repeated bootstrap-style setup functions",
            file.path.display()
        ),
        evidence: bootstrap_hits
            .iter()
            .map(|(name, line)| format!("{name} at line {line}"))
            .collect(),
    }]
}

fn path_attribute_findings(file: &ParsedFile) -> Vec<Finding> {
    let Some(attribute) = file_attributes(file)
        .iter()
        .find(|attribute| attribute.text.contains("#[path="))
    else {
        return Vec::new();
    };

    if attribute.text.contains("../") {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "rust_redundant_path_attribute".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: attribute.line,
        end_line: attribute.line,
        message: format!(
            "file {} uses a redundant #[path] module attribute",
            file.path.display()
        ),
        evidence: vec![attribute.text.clone()],
    }]
}

fn dead_code_allow_findings(file: &ParsedFile) -> Vec<Finding> {
    let Some(attribute) = file_attributes(file)
        .iter()
        .find(|attribute| attribute.text.contains("allow(dead_code)"))
    else {
        return Vec::new();
    };

    if is_main_like_file(file) || file.is_test_file {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "rust_broad_allow_dead_code".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: attribute.line,
        end_line: attribute.line,
        message: format!("file {} suppresses dead_code broadly", file.path.display()),
        evidence: vec![attribute.text.clone()],
    }]
}
