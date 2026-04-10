/// architecture.rs — Section 1 (layer boundaries, 20 rules) +
///                   Section 2 (async/concurrency correctness, 20 rules)
use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

// ── Helpers ───────────────────────────────────────────────────────────────────

fn is_role_file(file: &ParsedFile, roles: &[&str]) -> bool {
    let path = file.path.to_string_lossy().to_lowercase();
    roles.iter().any(|role| path.contains(role))
}

fn find_line(body: &str, needle: &str, base: usize) -> Option<usize> {
    body.lines()
        .enumerate()
        .find_map(|(i, l)| l.contains(needle).then_some(base + i))
}

fn make_finding(
    rule_id: &str,
    severity: Severity,
    file: &ParsedFile,
    function: &ParsedFunction,
    line: usize,
    msg: &str,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!("function {} {msg}", function.fingerprint.name),
        evidence: vec![format!("rule={rule_id}")],
    }
}

const SERVICE_ROLES: &[&str] = &["service", "services"];
const REPO_ROLES: &[&str] = &["repository", "repo", "repositories"];
const VIEW_ROLES: &[&str] = &["view", "views", "handler", "handlers"];
const DOMAIN_ROLES: &[&str] = &["domain", "model", "models", "entity", "entities"];
const ALL_ROLES: &[&str] = &[
    "service", "services", "repository", "repo", "repositories",
    "view", "views", "handler", "handlers", "domain", "model", "models",
    "api", "router", "middleware", "schema", "schemas", "dto",
];

// ── Section 1 · Architecture and Layer Boundaries ────────────────────────────

pub(super) fn service_accepts_http_request_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, SERVICE_ROLES) {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "flask.Request", "flask.request", "django.HttpRequest",
        "fastapi.Request", "Request,", ": Request)", ": Request,",
    ];
    for p in PATTERNS {
        if sig.contains(p) || body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "service_method_accepts_http_request_object",
                Severity::Warning, file, function, line,
                "accepts an HTTP request object directly; extract domain values at the transport layer first",
            )];
        }
    }
    Vec::new()
}

pub(super) fn repository_returns_query_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, REPO_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "return self.db.query(", "return session.query(", "return db.query(",
        "return QuerySet", "return self.session.query(",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "repository_returns_unexecuted_orm_query",
                Severity::Warning, file, function, line,
                "returns an unexecuted ORM query object; evaluate and return domain values instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn view_builds_orm_query_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, VIEW_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        ".filter(", ".annotate(", ".aggregate(", ".select_related(",
        "session.query(", ".objects.filter(",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "view_or_handler_constructs_orm_query_directly",
                Severity::Warning, file, function, line,
                "builds ORM query directly; delegate to a repository or query object",
            )];
        }
    }
    Vec::new()
}

pub(super) fn domain_imports_http_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, DOMAIN_ROLES) {
        return Vec::new();
    }
    let imports: Vec<&str> = file.imports.iter().map(|i| i.path.as_str()).collect();
    const HTTP_MODS: &[&str] = &["requests", "httpx", "http.client", "flask", "django.http", "fastapi"];
    for m in HTTP_MODS {
        if imports.iter().any(|i| i.starts_with(m)) {
            return vec![Finding {
                rule_id: "domain_model_class_imports_http_library".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line,
                end_line: function.fingerprint.start_line,
                message: format!(
                    "domain module imports HTTP library {}; domain should be transport-neutral",
                    m
                ),
                evidence: vec![format!("http_import={m}")],
            }];
        }
    }
    Vec::new()
}

