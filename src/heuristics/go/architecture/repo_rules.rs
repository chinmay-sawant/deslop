use std::path::{Path, PathBuf};

fn project_agnostic_repo_shape_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary_findings(files));
    findings.extend(root_main_go_in_layered_service_repo_findings(files));
    findings.extend(upstream_consumed_interface_declared_in_provider_package_findings(files));
    findings.extend(tool_appeasement_noop_type_in_production_package_findings(files));
    findings
}

fn gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary_findings(
    files: &[&ParsedFile],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let layered_repo = files.iter().any(|file| is_service_file(file))
        && files.iter().any(|file| is_repository_file(file));
    if !layered_repo {
        return findings;
    }

    let bootstrap_signal = files.iter().find_map(first_gorm_bootstrap_signal);
    let Some((bootstrap_path, bootstrap_line, bootstrap_text)) = bootstrap_signal else {
        return findings;
    };

    for file in files {
        if file.is_test_file || !is_repository_file(file) || suppressed_raw_sql_strategy_file(file) {
            continue;
        }

        for function in &file.functions {
            let lines = body_lines(function);
            let Some(unwrap_line) = lines
                .iter()
                .find(|line| line.text.contains(".DB()"))
                .map(|line| line.line)
            else {
                continue;
            };

            let raw_sql_line = function
                .go_evidence()
                .db_query_calls
                .iter()
                .find(|call| {
                    matches!(
                        call.method_name.as_str(),
                        "Query"
                            | "QueryContext"
                            | "QueryRow"
                            | "QueryRowContext"
                            | "Exec"
                            | "ExecContext"
                            | "Get"
                            | "Select"
                    )
                })
                .map(|call| call.line);

            let Some(raw_sql_line) = raw_sql_line else {
                continue;
            };

            findings.push(function_finding(
                file,
                function,
                "gorm_bootstrap_with_raw_sql_repositories_without_adapter_boundary",
                Severity::Info,
                raw_sql_line,
                "repo uses GORM for bootstrap but repository code unwraps into raw SQL without a clear adapter boundary",
                vec![
                    format!(
                        "GORM bootstrap signal at {}:{} -> {}",
                        bootstrap_path.display(),
                        bootstrap_line,
                        bootstrap_text
                    ),
                    format!("repository unwraps a GORM handle via .DB() at line {unwrap_line}"),
                    format!("raw SQL-style query execution follows at line {raw_sql_line}"),
                ],
            ));
            return findings;
        }
    }

    findings
}

fn root_main_go_in_layered_service_repo_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let workspace_root = workspace_root_path(files);
    let Some(root_main) = files
        .iter()
        .find(|file| is_repo_root_main_file(file, &workspace_root))
    else {
        return findings;
    };

    let has_cmd_layout = files
        .iter()
        .any(|file| is_cmd_main_file(file, &workspace_root));
    if has_cmd_layout {
        return findings;
    }

    let non_main_files = files
        .iter()
        .copied()
        .filter(|file| !is_repo_root_main_file(file, &workspace_root))
        .collect::<Vec<_>>();
    if non_main_files.len() < 3 {
        return findings;
    }

    let has_service = non_main_files
        .iter()
        .any(|file| file_has_dedicated_role_home(file, SERVICE_ROLE_HINTS));
    let has_repository = non_main_files
        .iter()
        .any(|file| file_has_dedicated_role_home(file, REPOSITORY_ROLE_HINTS));
    let has_handler = non_main_files
        .iter()
        .any(|file| file_has_dedicated_role_home(file, TRANSPORT_ROLE_HINTS));
    let has_router = non_main_files
        .iter()
        .any(|file| file_has_dedicated_role_home(file, ROUTER_ROLE_HINTS));
    let distinct_packages = non_main_files
        .iter()
        .filter_map(|file| file.package_name.clone())
        .collect::<BTreeSet<_>>();

    if !(has_service && has_repository && (has_handler || has_router)) {
        return findings;
    }

    if distinct_packages.len() <= 1 {
        return findings;
    }

    let line = root_main
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "main")
        .map(|function| function.fingerprint.start_line)
        .unwrap_or(1);
    findings.push(file_finding(
        root_main,
        "root_main_go_in_layered_service_repo",
        Severity::Info,
        line,
        "repo uses a repository-root main.go even though the codebase already looks like a layered service",
        vec![
            format!(
                "layered role packages observed: service={}, repository={}, handler={}, router={}",
                has_service, has_repository, has_handler, has_router
            ),
            format!(
                "distinct non-main packages observed: {}",
                distinct_packages.len()
            ),
            "no cmd/<binary>/main.go entrypoint was observed".to_string(),
        ],
    ));

    findings
}

