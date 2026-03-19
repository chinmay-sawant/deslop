use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{ImportSpec, ParsedFile, ParsedFunction};
use crate::index::{ImportResolution, RepositoryIndex};
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
    "append", "cap", "clear", "close", "complex", "copy", "delete", "imag", "len", "make", "max",
    "min", "new", "panic", "print", "println", "real", "recover",
];

const WEAK_CRYPTO_IMPORTS: &[&str] = &["crypto/md5", "crypto/sha1", "crypto/des", "crypto/rc4"];
const HTTP_CONTEXTLESS_CALLS: &[&str] = &["Get", "Head", "Post", "PostForm", "NewRequest"];
const EXEC_CONTEXTLESS_CALLS: &[&str] = &["Command"];
const NET_CONTEXTLESS_CALLS: &[&str] = &["Dial", "DialTimeout"];
const COORDINATION_METHODS: &[&str] = &["Add", "Done", "Wait", "Go"];

pub(crate) fn evaluate_findings(files: &[ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    for file in files {
        for function in &file.functions {
            if let Some(finding) = generic_name_finding(file, function) {
                findings.push(finding);
            }

            if let Some(finding) = overlong_name_finding(file, function) {
                findings.push(finding);
            }

            if let Some(finding) = weak_typing_finding(file, function) {
                findings.push(finding);
            }

            findings.extend(error_handling_findings(file, function));
            findings.extend(comment_style_findings(file, function));
            findings.extend(weak_crypto_findings(file, function));
            findings.extend(missing_context_findings(file, function));
            findings.extend(sleep_polling_findings(file, function));
            findings.extend(string_concat_in_loop_findings(file, function));
            findings.extend(goroutine_coordination_findings(file, function));

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

fn overlong_name_finding(file: &ParsedFile, function: &ParsedFunction) -> Option<Finding> {
    let token_count = identifier_token_count(&function.fingerprint.name);
    if function.fingerprint.name.len() < 24 || token_count < 4 {
        return None;
    }

    Some(Finding {
        rule_id: "overlong_name".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} uses an overly descriptive name",
            function.fingerprint.name
        ),
        evidence: vec![
            format!(
                "identifier length: {} characters",
                function.fingerprint.name.len()
            ),
            format!("identifier token count: {token_count}"),
        ],
    })
}

fn generic_name_finding(file: &ParsedFile, function: &ParsedFunction) -> Option<Finding> {
    let normalized = normalize_name(&function.fingerprint.name);
    if !is_generic_name(&normalized) {
        return None;
    }

    let mut evidence = Vec::new();
    let has_weak_typing =
        function.fingerprint.contains_any_type || function.fingerprint.contains_empty_interface;
    let has_high_symmetry = function.fingerprint.symmetry_score >= 0.5;
    let has_low_comment_specificity =
        function.fingerprint.comment_to_code_ratio <= 0.15 && function.fingerprint.line_count >= 5;

    if has_weak_typing {
        evidence.push("uses vague signature types".to_string());
    }
    if has_high_symmetry {
        evidence.push(format!(
            "high structural symmetry ({:.2})",
            function.fingerprint.symmetry_score
        ));
    }
    if has_low_comment_specificity {
        evidence.push(format!(
            "low comment specificity ({:.2})",
            function.fingerprint.comment_to_code_ratio
        ));
    }
    if function.fingerprint.type_assertion_count == 0 && has_weak_typing {
        evidence.push("no narrowing type assertions found".to_string());
    }

    if !has_weak_typing && !has_high_symmetry {
        return None;
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

fn error_handling_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();

    for line in &function.dropped_error_lines {
        findings.push(Finding {
            rule_id: "dropped_error".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} discards an error-like value with the blank identifier",
                function.fingerprint.name
            ),
            evidence: vec!["blank identifier assignment drops an err value".to_string()],
        });
    }

    for line in &function.panic_on_error_lines {
        findings.push(Finding {
            rule_id: "panic_on_error".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} escalates ordinary error handling through panic or fatal logging",
                function.fingerprint.name
            ),
            evidence: vec![
                "err != nil branch contains panic or log.Fatal style handling".to_string(),
            ],
        });
    }

    for call in &function.errorf_calls {
        if !call.mentions_err || call.uses_percent_w {
            continue;
        }

        let mut evidence = Vec::new();
        if let Some(format_string) = &call.format_string {
            evidence.push(format!("fmt.Errorf format string: {format_string}"));
        }
        evidence.push("call mentions err but does not use %w wrapping".to_string());

        findings.push(Finding {
            rule_id: "error_wrapping_misuse".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} uses fmt.Errorf without %w while referencing err",
                function.fingerprint.name
            ),
            evidence,
        });
    }

    findings
}

