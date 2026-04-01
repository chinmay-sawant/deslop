use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::model::Finding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SuppressionDirective {
    pub rule_id: String,
    pub line: usize,
    pub next_code_line: Option<usize>,
}

pub(super) fn is_suppressed(
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

pub(super) fn parse_suppression_directives(source: &str) -> Vec<SuppressionDirective> {
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

pub(super) fn parse_rule_ids(tail: &str) -> Vec<String> {
    tail.split([',', ' ', '\t'])
        .filter_map(|token| {
            let trimmed = token.trim_matches(|character: char| {
                !matches!(character, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-')
            });
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect()
}

pub(super) fn next_code_line(lines: &[&str], start_index: usize) -> Option<usize> {
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
