use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

macro_rules! pack_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "packaging",
            default_severity: RuleDefaultSeverity::Info,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: super::bindings::PYTHON_PACKAGING,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "cross_package_internal_import",
        language: RuleLanguage::Python,
        family: "packaging",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Local Python packages reaching into another package's internal or private modules.",
        binding_location: super::bindings::PYTHON_PACKAGING,
    },
    RuleDefinition {
        id: "pyproject_missing_requires_python",
        language: RuleLanguage::Python,
        family: "packaging",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "pyproject metadata missing an explicit Python runtime requirement.",
        binding_location: super::bindings::PYTHON_PACKAGING,
    },
    RuleDefinition {
        id: "pyproject_script_entrypoint_unresolved",
        language: RuleLanguage::Python,
        family: "packaging",
        default_severity: RuleDefaultSeverity::Warning,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "pyproject script entrypoints that do not resolve to a locally indexed module callable.",
        binding_location: super::bindings::PYTHON_PACKAGING,
    },
    RuleDefinition {
        id: "python_public_api_any_contract",
        language: RuleLanguage::Python,
        family: "packaging",
        default_severity: RuleDefaultSeverity::Warning,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Public Python APIs that expose Any in parameter or return contracts.",
        binding_location: super::bindings::PYTHON_PACKAGING,
    },
    pack_rule!(
        "heavy_optional_dependency_imported_by_package_root",
        "Flag package roots that import heavy optional dependencies by default."
    ),
    pack_rule!(
        "cli_only_dependency_imported_by_library_entry_module",
        "Flag library entry modules that pull in CLI-only dependencies on import."
    ),
    pack_rule!(
        "package_init_performs_metadata_version_lookup_on_import",
        "Flag package __init__ files that do runtime metadata lookups just to expose a version."
    ),
    pack_rule!(
        "environment_or_config_read_during_package_import",
        "Flag package import paths that read environment or config eagerly."
    ),
    pack_rule!(
        "circular_import_hidden_by_function_local_import_on_hot_path",
        "Flag circular import workarounds that push imports into frequent code paths."
    ),
    pack_rule!(
        "plugin_discovery_scans_filesystem_each_invocation",
        "Flag plugin or extension discovery that rescans disk on every invocation."
    ),
    pack_rule!(
        "package_exports_same_symbol_name_from_multiple_submodules_with_different_meanings",
        "Flag packages that reuse one exported name for unrelated implementations."
    ),
    pack_rule!(
        "runtime_data_file_assumption_in_implicit_namespace_package",
        "Flag implicit namespace packages that assume local data-file discovery at runtime."
    ),
    pack_rule!(
        "test_helpers_shipped_inside_production_package_path",
        "Flag test-only helpers living under the production import path."
    ),
    pack_rule!(
        "public_api_surface_defined_only_by_import_side_effects",
        "Flag packages whose public surface is created indirectly by import order."
    ),
    pack_rule!(
        "package_root_reexports_large_dependency_tree_by_default",
        "Flag root packages that re-export many heavy submodules automatically."
    ),
    pack_rule!(
        "monolithic_common_package_becomes_transitive_dependency_for_most_modules",
        "Flag package designs where one vague common package becomes a hidden dependency hub."
    ),
];
