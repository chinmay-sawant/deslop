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
mod structure;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use self::ai_smells::{
    enthusiastic_commentary_findings, mixed_naming_convention_findings,
    obvious_commentary_findings, textbook_docstring_findings, unrelated_heavy_import_findings,
};
use self::duplication::{
    cross_file_dupe_findings, cross_file_literal_findings, duplicate_query_fragment_findings,
    duplicate_transformation_pipeline_findings, repeated_exception_block_findings,
    repeated_string_literal_findings, repeated_validation_pipeline_findings,
    test_utility_logic_findings,
};
use self::framework::{
    celery_task_findings, click_typer_command_findings, django_extra_findings,
    django_loop_db_findings, django_n_plus_one_findings, django_queryset_findings,
    django_values_findings, fastapi_handler_findings, flask_handler_findings,
    handler_fanout_findings, middleware_findings, pydantic_v2_findings, response_extra_findings,
    sqlalchemy_findings, sqlmodel_findings, template_response_findings,
};
use self::hotpath::{
    csv_flush_per_row_findings, dict_materialization_in_loop_findings,
    enumerate_range_len_findings, in_list_literal_findings, json_repeated_call_findings,
    len_comprehension_findings, read_splitlines_findings, readlines_then_iterate_findings,
    regex_compile_in_hotpath_findings, repeated_open_findings, sorted_first_findings,
    startswith_chain_findings, write_in_loop_findings,
};
use self::hotpath_ext::{
    append_sort_in_loop_findings, concat_in_comprehension_findings, copy_in_loop_findings,
    datetime_strptime_repeated_findings, filter_count_iterate_findings,
    gzip_open_per_chunk_findings, hashlib_repeated_findings, index_in_loop_findings,
    invariant_call_in_loop_findings, isinstance_chain_findings, join_list_comp_findings,
    json_encoder_recreated_findings, nested_list_search_findings, pickle_in_loop_findings,
    repeated_format_findings, repeated_subscript_findings, sort_then_first_findings,
    tuple_unpacking_in_tight_loop_findings, xml_repeated_parse_findings,
    yaml_repeated_load_findings,
};
use self::maintainability::{
    api_type_hint_findings, broad_exception_handler_findings, builtin_reduction_findings,
    commented_out_code_findings, env_fallback_findings, eval_exec_findings,
    exception_swallowed_findings, hardcoded_business_rule_findings, hardcoded_path_findings,
    input_validation_findings, magic_value_branching_findings, missing_context_manager_findings,
    network_timeout_findings, none_comparison_findings, print_debugging_findings,
    redundant_return_none_findings, reinvented_utility_findings,
    side_effect_comprehension_findings, sync_async_module_findings, variadic_public_api_findings,
};
use self::mlops::{
    data_pipeline_findings, llm_findings, mlops_extra_findings, model_inference_findings,
    numpy_findings, pandas_findings,
};
use self::packaging::{public_api_any_contract_findings, pyproject_repo_findings};
use self::performance::{
    blocking_sync_io_findings, deque_candidate_findings, full_dataset_load_findings,
    list_materialization_findings, list_membership_findings, recursive_traversal_findings,
    repeated_len_findings, string_concat_findings, temp_collection_findings,
};
use self::quality::{quality_file_findings, quality_function_findings};
use self::structure::{
    deep_inheritance_findings, eager_constructor_collaborator_findings, god_class_findings,
    god_function_findings, mixed_concern_findings, module_name_mismatch_findings,
    monolithic_init_module_findings, monolithic_module_findings,
    name_responsibility_mismatch_findings, over_abstracted_wrapper_findings,
    tight_module_coupling_findings, too_many_attributes_findings,
};

