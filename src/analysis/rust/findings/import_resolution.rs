use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::analysis::{ImportSpec, ParsedFile, ParsedFunction};
use crate::index::{ImportResolution, PackageIndex, RepositoryIndex, RustModuleFileResolution};
use crate::model::{Finding, Severity};

use super::is_rust_import;

pub(super) fn rust_import_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
    imports: &[ImportSpec],
    current_package: &PackageIndex,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let mut findings = Vec::new();

    for call in &function.calls {
        let Some(receiver) = &call.receiver else {
            continue;
        };

        if !is_rust_path_receiver(receiver) {
            continue;
        }

        if current_package.has_symbol(receiver) {
            continue;
        }

        let import_aliases = visible_alias_lookup(imports, call.line);

        if import_aliases
            .get(receiver)
            .is_some_and(|import_spec| import_matches_item(index, &file.path, import_spec))
        {
            continue;
        }

        let Some(import_path) = rust_mod_for_receiver(receiver, &import_aliases) else {
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
                            format!(
                                "import receiver {receiver} resolves to local module path {import_path}"
                            ),
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

pub(crate) fn call_matches_import(
    index: &RepositoryIndex,
    file_path: &Path,
    import_spec: &ImportSpec,
) -> bool {
    import_matches_item(index, file_path, import_spec)
}

pub(crate) fn import_matches_item(
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

    match index.resolve_rust_module_file(file_path, module_path) {
        RustModuleFileResolution::Resolved(module_file) => {
            let mut visited = BTreeSet::new();
            module_namespace_matches_item(index, &module_file, item_name, &mut visited)
                || import_matches_local_module(index, file_path, module_path, item_name)
        }
        RustModuleFileResolution::Ambiguous(_) => true,
        RustModuleFileResolution::Unresolved => {
            match index.resolve_rust_import(file_path, module_path) {
                ImportResolution::Resolved(module_package) => {
                    module_package.has_function(item_name)
                        || module_package.has_symbol(item_name)
                        || import_matches_local_module(index, file_path, module_path, item_name)
                }
                ImportResolution::Ambiguous(_) => true,
                ImportResolution::Unresolved => {
                    import_matches_local_module(index, file_path, module_path, item_name)
                }
            }
        }
    }
}

pub(crate) fn alias_lookup(imports: &[ImportSpec]) -> BTreeMap<String, ImportSpec> {
    imports
        .iter()
        .map(|import| (import.alias.clone(), import.clone()))
        .collect()
}

pub(super) fn visible_alias_lookup(
    imports: &[ImportSpec],
    max_line: usize,
) -> BTreeMap<String, ImportSpec> {
    let visible_imports = imports
        .iter()
        .filter(|import| import.line <= max_line)
        .cloned()
        .collect::<Vec<_>>();

    alias_lookup(&visible_imports)
}

pub(super) fn wildcard_import_matches_item(
    index: &RepositoryIndex,
    file_path: &Path,
    import_spec: &ImportSpec,
    item_name: &str,
    visited: &mut BTreeSet<(String, String)>,
) -> bool {
    let Some(module_path) = import_spec
        .namespace_path
        .as_deref()
        .or_else(|| import_spec.path.strip_suffix("::*"))
        .or_else(|| {
            (import_spec.imported_name.as_deref() == Some("*") && import_spec.path != "*")
                .then_some(import_spec.path.as_str())
        })
    else {
        return false;
    };

    match index.resolve_rust_module_file(file_path, module_path) {
        RustModuleFileResolution::Resolved(module_file) => {
            module_namespace_matches_item(index, &module_file, item_name, visited)
        }
        RustModuleFileResolution::Ambiguous(_) => true,
        RustModuleFileResolution::Unresolved => {
            match index.resolve_rust_import(file_path, module_path) {
                ImportResolution::Resolved(module_package) => {
                    module_package.has_function(item_name) || module_package.has_symbol(item_name)
                }
                ImportResolution::Ambiguous(_) => true,
                ImportResolution::Unresolved => false,
            }
        }
    }
}

fn module_namespace_matches_item(
    index: &RepositoryIndex,
    module_file: &Path,
    item_name: &str,
    visited: &mut BTreeSet<(String, String)>,
) -> bool {
    let visit_key = (
        module_file.to_string_lossy().into_owned(),
        item_name.to_string(),
    );
    if !visited.insert(visit_key) {
        return false;
    }

    if index
        .package_for_rust_file(module_file)
        .is_some_and(|package| package.has_function(item_name) || package.has_symbol(item_name))
    {
        return true;
    }

    for include_file in index.rust_include_neighbors_for_file(module_file) {
        if index
            .package_for_rust_file(include_file)
            .is_some_and(|package| package.has_function(item_name) || package.has_symbol(item_name))
        {
            return true;
        }
    }

    let imports = index.rust_imports_for_file(module_file);
    let import_aliases = alias_lookup(imports);
    if let Some(import_spec) = import_aliases.get(item_name)
        && explicit_import_matches_item(index, module_file, import_spec, visited)
    {
        return true;
    }

    for import_spec in imports {
        if import_spec.alias != "*" || !is_rust_import(import_spec.path.as_str()) {
            continue;
        }
        if wildcard_import_matches_item(index, module_file, import_spec, item_name, visited) {
            return true;
        }
    }

    false
}

fn explicit_import_matches_item(
    index: &RepositoryIndex,
    module_file: &Path,
    import_spec: &ImportSpec,
    visited: &mut BTreeSet<(String, String)>,
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

    match index.resolve_rust_module_file(module_file, module_path) {
        RustModuleFileResolution::Resolved(target_file) => {
            module_namespace_matches_item(index, &target_file, item_name, visited)
        }
        RustModuleFileResolution::Ambiguous(_) => true,
        RustModuleFileResolution::Unresolved => {
            match index.resolve_rust_import(module_file, module_path) {
                ImportResolution::Resolved(module_package) => {
                    module_package.has_function(item_name) || module_package.has_symbol(item_name)
                }
                ImportResolution::Ambiguous(_) => true,
                ImportResolution::Unresolved => false,
            }
        }
    }
}

fn import_matches_local_module(
    index: &RepositoryIndex,
    file_path: &Path,
    module_path: &str,
    item_name: &str,
) -> bool {
    if !module_path.starts_with("super") {
        return false;
    }

    let Some(parent) = file_path.parent() else {
        return false;
    };

    let sibling_file = parent.join(format!("{item_name}.rs"));
    if index.package_for_rust_file(&sibling_file).is_some() {
        return true;
    }

    let sibling_mod = parent.join(item_name).join("mod.rs");
    index.package_for_rust_file(&sibling_mod).is_some()
}

fn is_rust_path_receiver(receiver: &str) -> bool {
    receiver.split("::").all(|segment| {
        !segment.is_empty()
            && segment
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
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
