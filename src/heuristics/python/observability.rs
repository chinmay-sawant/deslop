/// observability.rs — Section 9 (logging & observability, 15 rules) +
///                    Section 10 (module & package design, 15 rules) +
///                    Section 11 (data structure & algorithm choices, 15 rules) +
///                    Section 12 (web API design anti-patterns, 10 rules)
use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

// ── Helpers ───────────────────────────────────────────────────────────────────

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

// ── Section 9 · Logging and Observability ────────────────────────────────────

#[allow(dead_code)]
pub(super) fn logging_basic_config_in_library_findings(
    file: &ParsedFile,
    _function: &ParsedFunction,
) -> Vec<Finding> {
    let _ = file;
    Vec::new() // file-level handled below
}

pub(super) fn logging_basic_config_library_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }
    // Only flag if this is not an application entry point
    let is_entry = file.path.to_string_lossy().contains("__main__")
        || file.path.to_string_lossy().contains("app.py")
        || file.path.to_string_lossy().contains("main.py");
    if !is_entry {
        for call in &file.module_scope_calls {
            if call.text.contains("logging.basicConfig(") || call.text.contains("addHandler(") {
                return vec![Finding {
                    rule_id: "logging_basic_config_called_from_library_package".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: None,
                    start_line: call.line,
                    end_line: call.line,
                    message: "library module calls logging.basicConfig or addHandler at import time; this overrides the host application's log configuration".to_string(),
                    evidence: vec!["scope=library_module_scope".to_string()],
                }];
            }
        }
    }
    Vec::new()
}

pub(super) fn fstring_in_logging_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "logger.debug(f\"",
        "logger.info(f\"",
        "logger.warning(f\"",
        "logging.debug(f\"",
        "logging.info(f\"",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "f_string_evaluated_eagerly_inside_logging_call",
                Severity::Info,
                file,
                function,
                line,
                "evaluates f-string eagerly in logging call; use lazy % formatting or logger.isEnabledFor()",
            )];
        }
    }
    Vec::new()
}

pub(super) fn logger_error_without_exc_info_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut in_except = false;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("except") {
            in_except = true;
        }
        if in_except
            && (trimmed.contains("logger.error(") || trimmed.contains("logger.critical("))
            && !trimmed.contains("exc_info")
            && !trimmed.contains("exception(")
        {
            return vec![make_finding(
                "logger_error_inside_except_without_exc_info",
                Severity::Warning,
                file,
                function,
                function.fingerprint.start_line + i,
                "logs error inside except without exc_info=True; stack trace is lost from output",
            )];
        }
    }
    Vec::new()
}

#[allow(dead_code)]
pub(super) fn logging_level_hardcoded_module_scope_findings(
    file: &ParsedFile,
    _function: &ParsedFunction,
) -> Vec<Finding> {
    let _ = file;
    Vec::new() // file-level below
}

pub(super) fn logging_level_hardcoded_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }
    for call in &file.module_scope_calls {
        if call.text.contains(".setLevel(logging.DEBUG)")
            || call.text.contains(".setLevel(logging.INFO)")
        {
            return vec![Finding {
                rule_id: "logging_set_level_hardcoded_at_module_scope".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: call.line,
                end_line: call.line,
                message:
                    "log level hardcoded at module scope; configure via application-level settings"
                        .to_string(),
                evidence: vec!["scope=module_level_setLevel".to_string()],
            }];
        }
    }
    Vec::new()
}

pub(super) fn trace_span_no_parent_context_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if (body.contains("tracer.start_span(") || body.contains("start_as_current_span("))
        && !body.contains("context.extract")
        && !body.contains("parent=")
        && !body.contains("propagate")
    {
        let line = find_line(body, "start_span(", function.fingerprint.start_line)
            .or_else(|| {
                find_line(
                    body,
                    "start_as_current_span(",
                    function.fingerprint.start_line,
                )
            })
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "distributed_trace_span_created_without_parent_context_propagation",
            Severity::Info,
            file,
            function,
            line,
            "creates a trace span without extracting parent context from the incoming request",
        )];
    }
    Vec::new()
}

