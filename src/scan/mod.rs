mod walker;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;

use crate::analysis::{ParsedFile, backend_for_language, backend_for_path, supported_extensions};
use crate::heuristics::evaluate_shared;
use crate::index::build_repository_index;
use crate::model::{Finding, ParseFailure, ScanOptions, ScanReport, TimingBreakdown};
use crate::io::canonicalize_within_root;
use crate::{DEFAULT_MAX_BYTES, RepoConfig, Result, load_repository_config, read_to_string_limited};
use crate::scan::walker::discover_source_files;

pub fn scan_repository(options: &ScanOptions) -> Result<ScanReport> {
    let total_start = Instant::now();
    let canonical_root = options
        .root
        .canonicalize()
        .map_err(|error| crate::Error::io(&options.root, error))?;
    let repo_config = load_repository_config(&canonical_root)?;

    let discover_start = Instant::now();
    let supported_extensions = supported_extensions();
    let discovered_files =
        discover_source_files(&canonical_root, options.respect_ignore, &supported_extensions)?;
    let discover_ms = discover_start.elapsed().as_millis();

    let parse_start = Instant::now();
    let mut parsed_files = Vec::new();
    let mut parse_failures = Vec::new();
    let mut suppressions = BTreeMap::new();
    let mut outcomes = discovered_files
        .par_iter()
        .map(|path| analyze_file(path))
        .collect::<Vec<_>>();
    outcomes.sort_by(|left, right| left.path().cmp(right.path()));

    for outcome in outcomes {
        match outcome {
            FileOutcome::Parsed { file, suppressions: file_suppressions } => {
                suppressions.insert(file.path.clone(), file_suppressions);
                parsed_files.push(*file);
            }
            FileOutcome::Generated(_) => {}
            FileOutcome::Failed(failure) => parse_failures.push(failure),
        }
    }
    let parse_ms = parse_start.elapsed().as_millis();

    let index_start = Instant::now();
    let index = build_repository_index(&canonical_root, &parsed_files);
    let index_summary = index.summary();
    let index_ms = index_start.elapsed().as_millis();

    let heuristics_start = Instant::now();
    let findings = evaluate_findings(
        &parsed_files,
        &index,
        &suppressions,
        &repo_config,
        &canonical_root,
    );
    let heuristics_ms = heuristics_start.elapsed().as_millis();

    let files_analyzed = parsed_files.len();
    let functions_found = parsed_files.iter().map(|file| file.functions.len()).sum();
    let files = parsed_files.iter().map(ParsedFile::to_report).collect();

    Ok(ScanReport {
        root: canonical_root,
        files_discovered: discovered_files.len(),
        files_analyzed,
        functions_found,
        files,
        findings,
        index_summary,
        parse_failures,
        timings: TimingBreakdown {
            discover_ms,
            parse_ms,
            index_ms,
            heuristics_ms,
            total_ms: total_start.elapsed().as_millis(),
        },
    })
}

fn evaluate_findings(
    files: &[ParsedFile],
    index: &crate::index::RepositoryIndex,
    suppressions: &BTreeMap<PathBuf, Vec<SuppressionDirective>>,
    repo_config: &RepoConfig,
    root: &Path,
) -> Vec<Finding> {
    let mut findings = evaluate_shared(files, index);

    for file in files {
        if let Some(backend) = backend_for_language(file.language) {
            findings.extend(backend.evaluate_file(file, index));
        }
    }

    for backend in crate::analysis::registered_backends() {
        let backend_files = files
            .iter()
            .filter(|file| file.language == backend.language())
            .collect::<Vec<_>>();
        findings.extend(backend.evaluate_repo(&backend_files, index));
    }

    findings.retain(|finding| !is_suppressed(finding, suppressions));
    apply_repository_config(&mut findings, repo_config, root);

    findings.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    findings
}

enum FileOutcome {
    Parsed {
        file: Box<ParsedFile>,
        suppressions: Vec<SuppressionDirective>,
    },
    Generated(std::path::PathBuf),
    Failed(ParseFailure),
}

impl FileOutcome {
    fn path(&self) -> &std::path::Path {
        match self {
            Self::Parsed { file, .. } => &file.path,
            Self::Generated(path) => path,
            Self::Failed(failure) => &failure.path,
        }
    }
}

