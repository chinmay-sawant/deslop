use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::super::advanceplan3::{
    BodyLine, has_import_path, import_aliases_for, is_request_path_function,
};

pub(super) fn security_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(crypto_findings(file, function, lines));
    findings.extend(injection_findings(file, function, lines));
    findings.extend(auth_session_findings(file, function, lines));
    findings.extend(concurrency_security_findings(file, function, lines));
    findings.extend(network_tls_findings(file, function, lines));
    findings.extend(data_exposure_findings(file, function, lines));
    findings
}

pub(super) fn file_security_findings(file: &ParsedFile) -> Vec<Finding> {
    sensitive_struct_field_json_findings(file)
}

// ── Section A — Cryptographic Misuse ──

fn crypto_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(insecure_random(file, function, lines));
    findings.extend(tls_skip_verify(file, function, lines));
    findings.extend(tls_min_version(file, function, lines));
    findings.extend(weak_hash_for_integrity(file, function, lines));
    findings.extend(constant_encryption_key(file, function, lines));
    findings.extend(constant_iv_nonce(file, function, lines));
    findings.extend(ecb_mode_cipher(file, function, lines));
    findings.extend(jwt_none_algorithm_risk(file, function, lines));
    findings.extend(bcrypt_cost_low(file, function, lines));
    findings.extend(rsa_key_small(file, function, lines));
    findings
}

fn insecure_random(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let name_lower = function.fingerprint.name.to_lowercase();
    let is_security = [
        "token", "key", "password", "nonce", "salt", "session", "secret", "auth", "crypt",
    ]
    .iter()
    .any(|k| name_lower.contains(k));
    if !is_security {
        return findings;
    }
    for alias in import_aliases_for(file, "math/rand") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.")) {
                findings.push(Finding {
                    rule_id: "insecure_random_for_security".into(),
                    severity: Severity::Error,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses math/rand in security-sensitive context",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("math/rand usage at line {}", bl.line),
                        "use crypto/rand for security-critical randomness".into(),
                    ],
                });
                break;
            }
        }
    }
    findings
}

fn tls_skip_verify(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("InsecureSkipVerify") && bl.text.contains("true") {
            findings.push(Finding {
                rule_id: "hardcoded_tls_skip_verify".into(),
                severity: Severity::Error,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} disables TLS certificate verification",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("InsecureSkipVerify: true at line {}", bl.line),
                    "disables all TLS security, enables MITM attacks".into(),
                ],
            });
        }
    }
    findings
}

fn tls_min_version(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("MinVersion")
            && (bl.text.contains("VersionTLS10")
                || bl.text.contains("VersionTLS11")
                || bl.text.contains("VersionSSL30"))
        {
            findings.push(Finding {
                rule_id: "hardcoded_tls_min_version_too_low".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} sets TLS minimum version too low",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("weak TLS version at line {}", bl.line),
                    "TLS 1.2 minimum is required by PCI DSS".into(),
                ],
            });
        }
    }
    findings
}

fn weak_hash_for_integrity(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("md5.New(")
            || bl.text.contains("sha1.New(")
            || bl.text.contains("md5.Sum(")
            || bl.text.contains("sha1.Sum(")
        {
            findings.push(Finding {
                rule_id: "weak_hash_for_integrity".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} relies on MD5/SHA-1 for integrity-sensitive hashing",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("weak hash primitive at line {}", bl.line),
                    "MD5 and SHA-1 are collision-broken; prefer SHA-256 or stronger".into(),
                ],
            });
        }
    }
    findings
}

fn constant_encryption_key(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if (bl.text.contains("aes.NewCipher(") || bl.text.contains("NewGCM("))
            && bl.text.contains("[]byte(\"")
        {
            findings.push(Finding {
                rule_id: "constant_encryption_key".into(),
                severity: Severity::Error,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses hardcoded encryption key",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("literal []byte key at line {}", bl.line),
                    "keys in source are extractable from compiled binaries".into(),
                ],
            });
        }
    }
    findings
}

fn constant_iv_nonce(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains(".Seal(") || bl.text.contains("NewCBCEncrypter(") {
            if bl.text.contains("make([]byte,")
                || bl.text.contains("var iv")
                || bl.text.contains("[16]byte{}")
                || bl.text.contains("[]byte{0")
            {
                findings.push(Finding {
                    rule_id: "constant_iv_or_nonce".into(),
                    severity: Severity::Error,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses constant or zero IV/nonce",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("static IV/nonce at line {}", bl.line),
                        "reusing nonces with AES-GCM breaks authenticity".into(),
                    ],
                });
            }
        }
    }
    findings
}