pub(super) fn health_check_queries_slow_table_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_health = sig.contains("/health")
        || sig.contains("/healthz")
        || sig.contains("/readiness")
        || function.fingerprint.name.contains("health")
        || function.fingerprint.name.contains("readiness");
    if is_health {
        let has_slow = body.contains(".objects.all()")
            || body.contains(".objects.filter(")
            || body.contains("COUNT(*)")
            || body.contains("session.query(");
        let has_fast =
            body.contains("SELECT 1") || body.contains("ping(") || body.contains("is_active");
        if has_slow && !has_fast {
            let line = find_line(body, ".objects.", function.fingerprint.start_line)
                .or_else(|| find_line(body, "session.query(", function.fingerprint.start_line))
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "health_check_handler_queries_slow_database_table",
                Severity::Warning,
                file,
                function,
                line,
                "health check executes a full ORM query; use a lightweight SELECT 1 probe instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn exception_swallowed_before_sentry_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if (body.contains("sentry_sdk.capture_exception") || body.contains("rollbar.report_exc_info"))
        && body.contains("except ")
        && body.contains("raise ")
    {
        // Look for re-wrapping: except Err as e: raise WrappedErr(e) then capture
        if body.contains("raise wrapped_")
            || body.contains("raise AppError(e)")
            || body.contains("raise ServiceError(e)")
        {
            let line = find_line(body, "capture_exception", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "exception_swallowed_before_sentry_or_error_tracker_capture",
                Severity::Warning,
                file,
                function,
                line,
                "re-wraps exception before capturing to error tracker; original context may be lost",
            )];
        }
    }
    Vec::new()
}

pub(super) fn hot_path_logs_without_sampling_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_loop = body.contains("for ") || body.contains("while ");
    let has_log_in_loop =
        has_loop && (body.contains("logger.info(") || body.contains("logger.debug("));
    let has_sample = body.contains("sample_rate")
        || body.contains("random.random()")
        || body.contains("rate_limit");
    if has_log_in_loop && !has_sample {
        let line = find_line(body, "logger.info(", function.fingerprint.start_line)
            .or_else(|| find_line(body, "logger.debug(", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "high_frequency_code_path_logs_without_sampling_or_rate_limit",
            Severity::Info,
            file,
            function,
            line,
            "logs at info/debug level inside a loop without sampling guard; excessive log volume risk",
        )];
    }
    Vec::new()
}

pub(super) fn otel_span_attaches_pii_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PII_ATTRIBUTES: &[&str] = &[
        "\"user.email\"",
        "\"email\"",
        "\"user.phone\"",
        "\"ip_address\"",
        "\"user.name\"",
        "\"ssn\"",
    ];
    if body.contains("span.set_attribute(") {
        for p in PII_ATTRIBUTES {
            if body.contains(p) {
                let line = find_line(body, "set_attribute(", function.fingerprint.start_line)
                    .unwrap_or(function.fingerprint.start_line);
                return vec![make_finding(
                    "opentelemetry_span_attribute_attaches_pii_fields",
                    Severity::Warning,
                    file,
                    function,
                    line,
                    "attaches PII field to OpenTelemetry span; ensure scrubbing policy covers this attribute",
                )];
            }
        }
    }
    Vec::new()
}

pub(super) fn structured_log_no_trace_id_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_handler = sig.contains("@app.") || sig.contains("@router.") || sig.contains("request");
    if is_handler
        && (body.contains("structlog.") || body.contains("json.dumps({"))
        && !body.contains("trace_id")
        && !body.contains("request_id")
        && !body.contains("correlation_id")
    {
        let line = find_line(body, "structlog.", function.fingerprint.start_line)
            .or_else(|| find_line(body, "json.dumps({", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "structured_log_record_missing_trace_or_correlation_id",
            Severity::Info,
            file,
            function,
            line,
            "structured log record in request handler lacks trace_id/request_id field",
        )];
    }
    Vec::new()
}

