mod ai_smells;
mod duplication;
mod maintainability;
mod performance;
mod structure;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use self::ai_smells::{
    mixed_naming_convention_findings, obvious_commentary_findings,
    textbook_docstring_findings, enthusiastic_commentary_findings,
    unrelated_heavy_import_findings,
};
use self::duplication::{
    cross_file_copy_paste_function_findings, cross_file_repeated_literal_findings,
    duplicate_query_fragment_findings, duplicate_test_utility_logic_findings,
    duplicate_transformation_pipeline_findings,
    repeated_exception_block_findings, repeated_string_literal_findings,
    repeated_validation_pipeline_findings,
};
use self::maintainability::{
    broad_exception_handler_findings, builtin_reduction_findings, commented_out_code_findings,
    environment_boundary_without_fallback_findings, eval_exec_findings,
    exception_swallowed_findings, external_input_without_validation_findings,
    hardcoded_business_rule_findings, hardcoded_path_findings,
    magic_value_branching_findings, network_boundary_without_timeout_findings,
    missing_context_manager_findings, mixed_sync_async_module_findings,
    none_comparison_findings, print_debugging_findings, public_api_missing_type_hints_findings,
    reinvented_utility_findings,
    redundant_return_none_findings, side_effect_comprehension_findings,
    variadic_public_api_findings,
};
use self::performance::{
    blocking_sync_io_findings, deque_candidate_findings, full_dataset_load_findings,
    list_materialization_findings, list_membership_findings, recursive_traversal_findings,
    repeated_len_findings, string_concat_findings, temp_collection_findings,
};
use self::structure::{
    deep_inheritance_findings, eager_constructor_collaborator_findings, god_class_findings,
    god_function_findings, mixed_concern_findings, monolithic_init_module_findings,
    monolithic_module_findings, name_responsibility_mismatch_findings,
    module_name_responsibility_mismatch_findings, over_abstracted_wrapper_findings,
    tight_module_coupling_findings, too_many_instance_attributes_findings,
};

pub(crate) fn python_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(string_concat_findings(file, function));
    findings.extend(blocking_sync_io_findings(file, function));
    findings.extend(full_dataset_load_findings(file, function));
    findings.extend(list_materialization_findings(file, function));
    findings.extend(deque_candidate_findings(file, function));
    findings.extend(temp_collection_findings(file, function));
    findings.extend(recursive_traversal_findings(file, function));
    findings.extend(list_membership_findings(file, function));
    findings.extend(repeated_len_findings(file, function));
    findings.extend(exception_swallowed_findings(file, function));
    findings.extend(broad_exception_handler_findings(file, function));
    findings.extend(eval_exec_findings(file, function));
    findings.extend(print_debugging_findings(file, function));
    findings.extend(none_comparison_findings(file, function));
    findings.extend(side_effect_comprehension_findings(file, function));
    findings.extend(redundant_return_none_findings(file, function));
    findings.extend(hardcoded_path_findings(file, function));
    findings.extend(hardcoded_business_rule_findings(file, function));
    findings.extend(magic_value_branching_findings(file, function));
    findings.extend(reinvented_utility_findings(file, function));
    findings.extend(builtin_reduction_findings(file, function));
    findings.extend(missing_context_manager_findings(file, function));
    findings.extend(network_boundary_without_timeout_findings(file, function));
    findings.extend(environment_boundary_without_fallback_findings(file, function));
    findings.extend(external_input_without_validation_findings(file, function));
    findings.extend(public_api_missing_type_hints_findings(file, function));
    findings.extend(variadic_public_api_findings(file, function));
    findings.extend(god_function_findings(file, function));
    findings.extend(mixed_concern_findings(file, function));
    findings.extend(name_responsibility_mismatch_findings(file, function));
    findings.extend(textbook_docstring_findings(file, function));
    findings
}

pub(crate) fn python_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(monolithic_init_module_findings(file));
    findings.extend(monolithic_module_findings(file));
    findings.extend(too_many_instance_attributes_findings(file));
    findings.extend(god_class_findings(file));
    findings.extend(eager_constructor_collaborator_findings(file));
    findings.extend(over_abstracted_wrapper_findings(file));
    findings.extend(module_name_responsibility_mismatch_findings(file));
    findings.extend(mixed_naming_convention_findings(file));
    findings.extend(unrelated_heavy_import_findings(file));
    findings.extend(obvious_commentary_findings(file));
    findings.extend(enthusiastic_commentary_findings(file));
    findings.extend(commented_out_code_findings(file));
    findings.extend(mixed_sync_async_module_findings(file));
    findings.extend(repeated_string_literal_findings(file));
    findings.extend(repeated_exception_block_findings(file));
    findings.extend(repeated_validation_pipeline_findings(file));
    findings
}

pub(crate) fn python_repo_findings(files: &[&ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(deep_inheritance_findings(files));
    findings.extend(tight_module_coupling_findings(files, index));
    findings.extend(cross_file_copy_paste_function_findings(files));
    findings.extend(duplicate_test_utility_logic_findings(files));
    findings.extend(cross_file_repeated_literal_findings(files));
    findings.extend(duplicate_query_fragment_findings(files));
    findings.extend(duplicate_transformation_pipeline_findings(files));
    findings
}
