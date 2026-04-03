use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use super::{
    ai_smells, duplication, framework, hotpath, hotpath_ext, maintainability, mlops, packaging,
    performance, quality, structure,
};

type FunctionEvaluator = fn(&ParsedFile, &ParsedFunction) -> Vec<Finding>;
type FileEvaluator = fn(&ParsedFile) -> Vec<Finding>;
type RepoEvaluator = fn(&[&ParsedFile], &RepositoryIndex) -> Vec<Finding>;

#[allow(dead_code)]
pub(super) struct PythonFunctionRuleSpec {
    pub(super) family: &'static str,
    pub(super) rule_ids: &'static [&'static str],
    pub(super) evaluate: FunctionEvaluator,
}

#[allow(dead_code)]
pub(super) struct PythonFileRuleSpec {
    pub(super) family: &'static str,
    pub(super) rule_ids: &'static [&'static str],
    pub(super) evaluate: FileEvaluator,
}

#[allow(dead_code)]
pub(super) struct PythonRepoRuleSpec {
    pub(super) family: &'static str,
    pub(super) rule_ids: &'static [&'static str],
    pub(super) evaluate: RepoEvaluator,
}

const QUALITY_FUNCTION_RULE_IDS: &[&str] = &[
    "async_lock_held_across_await",
    "async_retry_sleep_without_backoff",
    "background_task_exception_unobserved",
    "dataclass_heavy_post_init",
    "mutable_default_argument",
    "pickle_deserialization_boundary",
    "public_any_type_leak",
    "subprocess_shell_true",
    "tar_extractall_unfiltered",
    "tempfile_without_cleanup",
    "typeddict_unchecked_access",
    "unsafe_yaml_loader",
    "untracked_asyncio_task",
];

const PERFORMANCE_FUNCTION_RULE_IDS: &[&str] = &[
    "blocking_sync_io_in_async",
    "deque_candidate_queue",
    "full_dataset_load",
    "list_materialization_first_element",
    "list_membership_in_loop",
    "recursive_traversal_risk",
    "repeated_len_in_loop",
    "string_concat_in_loop",
    "temporary_collection_in_loop",
];

const MAINTAINABILITY_FUNCTION_RULE_IDS: &[&str] = &[
    "broad_exception_handler",
    "builtin_reduction_candidate",
    "environment_boundary_without_fallback",
    "eval_exec_usage",
    "exception_swallowed",
    "external_input_without_validation",
    "hardcoded_business_rule",
    "hardcoded_path_string",
    "magic_value_branching",
    "missing_context_manager",
    "network_boundary_without_timeout",
    "none_comparison",
    "print_debugging_leftover",
    "public_api_missing_type_hints",
    "redundant_return_none",
    "reinvented_utility",
    "side_effect_comprehension",
    "variadic_public_api",
];

const STRUCTURE_FUNCTION_RULE_IDS: &[&str] = &[
    "god_function",
    "mixed_concerns_function",
    "name_responsibility_mismatch",
];

const AI_SMELLS_FUNCTION_RULE_IDS: &[&str] = &["textbook_docstring_small_helper"];

const HOTPATH_FUNCTION_RULE_IDS: &[&str] = &[
    "csv_writer_flush_per_row",
    "dict_items_or_keys_materialized_in_loop",
    "enumerate_on_range_len",
    "in_check_on_list_literal",
    "json_loads_same_payload_multiple_times",
    "list_comprehension_only_for_length",
    "read_then_splitlines",
    "readlines_then_iterate",
    "regex_compile_in_hot_path",
    "repeated_json_dumps_same_object",
    "repeated_open_same_file_in_function",
    "sorted_only_for_first_element",
    "string_startswith_endswith_chain",
    "write_without_buffering_in_loop",
];

const HOTPATH_EXT_FUNCTION_RULE_IDS: &[&str] = &[
    "append_then_sort_each_iteration",
    "concatenation_in_comprehension_body",
    "dict_copy_in_loop_same_source",
    "filter_then_count_then_iterate",
    "gzip_open_per_chunk",
    "json_encoder_recreated_per_item",
    "list_copy_in_loop_same_source",
    "nested_list_search_map_candidate",
    "path_resolve_or_expanduser_in_loop",
    "pickle_dumps_in_loop_same_structure",
    "repeated_datetime_strptime_same_format",
    "repeated_dict_get_same_key_no_cache",
    "repeated_hashlib_new_same_algorithm",
    "repeated_isinstance_chain_same_object",
    "repeated_list_index_lookup",
    "repeated_locale_or_codec_lookup_in_loop",
    "repeated_string_format_invariant_template",
    "set_created_per_iteration_same_elements",
    "sort_then_first_or_membership_only",
    "string_join_without_generator",
    "tuple_unpacking_in_tight_loop",
    "urlparse_in_loop_on_invariant_base",
    "xml_parse_same_payload_multiple_times",
    "yaml_load_same_payload_multiple_times",
];

