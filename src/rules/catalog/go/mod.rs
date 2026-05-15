pub(super) use super::{
    RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus, bindings,
};

pub(crate) mod architecture;
pub(crate) mod concurrency;
pub(crate) mod consistency;
pub(crate) mod context;
pub(crate) mod data_access;
pub(crate) mod errors;
pub(crate) mod gin;
pub(crate) mod hot_path;
pub(crate) mod idioms;
pub(crate) mod library;
pub(crate) mod mod_rules;
pub(crate) mod performance;
pub(crate) mod performance_extra;
pub(crate) mod performance_layers;
pub(crate) mod security;
pub(crate) mod style;

pub(crate) const FAMILY_RULES: &[&[RuleDefinition]] = &[
    architecture::RULE_DEFINITIONS,
    concurrency::RULE_DEFINITIONS,
    consistency::RULE_DEFINITIONS,
    context::RULE_DEFINITIONS,
    data_access::RULE_DEFINITIONS,
    errors::RULE_DEFINITIONS,
    gin::RULE_DEFINITIONS,
    hot_path::RULE_DEFINITIONS,
    idioms::RULE_DEFINITIONS,
    library::RULE_DEFINITIONS,
    mod_rules::RULE_DEFINITIONS,
    performance::RULE_DEFINITIONS,
    performance_extra::RULE_DEFINITIONS,
    performance_layers::RULE_DEFINITIONS,
    security::RULE_DEFINITIONS,
    style::RULE_DEFINITIONS,
];
