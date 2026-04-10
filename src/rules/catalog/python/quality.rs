use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

macro_rules! quality_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "quality",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: super::bindings::PYTHON_QUALITY,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "async_lock_held_across_await",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Async lock scopes or explicit acquire/release regions that continue across unrelated await points.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "async_retry_sleep_without_backoff",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Retry-style async loops that sleep a fixed interval without visible backoff, jitter, or bounded retry policy.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "background_task_exception_unobserved",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Background task bindings with no obvious await, callback, supervisor, or observation path.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "dataclass_heavy_post_init",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Dataclass __post_init__ methods that perform I/O, subprocess, network, or heavyweight client setup.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "dataclass_mutable_default",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Dataclass fields that use mutable defaults instead of default_factory.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "import_time_config_load",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Module-scope configuration or secret loading that runs during import instead of an explicit startup path.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "import_time_file_io",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Module-scope file reads, writes, or directory scans that happen during import.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "import_time_network_call",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Module-scope HTTP or socket calls executed while the module is imported.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "import_time_subprocess",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Subprocess launches triggered from module scope during import.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "module_singleton_client_side_effect",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Eagerly constructed network, database, or cloud clients bound at module scope.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "mutable_default_argument",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Function parameters that use mutable defaults such as [], {}, or set() directly in the signature.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "mutable_module_global_state",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Mutable module globals updated from multiple functions.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "option_bag_model",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Dataclass or TypedDict models that accumulate many optional fields and boolean switches.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "pickle_deserialization_boundary",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "pickle.load(s) or dill.load(s) style deserialization in production code.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "public_any_type_leak",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Public functions or model fields that expose Any, object, or similarly wide contracts.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "subprocess_shell_true",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Subprocess boundaries that enable shell=True.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "tar_extractall_unfiltered",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "tarfile.extractall(...) without an obvious filter, members list, or path-validation helper.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "tempfile_without_cleanup",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Temporary files or directories created without a visible cleanup or context-manager ownership path.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "typeddict_unchecked_access",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Direct indexing of optional TypedDict keys without an obvious guard path.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "unsafe_yaml_loader",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "yaml.load(...) or full_load(...) style loaders used where safe loading is more appropriate.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    RuleDefinition {
        id: "untracked_asyncio_task",
        language: RuleLanguage::Python,
        family: "quality",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "asyncio.create_task(...) or similar task creation whose handle is discarded immediately.",
        binding_location: super::bindings::PYTHON_QUALITY,
    },
    quality_rule!(
        "public_api_returns_none_or_value_without_explicit_optional_contract",
        "Flag public APIs that sometimes return None and sometimes a value without an explicit contract."
    ),
    quality_rule!(
        "fallback_branch_swallows_invariant_violation_and_returns_plausible_default",
        "Flag fallback logic that hides broken invariants by returning believable defaults."
    ),
    quality_rule!(
        "broad_except_used_to_mask_type_or_shape_bug",
        "Flag broad exception handling that conceals structural bugs in the input or code."
    ),
    quality_rule!(
        "order_dependent_set_to_list_conversion_exposed_in_public_result",
        "Flag public outputs that rely on unstable set iteration order."
    ),
    quality_rule!(
        "default_timeout_missing_on_external_boundary_wrapper",
        "Flag wrappers around external boundaries that omit a timeout policy."
    ),
    quality_rule!(
        "float_equality_controls_branching_on_computed_values",
        "Flag control flow that depends on exact float equality after computation."
    ),
    quality_rule!(
        "recursive_walk_over_untrusted_input_lacks_depth_limit",
        "Flag recursion over caller-provided structures with no depth guard."
    ),
    quality_rule!(
        "public_iterator_yields_heterogeneous_item_shapes",
        "Flag iterators that yield different shapes or types without an explicit sum-type contract."
    ),
    quality_rule!(
        "partial_update_mutates_input_before_validation_succeeds",
        "Flag update helpers that mutate caller data before all validation passes."
    ),
    quality_rule!(
        "cache_key_derived_from_stringified_mutable_object",
        "Flag cache keys built from unstable string forms of mutable objects."
    ),
    quality_rule!(
        "sort_order_depends_on_non_explicit_mapping_iteration_semantics",
        "Flag ordering logic that depends on implicit mapping iteration instead of explicit keys."
    ),
    quality_rule!(
        "duplicate_items_silently_dropped_without_contract_signal",
        "Flag code that deduplicates caller data without making that behavior explicit."
    ),
    quality_rule!(
        "timezone_naive_datetime_accepted_in_public_contract",
        "Flag public contracts that accept datetimes with ambiguous timezone semantics."
    ),
    quality_rule!(
        "atomic_replace_semantics_implemented_with_non_atomic_file_write",
        "Flag code that intends atomic replacement but uses non-atomic file writes."
    ),
    quality_rule!(
        "string_mode_parameter_replaces_enum_or_literal_contract",
        "Flag string mode parameters that should be constrained by an enum or literal contract."
    ),
    quality_rule!(
        "helper_returns_success_shape_even_when_substeps_partially_fail",
        "Flag helpers that claim success while hiding partial failures."
    ),
    quality_rule!(
        "comparison_or_merge_logic_assumes_unique_keys_without_assertion",
        "Flag merge logic that silently assumes uniqueness of keys or identifiers."
    ),
    quality_rule!(
        "validation_only_happens_after_expensive_side_effect_has_started",
        "Flag flows where validation happens only after I/O or irreversible work has begun."
    ),
];