fn ecb_mode_cipher(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if (bl.text.contains(".Encrypt(") || bl.text.contains(".Decrypt("))
            && !bl.text.contains("NewCBC")
            && !bl.text.contains("NewGCM")
            && !bl.text.contains("NewCTR")
            && !bl.text.contains("NewCFB")
        {
            findings.push(Finding {
                rule_id: "ecb_mode_cipher".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} appears to use a block cipher directly without a safe mode",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("direct block.Encrypt/Decrypt call at line {}", bl.line),
                    "direct block operations behave like ECB and leak plaintext patterns".into(),
                ],
            });
        }
    }
    findings
}

fn jwt_none_algorithm_risk(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_jwt = [
        "github.com/golang-jwt/jwt",
        "github.com/golang-jwt/jwt/v5",
        "github.com/dgrijalva/jwt-go",
    ]
    .iter()
    .any(|path| has_import_path(file, path));
    if !has_jwt {
        return findings;
    }
    let has_valid_methods = lines
        .iter()
        .any(|line| line.text.contains("WithValidMethods"));
    for bl in lines {
        if (bl.text.contains("jwt.Parse(") && !has_valid_methods) || bl.text.contains("\"none\"") {
            findings.push(Finding {
                rule_id: "jwt_none_algorithm_risk".into(),
                severity: Severity::Error,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} parses JWTs without constraining the signing method",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("JWT parse/alg handling at line {}", bl.line),
                    "restrict valid algorithms explicitly to avoid 'none' or downgrade bypasses"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn bcrypt_cost_low(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("GenerateFromPassword(") {
            if bl.text.contains("MinCost")
                || bl.text.contains(", 4)")
                || bl.text.contains(", 5)")
                || bl.text.contains(", 6)")
                || bl.text.contains(", 7)")
                || bl.text.contains(", 8)")
                || bl.text.contains(", 9)")
            {
                findings.push(Finding {
                    rule_id: "bcrypt_cost_too_low".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses low bcrypt cost",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("low bcrypt cost at line {}", bl.line),
                        "cost >= 12 recommended for production".into(),
                    ],
                });
            }
        }
    }
    findings
}

fn rsa_key_small(file: &ParsedFile, function: &ParsedFunction, lines: &[BodyLine]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("rsa.GenerateKey(") {
            if bl.text.contains(", 512)") || bl.text.contains(", 1024)") {
                findings.push(Finding {
                    rule_id: "rsa_key_size_too_small".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} generates RSA key smaller than 2048 bits",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("small RSA key at line {}", bl.line),
                        "NIST recommends 2048-bit minimum".into(),
                    ],
                });
            }
        }
    }
    findings
}

// ── Section B — Injection And Input Validation ──

