/// boundaries.rs — Section 6 (security boundaries, 20 rules) +
///                 Section 7 (memory and resource management, 15 rules) +
///                 Section 8 (configuration and secrets hygiene, 15 rules)
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

// ── Section 6 · Security Boundaries ──────────────────────────────────────────

pub(super) fn sql_string_formatting_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "execute(f\"",
        "execute(\"SELECT",
        "execute(\"INSERT",
        "execute(\"UPDATE",
        "execute(\"DELETE",
        "execute(f'",
        "\"SELECT %s %",
        "f\"SELECT ",
    ];
    for p in PATTERNS {
        if body.contains(p) && !body.contains("?") && !body.contains("%s") {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "sql_query_built_with_string_formatting_instead_of_parameters",
                Severity::Error,
                file,
                function,
                line,
                "builds SQL with string formatting; use parameterized queries to prevent SQL injection",
            )];
        }
    }
    Vec::new()
}

pub(super) fn file_path_without_normalization_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // user input flowing to open/unlink without normalization
    let has_user_path =
        (body.contains("request.") || body.contains("form.get(") || body.contains("args.get("))
            && (body.contains("open(") || body.contains("unlink(") || body.contains("rmtree("));
    let has_normalize =
        body.contains("normpath") || body.contains(".resolve()") || body.contains("commonpath");
    if has_user_path && !has_normalize {
        let line = find_line(body, "open(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "file_path_from_user_input_without_normalization_or_anchor_check",
            Severity::Error,
            file,
            function,
            line,
            "uses user-controlled file path without normpath/resolve check; path traversal risk",
        )];
    }
    Vec::new()
}

pub(super) fn xml_external_entity_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "etree.parse(",
        "ElementTree.parse(",
        "lxml.etree.fromstring(",
        "XMLParser(",
    ];
    let has_xml_parse = PATTERNS.iter().any(|p| body.contains(p));
    let has_safeguard = body.contains("resolve_entities=False")
        || body.contains("no_network=True")
        || body.contains("defusedxml");
    if has_xml_parse && !has_safeguard {
        let line = PATTERNS
            .iter()
            .find_map(|p| find_line(body, p, function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "xml_parsing_with_external_dtd_or_entity_processing_enabled",
            Severity::Error,
            file,
            function,
            line,
            "parses XML without disabling external entity processing; XXE attack risk",
        )];
    }
    Vec::new()
}

pub(super) fn http_url_from_user_input_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // User input flowing into HTTP call without allowlist
    let has_user_url = body.contains("request.")
        && (body.contains("requests.get(url") || body.contains("httpx.get(url"));
    let has_allowlist = body.contains("allowlist")
        || body.contains("trusted_hosts")
        || body.contains("startswith(\"https://");
    if has_user_url && !has_allowlist {
        let line = find_line(body, "requests.get(url", function.fingerprint.start_line)
            .or_else(|| find_line(body, "httpx.get(url", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "http_client_url_built_from_user_input_without_allowlist",
            Severity::Error,
            file,
            function,
            line,
            "builds HTTP client URL from user input without allowlist; SSRF risk",
        )];
    }
    Vec::new()
}

pub(super) fn subprocess_shell_true_user_input_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("shell=True") {
        let has_user_input =
            body.contains("request.") || body.contains("args.") || body.contains("input(");
        if has_user_input {
            let line = find_line(body, "shell=True", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "subprocess_invoked_with_shell_true_and_user_derived_input",
                Severity::Error,
                file,
                function,
                line,
                "runs subprocess with shell=True and user-derived input; command injection risk",
            )];
        }
    }
    Vec::new()
}

pub(super) fn jinja2_autoescape_disabled_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("jinja2.Environment(") && !body.contains("autoescape=") {
        let line = find_line(body, "jinja2.Environment(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "jinja2_environment_created_with_autoescape_disabled",
            Severity::Warning,
            file,
            function,
            line,
            "creates Jinja2 Environment without autoescape; XSS risk in rendered templates",
        )];
    }
    if body.contains("Environment(autoescape=False)") {
        let line = find_line(body, "autoescape=False", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "jinja2_environment_created_with_autoescape_disabled",
            Severity::Warning,
            file,
            function,
            line,
            "creates Jinja2 Environment with autoescape disabled; XSS risk",
        )];
    }
    Vec::new()
}

pub(super) fn jwt_decode_none_algorithm_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("jwt.decode(") {
        let has_algorithm = body.contains("algorithms=") || body.contains("algorithm=");
        let has_none = body.contains("\"none\"") || body.contains("'none'");
        if !has_algorithm || has_none {
            let line = find_line(body, "jwt.decode(", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "jwt_decode_allows_none_algorithm_or_no_algorithm_restriction",
                Severity::Error,
                file,
                function,
                line,
                "decodes JWT without algorithm restriction; algorithm confusion attack risk",
            )];
        }
    }
    Vec::new()
}

