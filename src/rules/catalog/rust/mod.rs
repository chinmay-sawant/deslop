pub(super) use super::{
    RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus, bindings,
};

pub(crate) mod api_design;
pub(crate) mod async_patterns;
pub(crate) mod bad_practices;
pub(crate) mod boundary;
pub(crate) mod domain_modeling;
pub(crate) mod hygiene;
pub(crate) mod module_surface;
pub(crate) mod performance;
pub(crate) mod performance_layers;
pub(crate) mod runtime_boundary;
pub(crate) mod runtime_ownership;
pub(crate) mod security_footguns;
pub(crate) mod unsafe_soundness;

pub(crate) const FAMILY_RULES: &[&[RuleDefinition]] = &[
    api_design::RULE_DEFINITIONS,
    async_patterns::RULE_DEFINITIONS,
    boundary::RULE_DEFINITIONS,
    domain_modeling::RULE_DEFINITIONS,
    hygiene::RULE_DEFINITIONS,
    module_surface::RULE_DEFINITIONS,
    performance::RULE_DEFINITIONS,
    performance_layers::RULE_DEFINITIONS,
    runtime_boundary::RULE_DEFINITIONS,
    runtime_ownership::RULE_DEFINITIONS,
    security_footguns::RULE_DEFINITIONS,
    unsafe_soundness::RULE_DEFINITIONS,
    bad_practices::RULE_DEFINITIONS,
];
