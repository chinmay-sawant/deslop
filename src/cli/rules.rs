use std::collections::BTreeMap;
use std::fmt::Write as _;

use anyhow::Result;

use deslop::{RuleConfigurability, RuleLanguage, RuleMetadata, RuleStatus, rule_registry};

pub(crate) fn filtered_rules(
    language: Option<RuleLanguage>,
    status: Option<RuleStatus>,
) -> Vec<&'static RuleMetadata> {
    rule_registry()
        .iter()
        .filter(|metadata| {
            language
                .as_ref()
                .is_none_or(|value| metadata.language == *value)
        })
        .filter(|metadata| {
            status
                .as_ref()
                .is_none_or(|value| metadata.status == *value)
        })
        .collect()
}

pub(crate) fn format_rules_report(
    rules: &[&RuleMetadata],
    language: Option<RuleLanguage>,
    status: Option<RuleStatus>,
) -> String {
    let mut output = String::new();

    let _ = writeln!(
        &mut output,
        "deslop rules: {} matching rule entries",
        rules.len()
    );
    if language.is_some() || status.is_some() {
        let _ = writeln!(
            &mut output,
            "filters: language={} status={}",
            language.as_ref().map_or("<all-languages>", language_label),
            status.as_ref().map_or("<all-statuses>", status_label)
        );
    }

    let mut by_language = BTreeMap::new();
    let mut by_status = BTreeMap::new();
    for metadata in rules {
        *by_language
            .entry(language_label(&metadata.language))
            .or_insert(0usize) += 1;
        *by_status
            .entry(status_label(&metadata.status))
            .or_insert(0usize) += 1;
    }

    if !by_language.is_empty() {
        let _ = writeln!(
            &mut output,
            "by language: {}",
            by_language
                .iter()
                .map(|(language, count)| format!("{language}={count}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !by_status.is_empty() {
        let _ = writeln!(
            &mut output,
            "by status: {}",
            by_status
                .iter()
                .map(|(status, count)| format!("{status}={count}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if rules.is_empty() {
        return output;
    }

    output.push('\n');

    for metadata in rules {
        let _ = writeln!(
            &mut output,
            "- {} [{} / {} / {} / {}]",
            metadata.id,
            language_label(&metadata.language),
            metadata.family,
            severity_label(&metadata.default_severity),
            status_label(&metadata.status),
        );
        let _ = writeln!(&mut output, "  {}", metadata.description);
        let _ = writeln!(
            &mut output,
            "  configurable via: {}",
            metadata
                .configurability
                .iter()
                .map(config_label)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    output
}

pub(crate) fn format_rules_report_json(rules: &[&RuleMetadata]) -> Result<String> {
    Ok(serde_json::to_string_pretty(rules)?)
}

fn language_label(language: &RuleLanguage) -> &'static str {
    match language {
        RuleLanguage::Common => "common",
        RuleLanguage::Go => "go",
        RuleLanguage::Python => "python",
        RuleLanguage::Rust => "rust",
    }
}

fn status_label(status: &RuleStatus) -> &'static str {
    match status {
        RuleStatus::Stable => "stable",
        RuleStatus::Experimental => "experimental",
        RuleStatus::Research => "research",
    }
}

fn severity_label(severity: &deslop::RuleDefaultSeverity) -> &'static str {
    match severity {
        deslop::RuleDefaultSeverity::Info => "info",
        deslop::RuleDefaultSeverity::Warning => "warning",
        deslop::RuleDefaultSeverity::Error => "error",
        deslop::RuleDefaultSeverity::Contextual => "contextual",
    }
}

fn config_label(config: &RuleConfigurability) -> &'static str {
    match config {
        RuleConfigurability::Disable => "disabled_rules",
        RuleConfigurability::Ignore => "scan --ignore",
        RuleConfigurability::SeverityOverride => "severity_overrides",
        RuleConfigurability::DetailsOnly => "--details",
        RuleConfigurability::GoSemanticExperimental => {
            "go_semantic_experimental / --enable-semantic"
        }
        RuleConfigurability::RustAsyncExperimental => "rust_async_experimental",
    }
}

#[cfg(test)]
mod tests {
    use deslop::{RuleLanguage, RuleStatus};

    use super::{filtered_rules, format_rules_report, format_rules_report_json};

    #[test]
    fn filters_rules_by_language_and_status() {
        let go_rules = filtered_rules(Some(RuleLanguage::Go), None);
        assert!(
            go_rules
                .iter()
                .all(|metadata| metadata.language == RuleLanguage::Go)
        );

        let experimental = filtered_rules(None, Some(RuleStatus::Experimental));
        assert!(
            experimental
                .iter()
                .all(|metadata| metadata.status == RuleStatus::Experimental)
        );
    }

    #[test]
    fn text_rules_report_includes_summary_and_config() {
        let rules = filtered_rules(Some(RuleLanguage::Common), None);
        let output = format_rules_report(&rules, Some(RuleLanguage::Common), None);

        assert!(output.contains("deslop rules:"));
        assert!(output.contains("filters: language=common status=<all-statuses>"));
        assert!(output.contains("hallucinated_import_call"));
        assert!(
            output.contains("configurable via: disabled_rules, scan --ignore, severity_overrides")
        );
    }

    #[test]
    fn json_rules_report_renders_metadata() {
        let rules = filtered_rules(None, Some(RuleStatus::Experimental));
        let output = format_rules_report_json(&rules).expect("json should render");

        assert!(output.contains("\"status\": \"experimental\""));
        assert!(output.contains("\"configurability\""));
    }
}
