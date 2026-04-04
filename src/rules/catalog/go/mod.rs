use std::sync::OnceLock;

pub(super) use super::{
    RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus,
};

mod concurrency;
mod consistency;
mod context;
mod data_access;
mod errors;
mod gin;
mod hot_path;
mod idioms;
mod library;
mod mod_rules;
mod performance;
mod security;
mod style;

pub(crate) fn rule_definitions() -> &'static [RuleDefinition] {
    static CATALOG: OnceLock<Vec<RuleDefinition>> = OnceLock::new();

    CATALOG
        .get_or_init(|| {
            let mut catalog = Vec::new();
            catalog.extend_from_slice(concurrency::RULE_DEFINITIONS);
            catalog.extend_from_slice(consistency::RULE_DEFINITIONS);
            catalog.extend_from_slice(context::RULE_DEFINITIONS);
            catalog.extend_from_slice(data_access::RULE_DEFINITIONS);
            catalog.extend_from_slice(errors::RULE_DEFINITIONS);
            catalog.extend_from_slice(gin::RULE_DEFINITIONS);
            catalog.extend_from_slice(hot_path::RULE_DEFINITIONS);
            catalog.extend_from_slice(idioms::RULE_DEFINITIONS);
            catalog.extend_from_slice(library::RULE_DEFINITIONS);
            catalog.extend_from_slice(mod_rules::RULE_DEFINITIONS);
            catalog.extend_from_slice(performance::RULE_DEFINITIONS);
            catalog.extend_from_slice(security::RULE_DEFINITIONS);
            catalog.extend_from_slice(style::RULE_DEFINITIONS);
            catalog
        })
        .as_slice()
}
