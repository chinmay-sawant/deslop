use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{ImportSpec, ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::{Finding, Severity};

const SUSPICIOUS_GENERIC_NAMES: &[&str] = &[
    "processdata",
    "handlerequest",
    "executetask",
    "convertvalue",
    "validateinput",
    "transformdata",
    "parsedata",
    "formatresponse",
    "processrequest",
];

const GO_BUILTINS: &[&str] = &[
    "append", "cap", "clear", "close", "complex", "copy", "delete", "imag", "len",
    "make", "max", "min", "new", "panic", "print", "println", "real", "recover",
];

pub(crate) fn evaluate_findings(files: &[ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    for file in files {
        for function in &file.functions {
            if let Some(finding) = generic_name_finding(file, function) {
                findings.push(finding);
            }

            if let Some(finding) = weak_typing_finding(file, function) {
                findings.push(finding);
            }

            findings.extend(local_hallucination_findings(file, function, index));
        }
    }

    findings.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    findings
}

fn generic_name_finding(file: &ParsedFile, function: &ParsedFunction) -> Option<Finding> {
    let normalized = normalize_name(&function.fingerprint.name);
    if !is_generic_name(&normalized) {
        return None;
    }

    let mut evidence = Vec::new();

    if function.fingerprint.contains_any_type || function.fingerprint.contains_empty_interface {
        evidence.push("uses vague signature types".to_string());
    }
    if function.fingerprint.symmetry_score >= 0.5 {
        evidence.push(format!(
            "high structural symmetry ({:.2})",
            function.fingerprint.symmetry_score
        ));
    }
    if function.fingerprint.comment_to_code_ratio <= 0.15 && function.fingerprint.line_count >= 5 {
        evidence.push(format!(
            "low comment specificity ({:.2})",
            function.fingerprint.comment_to_code_ratio
        ));
    }
    if function.fingerprint.type_assertion_count == 0
        && (function.fingerprint.contains_any_type || function.fingerprint.contains_empty_interface)
    {
        evidence.push("no narrowing type assertions found".to_string());
    }

    if evidence.is_empty() {
        return None;
    }

    Some(Finding {
        rule_id: "generic_name".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} uses a generic name without strong domain-specific signals",
            function.fingerprint.name
        ),
        evidence,
    })
}

fn weak_typing_finding(file: &ParsedFile, function: &ParsedFunction) -> Option<Finding> {
    if !function.fingerprint.contains_any_type && !function.fingerprint.contains_empty_interface {
        return None;
    }

    let severity = if function.fingerprint.type_assertion_count == 0 {
        Severity::Warning
    } else {
        Severity::Info
    };

    let mut evidence = Vec::new();
    if function.fingerprint.contains_any_type {
        evidence.push("signature contains any".to_string());
    }
    if function.fingerprint.contains_empty_interface {
        evidence.push("signature contains interface{}".to_string());
    }
    evidence.push(format!(
        "type assertions observed: {}",
        function.fingerprint.type_assertion_count
    ));

    Some(Finding {
        rule_id: "weak_typing".to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} relies on weakly typed inputs or outputs",
            function.fingerprint.name
        ),
        evidence,
    })
}

fn local_hallucination_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let package_name = match &file.package_name {
        Some(package_name) => package_name,
        None => return findings,
    };
    let Some(current_package) = index.package(package_name) else {
        return findings;
    };

    let import_aliases = import_alias_lookup(&file.imports);

    for call in &function.calls {
        match &call.receiver {
            Some(receiver) => {
                if let Some(import_path) = import_aliases.get(receiver) {
                    let Some(target_package) = index.resolve_import_alias(receiver) else {
                        continue;
                    };

                    if !target_package.has_function(&call.name) {
                        findings.push(Finding {
                            rule_id: "hallucinated_import_call".to_string(),
                            severity: Severity::Warning,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: call.line,
                            end_line: call.line,
                            message: format!(
                                "call to {}.{} has no matching symbol in locally indexed package {}",
                                receiver, call.name, import_path
                            ),
                            evidence: vec![
                                format!("import alias {} resolves to {}", receiver, import_path),
                                format!("locally indexed package {} does not expose {}", target_package.package_name, call.name),
                            ],
                        });
                    }
                } else if current_package.has_method(receiver, &call.name) {
                    continue;
                }
            }
            None => {
                if is_builtin(&call.name) || !looks_like_global_symbol(&call.name) {
                    continue;
                }

                if !current_package.has_function(&call.name) {
                    findings.push(Finding {
                        rule_id: "hallucinated_local_call".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: call.line,
                        end_line: call.line,
                        message: format!(
                            "call to {} has no matching symbol in package {}",
                            call.name, package_name
                        ),
                        evidence: vec![format!(
                            "package {} was indexed locally but {} was not found",
                            package_name, call.name
                        )],
                    });
                }
            }
        }
    }

    findings
}

fn import_alias_lookup(imports: &[ImportSpec]) -> BTreeMap<String, String> {
    imports
        .iter()
        .map(|import| (import.alias.clone(), import.path.clone()))
        .collect()
}

fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

fn is_generic_name(name: &str) -> bool {
    if SUSPICIOUS_GENERIC_NAMES.contains(&name) {
        return true;
    }

    let generic_tokens = BTreeSet::from([
        "process", "handle", "execute", "convert", "validate", "transform", "parse",
        "format", "request", "response", "data", "input", "output", "task", "value",
    ]);
    generic_tokens.iter().filter(|token| name.contains(*token)).count() >= 2
}

fn is_builtin(name: &str) -> bool {
    GO_BUILTINS.contains(&name)
}

fn looks_like_global_symbol(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}

#[cfg(test)]
mod tests {
    use super::{is_generic_name, looks_like_global_symbol, normalize_name};

    #[test]
    fn detects_generic_names() {
        assert!(is_generic_name(&normalize_name("processData")));
        assert!(is_generic_name(&normalize_name("formatResponse")));
        assert!(!is_generic_name(&normalize_name("BuildCustomerLedger")));
    }

    #[test]
    fn exported_names_look_global() {
        assert!(looks_like_global_symbol("SanitizeEmail"));
        assert!(!looks_like_global_symbol("sanitizeEmail"));
    }
}