use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::RepoConfig;
use crate::analysis::{AnalysisConfig, ParsedFile, registered_backends};
use crate::heuristics::{evaluate_file, evaluate_repo};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use crate::{
    RuleLanguage, default_finding_severity, is_async_rollout_rule, rule_metadata_variants,
};
use rayon::prelude::*;

use super::suppression::{SuppressionDirective, is_suppressed};

pub(super) fn evaluate_findings(
    files: &[ParsedFile],
    index: &RepositoryIndex,
    suppressions: &BTreeMap<PathBuf, Vec<SuppressionDirective>>,
    repo_config: &RepoConfig,
    root: &Path,
    analysis_config: &AnalysisConfig,
) -> Vec<Finding> {
    let mut findings: Vec<Finding> = files
        .par_iter()
        .flat_map(|file| evaluate_file(file, index, analysis_config))
        .collect();

    for &backend in registered_backends() {
        let backend_files = files
            .iter()
            .filter(|file| file.language == backend.language())
            .collect::<Vec<_>>();
        findings.extend(evaluate_repo(
            backend.language(),
            &backend_files,
            index,
            analysis_config,
        ));
    }

    findings.retain(|finding| !is_suppressed(finding, suppressions));
    apply_registry_defaults(&mut findings, files);
    apply_repository_config(&mut findings, files, repo_config, root);

    findings.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    // Remove any exact (path, start_line, rule_id) duplicates that could arise
    // if two evaluation paths both emit the same finding for the same location.
    // The sort above places identical keys adjacent, so dedup_by is O(n).
    findings.dedup_by(|a, b| {
        a.path == b.path && a.start_line == b.start_line && a.rule_id == b.rule_id
    });
    findings
}

fn apply_registry_defaults(findings: &mut [Finding], files: &[ParsedFile]) {
    for finding in findings {
        let Some(language) = finding_language(files, finding) else {
            continue;
        };
        if let Some(severity) = default_finding_severity(&finding.rule_id, language) {
            finding.severity = severity;
        }
    }
}

pub(super) fn apply_repository_config(
    findings: &mut Vec<Finding>,
    files: &[ParsedFile],
    repo_config: &RepoConfig,
    root: &Path,
) {
    findings.retain(|finding| {
        !repo_config
            .disabled_rules
            .iter()
            .any(|rule_id| rule_id == &finding.rule_id)
            && (repo_config.rust_async_experimental
                || finding_language(files, finding)
                    .is_none_or(|language| !is_async_rollout_rule(&finding.rule_id, language)))
            && !path_is_suppressed(&finding.path, root, &repo_config.suppressed_paths)
    });

    for finding in findings.iter_mut() {
        if let Some(severity) = repo_config.severity_overrides.get(&finding.rule_id) {
            finding.severity = severity.clone();
        }
    }
}

fn finding_language(files: &[ParsedFile], finding: &Finding) -> Option<RuleLanguage> {
    files
        .iter()
        .find(|file| file.path == finding.path)
        .map(|file| rule_language(file.language))
        .or_else(|| unique_rule_language(finding.rule_id.as_str()))
}

const fn rule_language(language: crate::analysis::Language) -> RuleLanguage {
    match language {
        crate::analysis::Language::Go => RuleLanguage::Go,
        crate::analysis::Language::Python => RuleLanguage::Python,
        crate::analysis::Language::Rust => RuleLanguage::Rust,
    }
}

fn unique_rule_language(rule_id: &str) -> Option<RuleLanguage> {
    let variants = rule_metadata_variants(rule_id);
    let first = variants.first()?;
    variants
        .iter()
        .all(|variant| variant.language == first.language)
        .then_some(first.language)
}

fn path_is_suppressed(path: &Path, root: &Path, suppressed_paths: &[PathBuf]) -> bool {
    suppressed_paths.iter().any(|prefix| {
        if prefix.is_absolute() {
            path.starts_with(prefix)
        } else {
            path.strip_prefix(root)
                .is_ok_and(|relative_path| relative_path.starts_with(prefix))
        }
    })
}
