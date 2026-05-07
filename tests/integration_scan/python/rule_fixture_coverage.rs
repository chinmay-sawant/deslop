use std::{collections::BTreeMap, fs, path::Path};

use deslop::{RuleLanguage, rule_registry};

use super::FixtureWorkspace;

const PLACEHOLDER_MARKERS: [&str; 7] = [
    "rule_id =",
    "family =",
    "severity =",
    "status =",
    "intent =",
    "description =",
    "return rule_id, family, severity, status, intent, description",
];

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("python")
        .join("rules_fixtures")
}

fn normalize_fixture_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| {
            let content = line.trim_start();
            let indent = line.len() - content.len();
            (indent, content)
        })
        .filter(|(_, content)| !content.is_empty())
        .filter(|(_, content)| *content != "from __future__ import annotations")
        .map(|(indent, content)| {
            let canonical = content
                .to_ascii_lowercase()
                .replace("_positive", "_polarity")
                .replace("_negative", "_polarity")
                .replace("positive", "__polarity__")
                .replace("negative", "__polarity__");
            format!(
                "{}:{}",
                indent,
                canonical.split_whitespace().collect::<Vec<_>>().join(" ")
            )
        })
        .collect()
}

fn is_placeholder_metadata_stub(text: &str) -> bool {
    let normalized = text
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    PLACEHOLDER_MARKERS
        .iter()
        .all(|marker| normalized.contains(marker))
}

fn is_polarity_only_clone(positive_text: &str, negative_text: &str) -> bool {
    let positive_lines = normalize_fixture_lines(positive_text);
    let negative_lines = normalize_fixture_lines(negative_text);
    positive_lines == negative_lines
}

#[test]
fn every_python_rule_has_positive_and_negative_fixture_text() {
    let missing = rule_registry()
        .iter()
        .filter(|metadata| metadata.language == RuleLanguage::Python)
        .flat_map(|metadata| {
            [
                python_rule_fixture_path(metadata, "positive"),
                python_rule_fixture_path(metadata, "negative"),
            ]
        })
        .filter(|path| !path.is_file())
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "missing Python rule fixture files: {:?}",
        missing
    );
}

#[test]
fn python_rule_coverage_fixtures_are_not_placeholder_metadata_stubs() {
    let fixture_root = fixture_root();

    let placeholders = rule_registry()
        .iter()
        .filter(|metadata| metadata.language == RuleLanguage::Python)
        .flat_map(|metadata| {
            [
                fixture_root
                    .join(metadata.id)
                    .join(format!("{}_positive.txt", metadata.id)),
                fixture_root
                    .join(metadata.id)
                    .join(format!("{}_negative.txt", metadata.id)),
            ]
        })
        .filter(|path| path.is_file())
        .filter(|path| {
            let text = std::fs::read_to_string(path).unwrap_or_else(|error| {
                panic!("failed to read fixture {}: {error}", path.display())
            });
            is_placeholder_metadata_stub(&text)
        })
        .collect::<Vec<_>>();

    assert!(
        placeholders.is_empty(),
        "Python rule fixtures collapsed back to placeholder metadata stubs: {:?}",
        placeholders
    );
}

