use super::*;

pub(super) fn error_interface_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(type_assertion_no_comma(file, function, lines));
    findings.extend(type_switch_repeated(file, function, lines));
    findings.extend(errors_new_hot_path(file, function, lines));
    findings.extend(errorf_no_wrap(file, function, lines));
    findings.extend(error_string_compare(file, function, lines));
    findings.extend(empty_interface_parameter_overuse(file, function, lines));
    findings.extend(panic_for_expected(file, function, lines));
    findings
}

// E1
fn type_assertion_no_comma(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains(".(")
            && !bl.text.contains(", ok")
            && !bl.text.contains(",ok")
            && !bl.text.contains("switch")
            && !bl.text.contains(".(type)")
        {
            let has_assign = bl.text.contains(":=") || bl.text.contains(" = ");
            if has_assign && bl.text.contains(")") {
                findings.push(Finding {
                    rule_id: "type_assertion_without_comma_ok".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses type assertion without comma-ok",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("unchecked type assertion at line {}", bl.line),
                        "v, ok := i.(T) prevents runtime panics".into(),
                    ],
                });
            }
        }
    }
    findings
}

// E2
fn type_switch_repeated(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut assertion_lines: Vec<usize> = Vec::new();
    for bl in lines {
        if bl.text.contains("if _, ok :=") && bl.text.contains(".(") {
            assertion_lines.push(bl.line);
        }
    }
    if assertion_lines.len() >= 3 {
        findings.push(Finding {
            rule_id: "type_switch_vs_repeated_assertions".into(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: assertion_lines[0],
            end_line: assertion_lines[assertion_lines.len() - 1],
            message: format!(
                "function {} uses sequential type assertions instead of type switch",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "{} sequential type assertions at lines {}",
                    assertion_lines.len(),
                    join_lines(&assertion_lines)
                ),
                "switch v := i.(type) compiles to a single dispatch".into(),
            ],
        });
    }
    findings
}

// E3
fn errors_new_hot_path(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "errors") {
        let mut error_new_lines: Vec<usize> = Vec::new();
        for bl in lines {
            if bl.text.contains(&format!("{alias}.New(\"")) {
                error_new_lines.push(bl.line);
            }
        }
        if error_new_lines.len() >= 2 {
            findings.push(Finding {
                rule_id: "errors_new_for_static_sentinel".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: error_new_lines[0],
                end_line: error_new_lines[error_new_lines.len() - 1],
                message: format!(
                    "function {} calls errors.New multiple times with static strings",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "{}.New called at lines {}",
                        alias,
                        join_lines(&error_new_lines)
                    ),
                    "package-level sentinel errors avoid repeated allocations".into(),
                ],
            });
        }
    }
    findings
}

// E4
fn errorf_no_wrap(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "fmt") {
        for bl in lines {
            let pat = format!("{alias}.Errorf(");
            if bl.text.contains(&pat)
                && bl.text.contains("%v")
                && bl.text.contains("err")
                && !bl.text.contains("%w")
            {
                findings.push(Finding {
                    rule_id: "fmt_errorf_without_wrap_verb".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses %v instead of %w for error wrapping",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Errorf with %v at line {}", alias, bl.line),
                        "%w wraps the error preserving errors.Is/As chain".into(),
                    ],
                });
            }
        }
    }
    findings
}

// E5
fn error_string_compare(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains(".Error()") && (bl.text.contains("== \"") || bl.text.contains("!= \""))
        {
            findings.push(Finding {
                rule_id: "error_string_comparison".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} compares errors by string value",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("err.Error() == \"...\" at line {}", bl.line),
                    "errors.Is(err, sentinel) is faster and semantically correct".into(),
                ],
            });
        }
    }
    findings
}

// E6
fn empty_interface_parameter_overuse(
    file: &ParsedFile,
    function: &ParsedFunction,
    _lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let exported = function
        .fingerprint
        .name
        .chars()
        .next()
        .is_some_and(char::is_uppercase);
    if exported
        && (function.fingerprint.contains_any_type || function.fingerprint.contains_empty_interface)
        && (function.signature_text.contains(" any")
            || function.signature_text.contains(" interface{}")
            || function.signature_text.contains("(any")
            || function.signature_text.contains("(interface{}"))
    {
        findings.push(Finding {
            rule_id: "empty_interface_parameter_overuse".into(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "exported function {} overuses any/interface{{}} in its signature",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("signature at line {} accepts any/interface{{}}", function.fingerprint.start_line),
                "concrete types or generics avoid heap escapes and make the API easier to reason about"
                    .into(),
            ],
        });
    }
    findings
}

// E7
fn panic_for_expected(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.starts_with("panic(") || bl.text.starts_with("panic (") {
            let msg = bl.text.to_lowercase();
            if msg.contains("invalid")
                || msg.contains("missing")
                || msg.contains("not found")
                || msg.contains("unsupported")
                || msg.contains("unexpected")
            {
                findings.push(Finding {
                    rule_id: "panic_for_expected_errors".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses panic for expected error conditions",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("panic with expected-error message at line {}", bl.line),
                        "returning an error is ~200× cheaper and doesn't crash the process".into(),
                    ],
                });
            }
        }
    }
    findings
}