pub(super) fn insecure_hash_for_security_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const WEAK: &[&str] = &[
        "hashlib.md5(",
        "hashlib.sha1(",
        "hashlib.new('md5",
        "hashlib.new('sha1",
    ];
    const SECURITY_CTX: &[&str] = &["password", "token", "secret", "signature", "verify"];
    for pat in WEAK {
        if body.contains(pat) && SECURITY_CTX.iter().any(|s| body.contains(s)) {
            let line = find_line(body, pat, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "insecure_hash_algorithm_used_for_security_sensitive_purpose",
                Severity::Error,
                file,
                function,
                line,
                "uses weak hash algorithm (MD5/SHA1) for security-sensitive purpose",
            )];
        }
    }
    Vec::new()
}

pub(super) fn pickle_from_external_source_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("pickle.loads(")
        || body.contains("pickle.load(")
        || body.contains("marshal.loads(")
    {
        let has_trusted = body.contains("trusted") || body.contains("internal_only");
        if !has_trusted {
            let line = find_line(body, "pickle.loads(", function.fingerprint.start_line)
                .or_else(|| find_line(body, "pickle.load(", function.fingerprint.start_line))
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "deserialization_from_external_or_user_controlled_source_with_pickle",
                Severity::Error,
                file,
                function,
                line,
                "deserializes with pickle from an external source; arbitrary code execution risk",
            )];
        }
    }
    Vec::new()
}

pub(super) fn debug_endpoint_without_env_guard_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_debug_route = sig.contains("/debug")
        || sig.contains("/admin")
        || sig.contains("/internal")
        || sig.contains("\"/_");
    let has_env_guard =
        body.contains("DEBUG") || body.contains("TESTING") || body.contains("os.getenv");
    if is_debug_route && !has_env_guard {
        return vec![make_finding(
            "debug_or_admin_endpoint_registered_without_environment_guard",
            Severity::Warning,
            file,
            function,
            function.fingerprint.start_line,
            "registers a debug/admin endpoint without an environment guard",
        )];
    }
    Vec::new()
}

pub(super) fn weak_random_for_security_token_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const WEAK_RNG: &[&str] = &["random.random()", "random.choice(", "random.randint("];
    const SECURITY_CTX: &[&str] = &["token", "session", "csrf", "nonce", "key", "secret"];
    for pat in WEAK_RNG {
        if body.contains(pat) && SECURITY_CTX.iter().any(|s| body.contains(s)) {
            let line = find_line(body, pat, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "weak_random_function_used_for_security_token_generation",
                Severity::Error,
                file,
                function,
                line,
                "uses random module for security token generation; use secrets module instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn open_redirect_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_redirect = body.contains("redirect(url")
        || body.contains("redirect(next")
        || body.contains("RedirectResponse(url");
    let has_user_url = body.contains("request.args")
        || body.contains("request.query_params")
        || body.contains(".GET[");
    let has_check = body.contains("same_origin")
        || body.contains("trusted_urls")
        || body.contains("startswith");
    if has_redirect && has_user_url && !has_check {
        let line = find_line(body, "redirect(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "open_redirect_via_user_supplied_url_without_allowlist",
            Severity::Warning,
            file,
            function,
            line,
            "redirects to a user-controlled URL without a same-origin check; open redirect risk",
        )];
    }
    Vec::new()
}

pub(super) fn arbitrary_file_write_user_path_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_write = body.contains("open(path, \"w\"")
        || body.contains("open(dst,")
        || body.contains("shutil.copy(");
    let has_user_path =
        body.contains("request.") || body.contains("form.") || body.contains("args.");
    let has_safe_check = body.contains("normpath")
        || body.contains("resolve()")
        || body.contains("startswith(upload_dir");
    if has_write && has_user_path && !has_safe_check {
        let line = find_line(body, "open(path,", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "arbitrary_file_write_via_user_controlled_path",
            Severity::Error,
            file,
            function,
            line,
            "writes to a file path derived from user input without a safe-destination check",
        )];
    }
    Vec::new()
}

pub(super) fn cors_allow_all_no_guard_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_wildcard = body.contains("allow_origins=[\"*\"]")
        || body.contains("Access-Control-Allow-Origin: *")
        || body.contains("cors_origins=\"*\"");
    let has_guard =
        body.contains("PRODUCTION") || body.contains("ENVIRONMENT") || body.contains("APP_ENV");
    if has_wildcard && !has_guard {
        let line = find_line(body, "[\"*\"]", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "cors_allow_all_origins_set_without_production_environment_check",
            Severity::Warning,
            file,
            function,
            line,
            "sets CORS allow-all-origins without a production environment guard",
        )];
    }
    Vec::new()
}

pub(super) fn ssti_user_input_in_template_source_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_from_string = body.contains("from_string(")
        || body.contains("Template(template_source")
        || body.contains("Template(user_")
        || body.contains("jinja2.Template(request");
    if has_from_string {
        let line = find_line(body, "from_string(", function.fingerprint.start_line)
            .or_else(|| find_line(body, "Template(", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "server_side_template_injection_via_user_input_in_template_source",
            Severity::Error,
            file,
            function,
            line,
            "passes user input as template source string; server-side template injection risk",
        )];
    }
    Vec::new()
}

