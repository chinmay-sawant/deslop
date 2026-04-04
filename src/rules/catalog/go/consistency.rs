use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "duplicate_struct_tag_key",
        language: RuleLanguage::Go,
        family: "consistency",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Struct field tags that repeat the same key more than once.",
        binding_location: super::bindings::GO_CONSISTENCY,
    },
    RuleDefinition {
        id: "malformed_struct_tag",
        language: RuleLanguage::Go,
        family: "consistency",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Struct field tags that do not parse as valid Go tag key/value pairs.",
        binding_location: super::bindings::GO_CONSISTENCY,
    },
    RuleDefinition {
        id: "mixed_receiver_kinds",
        language: RuleLanguage::Go,
        family: "consistency",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Methods on the same receiver type mix pointer and value receivers.",
        binding_location: super::bindings::GO_CONSISTENCY,
    },
];
