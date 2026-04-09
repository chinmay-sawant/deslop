fn repository_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
    returns_text: &str,
    params_text: &str,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_repository_file(file) {
        return findings;
    }
    let go = function.go_evidence();

    if constructor_like(function)
        && (function.signature_text.contains("*gin.Engine")
            || function.signature_text.contains("gin.IRouter")
            || function.signature_text.contains("gin.RouterGroup"))
    {
        findings.push(function_finding(
            file,
            function,
            "repository_constructor_accepts_gin_engine_or_router",
            Severity::Warning,
            function.fingerprint.start_line,
            "repository constructor accepts Gin router types",
            vec!["repositories usually should not depend on transport router objects".to_string()],
        ));
    }

    if function.signature_text.contains("*gin.Context") {
        findings.push(function_finding(
            file,
            function,
            "repository_accepts_gin_context",
            Severity::Warning,
            function.fingerprint.start_line,
            "repository method accepts *gin.Context directly",
            vec!["repositories usually should accept context.Context rather than transport-specific context objects".to_string()],
        ));
    }

    if signature_has_request_dto(params_text) {
        findings.push(function_finding(
            file,
            function,
            "repository_accepts_http_request_dto",
            Severity::Warning,
            function.fingerprint.start_line,
            "repository method accepts request-shaped DTO input",
            vec!["repositories usually should receive domain values or persistence filters instead of HTTP-facing request contracts".to_string()],
        ));
    }

    if returns_transport_dto(returns_text) {
        findings.push(function_finding(
            file,
            function,
            "repository_returns_transport_dto",
            Severity::Warning,
            function.fingerprint.start_line,
            "repository method returns transport-facing DTO types",
            vec!["repository APIs usually should not know about response envelopes or view models".to_string()],
        ));
    }

    if returns_framework_builder(returns_text) {
        findings.push(function_finding(
            file,
            function,
            "repository_returns_framework_builder_to_upper_layer",
            Severity::Warning,
            function.fingerprint.start_line,
            "repository method returns framework query-builder or row-handle primitives",
            vec!["upper layers usually should not finish repositories by mutating leaked driver or ORM handles".to_string()],
        ));
    }

    if returns_text.contains("*gorm.DB") {
        findings.push(function_finding(
            file,
            function,
            "repository_returns_gorm_query_builder",
            Severity::Warning,
            function.fingerprint.start_line,
            "repository method returns *gorm.DB for callers to finish",
            vec!["repository methods usually read more clearly when they return results instead of partially built chains".to_string()],
        ));
    }

    if has_http_status_usage(file, lines)
        && returns_text.contains("int")
        && returns_text.contains("error")
    {
        let line = first_http_status_line(file, lines).unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "repository_returns_http_status_errors",
            Severity::Warning,
            line,
            "repository method appears to translate persistence failures into HTTP status returns",
            vec!["HTTP status mapping usually belongs in handlers or transport adapters, not repositories".to_string()],
        ));
    }

    if let Some((line, evidence)) =
        repository_single_record_write_without_rows_affected_evidence(function, lines)
    {
        findings.push(function_finding(
            file,
            function,
            "repository_single_record_write_without_rows_affected_check",
            Severity::Warning,
            line,
            "repository write path looks single-record-oriented but never inspects RowsAffected",
            evidence,
        ));
    }

    if signature_mentions_transaction(&function.signature_text)
        && lines.iter().any(|line| line.text.contains("tx == nil"))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains("tx == nil"))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "optional_tx_nil_parameter_on_repository_api",
            Severity::Info,
            line,
            "repository method accepts a transaction parameter that is treated as optional",
            vec!["unit-of-work boundaries are usually easier to reason about than nil-able transaction parameters".to_string()],
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
            "repository_begins_transaction_without_uow_or_callback",
            Severity::Info,
            line,
            "repository method begins its own transaction inline",
            vec!["repositories usually compose better when transaction ownership is explicit at a higher layer or through unit-of-work helpers".to_string()],
        ));
    }

    if !go.db_query_calls.is_empty() && !go.gorm_query_chains.is_empty() {
        let line = go.db_query_calls[0].line.min(go.gorm_query_chains[0].line);
        findings.push(function_finding(
            file,
            function,
            "repository_mixes_raw_sql_and_gorm_same_method_without_adapter_boundary",
            Severity::Info,
            line,
            "repository method mixes raw SQL calls and GORM chains in one path",
            vec!["explicit adapter boundaries are usually clearer than one method that spans several persistence styles".to_string()],
        ));
    }

    if returns_text.contains("sql.Null") {
        findings.push(function_finding(
            file,
            function,
            "sql_null_types_escape_repository_boundary",
            Severity::Info,
            function.fingerprint.start_line,
            "repository return type exposes sql.Null* wrappers",
            vec!["boundary-friendly DTO or domain types usually communicate optional values more clearly".to_string()],
        ));
    }

    if let Some((line, evidence)) = db_query_argument_erased_to_any_evidence(function, lines) {
        findings.push(function_finding(
            file,
            function,
            "db_query_argument_erased_to_any",
            Severity::Info,
            line,
            "repository erases a concrete DB argument to any immediately before query execution",
            evidence,
        ));
    }

    if returns_text.contains("*gorm.DB")
        && (!go.gorm_query_chains.is_empty()
            || lines.iter().any(|line| line.text.contains(".Where(") || line.text.contains(".Scopes(")))
    {
        findings.push(function_finding(
            file,
            function,
            "repository_method_returns_partially_built_scopes_for_caller_chaining",
            Severity::Info,
            function.fingerprint.start_line,
            "repository method returns a partially built GORM chain",
            vec!["callers usually should not need to finish repository-owned scopes".to_string()],
        ));
    }

    let where_templates = go
        .gorm_query_chains
        .iter()
        .flat_map(|chain| chain.steps.iter())
        .filter(|step| step.method_name == "Where")
        .filter_map(|step| step.first_string_arg.clone())
        .collect::<Vec<_>>();
    if has_duplicate_string(&where_templates) {
        let line = go
            .gorm_query_chains
            .iter()
            .flat_map(|chain| chain.steps.iter())
            .find(|step| step.method_name == "Where" && step.first_string_arg.is_some())
            .map(|step| step.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "where_clause_templates_duplicated_across_repositories",
            Severity::Info,
            line,
            "repository repeats the same WHERE template in several places",
            vec!["shared query helpers or scopes usually reduce duplication in filter templates".to_string()],
        ));
    }
    if !has_duplicate_string(&where_templates)
        && lines
            .iter()
            .map(|line| line.text.match_indices("Where(\"status = ?\"").count())
            .sum::<usize>()
            >= 2
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains("Where(\"status = ?\""))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "where_clause_templates_duplicated_across_repositories",
            Severity::Info,
            line,
            "repository repeats the same WHERE template in several places",
            vec!["shared query helpers or scopes usually reduce duplication in filter templates".to_string()],
        ));
    }

    if let Some(line) = base_repository_reflection_line(function, lines) {
        findings.push(function_finding(
            file,
            function,
            "generic_base_repository_with_reflection_dispatch",
            Severity::Info,
            line,
            "generic repository helper depends on reflection-driven dispatch",
            vec!["bounded repository APIs are usually easier to reason about than reflection-heavy base repositories".to_string()],
        ));
    }

    findings
}

