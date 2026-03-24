use std::collections::BTreeMap;

use crate::analysis::{NamedLiteral, ParsedFile};
use crate::model::{Finding, Severity};

const REPEATED_LITERAL_THRESHOLD: usize = 3;
const REPEATED_LITERAL_MIN_LENGTH: usize = 12;

pub(super) fn repeated_string_literal_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let mut occurrences: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for literal in literal_iter(file) {
        if literal.value.len() < REPEATED_LITERAL_MIN_LENGTH {
            continue;
        }

        occurrences
            .entry(literal.value.clone())
            .or_default()
            .push(literal.line);
    }

    occurrences
        .into_iter()
        .filter(|(_, lines)| lines.len() >= REPEATED_LITERAL_THRESHOLD)
        .map(|(value, mut lines)| {
            lines.sort_unstable();
            Finding {
                rule_id: "repeated_string_literal".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: lines[0],
                end_line: lines[lines.len() - 1],
                message: "file repeats the same long string literal instead of centralizing it"
                    .to_string(),
                evidence: vec![
                    format!("occurrences={}", lines.len()),
                    format!("literal={}", preview_literal(&value)),
                ],
            }
        })
        .collect()
}

pub(super) fn repeated_exception_block_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let mut occurrences: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for function in &file.functions {
        for block in &function.exception_block_signatures {
            occurrences
                .entry(block.signature.clone())
                .or_default()
                .push(block.line);
        }
    }

    occurrences
        .into_iter()
        .filter(|(_, lines)| lines.len() >= 2)
        .map(|(signature, mut lines)| {
            lines.sort_unstable();
            Finding {
                rule_id: "duplicate_error_handler_block".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: lines[0],
                end_line: lines[lines.len() - 1],
                message: "file repeats highly similar exception-handling blocks".to_string(),
                evidence: vec![
                    format!("occurrences={}", lines.len()),
                    format!("shape={signature}"),
                ],
            }
        })
        .collect()
}

pub(super) fn repeated_validation_pipeline_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let mut occurrences: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for function in &file.functions {
        if let Some(signature) = &function.validation_signature {
            occurrences
                .entry(signature.signature.clone())
                .or_default()
                .push(function.fingerprint.start_line);
        }
    }

    occurrences
        .into_iter()
        .filter(|(_, lines)| lines.len() >= 2)
        .map(|(signature, mut lines)| {
            lines.sort_unstable();
            Finding {
                rule_id: "duplicate_validation_pipeline".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: lines[0],
                end_line: lines[lines.len() - 1],
                message: "file repeats the same validation pipeline across functions".to_string(),
                evidence: vec![
                    format!("occurrences={}", lines.len()),
                    format!("shape={signature}"),
                ],
            }
        })
        .collect()
}

pub(super) fn duplicate_test_utility_logic_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut shapes = BTreeMap::<String, Vec<(&ParsedFile, String, usize, bool)>>::new();

    for file in files {
        for function in &file.functions {
            if function.normalized_body.len() < 40 || function.fingerprint.line_count < 3 {
                continue;
            }

            shapes
                .entry(function.normalized_body.clone())
                .or_default()
                .push((
                    *file,
                    function.fingerprint.name.clone(),
                    function.fingerprint.start_line,
                    file.is_test_file || function.is_test_function,
                ));
        }
    }

    let mut findings = Vec::new();
    for records in shapes.into_values() {
        let has_test = records.iter().any(|(_, _, _, is_test)| *is_test);
        let has_prod = records.iter().any(|(_, _, _, is_test)| !*is_test);
        if !has_test || !has_prod {
            continue;
        }

        let anchor = records
            .iter()
            .min_by(|left, right| left.0.path.cmp(&right.0.path).then(left.2.cmp(&right.2)))
            .expect("duplicate logic group should not be empty");
        findings.push(Finding {
            rule_id: "duplicate_test_utility_logic".to_string(),
            severity: Severity::Info,
            path: anchor.0.path.clone(),
            function_name: Some(anchor.1.clone()),
            start_line: anchor.2,
            end_line: anchor.2,
            message: "test and production code share highly similar utility logic".to_string(),
            evidence: vec![format!("matching_function_shapes={}", records.len())],
        });
    }

    findings
}

pub(super) fn cross_file_repeated_literal_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut occurrences: BTreeMap<String, Vec<(&ParsedFile, usize)>> = BTreeMap::new();
    for file in files {
        if file.is_test_file {
            continue;
        }
        for literal in literal_iter(file) {
            if literal.value.len() < REPEATED_LITERAL_MIN_LENGTH {
                continue;
            }
            occurrences
                .entry(literal.value.clone())
                .or_default()
                .push((file, literal.line));
        }
    }

    let mut findings = Vec::new();
    for (value, records) in occurrences {
        let distinct_files = records
            .iter()
            .map(|(file, _)| file.path.clone())
            .collect::<std::collections::BTreeSet<_>>();
        if distinct_files.len() < 2 || records.len() < 4 {
            continue;
        }

        let anchor = records
            .iter()
            .min_by(|left, right| left.0.path.cmp(&right.0.path).then(left.1.cmp(&right.1)))
            .expect("cross-file literal cluster should not be empty");
        findings.push(Finding {
            rule_id: "cross_file_repeated_literal".to_string(),
            severity: Severity::Info,
            path: anchor.0.path.clone(),
            function_name: None,
            start_line: anchor.1,
            end_line: anchor.1,
            message: "repository repeats the same long string literal across multiple files"
                .to_string(),
            evidence: vec![
                format!("occurrences={}", records.len()),
                format!("files={}", distinct_files.len()),
                format!("literal={}", preview_literal(&value)),
            ],
        });
    }

    findings
}

fn literal_iter(file: &ParsedFile) -> impl Iterator<Item = &NamedLiteral> {
    file.pkg_strings
        .iter()
        .chain(file.functions.iter().flat_map(|function| function.local_strings.iter()))
}

fn preview_literal(value: &str) -> String {
    const PREVIEW_LIMIT: usize = 40;
    if value.len() <= PREVIEW_LIMIT {
        return value.to_string();
    }
    format!("{}...", &value[..PREVIEW_LIMIT])
}