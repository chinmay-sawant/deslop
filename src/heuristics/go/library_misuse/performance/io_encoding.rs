use super::*;

pub(super) fn io_encoding_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(ioutil_readall(file, function, lines));
    findings.extend(json_marshal_then_write(file, function, lines));
    findings.extend(binary_read_single_field(file, function, lines));
    findings.extend(json_number_without_use_number(file, function, lines));
    findings.extend(xml_decoder_trusted_strict(file, function, lines));
    findings.extend(csv_reader_reuse(file, function, lines));
    findings.extend(scanner_small_buffer(file, function, lines));
    findings.extend(http_body_readall_no_limit(file, function, lines));
    findings
}

// D1
fn ioutil_readall(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "io/ioutil") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.ReadAll(")) {
                findings.push(Finding {
                    rule_id: "ioutil_readall_still_used".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses deprecated ioutil.ReadAll",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.ReadAll at line {}", alias, bl.line),
                        "io.ReadAll is canonical since Go 1.16".into(),
                    ],
                });
            }
        }
    }
    findings
}

// D2
fn json_marshal_then_write(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "encoding/json") {
        for (i, bl) in lines.iter().enumerate() {
            if bl.text.contains(&format!("{alias}.Marshal(")) && !bl.text.contains("MarshalIndent")
            {
                for next in lines.iter().skip(i + 1).take(5) {
                    if next.text.contains(".Write(") || next.text.contains("w.Write(") {
                        findings.push(Finding {
                            rule_id: "json_marshal_then_write".into(),
                            severity: Severity::Info,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: bl.line, end_line: next.line,
                            message: format!("function {} marshals JSON then writes separately", function.fingerprint.name),
                            evidence: vec![
                                format!("{}.Marshal at line {}, Write at line {}", alias, bl.line, next.line),
                                "json.NewEncoder(w).Encode(v) streams directly, saving one allocation".into(),
                            ],
                        });
                        break;
                    }
                }
            }
        }
    }
    findings
}

// D3
fn binary_read_single_field(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "encoding/binary") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.Read(")) && bl.text.contains('&') {
                findings.push(Finding {
                    rule_id: "binary_read_for_single_field".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses binary.Read for a single scalar field",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Read(..., &value) at line {}", alias, bl.line),
                        "binary.ByteOrder.Uint32/Uint64 avoids reflection for scalar reads".into(),
                    ],
                });
            }
        }
    }
    findings
}

// D4
fn json_number_without_use_number(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let decodes_to_dynamic_map = lines.iter().any(|line| {
        line.text.contains("map[string]any") || line.text.contains("map[string]interface{}")
    });
    if !decodes_to_dynamic_map || lines.iter().any(|line| line.text.contains("UseNumber()")) {
        return findings;
    }
    for alias in import_aliases_for(file, "encoding/json") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.Unmarshal(")) || bl.text.contains(".Decode(&") {
                findings.push(Finding {
                    rule_id: "json_number_vs_float64_decode".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} decodes JSON numbers into map[string]any without UseNumber",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("dynamic JSON decode at line {}", bl.line),
                        "decoder.UseNumber() preserves large integer precision instead of coercing to float64"
                            .into(),
                    ],
                });
                break;
            }
        }
    }
    findings
}

// D5
fn xml_decoder_trusted_strict(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let name_lower = function.fingerprint.name.to_lowercase();
    let trusted_context = name_lower.contains("trusted")
        || name_lower.contains("internal")
        || name_lower.contains("feed");
    if !trusted_context {
        return findings;
    }
    for alias in import_aliases_for(file, "encoding/xml") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.NewDecoder("))
                && !lines
                    .iter()
                    .any(|line| line.text.contains(".Strict = false"))
            {
                findings.push(Finding {
                    rule_id: "xml_decoder_without_strict".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} parses trusted XML without relaxing Strict mode",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("trusted XML decoder at line {}", bl.line),
                        "for trusted feeds, decoder.Strict = false can avoid extra validation work"
                            .into(),
                    ],
                });
            }
        }
    }
    findings
}

// D6
fn csv_reader_reuse(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "encoding/csv") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.NewReader(")) {
                let has_reuse = lines
                    .iter()
                    .any(|l| l.text.contains("ReuseRecord") && l.text.contains("true"));
                if !has_reuse {
                    findings.push(Finding {
                        rule_id: "csv_reader_reuse_record".into(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} creates csv.Reader without ReuseRecord",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("{}.NewReader at line {}", alias, bl.line),
                            "ReuseRecord = true reduces allocations from N to 1".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

// D7
fn scanner_small_buffer(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let name_lower = function.fingerprint.name.to_lowercase();
    let large_line_context =
        name_lower.contains("line") || name_lower.contains("log") || name_lower.contains("scan");
    if !large_line_context {
        return findings;
    }
    for alias in import_aliases_for(file, "bufio") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.NewScanner("))
                && !lines.iter().any(|line| line.text.contains(".Buffer("))
            {
                findings.push(Finding {
                    rule_id: "bufio_scanner_small_buffer_for_large_lines".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses bufio.Scanner without raising the token buffer",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("bufio.NewScanner at line {}", bl.line),
                        "scanner.Buffer(...) is required when large lines can exceed the 64KB default"
                            .into(),
                    ],
                });
            }
        }
    }
    findings
}

// D8
fn http_body_readall_no_limit(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    for alias in import_aliases_for(file, "io") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.ReadAll("))
                && (bl.text.contains("req.Body")
                    || bl.text.contains("r.Body")
                    || bl.text.contains("request.Body"))
            {
                let has_limit = lines
                    .iter()
                    .any(|l| l.text.contains("LimitReader") || l.text.contains("MaxBytesReader"));
                if !has_limit {
                    findings.push(Finding {
                        rule_id: "http_body_readall_without_limitreader".into(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} reads HTTP body without size limit",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "{}.ReadAll(req.Body) at line {} without LimitReader",
                                alias, bl.line
                            ),
                            "unbounded body read is a DoS vector".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

// ── Section E — Error Handling And Interface Patterns ──