fn injection_findings(
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

fn auth_session_findings(
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

fn concurrency_security_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(race_on_shared_map(file, function, lines));
    findings.extend(toctou_file_check_then_open(file, function, lines));
    findings.extend(shared_slice_append_race(file, function, lines));
    findings.extend(goroutine_captures_loop_variable(file, function, lines));
    findings.extend(unsafe_pointer_cast(file, function, lines));
    findings.extend(cgo_string_lifetime(file, function, lines));
    findings.extend(global_rand_source_contention(file, function, lines));
    findings
}

fn race_on_shared_map(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_goroutine = lines.iter().any(|line| line.text.contains("go func"));
    let has_lock = lines
        .iter()
        .any(|line| line.text.contains(".Lock()") || line.text.contains(".RLock()"));
    if !has_goroutine || has_lock {
        return findings;
    }
    for bl in lines {
        if bl.text.contains('[') && bl.text.contains("] =") && !bl.text.contains(":=") {
            findings.push(Finding {
                rule_id: "race_on_shared_map".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} mutates a shared map while launching goroutines",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("map write with goroutines at line {}", bl.line),
                    "plain Go maps are not safe for concurrent mutation without synchronization"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn toctou_file_check_then_open(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, bl) in lines.iter().enumerate() {
        if bl.text.contains("os.Stat(") || bl.text.contains("os.Lstat(") {
            if let Some(next) =
                lines.iter().skip(index + 1).take(5).find(|line| {
                    line.text.contains("os.OpenFile(") || line.text.contains("os.Create(")
                })
            {
                findings.push(Finding {
                    rule_id: "toctou_file_check_then_open".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: next.line,
                    message: format!(
                        "function {} checks a path before opening it",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!(
                            "file check at line {}, open/create at line {}",
                            bl.line, next.line
                        ),
                        "the file can change between the check and the open, enabling TOCTOU races"
                            .into(),
                    ],
                });
            }
        }
    }
    findings
}

fn shared_slice_append_race(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_goroutine = lines.iter().any(|line| line.text.contains("go func"));
    if !has_goroutine {
        return findings;
    }
    for bl in lines {
        if bl.text.contains("= append(") {
            findings.push(Finding {
                rule_id: "shared_slice_append_race".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} appends to a shared slice while using goroutines",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "slice append in goroutine-heavy function at line {}",
                        bl.line
                    ),
                    "concurrent append can race on slice headers and backing arrays".into(),
                ],
            });
        }
    }
    findings
}

fn goroutine_captures_loop_variable(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, bl) in lines.iter().enumerate() {
        if !(bl.text.contains("for _, ") && bl.text.contains(":= range")) {
            continue;
        }
        let loop_var = bl
            .text
            .split("for _, ")
            .nth(1)
            .and_then(|suffix| suffix.split(":=").next())
            .map(str::trim)
            .unwrap_or("");
        if loop_var.is_empty() {
            continue;
        }
        let Some(go_line) = lines
            .iter()
            .skip(index + 1)
            .take(6)
            .find(|line| line.text.contains("go func()"))
        else {
            continue;
        };
        let uses_loop_var = lines
            .iter()
            .skip(index + 1)
            .take(10)
            .any(|line| line.text.contains(loop_var));
        if uses_loop_var {
            findings.push(Finding {
                rule_id: "goroutine_captures_loop_variable".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: go_line.line,
                message: format!(
                    "function {} captures a loop variable in a goroutine closure",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("range loop at line {}, goroutine at line {}", bl.line, go_line.line),
                    "capture the value as a parameter so each goroutine sees the intended iteration value"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn unsafe_pointer_cast(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("unsafe.Pointer(uintptr(") {
            findings.push(Finding {
                rule_id: "unsafe_pointer_cast".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses unsafe.Pointer arithmetic",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("unsafe cast at line {}", bl.line),
                    "uintptr values can become dangling pointers".into(),
                ],
            });
        }
    }
    findings
}

fn cgo_string_lifetime(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("C.CString(") {
            let has_free = lines
                .iter()
                .any(|l| l.text.contains("C.free(") && l.line > bl.line);
            if !has_free {
                findings.push(Finding {
                    rule_id: "cgo_string_lifetime".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} allocates C string without free",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("C.CString without C.free at line {}", bl.line),
                        "leaks C memory not tracked by Go GC".into(),
                    ],
                });
            }
        }
    }
    findings
}

fn global_rand_source_contention(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let hot = is_request_path_function(file, function)
        || lines.iter().any(|line| line.text.contains("go func"));
    if !hot {
        return findings;
    }
    for alias in import_aliases_for(file, "math/rand") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.Intn("))
                || bl.text.contains(&format!("{alias}.Float64("))
                || bl.text.contains(&format!("{alias}.Uint32("))
            {
                findings.push(Finding {
                    rule_id: "global_rand_source_contention".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses the global math/rand source on a hot path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("global math/rand call at line {}", bl.line),
                        "the package-global source uses a mutex and can become contended under load"
                            .into(),
                    ],
                });
            }
        }
    }
    findings
}

// ── Section E — Network And TLS Security ──

fn network_tls_findings(
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

fn data_exposure_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(sensitive_data_log(file, function, lines));
    findings.extend(error_detail_client(file, function, lines));
    findings.extend(debug_endpoint(file, function, lines));
    findings.extend(struct_field_json_exposed(file, function, lines));
    findings.extend(temp_file_predictable(file, function, lines));
    findings.extend(world_readable_perms(file, function, lines));
    findings.extend(fmt_print_sensitive_struct(file, function, lines));
    findings.extend(panic_stack_trace_to_client(file, function, lines));
    findings.extend(env_var_in_error_message(file, function, lines));
    findings
}

fn sensitive_data_log(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let sensitive = [
        "password",
        "passwd",
        "secret",
        "token",
        "apikey",
        "api_key",
        "creditcard",
        "credit_card",
        "ssn",
    ];
    for bl in lines {
        if bl.text.contains("log.")
            || bl.text.contains("logger.")
            || bl.text.contains("slog.")
            || bl.text.contains("zap.")
            || bl.text.contains("logrus.")
        {
            let lower = bl.text.to_lowercase();
            for s in &sensitive {
                if lower.contains(s) {
                    findings.push(Finding {
                        rule_id: "sensitive_data_in_log".into(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} may log sensitive data ({})",
                            function.fingerprint.name, s
                        ),
                        evidence: vec![
                            format!("sensitive field in log statement at line {}", bl.line),
                            "redact sensitive fields before logging".into(),
                        ],
                    });
                    break;
                }
            }
        }
    }
    findings
}

fn error_detail_client(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    for bl in lines {
        if (bl.text.contains("c.JSON(")
            || bl.text.contains("c.String(")
            || bl.text.contains("http.Error("))
            && bl.text.contains("err.Error()")
        {
            findings.push(Finding {
                rule_id: "error_detail_leaked_to_client".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} leaks error details to client",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("err.Error() in response at line {}", bl.line),
                    "internal errors can leak stack traces and schemas".into(),
                ],
            });
        }
    }
    findings
}

