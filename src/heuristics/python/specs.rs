#[path = "specs/catalog.rs"]
mod catalog;
#[path = "specs/runtime.rs"]
mod runtime;
#[path = "specs/types.rs"]
mod types;

pub(super) use catalog::{FILE_RULE_SPECS, FUNCTION_RULE_SPECS, REPO_RULE_SPECS};
pub(super) use runtime::{evaluate_file_specs, evaluate_function_specs, evaluate_repo_specs};

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::rules::{RuleLanguage, rule_registry};

    use super::{FILE_RULE_SPECS, FUNCTION_RULE_SPECS, REPO_RULE_SPECS};

    #[test]
    fn python_rule_specs_cover_registry() {
        let mut spec_rule_counts = BTreeMap::<&str, usize>::new();
        let spec_rule_ids = FUNCTION_RULE_SPECS
            .iter()
            .flat_map(|spec| spec.rule_ids.iter().copied())
            .chain(
                FILE_RULE_SPECS
                    .iter()
                    .flat_map(|spec| spec.rule_ids.iter().copied()),
            )
            .chain(
                REPO_RULE_SPECS
                    .iter()
                    .flat_map(|spec| spec.rule_ids.iter().copied()),
            )
            .inspect(|rule_id| {
                *spec_rule_counts.entry(*rule_id).or_default() += 1;
            })
            .collect::<BTreeSet<_>>();

        let registry_rule_ids = rule_registry()
            .iter()
            .filter(|metadata| metadata.language == RuleLanguage::Python)
            .map(|metadata| metadata.id)
            .collect::<BTreeSet<_>>();

        assert_eq!(spec_rule_ids, registry_rule_ids);

        let duplicate_rule_ids = spec_rule_counts
            .into_iter()
            .filter_map(|(rule_id, count)| (count > 1).then_some(rule_id))
            .collect::<Vec<_>>();
        assert_eq!(
            duplicate_rule_ids,
            vec!["name_responsibility_mismatch", "public_any_type_leak"]
        );
    }

    #[test]
    fn every_python_rule_spec_declares_its_owned_rules() {
        for spec in FUNCTION_RULE_SPECS {
            assert!(
                !spec.rule_ids.is_empty(),
                "function spec family {} should own at least one rule",
                spec.family
            );
        }

        for spec in FILE_RULE_SPECS {
            assert!(
                !spec.rule_ids.is_empty(),
                "file spec family {} should own at least one rule",
                spec.family
            );
        }

        for spec in REPO_RULE_SPECS {
            assert!(
                !spec.rule_ids.is_empty(),
                "repo spec family {} should own at least one rule",
                spec.family
            );
        }
    }
}
