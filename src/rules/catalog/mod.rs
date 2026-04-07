use std::sync::OnceLock;

pub(crate) use super::{RuleConfigurability, RuleDefaultSeverity, RuleLanguage, RuleStatus};

pub(crate) mod bindings;
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
            extend_definition_slices(&mut catalog, &[common::RULE_DEFINITIONS]);
            extend_definition_slices(&mut catalog, go::FAMILY_RULES);
            extend_definition_slices(&mut catalog, python::FAMILY_RULES);
            extend_definition_slices(&mut catalog, rust::FAMILY_RULES);
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

fn extend_definition_slices(catalog: &mut Vec<RuleDefinition>, groups: &[&[RuleDefinition]]) {
    for group in groups {
        catalog.extend_from_slice(group);
    }
}
