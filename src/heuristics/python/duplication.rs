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