pub(super) fn log_in_signal_handler_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let sig = &function.signature_text;
    let is_signal_handler =
        sig.contains("signum") || sig.contains("signal_handler") || body.contains("signal.signal(");
    if is_signal_handler
        && (body.contains("logging.") || body.contains("logger.") || body.contains("print("))
    {
        let line = find_line(body, "logging.", function.fingerprint.start_line)
            .or_else(|| find_line(body, "logger.", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "logging_call_inside_signal_handler_function",
            Severity::Warning,
            file,
            function,
            line,
            "calls logging inside a signal handler; not async-signal-safe, can cause deadlocks",
        )];
    }
    Vec::new()
}

pub(super) fn alert_threshold_hardcoded_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Heuristic: SLO/latency constants with no config reference
    let has_hardcoded = body.contains("if error_rate >")
        || body.contains("if latency_ms >")
        || body.contains("slo_threshold = 0.")
        || body.contains("error_budget =");
    let has_config =
        body.contains("config.") || body.contains("settings.") || body.contains("os.getenv(");
    if has_hardcoded && !has_config {
        let line = find_line(body, "if error_rate >", function.fingerprint.start_line)
            .or_else(|| find_line(body, "if latency_ms >", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "alert_or_slo_threshold_hardcoded_inside_application_logic",
            Severity::Info,
            file,
            function,
            line,
            "hardcodes SLO threshold in application code; manage thresholds via external configuration",
        )];
    }
    Vec::new()
}

pub(super) fn prometheus_metric_in_db_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_db_loop = (body.contains("for ") || body.contains("while "))
        && (body.contains(".fetchall()")
            || body.contains("for row in")
            || body.contains("for record in"));
    let has_metric = body.contains("counter.inc()")
        || body.contains("histogram.observe(")
        || body.contains("statsd.increment(")
        || body.contains(".inc()");
    if has_db_loop && has_metric {
        let line = find_line(body, ".inc()", function.fingerprint.start_line)
            .or_else(|| find_line(body, "histogram.observe(", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "prometheus_or_statsd_metric_emitted_inside_db_result_loop",
            Severity::Info,
            file,
            function,
            line,
            "emits metric inside per-row DB result loop; accumulate and emit once after the loop",
        )];
    }
    Vec::new()
}

pub(super) fn observability_inconsistent_metric_names_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Mixed dot and underscore in metric names
    if body.contains("statsd.") || body.contains("Counter(\"") {
        let has_dot_name = body.contains("Counter(\"app.") || body.contains("statsd.incr(\"app.");
        let has_underscore_name =
            body.contains("Counter(\"app_") || body.contains("statsd.incr(\"app_");
        if has_dot_name && has_underscore_name {
            let line = find_line(body, "Counter(\"", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "observability_metric_names_use_inconsistent_separators",
                Severity::Info,
                file,
                function,
                line,
                "metric names mix dot and underscore separators; standardize on one convention",
            )];
        }
    }
    Vec::new()
}

// ── Section 10 · Module and Package Design ────────────────────────────────────

pub(super) fn star_import_in_production_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let is_init = file.path.file_name().and_then(|n| n.to_str()) == Some("__init__.py");
    if is_init {
        return Vec::new();
    }
    for imp in &file.imports {
        if imp.path.ends_with(".*") || imp.alias == "*" {
            return vec![Finding {
                rule_id: "star_import_used_in_non_init_production_module".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line,
                end_line: function.fingerprint.start_line,
                message: format!(
                    "module uses star import from {}; pollutes namespace and hides dependency origin",
                    imp.path
                ),
                evidence: vec![format!("star_import={}", imp.path)],
            }];
        }
    }
    Vec::new()
}

pub(super) fn importlib_in_request_handler_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_handler = sig.contains("@app.") || sig.contains("@router.") || sig.contains("request");
    if is_handler && body.contains("importlib.import_module(") {
        let line = find_line(
            body,
            "importlib.import_module(",
            function.fingerprint.start_line,
        )
        .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "importlib_import_module_called_inside_request_handler",
            Severity::Warning,
            file,
            function,
            line,
            "calls importlib.import_module inside a request handler; resolve dynamic imports at startup",
        )];
    }
    Vec::new()
}

