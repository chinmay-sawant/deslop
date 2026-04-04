use crate::analysis::{AnalysisConfig, ImportSpec, ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use super::registry::{
    ConfigurableFunctionRule, FileFunctionRule, FileRule, FunctionRule,
    GO_CONFIGURABLE_FUNCTION_RULES, GO_FILE_FUNCTION_RULES, GO_FILE_RULES, GO_FUNCTION_RULES,
    GO_INDEXED_FUNCTION_RULES, GO_REPO_RULES, IndexedFunctionRule, IndexedRepoRule,
    OptionalFunctionRule, PYTHON_FILE_RULES, PYTHON_FUNCTION_RULES, PYTHON_INDEXED_FUNCTION_RULES,
    PYTHON_REPO_RULES, RepoRule, SHARED_FILE_RULES, SHARED_FUNCTION_RULES,
    SHARED_OPTIONAL_FUNCTION_RULES,
};

pub(crate) fn evaluate_shared_file(file: &ParsedFile, _index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    extend_file_rules(&mut findings, file, SHARED_FILE_RULES);

    for function in &file.functions {
        extend_optional_function_rules(
            &mut findings,
            file,
            function,
            SHARED_OPTIONAL_FUNCTION_RULES,
        );
        extend_function_rules(&mut findings, file, function, SHARED_FUNCTION_RULES);
    }

    findings
}

pub(crate) fn evaluate_go_file(
    file: &ParsedFile,
    index: &RepositoryIndex,
    analysis_config: &AnalysisConfig,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    extend_file_rules(&mut findings, file, GO_FILE_RULES);

    for function in &file.functions {
        extend_function_rules(&mut findings, file, function, GO_FUNCTION_RULES);
        extend_file_function_rules(
            &mut findings,
            file,
            function,
            &file.imports,
            GO_FILE_FUNCTION_RULES,
        );
        extend_indexed_function_rules(
            &mut findings,
            file,
            function,
            index,
            GO_INDEXED_FUNCTION_RULES,
        );
        extend_configurable_function_rules(
            &mut findings,
            file,
            function,
            analysis_config.enable_go_semantic,
            GO_CONFIGURABLE_FUNCTION_RULES,
        );
    }

    findings
}

pub(crate) fn evaluate_go_repo(files: &[&ParsedFile], _index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    extend_repo_rules(&mut findings, files, GO_REPO_RULES);

    findings
}

pub(crate) fn evaluate_python_file(file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    extend_file_rules(&mut findings, file, PYTHON_FILE_RULES);

    for function in &file.functions {
        extend_function_rules(&mut findings, file, function, PYTHON_FUNCTION_RULES);
        extend_indexed_function_rules(
            &mut findings,
            file,
            function,
            index,
            PYTHON_INDEXED_FUNCTION_RULES,
        );
    }

    findings
}

pub(crate) fn evaluate_python_repo(files: &[&ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    extend_indexed_repo_rules(&mut findings, files, index, PYTHON_REPO_RULES);

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
    imports: &[ImportSpec],
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