const FRAMEWORK_FUNCTION_RULE_IDS: &[&str] = &[
    "celery_delay_in_loop_without_canvas",
    "celery_result_get_inside_task",
    "celery_task_reads_env_per_invocation",
    "click_typer_config_file_loaded_per_command",
    "click_typer_env_lookup_per_command",
    "click_typer_http_client_created_per_command",
    "django_all_without_limit_in_view",
    "django_create_single_in_loop",
    "django_delete_single_in_loop",
    "django_migration_code_in_view",
    "django_n_plus_one_no_select_related",
    "django_queryset_count_then_exists",
    "django_queryset_evaluated_multiple_times",
    "django_queryset_len_instead_of_count",
    "django_queryset_order_by_random",
    "django_raw_sql_in_loop",
    "django_save_full_model_in_loop",
    "django_update_single_in_loop",
    "django_values_vs_full_model_in_loop",
    "fastapi_background_task_exception_silent",
    "fastapi_dependency_creates_client_per_request",
    "fastapi_response_model_without_orm_mode",
    "fastapi_sync_def_with_blocking_io",
    "flask_app_config_read_per_request",
    "flask_debug_mode_in_production_code",
    "flask_file_read_per_request",
    "flask_global_db_connection_per_request",
    "flask_json_encoder_per_request",
    "flask_no_streaming_for_large_response",
    "flask_request_body_parsed_multiple_times",
    "flask_template_rendered_from_string_in_view",
    "large_dict_literal_response_in_handler",
    "middleware_compiles_regex_per_request",
    "middleware_creates_http_client_per_request",
    "middleware_loads_config_file_per_request",
    "pydantic_model_dump_then_json_dumps",
    "pydantic_model_validate_after_json_loads",
    "response_json_dumps_then_response_object",
    "sqlalchemy_commit_per_row_in_loop",
    "sqlalchemy_create_engine_per_request",
    "sqlalchemy_expire_on_commit_default_in_async",
    "sqlalchemy_n_plus_one_lazy_load",
    "sqlalchemy_query_in_loop",
    "sqlalchemy_session_not_closed",
    "sqlmodel_commit_per_row_in_loop",
    "sqlmodel_session_exec_in_loop",
    "sqlmodel_unbounded_select_in_handler",
    "template_render_in_loop",
    "upstream_call_without_timeout_in_handler",
    "upstream_http_call_per_item_in_handler",
    "upstream_response_not_checked_before_decode",
];

const MLOPS_FUNCTION_RULE_IDS: &[&str] = &[
    "data_pipeline_no_error_handling",
    "dataset_not_using_dataloader",
    "embedding_computed_per_request",
    "embedding_dimension_mismatch_silent",
    "entire_dataframe_copied_for_transform",
    "global_state_in_data_pipeline",
    "gpu_memory_not_cleared_between_experiments",
    "hardcoded_api_key_in_source",
    "intermediate_dataframe_not_freed",
    "langchain_chain_built_per_request",
    "llm_api_call_in_loop_without_batching",
    "llm_full_response_loaded_into_memory",
    "llm_response_not_cached_same_input",
    "model_eval_mode_missing",
    "model_loaded_per_request",
    "model_to_device_in_loop",
    "no_schema_validation_on_external_data",
    "numpy_append_in_loop",
    "numpy_dtype_mismatch_implicit_cast",
    "numpy_python_loop_over_array",
    "numpy_tolist_in_hot_path",
    "numpy_vstack_hstack_in_loop",
    "pandas_apply_with_simple_vectorizable_op",
    "pandas_chain_assignment_warning",
    "pandas_concat_in_loop",
    "pandas_copy_in_loop",
    "pandas_eval_string_manipulation",
    "pandas_full_dataframe_print_in_production",
    "pandas_inplace_false_reassignment_missing",
    "pandas_iterrows_in_loop",
    "pandas_merge_without_validation",
    "pandas_read_csv_without_dtypes",
    "pandas_read_without_chunksize_large_file",
    "pandas_to_dict_records_in_loop",
    "print_metrics_instead_of_logging",
    "prompt_template_string_concat_in_loop",
    "random_seed_not_set",
    "retry_on_rate_limit_without_backoff",
    "token_count_not_checked_before_api_call",
    "tokenizer_encode_in_loop_without_cache",
    "tokenizer_loaded_per_request",
    "torch_no_grad_missing_in_inference",
    "training_loop_without_zero_grad",
    "vector_store_client_created_per_request",
    "wandb_mlflow_log_in_tight_loop",
];

