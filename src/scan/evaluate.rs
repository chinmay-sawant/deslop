use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::RepoConfig;
use crate::analysis::{AnalysisConfig, ParsedFile, backend_for_language, registered_backends};
use crate::heuristics::evaluate_shared;
use crate::index::RepositoryIndex;
use crate::model::Finding;

use super::suppression::{SuppressionDirective, is_suppressed};

pub(super) fn evaluate_findings(
    files: &[ParsedFile],
    index: &RepositoryIndex,
    suppressions: &BTreeMap<PathBuf, Vec<SuppressionDirective>>,
    repo_config: &RepoConfig,
    root: &Path,
    analysis_config: &AnalysisConfig,
) -> Vec<Finding> {
    let mut findings = evaluate_shared(files, index);

    for file in files {
        if let Some(backend) = backend_for_language(file.language) {
            findings.extend(backend.evaluate_file(file, index, analysis_config));
        }
    }

    for backend in registered_backends() {
        let backend_files = files
            .iter()
            .filter(|file| file.language == backend.language())
            .collect::<Vec<_>>();
        findings.extend(backend.evaluate_repo(&backend_files, index, analysis_config));
    }

    findings.retain(|finding| !is_suppressed(finding, suppressions));
    apply_repository_config(&mut findings, repo_config, root);

    findings.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    findings
}

pub(super) fn apply_repository_config(
    findings: &mut Vec<Finding>,
    repo_config: &RepoConfig,
    root: &Path,
) {
    findings.retain(|finding| {
        !repo_config
            .disabled_rules
            .iter()
            .any(|rule_id| rule_id == &finding.rule_id)
            && (repo_config.rust_async_experimental || !is_async_rollout_rule(&finding.rule_id))
            && !path_is_suppressed(&finding.path, root, &repo_config.suppressed_paths)
    });

    for finding in findings.iter_mut() {
        if let Some(severity) = repo_config.severity_overrides.get(&finding.rule_id) {
            finding.severity = severity.clone();
        }
    }
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

fn is_async_rollout_rule(rule_id: &str) -> bool {
    matches!(
        rule_id,
        "rust_blocking_io_in_async" | "rust_lock_across_await" | "rust_tokio_mutex_unnecessary"
    ) || rule_id.starts_with("rust_async_")
}