pub(super) fn optional_import_on_hot_path_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // try/except ImportError with nil check on hot path
    if body.contains("try:") && body.contains("except ImportError") && body.contains(" is None") {
        let sig = &function.signature_text;
        let is_handler =
            sig.contains("@app.") || sig.contains("request") || sig.contains("@router.");
        if is_handler {
            let line = find_line(body, "except ImportError", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "optional_library_import_checked_on_hot_code_path",
                Severity::Info,
                file,
                function,
                line,
                "checks optional import availability on request path; do this once at module initialization",
            )];
        }
    }
    Vec::new()
}

#[allow(dead_code)]
pub(super) fn module_side_effect_outside_main_guard_findings(
    file: &ParsedFile,
    _function: &ParsedFunction,
) -> Vec<Finding> {
    let _ = file;
    Vec::new() // file-level below
}

pub(super) fn module_side_effect_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }
    const SIDE_EFFECT_PATTERNS: &[&str] = &[
        "signal.signal(",
        "threading.Thread(",
        "threading.Timer(",
        "subprocess.Popen(",
    ];
    for call in &file.module_scope_calls {
        for p in SIDE_EFFECT_PATTERNS {
            if call.text.contains(p) {
                return vec![Finding {
                    rule_id: "module_level_side_effect_outside_main_guard".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: None,
                    start_line: call.line,
                    end_line: call.line,
                    message: format!(
                        "module-level side effect ({p}) outside __main__ guard; runs on import"
                    ),
                    evidence: vec![format!("side_effect_pattern={p}")],
                }];
            }
        }
    }
    Vec::new()
}

#[allow(dead_code)]
pub(super) fn test_helpers_in_production_package_findings(
    file: &ParsedFile,
    _function: &ParsedFunction,
) -> Vec<Finding> {
    let _ = file;
    Vec::new() // file-level below
}

pub(super) fn test_helpers_in_production_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }
    let path = file.path.to_string_lossy();
    // Production files named like test helpers
    if (path.ends_with("_test_helpers.py")
        || path.ends_with("_factories.py")
        || path.ends_with("_fakes.py"))
        && !path.contains("/tests/")
        && !path.contains("/test/")
    {
        return vec![Finding {
            rule_id: "test_support_helpers_located_inside_production_package".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: "test helper/factory file is inside the production package directory; move to tests/".to_string(),
            evidence: vec!["pattern=test_helper_in_production_path".to_string()],
        }];
    }
    Vec::new()
}

pub(super) fn dynamic_plugin_no_allowlist_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("importlib.import_module(plugin_")
        || body.contains("importlib.import_module(name")
    {
        let has_allowlist = body.contains("ALLOWED_PLUGINS")
            || body.contains("plugin_registry")
            || body.contains("in allowed_");
        if !has_allowlist {
            let line = find_line(
                body,
                "importlib.import_module(",
                function.fingerprint.start_line,
            )
            .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "dynamic_plugin_loaded_from_config_without_registry_allowlist",
                Severity::Warning,
                file,
                function,
                line,
                "loads plugin dynamically from config without validating against a registry allowlist",
            )];
        }
    }
    Vec::new()
}

pub(super) fn importlib_metadata_in_request_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_handler = sig.contains("@app.") || sig.contains("request") || sig.contains("@router.");
    if is_handler && body.contains("importlib.metadata.version(") {
        let line = find_line(
            body,
            "importlib.metadata.version(",
            function.fingerprint.start_line,
        )
        .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "importlib_metadata_version_queried_inside_request_loop",
            Severity::Info,
            file,
            function,
            line,
            "queries importlib.metadata.version inside request handler; cache the result at startup",
        )];
    }
    Vec::new()
}

pub(super) fn pkg_resources_runtime_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("pkg_resources.get_distribution(") || body.contains("pkg_resources.require(") {
        let line = find_line(
            body,
            "pkg_resources.get_distribution(",
            function.fingerprint.start_line,
        )
        .or_else(|| {
            find_line(
                body,
                "pkg_resources.require(",
                function.fingerprint.start_line,
            )
        })
        .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "pkg_resources_used_for_runtime_version_lookup",
            Severity::Info,
            file,
            function,
            line,
            "uses pkg_resources for version lookup; use importlib.metadata.version() instead",
        )];
    }
    Vec::new()
}