const PACKAGING_FUNCTION_RULE_IDS: &[&str] = &["python_public_api_any_contract"];

const QUALITY_FILE_RULE_IDS: &[&str] = &[
    "dataclass_mutable_default",
    "import_time_config_load",
    "import_time_file_io",
    "import_time_network_call",
    "import_time_subprocess",
    "module_singleton_client_side_effect",
    "mutable_module_global_state",
    "option_bag_model",
    "public_any_type_leak",
];

const STRUCTURE_FILE_RULE_IDS: &[&str] = &[
    "eager_constructor_collaborators",
    "god_class",
    "monolithic_init_module",
    "monolithic_module",
    "name_responsibility_mismatch",
    "over_abstracted_wrapper",
    "too_many_instance_attributes",
];

const AI_SMELLS_FILE_RULE_IDS: &[&str] = &[
    "enthusiastic_commentary",
    "mixed_naming_conventions",
    "obvious_commentary",
    "unrelated_heavy_import",
];

const MAINTAINABILITY_FILE_RULE_IDS: &[&str] = &["commented_out_code", "mixed_sync_async_module"];

const DUPLICATION_FILE_RULE_IDS: &[&str] = &[
    "duplicate_error_handler_block",
    "duplicate_validation_pipeline",
    "repeated_string_literal",
];

const STRUCTURE_REPO_RULE_IDS: &[&str] = &["deep_inheritance_hierarchy", "tight_module_coupling"];

const DUPLICATION_REPO_RULE_IDS: &[&str] = &[
    "cross_file_copy_paste_function",
    "cross_file_repeated_literal",
    "duplicate_query_fragment",
    "duplicate_test_utility_logic",
    "duplicate_transformation_pipeline",
];

const PACKAGING_REPO_RULE_IDS: &[&str] = &[
    "cross_package_internal_import",
    "pyproject_missing_requires_python",
    "pyproject_script_entrypoint_unresolved",
];

pub(super) const FUNCTION_RULE_SPECS: &[PythonFunctionRuleSpec] = &[
    PythonFunctionRuleSpec {
        family: "quality",
        rule_ids: QUALITY_FUNCTION_RULE_IDS,
        evaluate: quality::quality_function_findings,
    },
    PythonFunctionRuleSpec {
        family: "performance",
        rule_ids: PERFORMANCE_FUNCTION_RULE_IDS,
        evaluate: performance_family_findings,
    },
    PythonFunctionRuleSpec {
        family: "maintainability",
        rule_ids: MAINTAINABILITY_FUNCTION_RULE_IDS,
        evaluate: maintainability_family_findings,
    },
    PythonFunctionRuleSpec {
        family: "structure",
        rule_ids: STRUCTURE_FUNCTION_RULE_IDS,
        evaluate: structure_function_family_findings,
    },
    PythonFunctionRuleSpec {
        family: "ai_smells",
        rule_ids: AI_SMELLS_FUNCTION_RULE_IDS,
        evaluate: ai_smells_function_family_findings,
    },
    PythonFunctionRuleSpec {
        family: "hotpath",
        rule_ids: HOTPATH_FUNCTION_RULE_IDS,
        evaluate: hotpath_family_findings,
    },
    PythonFunctionRuleSpec {
        family: "hotpath_ext",
        rule_ids: HOTPATH_EXT_FUNCTION_RULE_IDS,
        evaluate: hotpath_ext_family_findings,
    },
    PythonFunctionRuleSpec {
        family: "framework",
        rule_ids: FRAMEWORK_FUNCTION_RULE_IDS,
        evaluate: framework_family_findings,
    },
    PythonFunctionRuleSpec {
        family: "mlops",
        rule_ids: MLOPS_FUNCTION_RULE_IDS,
        evaluate: mlops_family_findings,
    },
    PythonFunctionRuleSpec {
        family: "packaging",
        rule_ids: PACKAGING_FUNCTION_RULE_IDS,
        evaluate: packaging::public_api_any_contract_findings,
    },
];