pub(crate) fn python_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(quality_function_findings(file, function));
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
    findings.extend(network_timeout_findings(file, function));
    findings.extend(env_fallback_findings(file, function));
    findings.extend(input_validation_findings(file, function));
    findings.extend(api_type_hint_findings(file, function));
    findings.extend(variadic_public_api_findings(file, function));
    findings.extend(public_api_any_contract_findings(file, function));
    findings.extend(god_function_findings(file, function));
    findings.extend(mixed_concern_findings(file, function));
    findings.extend(name_responsibility_mismatch_findings(file, function));
    findings.extend(textbook_docstring_findings(file, function));
    findings.extend(regex_compile_in_hotpath_findings(file, function));
    findings.extend(json_repeated_call_findings(file, function));
    findings.extend(sorted_first_findings(file, function));
    findings.extend(len_comprehension_findings(file, function));
    findings.extend(readlines_then_iterate_findings(file, function));
    findings.extend(read_splitlines_findings(file, function));
    findings.extend(in_list_literal_findings(file, function));
    findings.extend(startswith_chain_findings(file, function));
    findings.extend(enumerate_range_len_findings(file, function));
    findings.extend(csv_flush_per_row_findings(file, function));
    findings.extend(write_in_loop_findings(file, function));
    findings.extend(repeated_open_findings(file, function));
    findings.extend(dict_materialization_in_loop_findings(file, function));
    findings.extend(yaml_repeated_load_findings(file, function));
    findings.extend(xml_repeated_parse_findings(file, function));
    findings.extend(datetime_strptime_repeated_findings(file, function));
    findings.extend(hashlib_repeated_findings(file, function));
    findings.extend(copy_in_loop_findings(file, function));
    findings.extend(invariant_call_in_loop_findings(file, function));
    findings.extend(index_in_loop_findings(file, function));
    findings.extend(append_sort_in_loop_findings(file, function));
    findings.extend(join_list_comp_findings(file, function));
    findings.extend(repeated_subscript_findings(file, function));
    findings.extend(nested_list_search_findings(file, function));
    findings.extend(sort_then_first_findings(file, function));
    findings.extend(filter_count_iterate_findings(file, function));
    findings.extend(django_queryset_findings(file, function));
    findings.extend(django_n_plus_one_findings(file, function));
    findings.extend(django_loop_db_findings(file, function));
    findings.extend(django_values_findings(file, function));
    findings.extend(flask_handler_findings(file, function));
    findings.extend(fastapi_handler_findings(file, function));
    findings.extend(sqlalchemy_findings(file, function));
    findings.extend(sqlmodel_findings(file, function));
    findings.extend(celery_task_findings(file, function));
    findings.extend(click_typer_command_findings(file, function));
    findings.extend(pydantic_v2_findings(file, function));
    findings.extend(middleware_findings(file, function));
    findings.extend(handler_fanout_findings(file, function));
    findings.extend(template_response_findings(file, function));
    findings.extend(pandas_findings(file, function));
    findings.extend(numpy_findings(file, function));
    findings.extend(model_inference_findings(file, function));
    findings.extend(llm_findings(file, function));
    findings.extend(data_pipeline_findings(file, function));
    findings.extend(repeated_format_findings(file, function));
    findings.extend(json_encoder_recreated_findings(file, function));
    findings.extend(gzip_open_per_chunk_findings(file, function));
    findings.extend(pickle_in_loop_findings(file, function));
    findings.extend(isinstance_chain_findings(file, function));
    findings.extend(concat_in_comprehension_findings(file, function));
    findings.extend(tuple_unpacking_in_tight_loop_findings(file, function));
    findings.extend(django_extra_findings(file, function));
    findings.extend(response_extra_findings(file, function));
    findings.extend(mlops_extra_findings(file, function));
    findings
}

pub(crate) fn python_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(quality_file_findings(file));
    findings.extend(monolithic_init_module_findings(file));
    findings.extend(monolithic_module_findings(file));
    findings.extend(too_many_attributes_findings(file));
    findings.extend(god_class_findings(file));
    findings.extend(eager_constructor_collaborator_findings(file));
    findings.extend(over_abstracted_wrapper_findings(file));
    findings.extend(module_name_mismatch_findings(file));
    findings.extend(mixed_naming_convention_findings(file));
    findings.extend(unrelated_heavy_import_findings(file));
    findings.extend(obvious_commentary_findings(file));
    findings.extend(enthusiastic_commentary_findings(file));
    findings.extend(commented_out_code_findings(file));
    findings.extend(sync_async_module_findings(file));
    findings.extend(repeated_string_literal_findings(file));
    findings.extend(repeated_exception_block_findings(file));
    findings.extend(repeated_validation_pipeline_findings(file));
    findings
}

pub(crate) fn python_repo_findings(files: &[&ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(deep_inheritance_findings(files));
    findings.extend(tight_module_coupling_findings(files, index));
    findings.extend(cross_file_dupe_findings(files));
    findings.extend(test_utility_logic_findings(files));
    findings.extend(cross_file_literal_findings(files));
    findings.extend(duplicate_query_fragment_findings(files));
    findings.extend(duplicate_transformation_pipeline_findings(files));
    findings.extend(pyproject_repo_findings(files, index));
    findings
}
