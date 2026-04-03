use super::*;

pub(crate) fn flask_handler_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "flask") {
        return Vec::new();
    }
    let body = &function.body_text;
    let sig = &function.signature_text;
    let is_view = sig.contains("@app.route") || sig.contains("@bp.route");
    let mut findings = Vec::new();

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

    if is_view && (body.contains("jsonify(") || body.contains("json.dumps(")) {
        let has_large_build = body.contains("for ")
            && (body.contains(".append(") || body.contains("results.extend("));
        if has_large_build
            && let Some(line) = find_line(body, "jsonify(", function.fingerprint.start_line)
                .or_else(|| find_line(body, "json.dumps(", function.fingerprint.start_line))
        {
            findings.push(make_finding(
                "flask_no_streaming_for_large_response",
                Severity::Info,
                file,
                function,
                line,
                "builds a large list then serializes; consider Response(generate(), ...) for streaming",
            ));
        }
    }

    findings
}

pub(crate) fn fastapi_handler_findings(
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

pub(crate) fn middleware_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !is_middleware(function) {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

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

pub(crate) fn handler_fanout_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_handler_or_view(function, file) {
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

    for call in &function.calls {
        if (call.name == "get"
            || call.name == "post"
            || call.name == "put"
            || call.name == "delete"
            || call.name == "request")
            && call.receiver.as_deref() == Some("requests")
        {
            let call_line_text = body
                .lines()
                .nth(call.line.saturating_sub(function.fingerprint.start_line));
            if let Some(lt) = call_line_text
                && !lt.contains("timeout")
            {
                findings.push(make_finding(
                    "upstream_call_without_timeout_in_handler",
                    Severity::Warning,
                    file,
                    function,
                    call.line,
                    "HTTP call without timeout in handler; add timeout= to prevent unbounded latency",
                ));
            }
        }
    }

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains(".json()") || trimmed.contains("json.loads(response") {
            let has_check = (i.saturating_sub(3)..i).any(|j| {
                let prev = lines.get(j).unwrap_or(&"").trim();
                prev.contains("status_code")
                    || prev.contains(".ok")
                    || prev.contains("raise_for_status")
            });
            if !has_check {
                findings.push(make_finding(
                    "upstream_response_not_checked_before_decode",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "decodes response without checking status; check response.ok or status_code first",
                ));
            }
        }
    }

    findings
}

pub(crate) fn template_response_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
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

pub(crate) fn response_extra_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    if is_handler_or_view(function, file)
        && (body.contains("jsonify(")
            || body.contains("JSONResponse(")
            || body.contains("json.dumps("))
    {
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
                "large_dict_literal_response_in_handler",
                Severity::Info,
                file,
                function,
                line,
                "builds a large inline dict for response; consider a Pydantic model or typed response",
            ));
        }
    }

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
