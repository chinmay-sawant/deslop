use std::path::{Path, PathBuf};

use toml::Value;

use crate::analysis::{Language, ParsedFile, ParsedFunction};
use crate::index::{ImportResolution, RepositoryIndex};
use crate::io::{DEFAULT_MAX_BYTES, read_to_string_limited};
use crate::model::{Finding, Severity};

pub(super) fn public_api_any_contract_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !looks_like_public_python_api(function) {
        return Vec::new();
    }

    let signature = function.signature_text.replace('\n', " ");
    if !signature_mentions_any(signature.as_str()) {
        return Vec::new();
    }

    let contract_shape = if signature.contains("-> Any") || signature.contains("-> typing.Any") {
        "return_type"
    } else {
        "parameter_or_container"
    };

    vec![Finding {
        rule_id: "python_public_api_any_contract".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.start_line,
        message: format!(
            "public API {} uses Any in its type contract",
            function.fingerprint.name
        ),
        evidence: vec![
            format!("contract_shape={contract_shape}"),
            format!("signature={signature}"),
        ],
    }]
}

pub(super) fn pyproject_repo_findings(
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let Some(pyproject_path) = locate_pyproject(index.root()) else {
        return Vec::new();
    };
    let Ok(source) = read_to_string_limited(&pyproject_path, DEFAULT_MAX_BYTES) else {
        return Vec::new();
    };
    let Ok(parsed) = source.parse::<Value>() else {
        return Vec::new();
    };

    let mut findings = Vec::new();
    findings.extend(pyproject_requires_python_findings(&pyproject_path, &parsed));
    findings.extend(pyproject_script_findings(&pyproject_path, &parsed, files, index));
    findings.extend(cross_package_internal_import_findings(files, index));
    findings
}

fn pyproject_requires_python_findings(pyproject_path: &Path, parsed: &Value) -> Vec<Finding> {
    let has_project_table = parsed.get("project").and_then(Value::as_table).is_some();
    let poetry_table = parsed
        .get("tool")
        .and_then(Value::as_table)
        .and_then(|tool| tool.get("poetry"))
        .and_then(Value::as_table);
    let has_poetry = poetry_table.is_some();
    if !has_project_table && !has_poetry {
        return Vec::new();
    }

    let pep621_requires_python = parsed
        .get("project")
        .and_then(Value::as_table)
        .and_then(|project| project.get("requires-python"))
        .and_then(Value::as_str);
    let poetry_python = poetry_table
        .and_then(|poetry| poetry.get("dependencies"))
        .and_then(Value::as_table)
        .and_then(|dependencies| dependencies.get("python"))
        .and_then(|python| {
            python
                .as_str()
                .map(str::to_string)
                .or_else(|| python.get("version").and_then(Value::as_str).map(str::to_string))
        });

    if pep621_requires_python.is_some() || poetry_python.is_some() {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "pyproject_missing_requires_python".to_string(),
        severity: Severity::Info,
        path: pyproject_path.to_path_buf(),
        function_name: None,
        start_line: 1,
        end_line: 1,
        message: "pyproject.toml does not declare a Python runtime requirement".to_string(),
        evidence: vec![
            "project metadata was found but no requires-python / poetry python version was declared"
                .to_string(),
        ],
    }]
}

fn pyproject_script_findings(
    pyproject_path: &Path,
    parsed: &Value,
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (script_name, target) in declared_script_targets(parsed) {
        let Some((module_path, callable_name)) = target.split_once(':') else {
            continue;
        };
        let callable_name = callable_name
            .split('.')
            .next()
            .unwrap_or(callable_name)
            .trim();
        let module_path = module_path.trim();
        let local_module = is_likely_local_module(module_path, files, index);

        match index.resolve_import_path(Language::Python, module_path) {
            ImportResolution::Resolved(package) => {
                if package.has_function(callable_name) || package.has_symbol(callable_name) {
                    continue;
                }

                findings.push(Finding {
                    rule_id: "pyproject_script_entrypoint_unresolved".to_string(),
                    severity: Severity::Warning,
                    path: pyproject_path.to_path_buf(),
                    function_name: None,
                    start_line: 1,
                    end_line: 1,
                    message: format!(
                        "pyproject script {script_name} points to {target}, but the callable was not found"
                    ),
                    evidence: vec![
                        format!("resolved local module path={module_path}"),
                        format!("expected callable={callable_name}"),
                        format!(
                            "matched package={} in {}",
                            package.package_name,
                            package.directory_display()
                        ),
                    ],
                });
            }
            ImportResolution::Unresolved if local_module => {
                findings.push(Finding {
                    rule_id: "pyproject_script_entrypoint_unresolved".to_string(),
                    severity: Severity::Warning,
                    path: pyproject_path.to_path_buf(),
                    function_name: None,
                    start_line: 1,
                    end_line: 1,
                    message: format!(
                        "pyproject script {script_name} points to {target}, but the module was not indexed"
                    ),
                    evidence: vec![format!("expected local module path={module_path}")],
                });
            }
            ImportResolution::Ambiguous(_) | ImportResolution::Unresolved => {}
        }
    }

    findings
}

