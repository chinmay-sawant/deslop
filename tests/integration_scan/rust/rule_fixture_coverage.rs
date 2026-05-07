use std::{collections::BTreeMap, fs, path::Path};

use deslop::{RuleLanguage, rule_registry};

use super::FixtureWorkspace;

#[test]
fn every_rust_rule_has_positive_and_negative_fixture_text() {
    let missing = rule_registry()
        .iter()
        .filter(|metadata| metadata.language == RuleLanguage::Rust)
        .flat_map(|metadata| {
            [
                rust_rule_fixture_path(metadata, "positive"),
                rust_rule_fixture_path(metadata, "negative"),
            ]
        })
        .filter(|path| !path.is_file())
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "missing Rust rule fixture files: {:?}",
        missing
    );
}

#[test]
fn rust_rule_fixtures_do_not_reuse_identical_text() {
    let mut fixture_texts: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for metadata in rust_rules() {
        let positive_path = rust_rule_fixture_path(metadata, "positive");
        let negative_path = rust_rule_fixture_path(metadata, "negative");
        let positive = read_fixture(&positive_path);
        let negative = read_fixture(&negative_path);

        assert_ne!(
            positive, negative,
            "positive and negative Rust rule fixtures should not be identical for {}",
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
        "duplicate Rust rule fixture text found: {duplicates:?}"
    );
}

#[test]
fn rust_rule_fixtures_do_not_reuse_normalized_scenario_shape() {
    let mut fixture_shapes: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for metadata in rust_rules() {
        for polarity in ["positive", "negative"] {
            let path = rust_rule_fixture_path(metadata, polarity);
            let fixture = read_fixture(&path);

            fixture_shapes
                .entry(normalized_fixture_shape(&fixture))
                .or_default()
                .push(path.display().to_string());
        }
    }

    let duplicates = fixture_shapes
        .values()
        .filter(|paths| paths.len() > 1)
        .collect::<Vec<_>>();
    assert!(
        duplicates.is_empty(),
        "duplicate normalized Rust rule scenario shapes found: {duplicates:?}"
    );
}

#[test]
fn rust_rule_fixture_batch_000_099_is_parseable_scenario_code() {
    assert_rust_rule_fixture_batch(0, 100);
}

#[test]
fn rust_rule_fixture_batch_100_199_is_parseable_scenario_code() {
    assert_rust_rule_fixture_batch(100, 200);
}

#[test]
fn rust_rule_fixture_batch_200_299_is_parseable_scenario_code() {
    assert_rust_rule_fixture_batch(200, 300);
}

#[test]
fn rust_rule_fixture_batch_300_349_is_parseable_scenario_code() {
    assert_rust_rule_fixture_batch(300, 350);
}

fn assert_rust_rule_fixture_batch(start: usize, end: usize) {
    let rust_rules = rust_rules();
    let end = end.min(rust_rules.len());
    assert!(start < end, "empty Rust rule fixture batch {start}..{end}");

    let workspace = FixtureWorkspace::new();
    let mut expected_files = 0;
    let mut expectations = Vec::<(&'static str, std::path::PathBuf, bool)>::new();
    for (index, metadata) in rust_rules[start..end].iter().enumerate() {
        for polarity in ["positive", "negative"] {
            let path = rust_rule_fixture_path(metadata, polarity);
            let fixture = read_fixture(&path);

            assert!(
                !fixture
                    .contains("return ruleID + family + severity + status + intent + description"),
                "Rust rule fixture still contains generated metadata stub: {}",
                path.display()
            );
            assert!(
                fixture.contains("fn ")
                    || fixture.contains("struct ")
                    || fixture.contains("enum ")
                    || fixture.contains("trait ")
                    || fixture.contains("impl ")
                    || fixture.contains("use ")
                    || fixture.contains("pub ")
                    || fixture.contains("mod ")
                    || fixture.contains("const ")
                    || fixture.contains("static ")
                    || fixture.contains("macro_rules!"),
                "Rust rule fixture should be Rust-shaped source: {}",
                path.display()
            );

            let relative_path = format!(
                "internal/rule_coverage/batch_{start:03}/{:03}_{}_{}.rs",
                start + index,
                metadata.id,
                polarity
            );
            workspace.write_file(&relative_path, &fixture);
            expectations.push((
                metadata.id,
                std::path::PathBuf::from(relative_path),
                polarity == "positive",
            ));
            expected_files += 1;
        }
    }

    let report = workspace.scan();
    assert!(
        report.parse_failures.is_empty(),
        "Rust rule fixture batch {start}..{end} had parse failures: {:?}",
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
        "Rust rule fixture batch {start}..{end} had syntax errors: {syntax_errors:?}"
    );
    assert_eq!(
        report.files_analyzed, expected_files,
        "Rust rule fixture batch {start}..{end} should scan every generated .rs fixture"
    );

    let polarity_mismatches = expectations
        .iter()
        .filter_map(|(rule_id, relative_path, should_flag)| {
            let found = report.findings.iter().any(|finding| {
                finding.rule_id == *rule_id && finding.path.ends_with(relative_path)
            });
            if found == *should_flag {
                None
            } else {
                Some(format!(
                    "{} expected {} for {} but observed {}",
                    rule_id,
                    if *should_flag {
                        "a finding"
                    } else {
                        "no finding"
                    },
                    relative_path.display(),
                    if found { "a finding" } else { "no finding" }
                ))
            }
        })
        .collect::<Vec<_>>();
    assert!(
        polarity_mismatches.is_empty(),
        "Rust rule fixture batch {start}..{end} had polarity mismatches: {:?}",
        polarity_mismatches
    );
}

fn rust_rule_fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("rust")
        .join("rules_fixtures")
}

fn rust_rule_fixture_path(metadata: &deslop::RuleMetadata, polarity: &str) -> std::path::PathBuf {
    rust_rule_fixture_root()
        .join(metadata.id)
        .join(format!("{}_{}.txt", metadata.id, polarity))
}

fn rust_rules() -> Vec<&'static deslop::RuleMetadata> {
    rule_registry()
        .iter()
        .filter(|metadata| metadata.language == RuleLanguage::Rust)
        .collect()
}

