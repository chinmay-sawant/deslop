use std::sync::OnceLock;

use super::{RuleConfigurability, RuleDefaultSeverity, RuleLanguage, RuleStatus};

mod common;
mod go;
mod python;
mod rust;

#[derive(Debug, Clone)]
pub(crate) struct RuleDefinition {
    pub id: &'static str,
    pub language: RuleLanguage,
    pub family: &'static str,
    pub default_severity: RuleDefaultSeverity,
    pub status: RuleStatus,
    pub configurability: &'static [RuleConfigurability],
    pub description: &'static str,
    pub binding_location: &'static str,
}

pub(crate) fn rule_catalog() -> &'static [RuleDefinition] {
    static CATALOG: OnceLock<Vec<RuleDefinition>> = OnceLock::new();

    CATALOG
        .get_or_init(|| {
            let mut catalog = Vec::new();
            catalog.extend_from_slice(common::RULE_DEFINITIONS);

            // Go Submodules
            catalog.extend_from_slice(go::concurrency::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::consistency::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::context::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::data_access::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::errors::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::gin::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::hot_path::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::idioms::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::library::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::mod_rules::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::performance::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::security::RULE_DEFINITIONS);
            catalog.extend_from_slice(go::style::RULE_DEFINITIONS);

            // Python Submodules
            catalog.extend_from_slice(python::ai_smells::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::duplication::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::framework::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::hot_path::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::hotpath::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::hotpath_ext::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::maintainability::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::mlops::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::packaging::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::performance::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::quality::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::structure::RULE_DEFINITIONS);

            catalog.extend_from_slice(rust::RULE_DEFINITIONS);
            catalog.sort_by(|left, right| {
                (&left.language, left.family, left.id).cmp(&(
                    &right.language,
                    right.family,
                    right.id,
                ))
            });
            catalog
        })
        .as_slice()
}
