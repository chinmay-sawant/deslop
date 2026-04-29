use std::path::Path;

use deslop::{RuleLanguage, rule_registry};

#[test]
fn every_go_rule_has_positive_and_negative_fixture_text() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("go")
        .join("rule_coverage");

    let missing = rule_registry()
        .iter()
        .filter(|metadata| metadata.language == RuleLanguage::Go)
        .flat_map(|metadata| {
            [
                fixture_root
                    .join(metadata.family)
                    .join(format!("{}_positive.txt", metadata.id)),
                fixture_root
                    .join(metadata.family)
                    .join(format!("{}_negative.txt", metadata.id)),
            ]
        })
        .filter(|path| !path.is_file())
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "missing Go rule fixture files: {:?}",
        missing
    );
}
