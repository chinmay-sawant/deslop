mod ai_smells;
mod duplication;
mod maintainability;
mod performance;
mod structure;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::Finding;

use self::ai_smells::{mixed_naming_convention_findings, textbook_docstring_findings};
use self::duplication::repeated_string_literal_findings;
use self::maintainability::{
    eval_exec_findings, exception_swallowed_findings, hardcoded_path_findings,
    none_comparison_findings, print_debugging_findings, redundant_return_none_findings,
    side_effect_comprehension_findings, variadic_public_api_findings,
};
use self::performance::{
    blocking_sync_io_findings, deque_candidate_findings, full_dataset_load_findings,
    list_materialization_findings, string_concat_findings,
};
use self::structure::{
    god_function_findings, monolithic_init_module_findings, too_many_instance_attributes_findings,
};

pub(crate) fn python_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(string_concat_findings(file, function));
    findings.extend(blocking_sync_io_findings(file, function));
    findings.extend(full_dataset_load_findings(file, function));
    findings.extend(list_materialization_findings(file, function));
    findings.extend(deque_candidate_findings(file, function));
    findings.extend(exception_swallowed_findings(file, function));
    findings.extend(eval_exec_findings(file, function));
    findings.extend(print_debugging_findings(file, function));
    findings.extend(none_comparison_findings(file, function));
    findings.extend(side_effect_comprehension_findings(file, function));
    findings.extend(redundant_return_none_findings(file, function));
    findings.extend(hardcoded_path_findings(file, function));
    findings.extend(variadic_public_api_findings(file, function));
    findings.extend(god_function_findings(file, function));
    findings.extend(textbook_docstring_findings(file, function));
    findings
}

pub(crate) fn python_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(monolithic_init_module_findings(file));
    findings.extend(too_many_instance_attributes_findings(file));
    findings.extend(mixed_naming_convention_findings(file));
    findings.extend(repeated_string_literal_findings(file));
    findings
}
