fn function_architecture_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let lines = body_lines(function);
    let tag_map = struct_tag_map(file);
    let gorm_structs = gorm_struct_names(file, &tag_map);
    let returns_text = signature_returns_text(&function.signature_text);
    let params_text = signature_params_text(&function.signature_text);

    let mut findings = Vec::new();
    findings.extend(handler_function_findings(file, function, &lines, &tag_map, &gorm_structs));
    findings.extend(service_function_findings(
        file,
        function,
        &lines,
        &gorm_structs,
        returns_text,
        params_text,
    ));
    findings.extend(repository_function_findings(
        file,
        function,
        &lines,
        returns_text,
        params_text,
    ));
    findings.extend(middleware_and_bootstrap_function_findings(
        file,
        function,
        &lines,
        &gorm_structs,
    ));
    findings.extend(transaction_and_misc_function_findings(
        file,
        function,
        &lines,
        &gorm_structs,
        returns_text,
    ));
    findings
}

fn handler_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
    tag_map: &BTreeMap<&str, Vec<&StructTag>>,
    gorm_structs: &BTreeSet<String>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    let go = function.go_evidence();

    if references_repository_directly(lines) && !references_service_directly(lines) {
        findings.push(function_finding(
            file,
            function,
            "handler_calls_repository_directly_without_service",
            Severity::Warning,
            function.fingerprint.start_line,
            "request handler reaches repository-like dependencies directly without an obvious service seam",
            vec!["direct repository orchestration from handlers often signals missing service-layer ownership".to_string()],
        ));
    }

    if !go.gorm_query_chains.is_empty() {
        let line = go.gorm_query_chains[0].line;
        findings.push(function_finding(
            file,
            function,
            "handler_calls_gorm_directly_outside_repository",
            Severity::Warning,
            line,
            "request handler builds GORM query chains directly",
            vec!["handlers usually read more clearly when GORM query shape lives in repository methods".to_string()],
        ));
        findings.push(function_finding(
            file,
            function,
            "gorm_chain_built_in_handler",
            Severity::Warning,
            line,
            "request handler owns GORM query construction directly",
            vec!["query-shape ownership is usually easier to test and reuse inside repositories".to_string()],
        ));
    }

    if !go.db_query_calls.is_empty() && has_sql_like_import(file) {
        let line = go.db_query_calls[0].line;
        findings.push(function_finding(
            file,
            function,
            "handler_calls_database_sql_directly_outside_repository",
            Severity::Warning,
            line,
            "request handler executes SQL-oriented calls directly",
            vec!["handlers usually should not own driver-level query execution".to_string()],
        ));
    }

    if is_gin_handler(file, function) {
        let body_bind_count = go
            .gin_calls
            .iter()
            .filter(|call| is_body_bind_operation(&call.operation))
            .count();
        if body_bind_count >= 2 {
            let line = go
                .gin_calls
                .iter()
                .filter(|call| is_body_bind_operation(&call.operation))
                .nth(1)
                .map(|call| call.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "gin_handler_accepts_more_than_one_body_contract",
                Severity::Info,
                line,
                "Gin handler appears to bind more than one body contract",
                vec!["one request path usually benefits from one clear body contract".to_string()],
            ));
        }

        for gin_call in go.gin_calls {
            let bind_targets_model = gin_call
                .assigned_binding
                .as_deref()
                .is_some_and(|binding| binding_looks_like_model(file, lines, gorm_structs, binding))
                || gin_call.argument_texts.iter().any(|argument: &String| {
                    argument
                        .trim()
                        .strip_prefix('&')
                        .is_some_and(|binding| binding_looks_like_model(file, lines, gorm_structs, binding))
                        || expression_looks_like_model(file, lines, gorm_structs, argument)
                });

            if is_body_bind_operation(&gin_call.operation) && bind_targets_model {
                findings.push(function_finding(
                    file,
                    function,
                    "gin_handler_binds_directly_into_model",
                    Severity::Warning,
                    gin_call.line,
                    "Gin handler binds request input directly into a persistence model",
                    vec!["binding into GORM-backed structs couples transport validation to persistence shape".to_string()],
                ));
                findings.push(function_finding(
                    file,
                    function,
                    "dto_to_model_mapping_in_handler",
                    Severity::Info,
                    gin_call.line,
                    "handler maps request input straight into persistence model state",
                    vec!["transport-to-model mapping is usually easier to evolve behind mappers or service inputs".to_string()],
                ));
            }

            if matches!(gin_call.operation.as_str(), "json" | "pure_json")
                && gin_call
                    .argument_texts
                    .last()
                    .is_some_and(|arg| expression_looks_like_model(file, lines, gorm_structs, arg))
            {
                findings.push(function_finding(
                    file,
                    function,
                    "gin_handler_returns_persistence_model_directly",
                    Severity::Warning,
                    gin_call.line,
                    "Gin handler serializes a persistence model directly",
                    vec!["response DTOs are usually easier to evolve than exposing database model shape".to_string()],
                ));
                findings.push(function_finding(
                    file,
                    function,
                    "model_to_dto_mapping_in_handler",
                    Severity::Info,
                    gin_call.line,
                    "handler shapes client response directly from persistence model data",
                    vec!["model-to-response mapping is usually easier to reuse in mapper or renderer helpers".to_string()],
                ));
            }
        }

        if multiple_response_shapes(go.gin_calls) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_returns_multiple_response_shapes",
                Severity::Info,
                function.fingerprint.start_line,
                "Gin handler mixes several response rendering styles",
                vec!["handlers that return multiple unrelated response shapes are often trying to cover too many behaviors".to_string()],
            ));
        }
        if !multiple_response_shapes(go.gin_calls)
            && lines.iter().any(|line| line.text.contains(".JSON("))
            && (lines.iter().any(|line| line.text.contains(".HTML("))
                || lines.iter().any(|line| line.text.contains(".File("))
                || lines.iter().any(|line| line.text.contains(".Data(")))
        {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_returns_multiple_response_shapes",
                Severity::Info,
                function.fingerprint.start_line,
                "Gin handler mixes several response rendering styles",
                vec!["handlers that return multiple unrelated response shapes are often trying to cover too many behaviors".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "gin_handler_mixes_html_json_and_file_responses",
                Severity::Info,
                function.fingerprint.start_line,
                "Gin handler mixes HTML, JSON, or file-style responses",
                vec!["combining several transport modes in one handler often hurts clarity".to_string()],
            ));
        }

        if let Some(query_call) = go.db_query_calls.iter().find(|call| call.query_text.is_some()) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_executes_raw_sql_inline",
                Severity::Warning,
                query_call.line,
                "Gin handler executes literal SQL inline",
                vec!["request handlers usually should not own raw SQL strings".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "raw_sql_literal_in_handler",
                Severity::Warning,
                query_call.line,
                "request handler owns a literal SQL statement",
                query_call
                    .query_text
                    .as_ref()
                    .map(|query| vec![format!("literal query: {query}")])
                    .unwrap_or_default(),
            ));
        }

        if let Some(line) = required_validation_line(lines)
            && (go.gin_calls.iter().any(|call| {
                is_body_bind_operation(&call.operation)
                    && call.assigned_binding.as_deref().is_some_and(|binding| {
                        file.go_structs().iter().any(|go_struct| {
                            go_struct.name == binding
                                && repeated_tagged_struct_use(tag_map, go_struct, &["binding:", "validate:"])
                        })
                    })
            }) || file.go_structs().iter().any(|go_struct| {
                request_struct_name(&go_struct.name)
                    && repeated_tagged_struct_use(tag_map, go_struct, &["binding:", "validate:"])
            }))
        {
            findings.push(function_finding(
                file,
                function,
                "manual_required_checks_after_validate_tags_available",
                Severity::Info,
                line,
                "handler manually re-checks required fields after binding into a validated DTO",
                vec!["boundary validation is usually clearer when one validation mechanism owns required-field checks".to_string()],
            ));
        }

        if lines.iter().any(|line| line.text.contains("validator.New(") || line.text.contains("RegisterValidation(")) {
            let line = lines
                .iter()
                .find(|line| line.text.contains("validator.New(") || line.text.contains("RegisterValidation("))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "custom_validator_registration_inside_handler",
                Severity::Info,
                line,
                "handler registers validators inline",
                vec!["validator registration is usually easier to govern in startup wiring than per-handler code".to_string()],
            ));
        }

        if repository_receiver_count(lines) >= 2 {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_calls_multiple_repositories_directly",
                Severity::Info,
                function.fingerprint.start_line,
                "Gin handler coordinates multiple repository-like dependencies directly",
                vec!["this usually signals missing service orchestration or an overgrown handler".to_string()],
            ));
        }

        if let Some(line) = first_config_lookup_line(file, lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_parses_config_or_feature_flags_inline",
                Severity::Info,
                line,
                "Gin handler reads configuration or feature flags inline",
                vec!["boundary handlers are usually easier to test when configuration arrives through dependencies instead of direct lookups".to_string()],
            ));
        }

        if lines.iter().any(|line| transaction_start_line(&line.text)) {
            let line = lines
                .iter()
                .find(|line| transaction_start_line(&line.text))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "gin_handler_starts_transaction_inline",
                Severity::Warning,
                line,
                "Gin handler starts a transaction inline",
                vec!["transport handlers usually should not own transaction lifetime".to_string()],
            ));
        }

        if let Some(line) = authorization_business_logic_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_runs_authorization_business_rules_inline",
                Severity::Info,
                line,
                "Gin handler appears to mix authorization policy with request orchestration",
                vec!["permission matrix logic is often easier to reuse in dedicated policy or service code".to_string()],
            ));
        }

        if let Some(line) = action_switch_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_uses_action_param_switch_for_many_use_cases",
                Severity::Info,
                line,
                "Gin handler branches on action or mode parameters for multiple workflows",
                vec!["one endpoint function usually reads better when it serves one core use case".to_string()],
            ));
        }

        if mixes_html_json_and_file(go.gin_calls) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_mixes_html_json_and_file_responses",
                Severity::Info,
                function.fingerprint.start_line,
                "Gin handler mixes HTML, JSON, or file-style responses",
                vec!["combining several transport modes in one handler often hurts clarity".to_string()],
            ));
        }

        if let Some(line) = passes_gin_context_beyond_boundary(lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_context_passed_beyond_request_boundary",
                Severity::Warning,
                line,
                "Gin handler passes *gin.Context into deeper layers or goroutines",
                vec!["transport-specific request context is usually better translated into context.Context and typed values".to_string()],
            ));
        }

        if let Some(line) = global_singleton_reference_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_uses_global_singletons",
                Severity::Info,
                line,
                "Gin handler reaches for singleton-like global dependencies",
                vec!["dependency injection usually keeps handlers easier to test than global runtime state".to_string()],
            ));
        }

        if let Some(line) = retry_or_backoff_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_contains_retry_or_backoff_orchestration",
                Severity::Info,
                line,
                "Gin handler contains retry or backoff orchestration",
                vec!["retry policy is often easier to own in services or outbound client layers".to_string()],
            ));
        }

        if lines.iter().any(|line| line.text.contains(".Group(\"/")) {
            let line = lines
                .iter()
                .find(|line| line.text.contains(".Group(\"/"))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "route_groups_created_inside_handlers",
                Severity::Info,
                line,
                "handler creates route groups instead of only serving requests",
                vec!["route-group ownership is usually clearer in router setup code".to_string()],
            ));
        }

        if let Some(line) = route_param_merge_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "multiple_bind_sources_into_same_struct_without_precedence_contract",
                Severity::Info,
                line,
                "handler merges several request sources into one DTO inline",
                vec!["normalizing request precedence before service calls usually reduces ambiguity".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "query_path_and_body_merge_performed_in_handler",
                Severity::Info,
                line,
                "handler merges path, query, or body values inline",
                vec!["shared bind helpers or boundary mappers usually keep merged request inputs clearer".to_string()],
            ));
        }

        if let Some(line) = required_validation_line(lines)
            && (!go.db_query_calls.is_empty() || !go.gorm_query_chains.is_empty() || references_repository_directly(lines))
        {
            findings.push(function_finding(
                file,
                function,
                "business_validation_mixed_with_persistence_calls_in_handler",
                Severity::Info,
                line,
                "handler interleaves validation logic with persistence or repository work",
                vec!["handlers usually read more clearly when request validation finishes before persistence begins".to_string()],
            ));
        }

        if let Some(line) = upload_write_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "file_upload_validation_mixed_with_storage_write",
                Severity::Info,
                line,
                "handler validates upload input and writes storage in the same step",
                vec!["splitting upload validation from storage writes usually makes the boundary easier to test".to_string()],
            ));
        }

        if let Some(line) = route_param_drift_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "route_param_name_and_dto_field_name_drift_without_mapping_helper",
                Severity::Info,
                line,
                "handler manually remaps route parameter names into DTO fields",
                vec!["shared mappers usually reduce repeated renaming between route params and request contracts".to_string()],
            ));
        }

        if let Some(line) = error_string_switch_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "handler_switches_on_error_strings",
                Severity::Warning,
                line,
                "handler branches on err.Error() text",
                vec!["typed errors are usually more stable than string-matching on error messages".to_string()],
            ));
        }

        if let Some(line) = raw_db_error_response_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "raw_db_error_exposed_to_client",
                Severity::Warning,
                line,
                "handler appears to serialize raw error text to clients",
                vec!["raw SQL or ORM error messages often leak implementation details into public contracts".to_string()],
            ));
        }

        if let Some(line) = client_input_500_line(function, lines) {
            findings.push(function_finding(
                file,
                function,
                "client_input_error_mapped_to_internal_server_error",
                Severity::Warning,
                line,
                "handler appears to translate client input errors into an internal-server response",
                vec![
                    "request parsing or binding errors usually belong to 4xx response classes rather than 500".to_string(),
                ],
            ));
        }

        if lines.iter().any(|line| line.text.contains("\"code\"")) && !import_path_has_any_role(file, &["code", "codes"]) {
            let line = lines
                .iter()
                .find(|line| line.text.contains("\"code\""))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "transport_layer_uses_untyped_string_codes_without_catalog",
                Severity::Info,
                line,
                "transport layer builds string error codes inline",
                vec!["typed or cataloged error codes are usually easier to govern than free-form strings".to_string()],
            ));
        }

        if let Some(line) = success_payload_with_error_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "success_response_contains_error_field_or_mixed_contract",
                Severity::Info,
                line,
                "success payload mixes data and error fields",
                vec!["response contracts are usually clearer when success and error shapes remain distinct".to_string()],
            ));
        }

        if let Some(line) = health_handler_repo_line(function, lines) {
            findings.push(function_finding(
                file,
                function,
                "health_or_readiness_handlers_reach_into_business_repositories_directly",
                Severity::Info,
                line,
                "health or readiness handler queries business repositories directly",
                vec!["operational probes usually stay more stable when they depend on focused health checks instead of full business repositories".to_string()],
            ));
        }

        if let Some(line) = migration_or_seed_handler_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "migration_or_seed_logic_callable_from_request_handlers",
                Severity::Warning,
                line,
                "handler can trigger migration or seed logic",
                vec!["schema or seed workflows are usually safer when kept out of request handlers".to_string()],
            ));
        }

        if let Some(line) = api_example_literal_line(function) {
            findings.push(function_finding(
                file,
                function,
                "api_examples_embedded_in_handlers_instead_of_transport_docs_helpers",
                Severity::Info,
                line,
                "handler embeds example JSON payload text inline",
                vec!["transport docs helpers usually keep examples easier to reuse than hardcoded handler examples".to_string()],
            ));
        }
    }

    findings
}

