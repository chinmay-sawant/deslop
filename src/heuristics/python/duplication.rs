use std::collections::BTreeMap;

use crate::analysis::{NamedLiteral, ParsedFile};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

const REPEATED_LITERAL_THRESHOLD: usize = 3;
const REPEATED_LITERAL_MIN_LENGTH: usize = 12;
const DUPLICATE_QUERY_FRAGMENT_THRESHOLD: usize = 3;
const DUPLICATE_QUERY_FRAGMENT_MIN_TOKENS: usize = 6;
const CROSS_FILE_COPY_PASTE_MIN_BODY_LENGTH: usize = 60;
const CROSS_FILE_COPY_PASTE_MIN_LINE_COUNT: usize = 4;
const TRANSFORMATION_PIPELINE_MIN_STAGES: usize = 4;

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
        for block in function.python_evidence().exception_block_signatures {
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
        if let Some(signature) = function.python_evidence().validation_signature {
            occurrences
                .entry(signature.signature.clone())
                .or_default()
                .push(function.fingerprint.start_line);
        }
    }

    occurrences
        .into_iter()
        .filter_map(|(signature, mut lines)| {
            if lines.len() < 2 {
                return None;
            }

            let matching_functions: Vec<_> = file
                .functions
                .iter()
                .filter(|function| {
                    function
                        .python_evidence()
                        .validation_signature
                        .is_some_and(|block| block.signature == signature)
                })
                .collect();

            if !matching_functions
                .iter()
                .any(|function| !function.fingerprint.name.starts_with('_'))
            {
                return None;
            }

            lines.sort_unstable();
            Some(Finding {
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
            })
        })
        .collect()
}

