mod ai_smells;
mod architecture;
mod boundaries;
mod discipline;
mod duplication;
mod framework;
mod hotpath;
mod hotpath_ext;
mod maintainability;
mod mlops;
mod observability;
mod packaging;
mod performance;
mod quality;
mod specs;
mod structure;

pub(crate) const AI_SMELLS_BINDING_LOCATION: &str = ai_smells::BINDING_LOCATION;
pub(crate) const ARCHITECTURE_BINDING_LOCATION: &str = architecture::BINDING_LOCATION;
pub(crate) const BOUNDARIES_BINDING_LOCATION: &str = boundaries::BINDING_LOCATION;
pub(crate) const DISCIPLINE_BINDING_LOCATION: &str = discipline::BINDING_LOCATION;
pub(crate) const OBSERVABILITY_BINDING_LOCATION: &str = observability::BINDING_LOCATION;
pub(crate) const DUPLICATION_BINDING_LOCATION: &str = duplication::BINDING_LOCATION;
pub(crate) const FRAMEWORK_BINDING_LOCATION: &str = framework::BINDING_LOCATION;
pub(crate) const HOTPATH_BINDING_LOCATION: &str = hotpath::BINDING_LOCATION;
pub(crate) const HOTPATH_EXT_BINDING_LOCATION: &str = hotpath_ext::BINDING_LOCATION;
pub(crate) const MAINTAINABILITY_BINDING_LOCATION: &str = maintainability::BINDING_LOCATION;
pub(crate) const MLOPS_BINDING_LOCATION: &str = mlops::BINDING_LOCATION;
pub(crate) const PACKAGING_BINDING_LOCATION: &str = packaging::BINDING_LOCATION;
pub(crate) const PERFORMANCE_BINDING_LOCATION: &str = performance::BINDING_LOCATION;
pub(crate) const QUALITY_BINDING_LOCATION: &str = quality::BINDING_LOCATION;
pub(crate) const STRUCTURE_BINDING_LOCATION: &str = structure::BINDING_LOCATION;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use self::specs::{
    FILE_RULE_SPECS, FUNCTION_RULE_SPECS, REPO_RULE_SPECS, evaluate_file_specs,
    evaluate_function_specs, evaluate_repo_specs,
};

pub(super) fn is_python_package_entrypoint(file: &ParsedFile) -> bool {
    file.path.file_name().and_then(|name| name.to_str()) == Some("__init__.py")
}

pub(super) fn is_to_dict_wrapper(function: &ParsedFunction) -> bool {
    if function.fingerprint.receiver_type.is_none() || function.fingerprint.name != "to_dict" {
        return false;
    }

    let body_lines = function
        .body_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();

    matches!(body_lines.as_slice(), [line] if line.starts_with("return _to_dict(self"))
}

pub(crate) fn should_skip_python_wide_contract(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> bool {
    quality::should_skip_wide_contract_function(file, function)
}

pub(crate) fn should_skip_python_weak_typing(file: &ParsedFile, function: &ParsedFunction) -> bool {
    quality::should_skip_weak_typing_function(file, function)
}

pub(crate) fn python_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    evaluate_function_specs(FUNCTION_RULE_SPECS, file, function)
}

pub(crate) fn python_file_findings(file: &ParsedFile) -> Vec<Finding> {
    evaluate_file_specs(FILE_RULE_SPECS, file)
}

pub(crate) fn python_repo_findings(files: &[&ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    evaluate_repo_specs(REPO_RULE_SPECS, files, index)
}
