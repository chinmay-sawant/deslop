use super::*;

pub(super) fn runtime_sync_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(sync_mutex_for_counter(file, function, lines));
    findings.extend(sync_mutex_for_readonly_config(file, function, lines));
    findings.extend(sync_pool_ignored_small_allocs(file, function, lines));
    findings.extend(mutex_value_receiver(file, function, lines));
    findings.extend(time_now_tight_loop(file, function, lines));
    findings.extend(defer_in_tight_loop(file, function, lines));
    findings.extend(select_single_case(file, function, lines));
    findings.extend(goroutine_for_sync(file, function, lines));
    findings.extend(unbuffered_channel_known_count(file, function, lines));
    findings.extend(waitgroup_add_in_loop(file, function, lines));
    findings
}

// C1
fn sync_mutex_for_counter(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (i, bl) in lines.iter().enumerate() {
        if bl.text.contains(".Lock()") {
            let next_lines: Vec<&BodyLine> = lines.iter().skip(i + 1).take(3).collect();
            for nl in &next_lines {
                if (nl.text.contains("++")
                    || nl.text.contains("+= 1")
                    || nl.text.contains("-= 1")
                    || nl.text.contains("--"))
                    && !nl.text.contains("//")
                    && !nl.text.contains('.')
                {
                    let unlock = next_lines.iter().any(|l| l.text.contains(".Unlock()"));
                    if unlock {
                        findings.push(Finding {
                            rule_id: "sync_mutex_for_atomic_counter".into(),
                            severity: Severity::Warning,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: bl.line,
                            end_line: nl.line,
                            message: format!(
                                "function {} uses mutex for simple counter increment",
                                function.fingerprint.name
                            ),
                            evidence: vec![
                                format!("Lock/counter/Unlock pattern at line {}", bl.line),
                                "atomic.AddInt64 is lock-free and ~5× faster".into(),
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

// C2
fn sync_mutex_for_readonly_config(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, bl) in lines.iter().enumerate() {
        if !bl.text.contains(".RLock()") {
            continue;
        }
        let Some(unlock_line) = lines
            .iter()
            .skip(index + 1)
            .take(6)
            .find(|line| line.text.contains(".RUnlock()"))
        else {
            continue;
        };
        let has_config_read = lines
            .iter()
            .skip(index + 1)
            .take_while(|line| line.line <= unlock_line.line)
            .any(|line| {
                line.text.contains("config.")
                    || line.text.contains("cfg.")
                    || line.text.contains("settings.")
            });
        if has_config_read {
            findings.push(Finding {
                rule_id: "sync_mutex_for_readonly_config".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: unlock_line.line,
                message: format!(
                    "function {} locks around read-mostly config access",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "RLock/RUnlock config read at lines {}-{}",
                        bl.line, unlock_line.line
                    ),
                    "atomic.Value avoids lock overhead on the hot read path".into(),
                ],
            });
        }
    }
    findings
}

// C3
fn sync_pool_ignored_small_allocs(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        let hot = bl.in_loop || is_request_path_function(file, function);
        let alloc = bl.text.contains("make([]byte, 1024")
            || bl.text.contains("make([]byte, 2048")
            || bl.text.contains("make([]byte, 4096")
            || bl.text.contains("make([]byte, 8192")
            || bl.text.contains("make([]byte, 16384")
            || bl.text.contains("make([]byte, 32768")
            || bl.text.contains("new(bytes.Buffer)")
            || bl.text.contains("new(strings.Builder)");
        if hot && alloc {
            findings.push(Finding {
                rule_id: "sync_pool_ignored_for_frequent_small_allocs".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} repeatedly allocates reusable scratch objects",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("scratch allocation at line {}", bl.line),
                    "sync.Pool can amortize hot-path buffer allocations after warmup".into(),
                ],
            });
        }
    }
    findings
}

// C4
fn mutex_value_receiver(
    file: &ParsedFile,
    function: &ParsedFunction,
    _lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let Some(receiver) = &function.fingerprint.receiver_type else {
        return findings;
    };
    if function.signature_text.contains(&format!("*{receiver}")) {
        return findings;
    }
    let Some(go_struct) = file
        .go_structs()
        .iter()
        .find(|summary| summary.name == *receiver)
    else {
        return findings;
    };
    if go_struct.fields.iter().any(|field| {
        field.type_text.contains("sync.Mutex") || field.type_text.contains("sync.RWMutex")
    }) {
        findings.push(Finding {
            rule_id: "mutex_value_receiver".into(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "method {} uses a value receiver on a type containing a mutex",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "value receiver on {receiver} at line {}",
                    function.fingerprint.start_line
                ),
                "copying a mutex by value is unsafe; use a pointer receiver".into(),
            ],
        });
    }
    findings
}

// C5
fn time_now_tight_loop(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "time") {
        for bl in lines {
            if bl.in_loop && bl.text.contains(&format!("{alias}.Now()")) {
                findings.push(Finding {
                    rule_id: "time_now_in_tight_loop".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} calls time.Now() inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Now() in loop at line {}", alias, bl.line),
                        "cache timestamp before loop if millisecond precision suffices".into(),
                    ],
                });
            }
        }
    }
    findings
}

