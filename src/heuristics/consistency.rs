use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::analysis::ParsedFile;
use crate::model::{Finding, Severity};

pub(super) fn mixed_receiver_kind_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut methods_by_receiver = BTreeMap::<(Option<String>, PathBuf, String), Vec<MethodRecord>>::new();

    for file in files {
        let directory = file
            .path
            .parent()
            .map(|path| path.to_path_buf())
            .unwrap_or_else(PathBuf::new);
        for symbol in &file.symbols {
            let Some(receiver_type) = &symbol.receiver_type else {
                continue;
            };
            let Some(receiver_is_pointer) = symbol.receiver_is_pointer else {
                continue;
            };

            methods_by_receiver
                .entry((file.package_name.clone(), directory.clone(), receiver_type.clone()))
                .or_default()
                .push(MethodRecord {
                    path: file.path.clone(),
                    line: symbol.line,
                    method_name: symbol.name.clone(),
                    receiver_is_pointer,
                });
        }
    }

    let mut findings = Vec::new();

    for ((_, _, receiver_type), methods) in methods_by_receiver {
        let has_pointer = methods.iter().any(|method| method.receiver_is_pointer);
        let has_value = methods.iter().any(|method| !method.receiver_is_pointer);
        if !has_pointer || !has_value {
            continue;
        }

        let anchor = methods
            .iter()
            .min_by(|left, right| left.path.cmp(&right.path).then(left.line.cmp(&right.line)))
            .expect("mixed receiver group should not be empty");

        let pointer_methods = methods
            .iter()
            .filter(|method| method.receiver_is_pointer)
            .map(|method| method.method_name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ");
        let value_methods = methods
            .iter()
            .filter(|method| !method.receiver_is_pointer)
            .map(|method| method.method_name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ");

        findings.push(Finding {
            rule_id: "mixed_receiver_kinds".to_string(),
            severity: Severity::Info,
            path: anchor.path.clone(),
            function_name: Some(anchor.method_name.clone()),
            start_line: anchor.line,
            end_line: anchor.line,
            message: format!(
                "type {receiver_type} mixes pointer and value receivers across methods"
            ),
            evidence: vec![
                format!("pointer receiver methods: {pointer_methods}"),
                format!("value receiver methods: {value_methods}"),
            ],
        });
    }

    findings
}

pub(super) fn struct_tag_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for tag in &file.struct_tags {
        match parse_struct_tag_keys(&tag.raw_tag) {
            None => findings.push(Finding {
                rule_id: "malformed_struct_tag".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: None,
                start_line: tag.line,
                end_line: tag.line,
                message: format!(
                    "struct {} field {} has a malformed tag",
                    tag.struct_name, tag.field_name
                ),
                evidence: vec![format!("raw tag: `{}`", tag.raw_tag)],
            }),
            Some(keys) => {
                let mut seen = BTreeSet::new();
                let duplicates = keys
                    .into_iter()
                    .filter(|key| !seen.insert(key.clone()))
                    .collect::<Vec<_>>();

                if !duplicates.is_empty() {
                    findings.push(Finding {
                        rule_id: "duplicate_struct_tag_key".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: None,
                        start_line: tag.line,
                        end_line: tag.line,
                        message: format!(
                            "struct {} field {} repeats struct tag keys",
                            tag.struct_name, tag.field_name
                        ),
                        evidence: vec![
                            format!("raw tag: `{}`", tag.raw_tag),
                            format!("duplicate keys: {}", duplicates.join(", ")),
                        ],
                    });
                }
            }
        }
    }

    findings
}

#[derive(Debug, Clone)]
struct MethodRecord {
    path: PathBuf,
    line: usize,
    method_name: String,
    receiver_is_pointer: bool,
}

fn parse_struct_tag_keys(raw_tag: &str) -> Option<Vec<String>> {
    let mut keys = Vec::new();
    let bytes = raw_tag.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }

        let key_start = index;
        while index < bytes.len() && bytes[index].is_ascii_alphanumeric() || index < bytes.len() && bytes[index] == b'_' {
            index += 1;
        }
        if key_start == index || index >= bytes.len() || bytes[index] != b':' {
            return None;
        }
        let key = raw_tag.get(key_start..index)?.to_string();
        index += 1;
        if index >= bytes.len() || bytes[index] != b'"' {
            return None;
        }

        index += 1;
        let mut escaped = false;
        while index < bytes.len() {
            match bytes[index] {
                b'\\' if !escaped => escaped = true,
                b'"' if !escaped => break,
                _ => escaped = false,
            }
            index += 1;
        }

        if index >= bytes.len() || bytes[index] != b'"' {
            return None;
        }
        index += 1;
        keys.push(key);
    }

    Some(keys)
}