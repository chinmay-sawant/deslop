use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::common::{identifier_tokens, import_alias_lookup};

const WEAK_CRYPTO_IMPORTS: &[&str] = &["crypto/md5", "crypto/sha1", "crypto/des", "crypto/rc4"];

pub(super) fn weak_crypto_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let import_aliases = import_alias_lookup(&file.imports);
    let mut findings = Vec::new();

    for call in &function.calls {
        let Some(receiver) = &call.receiver else {
            continue;
        };
        let Some(import_path) = import_aliases.get(receiver) else {
            continue;
        };
        if !WEAK_CRYPTO_IMPORTS.contains(&import_path.as_str()) {
            continue;
        }

        findings.push(Finding {
            rule_id: "weak_crypto".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} uses weak cryptographic primitive {}",
                function.fingerprint.name, import_path
            ),
            evidence: vec![
                format!("import alias {receiver} resolves to {import_path}"),
                format!("weak crypto call: {receiver}.{}", call.name),
            ],
        });
    }

    findings
}

pub(super) fn package_hardcoded_secret_findings(file: &ParsedFile) -> Vec<Finding> {
    file.package_string_literals
        .iter()
        .filter(|literal| is_secret_like_name(&literal.name) && looks_like_secret_value(&literal.value))
        .map(|literal| Finding {
            rule_id: "hardcoded_secret".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: None,
            start_line: literal.line,
            end_line: literal.line,
            message: format!(
                "file declares a hardcoded secret-like literal in {}",
                literal.name
            ),
            evidence: vec![
                format!("secret-like identifier: {}", literal.name),
                format!("literal length: {}", literal.value.len()),
            ],
        })
        .collect()
}

pub(super) fn hardcoded_secret_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    function
        .local_string_literals
        .iter()
        .filter(|literal| is_secret_like_name(&literal.name) && looks_like_secret_value(&literal.value))
        .map(|literal| Finding {
            rule_id: "hardcoded_secret".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: literal.line,
            end_line: literal.line,
            message: format!(
                "function {} assigns a hardcoded secret-like literal",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("secret-like identifier: {}", literal.name),
                format!("literal length: {}", literal.value.len()),
            ],
        })
        .collect()
}

pub(super) fn sql_string_concat_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    function
        .db_query_calls
        .iter()
        .filter(|query_call| query_call.query_uses_dynamic_construction)
        .map(|query_call| Finding {
            rule_id: "sql_string_concat".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: query_call.line,
            end_line: query_call.line,
            message: format!(
                "function {} builds a SQL query dynamically before execution",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("query method: {}", query_call.method_name),
                format!(
                    "query expression: {}",
                    query_call.query_argument_text.as_deref().unwrap_or("<unknown>")
                ),
            ],
        })
        .collect()
}

fn is_secret_like_name(name: &str) -> bool {
    let tokens = identifier_tokens(name);
    if tokens.is_empty() {
        return false;
    }

    let joined = tokens.join("_");
    matches!(
        joined.as_str(),
        "password"
            | "passwd"
            | "pwd"
            | "secret"
            | "client_secret"
            | "api_key"
            | "apikey"
            | "private_key"
            | "access_token"
            | "refresh_token"
            | "auth_token"
    ) || tokens.contains(&"password".to_string())
        || tokens.contains(&"secret".to_string())
        || tokens.contains(&"apikey".to_string())
        || tokens.contains(&"private".to_string()) && tokens.contains(&"key".to_string())
        || tokens.contains(&"token".to_string())
            && tokens.iter().any(|token| matches!(token.as_str(), "api" | "auth" | "access" | "refresh" | "client"))
}

fn looks_like_secret_value(value: &str) -> bool {
    let trimmed = value.trim();
    let normalized = trimmed.to_ascii_lowercase();

    trimmed.len() >= 8
        && !matches!(normalized.as_str(), "placeholder" | "example" | "sample" | "changeme" | "your-api-key" | "your-secret")
        && normalized != "bearer"
}