pub(super) fn regex_catastrophic_backtracking_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &["(a+)+", "(.*)*", "(.+)+", "(\\w+)+", r"(\w+)+", r"(a+)+"];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "regex_pattern_with_catastrophic_backtracking_applied_to_unbounded_input",
                Severity::Warning,
                file,
                function,
                line,
                "regex pattern has nested quantifiers that may cause catastrophic backtracking",
            )];
        }
    }
    Vec::new()
}

pub(super) fn ldap_injection_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_ldap = file.imports.iter().any(|i| i.path.contains("ldap"));
    if !has_ldap {
        return Vec::new();
    }
    let has_user_in_filter = (body.contains("search_filter") || body.contains("ldap_filter"))
        && (body.contains("request.") || body.contains("+ username") || body.contains("f\"("));
    let has_escape = body.contains("escape_filter_chars") || body.contains("escape_dn_chars");
    if has_user_in_filter && !has_escape {
        let line = find_line(body, "search_filter", function.fingerprint.start_line)
            .or_else(|| find_line(body, "ldap_filter", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "ldap_search_filter_built_from_user_input_without_escaping",
            Severity::Error,
            file,
            function,
            line,
            "builds LDAP filter from user input without escaping; LDAP injection risk",
        )];
    }
    Vec::new()
}

pub(super) fn state_changing_endpoint_missing_csrf_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_mutating = sig.contains("methods=[\"POST\"")
        || sig.contains("methods=['POST'")
        || sig.contains("@app.post")
        || sig.contains("@router.post");
    let has_csrf = body.contains("csrf") || sig.contains("csrf") || body.contains("csrf_token");
    let has_browser_auth_context = body.contains("cookie")
        || body.contains("auth")
        || body.contains("token")
        || body.contains("credential")
        || sig.contains("Depends(")
        || sig.contains("Security(");
    if is_mutating && has_browser_auth_context && !has_csrf {
        return vec![make_finding(
            "state_changing_endpoint_missing_csrf_protection",
            Severity::Warning,
            file,
            function,
            function.fingerprint.start_line,
            "POST endpoint lacks visible CSRF protection",
        )];
    }
    Vec::new()
}

/// File-level: hardcoded secrets in test fixtures or seeds
pub(super) fn hardcoded_secret_in_fixture_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if !file.is_test_file {
        return Vec::new();
    }
    const SECRET_NAMES: &[&str] = &["api_key", "api_secret", "secret_key", "password", "token"];
    for binding in &file.top_level_bindings {
        let name_lower = binding.name.to_lowercase();
        if SECRET_NAMES.iter().any(|k| name_lower == *k) {
            let val = &binding.value_text;
            // Only flag string literals that look like real credentials
            if (val.starts_with('"') || val.starts_with('\''))
                && (val.contains("secret") || val.contains("eyJ") || val.len() > 20)
            {
                return vec![Finding {
                    rule_id: "cryptographic_secret_hardcoded_in_test_fixture_or_seed".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: None,
                    start_line: binding.line,
                    end_line: binding.line,
                    message: "test fixture contains a hardcoded cryptographic secret or credential"
                        .to_string(),
                    evidence: vec![format!("binding={}", binding.name)],
                }];
            }
        }
    }
    Vec::new()
}

// ── Section 7 · Memory and Resource Management ───────────────────────────────

pub(super) fn unbounded_list_accumulation_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_append_in_loop = body.contains("while True") && body.contains(".append(");
    let has_bound = body.contains("maxsize")
        || body.contains("MAX_")
        || body.contains("len(") && body.contains(" >= ");
    if has_append_in_loop && !has_bound {
        let line = find_line(body, ".append(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "unbounded_list_accumulation_inside_long_running_function",
            Severity::Warning,
            file,
            function,
            line,
            "appends to a list in an unbounded loop without an eviction or capacity limit",
        )];
    }
    Vec::new()
}

pub(super) fn generator_consumed_twice_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Simple heuristic: variable used in for and then in list() / next()
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ")
            && trimmed.contains(" in gen_")
            && (body.contains("list(gen_") || body.contains("next(gen_"))
        {
            return vec![make_finding(
                "generator_consumed_twice_without_recreation",
                Severity::Warning,
                file,
                function,
                function.fingerprint.start_line + i,
                "generator consumed in for loop and then again via list()/next(); second pass yields nothing",
            )];
        }
    }
    Vec::new()
}

