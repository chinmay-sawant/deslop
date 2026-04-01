use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

// ── Import-gating helpers ─────────────────────────────────────────────────────

fn has_import(file: &ParsedFile, module: &str) -> bool {
    file.imports
        .iter()
        .any(|imp| imp.path.contains(module) || imp.alias.contains(module))
}

fn is_handler_or_view(function: &ParsedFunction, file: &ParsedFile) -> bool {
    // Check body_text of the function for common decorator patterns
    let sig = &function.signature_text;
    sig.contains("@app.route")
        || sig.contains("@bp.route")
        || sig.contains("@router.")
        || sig.contains("@api_view")
        || sig.contains("@action")
        || sig.contains("@app.get")
        || sig.contains("@app.post")
        || sig.contains("@app.put")
        || sig.contains("@app.delete")
        || (has_import(file, "django")
            && (function.fingerprint.name == "get"
                || function.fingerprint.name == "post"
                || function.fingerprint.name == "put"
                || function.fingerprint.name == "delete"
                || function.fingerprint.name == "list"
                || function.fingerprint.name == "create"
                || function.fingerprint.name == "update"
                || function.fingerprint.name == "destroy"))
}

fn is_middleware(function: &ParsedFunction) -> bool {
    let sig = &function.signature_text;
    sig.contains("@app.before_request")
        || sig.contains("@app.after_request")
        || sig.contains("@app.middleware")
        || sig.contains("process_request")
        || sig.contains("process_response")
        || sig.contains("process_view")
}

// ── Django ORM rules ──────────────────────────────────────────────────────────

pub(super) fn django_queryset_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "django") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // django_queryset_count_then_exists
    if (body.contains(".count() > 0")
        || body.contains(".count() >= 1")
        || body.contains(".count() != 0"))
        && let Some(line) = find_line(body, ".count()", function.fingerprint.start_line)
    {
        findings.push(Finding {
            rule_id: "django_queryset_count_then_exists".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "function {} uses .count() > 0; prefer .exists() to avoid full count",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=count_then_exists".to_string()],
        });
    }

    // django_queryset_len_instead_of_count
    for call in &function.calls {
        if call.name == "len" && body.contains("len(") && body.contains(".objects") {
            findings.push(Finding {
                rule_id: "django_queryset_len_instead_of_count".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} uses len(queryset) which loads all objects; prefer .count()",
                    function.fingerprint.name
                ),
                evidence: vec!["pattern=len_on_queryset".to_string()],
            });
            break;
        }
    }

    // django_all_without_limit_in_view
    if is_handler_or_view(function, file)
        && body.contains(".objects.all()")
        && !body.contains("[:")
        && !body.contains(".first()")
        && !body.contains("paginate")
        && !body.contains("Paginator")
        && let Some(line) = find_line(body, ".objects.all()", function.fingerprint.start_line)
    {
        findings.push(Finding {
            rule_id: "django_all_without_limit_in_view".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "function {} loads all() without pagination or limit in a view",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=unbounded_queryset_in_view".to_string()],
        });
    }

    // django_queryset_order_by_random
    if (body.contains(".order_by('?')") || body.contains(".order_by(\"?\")"))
        && let Some(line) = find_line(body, ".order_by(", function.fingerprint.start_line)
    {
        findings.push(Finding {
            rule_id: "django_queryset_order_by_random".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "function {} uses .order_by('?') which causes ORDER BY RANDOM() full table scan",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=order_by_random".to_string()],
        });
    }

    findings
}

pub(super) fn django_n_plus_one_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "django") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // Check for loops over querysets accessing FK attributes without select_related/prefetch_related
    let has_select_related = body.contains("select_related") || body.contains("prefetch_related");
    if !has_select_related {
        let lines: Vec<&str> = body.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("for ")
                && trimmed.contains(".objects")
                && trimmed.contains(".filter(")
            {
                findings.push(Finding {
                    rule_id: "django_n_plus_one_no_select_related".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: function.fingerprint.start_line + i,
                    end_line: function.fingerprint.start_line + i,
                    message: format!(
                        "function {} iterates over a queryset without select_related/prefetch_related",
                        function.fingerprint.name
                    ),
                    evidence: vec!["pattern=n_plus_one_query_risk".to_string()],
                });
            }
        }
    }

    findings
}