fn cross_package_internal_import_findings(
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for file in files {
        let Some(current_root) = file_root_package(file.path.as_path(), index.root()) else {
            continue;
        };

        for import in &file.imports {
            if !looks_internal_module(import.path.as_str()) {
                continue;
            }

            let import_root = import.path.split('.').next().unwrap_or_default();
            if import_root == current_root {
                continue;
            }

            if !matches!(
                index.resolve_import_path(Language::Python, import.path.as_str()),
                ImportResolution::Resolved(_)
            ) {
                continue;
            }

            findings.push(Finding {
                rule_id: "cross_package_internal_import".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: import.line,
                end_line: import.line,
                message: format!(
                    "module imports local internal package path {} across a package boundary",
                    import.path
                ),
                evidence: vec![
                    format!("current_root_package={current_root}"),
                    format!("import_root_package={import_root}"),
                ],
            });
        }
    }

    findings
}

fn locate_pyproject(root: &Path) -> Option<PathBuf> {
    let candidate = root.join("pyproject.toml");
    candidate.exists().then_some(candidate)
}

fn declared_script_targets(parsed: &Value) -> Vec<(String, String)> {
    let mut targets = Vec::new();

    if let Some(project_scripts) = parsed
        .get("project")
        .and_then(Value::as_table)
        .and_then(|project| project.get("scripts"))
        .and_then(Value::as_table)
    {
        for (name, value) in project_scripts {
            if let Some(target) = value.as_str() {
                targets.push((name.clone(), target.to_string()));
            }
        }
    }

    if let Some(poetry_scripts) = parsed
        .get("tool")
        .and_then(Value::as_table)
        .and_then(|tool| tool.get("poetry"))
        .and_then(Value::as_table)
        .and_then(|poetry| poetry.get("scripts"))
        .and_then(Value::as_table)
    {
        for (name, value) in poetry_scripts {
            if let Some(target) = value.as_str() {
                targets.push((name.clone(), target.to_string()));
            }
        }
    }

    targets
}

fn looks_like_public_python_api(function: &ParsedFunction) -> bool {
    let name = function.fingerprint.name.as_str();
    !name.starts_with('_')
        && !name.starts_with("__")
        && !function.fingerprint.kind.contains("lambda")
        && function.signature_text.contains("def ")
}

fn signature_mentions_any(signature: &str) -> bool {
    signature.contains(": Any")
        || signature.contains(": typing.Any")
        || signature.contains("-> Any")
        || signature.contains("-> typing.Any")
        || signature.contains("[Any]")
        || signature.contains(", Any]")
        || signature.contains(" Any |")
        || signature.contains("| Any")
}

fn is_likely_local_module(
    module_path: &str,
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> bool {
    let first_segment = module_path.split('.').next().unwrap_or_default();
    if first_segment.is_empty() {
        return false;
    }

    files.iter().any(|file| {
        file_root_package(file.path.as_path(), index.root())
            .is_some_and(|root_package| root_package == first_segment)
    })
}

fn file_root_package<'a>(path: &'a Path, root: &Path) -> Option<&'a str> {
    let relative = path.strip_prefix(root).ok()?;
    let components = relative
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()?;
    match components.as_slice() {
        ["src", first, ..] => Some(*first),
        [first, ..] => Some(*first),
        _ => None,
    }
}

fn looks_internal_module(path: &str) -> bool {
    path.contains(".internal")
        || path.contains(".private")
        || path.contains("._")
        || path.contains(".impl")
}
