use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

macro_rules! maint_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "maintainability",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: super::bindings::PYTHON_MAINTAINABILITY,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "broad_exception_handler",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Broad except Exception: style handlers that still obscure failure shape even when not fully swallowed.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "builtin_reduction_candidate",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Loop shapes that look like obvious sum, any, or all candidates.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "commented_out_code",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Blocks of commented-out source code left in production files.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "environment_boundary_without_fallback",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Environment-variable lookups that omit a default value or explicit failure handler.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "eval_exec_usage",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Direct eval() or exec() usage in non-test Python code.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "exception_swallowed",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Broad exception handlers like except: or except Exception: that immediately suppress the error with pass, continue, break, or return.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "external_input_without_validation",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Request or CLI entry points that trust external input without obvious validation or guard checks.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "hardcoded_business_rule",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Hardcoded threshold, rate-limit, or pricing-style literals assigned inside non-test Python functions.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "hardcoded_path_string",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Hardcoded filesystem path literals assigned inside non-test Python functions.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "magic_value_branching",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Repeated branch-shaping numeric or string literals that likely want an explicit constant or policy name.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "missing_context_manager",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Resource management (files, network connections) inside non-test Python functions that omits with-statement context managers.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "mixed_sync_async_module",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Modules that expose public sync and async entry points together.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "network_boundary_without_timeout",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Request, sync, or job-style Python functions that call HTTP boundaries with no obvious timeout or retry policy.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "none_comparison",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "== None or != None checks instead of is None or is not None.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "print_debugging_leftover",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "print() calls left in non-test Python functions that do not look like obvious main-entrypoint output.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "public_api_missing_type_hints",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Public Python functions that omit complete parameter or return annotations.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "redundant_return_none",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Explicit return None in simple code paths where Python would already return None implicitly.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "reinvented_utility",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Obvious locally implemented utility helpers that overlap with already-imported standard-library style helpers.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "side_effect_comprehension",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "List, set, or dicit comprehensions used as standalone statements where the result is discarded.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    RuleDefinition {
        id: "variadic_public_api",
        language: RuleLanguage::Python,
        family: "maintainability",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Public Python functions that expose *args or **kwargs instead of a clearer interface.",
        binding_location: super::bindings::PYTHON_MAINTAINABILITY,
    },
    maint_rule!(
        "magic_thresholds_duplicated_across_modules",
        "Flag repeated numeric thresholds with shared meaning that are copied across modules."
    ),
    maint_rule!(
        "tuple_return_with_three_or_more_positional_fields_in_public_api",
        "Flag public APIs that return positional tuples too wide to be self-documenting."
    ),
    maint_rule!(
        "parallel_lists_used_instead_of_record_object",
        "Flag code that keeps related values synchronized across multiple lists instead of using one record type."
    ),
    maint_rule!(
        "normalization_logic_duplicated_across_call_sites",
        "Flag the same value-cleaning or canonicalization logic repeated in several places."
    ),
    maint_rule!(
        "mutable_class_attribute_used_as_instance_storage",
        "Flag class attributes that accidentally store per-instance mutable state."
    ),
    maint_rule!(
        "helper_module_accumulates_unrelated_cross_domain_utilities",
        "Flag helper modules that mix string, filesystem, caching, parsing, and process logic in one place."
    ),
    maint_rule!(
        "string_sentinel_values_duplicated_instead_of_constant_or_enum",
        "Flag repeated mode or status strings that should be centralized."
    ),
    maint_rule!(
        "same_contextmanager_pattern_copied_across_modules",
        "Flag duplicate context manager behavior implemented in several modules."
    ),
    maint_rule!(
        "wrapper_function_only_renames_arguments_and_passthroughs_behavior",
        "Flag wrappers that add no policy, safety, or abstraction value."
    ),
    maint_rule!(
        "anonymous_dict_shape_repeated_without_shared_type_or_builder",
        "Flag repeated ad hoc dict payload shapes with no shared type or builder."
    ),
    maint_rule!(
        "branching_on_file_suffix_or_mode_string_scattered_across_codebase",
        "Flag suffix or mode dispatch logic duplicated across distant modules."
    ),
    maint_rule!(
        "hidden_dependency_arrives_via_import_time_side_effect",
        "Flag code that becomes initialized only because another import happened first."
    ),
    maint_rule!(
        "cache_object_exists_without_size_or_eviction_policy_documentation",
        "Flag caches that exist as long-lived process state with no documented retention policy."
    ),
    maint_rule!(
        "comment_required_to_explain_opaque_branching_that_code_could_express",
        "Flag code whose control flow stays unclear even with comments because the structure itself is weak."
    ),
    maint_rule!(
        "helper_returns_index_based_tuple_instead_of_named_structure",
        "Flag helper returns that require callers to remember tuple positions by convention."
    ),
    maint_rule!(
        "mixed_mutation_and_query_methods_share_same_manager_class",
        "Flag manager-style classes that both mutate and answer read-model queries with no boundary."
    ),
    maint_rule!(
        "monolithic_utils_module_becomes_default_dependency_sink",
        "Flag utility modules that attract unrelated dependencies and spread coupling."
    ),
    maint_rule!(
        "single_feature_requires_edits_in_many_unrelated_modules_due_to_scattered_policy",
        "Flag feature flows whose policy is fragmented across many unrelated modules."
    ),
];