fn comment_style_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    let Some(doc_comment) = &function.doc_comment else {
        return findings;
    };

    let first_line = doc_comment
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");

    if is_title_case_comment(first_line) {
        findings.push(Finding {
            rule_id: "comment_style_title_case".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "function {} uses Title Case documentation that reads more like a heading",
                function.fingerprint.name
            ),
            evidence: vec![format!("doc comment line: {first_line}")],
        });
    }

    if is_tutorial_style_comment(doc_comment) {
        findings.push(Finding {
            rule_id: "comment_style_tutorial".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "function {} has a verbose tutorial-style doc comment",
                function.fingerprint.name
            ),
            evidence: vec![format!(
                "doc comment spans {} lines",
                doc_comment.lines().count()
            )],
        });
    }

    findings
}

fn weak_crypto_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let import_aliases = import_alias_lookup(&file.imports);
    let mut findings = Vec::new();

    for call in &function.calls {
        let Some(receiver) = &call.receiver else {
            continue;
        };
        let Some(import_path) = import_aliases.get(receiver) else {
            continue;
        };
        if !WEAK_CRYPTO_IMPORTS.contains(&import_path.as_str()) {
            continue;
        }

        findings.push(Finding {
            rule_id: "weak_crypto".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} uses weak cryptographic primitive {}",
                function.fingerprint.name, import_path
            ),
            evidence: vec![
                format!("import alias {receiver} resolves to {import_path}"),
                format!("weak crypto call: {receiver}.{}", call.name),
            ],
        });
    }

    findings
}

fn missing_context_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.has_context_parameter {
        return Vec::new();
    }

    let import_aliases = import_alias_lookup(&file.imports);

    function
        .calls
        .iter()
        .filter_map(|call| {
            let receiver = call.receiver.as_ref()?;
            let import_path = import_aliases.get(receiver)?;

            let is_context_aware_api = matches!(import_path.as_str(), "net/http")
                && HTTP_CONTEXTLESS_CALLS.contains(&call.name.as_str())
                || matches!(import_path.as_str(), "os/exec")
                    && EXEC_CONTEXTLESS_CALLS.contains(&call.name.as_str())
                || matches!(import_path.as_str(), "net")
                    && NET_CONTEXTLESS_CALLS.contains(&call.name.as_str());

            if !is_context_aware_api {
                return None;
            }

            Some(Finding {
                rule_id: "missing_context".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} performs context-aware work without accepting context.Context",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "context-free API call: {receiver}.{} from {import_path}",
                        call.name
                    ),
                    "function signature does not accept context.Context".to_string(),
                ],
            })
        })
        .collect()
}

fn sleep_polling_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    function
        .sleep_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "sleep_polling".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} uses time.Sleep inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "time.Sleep appears inside a loop, which often indicates polling".to_string(),
            ],
        })
        .collect()
}

fn string_concat_in_loop_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    function
        .string_concat_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "string_concat_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} concatenates strings inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "loop-local string concatenation can create repeated allocations".to_string(),
            ],
        })
        .collect()
}

fn goroutine_coordination_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.goroutine_launch_lines.is_empty() || has_obvious_coordination_signal(function) {
        return Vec::new();
    }

    function
        .goroutine_launch_lines
        .iter()
        .map(|line| Finding {
            rule_id: "goroutine_without_coordination".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} launches a goroutine without an obvious coordination signal",
                function.fingerprint.name
            ),
            evidence: vec![
                "raw go statement observed".to_string(),
                "no context.Context parameter or WaitGroup-like coordination call found"
                    .to_string(),
            ],
        })
        .collect()
}