pub(super) const FILE_RULE_SPECS: &[PythonFileRuleSpec] = &[
    PythonFileRuleSpec {
        family: "quality",
        rule_ids: QUALITY_FILE_RULE_IDS,
        evaluate: quality::quality_file_findings,
    },
    PythonFileRuleSpec {
        family: "structure",
        rule_ids: STRUCTURE_FILE_RULE_IDS,
        evaluate: structure_file_family_findings,
    },
    PythonFileRuleSpec {
        family: "ai_smells",
        rule_ids: AI_SMELLS_FILE_RULE_IDS,
        evaluate: ai_smells_file_family_findings,
    },
    PythonFileRuleSpec {
        family: "maintainability",
        rule_ids: MAINTAINABILITY_FILE_RULE_IDS,
        evaluate: maintainability_file_family_findings,
    },
    PythonFileRuleSpec {
        family: "duplication",
        rule_ids: DUPLICATION_FILE_RULE_IDS,
        evaluate: duplication_file_family_findings,
    },
];

pub(super) const REPO_RULE_SPECS: &[PythonRepoRuleSpec] = &[
    PythonRepoRuleSpec {
        family: "structure",
        rule_ids: STRUCTURE_REPO_RULE_IDS,
        evaluate: structure_repo_family_findings,
    },
    PythonRepoRuleSpec {
        family: "duplication",
        rule_ids: DUPLICATION_REPO_RULE_IDS,
        evaluate: duplication_repo_family_findings,
    },
    PythonRepoRuleSpec {
        family: "packaging",
        rule_ids: PACKAGING_REPO_RULE_IDS,
        evaluate: packaging::pyproject_repo_findings,
    },
];

const PERFORMANCE_EVALUATORS: &[FunctionEvaluator] = &[
    performance::string_concat_findings,
    performance::blocking_sync_io_findings,
    performance::full_dataset_load_findings,
    performance::list_materialization_findings,
    performance::deque_candidate_findings,
    performance::temp_collection_findings,
    performance::recursive_traversal_findings,
    performance::list_membership_findings,
    performance::repeated_len_findings,
];

const MAINTAINABILITY_EVALUATORS: &[FunctionEvaluator] = &[
    maintainability::exception_swallowed_findings,
    maintainability::broad_exception_handler_findings,
    maintainability::eval_exec_findings,
    maintainability::print_debugging_findings,
    maintainability::none_comparison_findings,
    maintainability::side_effect_comprehension_findings,
    maintainability::redundant_return_none_findings,
    maintainability::hardcoded_path_findings,
    maintainability::hardcoded_business_rule_findings,
    maintainability::magic_value_branching_findings,
    maintainability::reinvented_utility_findings,
    maintainability::builtin_reduction_findings,
    maintainability::missing_context_manager_findings,
    maintainability::network_timeout_findings,
    maintainability::env_fallback_findings,
    maintainability::input_validation_findings,
    maintainability::api_type_hint_findings,
    maintainability::variadic_public_api_findings,
];

const STRUCTURE_FUNCTION_EVALUATORS: &[FunctionEvaluator] = &[
    structure::god_function_findings,
    structure::mixed_concern_findings,
    structure::name_responsibility_mismatch_findings,
];

const HOTPATH_EVALUATORS: &[FunctionEvaluator] = &[
    hotpath::regex_compile_in_hotpath_findings,
    hotpath::json_repeated_call_findings,
    hotpath::sorted_first_findings,
    hotpath::len_comprehension_findings,
    hotpath::readlines_then_iterate_findings,
    hotpath::read_splitlines_findings,
    hotpath::in_list_literal_findings,
    hotpath::startswith_chain_findings,
    hotpath::enumerate_range_len_findings,
    hotpath::csv_flush_per_row_findings,
    hotpath::write_in_loop_findings,
    hotpath::repeated_open_findings,
    hotpath::dict_materialization_in_loop_findings,
];

