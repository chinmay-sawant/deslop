mod ai_smells;
mod duplication;
mod framework;
mod hotpath;
mod hotpath_ext;
mod maintainability;
mod mlops;
mod packaging;
mod performance;
mod quality;
mod specs;
mod structure;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use self::specs::{
    FILE_RULE_SPECS, FUNCTION_RULE_SPECS, REPO_RULE_SPECS, evaluate_file_specs,
    evaluate_function_specs, evaluate_repo_specs,
};

pub(crate) fn python_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    evaluate_function_specs(FUNCTION_RULE_SPECS, file, function)
}

pub(crate) fn python_file_findings(file: &ParsedFile) -> Vec<Finding> {
    evaluate_file_specs(FILE_RULE_SPECS, file)
}

pub(crate) fn python_repo_findings(files: &[&ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    evaluate_repo_specs(REPO_RULE_SPECS, files, index)
}