fn repository_single_record_write_without_rows_affected_evidence(
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<(usize, Vec<String>)> {
    if !single_record_write_name(&function.fingerprint.name)
        || lines.iter().any(|line| line.text.contains("RowsAffected"))
    {
        return None;
    }

    let go = function.go_evidence();
    if let Some(call) = go.db_query_calls.iter().find(|call| {
        matches!(call.method_name.as_str(), "Exec" | "ExecContext")
            && call
                .query_text
                .as_deref()
                .is_some_and(sql_update_or_delete_query)
    }) {
        let query = call
            .query_text
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .to_string();
        return Some((
            call.line,
            vec![
                format!(
                    "single-record write-like repository method name: {}",
                    function.fingerprint.name
                ),
                format!("SQL write executed at line {}: {}", call.line, query),
                "no RowsAffected check was observed in the repository method".to_string(),
            ],
        ));
    }

    if let Some(chain) = go.gorm_query_chains.iter().find(|chain| {
        matches!(chain.terminal_method.as_str(), "Delete" | "Update" | "Updates")
    }) {
        return Some((
            chain.line,
            vec![
                format!(
                    "single-record write-like repository method name: {}",
                    function.fingerprint.name
                ),
                format!(
                    "GORM write chain ends with {} at line {}",
                    chain.terminal_method, chain.line
                ),
                "no RowsAffected check was observed in the repository method".to_string(),
            ],
        ));
    }

    None
}

fn single_record_write_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let write_signal = ["update", "delete", "remove", "archive", "deactivate", "restore"]
        .iter()
        .any(|needle| lower.contains(needle));
    let bulk_signal = ["all", "many", "batch", "bulk"]
        .iter()
        .any(|needle| lower.contains(needle));

    write_signal && !bulk_signal
}

fn sql_update_or_delete_query(query: &str) -> bool {
    let upper = query.trim_start().to_ascii_uppercase();
    upper.starts_with("UPDATE ") || upper.starts_with("DELETE ")
}

