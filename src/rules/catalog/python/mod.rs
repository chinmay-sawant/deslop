pub(super) use super::{
    RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus, bindings,
};

pub(crate) mod ai_smells;
pub(crate) mod architecture;
pub(crate) mod boundaries;
pub(crate) mod discipline;
pub(crate) mod duplication;
pub(crate) mod framework;
pub(crate) mod hot_path;
pub(crate) mod hotpath;
pub(crate) mod hotpath_ext;
pub(crate) mod maintainability;
pub(crate) mod mlops;
pub(crate) mod observability;
pub(crate) mod packaging;
pub(crate) mod performance;
pub(crate) mod performance_layers;
pub(crate) mod quality;
pub(crate) mod structure;

pub(crate) const FAMILY_RULES: &[&[RuleDefinition]] = &[
    ai_smells::RULE_DEFINITIONS,
    architecture::RULE_DEFINITIONS,
    boundaries::RULE_DEFINITIONS,
    discipline::RULE_DEFINITIONS,
    duplication::RULE_DEFINITIONS,
    framework::RULE_DEFINITIONS,
    hot_path::RULE_DEFINITIONS,
    hotpath::RULE_DEFINITIONS,
    hotpath_ext::RULE_DEFINITIONS,
    maintainability::RULE_DEFINITIONS,
    mlops::RULE_DEFINITIONS,
    observability::RULE_DEFINITIONS,
    packaging::RULE_DEFINITIONS,
    performance::RULE_DEFINITIONS,
    performance_layers::RULE_DEFINITIONS,
    quality::RULE_DEFINITIONS,
    structure::RULE_DEFINITIONS,
];