pub(super) fn relative_import_crossing_sibling_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    for imp in &file.imports {
        if imp.path.starts_with("...") {
            return vec![Finding {
                rule_id: "relative_import_crossing_sibling_package_boundary".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line,
                end_line: function.fingerprint.start_line,
                message: format!(
                    "relative import `{}` crosses into a sibling package; use absolute imports",
                    imp.path
                ),
                evidence: vec![format!("import={}", imp.path)],
            }];
        }
    }
    Vec::new()
}

#[allow(dead_code)]
pub(super) fn init_reexports_private_symbols_findings(
    file: &ParsedFile,
    _function: &ParsedFunction,
) -> Vec<Finding> {
    let _ = file;
    Vec::new() // file-level below
}

pub(super) fn init_reexports_private_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.path.file_name().and_then(|n| n.to_str()) != Some("__init__.py") {
        return Vec::new();
    }
    for import in &file.imports {
        // path looks like ".module._private_symbol"; imported_name starts with '_'
        let imported = import.imported_name.as_deref().unwrap_or("");
        if import.path.starts_with('.') && imported.starts_with('_') {
            return vec![Finding {
                rule_id: "init_file_re_exports_private_module_symbols".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: import.line,
                end_line: import.line,
                message: "__init__.py re-exports a symbol with a leading underscore; private symbols should not be part of the public API".to_string(),
                evidence: vec![format!("import={}", import.path)],
            }];
        }
    }
    Vec::new()
}

#[allow(dead_code)]
pub(super) fn public_package_missing_all_findings(
    file: &ParsedFile,
    _function: &ParsedFunction,
) -> Vec<Finding> {
    let _ = file;
    Vec::new() // file-level below
}

pub(super) fn public_package_missing_all_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.path.file_name().and_then(|n| n.to_str()) != Some("__init__.py") {
        return Vec::new();
    }
    // Only flag non-trivial packages
    if file.byte_size < 100 {
        return Vec::new();
    }
    let has_all = file.top_level_bindings.iter().any(|b| b.name == "__all__");
    let has_functions = !file.functions.is_empty();
    if !has_all && has_functions {
        return vec![Finding {
            rule_id: "public_package_missing_all_list".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: "package __init__.py defines public symbols without __all__; public API surface is ambiguous".to_string(),
            evidence: vec!["missing=__all__".to_string()],
        }];
    }
    Vec::new()
}

// ── Section 11 · Data Structure and Algorithm Choices ────────────────────────

pub(super) fn sorted_to_extract_top_n_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if (body.contains("sorted(") && body.contains("[0]") && !body.contains("heapq."))
        || (body.contains("sorted(") && body.contains("[:") && !body.contains("heapq."))
    {
        let line = find_line(body, "sorted(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "sorted_full_collection_to_extract_top_n_elements",
            Severity::Info,
            file,
            function,
            line,
            "sorts full collection to extract top-N; use heapq.nsmallest()/nlargest() instead",
        )];
    }
    Vec::new()
}

pub(super) fn linear_membership_in_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    if !python.list_membership_loop_lines.is_empty() {
        return python.list_membership_loop_lines.iter().map(|&line| make_finding(
            "linear_membership_test_in_loop_over_large_static_list",
            Severity::Info, file, function, line,
            "tests membership in a list inside a loop; convert the list to a set for O(1) lookups",
        )).collect();
    }
    Vec::new()
}

pub(super) fn manual_dict_increment_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("if key in d:") || (body.contains("if ") && body.contains("in counts:")) {
        if body.contains("d[key] += 1") || body.contains("counts[") && body.contains("+= 1") {
            let line = find_line(body, "+= 1", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "manual_dict_increment_instead_of_counter_or_defaultdict",
                Severity::Info,
                file,
                function,
                line,
                "maintains frequency count manually; use collections.Counter or defaultdict(int)",
            )];
        }
    }
    Vec::new()
}

