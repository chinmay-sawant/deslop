mod parser;

use std::collections::BTreeMap;
use std::path::Path;

use crate::analysis::{ImportSpec, ParsedFunction};
use crate::analysis::{Language, LanguageBackend, ParsedFile};
use crate::heuristics::rust::{
    async_file_findings, async_function_findings, domain_findings, performance_file_findings,
    performance_function_findings, unsafe_soundness_findings,
};
use crate::index::{ImportResolution, PackageIndex, RepositoryIndex};
use crate::model::{Finding, Severity};
use crate::Result;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RustAnalyzer;

impl LanguageBackend for RustAnalyzer {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn supports_path(&self, path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()) == Some("rs")
    }

    fn parse_file(&self, path: &Path, source: &str) -> Result<ParsedFile> {
        parser::parse_file(path, source)
    }

    fn evaluate_file(&self, file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
        evaluate_rust_findings(file, index)
    }
}

fn evaluate_rust_findings(file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();
    let import_aliases = alias_lookup(&file.imports);

    findings.extend(domain_findings(file));
    findings.extend(performance_file_findings(file));
    findings.extend(async_file_findings(file));

    for function in &file.functions {
        findings.extend(non_test_macro_findings(
            file,
            function,
            "todo!",
            "todo_macro_leftover",
            "leaves todo! in non-test Rust code",
        ));
        findings.extend(non_test_macro_findings(
            file,
            function,
            "unimplemented!",
            "unimplemented_macro_leftover",
            "leaves unimplemented! in non-test Rust code",
        ));
        findings.extend(non_test_macro_findings(
            file,
            function,
            "dbg!",
            "dbg_macro_leftover",
            "leaves dbg! in non-test Rust code",
        ));
        findings.extend(non_test_macro_findings(
            file,
            function,
            "panic!",
            "panic_macro_leftover",
            "leaves panic! in non-test Rust code",
        ));
        findings.extend(non_test_macro_findings(
            file,
            function,
            "unreachable!",
            "unreachable_macro_leftover",
            "leaves unreachable! in non-test Rust code",
        ));
        findings.extend(non_test_call_findings(
            file,
            function,
            "unwrap",
            "unwrap_in_non_test_code",
            "calls unwrap() in non-test Rust code",
        ));
        findings.extend(non_test_call_findings(
            file,
            function,
            "expect",
            "expect_in_non_test_code",
            "calls expect() in non-test Rust code",
        ));
        findings.extend(unsafe_findings(file, function));
        findings.extend(unsafe_soundness_findings(file, function));
        findings.extend(doc_marker_findings(file, function));
        findings.extend(performance_function_findings(file, function));
        findings.extend(async_function_findings(file, function));
        let Some(package_name) = &file.package_name else {
            continue;
        };
        let Some(current_package) =
            index.package_for_file(Language::Rust, &file.path, package_name)
        else {
            continue;
        };

        findings.extend(rust_import_findings(
            file,
            function,
            index,
            &import_aliases,
            current_package,
        ));
        findings.extend(rust_call_findings(file, function, index, &import_aliases));
    }

    findings
}

fn non_test_macro_findings(
    file: &ParsedFile,
    function: &crate::analysis::ParsedFunction,
    macro_name: &str,
    rule_id: &str,
    message_suffix: &str,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter(|call| call.name == macro_name)
        .map(|call| Finding {
            rule_id: rule_id.to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!("function {} {message_suffix}", function.fingerprint.name),
            evidence: vec![format!("macro invocation: {macro_name}")],
        })
        .collect()
}

fn non_test_call_findings(
    file: &ParsedFile,
    function: &crate::analysis::ParsedFunction,
    call_name: &str,
    rule_id: &str,
    message_suffix: &str,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter(|call| call.name == call_name)
        .map(|call| Finding {
            rule_id: rule_id.to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!("function {} {message_suffix}", function.fingerprint.name),
            evidence: vec![match &call.receiver {
                Some(receiver) => format!("method call: {receiver}.{call_name}()"),
                None => format!("call: {call_name}()"),
            }],
        })
        .collect()
}

