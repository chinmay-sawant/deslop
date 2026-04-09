fn authorization_business_logic_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            (lower.contains("role") || lower.contains("permission") || lower.contains("authorize"))
                && (lower.contains("if ") || lower.contains("switch "))
        })
        .map(|line| line.line)
}

fn action_switch_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("action")
                && (lower.contains("switch") || lower.contains("if "))
                || lower.contains("mode")
                    && (lower.contains("switch") || lower.contains("if "))
        })
        .map(|line| line.line)
}

fn mixes_html_json_and_file(calls: &[crate::analysis::GinCallSummary]) -> bool {
    multiple_response_shapes(calls)
}

fn passes_gin_context_beyond_boundary(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("(c)") || line.text.contains(", c)") || line.text.contains("go func(c")
        })
        .map(|line| line.line)
}

fn global_singleton_reference_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            ["DB.", "Client.", "Logger.", "Config.", "Settings.", "Global"]
                .iter()
                .any(|pattern| line.text.contains(pattern))
        })
        .map(|line| line.line)
}

fn retry_or_backoff_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("retry") || lower.contains("backoff") || lower.contains("sleep(")
        })
        .map(|line| line.line)
}

fn constructor_instantiates_dependencies(lines: &[super::framework_patterns::BodyLine]) -> bool {
    lines.iter().any(|line| constructor_dependency_line(&line.text))
}

fn constructor_dependency_line(text: &str) -> bool {
    text.contains("NewRepo(")
        || text.contains("NewRepository(")
        || text.contains("NewClient(")
        || text.contains("NewStore(")
        || text.contains("log.New(")
        || text.contains("gorm.Open(")
}

fn pagination_or_query_parsing_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("pagesize")
                || lower.contains("page_size")
                || lower.contains("page")
                    && (lower.contains("atoi(") || lower.contains("parseint(") || lower.contains("query("))
                || lower.contains("sort=")
        })
        .map(|line| line.line)
}

fn request_binding_or_header_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains(".Header(")
                || line.text.contains(".GetHeader(")
                || line.text.contains(".FormValue(")
                || line.text.contains(".Param(")
                || line.text.contains(".Query(")
        })
        .map(|line| line.line)
}

fn mutates_request_binding_in_place(
    params_text: &str,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    let requestish = REQUEST_STRUCT_SUFFIXES
        .iter()
        .find(|suffix| params_text.contains(**suffix))?;
    lines
        .iter()
        .find(|line| line.text.contains('=') && line.text.contains('.'))
        .filter(|_line| params_text.contains(*requestish))
        .map(|line| line.line)
}

fn not_found_nil_nil_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("return nil, nil"))
        .map(|line| line.line)
}

fn transaction_start_line(text: &str) -> bool {
    text.contains(".Begin()") || text.contains(".BeginTx(") || text.contains(".Transaction(")
}

fn repeated_route_path(lines: &[super::framework_patterns::BodyLine]) -> Option<(usize, String)> {
    let mut seen = BTreeMap::<String, usize>::new();
    for line in lines {
        if let Some(path) = quoted_route_path(&line.text) {
            if let Some(first_line) = seen.get(&path) {
                return Some((*first_line, path));
            }
            seen.insert(path, line.line);
        }
    }
    None
}

fn quoted_route_path(text: &str) -> Option<String> {
    let start = text.find("\"/")?;
    let rest = &text[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn route_registration_contains_business_logic_line(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    if !is_router_file(file) {
        return None;
    }
    if function.go_evidence().gorm_query_chains.is_empty() && function.go_evidence().db_query_calls.is_empty() {
        return None;
    }
    Some(
        function
            .go_evidence()
            .gorm_query_chains
            .first()
            .map(|chain| chain.line)
            .or_else(|| function.go_evidence().db_query_calls.first().map(|call| call.line))
            .or_else(|| lines.first().map(|line| line.line))
            .unwrap_or(function.fingerprint.start_line),
    )
}

fn router_constructs_dependencies_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| constructor_dependency_line(&line.text) || line.text.contains("http.Client{"))
        .map(|line| line.line)
}

