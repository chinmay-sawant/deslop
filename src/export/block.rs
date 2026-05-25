use std::collections::HashMap;
use std::path::PathBuf;

use crate::analysis::ParsedFile;
use crate::model::{Finding, ScanReport};
use crate::rules::{RuleLanguage, is_detail_only_rule, rule_metadata, rule_metadata_variants};

use super::function::{infer_language, resolve_function_context};
use super::triage::{summarize_rule_metadata, triage_finding};

const REVIEW_PLACEHOLDER: &str = "REVIEW_NEEDED";
const BLOCK_SEPARATOR: &str = "====================================================================================================";

pub(crate) struct FindingBlock {
    pub text: String,
}

pub(crate) fn build_finding_block(
    finding: &Finding,
    report: &ScanReport,
    parsed_files: &[ParsedFile],
    cached_lines: &mut HashMap<PathBuf, Vec<String>>,
    index: usize,
    total: usize,
    details: bool,
) -> FindingBlock {
    if !finding.path.exists() {
        return FindingBlock {
            text: build_missing_file_block(finding, report, index, total, details),
        };
    }

    let language = finding_language(report, finding);
    let metadata = language.and_then(|lang| rule_metadata(finding.rule_id.as_str(), lang));
    let function_context = resolve_function_context(finding, parsed_files, cached_lines);
    let source_lines = cached_lines.get(&finding.path).cloned().unwrap_or_default();
    let triage = triage_finding(
        finding,
        &source_lines,
        function_context.start_line,
        function_context.end_line,
        metadata,
    );
    let (family, severity, status, languages, description) =
        summarize_rule_metadata(finding.rule_id.as_str(), language);

    let mut block_lines = vec![format!("Finding {index}/{total}")];

    if details {
        block_lines.extend([
            format!("Source: {}:{}", finding.path.display(), finding.start_line),
            format!("Rule: [{}]", finding.rule_id),
            format!("Rule family: [{family}]"),
            format!("Rule severity: [{severity}]"),
            format!("Rule status: [{status}]"),
            format!("Rule languages: [{languages}]"),
            format!("Rule description: {description}"),
            format!("Message: {}", finding.message),
            format!(
                "Function range: [{}-{}]",
                function_context.start_line, function_context.end_line
            ),
            format!("Auto triage: [{}]", triage.label),
            format!("Auto triage note: {}", triage.note),
            format!("False positive: [{REVIEW_PLACEHOLDER}]"),
            format!(
                "Original finding:   - {}:{} {} [{}]",
                finding.path.display(),
                finding.start_line,
                finding.message,
                finding.rule_id
            ),
            "Function:".to_string(),
        ]);
    } else {
        block_lines.extend([
            format!("Source: {}:{}", finding.path.display(), finding.start_line),
            format!("Rule: [{}]", finding.rule_id),
            format!("Rule severity: [{severity}]"),
            format!("Rule status: [{status}]"),
            format!("Rule description: {description}"),
            format!("Message: {}", finding.message),
            format!(
                "Function range: [{}-{}]",
                function_context.start_line, function_context.end_line
            ),
            format!("Auto triage: [{}]", triage.label),
            format!("Auto triage note: {}", triage.note),
            "Function:".to_string(),
        ]);
    }

    if function_context.lines.is_empty() {
        block_lines.push("    [FUNCTION_NOT_FOUND]".to_string());
    } else {
        for line in &function_context.lines {
            if line.is_empty() {
                block_lines.push(String::new());
            } else {
                block_lines.push(format!("    {line}"));
            }
        }
    }

    FindingBlock {
        text: format!("{}\n", block_lines.join("\n")),
    }
}