pub(super) fn file_returned_without_close_path_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // open() without with, and return f or store f
    if body.contains(" = open(") && !body.contains("with open(") {
        let has_close = body.contains(".close()") || body.contains("finally:");
        let has_return_of_file = body.contains("return f") || body.contains("self._file = ");
        if has_return_of_file && !has_close {
            let line = find_line(body, " = open(", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "file_object_returned_or_stored_without_clear_close_path",
                Severity::Warning,
                file,
                function,
                line,
                "opens a file without context manager and returns/stores the handle without a close path",
            )];
        }
    }
    Vec::new()
}

pub(super) fn weakref_without_live_check_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("weakref.ref(") {
        // Look for dereference without check
        if body.contains("ref()") && !body.contains("if ref()") && !body.contains("is None") {
            let line = find_line(body, "weakref.ref(", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "weakref_dereferenced_without_live_check",
                Severity::Warning,
                file,
                function,
                line,
                "dereferences a weakref without checking if the object is still alive",
            )];
        }
    }
    Vec::new()
}

pub(super) fn lru_cache_on_instance_method_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    if (sig.contains("@lru_cache")
        || sig.contains("@functools.lru_cache")
        || sig.contains("@cache"))
        && (function.fingerprint.receiver_type.is_some())
    {
        return vec![make_finding(
            "functools_lru_cache_applied_to_instance_method",
            Severity::Warning,
            file,
            function,
            function.fingerprint.start_line,
            "applies @lru_cache to an instance method; `self` is in the key, keeping instances alive indefinitely",
        )];
    }
    Vec::new()
}

pub(super) fn subprocess_pipe_without_communicate_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("stdout=subprocess.PIPE") && !body.contains(".communicate()") {
        let line = find_line(
            body,
            "stdout=subprocess.PIPE",
            function.fingerprint.start_line,
        )
        .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "subprocess_pipe_without_communicate_for_large_output",
            Severity::Warning,
            file,
            function,
            line,
            "opens subprocess PIPE without .communicate(); large output may deadlock",
        )];
    }
    Vec::new()
}

pub(super) fn socket_without_close_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("socket.socket(") && !body.contains("with socket.socket(") {
        let has_close = body.contains(".close()") || body.contains("finally:");
        if !has_close {
            let line = find_line(body, "socket.socket(", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "socket_opened_without_context_manager_or_guaranteed_close",
                Severity::Warning,
                file,
                function,
                line,
                "opens socket without context manager or finally close(); socket leaks on exception",
            )];
        }
    }
    Vec::new()
}

pub(super) fn deepcopy_in_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_loop = body.contains("for ") || body.contains("while ");
    if has_loop && body.contains("copy.deepcopy(") {
        let line = find_line(body, "copy.deepcopy(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "repeated_deepcopy_in_loop_on_same_source_object",
            Severity::Info,
            file,
            function,
            line,
            "calls copy.deepcopy() inside a loop; hoist the copy before the loop if source is invariant",
        )];
    }
    Vec::new()
}

pub(super) fn redis_commands_in_loop_no_pipeline_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_redis_in_loop = (body.contains("for ") || body.contains("while "))
        && (body.contains("redis.set(")
            || body.contains("redis.get(")
            || body.contains("r.set(")
            || body.contains("r.get("));
    let has_pipeline = body.contains(".pipeline(") || body.contains("with r.pipeline()");
    if has_redis_in_loop && !has_pipeline {
        let line = find_line(body, "redis.set(", function.fingerprint.start_line)
            .or_else(|| find_line(body, "r.set(", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "redis_commands_issued_individually_in_loop_without_pipeline",
            Severity::Info,
            file,
            function,
            line,
            "issues Redis commands individually inside a loop; batch with pipeline() to reduce round trips",
        )];
    }
    Vec::new()
}

pub(super) fn tempfile_without_cleanup_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_cleanup = body.contains("finally:")
        && (body.contains(".rmdir()")
            || body.contains(".unlink(")
            || body.contains("shutil.rmtree")
            || body.contains(".cleanup()"));
    if (body.contains("tempfile.NamedTemporaryFile(") || body.contains("tempfile.mkdtemp("))
        && !body.contains("with tempfile.")
        && !body.contains("delete=True")
        && !body.contains("shutil.rmtree")
        && !has_cleanup
    {
        let line = find_line(body, "tempfile.", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "unclosed_tempfile_or_tmp_directory_from_tempfile_module",
            Severity::Warning,
            file,
            function,
            line,
            "creates tempfile without context manager, delete=True, or cleanup; temp files may persist",
        )];
    }
    Vec::new()
}

pub(super) fn db_pool_exceeds_server_max_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("create_engine(")
        && body.contains("pool_size=")
        && body.contains("max_overflow=")
    {
        // Heuristic: pool_size >= 20 is suspicious
        if body.contains("pool_size=20")
            || body.contains("pool_size=30")
            || body.contains("pool_size=50")
        {
            let line = find_line(body, "pool_size=", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "db_connection_pool_size_exceeds_server_max_connections",
                Severity::Warning,
                file,
                function,
                line,
                "SQLAlchemy pool_size + max_overflow may exceed database max_connections",
            )];
        }
    }
    Vec::new()
}

