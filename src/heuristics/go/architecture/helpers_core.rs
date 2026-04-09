fn struct_tag_map(file: &ParsedFile) -> BTreeMap<&str, Vec<&StructTag>> {
    let mut map = BTreeMap::<&str, Vec<&StructTag>>::new();
    for tag in file.struct_tags() {
        map.entry(tag.struct_name.as_str()).or_default().push(tag);
    }
    map
}

fn gorm_struct_names(file: &ParsedFile, tag_map: &BTreeMap<&str, Vec<&StructTag>>) -> BTreeSet<String> {
    file.go_structs()
        .iter()
        .filter(|go_struct| has_any_tag_for_struct(tag_map, &go_struct.name, &["gorm:"]))
        .map(|go_struct| go_struct.name.clone())
        .collect()
}

fn has_any_tag_for_struct(
    tag_map: &BTreeMap<&str, Vec<&StructTag>>,
    struct_name: &str,
    markers: &[&str],
) -> bool {
    tag_map.get(struct_name).is_some_and(|tags| {
        tags.iter()
            .any(|tag| markers.iter().any(|marker| tag.raw_tag.contains(marker)))
    })
}

fn file_finding(
    file: &ParsedFile,
    rule_id: &str,
    severity: Severity,
    line: usize,
    message: impl Into<String>,
    evidence: Vec<String>,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: None,
        start_line: line,
        end_line: line,
        message: message.into(),
        evidence,
    }
}

fn function_finding(
    file: &ParsedFile,
    function: &ParsedFunction,
    rule_id: &str,
    severity: Severity,
    line: usize,
    message: impl Into<String>,
    evidence: Vec<String>,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: message.into(),
        evidence,
    }
}

fn is_service_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, SERVICE_ROLE_HINTS)
}

fn is_repository_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, REPOSITORY_ROLE_HINTS)
}

fn is_model_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, MODEL_ROLE_HINTS)
}

fn is_transport_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, TRANSPORT_ROLE_HINTS)
}

fn is_router_file(file: &ParsedFile) -> bool {
    file_has_dedicated_role_home(file, ROUTER_ROLE_HINTS)
}

fn is_middleware_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, MIDDLEWARE_ROLE_HINTS)
}

fn file_matches_any_role(file: &ParsedFile, hints: &[&str]) -> bool {
    let path = file.path.to_string_lossy().to_ascii_lowercase();
    let package = file.package_name.as_deref().unwrap_or_default().to_ascii_lowercase();

    hints.iter().any(|hint| {
        path.contains(&format!("/{hint}/"))
            || path.contains(&format!("/{hint}_"))
            || path.contains(&format!("_{hint}."))
            || path.ends_with(&format!("/{hint}.go"))
            || path.contains(hint)
            || package == *hint
    })
}

fn file_has_dedicated_role_home(file: &ParsedFile, hints: &[&str]) -> bool {
    let path = file.path.to_string_lossy().to_ascii_lowercase();
    let package = file.package_name.as_deref().unwrap_or_default().to_ascii_lowercase();

    hints.iter().any(|hint| {
        package == *hint
            || path.contains(&format!("/{hint}/"))
            || path.ends_with(&format!("/{hint}.go"))
    })
}

fn import_path_has_any_role(file: &ParsedFile, hints: &[&str]) -> bool {
    file.imports.iter().any(|import| {
        let path = import.path.to_ascii_lowercase();
        hints.iter().any(|hint| {
            path.contains(&format!("/{hint}/"))
                || path.ends_with(&format!("/{hint}"))
                || path.contains(&format!("/{hint}_"))
                || path.contains(hint)
        })
    })
}

fn repository_type_name(name: &str) -> bool {
    name.ends_with("Repository") || name.ends_with("Repo") || name.ends_with("Store")
}

