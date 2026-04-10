use std::path::{Path, PathBuf};

use toml::Value;

use crate::analysis::{Language, ParsedFile, ParsedFunction};
use crate::index::{ImportResolution, RepositoryIndex};
use crate::io::{DEFAULT_MAX_BYTES, read_to_string_limited};
use crate::model::{Finding, Severity};

use super::{is_to_dict_wrapper, should_skip_python_wide_contract};

pub(crate) const BINDING_LOCATION: &str = file!();

pub(super) fn public_api_any_contract_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function
        || !looks_like_public_python_api(function)
        || is_to_dict_wrapper(function)
        || should_skip_python_wide_contract(file, function)
    {
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
    let mut findings = project_agnostic_packaging_repo_findings(files, index);
    let Some(pyproject_path) = locate_pyproject(index.root()) else {
        return findings;
    };
    let Ok(source) = read_to_string_limited(&pyproject_path, DEFAULT_MAX_BYTES) else {
        return findings;
    };
    let Ok(parsed) = source.parse::<Value>() else {
        return findings;
    };
    findings.extend(pyproject_requires_python_findings(&pyproject_path, &parsed));
    findings.extend(pyproject_script_findings(
        &pyproject_path,
        &parsed,
        files,
        index,
    ));
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
            python.as_str().map(str::to_string).or_else(|| {
                python
                    .get("version")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
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

fn project_agnostic_packaging_repo_findings(
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(heavy_optional_dependency_imported_by_package_root_findings(
        files,
    ));
    findings.extend(cli_only_dependency_imported_by_library_entry_module_findings(files));
    findings.extend(package_init_metadata_lookup_findings(files));
    findings.extend(environment_or_config_read_during_import_findings(files));
    findings.extend(circular_import_hidden_by_function_local_import_findings(
        files,
    ));
    findings.extend(plugin_discovery_scans_filesystem_each_invocation_findings(
        files,
    ));
    findings.extend(package_exports_same_symbol_name_findings(files));
    findings
        .extend(runtime_data_file_assumption_in_implicit_namespace_package_findings(files, index));
    findings.extend(test_helpers_shipped_inside_production_package_path_findings(files));
    findings.extend(public_api_surface_defined_only_by_import_side_effects_findings(files));
    findings.extend(package_root_reexports_large_dependency_tree_by_default_findings(files));
    findings.extend(
        monolithic_common_package_becomes_transitive_dependency_for_most_modules_findings(files),
    );
    findings
}

fn packaging_repo_finding(
    file: &ParsedFile,
    rule_id: &str,
    line: usize,
    severity: Severity,
    message: String,
    evidence: String,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: None,
        start_line: line,
        end_line: line,
        message,
        evidence: vec![evidence],
    }
}

fn heavy_optional_dependency_imported_by_package_root_findings(
    files: &[&ParsedFile],
) -> Vec<Finding> {
    const HEAVY: &[&str] = &[
        "pandas",
        "numpy",
        "torch",
        "tensorflow",
        "boto3",
        "sklearn",
        "matplotlib",
        "cv2",
    ];

    files
        .iter()
        .filter(|file| file.path.file_name().and_then(|name| name.to_str()) == Some("__init__.py"))
        .filter_map(|file| {
            let imported = file
                .imports
                .iter()
                .find(|import| HEAVY.iter().any(|heavy| import.path.starts_with(heavy)))?;
            Some(packaging_repo_finding(
                file,
                "heavy_optional_dependency_imported_by_package_root",
                imported.line,
                Severity::Info,
                format!(
                    "package root imports heavy optional dependency {} by default",
                    imported.path
                ),
                format!("import={}", imported.path),
            ))
        })
        .collect()
}

fn cli_only_dependency_imported_by_library_entry_module_findings(
    files: &[&ParsedFile],
) -> Vec<Finding> {
    files
        .iter()
        .filter(|file| {
            let path = file.path.to_string_lossy().to_ascii_lowercase();
            !path.contains("/cli") && !path.contains("command")
        })
        .filter_map(|file| {
            let imported = file.imports.iter().find(|import| {
                ["argparse", "click", "typer", "rich.console"]
                    .iter()
                    .any(|name| import.path.starts_with(name))
            })?;
            Some(packaging_repo_finding(
                file,
                "cli_only_dependency_imported_by_library_entry_module",
                imported.line,
                Severity::Info,
                format!(
                    "library-style module imports CLI-only dependency {}",
                    imported.path
                ),
                format!("import={}", imported.path),
            ))
        })
        .collect()
}

fn package_init_metadata_lookup_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for file in files
        .iter()
        .filter(|file| file.path.file_name().and_then(|name| name.to_str()) == Some("__init__.py"))
    {
        if let Some(call) = file.module_scope_calls.iter().find(|call| {
            call.text.contains("importlib.metadata.version(")
                || call.text.contains("pkg_resources.get_distribution(")
        }) {
            findings.push(packaging_repo_finding(
                file,
                "package_init_performs_metadata_version_lookup_on_import",
                call.line,
                Severity::Info,
                "package __init__ performs runtime metadata version lookup during import"
                    .to_string(),
                format!("module_scope_call={}", call.text.trim()),
            ));
            continue;
        }

        if let Some(binding) = file.top_level_bindings.iter().find(|binding| {
            binding.value_text.contains("version(")
                || binding.value_text.contains("get_distribution(")
        }) {
            findings.push(packaging_repo_finding(
                file,
                "package_init_performs_metadata_version_lookup_on_import",
                binding.line,
                Severity::Info,
                "package __init__ performs runtime metadata version lookup during import"
                    .to_string(),
                format!("binding={}", binding.name),
            ));
        }
    }
    findings
}

fn environment_or_config_read_during_import_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for file in files {
        if let Some(call) = file.module_scope_calls.iter().find(|call| {
            call.text.contains("os.getenv(")
                || call.text.contains("os.environ[")
                || call.text.contains("config.get(")
                || call.text.contains("load_dotenv(")
        }) {
            findings.push(packaging_repo_finding(
                file,
                "environment_or_config_read_during_package_import",
                call.line,
                Severity::Warning,
                "module reads environment or config while being imported".to_string(),
                format!("module_scope_call={}", call.text.trim()),
            ));
        }
    }
    findings
}