#[test]
fn python_rule_coverage_fixture_pairs_are_meaningfully_different() {
    let fixture_root = fixture_root();

    let clone_like_pairs = rule_registry()
        .iter()
        .filter(|metadata| metadata.language == RuleLanguage::Python)
        .filter_map(|metadata| {
            let positive_path = fixture_root
                .join(metadata.id)
                .join(format!("{}_positive.txt", metadata.id));
            let negative_path = fixture_root
                .join(metadata.id)
                .join(format!("{}_negative.txt", metadata.id));
            if !positive_path.is_file() || !negative_path.is_file() {
                return None;
            }

            let positive_text = std::fs::read_to_string(&positive_path).unwrap_or_else(|error| {
                panic!(
                    "failed to read fixture {}: {error}",
                    positive_path.display()
                )
            });
            let negative_text = std::fs::read_to_string(&negative_path).unwrap_or_else(|error| {
                panic!(
                    "failed to read fixture {}: {error}",
                    negative_path.display()
                )
            });

            if is_polarity_only_clone(&positive_text, &negative_text) {
                Some((positive_path, negative_path))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    assert!(
        clone_like_pairs.is_empty(),
        "Python positive/negative fixture pairs must not collapse to polarity-only clones: {:?}",
        clone_like_pairs
    );
}

#[test]
fn python_rule_fixtures_do_not_reuse_identical_text() {
    let mut fixture_texts: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for metadata in python_rules() {
        let positive_path = python_rule_fixture_path(metadata, "positive");
        let negative_path = python_rule_fixture_path(metadata, "negative");
        let positive = read_fixture(&positive_path);
        let negative = read_fixture(&negative_path);

        assert_ne!(
            positive, negative,
            "positive and negative Python rule fixtures should not be identical for {}",
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
        "duplicate Python rule fixture text found: {duplicates:?}"
    );
}

#[test]
fn python_rule_fixture_batch_000_099_is_parseable_scenario_code() {
    assert_python_rule_fixture_batch(0, 100);
}

#[test]
fn python_rule_fixture_batch_100_199_is_parseable_scenario_code() {
    assert_python_rule_fixture_batch(100, 200);
}

#[test]
fn python_rule_fixture_batch_200_299_is_parseable_scenario_code() {
    assert_python_rule_fixture_batch(200, 300);
}

#[test]
fn python_rule_fixture_batch_300_399_is_parseable_scenario_code() {
    assert_python_rule_fixture_batch(300, 400);
}

#[test]
fn python_rule_fixture_batch_400_499_is_parseable_scenario_code() {
    assert_python_rule_fixture_batch(400, 500);
}

#[test]
fn python_rule_fixture_batch_500_599_is_parseable_scenario_code() {
    assert_python_rule_fixture_batch(500, 600);
}

#[test]
fn python_rule_fixture_batch_600_690_is_parseable_scenario_code() {
    assert_python_rule_fixture_batch(600, 691);
}

fn assert_python_rule_fixture_batch(start: usize, end: usize) {
    let python_rules = python_rules();
    let end = end.min(python_rules.len());
    assert!(
        start < end,
        "empty Python rule fixture batch {start}..{end}"
    );

    let workspace = FixtureWorkspace::new();
    let mut expected_files = 0;
    for (index, metadata) in python_rules[start..end].iter().enumerate() {
        for polarity in ["positive", "negative"] {
            let path = python_rule_fixture_path(metadata, polarity);
            let fixture = read_fixture(&path);

            assert!(
                !fixture.contains("return rule_id, family, severity, status, intent, description"),
                "Python rule fixture still contains generated metadata stub: {}",
                path.display()
            );
            assert!(
                fixture.contains("def ")
                    || fixture.contains("class ")
                    || fixture.contains("import ")
                    || fixture.contains("from ")
                    || fixture.contains("async def")
                    || fixture.contains("@"),
                "Python rule fixture should be Python-shaped source: {}",
                path.display()
            );

            workspace.write_file(
                &format!(
                    "internal/rule_coverage/batch_{start:03}/{:03}_{}_{}.py",
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
        "Python rule fixture batch {start}..{end} had parse failures: {:?}",
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
        "Python rule fixture batch {start}..{end} had syntax errors: {syntax_errors:?}"
    );
    assert_eq!(
        report.files_analyzed, expected_files,
        "Python rule fixture batch {start}..{end} should scan every generated .py fixture"
    );
}

fn python_rule_fixture_path(metadata: &deslop::RuleMetadata, polarity: &str) -> std::path::PathBuf {
    fixture_root()
        .join(metadata.id)
        .join(format!("{}_{}.txt", metadata.id, polarity))
}

fn python_rules() -> Vec<&'static deslop::RuleMetadata> {
    rule_registry()
        .iter()
        .filter(|metadata| metadata.language == RuleLanguage::Python)
        .collect()
}

fn read_fixture(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        panic!(
            "failed to read Python rule fixture {}: {error}",
            path.display()
        )
    })
}