fn db_query_argument_erased_to_any_evidence(
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<(usize, Vec<String>)> {
    let go = function.go_evidence();
    if go.db_query_calls.is_empty() && go.gorm_query_chains.is_empty() {
        return None;
    }

    let mut bindings = BTreeMap::<String, (usize, String)>::new();
    for line in lines {
        if let Some((name, decl_text)) = any_binding_declaration(&line.text) {
            if is_deliberate_heterogeneous_binding(name, &decl_text) {
                continue;
            }
            bindings.insert(name.to_string(), (line.line, decl_text));
        }
    }

    for (name, (decl_line, decl_text)) in bindings {
        let assignment = lines.iter().find_map(|line| {
            any_binding_assignment(&line.text, &name).and_then(|rhs| {
                if rhs_looks_like_erased_db_argument(&rhs) {
                    Some((line.line, rhs))
                } else {
                    None
                }
            })
        });
        let Some((assignment_line, rhs)) = assignment else {
            continue;
        };

        if let Some(call_line) = lines
            .iter()
            .find(|line| line.line >= assignment_line && db_call_line_mentions_symbol(&line.text, &name))
            .map(|line| line.line)
            .or_else(|| {
                go.gorm_query_chains
                    .iter()
                    .flat_map(|chain| chain.steps.iter())
                    .find(|step| {
                        matches!(step.method_name.as_str(), "Raw" | "Create" | "Updates")
                            && step.argument_texts.iter().any(|arg| arg.trim() == name)
                    })
                    .map(|step| step.line)
            })
        {
            return Some((
                call_line,
                vec![
                    format!("{name} declared as weakly typed value at line {decl_line}: {decl_text}"),
                    format!("{name} assigned from concrete value at line {assignment_line}: {rhs}"),
                    format!("{name} then passed to DB execution at line {call_line}"),
                ],
            ));
        }
    }

    None
}

fn any_binding_declaration(text: &str) -> Option<(&str, String)> {
    let compact = text.trim();

    if let Some(rest) = compact.strip_prefix("var ") {
        if let Some((left, right)) = rest.split_once('=') {
            let mut parts = left.split_whitespace();
            let name = parts.next()?.trim();
            let type_text = parts.next()?.trim();
            if matches!(type_text, "any" | "interface{}") && is_identifier_name(name) {
                return Some((name, right.trim().to_string()));
            }
        }

        let mut parts = rest.split_whitespace();
        let name = parts.next()?.trim();
        let type_text = parts.next()?.trim();
        if matches!(type_text, "any" | "interface{}") && is_identifier_name(name) {
            return Some((name, compact.to_string()));
        }
    }

    let (left, right) = split_assignment(compact)?;
    let left = left.trim();
    let right = right.trim();
    if is_identifier_name(left) && right.starts_with("any(") {
        return Some((left, right.to_string()));
    }

    None
}

fn any_binding_assignment(text: &str, name: &str) -> Option<String> {
    let compact = text.trim();

    if let Some(rest) = compact.strip_prefix(&format!("var {name} "))
        && let Some((_, right)) = rest.split_once('=')
    {
        return Some(right.trim().to_string());
    }

    let (left, right) = split_assignment(compact)?;
    if left.trim() != name {
        return None;
    }

    Some(
        right.trim()
            .trim_start_matches("any(")
            .trim_end_matches(')')
            .trim()
            .to_string(),
    )
}

fn rhs_looks_like_erased_db_argument(rhs: &str) -> bool {
    if rhs.is_empty()
        || rhs == "nil"
        || rhs.contains("[]any")
        || rhs.contains("[]interface{}")
        || rhs.contains("map[")
        || rhs.contains("append(")
    {
        return false;
    }

    rhs.contains('.')
        || rhs.starts_with('*')
        || rhs.contains("sql.Null")
        || rhs.contains("Valid")
        || rhs.contains("String")
        || rhs.contains("Int")
        || rhs.contains("Bool")
}

fn is_deliberate_heterogeneous_binding(name: &str, decl_text: &str) -> bool {
    matches!(name, "args" | "params" | "values" | "bindings")
        || decl_text.contains("[]any")
        || decl_text.contains("[]interface{}")
}

fn db_call_line_mentions_symbol(text: &str, name: &str) -> bool {
    let db_marker = [
        ".Query(",
        ".QueryContext(",
        ".QueryRow(",
        ".QueryRowContext(",
        ".Exec(",
        ".ExecContext(",
        ".Get(",
        ".Select(",
        ".Raw(",
        ".Create(",
        ".Updates(",
    ]
    .iter()
    .any(|marker| text.contains(marker));

    db_marker
        && (text.contains(&format!(", {name}"))
            || text.contains(&format!(",{name}"))
            || text.contains(&format!("({name}"))
            || text.contains(&format!("{name},"))
            || text.ends_with(name))
}

fn split_assignment(text: &str) -> Option<(&str, &str)> {
    if let Some((left, right)) = text.split_once(":=") {
        return Some((left, right));
    }

    if text.contains("==") || text.contains("!=") || text.contains("<=") || text.contains(">=") {
        return None;
    }

    text.split_once(" = ")
        .or_else(|| text.split_once('='))
        .filter(|(left, _)| !left.trim_start().starts_with("if "))
}

fn is_identifier_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && text
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
}

