use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

pub(crate) mod catalog;

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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuleMetadata {
    pub id: &'static str,
    pub language: RuleLanguage,
    pub family: &'static str,
    pub default_severity: RuleDefaultSeverity,
    pub status: RuleStatus,
    pub configurability: &'static [RuleConfigurability],
    pub description: &'static str,
}

pub fn rule_registry() -> &'static [RuleMetadata] {
    static REGISTRY: OnceLock<Vec<RuleMetadata>> = OnceLock::new();

    REGISTRY
        .get_or_init(|| {
            catalog::rule_catalog()
                .iter()
                .map(rule_metadata_from_definition)
                .collect()
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

pub fn rule_binding_location(rule_id: &str, language: RuleLanguage) -> Option<&'static str> {
    catalog::rule_catalog()
        .iter()
        .find(|definition| definition.id == rule_id && definition.language == language)
        .map(|definition| definition.binding_location)
}

pub fn is_detail_only_rule(rule_id: &str) -> bool {
    rule_metadata_variants(rule_id).iter().any(|metadata| {
        metadata
            .configurability
            .contains(&RuleConfigurability::DetailsOnly)
    })
}

pub fn is_async_rollout_rule(rule_id: &str) -> bool {
    rule_metadata_variants(rule_id).iter().any(|metadata| {
        metadata
            .configurability
            .contains(&RuleConfigurability::RustAsyncExperimental)
    })
}

fn rule_metadata_from_definition(definition: &catalog::RuleDefinition) -> RuleMetadata {
    RuleMetadata {
        id: definition.id,
        language: definition.language.clone(),
        family: definition.family,
        default_severity: definition.default_severity.clone(),
        status: definition.status.clone(),
        configurability: definition.configurability,
        description: definition.description,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{
        RuleConfigurability, RuleLanguage, RuleStatus, catalog, is_detail_only_rule,
        rule_binding_location, rule_metadata, rule_metadata_variants, rule_registry,
    };
    use crate::analysis::Language;
    use crate::heuristics::registry::{language_rule_specs, shared_rule_specs};
    use crate::{DEFAULT_MAX_BYTES, read_to_string_limited};

    // Intentional maintenance guard. If this changes, review the source rule-id diff and
    // update [guides/inventory-regression-guards.md] in the same change.
    const EXPECTED_SOURCE_RULE_ID_COUNT: usize = 457;
    const EXPECTED_RULE_COUNTS_BY_LANGUAGE: &[(RuleLanguage, usize)] = &[
        (RuleLanguage::Common, 11),
        (RuleLanguage::Go, 637),
        (RuleLanguage::Python, 212),
        (RuleLanguage::Rust, 100),
    ];

    #[test]
    fn registry_is_unique_per_language_and_sorted() {
        let registry = rule_registry();
        assert!(!registry.is_empty(), "registry should not be empty");

        let mut entries = BTreeSet::new();
        let mut previous = None::<(&str, &RuleLanguage, &str)>;

        for metadata in registry {
            assert!(
                entries.insert((metadata.id, &metadata.language)),
                "duplicate language-scoped rule id in registry: {} ({:?})",
                metadata.id,
                metadata.language
            );

            if let Some((prev_id, prev_language, prev_family)) = previous {
                assert!(
                    (prev_language, prev_family, prev_id)
                        <= (&metadata.language, metadata.family, metadata.id),
                    "registry should stay sorted by language, family, then id"
                );
            }

            previous = Some((metadata.id, &metadata.language, metadata.family));
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
            !metadata
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

    #[test]
    fn detail_only_predicate_matches_rule_metadata_variants() {
        assert_eq!(
            is_detail_only_rule("placeholder_test_body"),
            rule_metadata_variants("placeholder_test_body")
                .iter()
                .any(|metadata| metadata
                    .configurability
                    .contains(&RuleConfigurability::DetailsOnly))
        );
    }

    #[test]
    fn runtime_policy_marks_experimental_rules_and_configurability() {
        let registry = rule_registry();
        let experimental_rules = registry
            .iter()
            .filter(|metadata| metadata.status == RuleStatus::Experimental)
            .map(|metadata| (metadata.language.clone(), metadata.id))
            .collect::<BTreeSet<_>>();

        let expected = BTreeSet::from([
            (RuleLanguage::Go, "likely_n_squared_allocation"),
            (RuleLanguage::Go, "likely_n_squared_string_concat"),
            (RuleLanguage::Rust, "rust_async_blocking_drop"),
            (RuleLanguage::Rust, "rust_async_hold_permit_across_await"),
            (RuleLanguage::Rust, "rust_async_invariant_broken_at_await"),
            (RuleLanguage::Rust, "rust_async_lock_order_cycle"),
            (RuleLanguage::Rust, "rust_async_missing_fuse_pin"),
            (RuleLanguage::Rust, "rust_async_monopolize_executor"),
            (RuleLanguage::Rust, "rust_async_recreate_future_in_select"),
            (RuleLanguage::Rust, "rust_async_spawn_cancel_at_await"),
            (RuleLanguage::Rust, "rust_async_std_mutex_await"),
            (RuleLanguage::Rust, "rust_blocking_io_in_async"),
            (RuleLanguage::Rust, "rust_lock_across_await"),
            (RuleLanguage::Rust, "rust_tokio_mutex_unnecessary"),
        ]);

        assert_eq!(experimental_rules, expected);

        for metadata in registry {
            match (metadata.language.clone(), metadata.id) {
                (RuleLanguage::Go, "likely_n_squared_allocation")
                | (RuleLanguage::Go, "likely_n_squared_string_concat") => {
                    assert_eq!(metadata.status, RuleStatus::Experimental);
                    assert!(
                        metadata
                            .configurability
                            .contains(&RuleConfigurability::GoSemanticExperimental)
                    );
                }
                (
                    RuleLanguage::Rust,
                    "rust_async_blocking_drop"
                    | "rust_async_hold_permit_across_await"
                    | "rust_async_invariant_broken_at_await"
                    | "rust_async_lock_order_cycle"
                    | "rust_async_missing_fuse_pin"
                    | "rust_async_monopolize_executor"
                    | "rust_async_recreate_future_in_select"
                    | "rust_async_spawn_cancel_at_await"
                    | "rust_async_std_mutex_await"
                    | "rust_blocking_io_in_async"
                    | "rust_lock_across_await"
                    | "rust_tokio_mutex_unnecessary",
                ) => {
                    assert_eq!(metadata.status, RuleStatus::Experimental);
                    assert!(
                        metadata
                            .configurability
                            .contains(&RuleConfigurability::RustAsyncExperimental)
                    );
                }
                _ => {
                    assert_eq!(metadata.status, RuleStatus::Stable);
                    assert_eq!(
                        metadata.configurability,
                        vec![
                            RuleConfigurability::Disable,
                            RuleConfigurability::Ignore,
                            RuleConfigurability::SeverityOverride,
                        ]
                    );
                }
            }
        }
    }

    #[test]
    fn source_rule_ids_match_public_registry() {
        let source_rule_ids =
            collect_source_rule_ids(Path::new(env!("CARGO_MANIFEST_DIR")).join("src"));
        let registry_rule_ids = rule_registry()
            .iter()
            .map(|metadata| metadata.id.to_string())
            .collect::<BTreeSet<_>>();

        assert!(
            source_rule_ids.is_subset(&registry_rule_ids),
            "source contains rule ids that are missing from the public registry"
        );
        assert_eq!(
            source_rule_ids.len(),
            EXPECTED_SOURCE_RULE_ID_COUNT,
            "source rule-id inventory changed; if intentional, update EXPECTED_SOURCE_RULE_ID_COUNT and guides/inventory-regression-guards.md"
        );
    }

    #[test]
    fn registry_rule_counts_remain_grouped_by_language() {
        let mut counts = BTreeMap::<RuleLanguage, usize>::new();

        for metadata in rule_registry() {
            *counts.entry(metadata.language.clone()).or_insert(0) += 1;
        }

        assert_eq!(
            counts,
            EXPECTED_RULE_COUNTS_BY_LANGUAGE
                .iter()
                .cloned()
                .collect::<BTreeMap<_, _>>(),
            "registry language breakdown changed; if intentional, update the grouped counts and guides/inventory-regression-guards.md"
        );
    }

    #[test]
    fn binding_locations_are_available_for_catalogued_rules() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));

        for definition in catalog::rule_catalog() {
            assert!(
                definition.binding_location.ends_with(".rs"),
                "binding location should point at a Rust source file for {} ({:?})",
                definition.id,
                definition.language
            );

            let path = repo_root.join(definition.binding_location);
            assert!(
                path.is_file(),
                "binding location should exist for {} ({:?}): {}",
                definition.id,
                definition.language,
                path.display()
            );
        }
    }

    #[test]
    fn python_hot_path_binding_locations_match_implementations() {
        let expected = [
            (
                "append_then_sort_each_iteration",
                "src/heuristics/python/hotpath_ext.rs",
            ),
            (
                "csv_writer_flush_per_row",
                "src/heuristics/python/hotpath.rs",
            ),
            (
                "filter_then_count_then_iterate",
                "src/heuristics/python/hotpath_ext.rs",
            ),
            (
                "json_encoder_recreated_per_item",
                "src/heuristics/python/hotpath_ext.rs",
            ),
        ];

        for (rule_id, expected_location) in expected {
            assert_eq!(
                rule_binding_location(rule_id, RuleLanguage::Python),
                Some(expected_location),
                "binding location drifted for {rule_id}"
            );
        }
    }

    #[test]
    fn go_library_binding_locations_match_leaf_implementation() {
        let go_library_rules = catalog::rule_catalog()
            .iter()
            .filter(|definition| {
                definition.language == RuleLanguage::Go && definition.family == "library"
            })
            .collect::<Vec<_>>();

        assert!(
            !go_library_rules.is_empty(),
            "go library catalog entries should be present"
        );

        for definition in go_library_rules {
            assert_eq!(
                definition.binding_location, "src/heuristics/go/library_misuse/library.rs",
                "go library binding location should stay pinned to the leaf implementation for {}",
                definition.id
            );
        }
    }

    #[test]
    fn execution_specs_cover_shared_and_language_specific_layers() {
        assert!(
            !shared_rule_specs().is_empty(),
            "shared rule execution specs should stay populated"
        );
        assert!(
            language_rule_specs(Language::Go).len() >= 5,
            "go should keep multiple execution families"
        );
        assert!(
            !language_rule_specs(Language::Python).is_empty(),
            "python should keep an execution registry entry"
        );
        assert!(
            language_rule_specs(Language::Rust).len() >= 10,
            "rust should remain decomposed into family-shaped execution specs instead of a single evaluator"
        );
    }

    #[test]
    fn every_execution_spec_has_a_family_name() {
        for spec in shared_rule_specs() {
            assert!(
                !spec.family.is_empty(),
                "shared execution specs should keep a stable family label"
            );
        }

        for language in [Language::Go, Language::Python, Language::Rust] {
            for spec in language_rule_specs(language) {
                assert!(
                    !spec.family.is_empty(),
                    "{language:?} execution specs should keep a stable family label"
                );
            }
        }
    }

    fn collect_source_rule_ids(root: PathBuf) -> BTreeSet<String> {
        let mut ids = BTreeSet::new();
        collect_rule_ids_from_dir(&root, &mut ids);
        ids
    }

    fn collect_rule_ids_from_dir(dir: &Path, ids: &mut BTreeSet<String>) {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(error) => unreachable!("failed to read directory {}: {error}", dir.display()),
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => unreachable!("failed to read entry in {}: {error}", dir.display()),
            };
            let path = entry.path();
            if path.is_dir() {
                collect_rule_ids_from_dir(&path, ids);
                continue;
            }

            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }

            let source = match read_to_string_limited(&path, DEFAULT_MAX_BYTES) {
                Ok(source) => source,
                Err(error) => unreachable!("failed to read {}: {error}", path.display()),
            };
            let production_source = source
                .split_once("#[cfg(test)]")
                .map(|(production, _)| production)
                .unwrap_or(&source);
            extract_rule_ids(production_source, ids);
        }
    }

    fn extract_rule_ids(source: &str, ids: &mut BTreeSet<String>) {
        let mut search_start = 0;

        while let Some(offset) = source[search_start..].find("rule_id: \"") {
            let start = search_start + offset + "rule_id: \"".len();
            let Some(end_offset) = source[start..].find('\"') else {
                break;
            };

            ids.insert(source[start..start + end_offset].to_string());
            search_start = start + end_offset + 1;
        }
    }
}