fn unsafe_findings(file: &ParsedFile, function: &crate::analysis::ParsedFunction) -> Vec<Finding> {
    function
        .unsafe_lines
        .iter()
        .filter(|unsafe_line| !has_safety_comment(**unsafe_line, &function.safety_comment_lines))
        .map(|unsafe_line| Finding {
            rule_id: "unsafe_without_safety_comment".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *unsafe_line,
            end_line: *unsafe_line,
            message: format!(
                "function {} uses unsafe without a nearby SAFETY comment",
                function.fingerprint.name
            ),
            evidence: vec![format!("unsafe usage line: {unsafe_line}")],
        })
        .collect()
}

fn has_safety_comment(unsafe_line: usize, safety_comment_lines: &[usize]) -> bool {
    let min_line = unsafe_line.saturating_sub(2);
    safety_comment_lines
        .iter()
        .any(|comment_line| *comment_line >= min_line && *comment_line <= unsafe_line)
}

fn doc_marker_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let Some(doc_comment) = &function.doc_comment else {
        return Vec::new();
    };
    let normalized = doc_comment.to_ascii_uppercase();
    let mut findings = Vec::new();

    if normalized.contains("TODO") {
        findings.push(Finding {
            rule_id: "todo_doc_comment_leftover".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "function {} has a TODO marker in its Rust doc comment",
                function.fingerprint.name
            ),
            evidence: vec![first_doc_comment_line(doc_comment)],
        });
    }

    if normalized.contains("FIXME") {
        findings.push(Finding {
            rule_id: "fixme_doc_comment_leftover".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "function {} has a FIXME marker in its Rust doc comment",
                function.fingerprint.name
            ),
            evidence: vec![first_doc_comment_line(doc_comment)],
        });
    }

    if normalized.contains("HACK") {
        findings.push(Finding {
            rule_id: "hack_doc_comment_leftover".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "function {} has a HACK marker in its Rust doc comment",
                function.fingerprint.name
            ),
            evidence: vec![first_doc_comment_line(doc_comment)],
        });
    }

    findings
}

fn first_doc_comment_line(doc_comment: &str) -> String {
    let line = doc_comment
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(doc_comment.trim());

    format!("doc comment line: {line}")
}

fn rust_import_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
    import_aliases: &BTreeMap<String, ImportSpec>,
    current_package: &PackageIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for call in &function.calls {
        let Some(receiver) = &call.receiver else {
            continue;
        };

        // If receiver is a local symbol, assume it's not a module/import call
        if current_package.has_symbol(receiver) {
            continue;
        }

        let Some(import_path) = rust_mod_for_receiver(receiver, import_aliases) else {
            continue;
        };

        match index.resolve_rust_import(&file.path, &import_path) {
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
                            "call to {receiver}::{} has no matching symbol in locally indexed Rust module {import_path}",
                            call.name
                        ),
                        evidence: vec![
                            format!("import receiver {receiver} resolves to local module path {import_path}"),
                            format!(
                                "matched local Rust module {} in directory {}",
                                target_package.package_name,
                                target_package.directory_display()
                            ),
                            format!(
                                "locally indexed Rust module {} in {} does not expose {}",
                                target_package.package_name,
                                target_package.directory_display(),
                                call.name
                            ),
                        ],
                    });
                }
            }
            ImportResolution::Ambiguous(_) => continue,
            ImportResolution::Unresolved => {
                findings.push(Finding {
                    rule_id: "hallucinated_import_call".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: call.line,
                    end_line: call.line,
                    message: format!(
                        "call to {receiver}::{} resolves to local Rust module path {import_path}, but no matching module was indexed",
                        call.name
                    ),
                    evidence: vec![format!(
                        "import receiver {receiver} resolves to local module path {import_path}"
                    )],
                });
            }
        }
    }

    findings
}

