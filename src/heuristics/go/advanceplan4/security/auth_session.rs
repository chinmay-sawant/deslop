use super::*;

pub(super) fn auth_session_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(cookie_no_secure(file, function, lines));
    findings.extend(cookie_no_httponly(file, function, lines));
    findings.extend(cookie_no_samesite(file, function, lines));
    findings.extend(cors_allow_all(file, function, lines));
    findings.extend(jwt_secret_in_source(file, function, lines));
    findings.extend(timing_attack_compare(file, function, lines));
    findings.extend(auth_missing_rate_limit(file, function, lines));
    findings.extend(password_plaintext_storage(file, function, lines));
    findings
}

fn cookie_no_secure(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("http.Cookie{") {
            let cookie_name_lower = bl.text.to_lowercase();
            if cookie_name_lower.contains("session")
                || cookie_name_lower.contains("auth")
                || cookie_name_lower.contains("token")
            {
                let has_secure = lines.iter().any(|l| {
                    l.text.contains("Secure:")
                        && l.text.contains("true")
                        && l.line >= bl.line
                        && l.line <= bl.line + 10
                });
                if !has_secure {
                    findings.push(Finding {
                        rule_id: "cookie_without_secure_flag".into(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} sets cookie without Secure flag",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("cookie at line {}", bl.line),
                            "without Secure, cookies sent over plain HTTP".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

fn cookie_no_httponly(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("http.Cookie{") {
            let cookie_name_lower = bl.text.to_lowercase();
            if cookie_name_lower.contains("session")
                || cookie_name_lower.contains("auth")
                || cookie_name_lower.contains("token")
            {
                let has_httponly = lines.iter().any(|l| {
                    l.text.contains("HttpOnly:")
                        && l.text.contains("true")
                        && l.line >= bl.line
                        && l.line <= bl.line + 10
                });
                if !has_httponly {
                    findings.push(Finding {
                        rule_id: "cookie_without_httponly".into(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} sets cookie without HttpOnly flag",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("cookie at line {}", bl.line),
                            "JavaScript can steal session via document.cookie".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

fn cookie_no_samesite(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("http.Cookie{") {
            let cookie_name_lower = bl.text.to_lowercase();
            if cookie_name_lower.contains("session")
                || cookie_name_lower.contains("auth")
                || cookie_name_lower.contains("token")
            {
                let has_samesite = lines.iter().any(|l| {
                    l.text.contains("SameSite:") && l.line >= bl.line && l.line <= bl.line + 10
                });
                if !has_samesite {
                    findings.push(Finding {
                        rule_id: "cookie_without_samesite".into(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} sets an auth cookie without SameSite protection",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("cookie at line {}", bl.line),
                            "SameSite reduces cross-site request abuse on cookie-backed sessions"
                                .into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

fn cors_allow_all(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if (bl.text.contains("AllowAllOrigins") && bl.text.contains("true"))
            || (bl.text.contains("Access-Control-Allow-Origin") && bl.text.contains("*"))
        {
            findings.push(Finding {
                rule_id: "cors_allow_all_origins".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} allows all CORS origins",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("wildcard CORS at line {}", bl.line),
                    "any site can make authenticated requests".into(),
                ],
            });
        }
    }
    findings
}

fn timing_attack_compare(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let name_lower = function.fingerprint.name.to_lowercase();
    let is_auth = [
        "auth", "token", "verify", "validate", "check", "hmac", "apikey",
    ]
    .iter()
    .any(|k| name_lower.contains(k));
    if !is_auth {
        return findings;
    }
    for bl in lines {
        if (bl.text.contains("==") || bl.text.contains("bytes.Equal("))
            && !bl.text.contains("subtle.")
            && !bl.text.contains("hmac.Equal(")
        {
            let has_token_var = ["token", "secret", "key", "hmac", "hash", "signature"]
                .iter()
                .any(|k| bl.text.to_lowercase().contains(k));
            if has_token_var {
                findings.push(Finding {
                    rule_id: "timing_attack_on_token_comparison".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} compares tokens with non-constant-time equality",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("timing-vulnerable comparison at line {}", bl.line),
                        "use subtle.ConstantTimeCompare or hmac.Equal".into(),
                    ],
                });
            }
        }
    }
    findings
}

fn jwt_secret_in_source(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        let lower = bl.text.to_lowercase();
        if lower.contains("jwt")
            && lower.contains("secret")
            && (bl.text.contains(":=") || bl.text.contains(" = "))
            && bl.text.contains('"')
        {
            findings.push(Finding {
                rule_id: "jwt_secret_in_source".into(),
                severity: Severity::Error,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} hardcodes a JWT secret in source",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("literal JWT secret at line {}", bl.line),
                    "hardcoded signing secrets are extractable from source control and binaries"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn auth_missing_rate_limit(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    let name_lower = function.fingerprint.name.to_lowercase();
    let is_auth_endpoint = ["login", "signin", "token", "auth", "password"]
        .iter()
        .any(|needle| name_lower.contains(needle));
    if !is_auth_endpoint {
        return findings;
    }
    let has_rate_limit = lines.iter().any(|line| {
        line.text.contains("limiter")
            || line.text.contains("rate.")
            || line.text.contains(".Allow()")
            || line.text.contains("Throttle")
    });
    if !has_rate_limit {
        findings.push(Finding {
            rule_id: "missing_rate_limiting_on_auth_endpoint".into(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "auth-style handler {} has no visible rate limiting guard",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "request handler {} appears to handle authentication",
                    function.fingerprint.name
                ),
                "login and token endpoints should apply rate limiting to slow brute-force attacks"
                    .into(),
            ],
        });
    }
    findings
}

fn password_plaintext_storage(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_hash = lines.iter().any(|line| {
        line.text.contains("bcrypt.GenerateFromPassword(")
            || line.text.contains("argon2.")
            || line.text.contains("scrypt.")
    });
    if has_hash {
        return findings;
    }
    for bl in lines {
        let lower = bl.text.to_lowercase();
        if lower.contains("password: password")
            || lower.contains("\"password\": password")
            || lower.contains("user.password = password")
        {
            findings.push(Finding {
                rule_id: "password_stored_as_plaintext".into(),
                severity: Severity::Error,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} appears to persist a password without hashing",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("plaintext password assignment at line {}", bl.line),
                    "passwords must be hashed before storage; never persist raw credentials".into(),
                ],
            });
        }
    }
    findings
}

// ── Section D — Concurrency Security ──