fn debug_endpoint(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("/debug/pprof") || bl.text.contains("net/http/pprof") {
            findings.push(Finding {
                rule_id: "debug_endpoint_in_production".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} exposes debug/pprof endpoint",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("pprof endpoint at line {}", bl.line),
                    "exposes heap dumps and goroutine stacks".into(),
                ],
            });
        }
    }
    findings
}

fn struct_field_json_exposed(
    _file: &ParsedFile,
    _function: &ParsedFunction,
    _lines: &[BodyLine],
) -> Vec<Finding> {
    // This is a file-level check done via struct field scanning - handled in the body text
    Vec::new()
}

fn sensitive_struct_field_json_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    let sensitive_names = ["Password", "Secret", "Token", "APIKey", "PrivateKey"];
    for go_struct in &file.go_structs {
        let uses_json_tags = file
            .struct_tags
            .iter()
            .any(|tag| tag.struct_name == go_struct.name && tag.raw_tag.contains("json:"));
        if !uses_json_tags {
            continue;
        }
        for field in &go_struct.fields {
            if !field.is_pub || !sensitive_names.iter().any(|name| field.name.contains(name)) {
                continue;
            }
            let hidden = file.struct_tags.iter().any(|tag| {
                tag.struct_name == go_struct.name
                    && tag.field_name == field.name
                    && tag.raw_tag.contains("json:\"-\"")
            });
            if hidden {
                continue;
            }
            findings.push(Finding {
                rule_id: "struct_field_exposed_in_json".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: None,
                start_line: field.line,
                end_line: field.line,
                message: format!(
                    "struct {} exposes sensitive field {} to JSON serialization",
                    go_struct.name, field.name
                ),
                evidence: vec![
                    format!(
                        "exported sensitive field {} at line {}",
                        field.name, field.line
                    ),
                    "use `json:\"-\"` on secret-bearing fields that should never leave the server"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn temp_file_predictable(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("os.Create(\"/tmp/") || bl.text.contains("os.OpenFile(\"/tmp/") {
            findings.push(Finding {
                rule_id: "temp_file_predictable_name".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} creates temp file with predictable name",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("predictable temp path at line {}", bl.line),
                    "use os.CreateTemp for random suffix".into(),
                ],
            });
        }
    }
    findings
}

fn world_readable_perms(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if (bl.text.contains("os.OpenFile(") || bl.text.contains("os.WriteFile("))
            && (bl.text.contains("0666") || bl.text.contains("0777") || bl.text.contains("0644"))
        {
            findings.push(Finding {
                rule_id: "world_readable_file_permissions".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} creates file with overly permissive permissions",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("permissive file mode at line {}", bl.line),
                    "use 0600 for sensitive files".into(),
                ],
            });
        }
    }
    findings
}

fn fmt_print_sensitive_struct(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_sensitive_struct = file.go_structs.iter().any(|go_struct| {
        go_struct.fields.iter().any(|field| {
            ["Password", "Secret", "Token", "APIKey", "PrivateKey"]
                .iter()
                .any(|name| field.name.contains(name))
        })
    });
    if !has_sensitive_struct {
        return findings;
    }
    for bl in lines {
        if (bl.text.contains("fmt.Printf(") || bl.text.contains("fmt.Sprintf("))
            && (bl.text.contains("%+v") || bl.text.contains("%v"))
        {
            findings.push(Finding {
                rule_id: "fmt_print_of_sensitive_struct".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} prints a struct in a way that can expose sensitive fields",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("fmt print of struct-like value at line {}", bl.line),
                    "%+v includes field names and values; redact secrets before formatting".into(),
                ],
            });
        }
    }
    findings
}

fn panic_stack_trace_to_client(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    let has_recover = lines.iter().any(|line| line.text.contains("recover()"));
    if !has_recover {
        return findings;
    }
    for bl in lines {
        if (bl.text.contains("w.Write(") || bl.text.contains("http.Error("))
            && (bl.text.contains("fmt.Sprintf(\"%v\"") || bl.text.contains("stack"))
        {
            findings.push(Finding {
                rule_id: "panic_stack_trace_to_client".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} writes recovered panic details back to the client",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("panic response at line {}", bl.line),
                    "panic details should be logged internally and replaced with a generic 500 response"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn env_var_in_error_message(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("fmt.Errorf(") && bl.text.contains("os.Getenv(") {
            findings.push(Finding {
                rule_id: "env_var_in_error_message".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} embeds an environment variable value in an error message",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("os.Getenv(...) inside fmt.Errorf at line {}", bl.line),
                    "error messages can be logged or returned, leaking secrets from the environment".into(),
                ],
            });
        }
    }
    findings
}