pub(super) fn closure_captures_large_object_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Large object (DataFrame/model) captured in nested function
    if (((body.contains("df") || body.contains("model") || body.contains("weights"))
        && body.contains("def _callback"))
        || body.contains("lambda:"))
        && (body.contains("pd.DataFrame") || body.contains("torch.") || body.contains("np.array"))
    {
        let line = find_line(body, "def _callback", function.fingerprint.start_line)
            .or_else(|| find_line(body, "lambda:", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "closure_captures_large_object_after_producing_function_returns",
            Severity::Info,
            file,
            function,
            line,
            "closure captures a large object (DataFrame/model); release before construction completes",
        )];
    }
    Vec::new()
}

pub(super) fn heavyweight_object_in_tight_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Check for known expensive initializations inside loops
    let has_loop = body.contains("for ") || body.contains("while ");
    const PATTERNS: &[&str] = &[
        "session = Session(",
        "Session()",
        "create_engine(",
        "httpx.Client()",
        "requests.Session()",
    ];
    if has_loop {
        for p in PATTERNS {
            if body.contains(p) {
                let line = find_line(body, p, function.fingerprint.start_line)
                    .unwrap_or(function.fingerprint.start_line);
                return vec![make_finding(
                    "object_allocated_in_tight_loop_expected_to_be_pooled",
                    Severity::Warning,
                    file,
                    function,
                    line,
                    "creates a heavyweight object inside a loop; hoist allocation outside the loop",
                )];
            }
        }
    }
    Vec::new()
}

// ── Section 8 · Configuration and Secrets Hygiene ────────────────────────────

#[allow(dead_code)]
pub(super) fn dotenv_called_multiple_modules_findings(
    file: &ParsedFile,
    _function: &ParsedFunction,
) -> Vec<Finding> {
    // File-level check: handled in file-level findings
    let _ = file;
    Vec::new()
}

pub(super) fn dotenv_load_dotenv_multi_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }
    let call_count = file
        .module_scope_calls
        .iter()
        .filter(|c| c.name == "load_dotenv" || c.text.contains("load_dotenv("))
        .count();
    if call_count >= 2 {
        return vec![Finding {
            rule_id: "dotenv_load_dotenv_called_from_multiple_modules".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: "load_dotenv() called multiple times in the same file; call once at application entry point".to_string(),
            evidence: vec![format!("call_count={call_count}")],
        }];
    }
    Vec::new()
}

pub(super) fn pydantic_settings_allows_mutation_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if (body.contains("BaseSettings") || body.contains("pydantic_settings"))
        && !body.contains("frozen=True")
        && !body.contains("allow_mutation = False")
    {
        return vec![make_finding(
            "pydantic_settings_model_allows_post_init_mutation",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "pydantic settings model without frozen=True; config can be mutated after initialization",
        )];
    }
    Vec::new()
}

#[allow(dead_code)]
pub(super) fn feature_flag_inline_env_lookup_findings(
    file: &ParsedFile,
    _function: &ParsedFunction,
) -> Vec<Finding> {
    // File-level handled below
    let _ = file;
    Vec::new()
}

pub(super) fn feature_flag_scattered_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }
    let count: usize = file
        .functions
        .iter()
        .map(|f| {
            f.body_text.matches("os.getenv(\"ENABLE_").count()
                + f.body_text.matches("os.environ.get(\"ENABLE_").count()
        })
        .sum();
    if count >= 3 {
        return vec![Finding {
            rule_id: "feature_flag_checked_via_inline_env_lookup_across_handlers".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: "feature flag checked via inline os.getenv in 3+ places; centralize in a feature flag interface".to_string(),
            evidence: vec![format!("inline_flag_lookups={count}")],
        }];
    }
    Vec::new()
}

pub(super) fn secrets_manager_per_call_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "boto3.client('secretsmanager')",
        "SecretManagerServiceClient()",
        "SecretClient(vault_url=",
        "hvac.Client(",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "secrets_manager_client_created_per_function_call",
                Severity::Warning,
                file,
                function,
                line,
                "creates a secrets manager client per function call; create once and reuse",
            )];
        }
    }
    Vec::new()
}

pub(super) fn toml_parsed_on_request_path_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_handler = sig.contains("@app.route")
        || sig.contains("@router.")
        || sig.contains("@app.get")
        || sig.contains("@app.post");
    if is_handler
        && (body.contains("tomllib.load(")
            || body.contains("tomli.load(")
            || body.contains("configparser.read("))
    {
        let line = find_line(body, "tomllib.load(", function.fingerprint.start_line)
            .or_else(|| find_line(body, "configparser.read(", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "toml_or_ini_config_file_parsed_on_request_path",
            Severity::Warning,
            file,
            function,
            line,
            "parses a config file on the request path; parse once at application startup",
        )];
    }
    Vec::new()
}