fn middleware_and_bootstrap_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
    gorm_structs: &BTreeSet<String>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let go = function.go_evidence();

    if !is_repository_file(file)
        && lines.iter().any(|line| line.text.contains(".Scan(") && line.text.to_ascii_lowercase().contains("rows"))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains(".Scan(") && line.text.to_ascii_lowercase().contains("rows"))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "sql_rows_scan_logic_outside_repository",
            Severity::Info,
            line,
            "SQL rows scanning happens outside repository code",
            vec!["row scanning is usually clearer when kept inside persistence adapters".to_string()],
        ));
    }

    if is_middleware_file(file) {
        if references_repository_directly(lines) {
            findings.push(function_finding(
                file,
                function,
                "middleware_calls_repository_directly",
                Severity::Info,
                function.fingerprint.start_line,
                "middleware reaches repository-like dependencies directly",
                vec!["cross-cutting middleware often stays simpler when it delegates business queries through dedicated services".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "middleware_contains_business_orchestration",
                Severity::Info,
                function.fingerprint.start_line,
                "middleware appears to own business orchestration",
                vec!["middleware is usually cleaner when limited to cross-cutting concerns".to_string()],
            ));
        }

        if let Some(line) = lines.iter().find(|line| transaction_start_line(&line.text)).map(|line| line.line) {
            findings.push(function_finding(
                file,
                function,
                "middleware_starts_db_transaction",
                Severity::Warning,
                line,
                "middleware starts a database transaction",
                vec!["request-wide transaction ownership in middleware can hide expensive coupling".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "middleware_opens_transaction",
                Severity::Warning,
                line,
                "middleware opens a transaction for downstream business logic",
                vec!["transaction lifetime is usually clearer in services or unit-of-work helpers".to_string()],
            ));
        }

        if go.gin_calls.iter().any(|call| matches!(call.operation.as_str(), "json" | "pure_json")) {
            let line = go
                .gin_calls
                .iter()
                .find(|call| matches!(call.operation.as_str(), "json" | "pure_json"))
                .map(|call| call.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "middleware_writes_business_response_payloads",
                Severity::Info,
                line,
                "middleware writes response payloads directly",
                vec!["middleware is usually easier to reason about when handlers own success-response shaping".to_string()],
            ));
        }

        if lines.iter().any(|line| {
            line.text.contains(".Create(")
                || line.text.contains(".Save(")
                || line.text.contains(".Updates(")
                || line.text.contains(".Delete(")
        }) {
            let line = lines
                .iter()
                .find(|line| {
                    line.text.contains(".Create(")
                        || line.text.contains(".Save(")
                        || line.text.contains(".Updates(")
                        || line.text.contains(".Delete(")
                })
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "middleware_mutates_domain_model_state",
                Severity::Warning,
                line,
                "middleware mutates persistence or domain state directly",
                vec!["cross-cutting middleware is usually safer when it avoids business writes".to_string()],
            ));
        }

        if let Some(line) = global_singleton_reference_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "middleware_uses_global_logger_or_config_singleton",
                Severity::Info,
                line,
                "middleware depends on global logger or config state",
                vec!["middleware is usually easier to test when logger and config are injected".to_string()],
            ));
        }
    }

    if constructor_like(function)
        && let Some(line) = first_env_lookup_line(file, lines)
    {
        findings.push(function_finding(
            file,
            function,
            "constructor_reads_env_directly",
            Severity::Info,
            line,
            "constructor reads environment or config directly",
            vec!["constructors are easier to test when normalized config is injected instead of looked up inline".to_string()],
        ));
    }

    if is_router_file(file)
        && (function.fingerprint.name.contains("Route")
            || function.fingerprint.name.contains("Router")
            || function.fingerprint.name.contains("Register"))
    {
        if let Some(line) = lines.iter().find(|line| migration_line(&line.text)).map(|line| line.line)
        {
            findings.push(function_finding(
                file,
                function,
                "router_setup_runs_migrations",
                Severity::Warning,
                line,
                "router setup performs migration or schema work",
                vec!["route wiring is usually easier to reason about when bootstrap, migrations, and transport registration stay separate".to_string()],
            ));
        }

        let inline_handler_count = lines.iter().filter(|line| line.text.contains("func(c *gin.Context)") || line.text.contains("func(c * gin.Context)")).count();
        if inline_handler_count >= 2 {
            let line = lines
                .iter()
                .find(|line| line.text.contains("func(c *gin.Context)") || line.text.contains("func(c * gin.Context)"))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "gin_route_registration_anonymous_function_overuse",
                Severity::Info,
                line,
                "route registration uses several inline anonymous Gin handlers",
                vec!["named handlers or handler methods are usually easier to test and reuse".to_string()],
            ));
        }

        if let Some((line, path)) = repeated_route_path(lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_route_paths_repeated_as_raw_strings",
                Severity::Info,
                line,
                format!("route path {path} is repeated as a raw string"),
                vec!["centralizing route path segments usually reduces drift".to_string()],
            ));
        }

        if let Some(line) = route_registration_contains_business_logic_line(file, function, lines)
        {
            findings.push(function_finding(
                file,
                function,
                "route_registration_contains_business_logic",
                Severity::Info,
                line,
                "route registration function also performs business or persistence work",
                vec!["router setup usually reads better when it only wires handlers and middleware".to_string()],
            ));
        }

        if let Some(line) = router_constructs_dependencies_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "router_constructor_builds_concrete_dependencies",
                Severity::Info,
                line,
                "router setup constructs concrete dependencies inline",
                vec!["router constructors are usually simpler when dependencies are assembled earlier in bootstrap".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "bootstrap_builds_clients_inside_route_registration",
                Severity::Info,
                line,
                "route registration hides client or repository construction",
                vec!["building clients and repositories inside route registration makes startup ownership harder to see".to_string()],
            ));
        }

        if let Some(line) = background_worker_registration_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "background_worker_started_from_http_handler_registration",
                Severity::Info,
                line,
                "route registration starts background work as a side effect",
                vec!["HTTP route setup and worker startup are usually easier to manage as separate bootstrap concerns".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "background_jobs_registered_from_gin_packages_instead_of_bootstrap",
                Severity::Info,
                line,
                "Gin package registers background jobs directly",
                vec!["schedulers and workers usually belong in application bootstrap, not transport packages".to_string()],
            ));
        }

        if let Some(line) = admin_debug_route_line(lines) {
            findings.push(function_finding(
                file,
                function,
                "admin_or_debug_endpoint_registration_mixed_into_public_router_setup",
                Severity::Info,
                line,
                "admin or debug routes are registered in the same public router setup",
                vec!["operational endpoints are easier to govern when clearly separated from public routing".to_string()],
            ));
        }

        let constructor_names = lines
            .iter()
            .filter_map(|line| extract_constructor_name(&line.text))
            .collect::<Vec<_>>();
        if has_duplicate_string(&constructor_names) {
            let line = lines.first().map(|line| line.line).unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "same_dependency_wired_in_multiple_bootstrap_locations",
                Severity::Info,
                line,
                "same dependency appears to be constructed repeatedly in one bootstrap path",
                vec!["shared bootstrap composition usually avoids repeated construction of the same dependency".to_string()],
            ));
        }

        if lines.iter().any(|line| migration_line(&line.text))
            && lines.iter().any(|line| line.text.contains(".Run(") || line.text.contains(".ListenAndServe("))
        {
            let line = lines
                .iter()
                .find(|line| migration_line(&line.text))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "migration_runner_and_api_server_bootstrap_coupled",
                Severity::Info,
                line,
                "bootstrap path runs migrations and serves traffic together",
                vec!["separating migration and serve lifecycles often makes deployments safer".to_string()],
            ));
        }
    }

    if function.fingerprint.name == "init"
        && let Some(line) = lines
            .iter()
            .find(|line| init_registers_dependencies_or_routes(&line.text))
            .map(|line| line.line)
    {
        findings.push(function_finding(
            file,
            function,
            "init_registers_routes_or_dependencies",
            Severity::Warning,
            line,
            "init() registers routes or constructs runtime dependencies",
            vec!["side-effectful startup registration is usually clearer in explicit bootstrap code".to_string()],
        ));
    }

    if is_gorm_hook(file, function, gorm_structs) {
        if let Some(line) = first_external_io_line(file, lines) {
            findings.push(function_finding(
                file,
                function,
                "gorm_model_hook_contains_external_io",
                Severity::Warning,
                line,
                "GORM model hook performs external I/O",
                vec!["model hooks are usually safer when they stay local to persistence state changes".to_string()],
            ));
        }

        if import_path_has_any_role(file, SERVICE_ROLE_HINTS)
            || import_path_has_any_role(file, REPOSITORY_ROLE_HINTS)
        {
            findings.push(function_finding(
                file,
                function,
                "gorm_model_hook_calls_service_or_repository",
                Severity::Warning,
                function.fingerprint.start_line,
                "GORM model hook depends on service or repository packages",
                vec!["model hooks usually should not reach upward into higher application layers".to_string()],
            ));
        }
    }

    if is_gorm_hook(file, function, gorm_structs)
        && (go
            .gorm_query_chains
            .iter()
            .any(|chain| chain.root_text.contains("Other") || chain.root_text.contains("Audit"))
            || lines.iter().any(|line| line.text.contains("Audit")))
    {
        let line = go
            .gorm_query_chains
            .iter()
            .find(|chain| chain.root_text.contains("Other") || chain.root_text.contains("Audit"))
            .map(|chain| chain.line)
            .or_else(|| lines.iter().find(|line| line.text.contains("Audit")).map(|line| line.line))
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "gorm_hook_mutates_unrelated_tables",
            Severity::Info,
            line,
            "GORM hook appears to touch unrelated models or tables",
            vec!["hooks are usually safer when limited to the owning aggregate".to_string()],
        ));
    }

    findings
}