fn request_struct_name(name: &str) -> bool {
    REQUEST_STRUCT_SUFFIXES
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn response_struct_name(name: &str) -> bool {
    RESPONSE_STRUCT_SUFFIXES
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn api_error_struct_name(name: &str) -> bool {
    API_ERROR_STRUCT_SUFFIXES
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn signature_returns_text(signature: &str) -> &str {
    let Some((_, close_idx)) = signature_group_bounds(signature, params_group_index(signature)) else {
        return "";
    };
    signature[close_idx + 1..].trim()
}

fn signature_params_text(signature: &str) -> &str {
    let Some((open_idx, close_idx)) = signature_group_bounds(signature, params_group_index(signature)) else {
        return "";
    };
    &signature[open_idx + 1..close_idx]
}

fn params_group_index(signature: &str) -> usize {
    if signature.trim_start().starts_with("func (") {
        1
    } else {
        0
    }
}

fn signature_group_bounds(signature: &str, group_index: usize) -> Option<(usize, usize)> {
    let mut depth = 0usize;
    let mut current_open = None;
    let mut groups = Vec::new();

    for (idx, ch) in signature.char_indices() {
        match ch {
            '(' => {
                if depth == 0 {
                    current_open = Some(idx);
                }
                depth += 1;
            }
            ')' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    groups.push((current_open?, idx));
                }
            }
            _ => {}
        }
    }

    groups.get(group_index).copied()
}

fn has_sql_like_import(file: &ParsedFile) -> bool {
    [
        "database/sql",
        "github.com/jmoiron/sqlx",
        "github.com/jackc/pgx/v5",
        "github.com/jackc/pgx/v4",
    ]
    .iter()
    .any(|path| has_import_path(file, path))
}

fn is_body_bind_operation(operation: &str) -> bool {
    matches!(
        operation,
        "bind_json"
            | "should_bind_json"
            | "bind"
            | "should_bind"
            | "should_bind_body_with"
    )
}

fn model_import_aliases(file: &ParsedFile) -> Vec<String> {
    file.imports
        .iter()
        .filter(|import| {
            let path = import.path.to_ascii_lowercase();
            MODEL_ROLE_HINTS.iter().any(|hint| {
                path.contains(&format!("/{hint}/")) || path.ends_with(&format!("/{hint}"))
            })
        })
        .map(|import| import.alias.clone())
        .collect()
}

fn binding_looks_like_model(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
    gorm_structs: &BTreeSet<String>,
    binding: &str,
) -> bool {
    let aliases = model_import_aliases(file);

    lines.iter().any(|line| {
        (line.text.contains(&format!("var {binding} "))
            || line.text.contains(&format!("{binding} :="))
            || line.text.contains(&format!("{binding}=")))
            && (aliases
                .iter()
                .any(|alias| line.text.contains(&format!("{alias}.")))
                || gorm_structs
                    .iter()
                    .any(|struct_name| line.text.contains(struct_name)))
    })
}

fn expression_looks_like_model(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
    gorm_structs: &BTreeSet<String>,
    expression: &str,
) -> bool {
    let trimmed = expression.trim().trim_start_matches('&');
    if binding_looks_like_model(file, lines, gorm_structs, trimmed) {
        return true;
    }

    let aliases = model_import_aliases(file);
    aliases
        .iter()
        .any(|alias| expression.contains(&format!("{alias}.")))
        || gorm_structs
            .iter()
            .any(|struct_name| expression.contains(struct_name))
}

fn references_repository_directly(lines: &[super::framework_patterns::BodyLine]) -> bool {
    repository_receiver_count(lines) > 0
}

fn references_service_directly(lines: &[super::framework_patterns::BodyLine]) -> bool {
    receiver_count_with_suffix(lines, &["Service"]) > 0
}

fn repository_receiver_count(lines: &[super::framework_patterns::BodyLine]) -> usize {
    receiver_count_with_suffix(lines, &["Repo", "Repository", "Store"])
}

fn receiver_count_with_suffix(
    lines: &[super::framework_patterns::BodyLine],
    suffixes: &[&str],
) -> usize {
    let mut names = BTreeSet::new();

    for line in lines {
        for token in line
            .text
            .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        {
            if suffixes.iter().any(|suffix| token.ends_with(suffix)) {
                names.insert(token.to_string());
            }
        }
    }

    names.len()
}

fn first_config_lookup_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    first_matching_line(
        lines,
        &config_lookup_patterns(file)
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )
}

