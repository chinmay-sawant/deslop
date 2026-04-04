use std::sync::OnceLock;

pub(super) use super::{
    RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus,
};

mod ai_smells;
mod duplication;
mod framework;
mod hot_path;
mod hotpath;
mod hotpath_ext;
mod maintainability;
mod mlops;
mod packaging;
mod performance;
mod quality;
mod structure;

pub(crate) fn rule_definitions() -> &'static [RuleDefinition] {
    static CATALOG: OnceLock<Vec<RuleDefinition>> = OnceLock::new();

    CATALOG
        .get_or_init(|| {
            let mut catalog = Vec::new();
            catalog.extend_from_slice(ai_smells::RULE_DEFINITIONS);
            catalog.extend_from_slice(duplication::RULE_DEFINITIONS);
            catalog.extend_from_slice(framework::RULE_DEFINITIONS);
            catalog.extend_from_slice(hot_path::RULE_DEFINITIONS);
            catalog.extend_from_slice(hotpath::RULE_DEFINITIONS);
            catalog.extend_from_slice(hotpath_ext::RULE_DEFINITIONS);
            catalog.extend_from_slice(maintainability::RULE_DEFINITIONS);
            catalog.extend_from_slice(mlops::RULE_DEFINITIONS);
            catalog.extend_from_slice(packaging::RULE_DEFINITIONS);
            catalog.extend_from_slice(performance::RULE_DEFINITIONS);
            catalog.extend_from_slice(quality::RULE_DEFINITIONS);
            catalog.extend_from_slice(structure::RULE_DEFINITIONS);
            catalog
        })
        .as_slice()
}