pub(super) fn service_raises_http_exception_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, SERVICE_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "HTTPException(", "raise HTTPException", "werkzeug.exceptions",
        "abort(", "raise abort(",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "service_raises_or_catches_http_exception_type",
                Severity::Warning, file, function, line,
                "raises or catches an HTTP exception type; use domain-neutral errors instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn handler_builds_raw_sql_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, VIEW_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "cursor.execute(", ".execute(f\"", ".execute(\"SELECT",
        ".execute(\"INSERT", ".execute(\"UPDATE", ".execute(\"DELETE",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "handler_or_view_builds_raw_sql",
                Severity::Warning, file, function, line,
                "builds raw SQL inside a handler; delegate database access to a repository",
            )];
        }
    }
    Vec::new()
}

pub(super) fn service_returns_http_response_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, SERVICE_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "return Response(", "return JsonResponse(", "return HTMLResponse(",
        "return jsonify(", "return make_response(",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "service_method_returns_http_response_object",
                Severity::Warning, file, function, line,
                "returns an HTTP response object; return domain results instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn handler_owns_transaction_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, VIEW_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "session.begin(", "transaction.atomic(", "db.begin(", "conn.begin(",
        "with db.transaction(",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "handler_or_view_owns_transaction_lifecycle",
                Severity::Warning, file, function, line,
                "owns transaction lifecycle directly; delegate transactional work to a service",
            )];
        }
    }
    Vec::new()
}

pub(super) fn service_reads_settings_inline_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, SERVICE_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &["os.getenv(", "os.environ[", "settings.", "config.get("];
    let count = PATTERNS.iter().filter(|p| body.contains(**p)).count();
    if count >= 2 {
        let line = PATTERNS
            .iter()
            .find_map(|p| find_line(body, p, function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "service_reads_settings_inline_instead_of_injected",
            Severity::Info, file, function, line,
            "reads configuration inline; inject configuration at construction time instead",
        )];
    }
    Vec::new()
}

pub(super) fn handler_direct_file_io_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, VIEW_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "open(", "os.rename(", "shutil.copy(", "shutil.move(",
        "os.remove(", "Path(", ".write_text(", ".read_text(",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "view_or_handler_performs_direct_file_system_io",
                Severity::Info, file, function, line,
                "performs direct filesystem I/O; delegate to a storage service",
            )];
        }
    }
    Vec::new()
}

pub(super) fn business_logic_in_middleware_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, &["middleware"]) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "pricing", "discount", "feature_flag", "charge(", "invoice",
        "billing", "workflow(", "business_rule",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "business_logic_inside_middleware",
                Severity::Warning, file, function, line,
                "implements business logic inside middleware; use a service instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn di_bypassed_singleton_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, ALL_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "_instance.", "_singleton.", "GlobalClient.", "global_client.",
        "MODULE_LEVEL_DB.", "APP_DB.",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "dependency_injection_bypassed_via_global_singleton",
                Severity::Info, file, function, line,
                "uses a module-level singleton directly; prefer constructor injection",
            )];
        }
    }
    Vec::new()
}

pub(super) fn auth_duplicated_across_views_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, VIEW_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "request.user", "g.user", "get_current_user()", "request.state.user",
    ];
    let count: usize = PATTERNS.iter().filter(|p| body.contains(**p)).count();
    if count >= 2 {
        let line = PATTERNS
            .iter()
            .find_map(|p| find_line(body, p, function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "auth_extraction_duplicated_across_views",
            Severity::Info, file, function, line,
            "duplicates auth extraction; centralize in middleware or a shared dependency",
        )];
    }
    Vec::new()
}

pub(super) fn background_job_uses_request_context_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_task = sig.contains("@celery.task") || sig.contains("@app.task")
        || sig.contains("@shared_task") || sig.contains("@dramatiq.actor");
    if !is_task {
        return Vec::new();
    }
    const PATTERNS: &[&str] = &["flask.g", "request.state", "django.http.request", "from flask import g"];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "background_job_depends_on_request_context_object",
                Severity::Warning, file, function, line,
                "references a request context object inside a background task",
            )];
        }
    }
    Vec::new()
}