fn read_fixture(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        panic!(
            "failed to read Rust rule fixture {}: {error}",
            path.display()
        )
    })
}

fn normalized_fixture_shape(fixture: &str) -> String {
    let mut without_comments = String::new();
    for line in fixture.lines() {
        if let Some((prefix, _)) = line.split_once("//!") {
            without_comments.push_str(prefix);
        } else if let Some((prefix, _)) = line.split_once("///") {
            without_comments.push_str(prefix);
        } else if let Some((prefix, _)) = line.split_once("//") {
            without_comments.push_str(prefix);
        } else {
            without_comments.push_str(line);
        }
        without_comments.push('\n');
    }

    let mut normalized_literals = String::new();
    let mut chars = without_comments.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                normalized_literals.push_str("\"STR\"");
                let mut escaped = false;
                for inner in chars.by_ref() {
                    if escaped {
                        escaped = false;
                    } else if inner == '\\' {
                        escaped = true;
                    } else if inner == '"' {
                        break;
                    }
                }
            }
            '/' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    let mut depth = 1u32;
                    while let Some(ch) = chars.next() {
                        if ch == '/' && chars.peek() == Some(&'*') {
                            chars.next();
                            depth += 1;
                        } else if ch == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                    }
                } else {
                    normalized_literals.push(ch);
                }
            }
            _ => normalized_literals.push(ch),
        }
    }

    let mut normalized_names = String::new();
    let mut token = String::new();
    for ch in normalized_literals.chars() {
        if ch == '_' || ch.is_ascii_alphanumeric() {
            token.push(ch);
            continue;
        }

        push_normalized_token(&mut normalized_names, &token);
        token.clear();
        if !ch.is_whitespace() {
            normalized_names.push(ch);
        } else if !normalized_names.ends_with(' ') {
            normalized_names.push(' ');
        }
    }
    push_normalized_token(&mut normalized_names, &token);

    normalized_names
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn push_normalized_token(output: &mut String, token: &str) {
    if token.is_empty() {
        return;
    }

    if token.starts_with("Positive") || token.starts_with("Negative") || token.starts_with("Case") {
        output.push_str("FixtureName");
    } else {
        output.push_str(token);
    }
}
