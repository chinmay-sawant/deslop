fn handler_module_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    let handler_functions = file
        .functions
        .iter()
        .filter(|function| is_request_path_function(file, function))
        .collect::<Vec<_>>();
    let middleware_functions = file
        .functions
        .iter()
        .filter(|function| {
            let lower = function.fingerprint.name.to_ascii_lowercase();
            lower.contains("middleware") || lower.contains("auth") || lower.contains("recover")
        })
        .collect::<Vec<_>>();

    if handler_functions
        .iter()
        .filter(|function| handler_recovery_line(function).is_some())
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| handler_recovery_line(function))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "custom_recovery_logic_repeated_across_handlers",
            Severity::Info,
            line,
            "several handlers implement custom recover logic inline",
            vec!["panic-to-response translation is usually easier to keep consistent in middleware".to_string()],
        ));
    }

    if handler_functions
        .iter()
        .filter(|function| request_identity_extraction_line(&body_lines(function)).is_some())
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| request_identity_extraction_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "auth_or_tenant_extraction_duplicated_across_handlers",
            Severity::Info,
            line,
            "auth or tenant extraction is duplicated across handlers",
            vec!["shared boundary helpers or middleware usually keep request identity extraction more consistent".to_string()],
        ));
        findings.push(file_finding(
            file,
            "request_context_value_extraction_duplicated_across_handlers",
            Severity::Info,
            line,
            "request context value extraction is duplicated across handlers",
            vec!["shared helpers often reduce drift in user, tenant, and locale extraction logic".to_string()],
        ));
    }

    if !is_middleware_file(file)
        && handler_functions
            .iter()
            .filter(|function| request_id_generation_line(&body_lines(function)).is_some())
            .count()
            >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| request_id_generation_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "request_id_generation_duplicated_outside_middleware",
            Severity::Info,
            line,
            "request-id generation is duplicated outside middleware",
            vec!["request correlation IDs are usually easier to keep consistent in middleware".to_string()],
        ));
    }

    if handler_functions
        .iter()
        .filter(|function| pagination_binding_line(&body_lines(function)).is_some())
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| pagination_binding_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "pagination_binding_duplicated_outside_boundary_helper",
            Severity::Info,
            line,
            "pagination binding logic is duplicated across handlers",
            vec!["shared boundary helpers usually reduce drift in paging defaults and parsing".to_string()],
        ));
    }

    if middleware_functions.iter().any(|function| {
        !body_lines(function).is_empty()
            && function
                .go_evidence()
                .gin_calls
                .iter()
                .any(|call| is_body_bind_operation(&call.operation))
    }) && handler_functions.iter().any(|function| {
        function
            .go_evidence()
            .gin_calls
            .iter()
            .any(|call| is_body_bind_operation(&call.operation))
    }) {
        let line = middleware_functions
            .iter()
            .find_map(|function| body_lines(function).first().map(|line| line.line))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "body_binding_done_in_middleware_and_handler",
            Severity::Info,
            line,
            "request body binding happens in both middleware and handlers",
            vec!["ownership of request parsing is usually clearer when one boundary layer owns body binding".to_string()],
        ));
    }

    if handler_functions
        .iter()
        .filter(|function| response_envelope_line(&body_lines(function)).is_some())
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| response_envelope_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "response_envelope_shaping_duplicated_across_handlers",
            Severity::Info,
            line,
            "response envelope shaping is duplicated across handlers",
            vec!["shared render helpers often keep transport envelopes more consistent".to_string()],
        ));
    }

    if handler_functions
        .iter()
        .filter(|function| required_validation_line(&body_lines(function)).is_some())
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| required_validation_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "validation_logic_duplicated_across_handlers",
            Severity::Info,
            line,
            "manual validation checks are duplicated across handlers",
            vec!["shared validators or DTO-based validation usually reduce repeated required-field checks".to_string()],
        ));
    }

    let validation_shapes = handler_functions
        .iter()
        .filter_map(|function| validation_error_shape(&body_lines(function)))
        .collect::<BTreeSet<_>>();
    if validation_shapes.len() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| {
                validation_error_shape(&body_lines(function))
                    .and(Some(function.fingerprint.start_line))
            })
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "validation_error_response_shape_inconsistent",
            Severity::Info,
            line,
            "validation failures serialize several different payload shapes in one module",
            vec!["consistent validation error envelopes are usually easier for clients to consume".to_string()],
        ));
    }

    if handler_functions
        .iter()
        .filter(|function| default_injection_line(&body_lines(function)).is_some())
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| default_injection_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "default_value_injection_scattered_across_handlers",
            Severity::Info,
            line,
            "default value injection is duplicated across handlers",
            vec!["shared contracts or bind helpers usually keep defaults more consistent".to_string()],
        ));
    }

    if handler_functions
        .iter()
        .filter(|function| path_param_parse_line(&body_lines(function)).is_some())
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| path_param_parse_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "path_param_parsing_duplicated_across_handlers",
            Severity::Info,
            line,
            "path parameter parsing is duplicated across handlers",
            vec!["shared param helpers usually reduce repeated strconv or UUID parsing boilerplate".to_string()],
        ));
    }

    if handler_functions
        .iter()
        .filter(|function| pagination_bounds_line(&body_lines(function)).is_some())
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| pagination_bounds_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "pagination_validation_missing_shared_bound_helper",
            Severity::Info,
            line,
            "page or page-size bound checks are repeated across handlers",
            vec!["shared pagination helpers usually keep request bounds consistent".to_string()],
        ));
    }

    if handler_functions
        .iter()
        .filter(|function| sort_whitelist_line(&body_lines(function)).is_some())
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| sort_whitelist_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "sort_or_filter_whitelist_logic_duplicated_across_handlers",
            Severity::Info,
            line,
            "sort or filter whitelist logic is duplicated across handlers",
            vec!["shared contract helpers often reduce drift in allowed client fields".to_string()],
        ));
    }

    let error_shapes = handler_functions
        .iter()
        .filter_map(|function| handler_error_shape_key(&body_lines(function)))
        .collect::<BTreeSet<_>>();
    if error_shapes.len() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| {
                handler_error_shape_key(&body_lines(function))
                    .and(Some(function.fingerprint.start_line))
            })
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "multiple_error_envelope_shapes_same_api_module",
            Severity::Info,
            line,
            "handlers in the same module use several error envelope shapes",
            vec!["consistent error contracts are usually easier for clients to depend on".to_string()],
        ));
    }

    if handler_functions
        .iter()
        .filter(|function| error_mapping_line(file, &body_lines(function)).is_some())
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| error_mapping_line(file, &body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "inline_error_to_status_mapping_duplicated",
            Severity::Info,
            line,
            "error-to-status translation is duplicated across handlers",
            vec!["centralizing transport error translation usually reduces drift".to_string()],
        ));
    }

    if let Some((line, code)) = is_transport_file(file)
        .then(|| {
            repeated_named_literals(file, |literal| {
                literal.name.to_ascii_lowercase().contains("code")
                    && literal
                        .value
                        .chars()
                        .all(|ch: char| ch.is_ascii_uppercase() || ch == '_' || ch == '-')
            })
        })
        .flatten()
    {
        findings.push(file_finding(
            file,
            "error_code_literals_duplicated_across_handlers",
            Severity::Info,
            line,
            format!("error code literal {code} is repeated across transport helpers"),
            vec!["shared typed error code catalogs usually make API errors easier to govern".to_string()],
        ));
    }

    if is_transport_file(file)
        && file
            .functions
            .iter()
            .flat_map(body_lines)
            .filter(|line| line.text.contains("\"code\"") && line.text.contains("USER_"))
            .count()
            >= 2
    {
        let line = file
            .functions
            .iter()
            .flat_map(body_lines)
            .find(|line| line.text.contains("\"code\"") && line.text.contains("USER_"))
            .map(|line| line.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "error_code_literals_duplicated_across_handlers",
            Severity::Info,
            line,
            "string error codes are duplicated across transport handlers",
            vec!["shared typed error code catalogs usually make API errors easier to govern".to_string()],
        ));
    }

    if let Some((line, value)) = (!is_repository_file(file))
        .then(|| {
            repeated_named_literals(file, |literal| {
                let value = literal.value.as_str();
                table_or_column_literal(value) && !(value.ends_with("_id") || value.ends_with("_at"))
            })
        })
        .flatten()
    {
        findings.push(file_finding(
            file,
            "table_name_literals_duplicated_outside_repository",
            Severity::Info,
            line,
            format!("SQL-oriented literal {value} is duplicated outside repository packages"),
            vec!["query-shape literals usually stay easier to govern when owned by repository code".to_string()],
        ));
    }

    if let Some((line, value)) = (!is_repository_file(file))
        .then(|| {
            repeated_named_literals(file, |literal| {
                let value = literal.value.as_str();
                table_or_column_literal(value) && (value.ends_with("_id") || value.ends_with("_at"))
            })
        })
        .flatten()
    {
        findings.push(file_finding(
            file,
            "column_name_literals_duplicated_outside_repository",
            Severity::Info,
            line,
            format!("SQL column literal {value} is duplicated outside repository packages"),
            vec!["column literals usually stay easier to govern when owned by repository code".to_string()],
        ));
    }

    if handler_functions
        .iter()
        .filter(|function| {
            let lines = body_lines(function);
            let has_mapping = lines
                .iter()
                .any(|line| line.text.contains("Response{") || line.text.contains("gin.H{"));
            let has_model_field = lines
                .iter()
                .any(|line| line.text.contains(".ID") || line.text.contains(".Name") || line.text.contains(".Email"));
            has_mapping && has_model_field
        })
        .count()
        >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| {
                let lines = body_lines(function);
                let has_mapping = lines
                    .iter()
                    .any(|line| line.text.contains("Response{") || line.text.contains("gin.H{"));
                let has_model_field = lines
                    .iter()
                    .any(|line| line.text.contains(".ID") || line.text.contains(".Name") || line.text.contains(".Email"));
                if has_mapping && has_model_field {
                    Some(function.fingerprint.start_line)
                } else {
                    None
                }
            })
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "mapping_logic_duplicated_across_handlers",
            Severity::Info,
            line,
            "similar transport mapping logic is duplicated across handlers",
            vec!["shared mappers or render helpers usually reduce repeated field-by-field response shaping".to_string()],
        ));
    }

    if is_transport_file(file)
        && file
            .go_structs()
            .iter()
            .any(|go_struct| go_struct.name.ends_with("Error"))
    {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| go_struct.name.ends_with("Error"))
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "domain_errors_declared_in_handler_package",
            Severity::Info,
            line,
            "handler package declares domain-like error types",
            vec!["reusable business errors are usually clearer in domain or service-facing packages".to_string()],
        ));
    }

    if middleware_functions
        .iter()
        .any(|function| error_mapping_line(file, &body_lines(function)).is_some())
        && handler_functions
            .iter()
            .any(|function| error_mapping_line(file, &body_lines(function)).is_some())
    {
        let line = middleware_functions
            .iter()
            .find_map(|function| error_mapping_line(file, &body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "middleware_and_handler_translate_same_error_domain",
            Severity::Info,
            line,
            "middleware and handlers both translate the same error family",
            vec!["keeping transport error translation in one layer usually reduces double-mapping drift".to_string()],
        ));
    }

    let mut error_statuses = BTreeMap::<String, BTreeSet<String>>::new();
    for function in &handler_functions {
        let lines = body_lines(function);
        let statuses = lines
            .iter()
            .filter_map(|line| extract_status_constant(&line.text))
            .collect::<BTreeSet<_>>();
        for error_name in lines
            .iter()
            .filter_map(|line| extract_domain_error_name(&line.text))
        {
            error_statuses
                .entry(error_name)
                .or_default()
                .extend(statuses.clone());
        }
    }
    if let Some((error_name, statuses)) =
        error_statuses.iter().find(|(_, statuses)| statuses.len() >= 2)
    {
        let line = handler_functions
            .first()
            .map(|function| function.fingerprint.start_line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "same_domain_error_mapped_to_multiple_statuses",
            Severity::Info,
            line,
            format!("domain error {error_name} is translated to multiple statuses"),
            vec![format!(
                "observed statuses: {}",
                statuses.iter().cloned().collect::<Vec<_>>().join(", ")
            )],
        ));
    }

    let trace_spans = handler_functions
        .iter()
        .flat_map(|function| body_lines(function))
        .filter_map(|line| tracing_span_name_literal(&line.text))
        .collect::<Vec<_>>();
    if has_duplicate_string(&trace_spans) {
        let line = handler_functions
            .first()
            .map(|function| function.fingerprint.start_line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "tracing_span_names_duplicated_as_raw_strings",
            Severity::Info,
            line,
            "raw tracing span names are duplicated across handlers",
            vec!["shared span helpers or constants usually reduce string drift in tracing".to_string()],
        ));
    }

    let logging_keys = handler_functions
        .iter()
        .flat_map(|function| body_lines(function))
        .filter_map(|line| request_logging_field_key(&line.text))
        .collect::<BTreeSet<_>>();
    if logging_keys.contains("request_id") && logging_keys.contains("requestId") {
        let line = handler_functions
            .first()
            .map(|function| function.fingerprint.start_line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "request_logging_fields_assembled_differently_across_handlers",
            Severity::Info,
            line,
            "request log field names differ across handlers",
            vec!["shared request logging helpers usually reduce field-name drift across an API module".to_string()],
        ));
    }

    if logging_keys.contains("user_id") && logging_keys.contains("userId") {
        let line = handler_functions
            .first()
            .map(|function| function.fingerprint.start_line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "domain_identifiers_logged_under_inconsistent_field_keys",
            Severity::Info,
            line,
            "same domain identifier is logged under inconsistent key names",
            vec!["stable logging keys usually make operational queries easier than mixed snake_case and camelCase identifiers".to_string()],
        ));
    }

    findings
}