pub(super) fn repository_accepts_pydantic_schema_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, REPO_ROLES) {
        return Vec::new();
    }
    let sig = &function.signature_text;
    const PATTERNS: &[&str] = &["CreateRequest", "UpdateRequest", "RequestBody", "Schema,", ": Schema)"];
    for p in PATTERNS {
        if sig.contains(p) {
            return vec![make_finding(
                "repository_method_accepts_pydantic_request_schema",
                Severity::Info, file, function, function.fingerprint.start_line,
                "accepts a request schema directly; use domain-level inputs in repositories",
            )];
        }
    }
    Vec::new()
}

pub(super) fn celery_task_imports_web_app_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let is_task = sig.contains("@celery") || sig.contains("@app.task") || sig.contains("@shared_task");
    if !is_task {
        return Vec::new();
    }
    let imports: Vec<&str> = file.imports.iter().map(|i| i.path.as_str()).collect();
    const WEB_MODS: &[&str] = &["flask", "fastapi", "django.wsgi", "wsgi"];
    for m in WEB_MODS {
        if imports.iter().any(|i| i.starts_with(m)) {
            return vec![Finding {
                rule_id: "celery_or_rq_task_imports_web_framework_app".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line,
                end_line: function.fingerprint.start_line,
                message: format!(
                    "task {} imports web framework application object; use app factory or separate config",
                    function.fingerprint.name
                ),
                evidence: vec![format!("web_import={m}")],
            }];
        }
    }
    Vec::new()
}

pub(super) fn persistent_model_transport_field_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, &["model", "models"]) {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &["status_code", "http_method", "response_body", "request_id ="];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "persistent_model_field_encodes_transport_concern",
                Severity::Info, file, function, line,
                "ORM model contains a transport-concern field; move it to the transport layer",
            )];
        }
    }
    Vec::new()
}

pub(super) fn orm_model_mixes_logic_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, &["model", "models"]) {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_orm = body.contains("Column(") || body.contains("Field(") || body.contains("relationship(");
    let has_business = body.contains("def calculate_") || body.contains("def apply_")
        || body.contains("def process_") || body.contains("def validate_business");
    let has_api = body.contains("def to_json") || body.contains("def serialize") || body.contains("def to_response");
    if has_orm && has_business && has_api {
        return vec![make_finding(
            "orm_model_mixes_domain_logic_and_persistence_mapping",
            Severity::Info, file, function, function.fingerprint.start_line,
            "ORM model mixes domain logic, persistence mapping, and API serialization",
        )];
    }
    Vec::new()
}

pub(super) fn validation_duplicated_dto_domain_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_role_file(file, ALL_ROLES) {
        return Vec::new();
    }
    let body = &function.body_text;
    const SCHEMA_PATTERNS: &[&str] = &["validator(", "@validator", "@field_validator", "model_validator"];
    const DOMAIN_PATTERNS: &[&str] = &["if not value", "raise ValueError", "raise ValidationError"];
    let has_schema = SCHEMA_PATTERNS.iter().any(|p| body.contains(p));
    let has_domain = DOMAIN_PATTERNS.iter().any(|p| body.contains(p));
    if has_schema && has_domain {
        let line = SCHEMA_PATTERNS
            .iter()
            .find_map(|p| find_line(body, p, function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "validation_rules_duplicated_at_dto_and_domain_layer",
            Severity::Info, file, function, line,
            "validates the same field in both the request schema and domain layer",
        )];
    }
    Vec::new()
}

// ── Section 2 · Async and Concurrency Correctness ────────────────────────────

pub(super) fn gather_without_return_exceptions_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("asyncio.gather(") && !body.contains("return_exceptions=True") {
        let line = find_line(body, "asyncio.gather(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "asyncio_gather_without_return_exceptions_on_partial_failure_path",
            Severity::Info, file, function, line,
            "uses asyncio.gather() without return_exceptions=True; individual task failures will propagate",
        )];
    }
    Vec::new()
}