fn upstream_consumed_interface_declared_in_provider_package_findings(
    files: &[&ParsedFile],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for ((directory, package_name), package_files) in group_go_files_by_package(files) {
        let provider_package = package_files.iter().any(|file| {
            file_has_dedicated_role_home(file, REPOSITORY_ROLE_HINTS)
                || file_has_dedicated_role_home(file, SERVICE_ROLE_HINTS)
        });
        if !provider_package {
            continue;
        }

        let mut receiver_methods = BTreeMap::<String, BTreeSet<String>>::new();
        for file in &package_files {
            for symbol in &file.symbols {
                if matches!(symbol.kind, crate::model::SymbolKind::Method)
                    && let Some(receiver) = &symbol.receiver_type
                {
                    receiver_methods
                        .entry(receiver.clone())
                        .or_default()
                        .insert(symbol.name.clone());
                }
            }
        }

        for file in &package_files {
            if file.is_test_file || suppressed_provider_interface_file(file) {
                continue;
            }

            for interface in file.interfaces() {
                if interface.methods.is_empty() || (interface.is_pub && sdkish_package_path(file)) {
                    continue;
                }

                let impl_candidates = receiver_methods
                    .iter()
                    .filter(|(receiver, methods)| {
                        *receiver != &interface.name
                            && interface
                                .methods
                                .iter()
                                .all(|method| methods.contains(method))
                    })
                    .map(|(receiver, _)| receiver.clone())
                    .collect::<Vec<_>>();
                if impl_candidates.len() != 1 {
                    continue;
                }

                let upstream_references = upstream_interface_references(
                    files,
                    &directory,
                    &package_name,
                    &interface.name,
                );
                if upstream_references.is_empty() {
                    continue;
                }

                findings.push(file_finding(
                    file,
                    "upstream_consumed_interface_declared_in_provider_package",
                    Severity::Info,
                    interface.line,
                    format!(
                        "provider package declares interface {} even though upstream packages appear to consume the seam",
                        interface.name
                    ),
                    vec![
                        format!("package role home: {}", directory.display()),
                        format!(
                            "obvious concrete implementation in same package: {}",
                            impl_candidates[0]
                        ),
                        format!(
                            "upstream references: {}",
                            upstream_references.join(", ")
                        ),
                    ],
                ));
            }
        }
    }

    findings
}