fn background_worker_registration_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("startworker")
                || lower.contains("schedule")
                || lower.contains("cron")
                || lower.contains("go ")
        })
        .map(|line| line.line)
}

fn admin_debug_route_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("\"/admin") || line.text.contains("\"/debug"))
        .map(|line| line.line)
}

fn danger_named(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("hard") || lower.contains("purge") || lower.contains("admin") || lower.contains("danger")
}

fn external_http_inside_transaction_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    action_inside_transaction_line(
        lines,
        &import_aliases_for(file, "net/http")
            .iter()
            .flat_map(|alias| [format!("{alias}.Get("), format!("{alias}.Post("), format!("{alias}.Do(")])
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )
}

fn action_inside_transaction_line(
    lines: &[super::framework_patterns::BodyLine],
    markers: &[&str],
) -> Option<usize> {
    let start = lines.iter().find(|line| transaction_start_line(&line.text))?.line;
    let end = lines
        .iter()
        .find(|line| line.line > start && (line.text.contains(".Commit(") || line.text.contains(".Rollback(")))
        .map(|line| line.line)
        .unwrap_or(usize::MAX);
    lines
        .iter()
        .find(|line| line.line > start && line.line < end && markers.iter().any(|marker| line.text.contains(marker)))
        .map(|line| line.line)
}

fn is_main_or_cmd_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, &["cmd", "main"])
}

fn route_registration_line_in_file(file: &ParsedFile) -> Option<usize> {
    file.module_scope_calls
        .iter()
        .find(|call| is_route_registration_name(&call.name))
        .map(|call| call.line)
        .or_else(|| {
            file.functions.iter().find_map(|function| {
                body_lines(function)
                    .iter()
                    .find(|line| {
                        [".GET(", ".POST(", ".PUT(", ".PATCH(", ".DELETE(", ".Any(", ".Use("]
                            .iter()
                            .any(|marker| line.text.contains(marker))
                    })
                    .map(|line| line.line)
            })
        })
}