fn first_env_lookup_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    first_matching_line(
        lines,
        &env_lookup_patterns(file)
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )
}

fn first_http_status_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    let http_aliases = import_aliases_for(file, "net/http");
    let mut patterns = http_aliases
        .iter()
        .map(|alias| format!("{alias}.Status"))
        .collect::<Vec<_>>();
    patterns.push("AbortWithStatus".to_string());
    patterns.push("AbortWithStatusJSON".to_string());

    first_matching_line(lines, &patterns.iter().map(String::as_str).collect::<Vec<_>>())
}

fn has_http_status_usage(file: &ParsedFile, lines: &[super::framework_patterns::BodyLine]) -> bool {
    first_http_status_line(file, lines).is_some()
}

fn first_http_or_abort_semantics_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    let http_aliases = import_aliases_for(file, "net/http");
    let mut patterns = http_aliases
        .iter()
        .map(|alias| format!("{alias}.Status"))
        .collect::<Vec<_>>();
    patterns.extend([
        "Abort(".to_string(),
        "AbortWithStatus".to_string(),
        "AbortWithStatusJSON".to_string(),
    ]);

    first_matching_line(lines, &patterns.iter().map(String::as_str).collect::<Vec<_>>())
}

fn config_lookup_patterns(file: &ParsedFile) -> Vec<String> {
    let mut patterns = env_lookup_patterns(file);
    patterns.extend([
        "viper.Get".to_string(),
        "featureFlag".to_string(),
        "FeatureFlag".to_string(),
        "flag.Lookup".to_string(),
    ]);
    patterns
}

fn env_lookup_patterns(file: &ParsedFile) -> Vec<String> {
    let mut patterns = import_aliases_for(file, "os")
        .iter()
        .flat_map(|alias| {
            [
                format!("{alias}.Getenv("),
                format!("{alias}.LookupEnv("),
            ]
        })
        .collect::<Vec<_>>();

    if patterns.is_empty() {
        patterns.extend(["os.Getenv(".to_string(), "os.LookupEnv(".to_string()]);
    }

    patterns
}

fn returns_framework_builder(returns_text: &str) -> bool {
    [
        "*gorm.DB",
        "*sql.Rows",
        "*sql.Row",
        "*sqlx.Rows",
        "pgx.Rows",
        "sql.Rows",
    ]
    .iter()
    .any(|pattern| returns_text.contains(pattern))
}

fn returns_transport_dto(returns_text: &str) -> bool {
    RESPONSE_STRUCT_SUFFIXES
        .iter()
        .any(|suffix| returns_text.contains(suffix))
        || returns_text.contains("Envelope")
        || returns_text.contains("View")
}

fn signature_has_request_dto(params_text: &str) -> bool {
    REQUEST_STRUCT_SUFFIXES
        .iter()
        .any(|suffix| params_text.contains(suffix))
}

fn signature_mentions_transaction(signature: &str) -> bool {
    signature.contains("tx *gorm.DB")
        || signature.contains("tx *sql.Tx")
        || signature.contains("tx *sqlx.Tx")
}

fn constructor_like(function: &ParsedFunction) -> bool {
    function.fingerprint.name.starts_with("New") || function.fingerprint.name.starts_with("Build")
}

fn migration_line(text: &str) -> bool {
    text.contains("AutoMigrate(") || text.contains("Migrate(") || text.contains("Migration")
}

fn init_registers_dependencies_or_routes(text: &str) -> bool {
    text.contains(".GET(")
        || text.contains(".POST(")
        || text.contains(".PUT(")
        || text.contains(".DELETE(")
        || text.contains(".Use(")
        || text.contains("NewService(")
        || text.contains("NewRepository(")
        || text.contains("gin.Default(")
        || text.contains("gin.New(")
}

fn is_gorm_hook(
    file: &ParsedFile,
    function: &ParsedFunction,
    gorm_structs: &BTreeSet<String>,
) -> bool {
    has_import_path(file, "gorm.io/gorm")
        && function
            .fingerprint
            .receiver_type
            .as_ref()
            .is_some_and(|receiver| gorm_structs.contains(receiver))
        && GORM_HOOK_METHODS.contains(&function.fingerprint.name.as_str())
}

