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
            catalog.extend_from_slice(go::RULE_DEFINITIONS);
            catalog.extend_from_slice(python::RULE_DEFINITIONS);
            catalog.extend_from_slice(rust::RULE_DEFINITIONS);
            catalog
        })
        .as_slice()
}
