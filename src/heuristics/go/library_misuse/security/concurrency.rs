use super::*;

pub(super) fn concurrency_security_findings(
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
        if (bl.text.contains("os.Stat(") || bl.text.contains("os.Lstat("))
            && let Some(next) =
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
    let transfers_result_ownership = function
        .doc_comment
        .as_deref()
        .map(str::to_ascii_lowercase)
        .is_some_and(|comment| {
            comment.contains("caller must free")
                || comment.contains("freed by the caller")
                || comment.contains("caller is responsible for freeing")
        });
    for bl in lines {
        if bl.text.contains("C.CString(") {
            let is_result_field_transfer =
                transfers_result_ownership && bl.text.contains("result.") && bl.text.contains('=');
            if is_result_field_transfer {
                continue;
            }
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
