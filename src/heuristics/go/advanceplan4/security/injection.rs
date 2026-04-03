use super::*;

pub(super) fn injection_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(os_exec_user_input(file, function, lines));
    findings.extend(template_html_unescaped(file, function, lines));
    findings.extend(text_template_html(file, function, lines));
    findings.extend(filepath_join_traversal(file, function, lines));
    findings.extend(url_redirect_open(file, function, lines));
    findings.extend(ssrf_user_url(file, function, lines));
    findings.extend(ldap_injection(file, function, lines));
    findings.extend(header_injection(file, function, lines));
    findings.extend(xml_decoder_entity(file, function, lines));
    findings.extend(yaml_unmarshal_untrusted(file, function, lines));
    findings
}

fn os_exec_user_input(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("exec.Command(")
            && bl.text.contains("\"sh\"")
            && bl.text.contains("\"-c\"")
        {
            findings.push(Finding {
                rule_id: "os_exec_command_with_user_input".into(),
                severity: Severity::Error,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} passes user input to shell command",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("exec.Command(\"sh\", \"-c\", ...) at line {}", bl.line),
                    "OS command injection allows arbitrary code execution".into(),
                ],
            });
        }
    }
    findings
}

fn template_html_unescaped(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("template.HTML(")
            || bl.text.contains("template.JS(")
            || bl.text.contains("template.CSS(")
        {
            findings.push(Finding {
                rule_id: "template_html_unescaped".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} bypasses template auto-escaping",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("template.HTML/JS/CSS at line {}", bl.line),
                    "bypasses XSS protection".into(),
                ],
            });
        }
    }
    findings
}

fn text_template_html(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !has_import_path(file, "text/template") {
        return findings;
    }
    for bl in lines {
        if bl.text.contains(".html")
            || bl.text.contains("<html")
            || bl.text.contains("<div")
            || bl.text.contains("<script")
        {
            findings.push(Finding {
                rule_id: "text_template_for_html".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses text/template for HTML content",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("HTML content with text/template at line {}", bl.line),
                    "use html/template for context-aware escaping".into(),
                ],
            });
            break;
        }
    }
    findings
}

fn filepath_join_traversal(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    for alias in import_aliases_for(file, "path/filepath") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.Join(")) {
                let has_check = lines.iter().any(|l| {
                    l.text.contains(&format!("{alias}.Rel("))
                        || l.text.contains("..")
                        || l.text.contains("Clean(")
                });
                if !has_check {
                    findings.push(Finding {
                        rule_id: "filepath_join_with_user_path".into(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} joins filepath without traversal check",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("{}.Join in handler at line {}", alias, bl.line),
                            "path traversal via ../../ can access arbitrary files".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

fn url_redirect_open(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("http.Redirect(") || bl.text.contains(".Redirect(") {
            if bl.text.contains("FormValue(")
                || bl.text.contains("Query(")
                || bl.text.contains("Param(")
            {
                findings.push(Finding {
                    rule_id: "url_redirect_without_validation".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} redirects to user-provided URL",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("unvalidated redirect at line {}", bl.line),
                        "open redirect enables phishing attacks".into(),
                    ],
                });
            }
        }
    }
    findings
}

fn ssrf_user_url(file: &ParsedFile, function: &ParsedFunction, lines: &[BodyLine]) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    for alias in import_aliases_for(file, "net/http") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.Get("))
                || bl.text.contains(&format!("{alias}.NewRequest("))
            {
                if bl.text.contains("FormValue(")
                    || bl.text.contains("Query(")
                    || bl.text.contains("Param(")
                {
                    findings.push(Finding {
                        rule_id: "ssrf_via_user_controlled_url".into(),
                        severity: Severity::Error,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} makes HTTP request to user-controlled URL",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("SSRF vector at line {}", bl.line),
                            "validate URL against allowlist; block private IPs".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

fn ldap_injection(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_ldap = ["github.com/go-ldap/ldap/v3", "gopkg.in/ldap.v2"]
        .iter()
        .any(|path| has_import_path(file, path));
    if !has_ldap {
        return findings;
    }
    for bl in lines {
        if (bl.text.contains("ldap.NewSearchRequest(") || bl.text.contains(".Search("))
            && bl.text.contains('+')
        {
            findings.push(Finding {
                rule_id: "ldap_injection_via_string_concat".into(),
                severity: Severity::Error,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} builds an LDAP filter with string concatenation",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("dynamic LDAP filter at line {}", bl.line),
                    "user-controlled filter fragments can change query semantics and bypass lookup intent"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn header_injection(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    for bl in lines {
        if (bl.text.contains("Header().Set(")
            || bl.text.contains("Header().Add(")
            || bl.text.contains(".Header("))
            && (bl.text.contains("FormValue(")
                || bl.text.contains("Query(")
                || bl.text.contains("Param("))
        {
            findings.push(Finding {
                rule_id: "header_injection_via_user_input".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} reflects user input into an HTTP header",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "header value sourced from request input at line {}",
                        bl.line
                    ),
                    "unsanitized CRLF sequences can inject extra headers or split responses".into(),
                ],
            });
        }
    }
    findings
}

fn xml_decoder_entity(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    for alias in import_aliases_for(file, "encoding/xml") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.NewDecoder(")) {
                let has_entity = lines
                    .iter()
                    .any(|l| l.text.contains(".Entity") || l.text.contains("LimitReader"));
                if !has_entity {
                    findings.push(Finding {
                        rule_id: "xml_decoder_without_entity_limit".into(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} parses XML without entity/size limits",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("{}.NewDecoder in handler at line {}", alias, bl.line),
                            "XXE and Billion Laughs DoS vulnerable".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

fn yaml_unmarshal_untrusted(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_yaml = [
        "gopkg.in/yaml.v3",
        "gopkg.in/yaml.v2",
        "github.com/go-yaml/yaml",
    ]
    .iter()
    .any(|path| has_import_path(file, path));
    if !has_yaml || !is_request_path_function(file, function) {
        return findings;
    }
    for bl in lines {
        if (bl.text.contains("yaml.Unmarshal(") || bl.text.contains("yaml.NewDecoder("))
            && (bl.text.contains("r.Body")
                || bl.text.contains("req.Body")
                || bl.text.contains("FormValue("))
        {
            findings.push(Finding {
                rule_id: "yaml_unmarshal_untrusted_input".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} unmarshals YAML directly from untrusted input",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("untrusted YAML decode at line {}", bl.line),
                    "YAML decoders accept richer syntax than JSON; validate or restrict the input shape first"
                        .into(),
                ],
            });
        }
    }
    findings
}

// ── Section C — Auth, Session, Access Control ──