fn transaction_and_misc_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
    _gorm_structs: &BTreeSet<String>,
    returns_text: &str,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let go = function.go_evidence();
    let is_handler = is_request_path_function(file, function);
    let is_service = is_service_file(file);
    let is_repository = is_repository_file(file);

    if let Some((line, evidence)) =
        placeholder_seed_function_in_production_evidence(file, function, lines)
    {
        findings.push(function_finding(
            file,
            function,
            "placeholder_seed_function_in_production",
            Severity::Info,
            line,
            "seed entrypoint still looks like a placeholder",
            evidence,
        ));
    }

    if is_handler
        && lines
            .iter()
            .any(|line| line.text.contains(".Begin()") || line.text.contains(".BeginTx(") || line.text.contains(".Transaction("))
    {
        let line = lines
            .iter()
            .find(|line| {
                line.text.contains(".Begin()")
                    || line.text.contains(".BeginTx(")
                    || line.text.contains(".Transaction(")
            })
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "handler_opens_transaction",
            Severity::Warning,
            line,
            "request handler opens a transaction directly",
            vec!["transaction ownership is usually clearer in services or unit-of-work helpers".to_string()],
        ));
    }

    if !is_repository && !is_handler && !is_middleware_file(file)
        && lines.iter().any(|line| line.text.contains(".Session(") || line.text.contains(".Clauses("))
        && has_import_path(file, "gorm.io/gorm")
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains(".Session(") || line.text.contains(".Clauses("))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "gorm_session_options_configured_outside_repository",
            Severity::Info,
            line,
            "GORM session or clause options are configured outside a repository boundary",
            vec!["query configuration usually belongs with repository-owned query shape".to_string()],
        ));
    }

    if !is_repository
        && lines.iter().any(|line| line.text.contains(".Session("))
        && has_import_path(file, "gorm.io/gorm")
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains(".Session("))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "gorm_session_options_configured_outside_repository",
            Severity::Info,
            line,
            "GORM session options are configured outside repository code",
            vec!["query configuration usually belongs with repository-owned query shape".to_string()],
        ));
    }

    if go
        .gorm_query_chains
        .iter()
        .any(|chain| chain.steps.iter().any(|step| step.method_name == "Unscoped"))
        && !danger_named(&function.fingerprint.name)
    {
        let line = go
            .gorm_query_chains
            .iter()
            .find(|chain| chain.steps.iter().any(|step| step.method_name == "Unscoped"))
            .map(|chain| chain.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "unscoped_query_without_explicit_danger_naming",
            Severity::Info,
            line,
            "Unscoped query appears in a generically named function",
            vec!["hard-delete or unscoped access is usually easier to review when the function name makes the danger explicit".to_string()],
        ));
    }
    if !danger_named(&function.fingerprint.name)
        && lines.iter().any(|line| line.text.contains(".Unscoped("))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains(".Unscoped("))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "unscoped_query_without_explicit_danger_naming",
            Severity::Info,
            line,
            "Unscoped query appears in a generically named function",
            vec!["hard-delete or unscoped access is usually easier to review when the function name makes the danger explicit".to_string()],
        ));
    }

    if is_service
        && returns_text.contains("*gorm.DB")
        && (function.fingerprint.name.contains("Tx") || function.fingerprint.name.contains("Transaction"))
    {
        findings.push(function_finding(
            file,
            function,
            "service_returns_tx_to_caller",
            Severity::Warning,
            function.fingerprint.start_line,
            "service method returns transaction-like state to callers",
            vec!["services usually should own transactional work instead of leaking transaction handles".to_string()],
        ));
    }

    if is_service && signature_mentions_transaction(&function.signature_text) && lines.iter().any(|line| transaction_start_line(&line.text)) {
        let line = lines
            .iter()
            .find(|line| transaction_start_line(&line.text))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "same_service_method_accepts_tx_and_begins_tx",
            Severity::Info,
            line,
            "service method both accepts a transaction and begins one",
            vec!["mixing caller-owned and self-owned transactions usually hides ownership".to_string()],
        ));
    }

    if let Some(line) = external_http_inside_transaction_line(file, lines) {
        findings.push(function_finding(
            file,
            function,
            "external_http_call_inside_transaction_scope",
            Severity::Warning,
            line,
            "network I/O occurs while a transaction appears to be open",
            vec!["holding open transactions across network calls usually increases contention and ambiguity".to_string()],
        ));
    }

    if let Some(line) = action_inside_transaction_line(lines, &["Publish(", "publish("]) {
        findings.push(function_finding(
            file,
            function,
            "event_publish_before_transaction_commit",
            Severity::Warning,
            line,
            "event publication appears before transaction commit",
            vec!["durable commit usually should happen before publishing dependent events".to_string()],
        ));
    }

    if let Some(line) = action_inside_transaction_line(lines, &["Invalidate(", ".Del(", ".Delete("]) {
        findings.push(function_finding(
            file,
            function,
            "cache_invalidation_before_transaction_commit",
            Severity::Warning,
            line,
            "cache invalidation appears before transaction commit",
            vec!["cache mutation before commit can leave the cache ahead of durable state".to_string()],
        ));
    }

    if let Some(line) = action_inside_transaction_line(lines, &["go func", "go "]) {
        findings.push(function_finding(
            file,
            function,
            "background_goroutine_started_inside_transaction_scope",
            Severity::Warning,
            line,
            "goroutine is started while transaction state appears active",
            vec!["background work that depends on uncommitted state is usually fragile".to_string()],
        ));
    }

    if !is_repository && gorm_scope_count(function) >= 2 {
        findings.push(function_finding(
            file,
            function,
            "gorm_scopes_defined_inline_repeatedly",
            Severity::Info,
            function.fingerprint.start_line,
            "function defines several inline GORM scopes",
            vec!["named shared scopes are usually easier to govern than repeated inline closures".to_string()],
        ));
    }

    if !is_repository
        && (preload_count(function) >= 2
            || lines.iter().filter(|line| line.text.contains(".Preload(")).count() >= 2)
    {
        findings.push(function_finding(
            file,
            function,
            "preload_rules_scattered_across_layers",
            Severity::Info,
            function.fingerprint.start_line,
            "preload rules are built outside a repository boundary",
            vec!["preload ownership is usually clearer when repositories own query shape".to_string()],
        ));
    }

    if soft_delete_line_count(lines) >= 2 {
        let line = lines
            .iter()
            .find(|line| line.text.contains("deleted_at") && line.text.contains("IS NULL"))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "soft_delete_filters_written_manually_in_many_queries",
            Severity::Info,
            line,
            "function repeats manual soft-delete filters",
            vec!["shared scopes or repository helpers usually keep soft-delete policy more consistent".to_string()],
        ));
    }

    if !is_repository && let Some(line) = locking_clause_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "gorm_locking_clauses_built_outside_repository",
            Severity::Warning,
            line,
            "locking clause is built outside repository code",
            vec!["locking behavior is usually easier to review when owned by repositories".to_string()],
        ));
    }

    if (function.fingerprint.name.contains("Patch") || function.fingerprint.name.contains("Update"))
        && let Some(line) = update_struct_without_field_intent_line(lines)
    {
        findings.push(function_finding(
            file,
            function,
            "updates_with_struct_used_for_patch_without_field_intent_helper",
            Severity::Info,
            line,
            "patch-like update uses a struct write without explicit field intent",
            vec!["patch semantics are usually safer when selected fields are made explicit".to_string()],
        ));
    }

    if !is_repository && let Some(line) = map_update_flow_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "map_based_updates_passed_from_handler_to_repository",
            Severity::Info,
            line,
            "map-based update payload flows through upper layers",
            vec!["typed update contracts usually age better than ad hoc update maps".to_string()],
        ));
    }

    if !is_repository && let Some(line) = transaction_cross_layer_line(function, lines) {
        findings.push(function_finding(
            file,
            function,
            "transaction_object_crosses_more_than_one_layer_boundary",
            Severity::Info,
            line,
            "transaction handle crosses layer boundaries as a regular argument",
            vec!["transaction ownership is usually clearer when hidden behind unit-of-work boundaries".to_string()],
        ));
    }

    if let Some(line) = split_commit_rollback_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "commit_or_rollback_split_across_functions_without_owner",
            Severity::Info,
            line,
            "transaction completion paths look split without one clear owner",
            vec!["one explicit transaction owner usually makes commit and rollback behavior easier to follow".to_string()],
        ));
    }

    if let Some(line) = nested_tx_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "savepoint_or_nested_tx_logic_scattered_without_dedicated_helper",
            Severity::Info,
            line,
            "nested transaction or savepoint logic appears inline",
            vec!["savepoint and nested transaction flow is usually easier to audit behind a dedicated helper".to_string()],
        ));
    }

    if !(constructor_like(function) || is_router_file(file))
        && let Some(line) = feature_flag_lookup_line(file, lines)
    {
        findings.push(function_finding(
            file,
            function,
            "feature_flag_lookup_without_config_abstraction",
            Severity::Info,
            line,
            "feature flag or config lookup happens inline",
            vec!["focused config interfaces usually make feature gating easier to test than direct lookups".to_string()],
        ));
    }

    if is_main_or_cmd_file(file)
        && lines.iter().any(|line| line.text.contains("flag.Parse("))
        && lines.iter().any(|line| line.text.contains("gorm.Open(") || line.text.contains("NewService(") || line.text.contains("NewRepository("))
        && lines.iter().any(|line| line.text.contains(".Run(") || line.text.contains("ListenAndServe("))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains("flag.Parse("))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "main_or_cmd_mixes_cli_parsing_server_bootstrap_and_business_config",
            Severity::Info,
            line,
            "main or cmd path mixes CLI parsing, server startup, and business wiring",
            vec!["startup composition helpers usually make entrypoints easier to maintain than one large bootstrap path".to_string()],
        ));
    }

    if is_main_or_cmd_file(file) && let Some(line) = lifecycle_start_without_shutdown_line(lines)
    {
        findings.push(function_finding(
            file,
            function,
            "application_lifecycle_missing_shutdown_owner",
            Severity::Info,
            line,
            "startup path launches long-lived resources without an obvious shutdown owner",
            vec!["application lifecycle wrappers usually make shutdown behavior more explicit".to_string()],
        ));
    }

    if is_main_or_cmd_file(file)
        && import_path_has_any_role(file, TRANSPORT_ROLE_HINTS)
        && lines.iter().any(|line| line.text.contains("NewHandlerService(") || line.text.contains("gin."))
    {
        findings.push(function_finding(
            file,
            function,
            "operational_command_handlers_reuse_http_services_without_adapter",
            Severity::Info,
            function.fingerprint.start_line,
            "operational command path appears to depend on HTTP-oriented service contracts",
            vec!["CLI and worker entrypoints usually compose better against transport-neutral application services".to_string()],
        ));
    }

    if is_handler && let Some(line) = metrics_label_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "metrics_labels_built_inline_in_handlers",
            Severity::Info,
            line,
            "handler builds metric labels inline",
            vec!["focused observation helpers usually keep metric label sets more consistent than ad hoc handler code".to_string()],
        ));
    }

    if is_repository && let Some(line) = repository_log_http_metadata_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "repository_logs_with_http_status_or_route_labels",
            Severity::Info,
            line,
            "repository log call includes HTTP route or status metadata",
            vec!["persistence logs are usually clearer when they stick to persistence-domain fields rather than transport labels".to_string()],
        ));
    }

    if is_handler && let Some(line) = audit_before_service_line(lines) {
        findings.push(function_finding(
            file,
            function,
            "audit_logging_executed_in_handler_before_service_success",
            Severity::Info,
            line,
            "handler performs audit logging before service success is known",
            vec!["audit logging often belongs after the business outcome is confirmed".to_string()],
        ));
    }

    if !is_repository
        && lines.iter().any(|line| line.text.contains(".Order(\""))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains(".Order(\""))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "order_by_clause_literals_scattered_across_layers",
            Severity::Info,
            line,
            "raw ORDER BY clause is built outside repository code",
            vec!["shared query helpers usually reduce scattered sort expressions across layers".to_string()],
        ));
    }

    if is_repository
        && lines.iter().any(|line| line.text.contains("tx.") && line.text.contains("Status"))
        && lines.iter().any(|line| line.text.contains("err"))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains("tx.") && line.text.contains("Status"))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "transaction_error_translation_done_in_repository_and_handler",
            Severity::Info,
            line,
            "repository translates transaction failures into transport status semantics",
            vec!["keeping transaction error translation in one layer usually reduces double handling".to_string()],
        ));
    }

    if file_matches_any_role(file, VALIDATION_ROLE_HINTS)
        && references_repository_directly(lines)
    {
        findings.push(function_finding(
            file,
            function,
            "validator_depends_on_repository_directly",
            Severity::Info,
            function.fingerprint.start_line,
            "validator code reaches repository-like dependencies directly",
            vec!["validators usually compose better from precomputed facts or focused services than direct repository calls".to_string()],
        ));
    }

    findings
}

