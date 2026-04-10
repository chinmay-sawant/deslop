use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

macro_rules! struct_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "structure",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: super::bindings::PYTHON_STRUCTURE,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "deep_inheritance_hierarchy",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Repository-local Python class chains with unusually deep inheritance depth.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    RuleDefinition {
        id: "eager_constructor_collaborators",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Constructors that instantiate several collaborators eagerly inside __init__.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    RuleDefinition {
        id: "god_class",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Python classes that concentrate unusually high method count, public surface area, and mutable instance state.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    RuleDefinition {
        id: "god_function",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Very large Python functions with high control-flow and call-surface concentration.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    RuleDefinition {
        id: "mixed_concerns_function",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Functions that mix HTTP, persistence, and filesystem-style concerns in one body.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    RuleDefinition {
        id: "monolithic_init_module",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "__init__.py files that carry enough imports and behavior to look like monolithic modules.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    RuleDefinition {
        id: "monolithic_module",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Non-__init__.py modules that are unusually large and combine many imports with orchestration-heavy behavior.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    RuleDefinition {
        id: "name_responsibility_mismatch",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Read-style, transformation-style, or utility-style names that still perform mutation or own multiple infrastructure concerns.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    RuleDefinition {
        id: "over_abstracted_wrapper",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Ceremonial wrapper-style or tiny data-container classes that add little beyond storing constructor state.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    RuleDefinition {
        id: "tight_module_coupling",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Modules that depend on a large number of repository-local Python modules.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    RuleDefinition {
        id: "too_many_instance_attributes",
        language: RuleLanguage::Python,
        family: "structure",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Classes that assign an unusually large number of instance attributes across their methods.",
        binding_location: super::bindings::PYTHON_STRUCTURE,
    },
    struct_rule!(
        "monolithic_module_owns_parsing_validation_execution_and_rendering",
        "Flag modules that contain the whole pipeline and become hard to reason about."
    ),
    struct_rule!(
        "class_mixes_factory_parsing_persistence_and_presentation_roles",
        "Flag classes that combine unrelated lifecycle roles."
    ),
    struct_rule!(
        "abstract_contracts_and_heavy_concrete_implementations_live_in_same_file",
        "Flag files that mix contracts with large concrete implementations and grow unstable dependencies."
    ),
    struct_rule!(
        "bidirectional_import_between_feature_modules",
        "Flag feature modules that depend on each other directly."
    ),
    struct_rule!(
        "generic_manager_or_processor_class_controls_many_unrelated_modes",
        "Flag vague manager-style classes that centralize unrelated behaviors behind mode flags."
    ),
    struct_rule!(
        "composition_candidate_for_optional_behavior_implemented_as_inheritance",
        "Flag optional behavior modeled through inheritance when composition would isolate concerns better."
    ),
    struct_rule!(
        "base_class_exists_only_to_share_data_fields_not_behavior",
        "Flag base classes that provide only data containers with no meaningful shared behavior."
    ),
    struct_rule!(
        "constructor_performs_real_work_beyond_state_initialization",
        "Flag constructors that do heavy work instead of leaving execution to explicit methods."
    ),
    struct_rule!(
        "module_global_registry_mutated_from_import_time_registration",
        "Flag global registries that are populated by import side effects."
    ),
    struct_rule!(
        "same_feature_path_crosses_many_layers_for_simple_data_transform",
        "Flag simple transformations that bounce through too many layers or wrappers."
    ),
    struct_rule!(
        "read_and_write_paths_share_mutable_internal_cache_without_boundary",
        "Flag read and write paths that couple through one mutable cache object."
    ),
    struct_rule!(
        "sync_and_async_contracts_mixed_on_same_interface_family",
        "Flag interface families that mix sync and async methods without a clear separation."
    ),
    struct_rule!(
        "helper_collection_object_also_owns_process_lifecycle",
        "Flag collection-like helper objects that also start, stop, or supervise process lifecycle work."
    ),
    struct_rule!(
        "sibling_modules_depend_on_private_helpers_from_each_other",
        "Flag sibling modules that reach into each other's private helpers instead of using a shared boundary."
    ),
    struct_rule!(
        "cross_cutting_policies_embedded_in_leaf_modules_instead_of_shared_boundary",
        "Flag retries, normalization, or policy checks embedded in leaf modules instead of one shared boundary."
    ),
    struct_rule!(
        "abstractions_named_utils_helpers_common_or_manager_hide_true_ownership",
        "Flag vague abstractions whose names obscure actual ownership and responsibility."
    ),
];