fn is_route_registration_name(name: &str) -> bool {
    matches!(name, "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "Any" | "Use" | "Group")
}

fn first_domain_constant_line(file: &ParsedFile) -> Option<usize> {
    file.top_level_bindings
        .iter()
        .find(|binding| {
            let name = binding.name.as_str();
            let value = binding.value_text.to_ascii_lowercase();
            (name.ends_with("Status")
                || name.ends_with("State")
                || name.ends_with("Type")
                || name.ends_with("Role"))
                && (value.contains('"') || value.contains("iota"))
        })
        .map(|binding| binding.line)
        .or_else(|| {
            file.pkg_strings
                .iter()
                .find(|literal| {
                    let name = literal.name.as_str();
                    name.ends_with("Status")
                        || name.ends_with("State")
                        || name.ends_with("Type")
                        || name.ends_with("Role")
                })
                .map(|literal| literal.line)
        })
}

fn first_mapper_symbol_line(file: &ParsedFile) -> Option<usize> {
    file.go_structs()
        .iter()
        .find(|go_struct| go_struct.name.ends_with("Mapper"))
        .map(|go_struct| go_struct.line)
        .or_else(|| {
            file.functions
                .iter()
                .find(|function| {
                    let name = function.fingerprint.name.as_str();
                    name.starts_with("Map")
                        || name.contains("ToModel")
                        || name.contains("ToDTO")
                        || name.contains("ToResponse")
                })
                .map(|function| function.fingerprint.start_line)
        })
}

fn struct_has_sql_null_field(go_struct: &crate::analysis::GoStructSummary) -> bool {
    go_struct
        .fields
        .iter()
        .any(|field| field.type_text.contains("sql.Null"))
}

fn first_non_pointer_patch_field(go_struct: &crate::analysis::GoStructSummary) -> Option<usize> {
    if !(go_struct.name.contains("Patch") || go_struct.name.contains("Update")) {
        return None;
    }

    go_struct
        .fields
        .iter()
        .find(|field| {
            let ty = field.type_text.trim();
            !ty.starts_with('*')
                && !ty.starts_with("[]")
                && !ty.contains("map[")
                && !ty.contains("sql.Null")
                && matches!(
                    ty,
                    "string"
                        | "bool"
                        | "int"
                        | "int32"
                        | "int64"
                        | "uint"
                        | "uint32"
                        | "uint64"
                        | "float32"
                        | "float64"
                        | "time.Time"
                )
        })
        .map(|field| field.line)
}

fn model_has_calculated_field(go_struct: &crate::analysis::GoStructSummary) -> Option<usize> {
    go_struct
        .fields
        .iter()
        .find(|field| {
            let name = field.name.as_str();
            name.ends_with("Display")
                || name.ends_with("Label")
                || name.ends_with("URL")
                || name.ends_with("Count")
                || name.ends_with("Summary")
        })
        .map(|field| field.line)
}

fn model_spans_multiple_subdomains(go_struct: &crate::analysis::GoStructSummary) -> bool {
    if go_struct.fields.len() < 10 {
        return false;
    }

    let mut domains = BTreeSet::new();
    for field in &go_struct.fields {
        let lower = field.name.to_ascii_lowercase();
        if ["billing", "invoice", "card"].iter().any(|marker| lower.contains(marker)) {
            domains.insert("billing");
        }
        if ["shipping", "address", "delivery"].iter().any(|marker| lower.contains(marker)) {
            domains.insert("shipping");
        }
        if ["profile", "avatar", "bio"].iter().any(|marker| lower.contains(marker)) {
            domains.insert("profile");
        }
        if ["inventory", "stock", "sku"].iter().any(|marker| lower.contains(marker)) {
            domains.insert("inventory");
        }
        if ["tenant", "org", "workspace"].iter().any(|marker| lower.contains(marker)) {
            domains.insert("tenant");
        }
    }

    domains.len() >= 3
}

fn repeated_tagged_struct_use(
    tag_map: &BTreeMap<&str, Vec<&StructTag>>,
    go_struct: &crate::analysis::GoStructSummary,
    markers: &[&str],
) -> bool {
    has_any_tag_for_struct(tag_map, &go_struct.name, markers)
}

fn handler_recovery_line(function: &ParsedFunction) -> Option<usize> {
    body_lines(function)
        .iter()
        .find(|line| line.text.contains("recover()") || line.text.contains("recover("))
        .map(|line| line.line)
}

fn request_identity_extraction_line(
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("MustGet(\"user")
                || line.text.contains("MustGet(\"tenant")
                || line.text.contains("Get(\"user")
                || line.text.contains("Get(\"tenant")
                || line.text.contains("GetString(\"request_id")
        })
        .map(|line| line.line)
}

fn request_id_generation_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("uuid.new") || lower.contains("request-id") || lower.contains("request_id")
        })
        .map(|line| line.line)
}

fn pagination_binding_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("Query(\"page")
                || line.text.contains("DefaultQuery(\"page")
                || line.text.contains("Query(\"page_size")
                || line.text.contains("DefaultQuery(\"page_size")
        })
        .map(|line| line.line)
}

fn response_envelope_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("gin.H{\"data\"")
                || line.text.contains("gin.H{\"meta\"")
                || (line.text.contains("Response{") && line.text.contains("Data:"))
        })
        .map(|line| line.line)
}

fn required_validation_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("== \"\"")
                || lower.contains("== 0")
                || lower.contains("len(") && lower.contains("== 0")
        })
        .map(|line| line.line)
}

fn validation_error_shape(lines: &[super::framework_patterns::BodyLine]) -> Option<&'static str> {
    lines.iter().find_map(|line| {
        if line.text.contains("\"errors\"") {
            Some("errors")
        } else if line.text.contains("\"error\"") {
            Some("error")
        } else if line.text.contains("\"message\"") {
            Some("message")
        } else {
            None
        }
    })
}

fn default_injection_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("pagesize =")
                || lower.contains("page_size =")
                || lower.contains("page = 1")
                || lower.contains("limit =")
        })
        .map(|line| line.line)
}

fn path_param_parse_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains("strconv.Atoi(") || line.text.contains("uuid.Parse("))
                && line.text.contains(".Param(")
        })
        .map(|line| line.line)
}

fn pagination_bounds_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            (lower.contains("pagesize") || lower.contains("page_size"))
                && (lower.contains(">") || lower.contains("<"))
        })
        .map(|line| line.line)
}