fn circular_import_hidden_by_function_local_import_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for file in files {
        for function in &file.functions {
            if function.body_text.contains("import ")
                && (function.body_text.contains("for ") || function.fingerprint.call_count >= 4)
            {
                findings.push(Finding {
                    rule_id: "circular_import_hidden_by_function_local_import_on_hot_path"
                        .to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: function.fingerprint.start_line,
                    end_line: function.fingerprint.start_line,
                    message: format!(
                        "function {} hides an import inside the body on a live call path",
                        function.fingerprint.name
                    ),
                    evidence: vec!["pattern=function_local_import".to_string()],
                });
                break;
            }
        }
    }
    findings
}

fn plugin_discovery_scans_filesystem_each_invocation_findings(
    files: &[&ParsedFile],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for file in files {
        for function in &file.functions {
            let lower = function.body_text.to_ascii_lowercase();
            if (lower.contains("pkgutil.iter_modules(")
                || lower.contains("os.listdir(")
                || lower.contains("glob(")
                || lower.contains("entry_points("))
                && (lower.contains("plugin") || lower.contains("extension"))
            {
                findings.push(Finding {
                    rule_id: "plugin_discovery_scans_filesystem_each_invocation".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: function.fingerprint.start_line,
                    end_line: function.fingerprint.start_line,
                    message: format!(
                        "function {} rescans plugins or extensions on each invocation",
                        function.fingerprint.name
                    ),
                    evidence: vec!["pattern=runtime_plugin_discovery".to_string()],
                });
                break;
            }
        }
    }
    findings
}

fn package_exports_same_symbol_name_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for file in files {
        if file.path.file_name().and_then(|name| name.to_str()) != Some("__init__.py") {
            continue;
        }
        let mut seen = std::collections::BTreeSet::new();
        for import in &file.imports {
            let exported = import
                .imported_name
                .as_deref()
                .unwrap_or(import.alias.as_str());
            if !seen.insert(exported.to_string()) {
                findings.push(packaging_repo_finding(
                    file,
                    "package_exports_same_symbol_name_from_multiple_submodules_with_different_meanings",
                    import.line,
                    Severity::Info,
                    format!("package re-exports the symbol name {} from multiple places", exported),
                    format!("export={exported}"),
                ));
                break;
            }
        }
    }
    findings
}