fn build_missing_file_block(
    finding: &Finding,
    report: &ScanReport,
    index: usize,
    total: usize,
    details: bool,
) -> String {
    let language = finding_language(report, finding);
    let metadata = language.and_then(|lang| rule_metadata(finding.rule_id.as_str(), lang));
    let triage = triage_finding(
        finding,
        &[],
        finding.start_line,
        finding.start_line,
        metadata,
    );
    let (family, severity, status, languages, description) =
        summarize_rule_metadata(finding.rule_id.as_str(), language);

    let mut block_lines = vec![format!("Finding {index}/{total}")];

    if details {
        block_lines.extend([
            format!("Source: {}:{}", finding.path.display(), finding.start_line),
            format!("Rule: [{}]", finding.rule_id),
            format!("Rule family: [{family}]"),
            format!("Rule severity: [{severity}]"),
            format!("Rule status: [{status}]"),
            format!("Rule languages: [{languages}]"),
            format!("Rule description: {description}"),
            format!("Message: {}", finding.message),
            format!("Auto triage: [{}]", triage.label),
            format!("Auto triage note: {}", triage.note),
            format!("False positive: [{REVIEW_PLACEHOLDER}]"),
            format!(
                "Original finding:   - {}:{} {} [{}]",
                finding.path.display(),
                finding.start_line,
                finding.message,
                finding.rule_id
            ),
            "Function: [FILE_NOT_FOUND]".to_string(),
        ]);
    } else {
        block_lines.extend([
            format!("Source: {}:{}", finding.path.display(), finding.start_line),
            format!("Rule: [{}]", finding.rule_id),
            format!("Rule severity: [{severity}]"),
            format!("Rule status: [{status}]"),
            format!("Rule description: {description}"),
            format!("Message: {}", finding.message),
            format!("Auto triage: [{}]", triage.label),
            format!("Auto triage note: {}", triage.note),
            "Function: [FILE_NOT_FOUND]".to_string(),
        ]);
    }

    format!("{}\n", block_lines.join("\n"))
}

pub(crate) fn visible_findings(report: &ScanReport, details: bool) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| {
            details
                || finding_language(report, finding)
                    .is_none_or(|language| !is_detail_only_rule(finding.rule_id.as_str(), language))
        })
        .collect()
}

fn finding_language(report: &ScanReport, finding: &Finding) -> Option<RuleLanguage> {
    report
        .files
        .iter()
        .find(|file| file.path == finding.path)
        .map(|file| file.language)
        .or_else(|| unique_rule_language(finding.rule_id.as_str()))
        .or_else(|| infer_language(&finding.path))
}

fn unique_rule_language(rule_id: &str) -> Option<RuleLanguage> {
    let variants = rule_metadata_variants(rule_id);
    let first = variants.first()?;
    variants
        .iter()
        .all(|variant| variant.language == first.language)
        .then_some(first.language)
}

pub(crate) fn block_separator() -> &'static str {
    BLOCK_SEPARATOR
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::model::{Finding, IndexSummary, ScanReport, Severity, TimingBreakdown};

    #[test]
    fn compact_block_includes_core_fields() {
        let finding = Finding {
            rule_id: "generic_function_name".to_string(),
            severity: Severity::Info,
            path: PathBuf::from("main.go"),
            function_name: Some("Handle".to_string()),
            start_line: 3,
            end_line: 3,
            message: "function Handle uses a generic name".to_string(),
            evidence: Vec::new(),
        };
        let report = ScanReport {
            root: PathBuf::from("."),
            files_discovered: 1,
            files_analyzed: 1,
            functions_found: 1,
            files: Vec::new(),
            findings: vec![finding.clone()],
            index_summary: IndexSummary {
                package_count: 0,
                symbol_count: 0,
                import_count: 0,
            },
            parse_failures: Vec::new(),
            timings: TimingBreakdown {
                discover_ms: 0,
                parse_ms: 0,
                index_ms: 0,
                heuristics_ms: 0,
                total_ms: 0,
            },
        };

        let block = build_finding_block(&finding, &report, &[], &mut HashMap::new(), 1, 1, false);
        assert!(block.text.contains("Finding 1/1"));
        assert!(block.text.contains("Rule: [generic_function_name]"));
        assert!(block.text.contains("Auto triage note:"));
    }
}