fn sort_whitelist_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            (lower.contains("sort") || lower.contains("filter"))
                && (lower.contains("switch") || lower.contains("allowed") || lower.contains("whitelist"))
        })
        .map(|line| line.line)
}

fn route_param_merge_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let has_bind = lines.iter().any(|line| {
        let lower = line.text.to_ascii_lowercase();
        lower.contains("bind") || lower.contains("shouldbind")
    });
    if !has_bind {
        return None;
    }
    lines
        .iter()
        .find(|line| line.text.contains(".Param(") || line.text.contains(".Query("))
        .map(|line| line.line)
}

fn upload_write_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let has_upload_validation = lines.iter().any(|line| line.text.contains("FormFile(") || line.text.contains("MultipartForm("));
    if !has_upload_validation {
        return None;
    }
    lines
        .iter()
        .find(|line| line.text.contains("SaveUploadedFile(") || line.text.contains("io.Copy("))
        .map(|line| line.line)
}

fn route_param_drift_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains(".Param(\"id\")") && line.text.contains("UserID"))
                || (line.text.contains(".Param(\"user_id\")") && line.text.contains("ID ="))
        })
        .map(|line| line.line)
}

fn error_mapping_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    if !lines.iter().any(|line| line.text.contains("errors.Is(") || line.text.contains("errors.As(")) {
        return None;
    }
    first_http_status_line(file, lines)
}

fn handler_error_shape_key(lines: &[super::framework_patterns::BodyLine]) -> Option<&'static str> {
    lines.iter().find_map(|line| {
        if line.text.contains("\"error\"") && line.text.contains("\"code\"") {
            Some("error+code")
        } else if line.text.contains("\"errors\"") {
            Some("errors")
        } else if line.text.contains("\"message\"") && line.text.contains("\"status\"") {
            Some("message+status")
        } else {
            None
        }
    })
}

fn error_string_switch_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("err.Error()"))
        .map(|line| line.line)
}

fn raw_db_error_response_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("err.Error()")
                && (line.text.contains("c.JSON(") || line.text.contains("gin.H") || line.text.contains("http.Error("))
        })
        .map(|line| line.line)
}

fn success_payload_with_error_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("\"data\"") && line.text.contains("\"error\""))
        .map(|line| line.line)
}

fn repeated_named_literals(file: &ParsedFile, matcher: impl Fn(&NamedLiteral) -> bool) -> Option<(usize, String)> {
    let mut counts = BTreeMap::<String, usize>::new();
    let mut lines = BTreeMap::<String, usize>::new();
    for literal in &file.pkg_strings {
        if matcher(literal) {
            *counts.entry(literal.value.clone()).or_default() += 1;
            lines.entry(literal.value.clone()).or_insert(literal.line);
        }
    }
    counts
        .into_iter()
        .find(|(_, count)| *count >= 2)
        .and_then(|(value, _)| lines.get(&value).copied().map(|line| (line, value)))
}

fn table_or_column_literal(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    !lower.contains(' ')
        && (lower.ends_with("s") || lower.ends_with("_id") || lower.ends_with("_at"))
        && lower.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_' || ch.is_ascii_digit())
}

fn gorm_scope_count(function: &ParsedFunction) -> usize {
    body_lines(function)
        .iter()
        .map(|line| line.text.match_indices("Scopes(func").count())
        .sum()
}

fn preload_count(function: &ParsedFunction) -> usize {
    let parsed = function
        .go_evidence()
        .gorm_query_chains
        .iter()
        .flat_map(|chain| chain.steps.iter())
        .filter(|step| step.method_name == "Preload")
        .count();
    if parsed > 0 {
        parsed
    } else {
        body_lines(function)
            .iter()
            .map(|line| line.text.match_indices(".Preload(").count())
            .sum()
    }
}

fn soft_delete_line_count(lines: &[super::framework_patterns::BodyLine]) -> usize {
    lines
        .iter()
        .map(|line| {
            let lower = line.text.to_ascii_lowercase();
            if lower.contains("deleted_at") && lower.contains("is null") {
                lower.match_indices("deleted_at").count()
            } else {
                0
            }
        })
        .sum()
}

