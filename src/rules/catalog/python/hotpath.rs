use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

macro_rules! hot_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "hot_path",
            default_severity: RuleDefaultSeverity::Info,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: super::bindings::PYTHON_HOTPATH,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "dict_items_or_keys_materialized_in_loop",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "dict.items(), keys(), or values() are repeatedly materialized inside loops.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "enumerate_on_range_len",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "enumerate(range(len(...))) style loops that add indexing ceremony without extra value.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "in_check_on_list_literal",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Membership tests against list literals where a tuple or set would be clearer or cheaper.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "json_loads_same_payload_multiple_times",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same JSON payload is decoded multiple times inside one function instead of caching the parsed value.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "list_comprehension_only_for_length",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "A list comprehension is built only so len(...) can be called on it.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "read_then_splitlines",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "File contents are fully read and then splitlines() is called instead of streaming lines.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "readlines_then_iterate",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "readlines() materializes the whole file before line-by-line iteration.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "regex_compile_in_hot_path",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "re.compile(...) or similar regex compilation repeated inside hot code paths.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "repeated_json_dumps_same_object",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "json.dumps(...) is repeated for the same object instead of caching the serialized value.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "repeated_open_same_file_in_function",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same file appears to be opened multiple times within one function.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "sorted_only_for_first_element",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "A sequence is fully sorted even though only the first or smallest element is used.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "string_startswith_endswith_chain",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Repeated startswith(...) or endswith(...) checks that can often be combined into tuple-based calls.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    RuleDefinition {
        id: "write_without_buffering_in_loop",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Repeated writes inside loops with no visible buffering or batching.",
        binding_location: super::bindings::PYTHON_HOTPATH,
    },
    hot_rule!(
        "regex_compiled_on_each_hot_call",
        "Flag hot functions that compile the same regular expression repeatedly instead of reusing a compiled pattern."
    ),
    hot_rule!(
        "json_roundtrip_used_for_object_copy",
        "Flag code that serializes and deserializes objects only to make a copy."
    ),
    hot_rule!(
        "repeated_datetime_parse_inside_loop",
        "Flag loops that parse timestamps repeatedly when values could be normalized once."
    ),
    hot_rule!(
        "repeated_split_or_join_on_invariant_separator_inside_loop",
        "Flag loops that repeatedly rebuild the same separator-driven string operations."
    ),
    hot_rule!(
        "repeated_attribute_chain_lookup_inside_tight_loop",
        "Flag tight loops that repeatedly traverse the same attribute chain instead of binding a local reference."
    ),
    hot_rule!(
        "exception_used_for_expected_lookup_miss_in_loop",
        "Flag loops that rely on exceptions for common cache or dict misses."
    ),
    hot_rule!(
        "membership_test_against_list_or_tuple_literal_inside_loop",
        "Flag repeated membership tests against small linear containers in a hot path."
    ),
    hot_rule!(
        "incremental_list_or_tuple_concatenation_in_accumulation_loop",
        "Flag accumulation loops that repeatedly concatenate instead of appending and joining once."
    ),
    hot_rule!(
        "constant_frozenset_or_dict_rebuilt_on_each_call",
        "Flag frequent call paths that reconstruct constant lookup tables each invocation."
    ),
    hot_rule!(
        "function_local_import_executed_in_frequent_path",
        "Flag hot call paths that perform imports inside the function body."
    ),
    hot_rule!(
        "pathlib_path_reconstructed_from_same_base_in_loop",
        "Flag loops that rebuild the same base Path object every iteration."
    ),
    hot_rule!(
        "environment_lookup_repeated_in_hot_path",
        "Flag hot functions that repeatedly read environment variables or global config values that do not change."
    ),
    hot_rule!(
        "repeated_normalization_of_same_string_in_loop",
        "Flag repeated .strip(), .lower(), .casefold(), or similar normalization on the same value inside a loop."
    ),
    hot_rule!(
        "full_sort_performed_inside_outer_iteration",
        "Flag outer loops that sort a full collection during each iteration."
    ),
    hot_rule!(
        "list_of_keys_materialized_for_membership_check",
        "Flag code that builds list(mapping.keys()) before checking membership."
    ),
    hot_rule!(
        "lambda_or_closure_allocated_per_item_when_static_helper_suffices",
        "Flag per-item closure allocation in hot loops when a stable helper would work."
    ),
    hot_rule!(
        "iterator_materialized_to_list_before_single_pass_loop",
        "Flag iterators converted to lists in hot paths when only one pass is needed."
    ),
    hot_rule!(
        "subprocess_or_shell_call_inside_record_processing_loop",
        "Flag per-record subprocess work that should be batched or hoisted."
    ),
    hot_rule!(
        "repeated_pure_helper_call_on_same_input_without_local_cache",
        "Flag hot loops that recompute the same pure helper result for identical inputs in one scope."
    ),
    hot_rule!(
        "same_buffer_or_prefix_reencoded_each_iteration",
        "Flag loops that re-encode identical prefixes or headers on each iteration."
    ),
];
