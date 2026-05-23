use crate::model::Finding;
use crate::rules::{RuleDefaultSeverity, RuleLanguage, RuleMetadata, RuleStatus};

const SUBJECTIVE_FAMILIES: &[&str] = &[
    "ai_smells",
    "api_design",
    "comments",
    "domain_modeling",
    "duplication",
    "maintainability",
    "mod",
    "module_surface",
    "naming",
    "packaging",
    "quality",
    "structure",
    "style",
    "test_quality",
];

const CONTEXT_FAMILIES: &[&str] = &[
    "async_patterns",
    "boundary",
    "concurrency",
    "consistency",
    "context",
    "data_access",
    "framework",
    "gin",
    "hot_path",
    "hot_path_ext",
    "idioms",
    "library",
    "mlops",
    "performance",
    "runtime_boundary",
    "runtime_ownership",
];

const RISK_FAMILIES: &[&str] = &[
    "errors",
    "hallucination",
    "hygiene",
    "security",
    "security_footguns",
    "unsafe_soundness",
];

pub(crate) struct TriageResult {
    pub label: &'static str,
    pub note: String,
}

pub(crate) fn triage_finding(
    finding: &Finding,
    source_lines: &[String],
    function_start: usize,
    function_end: usize,
    metadata: Option<&RuleMetadata>,
) -> TriageResult {
    let context_text = if source_lines.is_empty() || function_start == 0 || function_end == 0 {
        String::new()
    } else {
        source_lines[function_start.saturating_sub(1)..function_end.min(source_lines.len())]
            .join("\n")
            .to_lowercase()
    };

    let search_start = finding.start_line.saturating_sub(26);
    let search_end = (finding.start_line + 5).min(source_lines.len());
    let extended_text = if source_lines.is_empty() {
        String::new()
    } else {
        source_lines[search_start..search_end]
            .join("\n")
            .to_lowercase()
    };

    let current_line = source_lines
        .get(finding.start_line.saturating_sub(1))
        .map(|line| line.trim().to_string())
        .unwrap_or_default();

    if finding.rule_id == "cgo_string_lifetime"
        && format!("{context_text}\n{extended_text}").contains("caller must free")
    {
        return TriageResult {
            label: "LIKELY_FALSE_POSITIVE",
            note: "Nearby comments suggest the API intentionally transfers ownership of the allocated C string to the caller.".to_string(),
        };
    }

    if finding.rule_id == "len_string_for_empty_check" {
        return triage_len_empty(&current_line);
    }

    triage_by_metadata(metadata)
}

fn triage_len_empty(current_line: &str) -> TriageResult {
    const COLLECTION_HINTS: &[&str] = &[
        "parts", "items", "files", "rows", "entries", "fonts", "pages", "results", "matches",
        "tokens", "children", "values",
    ];

    if COLLECTION_HINTS
        .iter()
        .any(|hint| current_line.contains(hint))
    {
        return TriageResult {
            label: "LIKELY_FALSE_POSITIVE",
            note: "The flagged len(...) check appears to target a collection rather than a string empty-check.".to_string(),
        };
    }

    TriageResult {
        label: "CONTEXT_DEPENDENT",
        note: "This may be style-only or incorrect depending on the type of the value passed to len(...).".to_string(),
    }
}

fn triage_by_metadata(metadata: Option<&RuleMetadata>) -> TriageResult {
    let Some(metadata) = metadata else {
        return TriageResult {
            label: "REVIEW_NEEDED",
            note:
                "No safe automatic classification was inferred from the local code context alone."
                    .to_string(),
        };
    };

    let experimental_note = if metadata.status == RuleStatus::Experimental {
        " The rule is marked experimental in the registry, so keep a slightly higher false-positive bar."
    } else {
        ""
    };

    let family = metadata.family;
    let severity = severity_label(metadata.default_severity);

    if SUBJECTIVE_FAMILIES.contains(&family) {
        return TriageResult {
            label: "LIKELY_SUBJECTIVE",
            note: format!(
                "Registry metadata classifies this as {family} guidance with {severity} severity, so whether it matters depends on project conventions.{experimental_note}"
            ),
        };
    }

    if CONTEXT_FAMILIES.contains(&family)
        || metadata.default_severity == RuleDefaultSeverity::Contextual
    {
        return TriageResult {
            label: "CONTEXT_DEPENDENT",
            note: format!(
                "Registry metadata classifies this as {family} with {severity} severity, so runtime path, workload, and surrounding design matter before treating it as actionable.{experimental_note}"
            ),
        };
    }

    if RISK_FAMILIES.contains(&family)
        || matches!(
            metadata.default_severity,
            RuleDefaultSeverity::Warning | RuleDefaultSeverity::Error
        )
    {
        return TriageResult {
            label: "LIKELY_REAL",
            note: format!(
                "Registry metadata classifies this as {family} with {severity} severity, which usually maps to correctness, security, or production risk.{experimental_note}"
            ),
        };
    }

    if metadata.default_severity == RuleDefaultSeverity::Info {
        return TriageResult {
            label: "LIKELY_SUBJECTIVE",
            note: format!(
                "Registry metadata marks this as info-level guidance; treat it as a review prompt rather than a clear defect.{experimental_note}"
            ),
        };
    }

    TriageResult {
        label: "REVIEW_NEEDED",
        note: "No safe automatic classification was inferred from the local code context alone."
            .to_string(),
    }
}

pub(crate) fn summarize_rule_metadata(
    rule_id: &str,
    language: Option<RuleLanguage>,
) -> (String, String, String, String, String) {
    let variants = crate::rules::rule_metadata_variants(rule_id);
    if variants.is_empty() {
        return (
            "unknown".to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
            "Rule metadata not found in rules/registry.json.".to_string(),
        );
    }

    let resolved = language
        .and_then(|lang| {
            variants
                .iter()
                .find(|variant| variant.language == lang)
                .copied()
        })
        .or(variants.first().copied());

    let Some(first) = resolved else {
        return (
            "unknown".to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
            "Rule metadata not found in rules/registry.json.".to_string(),
        );
    };

    let languages = variants
        .iter()
        .map(|variant| language_code(variant.language))
        .collect::<Vec<_>>()
        .join(", ");

    (
        first.family.to_string(),
        severity_label(first.default_severity).to_string(),
        status_label(first.status).to_string(),
        languages,
        first.description.to_string(),
    )
}

fn severity_label(severity: RuleDefaultSeverity) -> &'static str {
    match severity {
        RuleDefaultSeverity::Info => "info",
        RuleDefaultSeverity::Warning => "warning",
        RuleDefaultSeverity::Error => "error",
        RuleDefaultSeverity::Contextual => "contextual",
    }
}

fn status_label(status: RuleStatus) -> &'static str {
    match status {
        RuleStatus::Stable => "stable",
        RuleStatus::Experimental => "experimental",
        RuleStatus::Research => "research",
    }
}

fn language_code(language: RuleLanguage) -> &'static str {
    match language {
        RuleLanguage::Common => "common",
        RuleLanguage::Go => "go",
        RuleLanguage::Python => "python",
        RuleLanguage::Rust => "rust",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triage_len_empty_collection_hint() {
        let result = triage_len_empty("if len(parts) == 0:");
        assert_eq!(result.label, "LIKELY_FALSE_POSITIVE");
    }
}
