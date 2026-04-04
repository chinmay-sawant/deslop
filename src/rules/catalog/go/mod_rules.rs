use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "json_unmarshal_same_payload_multiple_times",
        language: RuleLanguage::Go,
        family: "mod",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same local JSON payload binding is unmarshaled into multiple targets in one function.",
        binding_location: super::bindings::GO_MOD,
    },
    RuleDefinition {
        id: "proto_unmarshal_same_payload_multiple_times",
        language: RuleLanguage::Go,
        family: "mod",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same local protobuf payload binding is unmarshaled into multiple targets in one function.",
        binding_location: super::bindings::GO_MOD,
    },
    RuleDefinition {
        id: "xml_unmarshal_same_payload_multiple_times",
        language: RuleLanguage::Go,
        family: "mod",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same local XML payload binding is unmarshaled into multiple targets in one function.",
        binding_location: super::bindings::GO_MOD,
    },
    RuleDefinition {
        id: "yaml_unmarshal_same_payload_multiple_times",
        language: RuleLanguage::Go,
        family: "mod",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same local YAML payload binding is unmarshaled into multiple targets in one function.",
        binding_location: super::bindings::GO_MOD,
    },
];