fn runtime_data_file_assumption_in_implicit_namespace_package_findings(
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let root = index.root();
    for file in files {
        let path = file.path.to_string_lossy().to_string();
        if !file
            .module_scope_calls
            .iter()
            .any(|call| call.text.contains("read_text(") || call.text.contains("read_bytes("))
        {
            continue;
        }
        let Some(parent) = file.path.parent() else {
            continue;
        };
        if parent.join("__init__.py").exists() {
            continue;
        }
        if file.path.strip_prefix(root).is_err() {
            continue;
        }
        findings.push(packaging_repo_finding(
            file,
            "runtime_data_file_assumption_in_implicit_namespace_package",
            1,
            Severity::Info,
            "module reads package-adjacent data files from a directory without __init__.py"
                .to_string(),
            format!("path={path}"),
        ));
    }
    findings
}

fn test_helpers_shipped_inside_production_package_path_findings(
    files: &[&ParsedFile],
) -> Vec<Finding> {
    files
        .iter()
        .filter_map(|file| {
            let path = file.path.to_string_lossy().to_ascii_lowercase();
            if path.contains("/tests/") {
                return None;
            }
            if !(path.contains("fixture")
                || path.contains("fake")
                || path.contains("mock")
                || path.contains("factory"))
            {
                return None;
            }
            Some(packaging_repo_finding(
                file,
                "test_helpers_shipped_inside_production_package_path",
                1,
                Severity::Info,
                "test helper naming appears inside a production package path".to_string(),
                format!("path={path}"),
            ))
        })
        .collect()
}

fn public_api_surface_defined_only_by_import_side_effects_findings(
    files: &[&ParsedFile],
) -> Vec<Finding> {
    files
        .iter()
        .filter(|file| file.path.file_name().and_then(|name| name.to_str()) == Some("__init__.py"))
        .filter_map(|file| {
            if file.imports.len() < 4
                || file
                    .top_level_bindings
                    .iter()
                    .any(|binding| binding.name == "__all__")
            {
                return None;
            }
            Some(packaging_repo_finding(
                file,
                "public_api_surface_defined_only_by_import_side_effects",
                1,
                Severity::Info,
                "package root appears to define its public API primarily by import side effects"
                    .to_string(),
                format!("reexports={}", file.imports.len()),
            ))
        })
        .collect()
}

fn package_root_reexports_large_dependency_tree_by_default_findings(
    files: &[&ParsedFile],
) -> Vec<Finding> {
    files
        .iter()
        .filter(|file| file.path.file_name().and_then(|name| name.to_str()) == Some("__init__.py"))
        .filter_map(|file| {
            (file.imports.len() >= 6).then(|| {
                packaging_repo_finding(
                    file,
                    "package_root_reexports_large_dependency_tree_by_default",
                    1,
                    Severity::Info,
                    "package root re-exports a large dependency tree by default".to_string(),
                    format!("import_count={}", file.imports.len()),
                )
            })
        })
        .collect()
}

fn monolithic_common_package_becomes_transitive_dependency_for_most_modules_findings(
    files: &[&ParsedFile],
) -> Vec<Finding> {
    let import_count = files
        .iter()
        .flat_map(|file| file.imports.iter())
        .filter(|import| {
            let root = import.path.split('.').next().unwrap_or_default();
            matches!(root, "common" | "utils" | "shared" | "core")
        })
        .count();

    if import_count < 8 {
        return Vec::new();
    }

    let path = files
        .first()
        .map(|file| file.path.clone())
        .unwrap_or_default();
    vec![Finding {
        rule_id: "monolithic_common_package_becomes_transitive_dependency_for_most_modules"
            .to_string(),
        severity: Severity::Info,
        path,
        function_name: None,
        start_line: 1,
        end_line: 1,
        message: "repository relies heavily on a broad common/utils/shared package".to_string(),
        evidence: vec![format!("common_package_imports={import_count}")],
    }]
}