fn service_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
    gorm_structs: &BTreeSet<String>,
    returns_text: &str,
    params_text: &str,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_service_file(file) {
        return findings;
    }
    let go = function.go_evidence();

    if signature_has_request_dto(params_text) {
        findings.push(function_finding(
            file,
            function,
            "service_depends_on_transport_request_type",
            Severity::Warning,
            function.fingerprint.start_line,
            "service method accepts request-shaped transport DTO input",
            vec!["service APIs usually stay clearer when transport DTOs are translated before crossing the boundary".to_string()],
        ));
    }

    if function.signature_text.contains("*gin.Context") {
        findings.push(function_finding(
            file,
            function,
            "service_method_accepts_gin_context",
            Severity::Warning,
            function.fingerprint.start_line,
            "service method accepts *gin.Context directly",
            vec!["service methods usually prefer context.Context plus typed inputs".to_string()],
        ));
    }

    if has_http_status_usage(file, lines) && returns_text.contains("int") {
        let line = first_http_status_line(file, lines).unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "service_method_returns_http_status",
            Severity::Warning,
            line,
            "service method appears to own HTTP status return semantics",
            vec!["transport status translation usually belongs at the boundary layer".to_string()],
        ));
    }

    if returns_text.contains("gin.H") || returns_text.contains("http.ResponseWriter") {
        findings.push(function_finding(
            file,
            function,
            "service_method_returns_transport_writer_or_gin_h",
            Severity::Warning,
            function.fingerprint.start_line,
            "service method returns transport-facing response types",
            vec!["services usually should return domain results or DTO-ready values, not transport writers".to_string()],
        ));
    }

    if returns_framework_builder(returns_text) {
        findings.push(function_finding(
            file,
            function,
            "service_method_returns_gorm_db_or_sql_rows",
            Severity::Warning,
            function.fingerprint.start_line,
            "service method leaks ORM or SQL primitives to callers",
            vec!["service callers usually should not need *gorm.DB or row handles".to_string()],
        ));
    }

    if params_text.contains("map[string]any") || params_text.contains("map[string]interface{}") {
        findings.push(function_finding(
            file,
            function,
            "service_method_accepts_map_string_any_input",
            Severity::Info,
            function.fingerprint.start_line,
            "service method accepts map[string]any input",
            vec!["typed inputs usually age better than open-ended map payloads".to_string()],
        ));
    }

    if returns_text.contains("map[string]any") || returns_text.contains("map[string]interface{}") {
        findings.push(function_finding(
            file,
            function,
            "service_method_returns_map_string_any_output",
            Severity::Info,
            function.fingerprint.start_line,
            "service method returns map[string]any output",
            vec!["typed results are usually easier to validate and document".to_string()],
        ));
    }

    if let Some(line) = first_http_or_abort_semantics_line(file, lines) {
        findings.push(function_finding(
            file,
            function,
            "service_method_imports_http_status_or_abort_semantics",
            Severity::Warning,
            line,
            "service method owns transport abort or HTTP status semantics",
            vec!["services usually should return typed outcomes while handlers translate them into HTTP responses".to_string()],
        ));
    }

    if let Some(query_call) = go.db_query_calls.iter().find(|call| call.query_text.is_some()) {
        findings.push(function_finding(
            file,
            function,
            "raw_sql_literal_in_service",
            Severity::Warning,
            query_call.line,
            "service method executes literal SQL directly",
            query_call
                .query_text
                .as_ref()
                .map(|query| vec![format!("literal query: {query}")])
                .unwrap_or_default(),
        ));
    }

    if let Some(chain) = go.gorm_query_chains.first() {
        findings.push(function_finding(
            file,
            function,
            "gorm_chain_built_in_service",
            Severity::Warning,
            chain.line,
            "service method owns GORM query construction directly",
            vec!["query-chain ownership often reads better inside repositories".to_string()],
        ));
    }

    if constructor_like(function) && constructor_instantiates_dependencies(lines) {
        let line = lines
            .iter()
            .find(|line| constructor_dependency_line(&line.text))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "service_constructor_instantiates_dependencies_internally",
            Severity::Info,
            line,
            "service constructor instantiates concrete dependencies internally",
            vec!["accepting dependencies explicitly usually gives clearer wiring and easier tests".to_string()],
        ));
    }

    if let Some(service_struct) = file
        .go_structs()
        .iter()
        .find(|s| s.name.ends_with("Service") && s.fields.len() >= 5)
    {
        findings.push(file_finding(
            file,
            "service_struct_has_excessive_dependency_count",
            Severity::Info,
            service_struct.line,
            format!("service struct {} has many dependencies", service_struct.name),
            vec![format!("{} fields were observed on the service struct", service_struct.fields.len())],
        ));
    }

    if let Some(line) = pagination_or_query_parsing_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "service_method_handles_pagination_or_query_parsing",
            Severity::Info,
            line,
            "service method appears to parse pagination or query string details",
            vec!["query and page normalization is often cleaner at the transport boundary".to_string()],
        ));
    }

    if let Some(line) = request_binding_or_header_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "service_method_handles_request_binding_or_header_extraction",
            Severity::Info,
            line,
            "service method extracts request-boundary values directly",
            vec!["headers, params, and forms usually belong at the transport boundary".to_string()],
        ));
    }

    if let Some(line) = action_switch_line(lines)
        && lines.iter().any(|line| {
            line.text.contains("create")
                || line.text.contains("update")
                || line.text.contains("delete")
        })
    {
        findings.push(function_finding(
            file,
            function,
            "service_method_combines_unrelated_write_paths_via_action_string",
            Severity::Info,
            line,
            "service method uses action or mode switching to implement several write paths",
            vec!["splitting use cases usually reads better than one action-driven service method".to_string()],
        ));
    }

    if let Some(line) = mutates_request_binding_in_place(params_text, lines) {
        findings.push(function_finding(
            file,
            function,
            "service_method_mutates_transport_dto_in_place",
            Severity::Info,
            line,
            "service method mutates a request-shaped DTO in place",
            vec!["mapping into a service-layer input struct is usually clearer than rewriting transport DTOs".to_string()],
        ));
    }

    if signature_has_request_dto(params_text)
        && (params_text.contains("models.") || gorm_structs.iter().any(|name| params_text.contains(name)))
    {
        findings.push(function_finding(
            file,
            function,
            "service_method_accepts_dto_and_persistence_model_together",
            Severity::Warning,
            function.fingerprint.start_line,
            "service method signature mixes request DTOs and persistence model types",
            vec!["transport and persistence concerns are usually easier to evolve when they stay separate".to_string()],
        ));
    }

    if is_generic_name(&function.fingerprint.name)
        || matches!(function.fingerprint.name.as_str(), "Handle" | "Execute" | "Process")
    {
        findings.push(function_finding(
            file,
            function,
            "generic_process_execute_handle_service_name_without_domain_noun",
            Severity::Info,
            function.fingerprint.start_line,
            "service method uses a very generic name without an obvious domain noun",
            vec!["use-case services are usually easier to read when method names describe the domain action".to_string()],
        ));
    }

    if returns_text.contains("string") && lines.iter().any(|line| line.text.contains("fmt.Sprintf(") || line.text.contains("fmt.Errorf(")) {
        let line = lines
            .iter()
            .find(|line| line.text.contains("fmt.Sprintf(") || line.text.contains("fmt.Errorf("))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "service_returns_preformatted_client_message",
            Severity::Info,
            line,
            "service method appears to return preformatted client-facing messages",
            vec!["typed domain errors usually travel better across layers than final client text".to_string()],
        ));
    }

    if let Some(line) = not_found_nil_nil_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "not_found_semantics_implemented_by_nil_nil_return",
            Severity::Info,
            line,
            "service method encodes a missing result as nil, nil",
            vec!["explicit result contracts usually communicate absence more clearly than nil, nil".to_string()],
        ));
    }

    if function.signature_text.contains("*gin.Context")
        && let Some(line) = first_http_or_abort_semantics_line(file, lines)
    {
        findings.push(function_finding(
            file,
            function,
            "service_depends_on_gin_abort_or_context_error_response",
            Severity::Warning,
            line,
            "service method depends on Gin abort or response behavior",
            vec!["transport response ownership usually belongs at the boundary layer".to_string()],
        ));
    }

    if constructor_like(function)
        && (params_text.contains("map[string]any") || params_text.contains("map[string]interface{}"))
    {
        findings.push(function_finding(
            file,
            function,
            "service_constructor_accepts_untyped_config_map",
            Severity::Info,
            function.fingerprint.start_line,
            "service constructor accepts an untyped config map",
            vec!["typed config structs usually make service wiring easier to validate than generic maps".to_string()],
        ));
    }

    if let Some(line) = cross_repo_write_without_tx_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "cross_repository_write_flow_without_shared_uow_boundary",
            Severity::Warning,
            line,
            "service coordinates several repository writes without an obvious transaction boundary",
            vec!["shared unit-of-work boundaries usually make multi-repository write flows easier to reason about".to_string()],
        ));
    }

    if let Some((line, evidence)) =
        service_write_passthrough_without_domain_validation_evidence(function, lines)
    {
        findings.push(function_finding(
            file,
            function,
            "service_write_passthrough_without_domain_validation",
            Severity::Info,
            line,
            "write-style service method mostly forwards to a repository without visible domain guard checks",
            evidence,
        ));
    }

    if lines.iter().any(|line| (line.text.contains("logger.") || line.text.contains("log.")) && line.text.contains("err")) {
        let line = lines
            .iter()
            .find(|line| (line.text.contains("logger.") || line.text.contains("log.")) && line.text.contains("err"))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "handler_and_service_both_log_same_error_chain",
            Severity::Info,
            line,
            "service path logs an error that is likely also logged at the boundary",
            vec!["logging ownership is usually clearer when one layer owns the final request-path error log".to_string()],
        ));
    }

    findings
}