pub(super) fn startup_log_includes_secret_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &["logging.info(f\"", "logger.info(f\"", "print(f\""];
    const SENSITIVE: &[&str] = &["password", "secret", "token", "key", "credential"];
    for p in PATTERNS {
        if let Some(pos) = body.find(p) {
            let snippet = &body[pos..pos.min(pos + 150)];
            if SENSITIVE.iter().any(|s| snippet.to_lowercase().contains(s)) {
                let line = find_line(body, p, function.fingerprint.start_line)
                    .unwrap_or(function.fingerprint.start_line);
                return vec![make_finding(
                    "startup_log_statement_includes_raw_secret_value",
                    Severity::Error,
                    file,
                    function,
                    line,
                    "log statement interpolates sensitive value; mask or omit secrets from logs",
                )];
            }
        }
    }
    Vec::new()
}

pub(super) fn pydantic_settings_no_forbid_extra_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if (body.contains("BaseSettings") || body.contains("class Settings"))
        && !body.contains("extra = \"forbid\"")
        && !body.contains("extra=\"forbid\"")
        && !body.contains("extra='forbid'")
    {
        return vec![make_finding(
            "pydantic_settings_model_does_not_forbid_extra_fields",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "settings model does not forbid extra fields; unknown config keys are silently ignored",
        )];
    }
    Vec::new()
}

pub(super) fn yaml_unsafe_loader_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("yaml.load(")
        && (body.contains("Loader=yaml.Loader") || body.contains("yaml.full_load("))
    {
        let line = find_line(body, "yaml.load(", function.fingerprint.start_line)
            .or_else(|| find_line(body, "yaml.full_load(", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "yaml_config_loaded_without_safe_loader",
            Severity::Error,
            file,
            function,
            line,
            "loads YAML with Loader=yaml.Loader; use yaml.safe_load() to prevent code execution",
        )];
    }
    Vec::new()
}

pub(super) fn config_validated_lazily_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let is_request_handler =
        sig.contains("@app.") || sig.contains("@router.") || sig.contains("request");
    if is_request_handler
        && (body.contains("os.getenv(\"") && body.contains("if ") && body.contains("raise"))
    {
        let line = find_line(body, "os.getenv(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "application_config_values_validated_lazily_on_first_use",
            Severity::Info,
            file,
            function,
            line,
            "validates config on first use in a request handler; validate at application startup instead",
        )];
    }
    Vec::new()
}

pub(super) fn sensitive_key_in_debug_log_dump_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "logger.debug(\"config: %s\", vars(settings))",
        "logger.debug(config.__dict__)",
        "logger.debug(settings.__dict__)",
        "logger.debug(f\"config: {settings}",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "sensitive_config_key_included_in_debug_level_log_dict_dump",
                Severity::Warning,
                file,
                function,
                line,
                "debug log dumps settings dict which may contain sensitive configuration values",
            )];
        }
    }
    Vec::new()
}

pub(super) fn same_config_different_defaults_findings(
    file: &ParsedFile,
    _function: &ParsedFunction,
) -> Vec<Finding> {
    let _ = file;
    Vec::new() // cross-file analysis, placeholder for repo-level rule
}

pub(super) fn multiple_config_sources_no_precedence_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let sources: usize = [
        body.contains("os.getenv("),
        body.contains("argparse."),
        body.contains("tomllib.load(") || body.contains("configparser.read("),
        body.contains("yaml.safe_load("),
    ]
    .iter()
    .filter(|&&b| b)
    .count();
    if sources >= 3 {
        let line = find_line(body, "os.getenv(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "multiple_config_sources_merged_without_documented_precedence_order",
            Severity::Info,
            file,
            function,
            line,
            "merges 3+ config sources without documented precedence; resolution is unpredictable",
        )];
    }
    Vec::new()
}

pub(super) fn pydantic_settings_no_prefix_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("BaseSettings")
        && !body.contains("env_prefix")
        && !body.contains("model_config")
    {
        return vec![make_finding(
            "pydantic_settings_model_missing_env_prefix_isolation",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "BaseSettings model lacks env_prefix; environment variables may collide across services",
        )];
    }
    Vec::new()
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn boundary_param_names(signature: &str) -> Vec<String> {
    let Some(start) = signature.find('(') else {
        return Vec::new();
    };
    let Some(end) = signature.rfind(')') else {
        return Vec::new();
    };

    signature[start + 1..end]
        .split(',')
        .filter_map(|entry| {
            let trimmed = entry.trim().trim_start_matches('*').trim();
            if trimmed.is_empty() || matches!(trimmed, "/" | "*" | "self" | "cls") {
                return None;
            }
            Some(
                trimmed
                    .split('=')
                    .next()
                    .unwrap_or(trimmed)
                    .split(':')
                    .next()
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string(),
            )
        })
        .collect()
}

fn boundaries_file_finding(
    file: &ParsedFile,
    rule_id: &str,
    line: usize,
    severity: Severity,
    message: &str,
    evidence: &str,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: None,
        start_line: line,
        end_line: line,
        message: message.to_string(),
        evidence: vec![evidence.to_string()],
    }
}