const HOTPATH_EXT_EVALUATORS: &[FunctionEvaluator] = &[
    hotpath_ext::yaml_repeated_load_findings,
    hotpath_ext::xml_repeated_parse_findings,
    hotpath_ext::datetime_strptime_repeated_findings,
    hotpath_ext::hashlib_repeated_findings,
    hotpath_ext::copy_in_loop_findings,
    hotpath_ext::invariant_call_in_loop_findings,
    hotpath_ext::index_in_loop_findings,
    hotpath_ext::append_sort_in_loop_findings,
    hotpath_ext::join_list_comp_findings,
    hotpath_ext::repeated_subscript_findings,
    hotpath_ext::nested_list_search_findings,
    hotpath_ext::sort_then_first_findings,
    hotpath_ext::filter_count_iterate_findings,
    hotpath_ext::repeated_format_findings,
    hotpath_ext::json_encoder_recreated_findings,
    hotpath_ext::gzip_open_per_chunk_findings,
    hotpath_ext::pickle_in_loop_findings,
    hotpath_ext::isinstance_chain_findings,
    hotpath_ext::concat_in_comprehension_findings,
    hotpath_ext::tuple_unpacking_in_tight_loop_findings,
];

const FRAMEWORK_EVALUATORS: &[FunctionEvaluator] = &[
    framework::celery_task_findings,
    framework::click_typer_command_findings,
    framework::django_queryset_findings,
    framework::django_n_plus_one_findings,
    framework::django_loop_db_findings,
    framework::django_values_findings,
    framework::flask_handler_findings,
    framework::fastapi_handler_findings,
    framework::sqlalchemy_findings,
    framework::sqlmodel_findings,
    framework::pydantic_v2_findings,
    framework::middleware_findings,
    framework::handler_fanout_findings,
    framework::template_response_findings,
    framework::django_extra_findings,
    framework::response_extra_findings,
];

const MLOPS_EVALUATORS: &[FunctionEvaluator] = &[
    mlops::pandas_findings,
    mlops::numpy_findings,
    mlops::model_inference_findings,
    mlops::llm_findings,
    mlops::data_pipeline_findings,
    mlops::mlops_extra_findings,
];

const STRUCTURE_FILE_EVALUATORS: &[FileEvaluator] = &[
    structure::monolithic_init_module_findings,
    structure::monolithic_module_findings,
    structure::too_many_attributes_findings,
    structure::god_class_findings,
    structure::eager_constructor_collaborator_findings,
    structure::over_abstracted_wrapper_findings,
    structure::module_name_mismatch_findings,
];

const AI_SMELLS_FILE_EVALUATORS: &[FileEvaluator] = &[
    ai_smells::mixed_naming_convention_findings,
    ai_smells::unrelated_heavy_import_findings,
    ai_smells::obvious_commentary_findings,
    ai_smells::enthusiastic_commentary_findings,
];

const MAINTAINABILITY_FILE_EVALUATORS: &[FileEvaluator] = &[
    maintainability::commented_out_code_findings,
    maintainability::sync_async_module_findings,
];

const DUPLICATION_FILE_EVALUATORS: &[FileEvaluator] = &[
    duplication::repeated_string_literal_findings,
    duplication::repeated_exception_block_findings,
    duplication::repeated_validation_pipeline_findings,
];

const STRUCTURE_REPO_EVALUATORS: &[RepoEvaluator] = &[
    deep_inheritance_repo_findings,
    structure::tight_module_coupling_findings,
];

const DUPLICATION_REPO_EVALUATORS: &[RepoEvaluator] = &[
    cross_file_dupe_repo_findings,
    test_utility_logic_repo_findings,
    cross_file_literal_repo_findings,
    duplicate_query_fragment_repo_findings,
    duplicate_transformation_pipeline_repo_findings,
];

pub(super) fn evaluate_function_specs(
    specs: &[PythonFunctionRuleSpec],
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for spec in specs {
        findings.extend((spec.evaluate)(file, function));
    }
    findings
}

pub(super) fn evaluate_file_specs(specs: &[PythonFileRuleSpec], file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    for spec in specs {
        findings.extend((spec.evaluate)(file));
    }
    findings
}

pub(super) fn evaluate_repo_specs(
    specs: &[PythonRepoRuleSpec],
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for spec in specs {
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

fn ai_smells_function_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    ai_smells::textbook_docstring_findings(file, function)
}

fn performance_family_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    evaluate_function_group(PERFORMANCE_EVALUATORS, file, function)
}

fn maintainability_family_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    evaluate_function_group(MAINTAINABILITY_EVALUATORS, file, function)
}

fn structure_function_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    evaluate_function_group(STRUCTURE_FUNCTION_EVALUATORS, file, function)
}

fn hotpath_family_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    evaluate_function_group(HOTPATH_EVALUATORS, file, function)
}

fn hotpath_ext_family_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    evaluate_function_group(HOTPATH_EXT_EVALUATORS, file, function)
}

fn framework_family_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    evaluate_function_group(FRAMEWORK_EVALUATORS, file, function)
}