fn service_write_passthrough_without_domain_validation_evidence(
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<(usize, Vec<String>)> {
    if function.fingerprint.line_count > 8 || function.fingerprint.call_count > 2 {
        return None;
    }

    let write_call = function.calls.iter().find(|call| {
        let receiver = call.receiver.as_deref().unwrap_or_default().to_ascii_lowercase();
        let name = call.name.to_ascii_lowercase();
        (receiver.contains("repo") || receiver.contains("repository") || receiver.contains("store"))
            && matches!(
                name.as_str(),
                "create" | "save" | "update" | "delete" | "insert" | "upsert" | "add" | "remove"
            )
    })?;

    if signature_params_text(&function.signature_text).trim().is_empty()
        || signature_params_text(&function.signature_text).trim() == "ctx context.Context"
    {
        return None;
    }

    if required_validation_line(lines).is_some()
        || lines.iter().any(|line| obvious_domain_validation_line(&line.text))
    {
        return None;
    }

    let meaningful_lines = lines
        .iter()
        .filter(|line| !line.text.is_empty() && line.text != "return" && !line.text.starts_with('}'))
        .count();
    if meaningful_lines > 4 {
        return None;
    }

    Some((
        write_call.line,
        vec![
            format!(
                "repository write call: {}.{} at line {}",
                write_call.receiver.as_deref().unwrap_or("repo"),
                write_call.name,
                write_call.line
            ),
            format!("service method line count: {}", function.fingerprint.line_count),
            "no visible validation branch, guard clause, or invariant check was observed before persistence".to_string(),
        ],
    ))
}

fn obvious_domain_validation_line(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("validate(")
        || lower.contains(".validate(")
        || lower.contains("ensure")
        || lower.contains("guard")
        || lower.contains("check")
        || lower.contains("assert")
        || lower.contains("normalize")
        || lower.contains("sanitize")
        || lower.contains("if ")
}
