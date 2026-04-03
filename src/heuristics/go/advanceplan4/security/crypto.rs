use super::*;

pub(super) fn crypto_findings(
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
        if (bl.text.contains(".Seal(") || bl.text.contains("NewCBCEncrypter("))
            && (bl.text.contains("make([]byte,")
                || bl.text.contains("var iv")
                || bl.text.contains("[16]byte{}")
                || bl.text.contains("[]byte{0"))
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
        if bl.text.contains("GenerateFromPassword(")
            && (bl.text.contains("MinCost")
                || bl.text.contains(", 4)")
                || bl.text.contains(", 5)")
                || bl.text.contains(", 6)")
                || bl.text.contains(", 7)")
                || bl.text.contains(", 8)")
                || bl.text.contains(", 9)"))
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
    findings
}

fn rsa_key_small(file: &ParsedFile, function: &ParsedFunction, lines: &[BodyLine]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("rsa.GenerateKey(")
            && (bl.text.contains(", 512)") || bl.text.contains(", 1024)"))
        {
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
    findings
}

// ── Section B — Injection And Input Validation ──