fn mlops_family_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    evaluate_function_group(MLOPS_EVALUATORS, file, function)
}

fn structure_file_family_findings(file: &ParsedFile) -> Vec<Finding> {
    evaluate_file_group(STRUCTURE_FILE_EVALUATORS, file)
}

fn ai_smells_file_family_findings(file: &ParsedFile) -> Vec<Finding> {
    evaluate_file_group(AI_SMELLS_FILE_EVALUATORS, file)
}

fn maintainability_file_family_findings(file: &ParsedFile) -> Vec<Finding> {
    evaluate_file_group(MAINTAINABILITY_FILE_EVALUATORS, file)
}

fn duplication_file_family_findings(file: &ParsedFile) -> Vec<Finding> {
    evaluate_file_group(DUPLICATION_FILE_EVALUATORS, file)
}

fn structure_repo_family_findings(files: &[&ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    evaluate_repo_group(STRUCTURE_REPO_EVALUATORS, files, index)
}

fn duplication_repo_family_findings(
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    evaluate_repo_group(DUPLICATION_REPO_EVALUATORS, files, index)
}

fn deep_inheritance_repo_findings(files: &[&ParsedFile], _index: &RepositoryIndex) -> Vec<Finding> {
    structure::deep_inheritance_findings(files)
}

fn cross_file_dupe_repo_findings(files: &[&ParsedFile], _index: &RepositoryIndex) -> Vec<Finding> {
    duplication::cross_file_dupe_findings(files)
}

fn test_utility_logic_repo_findings(
    files: &[&ParsedFile],
    _index: &RepositoryIndex,
) -> Vec<Finding> {
    duplication::test_utility_logic_findings(files)
}

fn cross_file_literal_repo_findings(
    files: &[&ParsedFile],
    _index: &RepositoryIndex,
) -> Vec<Finding> {
    duplication::cross_file_literal_findings(files)
}

fn duplicate_query_fragment_repo_findings(
    files: &[&ParsedFile],
    _index: &RepositoryIndex,
) -> Vec<Finding> {
    duplication::duplicate_query_fragment_findings(files)
}

fn duplicate_transformation_pipeline_repo_findings(
    files: &[&ParsedFile],
    _index: &RepositoryIndex,
) -> Vec<Finding> {
    duplication::duplicate_transformation_pipeline_findings(files)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::rules::{RuleLanguage, rule_registry};

    use super::{FILE_RULE_SPECS, FUNCTION_RULE_SPECS, REPO_RULE_SPECS};

    #[test]
    fn python_rule_specs_cover_registry() {
        let mut spec_rule_counts = BTreeMap::<&str, usize>::new();
        let spec_rule_ids = FUNCTION_RULE_SPECS
            .iter()
            .flat_map(|spec| spec.rule_ids.iter().copied())
            .chain(
                FILE_RULE_SPECS
                    .iter()
                    .flat_map(|spec| spec.rule_ids.iter().copied()),
            )
            .chain(
                REPO_RULE_SPECS
                    .iter()
                    .flat_map(|spec| spec.rule_ids.iter().copied()),
            )
            .inspect(|rule_id| {
                *spec_rule_counts.entry(*rule_id).or_default() += 1;
            })
            .collect::<BTreeSet<_>>();

        let registry_rule_ids = rule_registry()
            .iter()
            .filter(|metadata| metadata.language == RuleLanguage::Python)
            .map(|metadata| metadata.id.as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(spec_rule_ids, registry_rule_ids);

        let duplicate_rule_ids = spec_rule_counts
            .into_iter()
            .filter_map(|(rule_id, count)| (count > 1).then_some(rule_id))
            .collect::<Vec<_>>();
        assert_eq!(
            duplicate_rule_ids,
            vec!["name_responsibility_mismatch", "public_any_type_leak"]
        );
    }

    #[test]
    fn every_python_rule_spec_declares_its_owned_rules() {
        for spec in FUNCTION_RULE_SPECS {
            assert!(
                !spec.rule_ids.is_empty(),
                "function spec family {} should own at least one rule",
                spec.family
            );
        }

        for spec in FILE_RULE_SPECS {
            assert!(
                !spec.rule_ids.is_empty(),
                "file spec family {} should own at least one rule",
                spec.family
            );
        }

        for spec in REPO_RULE_SPECS {
            assert!(
                !spec.rule_ids.is_empty(),
                "repo spec family {} should own at least one rule",
                spec.family
            );
        }
    }
}