fn locking_clause_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("Locking") || line.text.contains("FOR UPDATE"))
        .map(|line| line.line)
}

fn update_struct_without_field_intent_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains(".Updates(") || line.text.contains(".Save("))
                && !line.text.contains("Select(")
                && !line.text.contains("Omit(")
        })
        .map(|line| line.line)
}

fn map_update_flow_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let has_map = lines
        .iter()
        .any(|line| line.text.contains("map[string]any{") || line.text.contains("map[string]interface{}{"));
    if !has_map {
        return None;
    }
    lines
        .iter()
        .find(|line| line.text.contains("Update(") || line.text.contains("Updates("))
        .map(|line| line.line)
}

fn base_repository_reflection_line(
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    if function.fingerprint.name.contains("Base")
        || function.fingerprint.name.contains("Generic")
        || function.fingerprint.receiver_type.as_ref().is_some_and(|receiver| receiver.contains("Base") || receiver.contains("Generic"))
    {
        return lines
            .iter()
            .find(|line| line.text.contains("reflect."))
            .map(|line| line.line);
    }
    None
}

fn transaction_cross_layer_line(function: &ParsedFunction, lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    if !signature_mentions_transaction(&function.signature_text) {
        return None;
    }
    lines
        .iter()
        .find(|line| line.text.contains("svc.") || line.text.contains("service.") || line.text.contains("repo."))
        .map(|line| line.line)
}

fn cross_repo_write_without_tx_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let repo_refs = repository_receiver_count(lines);
    let has_write = lines.iter().any(|line| {
        line.text.contains(".Create(")
            || line.text.contains(".Save(")
            || line.text.contains(".Delete(")
            || line.text.contains(".Updates(")
    });
    if repo_refs >= 2 && has_write && !lines.iter().any(|line| transaction_start_line(&line.text)) {
        return lines.first().map(|line| line.line);
    }
    None
}

fn split_commit_rollback_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let has_commit = lines.iter().any(|line| line.text.contains(".Commit("));
    let has_rollback = lines.iter().any(|line| line.text.contains(".Rollback("));
    if has_commit && has_rollback && !lines.iter().any(|line| line.text.contains("defer")) {
        lines.iter()
            .find(|line| line.text.contains(".Commit(") || line.text.contains(".Rollback("))
            .map(|line| line.line)
    } else {
        None
    }
}

fn nested_tx_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("SavePoint(") || line.text.contains("RollbackTo("))
        .map(|line| line.line)
        .or_else(|| {
            let tx_begins = lines
                .iter()
                .filter(|line| transaction_start_line(&line.text))
                .count();
            if tx_begins >= 2 {
                lines.iter().find(|line| transaction_start_line(&line.text)).map(|line| line.line)
            } else {
                None
            }
        })
}

fn mutable_config_package_var(file: &ParsedFile) -> Option<usize> {
    file.package_vars()
        .iter()
        .find(|var| {
            let lower = var.name.to_ascii_lowercase();
            (lower.contains("config") || lower.contains("setting"))
                && !var.value_text.as_deref().unwrap_or_default().contains("const")
        })
        .map(|var| var.line)
}

fn feature_flag_lookup_line(file: &ParsedFile, lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    first_matching_line(
        lines,
        &config_lookup_patterns(file)
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )
}

fn lifecycle_start_without_shutdown_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let start_line = lines
        .iter()
        .find(|line| line.text.contains(".Run(") || line.text.contains("ListenAndServe("))
        .map(|line| line.line)?;
    let has_shutdown = lines.iter().any(|line| {
        line.text.contains("Shutdown(") || line.text.contains("Close(") || line.text.contains("Stop(")
    });
    if has_shutdown {
        None
    } else {
        Some(start_line)
    }
}

fn test_fixture_builder_line(function: &ParsedFunction) -> Option<usize> {
    let name = function.fingerprint.name.as_str();
    if name.starts_with("NewTest") || name.contains("Fixture") || name.starts_with("BuildTest") {
        Some(function.fingerprint.start_line)
    } else {
        None
    }
}