pub(super) fn django_loop_db_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "django") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
        if loop_indent.is_none() || trimmed.is_empty() {
            continue;
        }
        let sl = function.fingerprint.start_line + i;

        // django_save_full_model_in_loop
        if trimmed.contains(".save(") && !trimmed.contains("update_fields") {
            findings.push(make_finding(
                "django_save_full_model_in_loop",
                Severity::Info,
                file,
                function,
                sl,
                "saves a full model inside a loop without update_fields; consider bulk_update()",
            ));
        }
        // django_create_single_in_loop
        if trimmed.contains(".objects.create(")
            || (trimmed.ends_with(".save()") && trimmed.contains("("))
        {
            findings.push(make_finding(
                "django_create_single_in_loop",
                Severity::Info,
                file,
                function,
                sl,
                "creates a single model inside a loop; consider bulk_create()",
            ));
        }
        // django_delete_single_in_loop
        if trimmed.contains(".delete()") && !trimmed.contains(".objects") {
            findings.push(make_finding(
                "django_delete_single_in_loop",
                Severity::Info,
                file,
                function,
                sl,
                "deletes instances in a loop; consider QuerySet.delete()",
            ));
        }
        // django_raw_sql_in_loop
        if trimmed.contains(".execute(") && (body.contains("cursor") || body.contains(".raw(")) {
            findings.push(make_finding(
                "django_raw_sql_in_loop",
                Severity::Info,
                file,
                function,
                sl,
                "executes raw SQL inside a loop; consider batching",
            ));
        }
    }

    findings
}

pub(super) fn django_values_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "django") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // django_queryset_evaluated_multiple_times - same queryset var iterated multiple times
    // Simple heuristic: if we see the same `.filter(` result used in multiple for loops
    let lines: Vec<&str> = body.lines().collect();
    let mut qs_vars: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for line in &lines {
        let trimmed = line.trim();
        if let Some(eq_idx) = trimmed.find(" = ") {
            let rhs = &trimmed[eq_idx + 3..];
            if rhs.contains(".objects.") || rhs.contains(".filter(") || rhs.contains(".all()") {
                let var = trimmed[..eq_idx].trim().to_string();
                qs_vars.insert(var, 0);
            }
        }
    }
    let qs_keys: Vec<String> = qs_vars.keys().cloned().collect();
    for line in &lines {
        let trimmed = line.trim();
        for var in &qs_keys {
            if trimmed.starts_with("for ") && trimmed.contains(&format!(" in {var}"))
                || trimmed.contains(&format!("len({var})"))
                || trimmed.contains(&format!("list({var})"))
                || trimmed.contains(&format!("if {var}:"))
            {
                *qs_vars.entry(var.clone()).or_default() += 1;
            }
        }
    }
    for (var, count) in &qs_vars {
        if *count >= 2
            && let Some(line) = find_line(body, var, function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "django_queryset_evaluated_multiple_times",
                Severity::Warning,
                file,
                function,
                line,
                &format!(
                    "queryset '{var}' appears to be evaluated multiple times, causing duplicate SQL"
                ),
            ));
        }
    }

    findings
}

// ── Flask rules ───────────────────────────────────────────────────────────────