pub(super) fn project_agnostic_boundaries_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let sig = function.signature_text.replace('\n', " ");
    let body = &function.body_text;
    let lower_body = body.to_ascii_lowercase();
    let params = boundary_param_names(&sig);
    let public_api = !function.fingerprint.name.starts_with('_');

    if sig.contains("=[]")
        || sig.contains("={}")
        || sig.contains("=set()")
        || sig.contains("=list()")
        || sig.contains("=dict()")
    {
        findings.push(make_finding(
            "mutable_default_argument_leaks_state_across_calls",
            Severity::Warning,
            file,
            function,
            function.fingerprint.start_line,
            "uses a mutable default argument and risks leaking state across calls",
        ));
    }

    if public_api
        && contains_any(
            body,
            &[
                "return self._",
                "return self.cache",
                "return self.items",
                "return self.data",
            ],
        )
        && !contains_any(
            body,
            &[
                "return list(",
                "return dict(",
                "return set(",
                ".copy()",
                "[:]",
                "return self._generate",
                "return self._build",
                "return self._create",
            ],
        )
    {
        let line = find_line(body, "return self.", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(make_finding(
            "helper_returns_live_internal_collection_reference",
            Severity::Info,
            file,
            function,
            line,
            "returns a live internal mutable collection reference",
        ));
    }

    if (body.contains("for ") || body.contains("while "))
        && contains_any(body, &["lambda", "def inner"])
        && !contains_any(body, &["lambda item=", "lambda value=", "def inner(item="])
    {
        findings.push(make_finding(
            "closure_captures_loop_variable_without_binding",
            Severity::Warning,
            file,
            function,
            function.fingerprint.start_line,
            "creates closures inside a loop without binding the current loop value",
        ));
    }

    if public_api {
        for param in &params {
            if contains_any(
                body,
                &[
                    &format!("{param}.append("),
                    &format!("{param}.extend("),
                    &format!("{param}.update("),
                    &format!("{param}.pop("),
                    &format!("{param}.sort("),
                    &format!("{param}.clear("),
                ],
            ) {
                findings.push(make_finding(
                    "public_api_mutates_argument_in_place_without_signal",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line,
                    "mutates a caller-owned argument in place without an explicit boundary contract",
                ));
                break;
            }
        }
    }

    if lower_body.contains("yield ")
        && contains_any(
            body,
            &[
                "yield CACHE",
                "yield cache",
                "yield registry",
                "yield shared_state",
            ],
        )
    {
        findings.push(make_finding(
            "context_manager_yields_global_mutable_resource",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "yields a shared mutable resource from a context manager and blurs ownership",
        ));
    }

    if sig.contains("Mapping[") || sig.contains("MutableMapping") || sig.contains("Sequence[") {
        for param in &params {
            if contains_any(
                body,
                &[
                    &format!("{param}.update("),
                    &format!("{param}.append("),
                    &format!("{param}["),
                ],
            ) {
                findings.push(make_finding(
                    "function_accepts_mapping_protocol_but_mutates_input",
                    Severity::Warning,
                    file,
                    function,
                    function.fingerprint.start_line,
                    "accepts a broad protocol type but mutates the received object",
                ));
                break;
            }
        }
    }

    if sig.contains("Iterator[") || sig.contains("Iterable[") {
        for param in &params {
            if contains_any(
                body,
                &[
                    &format!("list({param})"),
                    &format!("tuple({param})"),
                    &format!("next({param})"),
                ],
            ) && contains_any(
                body,
                &[
                    &format!("for item in {param}"),
                    &format!("for value in {param}"),
                    &format!("return {param}"),
                ],
            ) {
                findings.push(make_finding(
                    "iterator_argument_consumed_then_reused_later",
                    Severity::Warning,
                    file,
                    function,
                    function.fingerprint.start_line,
                    "consumes an iterator argument and later treats it as reusable data",
                ));
                break;
            }
        }
    }

    if public_api
        && contains_any(
            body,
            &[
                "except requests.",
                "except httpx.",
                "except sqlalchemy.",
                "raise requests.",
                "raise httpx.",
            ],
        )
    {
        findings.push(make_finding(
            "public_api_forwards_library_specific_exception_shape",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "forwards library-specific exception shapes directly through a public boundary",
        ));
    }

    if contains_any(body, &["datetime.now()", "datetime.utcnow()", "tzinfo="])
        && contains_any(
            body,
            &[
                "astimezone(",
                "timezone.utc",
                "replace(tzinfo",
                "fromisoformat(",
            ],
        )
    {
        findings.push(make_finding(
            "datetime_boundary_mixes_naive_and_aware_values",
            Severity::Warning,
            file,
            function,
            function.fingerprint.start_line,
            "mixes naive and timezone-aware datetime handling in one boundary",
        ));
    }

    if (contains_any(body, &[".encode()", ".decode()", "open("]))
        && !contains_any(
            body,
            &[
                "encoding=",
                ".encode(\"",
                ".decode(\"",
                ".encode('",
                ".decode('",
            ],
        )
        && !contains_any(
            body,
            &[
                "\"wb\"", "\"rb\"", "\"ab\"", "'wb'", "'rb'", "'ab'",
                "\"w+b\"", "\"r+b\"", "'w+b'", "'r+b'",
                "iter_content", "wave.", "audio", "pcm",
                "StreamingResponse", "BytesIO",
            ],
        )
    {
        findings.push(make_finding(
            "text_bytes_boundary_relies_on_implicit_default_encoding",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "crosses text and bytes boundaries without an explicit encoding contract",
        ));
    }

    let uses_path_directly = body.contains("open(path")
        || body.contains("Path(path")
        || body.contains("os.path.join(path");
    if params.iter().any(|param| param.contains("path"))
        && uses_path_directly
        && !contains_any(
            body,
            &["resolve()", "expanduser()", "normpath(", "absolute("],
        )
    {
        findings.push(make_finding(
            "path_boundary_accepts_unexpanded_or_relative_input_without_normalization",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "accepts a path-like argument without normalizing it before use",
        ));
    }

    if sig.contains("=0") || sig.contains("=''") || sig.contains("=\"\"") {
        findings.push(make_finding(
            "sentinel_default_value_overlaps_valid_business_value",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "uses a sentinel default that could also be a valid business value",
        ));
    }

    if !sig.contains("async def")
        && contains_any(body, &["iscoroutine(", "awaitable", "asyncio.iscoroutine("])
    {
        findings.push(make_finding(
            "sync_api_accepts_coroutine_object_as_regular_value",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "appears to accept coroutine objects in a synchronous API surface",
        ));
    }

    if sig.contains("async def")
        && contains_any(body, &["return iter(", "return map(", "return filter("])
    {
        findings.push(make_finding(
            "async_api_returns_plain_iterator_with_blocking_iteration",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "returns a plain iterator from an async boundary without clarifying blocking iteration semantics",
        ));
    }

    if contains_any(
        body,
        &[
            "return self._cache",
            "return self.cache",
            "return self._memo",
        ],
    ) {
        findings.push(make_finding(
            "property_returns_live_internal_cache_object",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "returns a live internal cache object directly to callers",
        ));
    }

    if contains_any(&lower_body, &["acquire_lock(", "lock.acquire("])
        && contains_any(&lower_body, &["release_lock(", "lock.release("])
        && lower_body.matches("def ").count() == 0
    {
        findings.push(make_finding(
            "lock_acquire_and_release_owned_by_different_callers",
            Severity::Warning,
            file,
            function,
            function.fingerprint.start_line,
            "spreads lock acquire and release responsibility across callers",
        ));
    }

    if contains_any(
        &lower_body,
        &[
            "must call",
            "before calling",
            "after calling",
            "initialize first",
            "load first",
        ],
    ) {
        findings.push(make_finding(
            "helper_requires_caller_to_know_hidden_ordering_constraints",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "depends on hidden call ordering constraints rather than encoding them in the API",
        ));
    }

    findings
}