pub(super) fn list_pop_zero_as_queue_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains(".pop(0)") || body.contains(".insert(0,") {
        let line = find_line(body, ".pop(0)", function.fingerprint.start_line)
            .or_else(|| find_line(body, ".insert(0,", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "list_pop_zero_used_as_queue_operation",
            Severity::Info,
            file,
            function,
            line,
            "uses list.pop(0) as a queue; use collections.deque.popleft() for O(1) performance",
        )];
    }
    Vec::new()
}

pub(super) fn zip_range_len_instead_of_enumerate_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("zip(range(len(") {
        let line = find_line(body, "zip(range(len(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "zip_range_len_used_instead_of_enumerate",
            Severity::Info,
            file,
            function,
            line,
            "uses zip(range(len(x)), x); use enumerate(x) instead",
        )];
    }
    Vec::new()
}

pub(super) fn defaultdict_lambda_instead_of_builtin_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "defaultdict(lambda: [])",
        "defaultdict(lambda: {})",
        "defaultdict(lambda: 0)",
        "defaultdict(lambda: set())",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "defaultdict_created_with_lambda_instead_of_builtin_factory",
                Severity::Info,
                file,
                function,
                line,
                "uses lambda default factory; use defaultdict(list)/defaultdict(int)/defaultdict(dict) instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn filter_map_materialized_each_step_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("list(filter(") && body.contains("list(map(") {
        let line = find_line(body, "list(filter(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "filter_and_map_results_materialized_to_list_at_each_step",
            Severity::Info,
            file,
            function,
            line,
            "materializes intermediate list at each filter/map step; use a generator pipeline instead",
        )];
    }
    Vec::new()
}

pub(super) fn frozenset_not_used_for_constant_set_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Constant sets reassigned each call (inside function, not module-level)
    if body.contains("ALLOWED_")
        && body.contains("= {")
        && body.contains("\"")
        && !body.contains("frozenset")
    {
        let line = find_line(body, "ALLOWED_", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "frozenset_not_used_for_constant_membership_set_rebuilt_per_call",
            Severity::Info,
            file,
            function,
            line,
            "constant membership set rebuilt per call; hoist as frozenset at module level",
        )];
    }
    Vec::new()
}

pub(super) fn sorted_list_with_sort_instead_of_bisect_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if (body.contains("list.sort()") || body.contains(".sort()")) && body.contains(".append(") {
        let line = find_line(body, ".sort()", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "sorted_list_maintained_with_insert_instead_of_bisect_insort",
            Severity::Info,
            file,
            function,
            line,
            "re-sorts list after each append; use bisect.insort() to maintain sorted order cheaply",
        )];
    }
    Vec::new()
}

pub(super) fn namedtuple_index_access_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Detect point[0], record[2] patterns on namedtuple-like variable names
    let file_uses_namedtuple = file.imports.iter().any(|i| i.path.contains("namedtuple"))
        || file
            .functions
            .iter()
            .any(|f| f.body_text.contains("namedtuple("));
    if (body.contains("_tuple[") || body.contains("record[") || body.contains("row["))
        && file_uses_namedtuple
    {
        let line = find_line(body, "[0]", function.fingerprint.start_line)
            .or_else(|| find_line(body, "record[", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "namedtuple_fields_accessed_by_integer_index",
            Severity::Info,
            file,
            function,
            line,
            "accesses namedtuple field by integer index; use named field access for clarity",
        )];
    }
    Vec::new()
}

pub(super) fn counter_most_common_all_for_top_one_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains(".most_common()") && body.contains("[0]") {
        let line = find_line(body, ".most_common()", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "counter_most_common_all_items_retrieved_for_top_one",
            Severity::Info,
            file,
            function,
            line,
            "calls Counter.most_common() then [0] to get the top item; use max(counter, key=counter.get)",
        )];
    }
    Vec::new()
}

