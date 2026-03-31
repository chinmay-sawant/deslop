mod data_access;
mod gin;
mod hot_path;

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use self::data_access::data_access_performance_findings;
use self::gin::gin_request_performance_findings;
use self::hot_path::core_hot_path_findings as run_hot_path_findings;

#[derive(Debug, Clone)]
pub(super) struct BodyLine {
    pub(super) line: usize,
    pub(super) text: String,
    pub(super) in_loop: bool,
}

pub(super) const LARGE_MULTIPART_FORM_BYTES: u64 = 32 * 1024 * 1024;

pub(super) fn go_advanceplan3_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for function in &file.functions {
        findings.extend(go_advanceplan3_function_findings(file, function));
    }

    findings
}

fn go_advanceplan3_function_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if file.is_test_file || function.is_test_function {
        return Vec::new();
    }

    let mut findings = core_hot_path_findings(file, function);
    findings.extend(data_access_performance_findings(file, function));
    findings.extend(gin_request_performance_findings(file, function));
    findings
}

fn core_hot_path_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    run_hot_path_findings(file, function)
}

pub(super) fn is_request_path_function(file: &ParsedFile, function: &ParsedFunction) -> bool {
    is_gin_handler(file, function) || is_http_handler(file, function)
}

pub(super) fn is_gin_handler(file: &ParsedFile, function: &ParsedFunction) -> bool {
    import_aliases_for(file, "github.com/gin-gonic/gin")
        .into_iter()
        .any(|alias| {
            function
                .signature_text
                .contains(&format!("*{alias}.Context"))
        })
}

pub(super) fn is_http_handler(file: &ParsedFile, function: &ParsedFunction) -> bool {
    import_aliases_for(file, "net/http")
        .into_iter()
        .any(|alias| {
            function
                .signature_text
                .contains(&format!("{alias}.ResponseWriter"))
                && function
                    .signature_text
                    .contains(&format!("*{alias}.Request"))
        })
}

pub(super) fn has_sql_like_import(file: &ParsedFile) -> bool {
    [
        "database/sql",
        "github.com/jmoiron/sqlx",
        "github.com/jackc/pgx/v5",
        "github.com/jackc/pgx/v5/pgxpool",
        "github.com/jackc/pgx/v4",
        "github.com/jackc/pgx/v4/pgxpool",
    ]
    .iter()
    .any(|path| has_import_path(file, path))
}

pub(super) fn has_import_path(file: &ParsedFile, path: &str) -> bool {
    file.imports.iter().any(|import| import.path == path)
}

pub(super) fn binding_matches(
    lines: &[BodyLine],
    patterns: &[&str],
) -> Vec<(String, usize, String)> {
    let mut matches = Vec::new();

    for body_line in lines {
        if let Some((name, target)) = binding_for_patterns(&body_line.text, patterns) {
            matches.push((name, body_line.line, target));
        }
    }

    matches
}

pub(super) fn binding_for_patterns(text: &str, patterns: &[&str]) -> Option<(String, String)> {
    let (left, right) = split_assignment(text)?;
    let target = patterns
        .iter()
        .find(|pattern| right.contains(**pattern))?
        .to_string();
    let binding = left
        .trim()
        .trim_start_matches("var ")
        .split(',')
        .next()?
        .split_whitespace()
        .next()?
        .trim();
    is_identifier_name(binding).then(|| (binding.to_string(), target))
}

pub(super) fn split_assignment(text: &str) -> Option<(&str, &str)> {
    if let Some((left, right)) = text.split_once(":=") {
        return Some((left, right));
    }

    if text.contains("==") || text.contains("!=") || text.contains("<=") || text.contains(">=") {
        return None;
    }

    text.split_once(" = ")
        .or_else(|| text.split_once('='))
        .filter(|(left, _)| !left.trim_start().starts_with("if "))
}

pub(super) fn body_lines(function: &ParsedFunction) -> Vec<BodyLine> {
    let mut brace_depth = 0usize;
    let mut loop_exit_depths = Vec::new();
    let mut lines = Vec::new();

    for (offset, raw_line) in function.body_text.lines().enumerate() {
        let line_no = function.body_start_line + offset;
        let stripped = strip_line_comment(raw_line).trim().to_string();
        let closing_braces = stripped
            .chars()
            .filter(|character| *character == '}')
            .count();
        for _ in 0..closing_braces {
            brace_depth = brace_depth.saturating_sub(1);
            while loop_exit_depths
                .last()
                .is_some_and(|exit_depth| *exit_depth > brace_depth)
            {
                loop_exit_depths.pop();
            }
        }

        let starts_loop = contains_keyword(&stripped, "for");
        let in_loop = !loop_exit_depths.is_empty() || starts_loop;
        let opening_braces = stripped
            .chars()
            .filter(|character| *character == '{')
            .count();
        if starts_loop {
            loop_exit_depths.push(brace_depth + opening_braces.max(1));
        }

        brace_depth += opening_braces;
        lines.push(BodyLine {
            line: line_no,
            text: stripped,
            in_loop,
        });
    }

    lines
}

