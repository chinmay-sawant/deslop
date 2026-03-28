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

    fn parse_file(&self, path: &Path, source: &str) -> crate::Result<ParsedFile> {
        parser::parse_file(path, source).map_err(crate::Error::from)
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

        if !is_rust_path_receiver(receiver) {
            continue;
        }

        // If receiver is a local symbol, assume it's not a module/import call
        if current_package.has_symbol(receiver) {
            continue;
        }

        if import_aliases
            .get(receiver)
            .is_some_and(|import_spec| import_matches_item(index, &file.path, import_spec))
        {
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
        if is_rust_prelude_function(&call.name) {
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
    import_matches_item(index, file_path, import_spec)
}

fn import_matches_item(
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

fn is_rust_prelude_function(name: &str) -> bool {
    matches!(name, "drop")
}

fn is_rust_path_receiver(receiver: &str) -> bool {
    receiver.split("::").all(|segment| {
        !segment.is_empty()
            && segment
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
    })
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{alias_lookup, call_matches_import, evaluate_rust_findings, import_matches_item};
    use crate::analysis::rust::parser;
    use crate::index::build_repository_index;

    fn parse_file(path: &str, source: &str) -> crate::analysis::ParsedFile {
        let parsed = parser::parse_file(Path::new(path), source);
        assert!(parsed.is_ok(), "rust source should parse");
        match parsed {
            Ok(file) => file,
            Err(_) => unreachable!("asserted above"),
        }
    }

    #[test]
    fn grouped_imported_function_matches_current_module_index() {
        let current = parse_file(
            "/repo/src/analysis/go/mod.rs",
            r#"
use crate::heuristics::{evaluate_go_file, evaluate_go_repo};

fn run() {
    evaluate_go_file();
    evaluate_go_repo();
}
"#,
        );
        let heuristics = parse_file(
            "/repo/src/heuristics/mod.rs",
            r#"
pub fn evaluate_go_file() {}
pub fn evaluate_go_repo() {}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), heuristics]);
        let import_aliases = alias_lookup(&current.imports);
        let import_spec = import_aliases.get("evaluate_go_file");

        assert!(import_spec.is_some(), "grouped import should be indexed");
        let import_spec = match import_spec {
            Some(import_spec) => import_spec,
            None => unreachable!("asserted above"),
        };

        assert!(call_matches_import(&index, &current.path, import_spec));
        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| finding.rule_id == "hallucinated_import_call")
        );
    }

    #[test]
    fn self_group_imported_function_matches_current_module_index() {
        let current = parse_file(
            "/repo/src/analysis/go/parser/mod.rs",
            r#"
use self::general::{collect_calls, find_package_name};

fn run() {
    collect_calls();
    find_package_name();
}
"#,
        );
        let general = parse_file(
            "/repo/src/analysis/go/parser/general.rs",
            r#"
pub(super) fn collect_calls() {}
pub(super) fn find_package_name() {}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), general]);
        let import_aliases = alias_lookup(&current.imports);

        assert!(call_matches_import(
            &index,
            &current.path,
            match import_aliases.get("collect_calls") {
                Some(import_spec) => import_spec,
                None => unreachable!("collect_calls import should exist"),
            }
        ));
        assert!(call_matches_import(
            &index,
            &current.path,
            match import_aliases.get("find_package_name") {
                Some(import_spec) => import_spec,
                None => unreachable!("find_package_name import should exist"),
            }
        ));
        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| finding.rule_id == "hallucinated_import_call")
        );
    }

    #[test]
    fn imported_type_receiver_is_not_treated_as_missing_module() {
        let current = parse_file(
            "/repo/src/analysis/go/parser/mod.rs",
            r#"
use crate::analysis::Error;

fn run() {
    let _ = Error::parser_configuration();
}
"#,
        );
        let analysis = parse_file(
            "/repo/src/analysis/mod.rs",
            r#"
pub use error::Error;
mod error;
"#,
        );
        let error = parse_file(
            "/repo/src/analysis/error.rs",
            r#"
pub struct Error;

impl Error {
    pub fn parser_configuration() -> Self {
        Self
    }
}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), analysis, error]);
        let import_aliases = alias_lookup(&current.imports);
        let import_spec = import_aliases.get("Error");

        assert!(import_spec.is_some(), "type import should be indexed");
        let import_spec = match import_spec {
            Some(import_spec) => import_spec,
            None => unreachable!("asserted above"),
        };

        assert!(import_matches_item(&index, &current.path, import_spec));
        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| {
                    finding.rule_id == "hallucinated_import_call"
                        && finding.message.contains("Error::parser_configuration")
                })
        );
    }

    #[test]
    fn rust_prelude_drop_is_not_flagged_as_local_hallucination() {
        let current = parse_file(
            "/repo/src/lib.rs",
            r#"
pub fn release(value: String) {
    drop(value);
}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), std::slice::from_ref(&current));

        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| {
                    finding.rule_id == "hallucinated_local_call" && finding.message.contains("drop")
                })
        );
    }

    #[test]
    fn actual_go_module_imported_heuristic_call_is_resolved() {
        let current = parse_file("/repo/src/analysis/go/mod.rs", include_str!("../go/mod.rs"));
        let heuristics = parse_file(
            "/repo/src/heuristics/mod.rs",
            include_str!("../../heuristics/mod.rs"),
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), heuristics]);
        let import_aliases = alias_lookup(&current.imports);
        let import_spec = import_aliases.get("evaluate_go_file");

        assert!(
            import_spec.is_some(),
            "evaluate_go_file import should exist"
        );
        let import_spec = match import_spec {
            Some(import_spec) => import_spec,
            None => unreachable!("asserted above"),
        };

        assert!(call_matches_import(&index, &current.path, import_spec));
    }

    #[test]
    fn actual_go_parser_error_import_is_indexed_as_item() {
        let current = parse_file(
            "/repo/src/analysis/go/parser/mod.rs",
            include_str!("../go/parser/mod.rs"),
        );
        let analysis = parse_file("/repo/src/analysis/mod.rs", include_str!("../mod.rs"));
        let error = parse_file("/repo/src/analysis/error.rs", include_str!("../error.rs"));

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), analysis, error]);
        let import_aliases = alias_lookup(&current.imports);
        let import_spec = import_aliases.get("Error");

        assert!(import_spec.is_some(), "Error import should exist");
        let import_spec = match import_spec {
            Some(import_spec) => import_spec,
            None => unreachable!("asserted above"),
        };

        assert!(import_matches_item(&index, &current.path, import_spec));
    }

    #[test]
    fn expression_receivers_are_not_treated_as_import_paths() {
        let current = parse_file(
            "/repo/src/lib.rs",
            r#"
mod parser {
    pub fn parse_file() -> Result<(), ()> {
        Ok(())
    }
}

pub fn run() {
    let _ = parser::parse_file().map_err(|_| ());
}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), std::slice::from_ref(&current));

        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| {
                    finding.rule_id == "hallucinated_import_call"
                        && finding.message.contains("parser::parse_file")
                })
        );
    }
}