pub(super) fn thread_local_in_async_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    if !python.is_async {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("threading.local()") || body.contains("_local.") {
        let line = find_line(body, "threading.local", function.fingerprint.start_line)
            .or_else(|| find_line(body, "_local.", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "thread_local_storage_read_from_async_function",
            Severity::Warning, file, function, line,
            "reads thread-local storage inside an async function; thread identity is not stable across await points",
        )];
    }
    Vec::new()
}

pub(super) fn run_until_complete_in_running_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    let body = &function.body_text;
    if python.is_async && body.contains("loop.run_until_complete(") {
        let line = find_line(body, "loop.run_until_complete(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "loop_run_until_complete_inside_running_loop",
            Severity::Warning, file, function, line,
            "calls loop.run_until_complete() inside an already running loop",
        )];
    }
    Vec::new()
}

pub(super) fn sleep_zero_busy_wait_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if (body.contains("asyncio.sleep(0)") || body.contains("await asyncio.sleep(0)"))
        && (body.contains("while ") || body.contains("for "))
    {
        let line = find_line(body, "asyncio.sleep(0)", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "asyncio_sleep_zero_busy_wait_pattern",
            Severity::Info, file, function, line,
            "uses asyncio.sleep(0) as a busy-wait yield; use asyncio.Event or a condition variable",
        )];
    }
    Vec::new()
}

pub(super) fn non_daemon_thread_in_server_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_framework = file.imports.iter().any(|i| {
        i.path.starts_with("flask") || i.path.starts_with("fastapi")
            || i.path.starts_with("django")
    });
    if !has_framework {
        return Vec::new();
    }
    if body.contains("threading.Thread(") && !body.contains("daemon=True") {
        let line = find_line(body, "threading.Thread(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "threading_thread_without_daemon_true_in_server_code",
            Severity::Info, file, function, line,
            "creates a non-daemon thread in server code; set daemon=True to avoid blocking shutdown",
        )];
    }
    Vec::new()
}

pub(super) fn shared_mutable_mutated_across_threads_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_thread = body.contains("threading.Thread(") || body.contains("Thread(target=");
    if !has_thread {
        return Vec::new();
    }
    const PATTERNS: &[&str] = &[".append(", ".update(", ".pop(", "[key] =", ".setdefault("];
    let has_mutation = PATTERNS.iter().any(|p| body.contains(p));
    let has_lock = body.contains("threading.Lock()") || body.contains("with lock:");
    if has_mutation && !has_lock {
        let line = PATTERNS
            .iter()
            .find_map(|p| find_line(body, p, function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "shared_mutable_collection_mutated_across_threads_without_lock",
            Severity::Warning, file, function, line,
            "mutates a shared collection from spawned threads without a visible lock",
        )];
    }
    Vec::new()
}

pub(super) fn multiprocessing_pool_not_closed_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("multiprocessing.Pool(") && !body.contains("with multiprocessing.Pool(")
        && !(body.contains(".terminate()") || body.contains(".close()"))
    {
        let line = find_line(body, "multiprocessing.Pool(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "multiprocessing_pool_created_without_context_manager_or_terminate",
            Severity::Warning, file, function, line,
            "creates multiprocessing.Pool without a context manager or explicit terminate/close",
        )];
    }
    Vec::new()
}

pub(super) fn executor_not_shut_down_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_executor = body.contains("ThreadPoolExecutor(") || body.contains("ProcessPoolExecutor(");
    let has_cleanup = body.contains("with ThreadPoolExecutor") || body.contains("with ProcessPoolExecutor")
        || body.contains(".shutdown(");
    if has_executor && !has_cleanup {
        let line = find_line(body, "Executor(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "concurrent_futures_executor_not_shut_down",
            Severity::Warning, file, function, line,
            "creates a futures Executor without context manager or .shutdown(wait=True)",
        )];
    }
    Vec::new()
}