pub(super) fn flask_handler_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "flask") {
        return Vec::new();
    }
    let body = &function.body_text;
    let sig = &function.signature_text;
    let is_view = sig.contains("@app.route") || sig.contains("@bp.route");
    let mut findings = Vec::new();

    // flask_request_body_parsed_multiple_times
    if is_view {
        let get_json_count =
            body.matches("request.get_json()").count() + body.matches("request.json").count();
        if get_json_count >= 2
            && let Some(line) = find_line(body, "request.get_json", function.fingerprint.start_line)
                .or_else(|| find_line(body, "request.json", function.fingerprint.start_line))
        {
            findings.push(make_finding(
                "flask_request_body_parsed_multiple_times",
                Severity::Info,
                file,
                function,
                line,
                "parses request body multiple times; cache in a local variable",
            ));
        }
    }

    // flask_global_db_connection_per_request
    if is_view {
        for pattern in &[
            "sqlite3.connect(",
            "pymongo.MongoClient(",
            "psycopg2.connect(",
            "mysql.connector.connect(",
        ] {
            if body.contains(pattern)
                && let Some(line) = find_line(body, pattern, function.fingerprint.start_line)
            {
                findings.push(make_finding(
                    "flask_global_db_connection_per_request",
                    Severity::Warning,
                    file,
                    function,
                    line,
                    "creates a database connection per request; use app-scoped connection pooling",
                ));
            }
        }
    }

    // flask_app_config_read_per_request
    if is_view && body.contains("app.config[") {
        let config_count = body.matches("app.config[").count();
        if config_count >= 2
            && let Some(line) = find_line(body, "app.config[", function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "flask_app_config_read_per_request",
                Severity::Info,
                file,
                function,
                line,
                "reads app.config multiple times per request; read once at startup",
            ));
        }
    }

    // flask_template_rendered_from_string_in_view
    if is_view
        && body.contains("render_template_string(")
        && let Some(line) = find_line(
            body,
            "render_template_string(",
            function.fingerprint.start_line,
        )
    {
        findings.push(make_finding(
            "flask_template_rendered_from_string_in_view",
            Severity::Info,
            file,
            function,
            line,
            "renders template from string in view; use render_template() with a file instead",
        ));
    }

    // flask_file_read_per_request
    if is_view
        && (body.contains("open(") || body.contains(".read_text()"))
        && let Some(line) = find_line(body, "open(", function.fingerprint.start_line)
            .or_else(|| find_line(body, ".read_text()", function.fingerprint.start_line))
    {
        findings.push(make_finding(
            "flask_file_read_per_request",
            Severity::Info,
            file,
            function,
            line,
            "reads a file per request; consider caching static content at startup",
        ));
    }

    // flask_debug_mode_in_production_code
    if body.contains("app.run(")
        && body.contains("debug=True")
        && let Some(line) = find_line(body, "debug=True", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "flask_debug_mode_in_production_code",
            Severity::Warning,
            file,
            function,
            line,
            "runs app with debug=True which exposes the debugger in production",
        ));
    }

    // flask_json_encoder_per_request
    if is_view
        && body.contains("JSONEncoder(")
        && let Some(line) = find_line(body, "JSONEncoder(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "flask_json_encoder_per_request",
            Severity::Info,
            file,
            function,
            line,
            "creates JSONEncoder per request; configure app-level encoder instead",
        ));
    }

    // flask_no_streaming_for_large_response
    if is_view && (body.contains("jsonify(") || body.contains("json.dumps(")) {
        // Check if building a large list before serializing
        let has_large_build = body.contains("for ")
            && (body.contains(".append(") || body.contains("results.extend("));
        if has_large_build
            && let Some(line) = find_line(body, "jsonify(", function.fingerprint.start_line)
                .or_else(|| find_line(body, "json.dumps(", function.fingerprint.start_line))
        {
            findings.push(make_finding(
                    "flask_no_streaming_for_large_response", Severity::Info, file, function, line,
                    "builds a large list then serializes; consider Response(generate(), ...) for streaming",
                ));
        }
    }

    findings
}

// ── FastAPI rules ─────────────────────────────────────────────────────────────

