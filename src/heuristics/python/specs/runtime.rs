use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use super::super::{ai_smells, duplication, structure};
use super::catalog::{
    AI_SMELLS_FILE_EVALUATORS, DUPLICATION_FILE_EVALUATORS, DUPLICATION_REPO_EVALUATORS,
    FRAMEWORK_EVALUATORS, HOTPATH_EVALUATORS, HOTPATH_EXT_EVALUATORS, MAINTAINABILITY_EVALUATORS,
    MAINTAINABILITY_FILE_EVALUATORS, MLOPS_EVALUATORS, PERFORMANCE_EVALUATORS,
    STRUCTURE_FILE_EVALUATORS, STRUCTURE_FUNCTION_EVALUATORS, STRUCTURE_REPO_EVALUATORS,
};
use super::types::{
    FileEvaluator, FunctionEvaluator, PythonFileRuleSpec, PythonFunctionRuleSpec,
    PythonRepoRuleSpec, RepoEvaluator,
};

pub(crate) fn evaluate_function_specs(
    specs: &[PythonFunctionRuleSpec],
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for spec in specs {
        let _ = spec.family;
        let _ = spec.rule_ids;
        findings.extend((spec.evaluate)(file, function));
    }
    findings
}

pub(crate) fn evaluate_file_specs(specs: &[PythonFileRuleSpec], file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    for spec in specs {
        let _ = spec.family;
        let _ = spec.rule_ids;
        findings.extend((spec.evaluate)(file));
    }
    findings
}

pub(crate) fn evaluate_repo_specs(
    specs: &[PythonRepoRuleSpec],
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for spec in specs {
        let _ = spec.family;
        let _ = spec.rule_ids;
        findings.extend((spec.evaluate)(files, index));
    }
    findings
}

fn evaluate_function_group(
    evaluators: &[FunctionEvaluator],
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for evaluator in evaluators {
        findings.extend(evaluator(file, function));
    }
    findings
}

fn evaluate_file_group(evaluators: &[FileEvaluator], file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    for evaluator in evaluators {
        findings.extend(evaluator(file));
    }
    findings
}

fn evaluate_repo_group(
    evaluators: &[RepoEvaluator],
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for evaluator in evaluators {
        findings.extend(evaluator(files, index));
    }
    findings
}

pub(crate) fn ai_smells_function_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    ai_smells::textbook_docstring_findings(file, function)
}

pub(crate) fn performance_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    evaluate_function_group(PERFORMANCE_EVALUATORS, file, function)
}

pub(crate) fn maintainability_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    evaluate_function_group(MAINTAINABILITY_EVALUATORS, file, function)
}

pub(crate) fn structure_function_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    evaluate_function_group(STRUCTURE_FUNCTION_EVALUATORS, file, function)
}

pub(crate) fn hotpath_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    evaluate_function_group(HOTPATH_EVALUATORS, file, function)
}

pub(crate) fn hotpath_ext_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    evaluate_function_group(HOTPATH_EXT_EVALUATORS, file, function)
}

pub(crate) fn framework_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    evaluate_function_group(FRAMEWORK_EVALUATORS, file, function)
}

pub(crate) fn mlops_family_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    evaluate_function_group(MLOPS_EVALUATORS, file, function)
}

pub(crate) fn structure_file_family_findings(file: &ParsedFile) -> Vec<Finding> {
    evaluate_file_group(STRUCTURE_FILE_EVALUATORS, file)
}

pub(crate) fn ai_smells_file_family_findings(file: &ParsedFile) -> Vec<Finding> {
    evaluate_file_group(AI_SMELLS_FILE_EVALUATORS, file)
}

pub(crate) fn maintainability_file_family_findings(file: &ParsedFile) -> Vec<Finding> {
    evaluate_file_group(MAINTAINABILITY_FILE_EVALUATORS, file)
}

pub(crate) fn duplication_file_family_findings(file: &ParsedFile) -> Vec<Finding> {
    evaluate_file_group(DUPLICATION_FILE_EVALUATORS, file)
}

pub(crate) fn structure_repo_family_findings(
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    evaluate_repo_group(STRUCTURE_REPO_EVALUATORS, files, index)
}

pub(crate) fn duplication_repo_family_findings(
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    evaluate_repo_group(DUPLICATION_REPO_EVALUATORS, files, index)
}

pub(crate) fn deep_inheritance_repo_findings(
    files: &[&ParsedFile],
    _index: &RepositoryIndex,
) -> Vec<Finding> {
    structure::deep_inheritance_findings(files)
}

pub(crate) fn cross_file_dupe_repo_findings(
    files: &[&ParsedFile],
    _index: &RepositoryIndex,
) -> Vec<Finding> {
    duplication::cross_file_dupe_findings(files)
}

pub(crate) fn test_utility_logic_repo_findings(
    files: &[&ParsedFile],
    _index: &RepositoryIndex,
) -> Vec<Finding> {
    duplication::test_utility_logic_findings(files)
}

pub(crate) fn cross_file_literal_repo_findings(
    files: &[&ParsedFile],
    _index: &RepositoryIndex,
) -> Vec<Finding> {
    duplication::cross_file_literal_findings(files)
}

pub(crate) fn duplicate_query_fragment_repo_findings(
    files: &[&ParsedFile],
    _index: &RepositoryIndex,
) -> Vec<Finding> {
    duplication::duplicate_query_fragment_findings(files)
}

pub(crate) fn duplicate_transformation_pipeline_repo_findings(
    files: &[&ParsedFile],
    _index: &RepositoryIndex,
) -> Vec<Finding> {
    duplication::duplicate_transformation_pipeline_findings(files)
}