fn raw_json_assertion_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains("{\"") || line.text.contains("assert.JSONEq("))
                && (line.text.contains("assert.") || line.text.contains("require.") || line.text.contains("cmp."))
        })
        .map(|line| line.line)
}

fn gin_context_stub_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("gin.CreateTestContext("))
        .map(|line| line.line)
}

fn transport_test_repo_touch_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("NewRepo(") || line.text.contains("Repository{"))
        .map(|line| line.line)
}

fn sql_query_assertion_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains("SELECT ") || line.text.contains("UPDATE ") || line.text.contains("DELETE "))
                && (line.text.contains("assert.") || line.text.contains("require.") || line.text.contains("Expect("))
        })
        .map(|line| line.line)
}

fn table_driven_multi_domain_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("[]struct")
                && (line.text.contains("user") || line.text.contains("order") || line.text.contains("invoice"))
        })
        .map(|line| line.line)
}

fn metrics_label_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("prometheus.Labels{") || line.text.contains("WithLabelValues("))
        .map(|line| line.line)
}

fn tracing_span_name_literal(line: &str) -> Option<String> {
    let marker = "Start(";
    let idx = line.find(marker)?;
    let rest = &line[idx + marker.len()..];
    let first_quote = rest.find('"')?;
    let rest = &rest[first_quote + 1..];
    let end_quote = rest.find('"')?;
    Some(rest[..end_quote].to_string())
}

fn repository_log_http_metadata_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains("logger.") || line.text.contains("log."))
                && (line.text.contains("status") || line.text.contains("route") || line.text.contains("path"))
        })
        .map(|line| line.line)
}

fn audit_before_service_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let audit_line = lines
        .iter()
        .find(|line| line.text.to_ascii_lowercase().contains("audit"))
        .map(|line| line.line)?;
    let service_line = lines
        .iter()
        .find(|line| line.text.contains("svc.") || line.text.contains("service."))
        .map(|line| line.line)?;
    if audit_line < service_line {
        Some(audit_line)
    } else {
        None
    }
}

fn request_logging_field_key(line: &str) -> Option<&'static str> {
    if line.contains("\"request_id\"") {
        Some("request_id")
    } else if line.contains("\"requestId\"") {
        Some("requestId")
    } else if line.contains("\"user_id\"") {
        Some("user_id")
    } else if line.contains("\"userId\"") {
        Some("userId")
    } else {
        None
    }
}

fn health_handler_repo_line(function: &ParsedFunction, lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let lower = function.fingerprint.name.to_ascii_lowercase();
    if !(lower.contains("health") || lower.contains("ready")) {
        return None;
    }
    if references_repository_directly(lines) {
        Some(function.fingerprint.start_line)
    } else {
        None
    }
}

fn migration_or_seed_handler_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| migration_line(&line.text) || line.text.contains("Seed("))
        .map(|line| line.line)
}

fn api_example_literal_line(function: &ParsedFunction) -> Option<usize> {
    function
        .local_strings
        .iter()
        .find(|literal| literal.value.contains("{\"") && literal.value.contains("example"))
        .map(|literal| literal.line)
        .or_else(|| {
            body_lines(function)
                .iter()
                .find(|line| line.text.contains("{\\\"example\\\"") || line.text.contains("{\"example\""))
                .map(|line| line.line)
        })
}

fn extract_constructor_name(text: &str) -> Option<String> {
    let start = text.find("New")?;
    let rest = &text[start..];
    let end = rest.find('(')?;
    let name = &rest[..end];
    if name.len() > 3
        && name.chars().next().is_some_and(|ch| ch == 'N')
        && name.chars().all(|ch| ch.is_ascii_alphanumeric())
    {
        Some(name.to_string())
    } else {
        None
    }
}

fn has_duplicate_string(values: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    values.iter().any(|value| !seen.insert(value.as_str()))
}

fn extract_status_constant(text: &str) -> Option<String> {
    for part in text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '.')) {
        if part.contains("Status") {
            return Some(part.to_string());
        }
    }
    None
}

fn extract_domain_error_name(text: &str) -> Option<String> {
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .find(|token| token.starts_with("Err") && token.len() > 3)
        .map(ToString::to_string)
}