fn rust_call_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
    import_aliases: &BTreeMap<String, ImportSpec>,
) -> Vec<Finding> {
    let Some(package_name) = &file.package_name else {
        return Vec::new();
    };
    let Some(current_package) = index.package_for_file(Language::Rust, &file.path, package_name)
    else {
        return Vec::new();
    };

    let mut findings = Vec::new();

    for call in &function.calls {
        if call.receiver.is_some() || call.name.ends_with('!') {
            continue;
        }
        if function
            .local_binding_names
            .iter()
            .any(|name| name == &call.name)
        {
            continue;
        }

        if current_package.has_function(&call.name) || current_package.has_symbol(&call.name) {
            continue;
        }

        if let Some(import_spec) = import_aliases.get(&call.name) {
            if !is_rust_import(import_spec.path.as_str()) {
                continue;
            }

            if call_matches_import(index, &file.path, import_spec) {
                continue;
            }

            findings.push(Finding {
                rule_id: "hallucinated_import_call".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "direct call to {} resolves to local Rust import path {}, but no matching callable symbol was indexed",
                    call.name, import_spec.path
                ),
                evidence: vec![format!(
                    "direct call name {} matches local import alias {}",
                    call.name, call.name
                )],
            });
            continue;
        }

        if is_rust_local_sym(&call.name) {
            findings.push(Finding {
                rule_id: "hallucinated_local_call".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "direct call to {} has no matching symbol in locally indexed Rust module {}",
                    call.name, package_name
                ),
                evidence: vec![format!(
                    "Rust module {} in directory {} was indexed locally but {} was not found",
                    package_name,
                    current_package.directory_display(),
                    call.name
                )],
            });
        }
    }

    findings
}

fn call_matches_import(
    index: &RepositoryIndex,
    file_path: &Path,
    import_spec: &ImportSpec,
) -> bool {
    let Some(module_path) = import_spec.namespace_path.as_deref() else {
        return false;
    };
    let Some(item_name) = import_spec.imported_name.as_deref() else {
        return false;
    };
    if item_name == "*" {
        return false;
    }

    match index.resolve_rust_import(file_path, module_path) {
        ImportResolution::Resolved(module_package) => {
            module_package.has_function(item_name) || module_package.has_symbol(item_name)
        }
        ImportResolution::Ambiguous(_) => true,
        ImportResolution::Unresolved => false,
    }
}

fn is_rust_local_sym(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };

    (first.is_ascii_lowercase() || first == '_')
        && characters.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
}

fn rust_mod_for_receiver(
    receiver: &str,
    import_aliases: &BTreeMap<String, ImportSpec>,
) -> Option<String> {
    let (alias, import_spec) = import_aliases
        .iter()
        .filter(|(alias, import_spec)| {
            alias.as_str() != "*"
                && is_rust_import(import_spec.path.as_str())
                && (receiver == alias.as_str()
                    || receiver
                        .strip_prefix(alias.as_str())
                        .is_some_and(|suffix| suffix.starts_with("::")))
        })
        .max_by_key(|(alias, _)| alias.len())?;

    let suffix = receiver
        .strip_prefix(alias.as_str())
        .and_then(|value| value.strip_prefix("::"));

    Some(match suffix {
        Some(suffix) if !suffix.is_empty() => format!("{}::{suffix}", import_spec.path),
        _ => import_spec.path.clone(),
    })
}

fn is_rust_import(import_path: &str) -> bool {
    import_path.starts_with("crate::")
        || import_path.starts_with("self::")
        || import_path.starts_with("super::")
}

fn alias_lookup(imports: &[ImportSpec]) -> BTreeMap<String, ImportSpec> {
    imports
        .iter()
        .map(|import| (import.alias.clone(), import.clone()))
        .collect()
}