#[allow(dead_code)]
pub(super) fn get_event_loop_at_module_scope_findings(
    file: &ParsedFile,
    _function: &ParsedFunction,
) -> Vec<Finding> {
    // File-level check: module scope usage
    let _ = file;
    Vec::new() // handled at file level
}

pub(super) fn blocking_lock_in_async_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    if !python.is_async {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "threading.Lock().acquire()", "threading.RLock().acquire()",
        ".acquire(blocking=True)", ".acquire(True)", "_lock.acquire()",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "threading_lock_acquired_blocking_inside_async_def",
                Severity::Warning, file, function, line,
                "acquires a threading.Lock inside an async function; use asyncio.Lock instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn asyncio_queue_no_maxsize_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if (body.contains("asyncio.Queue()") || body.contains("asyncio.LifoQueue()"))
        && !body.contains("maxsize=")
    {
        let line = find_line(body, "asyncio.Queue()", function.fingerprint.start_line)
            .or_else(|| find_line(body, "asyncio.LifoQueue()", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "asyncio_queue_created_without_maxsize_in_producer_path",
            Severity::Info, file, function, line,
            "creates asyncio.Queue without maxsize; unbounded queues can cause memory growth",
        )];
    }
    Vec::new()
}

pub(super) fn coroutine_result_discarded_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Look for calls to async helpers without await
    // Simple signal: assignment without await when body has async defs
    if body.contains("async def ") && body.contains("= coro(") && !body.contains("await coro(") {
        let line = find_line(body, "= coro(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "coroutine_result_discarded_without_await",
            Severity::Warning, file, function, line,
            "coroutine result assigned without await; the coroutine will not execute",
        )];
    }
    Vec::new()
}

pub(super) fn sync_called_from_async_without_executor_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    if !python.is_async {
        return Vec::new();
    }
    let body = &function.body_text;
    const BLOCKING: &[&str] = &[
        "time.sleep(", "requests.get(", "requests.post(", "subprocess.run(",
        "urllib.request.urlopen(",
    ];
    let has_executor = body.contains("run_in_executor") || body.contains("loop.run_in_executor");
    if has_executor {
        return Vec::new();
    }
    for p in BLOCKING {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "sync_function_called_from_async_without_executor",
                Severity::Warning, file, function, line,
                "calls a blocking function inside async def without run_in_executor",
            )];
        }
    }
    Vec::new()
}

pub(super) fn untracked_create_task_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Detect create_task result not saved
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("asyncio.create_task(")
            && !trimmed.starts_with("task ")
            && !trimmed.starts_with("_ =")
            && !trimmed.contains(" = asyncio.create_task(")
        {
            return vec![make_finding(
                "untracked_create_task_result_may_hide_exception",
                Severity::Warning, file, function,
                function.fingerprint.start_line + i,
                "discards asyncio.create_task() result; save the reference to observe exceptions",
            )];
        }
    }
    Vec::new()
}

pub(super) fn semaphore_without_async_with_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("semaphore.acquire()") && !body.contains("async with semaphore") {
        let line = find_line(body, "semaphore.acquire()", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "semaphore_acquired_without_async_with_context_manager",
            Severity::Warning, file, function, line,
            "acquires semaphore manually; use `async with semaphore` to ensure release on exception",
        )];
    }
    Vec::new()
}

/// File-level check: asyncio.get_event_loop at module scope
pub(super) fn event_loop_at_module_scope_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }
    for call in &file.module_scope_calls {
        if call.text.contains("asyncio.get_event_loop()")
            || call.text.contains("asyncio.get_running_loop()")
        {
            return vec![Finding {
                rule_id: "asyncio_get_event_loop_at_module_scope".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: None,
                start_line: call.line,
                end_line: call.line,
                message: "asyncio event loop obtained at module scope; call inside an async entry point".to_string(),
                evidence: vec!["pattern=module_scope_get_event_loop".to_string()],
            }];
        }
    }
    Vec::new()
}