pub(super) fn fastapi_handler_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "fastapi") {
        return Vec::new();
    }
    let body = &function.body_text;
    let sig = &function.signature_text;
    let python = function.python_evidence();
    let mut findings = Vec::new();

    // fastapi_sync_def_with_blocking_io
    let is_route = sig.contains("@router.") || sig.contains("@app.");
    if is_route && !python.is_async {
        let has_blocking = body.contains("requests.get(")
            || body.contains("requests.post(")
            || body.contains("open(")
            || body.contains("time.sleep(")
            || body.contains("subprocess.");
        if has_blocking {
            findings.push(make_finding(
                "fastapi_sync_def_with_blocking_io",
                Severity::Warning,
                file,
                function,
                function.fingerprint.start_line,
                "sync def route handler contains blocking I/O; use async def or run_in_executor",
            ));
        }
    }

    // fastapi_dependency_creates_client_per_request
    if sig.contains("Depends") || function.fingerprint.name.starts_with("get_") {
        let creates_client = body.contains("httpx.Client(")
            || body.contains("httpx.AsyncClient(")
            || body.contains("requests.Session(")
            || body.contains("aiohttp.ClientSession(");
        if creates_client
            && let Some(line) = find_line(body, "Client(", function.fingerprint.start_line)
                .or_else(|| find_line(body, "Session(", function.fingerprint.start_line))
        {
            findings.push(make_finding(
                "fastapi_dependency_creates_client_per_request",
                Severity::Warning,
                file,
                function,
                line,
                "creates HTTP client per request in dependency; use app lifespan",
            ));
        }
    }

    // fastapi_response_model_without_orm_mode
    // Skipped: too complex for body_text analysis, would need type resolution

    // fastapi_background_task_exception_silent
    if body.contains("add_task(")
        && body.contains("BackgroundTask")
        && let Some(line) = find_line(body, "add_task(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "fastapi_background_task_exception_silent",
            Severity::Info,
            file,
            function,
            line,
            "background task may silently swallow exceptions; add error handling",
        ));
    }

    findings
}

// ── SQLAlchemy rules ──────────────────────────────────────────────────────────

pub(super) fn sqlalchemy_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !(has_import(file, "sqlalchemy") || has_import(file, "Session"))
    {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // sqlalchemy_session_not_closed
    if body.contains("Session(")
        && !body.contains("with ")
        && !body.contains(".close()")
        && let Some(line) = find_line(body, "Session(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
                "sqlalchemy_session_not_closed", Severity::Warning, file, function, line,
                "creates a Session without context manager or .close(); use 'with Session() as session:'",
            ));
    }

    // sqlalchemy_create_engine_per_request
    if is_handler_or_view(function, file)
        && body.contains("create_engine(")
        && let Some(line) = find_line(body, "create_engine(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "sqlalchemy_create_engine_per_request",
            Severity::Warning,
            file,
            function,
            line,
            "creates engine per request; reuse a process-level engine",
        ));
    }

    // sqlalchemy_commit_per_row_in_loop
    let lines: Vec<&str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
        if loop_indent.is_some() && !trimmed.is_empty() {
            if trimmed.contains("session.commit()") || trimmed.contains(".commit()") {
                findings.push(make_finding(
                    "sqlalchemy_commit_per_row_in_loop",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "commits inside a loop; batch changes and commit once after the loop",
                ));
            }
            if trimmed.contains("session.query(") || trimmed.contains("session.execute(") {
                findings.push(make_finding(
                    "sqlalchemy_query_in_loop",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "queries inside a loop; batch with .in_() or bulk operations",
                ));
            }
        }
    }

    // sqlalchemy_n_plus_one_lazy_load
    if body.contains("session.query(")
        && !body.contains("joinedload")
        && !body.contains("subqueryload")
        && !body.contains("selectinload")
    {
        // Check for attribute access in loops on query results
        let has_loop_access = lines.iter().enumerate().any(|(i, line)| {
            let t = line.trim();
            i > 0
                && t.starts_with("for ")
                && lines[0..i].iter().any(|l| l.contains("session.query("))
        });
        if has_loop_access
            && let Some(line) = find_line(body, "session.query(", function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "sqlalchemy_n_plus_one_lazy_load",
                Severity::Info,
                file,
                function,
                line,
                "queries without eager loading; add joinedload/subqueryload to prevent N+1",
            ));
        }
    }

    // sqlalchemy_expire_on_commit_default_in_async
    let python = function.python_evidence();
    if python.is_async
        && body.contains("Session(")
        && !body.contains("expire_on_commit=False")
        && let Some(line) = find_line(body, "Session(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "sqlalchemy_expire_on_commit_default_in_async",
            Severity::Info,
            file,
            function,
            line,
            "async session uses expire_on_commit=True (default); set False to avoid implicit I/O",
        ));
    }

    findings
}

