use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "dropped_error",
        language: RuleLanguage::Go,
        family: "errors",
        default_severity: RuleDefaultSeverity::Warning,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Blank identifier assignments that discard an err-like value.",
        binding_location: "src/heuristics/go/errors.rs",
    },
    RuleDefinition {
        id: "error_wrapping_misuse",
        language: RuleLanguage::Go,
        family: "errors",
        default_severity: RuleDefaultSeverity::Warning,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "fmt.Errorf calls that reference err without %w.",
        binding_location: "src/heuristics/go/errors.rs",
    },
    RuleDefinition {
        id: "panic_on_error",
        language: RuleLanguage::Go,
        family: "errors",
        default_severity: RuleDefaultSeverity::Warning,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "err != nil branches that jump straight to panic or log.Fatal style exits.",
        binding_location: "src/heuristics/go/errors.rs",
    },
];
