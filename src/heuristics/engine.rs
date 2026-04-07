use crate::analysis::{AnalysisConfig, Language, ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use super::registry::{
    ConfigurableFunctionRule, FileFunctionRule, FileRule, FunctionRule, IndexedFileRule,
    IndexedFunctionRule, IndexedRepoRule, OptionalFunctionRule, RepoRule, RuleExecutionSpec,
    language_rule_specs, shared_rule_specs,
};

pub(crate) fn evaluate_file(
    file: &ParsedFile,
    index: &RepositoryIndex,
    analysis_config: &AnalysisConfig,
) -> Vec<Finding> {
    let mut findings = evaluate_file_specs(shared_rule_specs(), file, index, analysis_config);
    findings.extend(evaluate_file_specs(
        language_rule_specs(file.language),
        file,
        index,
        analysis_config,
    ));
    findings
}

pub(crate) fn evaluate_repo(
    language: Language,
    files: &[&ParsedFile],
    index: &RepositoryIndex,
    analysis_config: &AnalysisConfig,
) -> Vec<Finding> {
    evaluate_repo_specs(language_rule_specs(language), files, index, analysis_config)
}

fn evaluate_file_specs(
    specs: &[RuleExecutionSpec],
    file: &ParsedFile,
    index: &RepositoryIndex,
    analysis_config: &AnalysisConfig,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for spec in specs {
        let _ = spec.family;
        extend_file_rules(&mut findings, file, spec.file_rules);
        extend_indexed_file_rules(&mut findings, file, index, spec.indexed_file_rules);

        for function in &file.functions {
            extend_optional_function_rules(
                &mut findings,
                file,
                function,
                spec.optional_function_rules,
            );
            extend_function_rules(&mut findings, file, function, spec.function_rules);
            extend_file_function_rules(
                &mut findings,
                file,
                function,
                &file.imports,
                spec.file_function_rules,
            );
            extend_indexed_function_rules(
                &mut findings,
                file,
                function,
                index,
                spec.indexed_function_rules,
            );
            extend_configurable_function_rules(
                &mut findings,
                file,
                function,
                (spec.configurable_enabled)(analysis_config),
                spec.configurable_function_rules,
            );
        }
    }

    findings
}

fn evaluate_repo_specs(
    specs: &[RuleExecutionSpec],
    files: &[&ParsedFile],
    index: &RepositoryIndex,
    _analysis_config: &AnalysisConfig,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for spec in specs {
        let _ = spec.family;
        extend_repo_rules(&mut findings, files, spec.repo_rules);
        extend_indexed_repo_rules(&mut findings, files, index, spec.indexed_repo_rules);
    }

    findings
}

pub(crate) fn extend_file_rules(
    findings: &mut Vec<Finding>,
    file: &ParsedFile,
    rules: &[FileRule],
) {
    for rule in rules {
        findings.extend(rule(file));
    }
}

pub(crate) fn extend_indexed_file_rules(
    findings: &mut Vec<Finding>,
    file: &ParsedFile,
    index: &RepositoryIndex,
    rules: &[IndexedFileRule],
) {
    for rule in rules {
        findings.extend(rule(file, index));
    }
}

pub(crate) fn extend_function_rules(
    findings: &mut Vec<Finding>,
    file: &ParsedFile,
    function: &ParsedFunction,
    rules: &[FunctionRule],
) {
    for rule in rules {
        findings.extend(rule(file, function));
    }
}

pub(crate) fn extend_optional_function_rules(
    findings: &mut Vec<Finding>,
    file: &ParsedFile,
    function: &ParsedFunction,
    rules: &[OptionalFunctionRule],
) {
    for rule in rules {
        if let Some(finding) = rule(file, function) {
            findings.push(finding);
        }
    }
}

pub(crate) fn extend_file_function_rules(
    findings: &mut Vec<Finding>,
    file: &ParsedFile,
    function: &ParsedFunction,
    imports: &[crate::analysis::ImportSpec],
    rules: &[FileFunctionRule],
) {
    for rule in rules {
        findings.extend(rule(file, function, imports));
    }
}

pub(crate) fn extend_indexed_function_rules(
    findings: &mut Vec<Finding>,
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
    rules: &[IndexedFunctionRule],
) {
    for rule in rules {
        findings.extend(rule(file, function, index));
    }
}

pub(crate) fn extend_configurable_function_rules(
    findings: &mut Vec<Finding>,
    file: &ParsedFile,
    function: &ParsedFunction,
    enabled: bool,
    rules: &[ConfigurableFunctionRule],
) {
    for rule in rules {
        findings.extend(rule(file, function, enabled));
    }
}

pub(crate) fn extend_repo_rules(
    findings: &mut Vec<Finding>,
    files: &[&ParsedFile],
    rules: &[RepoRule],
) {
    for rule in rules {
        findings.extend(rule(files));
    }
}

pub(crate) fn extend_indexed_repo_rules(
    findings: &mut Vec<Finding>,
    files: &[&ParsedFile],
    index: &RepositoryIndex,
    rules: &[IndexedRepoRule],
) {
    for rule in rules {
        findings.extend(rule(files, index));
    }
}
