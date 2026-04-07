use std::collections::BTreeSet;

use crate::analysis::{Language, ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::{Finding, Severity};

use super::import_resolution::{
    call_matches_import, visible_alias_lookup, wildcard_import_matches_item,
};
use super::is_rust_import;

pub(super) fn rust_call_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
    imports: &[crate::analysis::ImportSpec],
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

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

        let import_aliases = visible_alias_lookup(imports, call.line);

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

        let mut visited = BTreeSet::new();
        if import_aliases
            .values()
            .filter(|import_spec| import_spec.alias == "*")
            .any(|import_spec| {
                wildcard_import_matches_item(
                    index,
                    &file.path,
                    import_spec,
                    &call.name,
                    &mut visited,
                )
            })
        {
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

fn is_rust_prelude_function(name: &str) -> bool {
    matches!(name, "drop")
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