// ── Middleware rules ──────────────────────────────────────────────────────────

pub(super) fn middleware_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !is_middleware(function) {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // middleware_creates_http_client_per_request
    for pattern in &[
        "requests.Session(",
        "httpx.Client(",
        "aiohttp.ClientSession(",
    ] {
        if body.contains(pattern)
            && let Some(line) = find_line(body, pattern, function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "middleware_creates_http_client_per_request",
                Severity::Warning,
                file,
                function,
                line,
                "creates HTTP client per request in middleware; use app-scoped client",
            ));
        }
    }

    // middleware_loads_config_file_per_request
    for pattern in &[
        "yaml.safe_load(",
        "json.load(",
        "toml.load(",
        "configparser.",
    ] {
        if body.contains(pattern)
            && let Some(line) = find_line(body, pattern, function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "middleware_loads_config_file_per_request",
                Severity::Info,
                file,
                function,
                line,
                "loads config per request in middleware; read once at startup",
            ));
        }
    }

    // middleware_compiles_regex_per_request
    if body.contains("re.compile(")
        && let Some(line) = find_line(body, "re.compile(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "middleware_compiles_regex_per_request",
            Severity::Info,
            file,
            function,
            line,
            "compiles regex per request in middleware; precompile at module level",
        ));
    }

    findings
}

// ── Handler fanout rules ─────────────────────────────────────────────────────

pub(super) fn handler_fanout_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_handler_or_view(function, file) {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // upstream_http_call_per_item_in_handler
    let lines: Vec<&str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
        if loop_indent.is_some()
            && !trimmed.is_empty()
            && (trimmed.contains("requests.get(")
                || trimmed.contains("requests.post(")
                || trimmed.contains("httpx.get(")
                || trimmed.contains("httpx.post(")
                || trimmed.contains("aiohttp"))
        {
            findings.push(make_finding(
                "upstream_http_call_per_item_in_handler",
                Severity::Warning,
                file,
                function,
                function.fingerprint.start_line + i,
                "makes sequential HTTP calls inside a loop in handler; batch or parallelize",
            ));
        }
    }

    // upstream_call_without_timeout_in_handler
    for call in &function.calls {
        if (call.name == "get"
            || call.name == "post"
            || call.name == "put"
            || call.name == "delete"
            || call.name == "request")
            && call.receiver.as_deref() == Some("requests")
        {
            // Check if the call line has timeout=
            let call_line_text = body
                .lines()
                .nth(call.line.saturating_sub(function.fingerprint.start_line));
            if let Some(lt) = call_line_text
                && !lt.contains("timeout")
            {
                findings.push(make_finding(
                        "upstream_call_without_timeout_in_handler", Severity::Warning, file, function,
                        call.line,
                        "HTTP call without timeout in handler; add timeout= to prevent unbounded latency",
                    ));
            }
        }
    }

    // upstream_response_not_checked_before_decode
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains(".json()") || trimmed.contains("json.loads(response") {
            // Check previous lines for status_code check
            let has_check = (i.saturating_sub(3)..i).any(|j| {
                let prev = lines.get(j).unwrap_or(&"").trim();
                prev.contains("status_code")
                    || prev.contains(".ok")
                    || prev.contains("raise_for_status")
            });
            if !has_check {
                findings.push(make_finding(
                    "upstream_response_not_checked_before_decode", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "decodes response without checking status; check response.ok or status_code first",
                ));
            }
        }
    }

    findings
}

// ── Template and response rules ──────────────────────────────────────────────

pub(super) fn template_response_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // template_render_in_loop
    let lines: Vec<&str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
        if loop_indent.is_some() && !trimmed.is_empty() {
            if trimmed.contains("Template(") && trimmed.contains(".render(") {
                findings.push(make_finding(
                    "template_render_in_loop",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "renders a template inside a loop; render once with loop data",
                ));
            }
            if trimmed.contains("render_template_string(") {
                findings.push(make_finding(
                    "template_render_in_loop",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "renders template string inside a loop; use a single template",
                ));
            }
        }
    }

    // response_json_dumps_then_response_object
    if body.contains("json.dumps(")
        && body.contains("Response(")
        && (has_import(file, "flask") || has_import(file, "fastapi"))
        && let Some(line) = find_line(body, "json.dumps(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "response_json_dumps_then_response_object",
            Severity::Info,
            file,
            function,
            line,
            "manually dumps JSON then wraps in Response; use jsonify() or JSONResponse()",
        ));
    }

    findings
}

