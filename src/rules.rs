use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RuleLanguage {
    Common,
    Go,
    Python,
    Rust,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RuleStatus {
    Stable,
    Experimental,
    Research,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RuleDefaultSeverity {
    Info,
    Warning,
    Error,
    Contextual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RuleConfigurability {
    Disable,
    Ignore,
    SeverityOverride,
    DetailsOnly,
    GoSemanticExperimental,
    RustAsyncExperimental,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuleMetadata {
    pub id: String,
    pub language: RuleLanguage,
    pub family: String,
    pub default_severity: RuleDefaultSeverity,
    pub status: RuleStatus,
    pub configurability: Vec<RuleConfigurability>,
    pub description: String,
}

pub fn rule_registry() -> &'static [RuleMetadata] {
    static REGISTRY: OnceLock<Vec<RuleMetadata>> = OnceLock::new();

    REGISTRY
        .get_or_init(|| {
            serde_json::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/rules/registry.json"
            )))
            .unwrap_or_else(|error| {
                unreachable!("rules/registry.json should be valid registry metadata: {error}")
            })
        })
        .as_slice()
}

pub fn rule_metadata(rule_id: &str, language: RuleLanguage) -> Option<&'static RuleMetadata> {
    rule_registry()
        .iter()
        .find(|metadata| metadata.id == rule_id && metadata.language == language)
}

pub fn rule_metadata_variants(rule_id: &str) -> Vec<&'static RuleMetadata> {
    rule_registry()
        .iter()
        .filter(|metadata| metadata.id == rule_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        RuleConfigurability, RuleLanguage, RuleStatus, rule_metadata, rule_metadata_variants,
        rule_registry,
    };

    #[test]
    fn registry_is_unique_per_language_and_sorted() {
        let registry = rule_registry();
        assert!(!registry.is_empty(), "registry should not be empty");

        let mut entries = BTreeSet::new();
        let mut previous = None::<(&str, &RuleLanguage, &str)>;

        for metadata in registry {
            assert!(
                entries.insert((metadata.id.as_str(), &metadata.language)),
                "duplicate language-scoped rule id in registry: {} ({:?})",
                metadata.id,
                metadata.language
            );

            if let Some((prev_id, prev_language, prev_family)) = previous {
                assert!(
                    (prev_language, prev_family, prev_id)
                        <= (
                            &metadata.language,
                            metadata.family.as_str(),
                            metadata.id.as_str()
                        ),
                    "registry should stay sorted by language, family, then id"
                );
            }

            previous = Some((
                metadata.id.as_str(),
                &metadata.language,
                metadata.family.as_str(),
            ));
        }
    }

    #[test]
    fn finds_known_rule_metadata() {
        let metadata = rule_metadata("dropped_error", RuleLanguage::Go)
            .unwrap_or_else(|| unreachable!("rule should be present"));

        assert_eq!(metadata.language, RuleLanguage::Go);
        assert_eq!(metadata.status, RuleStatus::Stable);
        assert!(
            metadata
                .configurability
                .contains(&RuleConfigurability::SeverityOverride)
        );
        assert!(metadata.description.contains("discard"));
    }

    #[test]
    fn tracks_language_scoped_variants_and_rollout_controls() {
        let full_dataset_load = rule_metadata_variants("full_dataset_load");
        assert_eq!(full_dataset_load.len(), 2);
        assert!(full_dataset_load.iter().all(|metadata| {
            metadata
                .configurability
                .contains(&RuleConfigurability::DetailsOnly)
        }));
        assert!(
            full_dataset_load
                .iter()
                .any(|metadata| metadata.language == RuleLanguage::Go)
        );
        assert!(
            full_dataset_load
                .iter()
                .any(|metadata| metadata.language == RuleLanguage::Python)
        );

        let async_rule = rule_metadata("rust_async_lock_order_cycle", RuleLanguage::Rust)
            .unwrap_or_else(|| unreachable!("rust async rollout rule should be present"));
        assert_eq!(async_rule.status, RuleStatus::Experimental);
        assert!(
            async_rule
                .configurability
                .contains(&RuleConfigurability::RustAsyncExperimental)
        );

        let go_experimental = rule_metadata("likely_n_squared_allocation", RuleLanguage::Go)
            .unwrap_or_else(|| unreachable!("go semantic rollout rule should be present"));
        assert!(
            go_experimental
                .configurability
                .contains(&RuleConfigurability::GoSemanticExperimental)
        );
        assert!(
            !go_experimental
                .configurability
                .contains(&RuleConfigurability::RustAsyncExperimental)
        );
    }
}
