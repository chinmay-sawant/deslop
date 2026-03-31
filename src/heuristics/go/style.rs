use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::analysis::{ImportSpec, ParsedFile};
use crate::model::{Finding, Severity};

pub(crate) fn package_name_consistency(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut files_by_directory = BTreeMap::<PathBuf, Vec<&ParsedFile>>::new();

    for file in files {
        let Some(package_name) = file.package_name.as_deref() else {
            continue;
        };
        if package_name.is_empty() {
            continue;
        }

        let directory = file
            .path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        files_by_directory.entry(directory).or_default().push(*file);
    }

    let mut findings = Vec::new();

    for (directory, directory_files) in files_by_directory {
        let mut normalized_packages = BTreeMap::<String, Vec<(&str, &PathBuf)>>::new();
        let mut observed_packages = BTreeSet::new();

        for file in &directory_files {
            let Some(package_name) = file.package_name.as_deref() else {
                continue;
            };
            observed_packages.insert(package_name.to_string());
            normalized_packages
                .entry(normalize_package_name(package_name))
                .or_default()
                .push((package_name, &file.path));
        }

        if normalized_packages.len() <= 1 {
            continue;
        }

        let Some(anchor) = directory_files
            .iter()
            .min_by(|left, right| left.path.cmp(&right.path))
        else {
            continue;
        };

        let mut evidence = vec![format!("directory: {}", display_path(&directory))];
        evidence.push(format!(
            "observed packages: {}",
            observed_packages.into_iter().collect::<Vec<_>>().join(", ")
        ));
        evidence.extend(normalized_packages.into_iter().map(|(base_name, entries)| {
            let examples = entries
                .into_iter()
                .map(|(package_name, path)| format!("{} ({package_name})", display_path(path)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("normalized package {base_name}: {examples}")
        }));

        findings.push(Finding {
            rule_id: "inconsistent_package_name".to_string(),
            severity: Severity::Warning,
            path: anchor.path.clone(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: "directory mixes Go package names after normalizing _test suffixes"
                .to_string(),
            evidence,
        });
    }

    findings
}

pub(crate) fn import_grouping_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut imports_by_group = BTreeMap::<usize, Vec<&ImportSpec>>::new();

    for import in &file.imports {
        imports_by_group
            .entry(import.group_line)
            .or_default()
            .push(import);
    }

    let mut findings = Vec::new();

    for (group_line, mut group_imports) in imports_by_group {
        group_imports
            .sort_by(|left, right| left.line.cmp(&right.line).then(left.path.cmp(&right.path)));

        let mut first_third_party = None;
        let mut first_misgrouped_stdlib = None;

        for import in group_imports {
            if is_stdlib_import(import) {
                if first_third_party.is_some() {
                    first_misgrouped_stdlib = Some(import);
                    break;
                }
            } else if first_third_party.is_none() {
                first_third_party = Some(import);
            }
        }

        let (Some(third_party_import), Some(stdlib_import)) =
            (first_third_party, first_misgrouped_stdlib)
        else {
            continue;
        };

        findings.push(Finding {
            rule_id: "misgrouped_imports".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: stdlib_import.line,
            end_line: stdlib_import.line,
            message: "stdlib imports appear after third-party imports in one import block"
                .to_string(),
            evidence: vec![
                format!("import block starts at line {group_line}"),
                format!(
                    "third-party import {} appears before stdlib import {}",
                    third_party_import.path, stdlib_import.path
                ),
                "group stdlib imports before third-party imports".to_string(),
            ],
        });
    }

    findings
}

fn normalize_package_name(package_name: &str) -> String {
    package_name
        .strip_suffix("_test")
        .unwrap_or(package_name)
        .to_string()
}

fn is_stdlib_import(import: &ImportSpec) -> bool {
    !import.path.contains('.')
}

fn display_path(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        path.display().to_string()
    }
}
