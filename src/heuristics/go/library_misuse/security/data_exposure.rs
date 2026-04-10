use super::*;

pub(super) fn file_security_findings(file: &ParsedFile) -> Vec<Finding> {
    sensitive_struct_field_json_findings(file)
}

pub(super) fn data_exposure_findings(
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

pub(super) fn sensitive_struct_field_json_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    let sensitive_names = ["Password", "Secret", "Token", "APIKey", "PrivateKey"];
    for go_struct in file.go_structs() {
        let uses_json_tags = file
            .struct_tags()
            .iter()
            .any(|tag| tag.struct_name == go_struct.name && tag.raw_tag.contains("json:"));
        if !uses_json_tags {
            continue;
        }
        for field in &go_struct.fields {
            if !field.is_pub || !sensitive_names.iter().any(|name| field.name.contains(name)) {
                continue;
            }
            let hidden = file.struct_tags().iter().any(|tag| {
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
        if !looks_like_file_creation_call(&bl.text) {
            continue;
        }

        let looks_sensitive_path = line_mentions_sensitive_path(&bl.text);
        for mode_literal in octal_mode_literals(&bl.text) {
            let Some(mode) = parse_go_octal_mode(&mode_literal) else {
                continue;
            };
            if !is_overly_permissive_mode(mode, looks_sensitive_path) {
                continue;
            }

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
                    format!("permissive file mode {mode_literal} at line {}", bl.line),
                    "prefer 0600 for secret material and avoid world-writable modes".into(),
                ],
            });
            break;
        }
    }
    findings
}

fn looks_like_file_creation_call(line: &str) -> bool {
    line.contains("os.OpenFile(") || line.contains("os.WriteFile(")
}

fn line_mentions_sensitive_path(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "secret",
        "token",
        "passwd",
        "password",
        "private",
        "apikey",
        "api_key",
        "credential",
        ".pem",
        ".key",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn octal_mode_literals(line: &str) -> Vec<String> {
    line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|token| {
            (token.starts_with('0') && token.len() >= 4 && token[1..].chars().all(is_octal_digit))
                || (token.starts_with("0o")
                    && token.len() >= 4
                    && token[2..].chars().all(is_octal_digit))
        })
        .map(ToString::to_string)
        .collect()
}

fn is_octal_digit(ch: char) -> bool {
    matches!(ch, '0'..='7')
}

fn parse_go_octal_mode(literal: &str) -> Option<u32> {
    if let Some(rest) = literal.strip_prefix("0o") {
        return u32::from_str_radix(rest, 8).ok();
    }
    if literal.len() >= 2 && literal.starts_with('0') && literal[1..].chars().all(is_octal_digit) {
        return u32::from_str_radix(&literal[1..], 8).ok();
    }
    None
}

fn is_overly_permissive_mode(mode: u32, sensitive_path: bool) -> bool {
    let world_writable = mode & 0o002 != 0;
    let world_readable = mode & 0o004 != 0;
    world_writable || (sensitive_path && world_readable)
}

fn fmt_print_sensitive_struct(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_sensitive_struct = file.go_structs().iter().any(|go_struct| {
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
                    "error messages can be logged or returned, leaking secrets from the environment"
                        .into(),
                ],
            });
        }
    }
    findings
}