fn analyze_file(path: &Path) -> FileOutcome {
    let path = match canonicalize_within_root(path.parent().unwrap_or(path), path) {
        Ok(path) => path,
        Err(error) => {
            return FileOutcome::Failed(ParseFailure {
                path: path.to_path_buf(),
                message: error.to_string(),
            });
        }
    };

    match read_to_string_limited(&path, DEFAULT_MAX_BYTES) {
        Ok(source) => {
            if is_generated(&source) {
                return FileOutcome::Generated(path.clone());
            }

            let suppressions = parse_suppression_directives(&source);

            let Some(analyzer) = backend_for_path(&path) else {
                return FileOutcome::Failed(ParseFailure {
                    path: path.clone(),
                    message: format!("no analyzer registered for {}", path.display()),
                });
            };

            match analyzer.parse_file(&path, &source) {
                Ok(file) => FileOutcome::Parsed {
                    file: Box::new(file),
                    suppressions,
                },
                Err(error) => FileOutcome::Failed(ParseFailure {
                    path: path.clone(),
                    message: error.to_string(),
                }),
            }
        }
        Err(error) => FileOutcome::Failed(ParseFailure {
            path,
            message: error.to_string(),
        }),
    }
}

fn is_generated(source: &str) -> bool {
    source.lines().take(5).any(|line| {
        let normalized = line.trim();
        normalized.contains("Code generated") && normalized.contains("DO NOT EDIT")
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SuppressionDirective {
    rule_id: String,
    line: usize,
    next_code_line: Option<usize>,
}

fn is_suppressed(
    finding: &Finding,
    suppressions: &BTreeMap<PathBuf, Vec<SuppressionDirective>>,
) -> bool {
    suppressions.get(&finding.path).is_some_and(|directives| {
        directives.iter().any(|directive| {
            directive.rule_id == finding.rule_id
                && (directive.line == finding.start_line
                    || directive.next_code_line == Some(finding.start_line))
        })
    })
}

fn apply_repository_config(findings: &mut Vec<Finding>, repo_config: &RepoConfig, root: &Path) {
    findings.retain(|finding| {
        !repo_config.disabled_rules.iter().any(|rule_id| rule_id == &finding.rule_id)
            && (repo_config.rust_async_experimental || !is_async_rollout_rule(&finding.rule_id))
            && !path_is_suppressed(&finding.path, root, &repo_config.suppressed_paths)
    });

    for finding in findings.iter_mut() {
        if let Some(severity) = repo_config.severity_overrides.get(&finding.rule_id) {
            finding.severity = severity.clone();
        }
    }
}

fn path_is_suppressed(path: &Path, root: &Path, suppressed_paths: &[PathBuf]) -> bool {
    suppressed_paths.iter().any(|prefix| {
        if prefix.is_absolute() {
            path.starts_with(prefix)
        } else {
            path.strip_prefix(root)
                .is_ok_and(|relative_path| relative_path.starts_with(prefix))
        }
    })
}

fn is_async_rollout_rule(rule_id: &str) -> bool {
    matches!(
        rule_id,
        "rust_blocking_io_in_async"
            | "rust_lock_across_await"
            | "rust_tokio_mutex_unnecessary"
    ) || rule_id.starts_with("rust_async_")
}

fn parse_suppression_directives(source: &str) -> Vec<SuppressionDirective> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut directives = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let Some((_, tail)) = line.split_once("deslop-ignore:") else {
            continue;
        };

        let next_code_line = next_code_line(&lines, index + 1);
        for rule_id in parse_rule_ids(tail) {
            directives.push(SuppressionDirective {
                rule_id,
                line: index + 1,
                next_code_line,
            });
        }
    }

    directives
}

fn parse_rule_ids(tail: &str) -> Vec<String> {
    tail.split([',', ' ', '\t'])
        .filter_map(|token| {
            let trimmed = token
                .trim_matches(|character: char| !matches!(character, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-'));
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect()
}

fn next_code_line(lines: &[&str], start_index: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(start_index)
        .find_map(|(index, line)| is_code_line(line).then_some(index + 1))
}

fn is_code_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with("//")
        && !trimmed.starts_with('#')
        && !trimmed.starts_with("/*")
        && !trimmed.starts_with('*')
        && !trimmed.starts_with("*/")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        SuppressionDirective, apply_repository_config, is_generated, next_code_line,
        parse_rule_ids, parse_suppression_directives, scan_repository,
    };
    use crate::RepoConfig;
    use crate::model::{Finding, Severity};
    use crate::model::ScanOptions;

    fn sample_finding(rule_id: &str, severity: Severity) -> Finding {
        Finding {
            rule_id: rule_id.to_string(),
            severity,
            path: std::path::PathBuf::from("src/lib.rs"),
            function_name: Some("demo".to_string()),
            start_line: 1,
            end_line: 1,
            message: "demo".to_string(),
            evidence: Vec::new(),
        }
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("deslop-scan-{name}-{nonce}"));
        fs::create_dir_all(&path).expect("scan temp dir should be created");
        path
    }

    #[test]
    fn test_is_generated() {
        let generated = "// Code generated by mockery. DO NOT EDIT.\npackage sample\n";
        assert!(is_generated(generated));
    }

    #[test]
    fn parses_rule_ids_from_inline_directive() {
        assert_eq!(
            parse_rule_ids("unwrap_in_non_test_code, panic_macro_leftover */"),
            vec![
                "unwrap_in_non_test_code".to_string(),
                "panic_macro_leftover".to_string()
            ]
        );
    }

    #[test]
    fn finds_next_code_line_after_directive_comments() {
        let lines = vec!["// deslop-ignore:unwrap_in_non_test_code", "", "// note", "value.unwrap();"];
        assert_eq!(next_code_line(&lines, 1), Some(4));
    }

    #[test]
    fn parses_same_line_and_next_line_suppressions() {
        let source = "fn demo() {\n    let _ = option.unwrap(); // deslop-ignore:unwrap_in_non_test_code\n    // deslop-ignore:panic_macro_leftover\n    panic!(\"boom\");\n}\n";

        assert_eq!(
            parse_suppression_directives(source),
            vec![
                SuppressionDirective {
                    rule_id: "unwrap_in_non_test_code".to_string(),
                    line: 2,
                    next_code_line: Some(4),
                },
                SuppressionDirective {
                    rule_id: "panic_macro_leftover".to_string(),
                    line: 3,
                    next_code_line: Some(4),
                }
            ]
        );
    }

    #[test]
    fn applies_disabled_rules_and_severity_overrides() {
        let mut findings = vec![
            sample_finding("panic_macro_leftover", Severity::Warning),
            sample_finding("unwrap_in_non_test_code", Severity::Warning),
        ];
        let mut repo_config = RepoConfig {
            rust_async_experimental: true,
            disabled_rules: vec!["panic_macro_leftover".to_string()],
            suppressed_paths: Vec::new(),
            severity_overrides: std::collections::BTreeMap::new(),
        };
        repo_config
            .severity_overrides
            .insert("unwrap_in_non_test_code".to_string(), Severity::Error);

        apply_repository_config(&mut findings, &repo_config, std::path::Path::new("."));

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "unwrap_in_non_test_code");
        assert_eq!(findings[0].severity, Severity::Error);
    }

    #[test]
    fn disables_async_rollout_rules_when_flag_is_off() {
        let mut findings = vec![
            sample_finding("rust_async_std_mutex_await", Severity::Error),
            sample_finding("rust_lock_across_await", Severity::Warning),
            sample_finding("unwrap_in_non_test_code", Severity::Warning),
        ];
        let repo_config = RepoConfig {
            rust_async_experimental: false,
            disabled_rules: Vec::new(),
            suppressed_paths: Vec::new(),
            severity_overrides: std::collections::BTreeMap::new(),
        };

        apply_repository_config(&mut findings, &repo_config, std::path::Path::new("."));

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "unwrap_in_non_test_code");
    }

    #[test]
    fn suppresses_findings_under_configured_paths() {
        let root = temp_dir("suppressed-paths");
        let mut findings = vec![
            Finding {
                path: root.join("tests/fixtures/rust/async/positive.rs"),
                ..sample_finding("rust_blocking_io_in_async", Severity::Warning)
            },
            Finding {
                path: root.join("src/lib.rs"),
                ..sample_finding("unwrap_in_non_test_code", Severity::Warning)
            },
        ];
        let repo_config = RepoConfig {
            rust_async_experimental: true,
            disabled_rules: Vec::new(),
            suppressed_paths: vec![std::path::PathBuf::from("tests/fixtures")],
            severity_overrides: std::collections::BTreeMap::new(),
        };

        apply_repository_config(&mut findings, &repo_config, &root);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].path, root.join("src/lib.rs"));

        fs::remove_dir_all(root).expect("scan temp dir should be removed");
    }

    #[test]
    fn scan_uses_canonical_root_for_index_resolution() {
        let root = temp_dir("canonical-root");
        let src = root.join("src");
        let config = src.join("config");
        fs::create_dir_all(&config).expect("config dir should be created");
        fs::write(
            src.join("lib.rs"),
            "use crate::config::render::normalize as normalize_fn;\n\npub fn run() {\n    normalize_fn();\n}\n",
        )
        .expect("lib fixture should be written");
        fs::write(config.join("render.rs"), "pub fn normalize() {}\n")
            .expect("render fixture should be written");

        let report = scan_repository(&ScanOptions {
            root: root.join("."),
            respect_ignore: true,
        })
        .expect("scan should succeed");

        assert!(!report.findings.iter().any(|finding| {
            finding.rule_id == "hallucinated_import_call"
                && finding.function_name.as_deref() == Some("run")
        }));

        fs::remove_dir_all(root).expect("scan temp dir should be removed");
    }
}
