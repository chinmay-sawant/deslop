fn file_architecture_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    let import_line = file.imports.first().map(|import| import.line).unwrap_or(1);
    let tag_map = struct_tag_map(file);
    let gorm_structs = gorm_struct_names(file, &tag_map);

    findings.extend(import_role_file_findings(file, import_line));
    findings.extend(struct_contract_file_findings(
        file,
        &tag_map,
        &gorm_structs,
    ));
    findings.extend(module_shape_file_findings(
        file,
        import_line,
        &tag_map,
        &gorm_structs,
    ));
    findings.extend(handler_module_file_findings(file));

    findings
}

fn import_role_file_findings(file: &ParsedFile, import_line: usize) -> Vec<Finding> {
    let mut findings = Vec::new();

    if is_service_file(file) && has_import_path(file, "github.com/gin-gonic/gin") {
        findings.push(file_finding(
            file,
            "service_imports_gin_directly",
            Severity::Warning,
            import_line,
            "service package depends on Gin directly",
            vec!["service-oriented files should prefer transport-neutral contracts over *gin.Context or Gin-only helpers".to_string()],
        ));
    }

    if is_repository_file(file)
        && (has_import_path(file, "github.com/gin-gonic/gin") || has_import_path(file, "net/http"))
    {
        findings.push(file_finding(
            file,
            "repository_depends_on_gin_or_http",
            Severity::Warning,
            import_line,
            "repository package imports transport-facing HTTP libraries",
            vec!["repositories should stay below the transport boundary".to_string()],
        ));
    }

    if is_repository_file(file) && import_path_has_any_role(file, SERVICE_ROLE_HINTS) {
        findings.push(file_finding(
            file,
            "repository_depends_on_service_package",
            Severity::Warning,
            import_line,
            "repository package depends on a service package",
            vec!["this inverts the usual dependency direction from service to repository".to_string()],
        ));
    }

    if is_model_file(file)
        && (has_import_path(file, "github.com/gin-gonic/gin")
            || has_import_path(file, "net/http")
            || import_path_has_any_role(file, TRANSPORT_ROLE_HINTS))
    {
        findings.push(file_finding(
            file,
            "model_package_depends_on_transport_or_gin",
            Severity::Warning,
            import_line,
            "model-oriented package depends on transport-facing code",
            vec!["models should not need Gin, net/http, or handler DTO packages".to_string()],
        ));
    }

    if is_service_file(file) && import_path_has_any_role(file, TRANSPORT_ROLE_HINTS) {
        findings.push(file_finding(
            file,
            "service_calls_handler_helper",
            Severity::Warning,
            import_line,
            "service file imports transport or handler helper packages",
            vec!["service logic should stay above repositories and below transport adapters".to_string()],
        ));
    }

    if is_transport_file(file)
        && file
            .go_structs()
            .iter()
            .any(|go_struct| go_struct.fields.iter().any(|field| field.type_text.contains("*gin.Context")))
    {
        let line = file
            .go_structs()
            .iter()
            .flat_map(|go_struct| go_struct.fields.iter())
            .find(|field| field.type_text.contains("*gin.Context"))
            .map(|field| field.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "gin_handler_stores_context_in_struct_field",
            Severity::Warning,
            line,
            "transport struct stores *gin.Context in a field",
            vec!["Gin request contexts are request-scoped and usually should not be retained on long-lived structs".to_string()],
        ));
    }

    if let Some(var) = file.package_vars().iter().find(|var| {
        is_transport_file(file)
            && (global_singleton_name(&var.name)
                || type_looks_like_global_singleton(var.type_text.as_deref().unwrap_or_default()))
    }) {
        findings.push(file_finding(
            file,
            "gin_handler_uses_global_singletons",
            Severity::Info,
            var.line,
            "transport package defines mutable singleton-like state",
            vec![format!("package variable {} looks like a shared runtime dependency", var.name)],
        ));
    }

    if file.package_vars().iter().any(|var| {
        var.type_text
            .as_deref()
            .is_some_and(|text| text.contains("*gin.Engine"))
    }) {
        let line = file
            .package_vars()
            .iter()
            .find(|var| {
                var.type_text
                    .as_deref()
                    .is_some_and(|text| text.contains("*gin.Engine"))
            })
            .map(|var| var.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "gin_engine_as_global_singleton",
            Severity::Warning,
            line,
            "package defines a package-level *gin.Engine singleton",
            vec!["explicit startup-owned router construction is usually easier to test and compose".to_string()],
        ));
    }

    if file_matches_any_role(file, &["helper", "helpers", "util", "utils", "common"])
        && (import_path_has_any_role(file, SERVICE_ROLE_HINTS)
            || import_path_has_any_role(file, REPOSITORY_ROLE_HINTS)
            || import_path_has_any_role(file, MODEL_ROLE_HINTS)
            || has_import_path(file, "gorm.io/gorm"))
    {
        findings.push(file_finding(
            file,
            "helper_or_utils_package_contains_domain_logic",
            Severity::Info,
            import_line,
            "generic helper or utils package imports domain-specific layers",
            vec!["generic packages usually become hard to govern when they absorb concrete business behavior".to_string()],
        ));
    }

    if file_matches_any_role(file, &["cmd", "main"])
        && (import_path_has_any_role(file, REPOSITORY_ROLE_HINTS)
            || has_import_path(file, "gorm.io/gorm")
            || has_sql_like_import(file))
    {
        findings.push(file_finding(
            file,
            "cmd_or_main_contains_domain_rules",
            Severity::Info,
            import_line,
            "main or cmd package depends directly on business persistence behavior",
            vec!["entrypoints are usually clearer when they focus on bootstrap and delegate business rules".to_string()],
        ));
    }

    if file.go_structs().iter().any(|s| s.name.ends_with("Validator")) && !file_matches_any_role(file, VALIDATION_ROLE_HINTS) {
        let line = file
            .go_structs()
            .iter()
            .find(|s| s.name.ends_with("Validator"))
            .map(|s| s.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "validator_outside_validation_package",
            Severity::Info,
            line,
            "validator type lives outside a validation-oriented package",
            vec!["shared validators are usually easier to find and reuse when they have a dedicated home".to_string()],
        ));
    }

    if file.go_structs().iter().any(|s| s.name.ends_with("Middleware")) && !file_matches_any_role(file, MIDDLEWARE_ROLE_HINTS) {
        let line = file
            .go_structs()
            .iter()
            .find(|s| s.name.ends_with("Middleware"))
            .map(|s| s.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "middleware_type_outside_middleware_package",
            Severity::Info,
            line,
            "middleware type lives outside a middleware-oriented package",
            vec!["cross-cutting middleware is usually easier to govern when it stays in a dedicated package".to_string()],
        ));
    }

    if let Some(literal) = file.pkg_strings.iter().find(|literal| {
        !is_repository_file(file) && !file.is_test_file && looks_like_sql_literal(&literal.value)
    }) {
        findings.push(file_finding(
            file,
            "sql_query_constants_outside_repository_package",
            Severity::Info,
            literal.line,
            "SQL query template is declared outside a repository-oriented package",
            vec![format!("query-like literal assigned to {}", literal.name)],
        ));
    }

    if let Some(function) = file.functions.iter().find(|function| {
        !is_repository_file(file)
            && !file_matches_any_role(file, UOW_ROLE_HINTS)
            && transaction_helper_name(&function.fingerprint.name)
    }) {
        findings.push(file_finding(
            file,
            "transaction_helper_outside_repository_or_uow_package",
            Severity::Info,
            function.fingerprint.start_line,
            "transaction helper lives outside repository or unit-of-work packages",
            vec!["transaction orchestration helpers are usually easier to reason about when grouped near repository or unit-of-work code".to_string()],
        ));
    }

    if file_matches_any_role(file, &["common", "base", "shared", "utils"])
        && mixed_role_symbol_count(file) >= 3
    {
        findings.push(file_finding(
            file,
            "shared_package_named_common_base_utils_with_mixed_exports",
            Severity::Info,
            1,
            "shared package mixes several architectural roles under one generic namespace",
            vec!["generic shared packages often become dumping grounds for unrelated concerns".to_string()],
        ));
    }

    if cross_layer_import_violation(file) {
        findings.push(file_finding(
            file,
            "cross_layer_import_violation_by_package_name",
            Severity::Warning,
            import_line,
            "package imports appear to violate the expected layer direction",
            vec!["the import graph suggests a lower layer reaching upward into transport or service code".to_string()],
        ));
    }

    if let Some((line, role, mismatch)) = role_drift(file) {
        findings.push(file_finding(
            file,
            "package_name_role_drift",
            Severity::Info,
            line,
            format!("package named {role} mainly exports {mismatch}-oriented symbols"),
            vec!["package naming becomes misleading when most exported types belong to another role".to_string()],
        ));
    }

    findings
}

