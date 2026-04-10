use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

macro_rules! hot_ext_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "hot_path_ext",
            default_severity: RuleDefaultSeverity::Info,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: super::bindings::PYTHON_HOTPATH_EXT,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "concatenation_in_comprehension_body",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "String or collection concatenation happens inside a comprehension body, creating avoidable churn.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "dict_copy_in_loop_same_source",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "A dict-like source is copied on each loop iteration instead of being reused or hoisted.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "gzip_open_per_chunk",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "gzip open/create calls are repeated per chunk instead of per stream.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "list_copy_in_loop_same_source",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "A list is copied on each loop iteration even though the source appears unchanged.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "nested_list_search_map_candidate",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Nested linear list searches that look like they want a temporary map or set index.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "path_resolve_or_expanduser_in_loop",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Path resolution helpers such as resolve() or expanduser() run inside loops on invariant inputs.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "pickle_dumps_in_loop_same_structure",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "pickle.dumps(...) is called repeatedly for the same structural shape in a loop.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "repeated_datetime_strptime_same_format",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "datetime.strptime(...) is repeated with the same format string instead of reusing a parsed shape or preprocessing once.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "repeated_dict_get_same_key_no_cache",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same dictionary key is fetched repeatedly instead of storing the value in a local binding.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "repeated_hashlib_new_same_algorithm",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same hashing algorithm is repeatedly constructed in a loop or tight path.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "repeated_isinstance_chain_same_object",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same object goes through repeated isinstance(...) checks that could be consolidated.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "repeated_list_index_lookup",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same list index lookup is performed repeatedly instead of caching the accessed value.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "repeated_locale_or_codec_lookup_in_loop",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Locale or codec lookups repeat inside loops instead of being cached once.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "repeated_string_format_invariant_template",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "An invariant string template is formatted repeatedly in a loop instead of being partially precomputed.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "set_created_per_iteration_same_elements",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "A set with the same elements is rebuilt on each iteration instead of being hoisted.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "sort_then_first_or_membership_only",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "A collection is sorted even though only the first element or a membership-style check is needed.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "string_join_without_generator",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "String joins that materialize an unnecessary list comprehension instead of using a generator or direct iterable.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "tuple_unpacking_in_tight_loop",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Tuple unpacking is repeated in tight loops where reducing per-iteration overhead may help.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "urlparse_in_loop_on_invariant_base",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "urlparse() or urlsplit() is repeated inside loops for invariant base values.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "xml_parse_same_payload_multiple_times",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same XML payload is parsed repeatedly within one function.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    RuleDefinition {
        id: "yaml_load_same_payload_multiple_times",
        language: RuleLanguage::Python,
        family: "hot_path_ext",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same YAML payload is parsed repeatedly within one function.",
        binding_location: super::bindings::PYTHON_HOTPATH_EXT,
    },
    hot_ext_rule!(
        "blocking_io_call_executed_per_item_without_batching",
        "Flag per-item blocking I/O where batching or grouping is possible."
    ),
    hot_ext_rule!(
        "repeated_directory_scan_inside_nested_loop",
        "Flag nested loops that rescan the same directory or file listing."
    ),
    hot_ext_rule!(
        "invariant_computation_not_hoisted_out_of_nested_loop",
        "Flag nested loops that recompute invariant values on every inner iteration."
    ),
    hot_ext_rule!(
        "any_or_all_wraps_list_comprehension_instead_of_generator",
        "Flag any([...]) and all([...]) forms that force unnecessary list materialization."
    ),
    hot_ext_rule!(
        "sum_max_min_wrap_list_comprehension_instead_of_generator",
        "Flag reductions that build transient lists instead of using generator expressions."
    ),
    hot_ext_rule!(
        "per_item_copy_of_large_config_or_context_object",
        "Flag loops that copy large config or context structures for each item despite read-mostly behavior."
    ),
    hot_ext_rule!(
        "same_sequence_scanned_multiple_times_for_related_aggregates",
        "Flag code that makes several full passes over the same data for related summaries."
    ),
    hot_ext_rule!(
        "generator_pipeline_materialized_between_each_transformation_stage",
        "Flag pipelines that repeatedly materialize intermediate lists between stages."
    ),
    hot_ext_rule!(
        "linear_search_helper_called_from_nested_loops",
        "Flag nested loops that call helpers performing linear scans on each iteration."
    ),
    hot_ext_rule!(
        "repeated_path_exists_check_before_open_or_replace_in_loop",
        "Flag loops that perform a separate existence check before every file operation."
    ),
    hot_ext_rule!(
        "serialization_then_deserialization_between_adjacent_helpers",
        "Flag neighboring helpers that bounce data through serialized text or bytes without need."
    ),
    hot_ext_rule!(
        "large_slice_copy_created_each_iteration_for_sliding_window",
        "Flag windowed algorithms that copy large slices every step instead of tracking indexes."
    ),
    hot_ext_rule!(
        "per_item_deduplication_uses_list_instead_of_hash_index",
        "Flag high-cardinality deduplication done with linear container membership checks."
    ),
    hot_ext_rule!(
        "expensive_sort_key_recomputed_without_preprojection",
        "Flag repeated expensive key computation during sorting when values could be precomputed once."
    ),
    hot_ext_rule!(
        "repeated_casefold_or_lower_calls_before_multiple_comparisons",
        "Flag code that normalizes the same candidate repeatedly before several comparisons."
    ),
    hot_ext_rule!(
        "formatted_log_or_debug_payload_built_for_each_item_without_guard",
        "Flag per-item log payload construction without checking whether the level is enabled."
    ),
    hot_ext_rule!(
        "repeated_open_read_close_of_same_small_file_in_single_workflow",
        "Flag workflows that reopen the same file many times instead of caching its contents or handle."
    ),
    hot_ext_rule!(
        "polling_loop_uses_tiny_sleep_instead_of_blocking_primitive",
        "Flag loops that wake repeatedly on a tiny sleep interval instead of a blocking queue, event, or condition."
    ),
    hot_ext_rule!(
        "invariant_template_or_prefix_string_reformatted_inside_loop",
        "Flag loops that reformat constant template fragments on each iteration."
    ),
    hot_ext_rule!(
        "lookup_table_derived_from_constants_rebuilt_per_invocation",
        "Flag functions that reconstruct lookup tables from static constants every call."
    ),
];
