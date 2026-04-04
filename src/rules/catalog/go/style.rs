use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "inconsistent_package_name",
        language: RuleLanguage::Go,
        family: "style",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Directories that mix base Go package names after ignoring the _test suffix.",
        binding_location: super::bindings::GO_STYLE,
    },
    RuleDefinition {
        id: "misgrouped_imports",
        language: RuleLanguage::Go,
        family: "style",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Import blocks that place stdlib imports after third-party imports.",
        binding_location: super::bindings::GO_STYLE,
    },
];