pub(super) fn import_aliases_for(file: &ParsedFile, import_path: &str) -> Vec<String> {
    file.imports
        .iter()
        .filter(|import| import.path == import_path)
        .map(|import| import.alias.clone())
        .collect()
}

pub(super) fn json_aliases(file: &ParsedFile) -> Vec<String> {
    import_aliases_for(file, "encoding/json")
}

pub(super) fn first_line_with_any(function: &ParsedFunction, markers: &[&str]) -> Option<usize> {
    function
        .body_text
        .lines()
        .enumerate()
        .find(|(_, line)| markers.iter().any(|marker| line.contains(marker)))
        .map(|(offset, _)| function.body_start_line + offset)
}

pub(super) fn has_prior_loop_line(function: &ParsedFunction, line_no: usize) -> bool {
    function
        .body_text
        .lines()
        .enumerate()
        .take_while(|(offset, _)| function.body_start_line + *offset < line_no)
        .any(|(_, line)| contains_keyword(strip_line_comment(line), "for"))
}

pub(super) fn is_identifier_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && text
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
}

pub(super) fn join_lines(lines: &[usize]) -> String {
    lines
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn contains_keyword(line: &str, keyword: &str) -> bool {
    let bytes = line.as_bytes();
    let keyword_bytes = keyword.as_bytes();

    if keyword_bytes.is_empty() || bytes.len() < keyword_bytes.len() {
        return false;
    }

    for start in 0..=bytes.len() - keyword_bytes.len() {
        if &bytes[start..start + keyword_bytes.len()] != keyword_bytes {
            continue;
        }

        let left_ok =
            start == 0 || (!bytes[start - 1].is_ascii_alphanumeric() && bytes[start - 1] != b'_');
        let right_index = start + keyword_bytes.len();
        let right_ok = right_index == bytes.len()
            || (!bytes[right_index].is_ascii_alphanumeric() && bytes[right_index] != b'_');

        if left_ok && right_ok {
            return true;
        }
    }

    false
}

pub(super) fn strip_line_comment(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

pub(super) fn repeated_parse_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (parser_family, rule_id, payload_label) in [
        (
            "json_unmarshal",
            "json_unmarshal_same_payload_multiple_times",
            "JSON",
        ),
        (
            "xml_unmarshal",
            "xml_unmarshal_same_payload_multiple_times",
            "XML",
        ),
        (
            "yaml_unmarshal",
            "yaml_unmarshal_same_payload_multiple_times",
            "YAML",
        ),
        (
            "proto_unmarshal",
            "proto_unmarshal_same_payload_multiple_times",
            "protobuf",
        ),
    ] {
        findings.extend(repeated_parse_family_findings(
            file,
            function,
            parser_family,
            rule_id,
            payload_label,
        ));
    }

    findings
}

fn repeated_parse_family_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    parser_family: &str,
    rule_id: &str,
    payload_label: &str,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut groups = BTreeMap::<String, Vec<_>>::new();

    for call in go
        .parse_input_calls
        .iter()
        .filter(|call| call.parser_family == parser_family && call.input_binding.is_some())
    {
        groups
            .entry(call.input_binding.clone().unwrap_or_default())
            .or_default()
            .push(call);
    }

    let mut findings = Vec::new();
    for (input_binding, calls) in groups {
        if calls.len() < 2 {
            continue;
        }

        let distinct_targets = calls
            .iter()
            .filter_map(|call| call.target_text.clone())
            .collect::<BTreeSet<_>>();
        if distinct_targets.len() < 2 {
            continue;
        }

        let repeated_lines = calls.iter().map(|call| call.line).collect::<Vec<_>>();
        let anchor_line = repeated_lines[1];
        findings.push(Finding {
            rule_id: rule_id.to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: anchor_line,
            end_line: anchor_line,
            message: format!(
                "function {} unmarshals the same {} payload binding multiple times",
                function.fingerprint.name, payload_label
            ),
            evidence: vec![
                format!(
                    "input binding {input_binding} was unmarshaled at lines {}",
                    join_lines(&repeated_lines)
                ),
                format!("normalized input text: {}", calls[0].input_text),
                format!(
                    "distinct targets observed: {}",
                    distinct_targets.into_iter().collect::<Vec<_>>().join(", ")
                ),
            ],
        });
    }

    findings
}
