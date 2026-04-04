use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

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
];
