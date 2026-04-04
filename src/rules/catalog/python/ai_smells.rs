use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "enthusiastic_commentary",
        language: RuleLanguage::Python,
        family: "ai_smells",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Unusually enthusiastic or emoji-heavy production comments.",
        binding_location: super::bindings::PYTHON_AI_SMELLS,
    },
    RuleDefinition {
        id: "mixed_naming_conventions",
        language: RuleLanguage::Python,
        family: "ai_smells",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "File mixes snake_case and camelCase function naming conventions.",
        binding_location: super::bindings::PYTHON_AI_SMELLS,
    },
    RuleDefinition {
        id: "obvious_commentary",
        language: RuleLanguage::Python,
        family: "ai_smells",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Comments that narrate obvious implementation steps instead of explaining intent.",
        binding_location: super::bindings::PYTHON_AI_SMELLS,
    },
    RuleDefinition {
        id: "textbook_docstring_small_helper",
        language: RuleLanguage::Python,
        family: "ai_smells",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Very small helper functions that have unusually long, textbook-style docstrings.",
        binding_location: super::bindings::PYTHON_AI_SMELLS,
    },
    RuleDefinition {
        id: "unrelated_heavy_import",
        language: RuleLanguage::Python,
        family: "ai_smells",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Heavy ecosystem imports with little local evidence of real need.",
        binding_location: super::bindings::PYTHON_AI_SMELLS,
    },
];