pub(super) fn test_utility_logic_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut shapes = BTreeMap::<String, Vec<(&ParsedFile, String, usize, bool)>>::new();

    for file in files {
        for function in &file.functions {
            let norm = function.python_evidence().normalized_body;
            if norm.len() < 40 || function.fingerprint.line_count < 3 {
                continue;
            }

            shapes.entry(norm.to_owned()).or_default().push((
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

        let Some(anchor) = records
            .iter()
            .min_by(|left, right| left.0.path.cmp(&right.0.path).then(left.2.cmp(&right.2)))
        else {
            continue;
        };
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

pub(super) fn cross_file_dupe_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut shapes = BTreeMap::<String, Vec<(&ParsedFile, String, usize)>>::new();

    for file in files {
        if file.is_test_file {
            continue;
        }

        for function in &file.functions {
            if function.is_test_function {
                continue;
            }
            let norm = function.python_evidence().normalized_body;
            if norm.len() < CROSS_FILE_COPY_PASTE_MIN_BODY_LENGTH
                || function.fingerprint.line_count < CROSS_FILE_COPY_PASTE_MIN_LINE_COUNT
            {
                continue;
            }

            shapes.entry(norm.to_owned()).or_default().push((
                *file,
                function.fingerprint.name.clone(),
                function.fingerprint.start_line,
            ));
        }
    }

    let mut findings = Vec::new();
    for records in shapes.into_values() {
        let distinct_files = records
            .iter()
            .map(|(file, _, _)| file.path.clone())
            .collect::<std::collections::BTreeSet<_>>();
        if distinct_files.len() < 2 {
            continue;
        }

        let Some(anchor) = records
            .iter()
            .min_by(|left, right| left.0.path.cmp(&right.0.path).then(left.2.cmp(&right.2)))
        else {
            continue;
        };
        findings.push(Finding {
            rule_id: "cross_file_copy_paste_function".to_string(),
            severity: Severity::Info,
            path: anchor.0.path.clone(),
            function_name: Some(anchor.1.clone()),
            start_line: anchor.2,
            end_line: anchor.2,
            message: "repository repeats a highly similar non-test function body across files"
                .to_string(),
            evidence: vec![
                format!("matching_function_shapes={}", records.len()),
                format!("files={}", distinct_files.len()),
            ],
        });
    }

    findings
}

pub(super) fn duplicate_transformation_pipeline_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut pipelines = BTreeMap::<String, Vec<(&ParsedFile, String, usize)>>::new();

    for file in files {
        if file.is_test_file {
            continue;
        }

        for function in &file.functions {
            if function.is_test_function {
                continue;
            }

            let Some(signature) = transform_pipeline_sig(file, function) else {
                continue;
            };
            pipelines.entry(signature).or_default().push((
                *file,
                function.fingerprint.name.clone(),
                function.fingerprint.start_line,
            ));
        }
    }

    let mut findings = Vec::new();
    for (signature, records) in pipelines {
        let distinct_files = records
            .iter()
            .map(|(file, _, _)| file.path.clone())
            .collect::<std::collections::BTreeSet<_>>();
        if distinct_files.len() < 2 || records.len() < 2 {
            continue;
        }

        let Some(anchor) = records
            .iter()
            .min_by(|left, right| left.0.path.cmp(&right.0.path).then(left.2.cmp(&right.2)))
        else {
            continue;
        };
        findings.push(Finding {
            rule_id: "duplicate_transformation_pipeline".to_string(),
            severity: Severity::Info,
            path: anchor.0.path.clone(),
            function_name: Some(anchor.1.clone()),
            start_line: anchor.2,
            end_line: anchor.2,
            message: "repository repeats the same transformation pipeline across functions"
                .to_string(),
            evidence: vec![
                format!("occurrences={}", records.len()),
                format!("files={}", distinct_files.len()),
                format!("stages={signature}"),
            ],
        });
    }

    findings
}

pub(super) fn cross_file_literal_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut occurrences: BTreeMap<String, Vec<(&ParsedFile, usize)>> = BTreeMap::new();
    for file in files {
        if file.is_test_file {
            continue;
        }
        for literal in literal_iter(file) {
            if literal.value.len() < REPEATED_LITERAL_MIN_LENGTH
                || normalize_query_fragment(&literal.value).is_some()
            {
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

        let Some(anchor) = records
            .iter()
            .min_by(|left, right| left.0.path.cmp(&right.0.path).then(left.1.cmp(&right.1)))
        else {
            continue;
        };
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

pub(super) fn duplicate_query_fragment_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut occurrences: BTreeMap<String, Vec<(&ParsedFile, usize, String)>> = BTreeMap::new();
    for file in files {
        if file.is_test_file || skip_query_fragment_file(&file.path) {
            continue;
        }

        for literal in literal_iter(file) {
            let Some(normalized) = normalize_query_fragment(&literal.value) else {
                continue;
            };

            occurrences.entry(normalized).or_default().push((
                file,
                literal.line,
                literal.value.clone(),
            ));
        }
    }

    let mut findings = Vec::new();
    for (normalized, records) in occurrences {
        let distinct_files = records
            .iter()
            .map(|(file, _, _)| file.path.clone())
            .collect::<std::collections::BTreeSet<_>>();
        if distinct_files.len() < 2 || records.len() < DUPLICATE_QUERY_FRAGMENT_THRESHOLD {
            continue;
        }

        let Some(anchor) = records
            .iter()
            .min_by(|left, right| left.0.path.cmp(&right.0.path).then(left.1.cmp(&right.1)))
        else {
            continue;
        };
        findings.push(Finding {
            rule_id: "duplicate_query_fragment".to_string(),
            severity: Severity::Info,
            path: anchor.0.path.clone(),
            function_name: None,
            start_line: anchor.1,
            end_line: anchor.1,
            message: "repository repeats the same query-like string fragment across multiple files"
                .to_string(),
            evidence: vec![
                format!("occurrences={}", records.len()),
                format!("files={}", distinct_files.len()),
                format!("query={}", preview_literal(&normalized)),
            ],
        });
    }

    findings
}

fn literal_iter(file: &ParsedFile) -> impl Iterator<Item = &NamedLiteral> {
    file.pkg_strings.iter().chain(
        file.functions
            .iter()
            .flat_map(|function| function.local_strings.iter()),
    )
}

fn transform_pipeline_sig(
    file: &ParsedFile,
    function: &crate::analysis::ParsedFunction,
) -> Option<String> {
    let alias_lookup = file
        .imports
        .iter()
        .map(|import| (import.alias.as_str(), import.path.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut staged_calls = function
        .calls
        .iter()
        .filter_map(|call| {
            let receiver = call.receiver.as_deref().unwrap_or(call.name.as_str());
            let import_path = alias_lookup.get(receiver).copied().unwrap_or(receiver);
            classify_pipeline_stage(import_path, &call.name).map(|stage| (call.line, stage))
        })
        .collect::<Vec<_>>();
    staged_calls.sort_by(|left, right| left.0.cmp(&right.0));

    let mut stages = Vec::<&'static str>::new();
    if function.python_evidence().validation_signature.is_some() {
        stages.push("validate");
    }
    for (_, stage) in staged_calls {
        if stages.last() == Some(&stage) {
            continue;
        }
        stages.push(stage);
    }

    let distinct_stage_count = stages
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    (stages.len() >= TRANSFORMATION_PIPELINE_MIN_STAGES && distinct_stage_count >= 3)
        .then(|| stages.join("->"))
}

fn classify_pipeline_stage(import_path: &str, call_name: &str) -> Option<&'static str> {
    let normalized_call = call_name.to_ascii_lowercase();
    let normalized_import = import_path.to_ascii_lowercase();

    if normalized_import.starts_with("json")
        || normalized_import.starts_with("yaml")
        || normalized_import.starts_with("csv")
        || stage_name_matches(&normalized_call, &["load", "loads", "parse", "decode"])
    {
        return Some("parse");
    }

    if stage_name_matches(&normalized_call, &["validate", "verify", "check", "ensure"]) {
        return Some("validate");
    }

    if stage_name_matches(
        &normalized_call,
        &["normalize", "transform", "convert", "map", "clean"],
    ) {
        return Some("transform");
    }

    if stage_name_matches(&normalized_call, &["filter", "exclude", "select"]) {
        return Some("filter");
    }

    if stage_name_matches(
        &normalized_call,
        &["fetch", "lookup", "get", "request", "enrich", "join"],
    ) {
        return Some("enrich");
    }

    if stage_name_matches(
        &normalized_call,
        &["sum", "reduce", "aggregate", "count", "groupby"],
    ) {
        return Some("aggregate");
    }

    if normalized_import.starts_with("json")
        || stage_name_matches(
            &normalized_call,
            &[
                "dump",
                "dumps",
                "serialize",
                "render",
                "write",
                "write_text",
            ],
        )
    {
        return Some("serialize");
    }

    None
}

fn stage_name_matches(call_name: &str, prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|prefix| call_name == *prefix || call_name.starts_with(&format!("{prefix}_")))
}

use std::path::Path;
fn skip_query_fragment_file(path: &Path) -> bool {
    path.components().any(|component| {
        let part = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        matches!(
            part.as_str(),
            "migration" | "migrations" | "alembic" | "versions"
        ) || part.starts_with("migration_")
            || part.ends_with("_migration.py")
            || part.ends_with("_migrations.py")
    })
}

fn normalize_query_fragment(value: &str) -> Option<String> {
    let collapsed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() < REPEATED_LITERAL_MIN_LENGTH {
        return None;
    }

    let upper = collapsed.to_ascii_uppercase();
    let keyword_count = [
        "SELECT", "FROM", "WHERE", "JOIN", "UPDATE", "INSERT", "DELETE", "ORDER BY", "GROUP BY",
        "LIMIT",
    ]
    .into_iter()
    .filter(|keyword| upper.contains(keyword))
    .count();
    if keyword_count < 2 {
        return None;
    }

    let normalized_tokens = collapsed
        .split_whitespace()
        .map(normalize_query_token)
        .collect::<Vec<_>>();
    if normalized_tokens.len() < DUPLICATE_QUERY_FRAGMENT_MIN_TOKENS {
        return None;
    }

    Some(normalized_tokens.join(" "))
}

fn normalize_query_token(token: &str) -> String {
    let trimmed = token.trim_matches(|ch: char| matches!(ch, ',' | ';' | '(' | ')'));
    if trimmed.starts_with('"')
        || trimmed.ends_with('"')
        || trimmed.starts_with('\'')
        || trimmed.ends_with('\'')
        || trimmed.chars().all(|ch| ch.is_ascii_digit())
    {
        return "?".to_string();
    }

    trimmed.to_ascii_uppercase()
}

fn preview_literal(value: &str) -> String {
    const PREVIEW_LIMIT: usize = 40;
    if value.len() <= PREVIEW_LIMIT {
        return value.to_string();
    }
    format!(
        "{}...",
        value.chars().take(PREVIEW_LIMIT).collect::<String>()
    )
}
