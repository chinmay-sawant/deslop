use super::*;

pub(super) fn network_tls_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(http_handler_without_csrf(file, function, lines));
    findings.extend(http_handler_missing_security_headers(file, function, lines));
    findings.extend(http_listen_non_tls(file, function, lines));
    findings.extend(dns_lookup_for_access_control(file, function, lines));
    findings.extend(grpc_without_tls(file, function, lines));
    findings.extend(ssh_host_key_insecure(file, function, lines));
    findings.extend(websocket_no_origin(file, function, lines));
    findings.extend(smtp_plaintext_auth(file, function, lines));
    findings
}

fn http_handler_without_csrf(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    let name_lower = function.fingerprint.name.to_lowercase();
    let state_changing = ["post", "create", "update", "delete", "transfer", "login"]
        .iter()
        .any(|needle| name_lower.contains(needle));
    let has_csrf = lines.iter().any(|line| {
        let lower = line.text.to_lowercase();
        lower.contains("csrf") || lower.contains("nosurf") || line.text.contains("VerifyToken")
    });
    if state_changing && !has_csrf {
        findings.push(Finding {
            rule_id: "http_handler_without_csrf_protection".into(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "handler {} changes state without visible CSRF protection",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("state-changing handler {} has no CSRF signal", function.fingerprint.name),
                "cookie-authenticated POST-style endpoints should verify a CSRF token or use middleware"
                    .into(),
            ],
        });
    }
    findings
}

fn http_handler_missing_security_headers(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    let serves_html = lines.iter().any(|line| {
        line.text.contains("<html")
            || line.text.contains("text/html")
            || line.text.contains("template.Execute")
    });
    if !serves_html {
        return findings;
    }
    let has_headers = lines.iter().any(|line| {
        line.text.contains("X-Content-Type-Options")
            || line.text.contains("X-Frame-Options")
            || line.text.contains("Content-Security-Policy")
    });
    if !has_headers {
        findings.push(Finding {
            rule_id: "http_handler_missing_security_headers".into(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "HTML-serving handler {} sets no visible security headers",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("HTML response path in {}", function.fingerprint.name),
                "set headers such as nosniff, frame protection, and CSP or rely on a security middleware"
                    .into(),
            ],
        });
    }
    findings
}

fn http_listen_non_tls(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("http.ListenAndServe(")
            && !bl.text.contains("localhost")
            && !bl.text.contains("127.0.0.1")
        {
            findings.push(Finding {
                rule_id: "http_listen_non_tls".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} starts an HTTP listener without TLS",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("http.ListenAndServe at line {}", bl.line),
                    "production listeners should terminate TLS directly or sit behind a trusted TLS proxy"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn dns_lookup_for_access_control(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let name_lower = function.fingerprint.name.to_lowercase();
    for bl in lines {
        let lower = bl.text.to_lowercase();
        if (bl.text.contains("net.LookupHost(") || bl.text.contains("net.LookupAddr("))
            && (lower.contains("allow")
                || lower.contains("deny")
                || lower.contains("trusted")
                || lower.contains("admin")
                || name_lower.contains("allow")
                || name_lower.contains("deny")
                || name_lower.contains("trust")
                || name_lower.contains("admin"))
        {
            findings.push(Finding {
                rule_id: "dns_lookup_for_access_control".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} appears to use DNS lookups for an access-control decision",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("DNS-based allow/deny logic at line {}", bl.line),
                    "DNS answers can be spoofed or rebound; enforce access control with validated IPs instead"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn grpc_without_tls(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("grpc.WithInsecure()") || bl.text.contains("insecure.NewCredentials()")
        {
            findings.push(Finding {
                rule_id: "grpc_without_tls_credentials".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses gRPC without TLS",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("insecure gRPC at line {}", bl.line),
                    "all RPC traffic is plaintext".into(),
                ],
            });
        }
    }
    findings
}

fn ssh_host_key_insecure(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("InsecureIgnoreHostKey()") {
            findings.push(Finding {
                rule_id: "ssh_host_key_callback_insecure".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} ignores SSH host key verification",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("InsecureIgnoreHostKey at line {}", bl.line),
                    "enables MITM attacks on SSH connections".into(),
                ],
            });
        }
    }
    findings
}

fn websocket_no_origin(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("CheckOrigin") && bl.text.contains("return true") {
            findings.push(Finding {
                rule_id: "websocket_without_origin_check".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} accepts WebSocket connections from any origin",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("open CheckOrigin at line {}", bl.line),
                    "cross-site WebSocket hijacking".into(),
                ],
            });
        }
    }
    findings
}

fn smtp_plaintext_auth(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_plain_auth = lines
        .iter()
        .any(|line| line.text.contains("smtp.PlainAuth("));
    if !has_plain_auth {
        return findings;
    }
    for bl in lines {
        if bl.text.contains("smtp.SendMail(")
            && (bl.text.contains(":25") || bl.text.contains("mail.example.com"))
            && !lines.iter().any(|line| line.text.contains("StartTLS"))
        {
            findings.push(Finding {
                rule_id: "smtp_plaintext_auth".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses SMTP PLAIN auth without visible TLS",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("smtp.SendMail over plaintext endpoint at line {}", bl.line),
                    "PLAIN auth should only be used over STARTTLS or an already-secure transport"
                        .into(),
                ],
            });
        }
    }
    findings
}

// ── Section F — Data Exposure And Logging ──