fn struct_contract_file_findings(
    file: &ParsedFile,
    tag_map: &BTreeMap<&str, Vec<&StructTag>>,
    gorm_structs: &BTreeSet<String>,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for go_struct in file.go_structs() {
        if go_struct.name.ends_with("Service") && !is_service_file(file) {
            findings.push(file_finding(
                file,
                "service_type_outside_service_package",
                Severity::Info,
                go_struct.line,
                format!("service type {} lives outside a service-oriented package", go_struct.name),
                vec!["layered repos usually keep concrete services under service or services packages".to_string()],
            ));
        }

        if repository_type_name(&go_struct.name) && !is_repository_file(file) {
            findings.push(file_finding(
                file,
                "repository_type_outside_repository_package",
                Severity::Info,
                go_struct.line,
                format!(
                    "repository-like type {} lives outside a repository-oriented package",
                    go_struct.name
                ),
                vec!["repository, repo, or store implementations are easier to govern when they live together".to_string()],
            ));
        }

        if request_struct_name(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["json:", "form:", "uri:", "binding:", "validate:"])
            && !is_transport_file(file)
        {
            findings.push(file_finding(
                file,
                "request_dto_outside_transport_package",
                Severity::Info,
                go_struct.line,
                format!("request DTO {} lives outside a transport-oriented package", go_struct.name),
                vec!["request binding contracts are usually easier to own at the API boundary".to_string()],
            ));
        }

        if response_struct_name(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["json:"])
            && !is_transport_file(file)
        {
            findings.push(file_finding(
                file,
                "response_dto_outside_transport_package",
                Severity::Info,
                go_struct.line,
                format!(
                    "response DTO {} lives outside a transport-oriented package",
                    go_struct.name
                ),
                vec!["response contracts are usually easier to evolve when they stay at the boundary".to_string()],
            ));
        }

        if api_error_struct_name(&go_struct.name) && !is_transport_file(file) {
            findings.push(file_finding(
                file,
                "api_error_type_outside_transport_package",
                Severity::Info,
                go_struct.line,
                format!(
                    "API-facing error contract {} lives outside a transport-oriented package",
                    go_struct.name
                ),
                vec!["client error payload types usually belong with transport contracts".to_string()],
            ));
        }

        if gorm_structs.contains(&go_struct.name) && !is_model_file(file) {
            findings.push(file_finding(
                file,
                "gorm_model_outside_models_package",
                Severity::Info,
                go_struct.line,
                format!("GORM-backed struct {} lives outside a model-oriented package", go_struct.name),
                vec!["repos that already separate handlers, services, and repositories usually benefit from a model or models package for persistence structs".to_string()],
            ));
        }
    }

    for go_struct in file.go_structs() {
        if request_struct_name(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["gorm:"])
        {
            findings.push(file_finding(
                file,
                "request_dto_contains_gorm_tags",
                Severity::Warning,
                go_struct.line,
                format!("request DTO {} carries GORM tags", go_struct.name),
                vec!["request binding contracts should not double as persistence schema".to_string()],
            ));
        }

        if response_struct_name(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["gorm:"])
        {
            findings.push(file_finding(
                file,
                "response_dto_contains_gorm_tags",
                Severity::Warning,
                go_struct.line,
                format!("response DTO {} carries GORM tags", go_struct.name),
                vec!["response contracts should not leak persistence mapping tags".to_string()],
            ));
        }

        if gorm_structs.contains(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["binding:", "validate:"])
        {
            findings.push(file_finding(
                file,
                "gorm_model_contains_binding_or_validate_tags",
                Severity::Warning,
                go_struct.line,
                format!(
                    "GORM model {} carries request validation or binding tags",
                    go_struct.name
                ),
                vec!["persistence structs are usually cleaner when boundary validation stays in dedicated DTOs".to_string()],
            ));
        }

        if gorm_structs.contains(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["form:", "uri:"])
        {
            findings.push(file_finding(
                file,
                "persistence_model_contains_http_form_or_uri_tags",
                Severity::Warning,
                go_struct.line,
                format!(
                    "GORM model {} carries form or URI binding tags",
                    go_struct.name
                ),
                vec!["persistence models should not have to understand HTTP binding details".to_string()],
            ));
        }
    }

    if file.go_structs().iter().any(|go_struct| {
        repeated_tagged_struct_use(tag_map, go_struct, &["gorm:"])
            && repeated_tagged_struct_use(tag_map, go_struct, &["json:", "binding:", "validate:"])
    }) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| {
                repeated_tagged_struct_use(tag_map, go_struct, &["gorm:"])
                    && repeated_tagged_struct_use(tag_map, go_struct, &["json:", "binding:", "validate:"])
            })
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "same_struct_used_for_bind_persist_and_respond",
            Severity::Warning,
            line,
            "one struct appears to serve transport, validation, and persistence concerns at once",
            vec!["shared bind-persist-respond structs usually make contracts harder to evolve".to_string()],
        ));
    }

    if file_matches_any_role(file, &["entity", "domain"])
        && file.go_structs().iter().any(|go_struct| {
            repeated_tagged_struct_use(tag_map, go_struct, &["json:"])
                && !request_struct_name(&go_struct.name)
                && !response_struct_name(&go_struct.name)
        })
    {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| {
                repeated_tagged_struct_use(tag_map, go_struct, &["json:"])
                    && !request_struct_name(&go_struct.name)
                    && !response_struct_name(&go_struct.name)
            })
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "domain_entity_contains_json_tags_without_boundary_exception",
            Severity::Info,
            line,
            "domain or entity struct carries JSON transport tags",
            vec!["core entities are usually cleaner when serialization concerns stay at the boundary".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| {
        (request_struct_name(&go_struct.name) || response_struct_name(&go_struct.name))
            && struct_has_sql_null_field(go_struct)
    }) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| {
                (request_struct_name(&go_struct.name) || response_struct_name(&go_struct.name))
                    && struct_has_sql_null_field(go_struct)
            })
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "nullable_sql_types_leak_into_api_dto",
            Severity::Info,
            line,
            "API DTO exposes sql.Null* field types",
            vec!["boundary-facing contracts usually read better with transport-friendly optional types".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| first_non_pointer_patch_field(go_struct).is_some()) {
        let line = file
            .go_structs()
            .iter()
            .find_map(first_non_pointer_patch_field)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "patch_dto_uses_non_pointer_fields_for_optional_updates",
            Severity::Warning,
            line,
            "PATCH-like DTO uses non-pointer scalar fields",
            vec!["partial update contracts usually need pointer or explicit field-intent semantics to distinguish omitted from zero".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| {
        (go_struct.name.contains("CreateUpdate") || go_struct.name.contains("Upsert"))
            && repeated_tagged_struct_use(tag_map, go_struct, &["binding:"])
    }) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| {
                (go_struct.name.contains("CreateUpdate") || go_struct.name.contains("Upsert"))
                    && repeated_tagged_struct_use(tag_map, go_struct, &["binding:"])
            })
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "create_and_update_share_same_dto_despite_conflicting_requiredness",
            Severity::Info,
            line,
            "single DTO appears to cover both create and update semantics",
            vec!["create and update contracts often diverge on requiredness and are usually clearer as separate types".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| {
        response_struct_name(&go_struct.name) && repeated_tagged_struct_use(tag_map, go_struct, &[",omitempty"])
    }) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| {
                response_struct_name(&go_struct.name)
                    && repeated_tagged_struct_use(tag_map, go_struct, &[",omitempty"])
            })
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "response_dto_uses_omitempty_on_required_contract_fields",
            Severity::Info,
            line,
            "response DTO uses omitempty on contract fields",
            vec!["stable API contracts usually read more clearly when required fields are always present".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| gorm_structs.contains(&go_struct.name) && model_has_calculated_field(go_struct).is_some()) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| gorm_structs.contains(&go_struct.name) && model_has_calculated_field(go_struct).is_some())
            .and_then(model_has_calculated_field)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "gorm_model_contains_calculated_response_fields",
            Severity::Info,
            line,
            "GORM model carries calculated presentation-oriented fields",
            vec!["API presentation fields are usually easier to evolve in DTOs or mappers than in persistence structs".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| gorm_structs.contains(&go_struct.name) && model_spans_multiple_subdomains(go_struct)) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| gorm_structs.contains(&go_struct.name) && model_spans_multiple_subdomains(go_struct))
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "giant_model_struct_spans_multiple_subdomains",
            Severity::Info,
            line,
            "large persistence model appears to mix several subdomain concerns",
            vec!["splitting oversized models often makes ownership boundaries easier to maintain".to_string()],
        ));
    }

    findings
}

