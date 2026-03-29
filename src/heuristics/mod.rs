mod comments;
mod common;
mod concurrency;
mod consistency;
mod context;
mod errors;
mod hallucination;
mod naming;
mod performance;
mod python;
pub(crate) mod rust;
mod security;
mod style;
mod test_quality;
#[cfg(test)]
mod tests;

use crate::analysis::AnalysisConfig;
use crate::analysis::ParsedFile;
use crate::index::RepositoryIndex;
use crate::model::Finding;

use self::comments::comment_findings;
use self::concurrency::{
    coordination_findings, deeper_goroutine_lifetime_findings, mutex_findings, shutdown_findings,
};
use self::consistency::{receiver_findings, tag_findings};
use self::context::{
    busy_findings, cancel_findings, ctx_findings, propagate_findings, sleep_findings,
};
use self::errors::error_findings;
use self::hallucination::hallucination_findings;
use self::naming::{generic_finding, overlong_finding, weak_finding};
use self::performance::{
    alloc_findings, concat_findings, db_findings, fmt_findings, json_findings, load_findings,
    n_squared_findings, reflect_findings,
};
use self::python::{python_file_findings, python_findings, python_repo_findings};
use self::security::{crypto_findings, pkg_secret_findings, secret_findings, sql_findings};
use self::style::{import_grouping_findings, package_name_consistency};
use self::test_quality::test_findings;

pub(crate) fn evaluate_shared(files: &[ParsedFile], _index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    for file in files {
        findings.extend(pkg_secret_findings(file));

        for function in &file.functions {
            if let Some(finding) = generic_finding(file, function) {
                findings.push(finding);
            }

            if let Some(finding) = overlong_finding(file, function) {
                findings.push(finding);
            }

            if let Some(finding) = weak_finding(file, function) {
                findings.push(finding);
            }

            findings.extend(comment_findings(file, function));
            findings.extend(secret_findings(file, function));
            findings.extend(test_findings(file, function));
        }
    }

    findings
}

pub(crate) fn evaluate_go_file(
    file: &ParsedFile,
    index: &RepositoryIndex,
    analysis_config: &AnalysisConfig,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    findings.extend(tag_findings(file));
    findings.extend(import_grouping_findings(file));

    for function in &file.functions {
        findings.extend(error_findings(file, function));
        findings.extend(crypto_findings(file, function));
        findings.extend(sql_findings(file, function));
        findings.extend(ctx_findings(file, function));
        findings.extend(propagate_findings(file, function, index));
        findings.extend(cancel_findings(file, function));
        findings.extend(sleep_findings(file, function));
        findings.extend(busy_findings(file, function));
        findings.extend(shutdown_findings(file, function));
        findings.extend(deeper_goroutine_lifetime_findings(file, function));
        findings.extend(mutex_findings(file, function, &file.imports));
        findings.extend(alloc_findings(file, function));
        findings.extend(fmt_findings(file, function));
        findings.extend(reflect_findings(file, function));
        findings.extend(concat_findings(file, function));
        findings.extend(n_squared_findings(
            file,
            function,
            analysis_config.enable_go_semantic,
        ));
        findings.extend(json_findings(file, function));
        findings.extend(db_findings(
            file,
            function,
            analysis_config.enable_go_semantic,
        ));
        findings.extend(load_findings(file, function));
        findings.extend(coordination_findings(file, function));
        findings.extend(hallucination_findings(file, function, index));
    }

    findings
}

pub(crate) fn evaluate_go_repo(files: &[&ParsedFile], _index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = receiver_findings(files);
    findings.extend(package_name_consistency(files));
    findings
}

pub(crate) fn evaluate_python_file(file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = python_file_findings(file);

    for function in &file.functions {
        findings.extend(python_findings(file, function));
        findings.extend(hallucination_findings(file, function, index));
    }

    findings
}

pub(crate) fn evaluate_python_repo(files: &[&ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    python_repo_findings(files, index)
}
