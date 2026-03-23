use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::{ImportResolution, RepositoryIndex};
use crate::model::{Finding, Severity};

use super::common::{import_alias_lookup, is_builtin, is_global_sym};

pub(super) fn hallucination_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let package_name = match &file.package_name {
        Some(package_name) => package_name,
        None => return findings,
    };
    let Some(current_package) = index.package_for_file(file.language, &file.path, package_name) else {
        return findings;
    };

    let import_aliases = import_alias_lookup(&file.imports);

    for call in &function.calls {
        match &call.receiver {
            Some(receiver) => {
                if let Some(import_path) = import_aliases.get(receiver) {
                    match index.resolve_import_path(file.language, import_path) {
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
                if is_builtin(&call.name) || !is_global_sym(&call.name) {
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
