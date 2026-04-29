use std::{collections::BTreeMap, fs, path::Path};

use deslop::{RuleLanguage, rule_registry};

use super::FixtureWorkspace;

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

#[test]
fn go_rule_fixtures_do_not_reuse_identical_text() {
    let fixture_root = go_rule_fixture_root();
    let mut fixture_texts: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for metadata in go_rules() {
        let positive_path = fixture_root
            .join(metadata.family)
            .join(format!("{}_positive.txt", metadata.id));
        let negative_path = fixture_root
            .join(metadata.family)
            .join(format!("{}_negative.txt", metadata.id));
        let positive = read_fixture(&positive_path);
        let negative = read_fixture(&negative_path);

        assert_ne!(
            positive, negative,
            "positive and negative Go rule fixtures should not be identical for {}",
            metadata.id
        );

        fixture_texts
            .entry(positive)
            .or_default()
            .push(positive_path.display().to_string());
        fixture_texts
            .entry(negative)
            .or_default()
            .push(negative_path.display().to_string());
    }

    let duplicates = fixture_texts
        .values()
        .filter(|paths| paths.len() > 1)
        .collect::<Vec<_>>();
    assert!(
        duplicates.is_empty(),
        "duplicate Go rule fixture text found: {duplicates:?}"
    );
}

#[test]
fn go_rule_fixture_batch_000_099_is_parseable_scenario_code() {
    assert_go_rule_fixture_batch(0, 100);
}

#[test]
fn go_rule_fixture_batch_100_199_is_parseable_scenario_code() {
    assert_go_rule_fixture_batch(100, 200);
}

#[test]
fn go_rule_fixture_batch_200_299_is_parseable_scenario_code() {
    assert_go_rule_fixture_batch(200, 300);
}

#[test]
fn go_rule_fixture_batch_300_399_is_parseable_scenario_code() {
    assert_go_rule_fixture_batch(300, 400);
}

#[test]
fn go_rule_fixture_batch_400_499_is_parseable_scenario_code() {
    assert_go_rule_fixture_batch(400, 500);
}

#[test]
fn go_rule_fixture_batch_500_599_is_parseable_scenario_code() {
    assert_go_rule_fixture_batch(500, 600);
}

#[test]
fn go_rule_fixture_batch_600_699_is_parseable_scenario_code() {
    assert_go_rule_fixture_batch(600, 700);
}

#[test]
fn go_rule_fixture_batch_700_752_is_parseable_scenario_code() {
    assert_go_rule_fixture_batch(700, 753);
}

fn assert_go_rule_fixture_batch(start: usize, end: usize) {
    let fixture_root = go_rule_fixture_root();
    let go_rules = go_rules();
    let end = end.min(go_rules.len());
    assert!(start < end, "empty Go rule fixture batch {start}..{end}");

    let workspace = FixtureWorkspace::new();
    let mut expected_files = 0;
    for (index, metadata) in go_rules[start..end].iter().enumerate() {
        for polarity in ["positive", "negative"] {
            let path = fixture_root
                .join(metadata.family)
                .join(format!("{}_{}.txt", metadata.id, polarity));
            let fixture = read_fixture(&path);

            assert!(
                fixture.contains(&format!("scenario for {}", metadata.id)),
                "Go rule fixture should describe a concrete scenario: {}",
                path.display()
            );
            assert!(
                !fixture
                    .contains("return ruleID + family + severity + status + intent + description"),
                "Go rule fixture still contains generated metadata stub: {}",
                path.display()
            );
            assert!(
                fixture.trim_start().starts_with("package "),
                "Go rule fixture should be Go-shaped source: {}",
                path.display()
            );

            workspace.write_file(
                &format!(
                    "internal/rule_coverage/batch_{start:03}/{:03}_{}_{}.go",
                    start + index,
                    metadata.id,
                    polarity
                ),
                &fixture,
            );
            expected_files += 1;
        }
    }

    let report = workspace.scan();
    assert!(
        report.parse_failures.is_empty(),
        "Go rule fixture batch {start}..{end} had parse failures: {:?}",
        report.parse_failures
    );

    let syntax_errors = report
        .files
        .iter()
        .filter(|file| file.syntax_error)
        .map(|file| file.path.display().to_string())
        .collect::<Vec<_>>();
    assert!(
        syntax_errors.is_empty(),
        "Go rule fixture batch {start}..{end} had syntax errors: {syntax_errors:?}"
    );
    assert_eq!(
        report.files_analyzed, expected_files,
        "Go rule fixture batch {start}..{end} should scan every generated .go fixture"
    );
}

fn go_rule_fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("go")
        .join("rule_coverage")
}

fn go_rules() -> Vec<&'static deslop::RuleMetadata> {
    rule_registry()
        .iter()
        .filter(|metadata| metadata.language == RuleLanguage::Go)
        .collect()
}

fn read_fixture(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read Go rule fixture {}: {error}", path.display())
    })
}