pub(super) fn project_agnostic_boundaries_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let source = file
        .top_level_bindings
        .iter()
        .map(|binding| binding.value_text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    if source.contains("@dataclass")
        && (source.contains("= []") || source.contains("= {}") || source.contains("= set()"))
        && !source.contains("default_factory")
    {
        findings.push(boundaries_file_finding(
            file,
            "dataclass_mutable_default_without_default_factory",
            1,
            Severity::Warning,
            "dataclass-like module contains a mutable default without default_factory",
            "pattern=mutable_dataclass_default",
        ));
    }

    let cache_bindings = file
        .top_level_bindings
        .iter()
        .filter(|binding| {
            let lower = binding.name.to_ascii_lowercase();
            lower.contains("cache") || lower.contains("memo") || lower.contains("registry")
        })
        .count();
    if cache_bindings > 0
        && !file.functions.iter().any(|function| {
            let lower = function.fingerprint.name.to_ascii_lowercase();
            lower.contains("invalidate") || lower.contains("clear") || lower.contains("reset")
        })
    {
        findings.push(boundaries_file_finding(
            file,
            "module_cache_exposed_without_invalidation_boundary",
            1,
            Severity::Info,
            "module exposes cache-like state without an explicit invalidation boundary",
            &format!("cache_bindings={cache_bindings}"),
        ));
    }

    for binding in &file.top_level_bindings {
        if binding
            .name
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch == '_')
            && contains_any(&binding.value_text, &["[", "{", "set("])
        {
            findings.push(boundaries_file_finding(
                file,
                "module_constant_rebound_after_public_import",
                binding.line,
                Severity::Info,
                "module defines mutable constant-like state at top level",
                &format!("binding={}", binding.name),
            ));
        }
    }

    findings
}