fn placeholder_seed_function_in_production_evidence(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<(usize, Vec<String>)> {
    if file.is_test_file || function.is_test_function || !seed_entrypoint_candidate(file, function) {
        return None;
    }

    let non_empty = lines
        .iter()
        .filter(|line| !line.text.trim().is_empty())
        .collect::<Vec<_>>();
    if non_empty.is_empty() {
        return Some((
            function.fingerprint.start_line,
            vec![
                format!("seed-like function: {}", function.fingerprint.name),
                "function body is effectively empty".to_string(),
            ],
        ));
    }

    if let Some(line) = non_empty
        .iter()
        .find(|line| seed_placeholder_marker(&line.text))
        .map(|line| line.line)
    {
        return Some((
            line,
            vec![
                format!("seed-like function: {}", function.fingerprint.name),
                "body still contains TODO or not-implemented placeholder markers".to_string(),
            ],
        ));
    }

    if non_empty.len() <= 3
        && non_empty
            .iter()
            .all(|line| trivial_seed_line(&line.text))
    {
        return Some((
            non_empty[0].line,
            vec![
                format!("seed-like function: {}", function.fingerprint.name),
                "body only returns trivial values without visible seeding work".to_string(),
            ],
        ));
    }

    None
}

fn seed_entrypoint_candidate(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let lower_path = file.path.to_string_lossy().to_ascii_lowercase();
    let lower_package = file
        .package_name
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let lower_name = function.fingerprint.name.to_ascii_lowercase();

    if lower_name.starts_with("seed")
        || lower_name.contains("seeddata")
        || lower_name.contains("runseed")
    {
        return true;
    }

    if function.fingerprint.name == "main"
        && (lower_path.contains("/cmd/seed/")
            || lower_path.contains("/cmd/seeds/")
            || lower_path.ends_with("/seed/main.go"))
    {
        return true;
    }

    lower_path.contains("/seed/")
        || lower_path.contains("/seeds/")
        || lower_path.contains("/seeder/")
        || lower_path.ends_with("/seed.go")
        || lower_package == "seed"
        || lower_package == "seeds"
}

fn seed_placeholder_marker(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("todo")
        || lower.contains("not implemented")
        || lower.contains("panic(\"todo")
        || lower.contains("panic(\"not implemented")
}

fn trivial_seed_line(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed == "return"
        || trimmed.starts_with("if ")
        || trimmed.starts_with('{')
        || trimmed.starts_with('}')
        || noop_like_return_line(trimmed)
}
