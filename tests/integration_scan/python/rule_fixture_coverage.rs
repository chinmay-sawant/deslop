use std::path::Path;

use deslop::{RuleLanguage, rule_registry};

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
        .join("rule_coverage")
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
fn python_rule_coverage_fixtures_are_not_placeholder_metadata_stubs() {
    let fixture_root = fixture_root();

    let placeholders = rule_registry()
        .iter()
        .filter(|metadata| metadata.language == RuleLanguage::Python)
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
                .join(metadata.family)
                .join(format!("{}_positive.txt", metadata.id));
            let negative_path = fixture_root
                .join(metadata.family)
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