// ── Remaining Plan 2 Wave 5 rules ────────────────────────────────────────────

pub(super) fn django_extra_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "django") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // django_update_single_in_loop
    let lines: Vec<&str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
        if loop_indent.is_some()
            && !trimmed.is_empty()
            && trimmed.contains(".update(")
            && trimmed.contains(".objects.filter(")
        {
            findings.push(make_finding(
                "django_update_single_in_loop",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "updates single objects in a loop; consider bulk_update() or QuerySet.update()",
            ));
        }
    }

    // django_migration_code_in_view
    if is_handler_or_view(function, file) {
        for pattern in &["migrate", "makemigrations", "schema_editor", "RunPython"] {
            if body.contains(pattern)
                && let Some(line) = find_line(body, pattern, function.fingerprint.start_line)
            {
                findings.push(make_finding(
                        "django_migration_code_in_view", Severity::Warning, file, function, line,
                        "references migration/schema operations in a view; these belong in migration files",
                    ));
            }
        }
    }

    // django_values_vs_full_model_in_loop
    if body.contains(".objects.filter(") || body.contains(".objects.all()") {
        // Check if iterating then only accessing 1-2 attributes
        let has_values =
            body.contains(".values(") || body.contains(".values_list(") || body.contains(".only(");
        if !has_values {
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("for ") && trimmed.contains(".objects.") {
                    findings.push(make_finding(
                        "django_values_vs_full_model_in_loop", Severity::Info, file, function,
                        function.fingerprint.start_line + i,
                        "hydrates full models; use .values() or .only() if only a few fields are needed",
                    ));
                }
            }
        }
    }

    findings
}

pub(super) fn response_extra_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // large_dict_literal_response_in_handler
    if is_handler_or_view(function, file)
        && (body.contains("jsonify(")
            || body.contains("JSONResponse(")
            || body.contains("json.dumps("))
    {
        // Count inline dict literal keys (heuristic: count lines with `"key":`)
        let dict_key_count = body
            .lines()
            .filter(|l| {
                let t = l.trim();
                (t.contains("\": ") || t.contains("': ")) && !t.starts_with('#')
            })
            .count();
        if dict_key_count >= 8
            && let Some(line) = find_line(body, "jsonify(", function.fingerprint.start_line)
                .or_else(|| find_line(body, "JSONResponse(", function.fingerprint.start_line))
                .or_else(|| find_line(body, "json.dumps(", function.fingerprint.start_line))
        {
            findings.push(make_finding(
                    "large_dict_literal_response_in_handler", Severity::Info, file, function, line,
                    "builds a large inline dict for response; consider a Pydantic model or typed response",
                ));
        }
    }

    // fastapi_response_model_without_orm_mode
    if has_import(file, "fastapi")
        && body.contains("response_model")
        && body.contains(".from_orm(")
        && !body.contains("model_config")
        && !body.contains("orm_mode")
        && let Some(line) = find_line(body, ".from_orm(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "fastapi_response_model_without_orm_mode",
            Severity::Info,
            file,
            function,
            line,
            "uses .from_orm() without orm_mode; configure model_config for ORM compatibility",
        ));
    }

    findings
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_line(body: &str, needle: &str, base_line: usize) -> Option<usize> {
    for (i, line) in body.lines().enumerate() {
        if line.contains(needle) {
            return Some(base_line + i);
        }
    }
    None
}

fn make_finding(
    rule_id: &str,
    severity: Severity,
    file: &ParsedFile,
    function: &ParsedFunction,
    line: usize,
    msg_suffix: &str,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!("function {} {msg_suffix}", function.fingerprint.name),
        evidence: vec![format!("rule={rule_id}")],
    }
}