pub(super) fn repeated_key_hash_in_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let python = function.python_evidence();
    if !python.repeated_subscript_lines.is_empty() {
        return python.repeated_subscript_lines.iter().map(|&line| make_finding(
            "repeated_key_hash_via_dict_lookup_in_tight_loop",
            Severity::Info, file, function, line,
            "accesses the same dict key repeatedly in a loop; cache the value in a local variable",
        )).collect();
    }
    Vec::new()
}

pub(super) fn chain_or_conditions_not_using_in_operator_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Look for `x == "a" or x == "b" or x == "c"` patterns
    if body.contains(" == ") && body.matches(" or ").count() >= 2 {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            let eq_count = trimmed.matches(" == ").count();
            let or_count = trimmed.matches(" or ").count();
            if eq_count >= 3 && or_count >= 2 && !trimmed.contains(" in ") {
                return vec![make_finding(
                    "chain_of_boolean_or_conditions_over_same_value_not_using_in_operator",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "uses chain of == / or conditions; use `x in {\"a\", \"b\", \"c\"}` instead",
                )];
            }
        }
    }
    Vec::new()
}

pub(super) fn ordered_dict_in_python37_plus_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("OrderedDict(") && !body.contains("move_to_end") && !body.contains("last=") {
        let line = find_line(body, "OrderedDict(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "ordered_dict_used_in_python_37_plus_where_dict_suffices",
            Severity::Info,
            file,
            function,
            line,
            "uses OrderedDict where plain dict preserves insertion order in Python 3.7+",
        )];
    }
    Vec::new()
}

// ── Section 12 · Web API Design Anti-patterns ─────────────────────────────────

pub(super) fn api_endpoint_no_response_schema_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_api = sig.contains("@app.") || sig.contains("@router.") || sig.contains("@bp.route");
    if is_api
        && (body.contains("jsonify(")
            || body.contains("JSONResponse(")
            || body.contains("return {"))
        && !sig.contains("response_model=")
        && !sig.contains("-> JSONResponse")
        && !sig.contains("-> dict")
    {
        return vec![make_finding(
            "api_endpoint_returns_json_without_documented_response_schema",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "API endpoint returns JSON without a documented response model or schema annotation",
        )];
    }
    Vec::new()
}

pub(super) fn cursor_pagination_no_tiebreaker_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let is_pagination =
        body.contains("cursor") && (body.contains("limit") || body.contains("next_cursor"));
    let has_tiebreaker =
        body.contains("order_by(") && (body.contains("id") || body.contains("created_at"));
    if is_pagination && !has_tiebreaker {
        let line = find_line(body, "cursor", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "cursor_based_pagination_missing_stable_sort_tiebreaker",
            Severity::Info,
            file,
            function,
            line,
            "cursor-based pagination lacks a stable unique sort tiebreaker; pages may be non-deterministic",
        )];
    }
    Vec::new()
}

pub(super) fn bulk_endpoint_no_partial_failure_contract_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_bulk = function.fingerprint.name.contains("bulk")
        || sig.contains("batch")
        || body.contains("for item in items:") && body.contains(".create(");
    let has_contract =
        body.contains("errors") && body.contains("return") || body.contains("transaction.atomic");
    if is_bulk && !has_contract {
        let line = find_line(body, "for item in items:", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "bulk_endpoint_partial_failure_contract_ambiguous",
            Severity::Info,
            file,
            function,
            line,
            "bulk endpoint does not document or enforce all-or-nothing vs partial-success behavior",
        )];
    }
    Vec::new()
}

pub(super) fn rate_limit_no_retry_after_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("429") || body.contains("TOO_MANY_REQUESTS") {
        let has_retry_after = body.contains("Retry-After") || body.contains("retry_after");
        if !has_retry_after {
            let line = find_line(body, "429", function.fingerprint.start_line)
                .or_else(|| find_line(body, "TOO_MANY_REQUESTS", function.fingerprint.start_line))
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "rate_limit_429_response_missing_retry_after_header_or_stable_body",
                Severity::Info,
                file,
                function,
                line,
                "returns HTTP 429 without a Retry-After header; clients cannot determine backoff interval",
            )];
        }
    }
    Vec::new()
}