fn tool_appeasement_noop_type_in_production_package_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for ((_directory, _package_name), package_files) in group_go_files_by_package(files) {
        for file in &package_files {
            if file.is_test_file || suppressed_noop_file(file) {
                continue;
            }

            for go_struct in file.go_structs() {
                let name_signal = noop_type_name_signal(&go_struct.name);
                if name_signal == NoopNameSignal::None {
                    continue;
                }

                let tooling_comment = nearest_tooling_comment(file, go_struct.line);
                if name_signal == NoopNameSignal::Secondary && tooling_comment.is_none() {
                    continue;
                }

                let receiver_methods = package_files
                    .iter()
                    .flat_map(|package_file| package_file.functions.iter())
                    .filter(|function| {
                        function.fingerprint.receiver_type.as_deref() == Some(go_struct.name.as_str())
                    })
                    .collect::<Vec<_>>();
                if receiver_methods.is_empty() {
                    continue;
                }

                let trivial_methods = receiver_methods
                    .iter()
                    .all(|function| noop_like_method(function));
                let usage_count =
                    non_method_struct_usage_count(files, &file.path, &go_struct.name);

                if tooling_comment.is_none() && !(trivial_methods && usage_count == 0) {
                    continue;
                }

                let mut evidence = vec![format!("suspicious production type name: {}", go_struct.name)];
                if let Some((line, text)) = tooling_comment {
                    evidence.push(format!("nearby tooling comment at line {line}: {text}"));
                }
                if trivial_methods {
                    evidence.push(format!(
                        "methods only return trivial values: {}",
                        receiver_methods
                            .iter()
                            .map(|function| function.fingerprint.name.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                evidence.push(format!(
                    "non-method production references to {}: {}",
                    go_struct.name, usage_count
                ));

                findings.push(file_finding(
                    file,
                    "tool_appeasement_noop_type_in_production_package",
                    Severity::Info,
                    go_struct.line,
                    format!(
                        "production type {} looks like a tooling-appeasement stub rather than a runtime implementation",
                        go_struct.name
                    ),
                    evidence,
                ));
            }
        }
    }

    findings
}

fn upstream_interface_references(
    files: &[&ParsedFile],
    directory: &Path,
    package_name: &str,
    interface_name: &str,
) -> Vec<String> {
    let mut references = Vec::new();

    for file in files {
        if file.is_test_file {
            continue;
        }

        let same_dir = file.path.parent().unwrap_or_else(|| Path::new("")) == directory;
        let same_package = file.package_name.as_deref() == Some(package_name);
        if same_dir && same_package {
            continue;
        }

        let aliases = package_import_aliases(file, package_name);
        if aliases.is_empty() {
            continue;
        }

        for alias in aliases {
            let qualified_name = format!("{alias}.{interface_name}");
            if let Some(function) = file
                .functions
                .iter()
                .find(|function| function.signature_text.contains(&qualified_name))
            {
                references.push(format!("{}:{} via {}", file.path.display(), function.fingerprint.start_line, qualified_name));
                break;
            }

            if let Some(field) = file
                .go_structs()
                .iter()
                .flat_map(|go_struct| go_struct.fields.iter())
                .find(|field| field.type_text.contains(&qualified_name))
            {
                references.push(format!("{}:{} via {}", file.path.display(), field.line, qualified_name));
                break;
            }
        }
    }

    references
}

fn package_import_aliases(file: &ParsedFile, package_name: &str) -> Vec<String> {
    file.imports
        .iter()
        .filter(|import| {
            import
                .path
                .rsplit('/')
                .next()
                .is_some_and(|segment| segment == package_name)
        })
        .filter(|import| import.alias != ".")
        .map(|import| import.alias.clone())
        .collect::<Vec<_>>()
}

fn group_go_files_by_package<'a>(
    files: &'a [&'a ParsedFile],
) -> BTreeMap<(PathBuf, String), Vec<&'a ParsedFile>> {
    let mut groups = BTreeMap::<(PathBuf, String), Vec<&ParsedFile>>::new();
    for file in files {
        let package_name = file
            .package_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let directory = file.path.parent().map(PathBuf::from).unwrap_or_default();
        groups
            .entry((directory, package_name))
            .or_default()
            .push(*file);
    }
    groups
}

fn workspace_root_path(files: &[&ParsedFile]) -> PathBuf {
    let mut root = files
        .first()
        .and_then(|file| file.path.parent().map(PathBuf::from))
        .unwrap_or_default();

    for file in files.iter().skip(1) {
        let Some(parent) = file.path.parent() else {
            continue;
        };
        while !parent.starts_with(&root) {
            if !root.pop() {
                break;
            }
        }
    }

    root
}

fn is_repo_root_main_file(file: &ParsedFile, workspace_root: &Path) -> bool {
    file.path
        .strip_prefix(workspace_root)
        .ok()
        .is_some_and(|relative| relative == Path::new("main.go"))
}

fn is_cmd_main_file(file: &ParsedFile, workspace_root: &Path) -> bool {
    file.path
        .strip_prefix(workspace_root)
        .ok()
        .and_then(|relative| relative.to_str())
        .is_some_and(|relative| {
            relative.starts_with("cmd/")
                && relative.ends_with("/main.go")
                && relative.trim_matches('/').split('/').count() >= 3
        })
}

fn suppressed_provider_interface_file(file: &ParsedFile) -> bool {
    let lower = file.path.to_string_lossy().to_ascii_lowercase();
    lower.contains("/adapter/")
        || lower.contains("/adapters/")
        || lower.contains("/mock/")
        || lower.contains("/mocks/")
        || lower.contains("generated")
}

fn sdkish_package_path(file: &ParsedFile) -> bool {
    let lower = file.path.to_string_lossy().to_ascii_lowercase();
    lower.contains("/sdk/")
        || lower.contains("/client/")
        || lower.contains("/pkg/")
}