fn first_external_io_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    let mut patterns = Vec::new();
    for alias in import_aliases_for(file, "net/http") {
        patterns.push(format!("{alias}.Get("));
        patterns.push(format!("{alias}.Post("));
        patterns.push(format!("{alias}.Do("));
    }
    for alias in import_aliases_for(file, "os") {
        patterns.push(format!("{alias}.WriteFile("));
        patterns.push(format!("{alias}.Create("));
        patterns.push(format!("{alias}.Open("));
    }
    for alias in import_aliases_for(file, "os/exec") {
        patterns.push(format!("{alias}.Command("));
    }
    patterns.push("publish(".to_string());
    patterns.push("Publish(".to_string());

    first_matching_line(lines, &patterns.iter().map(String::as_str).collect::<Vec<_>>())
}

fn first_matching_line(
    lines: &[super::framework_patterns::BodyLine],
    patterns: &[&str],
) -> Option<usize> {
    lines
        .iter()
        .find(|line| patterns.iter().any(|pattern| line.text.contains(pattern)))
        .map(|line| line.line)
}

fn global_singleton_name(name: &str) -> bool {
    matches!(name, "DB" | "Client" | "Logger" | "Config" | "Settings" | "Engine" | "Repo")
}

fn type_looks_like_global_singleton(type_text: &str) -> bool {
    ["*gorm.DB", "*sql.DB", "*gin.Engine", "*http.Client", "Logger", "Config"]
        .iter()
        .any(|pattern| type_text.contains(pattern))
}

fn looks_like_sql_literal(value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    ["SELECT ", "INSERT ", "UPDATE ", "DELETE ", " FROM ", " WHERE "]
        .iter()
        .any(|pattern| upper.contains(pattern))
}

fn transaction_helper_name(name: &str) -> bool {
    name.contains("Transaction") || name.contains("Transactional") || name.ends_with("Tx") || name.starts_with("WithTx")
}

fn mixed_role_symbol_count(file: &ParsedFile) -> usize {
    let mut roles = BTreeSet::new();
    for s in file.go_structs() {
        let name = s.name.as_str();
        if name.ends_with("Service") {
            roles.insert("service");
        }
        if repository_type_name(name) {
            roles.insert("repository");
        }
        if request_struct_name(name) || response_struct_name(name) {
            roles.insert("transport");
        }
        if name.ends_with("Validator") {
            roles.insert("validation");
        }
    }
    roles.len()
}

fn cross_layer_import_violation(file: &ParsedFile) -> bool {
    (is_repository_file(file) && import_path_has_any_role(file, TRANSPORT_ROLE_HINTS))
        || (is_service_file(file) && import_path_has_any_role(file, TRANSPORT_ROLE_HINTS))
        || (is_model_file(file) && import_path_has_any_role(file, TRANSPORT_ROLE_HINTS))
}

fn role_drift(file: &ParsedFile) -> Option<(usize, &'static str, &'static str)> {
    if is_service_file(file)
        && file.go_structs().iter().filter(|s| repository_type_name(&s.name)).count() > file.go_structs().iter().filter(|s| s.name.ends_with("Service")).count()
    {
        let line = file.go_structs().first().map(|s| s.line).unwrap_or(1);
        return Some((line, "service", "repository"));
    }
    if is_model_file(file)
        && file.go_structs().iter().filter(|s| request_struct_name(&s.name) || response_struct_name(&s.name)).count() > file.go_structs().iter().filter(|s| !request_struct_name(&s.name) && !response_struct_name(&s.name)).count()
    {
        let line = file.go_structs().first().map(|s| s.line).unwrap_or(1);
        return Some((line, "model", "transport"));
    }
    None
}

fn multiple_response_shapes(calls: &[crate::analysis::GinCallSummary]) -> bool {
    let kinds = calls
        .iter()
        .filter_map(|call| match call.operation.as_str() {
            "json" | "pure_json" | "indented_json" => Some("json"),
            "html" => Some("html"),
            "data" | "file" => Some("file"),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    kinds.len() >= 2
}