fn has_obvious_coordination_signal(function: &ParsedFunction) -> bool {
    function.has_context_parameter
        || function.calls.iter().any(|call| {
            call.receiver.as_ref().is_some_and(|receiver| {
                COORDINATION_METHODS.contains(&call.name.as_str())
                    && matches!(
                        receiver.as_str(),
                        "wg" | "group" | "g" | "errGroup" | "errgroup"
                    )
            })
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
    let Some(current_package) = index.package_for_file(&file.path, package_name) else {
        return findings;
    };

    let import_aliases = import_alias_lookup(&file.imports);

    for call in &function.calls {
        match &call.receiver {
            Some(receiver) => {
                if let Some(import_path) = import_aliases.get(receiver) {
                    match index.resolve_import_path(import_path) {
                        ImportResolution::Resolved(target_package) => {
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
                                        format!(
                                            "matched local package {} in directory {}",
                                            target_package.package_name,
                                            target_package.directory_display()
                                        ),
                                        format!(
                                            "locally indexed package {} in {} does not expose {}",
                                            target_package.package_name,
                                            target_package.directory_display(),
                                            call.name
                                        ),
                                    ],
                                });
                            }
                        }
                        ImportResolution::Ambiguous(candidates) => {
                            let _candidate_count = candidates.len();
                            continue;
                        }
                        ImportResolution::Unresolved => continue,
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
                            "package {} in directory {} was indexed locally but {} was not found",
                            package_name,
                            current_package.directory_display(),
                            call.name
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
        "process",
        "handle",
        "execute",
        "convert",
        "validate",
        "transform",
        "parse",
        "format",
        "request",
        "response",
        "data",
        "input",
        "output",
        "task",
        "value",
    ]);
    generic_tokens
        .iter()
        .filter(|token| name.contains(*token))
        .count()
        >= 2
}

fn is_builtin(name: &str) -> bool {
    GO_BUILTINS.contains(&name)
}

fn looks_like_global_symbol(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}

fn identifier_token_count(name: &str) -> usize {
    let mut count = 0usize;
    let mut previous_was_separator = true;
    let mut previous_is_lower = false;

    for character in name.chars() {
        if character == '_' || character == '-' {
            previous_was_separator = true;
            previous_is_lower = false;
            continue;
        }

        if !character.is_ascii_alphanumeric() {
            previous_was_separator = true;
            previous_is_lower = false;
            continue;
        }

        if count == 0
            || previous_was_separator
            || character.is_ascii_uppercase() && previous_is_lower
        {
            count += 1;
        }

        previous_was_separator = false;
        previous_is_lower = character.is_ascii_lowercase();
    }

    count
}

fn is_title_case_comment(line: &str) -> bool {
    let words = line
        .split_whitespace()
        .map(|word| word.trim_matches(|character: char| !character.is_ascii_alphanumeric()))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();

    words.len() >= 3
        && !line.ends_with('.')
        && words.iter().all(|word| {
            word.chars().next().is_some_and(|character| {
                !character.is_ascii_alphabetic() || character.is_ascii_uppercase()
            })
        })
}

fn is_tutorial_style_comment(comment: &str) -> bool {
    let normalized = comment.to_ascii_lowercase();
    comment.lines().count() >= 2
        && (normalized.contains("this function")
            || normalized.contains("this method")
            || normalized.contains("by doing")
            || normalized.contains("because"))
}

#[cfg(test)]
mod tests {
    use super::{
        identifier_token_count, is_generic_name, is_title_case_comment, is_tutorial_style_comment,
        looks_like_global_symbol, normalize_name,
    };

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

    #[test]
    fn counts_identifier_tokens() {
        assert_eq!(identifier_token_count("processUserInputAndValidateIt"), 6);
        assert_eq!(identifier_token_count("process_user_input"), 3);
    }

    #[test]
    fn detects_title_case_comments() {
        assert!(is_title_case_comment("Run Processes Incoming Payloads"));
        assert!(!is_title_case_comment("Run processes incoming payloads."));
    }

    #[test]
    fn detects_tutorial_style_comments() {
        assert!(is_tutorial_style_comment(
            "Run Processes Incoming Payloads\nThis function does X by doing Y because Z"
        ));
        assert!(!is_tutorial_style_comment("Run validates invoices."));
    }
}