fn suppressed_noop_file(file: &ParsedFile) -> bool {
    let lower = file.path.to_string_lossy().to_ascii_lowercase();
    lower.contains("/adapter/")
        || lower.contains("/adapters/")
        || lower.contains("/example/")
        || lower.contains("/examples/")
        || lower.contains("/sample/")
        || lower.contains("/samples/")
        || lower.contains("generated")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoopNameSignal {
    None,
    Primary,
    Secondary,
}

fn noop_type_name_signal(name: &str) -> NoopNameSignal {
    let lower = name.to_ascii_lowercase();
    if lower.contains("noop") || lower.contains("dummy") {
        return NoopNameSignal::Primary;
    }
    if lower.contains("stub") || lower.contains("mock") {
        return NoopNameSignal::Secondary;
    }
    NoopNameSignal::None
}

fn nearest_tooling_comment(file: &ParsedFile, target_line: usize) -> Option<(usize, String)> {
    file.comments
        .iter()
        .filter(|comment| comment.line <= target_line + 3)
        .filter(|comment| comment.line + 4 >= target_line)
        .find(|comment| tooling_comment_text(&comment.text))
        .map(|comment| (comment.line, compact_comment(&comment.text)))
}

fn tooling_comment_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    ["lint", "linter", "analyzer", "tooling", "satisfy", "appease", "staticcheck", "golangci"]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn compact_comment(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn first_gorm_bootstrap_signal(file: &&ParsedFile) -> Option<(PathBuf, usize, String)> {
    if file.is_test_file || is_repository_file(file) {
        return None;
    }

    for function in &file.functions {
        let lines = body_lines(function);
        if let Some(line) = lines
            .iter()
            .find(|line| line.text.contains("gorm.Open(") || line.text.contains(".AutoMigrate("))
        {
            return Some((
                file.path.clone(),
                line.line,
                compact_comment(&line.text),
            ));
        }
    }

    file.module_scope_calls
        .iter()
        .find(|call| call.text.contains("gorm.Open(") || call.text.contains(".AutoMigrate("))
        .map(|call| (file.path.clone(), call.line, compact_comment(&call.text)))
}

fn suppressed_raw_sql_strategy_file(file: &ParsedFile) -> bool {
    let lower = file.path.to_string_lossy().to_ascii_lowercase();
    lower.contains("/migration/")
        || lower.contains("/migrations/")
        || lower.contains("/adapter/")
        || lower.contains("/adapters/")
        || lower.contains("/sqlc/")
        || lower.contains("/query/")
        || lower.contains("/queries/")
}

fn noop_like_method(function: &ParsedFunction) -> bool {
    if function.fingerprint.line_count > 8 {
        return false;
    }

    let lines = body_lines(function)
        .into_iter()
        .filter(|line| !line.text.is_empty())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        return false;
    }

    lines.iter().all(|line| {
        line.text == "return"
            || line.text.starts_with("if ")
            || line.text.starts_with('}')
            || line.text.starts_with('{')
            || noop_like_return_line(&line.text)
    })
}

fn noop_like_return_line(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("return ") else {
        return false;
    };
    rest.split(',')
        .map(|part| part.trim())
        .all(|part| matches!(part, "nil" | "\"\"" | "0" | "false" | "true"))
}

fn non_method_struct_usage_count(files: &[&ParsedFile], defining_path: &Path, struct_name: &str) -> usize {
    let mut count = 0usize;

    for file in files {
        if file.is_test_file {
            continue;
        }

        for function in &file.functions {
            if function.fingerprint.receiver_type.as_deref() == Some(struct_name) {
                continue;
            }
            if file.path != defining_path
                && (function.signature_text.contains(struct_name)
                    || function.body_text.contains(struct_name))
            {
                count += 1;
            }
        }

        for field in file.go_structs().iter().flat_map(|go_struct| go_struct.fields.iter()) {
            if field.type_text.contains(struct_name) {
                count += 1;
            }
        }

        for package_var in file.package_vars() {
            if package_var
                .type_text
                .as_deref()
                .is_some_and(|text| text.contains(struct_name))
            {
                count += 1;
            }
        }
    }

    count
}