// C6
fn defer_in_tight_loop(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.in_loop
            && bl.text.starts_with("defer ")
            && !bl.text.contains("wg.Done()")
            && !bl.text.contains(".Done()")
        {
            findings.push(Finding {
                rule_id: "defer_in_tight_loop".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses defer inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("defer in loop at line {}", bl.line),
                    "defers accumulate until function exit; extract to helper function".into(),
                ],
            });
        }
    }
    findings
}

// C7
fn select_single_case(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (i, bl) in lines.iter().enumerate() {
        if bl.text.starts_with("select {") || bl.text == "select{" {
            let mut case_count = 0;
            let mut has_default = false;
            for next in lines.iter().skip(i + 1).take(10) {
                if next.text.starts_with("case ") {
                    case_count += 1;
                }
                if next.text.starts_with("default:") {
                    has_default = true;
                }
                if next.text == "}" {
                    break;
                }
            }
            if case_count == 1 && !has_default {
                findings.push(Finding {
                    rule_id: "select_with_single_case".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses select with only one case",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("single-case select at line {}", bl.line),
                        "direct channel receive is ~40% faster".into(),
                    ],
                });
            }
        }
    }
    findings
}

// C8
fn goroutine_for_sync(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (i, bl) in lines.iter().enumerate() {
        if bl.text.contains("go func()") {
            for next in lines.iter().skip(i + 1).take(10) {
                if next.text.contains("<-") && !next.text.contains("go ") {
                    findings.push(Finding {
                        rule_id: "goroutine_for_sync_work".into(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: next.line,
                        message: format!(
                            "function {} spawns goroutine for immediately-awaited work",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "go func() at line {}, awaited at line {}",
                                bl.line, next.line
                            ),
                            "direct function call avoids ~1μs goroutine spawn overhead".into(),
                        ],
                    });
                    break;
                }
            }
        }
    }
    findings
}

// C9
fn unbuffered_channel_known_count(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, bl) in lines.iter().enumerate() {
        let Some((left, right)) = split_assignment(&bl.text) else {
            continue;
        };
        if !right.contains("make(chan") {
            continue;
        }
        let make_args = right.split("make(chan").nth(1).unwrap_or("");
        let inside = make_args.split(')').next().unwrap_or("");
        if inside.contains(',') {
            continue;
        }
        let channel = left
            .trim()
            .split(',')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim();
        if !is_identifier_name(channel) {
            continue;
        }
        let has_loop_send = lines
            .iter()
            .skip(index + 1)
            .take(12)
            .any(|line| line.text.contains(&format!("{channel} <-")) && line.in_loop);
        if has_loop_send {
            findings.push(Finding {
                rule_id: "unbuffered_channel_for_known_producer_count".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} creates an unbuffered channel for a bounded producer loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("make(chan T) at line {} with looped sends", bl.line),
                    "a bounded buffer can avoid repeated park/unpark synchronization".into(),
                ],
            });
        }
    }
    findings
}

// C10
fn waitgroup_add_in_loop(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.in_loop && bl.text.contains(".Add(1)") && bl.text.contains("wg") {
            findings.push(Finding {
                rule_id: "waitgroup_add_inside_loop".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} calls wg.Add(1) inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("wg.Add(1) in loop at line {}", bl.line),
                    "wg.Add(n) before the loop uses one atomic op instead of n".into(),
                ],
            });
        }
    }
    findings
}

// ── Section D — I/O And Encoding ──