pub(super) fn pydantic_validation_error_leaks_aliases_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("ValidationError")
        && (body.contains(".errors()") || body.contains("detail=exc.errors()"))
    {
        let has_mask = body.contains("loc") && body.contains("msg") && body.contains("stripped");
        if !has_mask {
            let line = find_line(body, ".errors()", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "pydantic_validation_error_detail_forwarded_with_internal_field_aliases",
                Severity::Info,
                file,
                function,
                line,
                "forwards raw Pydantic validation error details; may expose internal field aliases to clients",
            )];
        }
    }
    Vec::new()
}

pub(super) fn state_changing_endpoint_returns_200_empty_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_mutating = sig.contains("@app.post")
        || sig.contains("@router.post")
        || sig.contains("@app.put")
        || sig.contains("@router.delete");
    if is_mutating
        && (body.contains("return Response(status_code=200)") || body.contains("return {}, 200"))
    {
        let line = find_line(body, "return Response", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "state_changing_endpoint_returns_200_with_empty_body",
            Severity::Info,
            file,
            function,
            line,
            "state-changing endpoint returns HTTP 200 with empty body; use 201/202/204 with proper body",
        )];
    }
    Vec::new()
}

pub(super) fn binary_response_no_content_type_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let is_binary = body.contains("StreamingResponse(")
        || body.contains("FileResponse(")
        || body.contains("io.BytesIO(");
    let has_content_type = body.contains("media_type=") || body.contains("Content-Type");
    if is_binary && !has_content_type {
        let line = find_line(body, "StreamingResponse(", function.fingerprint.start_line)
            .or_else(|| find_line(body, "FileResponse(", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "binary_or_multipart_response_missing_explicit_content_type",
            Severity::Info,
            file,
            function,
            line,
            "streams binary response without explicit Content-Type; clients rely on content sniffing",
        )];
    }
    Vec::new()
}

pub(super) fn large_response_fully_buffered_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_handler =
        sig.contains("@app.get") || sig.contains("@router.get") || sig.contains("@app.route");
    let has_large_buffer = body.contains(".fetchall()")
        || body.contains("list(queryset)")
        || body.contains("json.dumps(all_");
    let has_streaming =
        body.contains("StreamingResponse") || body.contains("generator") || body.contains("yield");
    if is_handler && has_large_buffer && !has_streaming {
        let line = find_line(body, ".fetchall()", function.fingerprint.start_line)
            .or_else(|| find_line(body, "json.dumps(all_", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "large_response_body_fully_buffered_in_memory_before_send",
            Severity::Info,
            file,
            function,
            line,
            "buffers large response fully in memory; use StreamingResponse or generator-based response",
        )];
    }
    Vec::new()
}

pub(super) fn api_versioning_no_router_group_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let has_version_path = sig.contains("\"/v1/")
        || sig.contains("\"/v2/")
        || sig.contains("'/v1/")
        || body.contains("prefix=\"/v1\"")
        || body.contains("prefix='/v2'");
    let has_router_group = body.contains("APIRouter(prefix")
        || body.contains("Blueprint(")
        || body.contains("include_router(");
    if has_version_path && !has_router_group {
        return vec![make_finding(
            "api_versioning_in_url_without_matching_router_group",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "URL path contains version segment without a matching versioned router group or blueprint",
        )];
    }
    Vec::new()
}

pub(super) fn response_envelope_inconsistent_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Mixing {"data": ...} envelope and bare {"items": ...} in same file
    let has_data_envelope = file
        .functions
        .iter()
        .any(|f| f.body_text.contains("\"data\":") || f.body_text.contains("'data':"));
    let has_bare = file
        .functions
        .iter()
        .any(|f| f.body_text.contains("\"items\":") && !f.body_text.contains("\"data\":"));
    if has_data_envelope && has_bare && body.contains("return ") {
        let line = find_line(body, "return ", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "response_envelope_shape_inconsistent_across_siblings_in_same_router",
            Severity::Info,
            file,
            function,
            line,
            "response envelope shape is inconsistent across sibling endpoints in the same router",
        )];
    }
    Vec::new()
}