fn module_shape_file_findings(
    file: &ParsedFile,
    import_line: usize,
    tag_map: &BTreeMap<&str, Vec<&StructTag>>,
    gorm_structs: &BTreeSet<String>,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    if is_repository_file(file)
        && file
            .imports
            .iter()
            .any(|import| import.path.contains("prometheus") || import.path.contains("otel"))
    {
        findings.push(file_finding(
            file,
            "transport_metrics_emitted_from_repository_layer",
            Severity::Info,
            import_line,
            "repository package imports transport-oriented metrics or tracing dependencies",
            vec!["repository metrics can be valid, but HTTP-route or request-surface metrics usually belong higher in the stack".to_string()],
        ));
    }

    if is_model_file(file)
        && (file.comments.iter().any(|comment| {
            let lower = comment.text.to_ascii_lowercase();
            lower.contains("@summary") || lower.contains("@router") || lower.contains("@param")
        })
            || file
                .imports
                .iter()
                .any(|import| import.path.contains("swag") || import.path.contains("/docs")))
    {
        let line = file
            .comments
            .iter()
            .find(|comment| {
                let lower = comment.text.to_ascii_lowercase();
                lower.contains("@summary") || lower.contains("@router") || lower.contains("@param")
            })
            .map(|comment| comment.line)
            .or_else(|| {
                file.imports
                    .iter()
                    .find(|import| import.path.contains("swag") || import.path.contains("/docs"))
                    .map(|import| import.line)
            })
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "swagger_or_openapi_annotations_on_persistence_models",
            Severity::Info,
            line,
            "persistence model file carries OpenAPI or Swagger annotations",
            vec!["public docs contracts usually read more clearly when they describe transport DTOs instead of persistence models".to_string()],
        ));
    }

    if (is_repository_file(file) || is_service_file(file))
        && file
            .imports
            .iter()
            .any(|import| import.path.contains("swag") || import.path.contains("/docs"))
    {
        findings.push(file_finding(
            file,
            "repository_or_service_packages_import_docs_or_generator_annotations",
            Severity::Info,
            import_line,
            "core package imports docs or generator-oriented dependencies",
            vec!["business and persistence packages usually stay cleaner when docs tooling stays at the boundary".to_string()],
        ));
    }

    if let Some(line) = is_transport_file(file)
        .then(|| first_domain_constant_line(file))
        .flatten()
    {
        findings.push(file_finding(
            file,
            "domain_constants_declared_in_handler_package",
            Severity::Info,
            line,
            "transport package declares domain status, role, or state constants",
            vec!["domain lifecycle constants are usually easier to reuse from domain-facing packages".to_string()],
        ));
    }

    if let Some(line) = (!file_matches_any_role(file, MAPPER_ROLE_HINTS)
        && import_path_has_any_role(file, REPOSITORY_ROLE_HINTS))
        .then(|| first_mapper_symbol_line(file))
        .flatten()
    {
        findings.push(file_finding(
            file,
            "mapper_outside_mapper_package_when_repo_uses_mappers",
            Severity::Info,
            line,
            "mapping helpers live outside a mapper-oriented package",
            vec!["repos that already lean on explicit mapping usually benefit from keeping those adapters together".to_string()],
        ));
    }

    if let Some(line) = (!is_router_file(file) && is_transport_file(file))
        .then(|| route_registration_line_in_file(file))
        .flatten()
    {
        findings.push(file_finding(
            file,
            "route_setup_scattered_without_router_package",
            Severity::Info,
            line,
            "route registration lives outside a router-oriented package",
            vec!["larger Gin services usually stay easier to navigate when route setup has a clear home".to_string()],
        ));
    }

    if !is_router_file(file)
        && is_transport_file(file)
        && file.functions.iter().any(|function| {
            let name = function.fingerprint.name.as_str();
            (name.contains("Route") || name.contains("Register"))
                && body_lines(function).iter().any(|line| {
                    [".GET(", ".POST(", ".PUT(", ".PATCH(", ".DELETE("]
                        .iter()
                        .any(|marker| line.text.contains(marker))
                })
        })
    {
        let line = file
            .functions
            .iter()
            .find(|function| {
                let name = function.fingerprint.name.as_str();
                (name.contains("Route") || name.contains("Register"))
                    && body_lines(function).iter().any(|line| {
                        [".GET(", ".POST(", ".PUT(", ".PATCH(", ".DELETE("]
                            .iter()
                            .any(|marker| line.text.contains(marker))
                    })
            })
            .map(|function| function.fingerprint.start_line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "route_setup_scattered_without_router_package",
            Severity::Info,
            line,
            "route registration is spread into a non-router transport file",
            vec!["larger Gin services usually benefit from giving route setup a dedicated router home".to_string()],
        ));
    }

    if !is_router_file(file)
        && is_transport_file(file)
        && file.path.to_string_lossy().to_ascii_lowercase().contains("route")
    {
        findings.push(file_finding(
            file,
            "route_setup_scattered_without_router_package",
            Severity::Info,
            1,
            "route setup lives in a non-router transport file",
            vec!["larger Gin services usually benefit from giving route setup a dedicated router home".to_string()],
        ));
    }

    if !is_router_file(file) {
        let lower_path = file.path.to_string_lossy().to_ascii_lowercase();
        if lower_path.contains("transport") && lower_path.ends_with("routes.go") {
            findings.push(file_finding(
                file,
                "route_setup_scattered_without_router_package",
                Severity::Info,
                1,
                "route setup file lives under transport instead of a router package",
                vec!["larger Gin services usually benefit from giving route setup a dedicated router home".to_string()],
            ));
        }
    }

    if let Some(line) = (is_transport_file(file) || is_service_file(file))
        .then(|| mutable_config_package_var(file))
        .flatten()
    {
        findings.push(file_finding(
            file,
            "package_level_mutable_config_used_by_handlers_services",
            Severity::Info,
            line,
            "package-level mutable config is available to request-path code",
            vec!["request-path code is usually easier to reason about when config is injected rather than read from shared mutable globals".to_string()],
        ));
    }

    if is_repository_file(file)
        && file.package_vars().iter().any(|var| {
            var.type_text
                .as_deref()
                .is_some_and(|text| text.contains("*gorm.DB"))
                && var.value_text
                    .as_deref()
                    .is_some_and(|value| value.contains(".Session(") || value.contains(".WithContext("))
        })
    {
        let line = file
            .package_vars()
            .iter()
            .find(|var| {
                var.type_text
                    .as_deref()
                    .is_some_and(|text| text.contains("*gorm.DB"))
                    && var.value_text
                        .as_deref()
                        .is_some_and(|value| value.contains(".Session(") || value.contains(".WithContext("))
            })
            .map(|var| var.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "shared_gorm_db_state_mutated_and_reused_across_requests",
            Severity::Warning,
            line,
            "repository package mutates and stores shared *gorm.DB state",
            vec!["shared mutable GORM handles can make request-path behavior harder to predict".to_string()],
        ));
    }

    if file.functions.iter().filter(|function| function.fingerprint.name == "TableName").count() >= 2 {
        let line = file
            .functions
            .iter()
            .find(|function| function.fingerprint.name == "TableName")
            .map(|function| function.fingerprint.start_line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "table_name_override_or_scope_logic_duplicated_across_models",
            Severity::Info,
            line,
            "several TableName overrides live together in one model file",
            vec!["repeated model-level table customization can be a sign that a shared convention is missing".to_string()],
        ));
    }

    if !file.is_test_file {
        for function in file.functions.iter().filter(|function| !function.is_test_function) {
            if test_fixture_builder_line(function).is_some() {
                findings.push(file_finding(
                    file,
                    "test_fixture_builders_live_in_production_packages",
                    Severity::Info,
                    function.fingerprint.start_line,
                    "production file exports test fixture builder helpers",
                    vec!["fixture and test factory helpers are usually clearer when they stay in test-only packages".to_string()],
                ));
                break;
            }
        }
    }

    if !file.is_test_file
        && file
            .imports
            .iter()
            .any(|import| import.path.contains("/test") || import.path.contains("testsupport") || import.path.contains("/fixtures"))
    {
        findings.push(file_finding(
            file,
            "test_bootstrap_package_reused_by_production_wiring",
            Severity::Info,
            import_line,
            "production file imports test-only bootstrap or fixture helpers",
            vec!["production wiring usually stays easier to govern when it does not depend on test support packages".to_string()],
        ));
    }

    let _ = (tag_map, gorm_structs);
    findings
}
