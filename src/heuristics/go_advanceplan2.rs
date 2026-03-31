use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::analysis::{InterfaceSummary, ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity, SymbolKind};

use super::common::import_alias_lookup;

#[derive(Debug, Clone)]
struct BodyLine {
    line: usize,
    text: String,
    in_loop: bool,
}

pub(super) fn go_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = mutable_package_global_findings(file);

    for function in &file.functions {
        findings.extend(go_function_findings(file, function));
    }

    findings
}

pub(super) fn go_function_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if file.is_test_file || function.is_test_function {
        return Vec::new();
    }

    let mut findings = channel_and_timer_findings(file, function);
    findings.extend(http_boundary_findings(file, function));
    findings.extend(resource_hygiene_findings(file, function));
    findings.extend(init_side_effect_findings(file, function));
    findings.extend(public_bool_parameter_findings(file, function));
    findings
}

pub(super) fn go_repo_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = single_impl_interface_findings(files);
    findings.extend(passthrough_wrapper_interface_findings(files));
    findings
}

fn channel_and_timer_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines = body_lines(function);
    let time_aliases = import_aliases_for(file, "time");

    let local_channels = binding_matches(&lines, &["make(chan ", "make(chan<-"]);
    for (name, line, _) in &local_channels {
        let close_lines = lines_for(&lines, &format!("close({name})"));
        let range_line = lines
            .iter()
            .find(|body_line| {
                body_line.text.contains("for")
                    && body_line.text.contains("range")
                    && body_line.text.contains(name)
            })
            .map(|body_line| body_line.line);

        if let Some(range_line) = range_line
            && close_lines.is_empty()
            && !returns_binding(function, name)
        {
            findings.push(Finding {
                rule_id: "range_over_local_channel_without_close".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: range_line,
                end_line: range_line,
                message: format!(
                    "function {} ranges over local channel {} without an observed close path",
                    function.fingerprint.name, name
                ),
                evidence: vec![
                    format!("local channel {name} created at line {line}"),
                    format!("for-range over {name} observed at line {range_line}"),
                    format!("no close({name}) call was observed in the owning function"),
                ],
            });
        }

        if close_lines.len() > 1 {
            findings.push(Finding {
                rule_id: "double_close_local_channel".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: close_lines[1],
                end_line: close_lines[1],
                message: format!(
                    "function {} closes local channel {} more than once",
                    function.fingerprint.name, name
                ),
                evidence: vec![
                    format!("local channel {name} created at line {line}"),
                    format!(
                        "observed close({name}) at lines {}",
                        join_lines(&close_lines)
                    ),
                ],
            });
        }

        if let Some(first_close) = close_lines.iter().copied().min() {
            let send_line = lines
                .iter()
                .find(|body_line| {
                    body_line.line > first_close && body_line.text.contains(&format!("{name} <-"))
                })
                .map(|body_line| body_line.line);
            if let Some(send_line) = send_line {
                findings.push(Finding {
                    rule_id: "send_after_local_close_risk".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: send_line,
                    end_line: send_line,
                    message: format!(
                        "function {} sends on local channel {} after it appears closed",
                        function.fingerprint.name, name
                    ),
                    evidence: vec![
                        format!("close({name}) observed at line {first_close}"),
                        format!("send on {name} observed later at line {send_line}"),
                    ],
                });
            }
        }
    }

    for alias in &time_aliases {
        for body_line in &lines {
            if body_line.in_loop && body_line.text.contains(&format!("{alias}.After(")) {
                findings.push(Finding {
                    rule_id: "time_after_in_loop".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} allocates time.After inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{alias}.After(...) observed inside a loop at line {}", body_line.line),
                        "reusing a timer or context deadline is usually more stable than allocating a fresh timer each iteration"
                            .to_string(),
                    ],
                });
            }
        }
    }

    for alias in &time_aliases {
        let ticker_bindings = binding_matches(&lines, &[&format!("{alias}.NewTicker(")]);
        for (name, line, _) in ticker_bindings {
            if !contains_text(&lines, &format!("{name}.Stop()")) {
                findings.push(Finding {
                    rule_id: "ticker_without_stop".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: line,
                    end_line: line,
                    message: format!(
                        "function {} creates ticker {} without an observed Stop() call",
                        function.fingerprint.name, name
                    ),
                    evidence: vec![
                        format!("{name} bound from {alias}.NewTicker(...) at line {line}"),
                        format!("no {}.Stop() call was observed in the function", name),
                    ],
                });
            }
        }
    }

    findings
}

fn http_boundary_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines = body_lines(function);
    let http_aliases = import_aliases_for(file, "net/http");

    let response_bindings = http_response_bindings(&lines, &http_aliases);
    for (name, line, target) in &response_bindings {
        if !contains_text(&lines, &format!("{name}.Body.Close()"))
            && !returns_binding(function, name)
        {
            findings.push(Finding {
                rule_id: "http_response_body_not_closed".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: *line,
                end_line: *line,
                message: format!(
                    "function {} obtains HTTP response {} without an observed Body.Close() call",
                    function.fingerprint.name, name
                ),
                evidence: vec![
                    format!("response binding {name} created from {target} at line {line}"),
                    format!(
                        "no {}.Body.Close() call was observed in the owning function",
                        name
                    ),
                ],
            });
        }

        let decode_line = lines
            .iter()
            .find(|body_line| {
                body_line.text.contains(&format!("{}.Body", name))
                    && (body_line.text.contains("Decode(")
                        || body_line.text.contains("ReadAll(")
                        || body_line.text.contains("Read(")
                        || body_line.text.contains("Unmarshal("))
            })
            .map(|body_line| body_line.line);
        if let Some(decode_line) = decode_line
            && !contains_text(&lines, &format!("{name}.StatusCode"))
        {
            findings.push(Finding {
                rule_id: "http_status_ignored_before_decode".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: decode_line,
                end_line: decode_line,
                message: format!(
                    "function {} decodes HTTP response {} without an observed status check",
                    function.fingerprint.name, name
                ),
                evidence: vec![
                    format!("response binding {name} created at line {line}"),
                    format!("response body used at line {decode_line}"),
                    format!("no {}.StatusCode check was observed before decode", name),
                ],
            });
        }
    }

    for alias in &http_aliases {
        for (line, literal) in composite_literal_blocks(function, &format!("{alias}.Client{{")) {
            if !literal.contains("Timeout:") {
                findings.push(Finding {
                    rule_id: "http_client_without_timeout".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: line,
                    end_line: line,
                    message: format!(
                        "function {} constructs http.Client without an explicit timeout",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{alias}.Client{{...}} literal observed at line {line}"),
                        "no Timeout field was present in the client literal".to_string(),
                    ],
                });
            }
        }

        for (line, literal) in composite_literal_blocks(function, &format!("{alias}.Server{{")) {
            if !literal.contains("ReadTimeout:")
                && !literal.contains("WriteTimeout:")
                && !literal.contains("IdleTimeout:")
                && !literal.contains("ReadHeaderTimeout:")
            {
                findings.push(Finding {
                    rule_id: "http_server_without_timeouts".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: line,
                    end_line: line,
                    message: format!(
                        "function {} constructs http.Server without explicit timeout fields",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{alias}.Server{{...}} literal observed at line {line}"),
                        "no ReadTimeout, WriteTimeout, IdleTimeout, or ReadHeaderTimeout field was present"
                            .to_string(),
                    ],
                });
            }
        }
    }

    findings.extend(write_header_order_findings(file, function, &lines));
    findings
}

fn resource_hygiene_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines = body_lines(function);
    let os_aliases = import_aliases_for(file, "os");

    for alias in &os_aliases {
        for (name, line, target) in binding_matches(
            &lines,
            &[
                &format!("{alias}.Open("),
                &format!("{alias}.Create("),
                &format!("{alias}.OpenFile("),
            ],
        ) {
            if !contains_text(&lines, &format!("{name}.Close()"))
                && !returns_binding(function, &name)
            {
                findings.push(Finding {
                    rule_id: "file_handle_without_close".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: line,
                    end_line: line,
                    message: format!(
                        "function {} opens file handle {} without an observed Close() call",
                        function.fingerprint.name, name
                    ),
                    evidence: vec![
                        format!("file handle {name} created from {target} at line {line}"),
                        format!(
                            "no {}.Close() call was observed in the owning function",
                            name
                        ),
                    ],
                });
            }
        }
    }

    for (name, line, target) in binding_matches(&lines, &[".Query(", ".QueryContext("]) {
        if !contains_text(&lines, &format!("{name}.Close()")) && !returns_binding(function, &name) {
            findings.push(Finding {
                rule_id: "rows_without_close".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: line,
                end_line: line,
                message: format!(
                    "function {} uses rows handle {} without an observed Close() call",
                    function.fingerprint.name, name
                ),
                evidence: vec![
                    format!("rows binding {name} created from {target} at line {line}"),
                    format!(
                        "no {}.Close() call was observed in the owning function",
                        name
                    ),
                ],
            });
        }
    }

    for (name, line, target) in binding_matches(&lines, &[".Prepare(", ".PrepareContext("]) {
        if !contains_text(&lines, &format!("{name}.Close()")) && !returns_binding(function, &name) {
            findings.push(Finding {
                rule_id: "stmt_without_close".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: line,
                end_line: line,
                message: format!(
                    "function {} prepares statement {} without an observed Close() call",
                    function.fingerprint.name, name
                ),
                evidence: vec![
                    format!("statement binding {name} created from {target} at line {line}"),
                    format!(
                        "no {}.Close() call was observed in the owning function",
                        name
                    ),
                ],
            });
        }
    }

    for (name, line, target) in binding_matches(&lines, &[".Begin(", ".BeginTx("]) {
        if contains_text(&lines, &format!("{name}.Commit()"))
            && !contains_text(&lines, &format!("{name}.Rollback()"))
        {
            findings.push(Finding {
                rule_id: "tx_without_rollback_guard".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: line,
                end_line: line,
                message: format!(
                    "function {} begins transaction {} without an observed rollback guard",
                    function.fingerprint.name, name
                ),
                evidence: vec![
                    format!("transaction binding {name} created from {target} at line {line}"),
                    format!(
                        "{}.Commit() was observed but {}.Rollback() was not",
                        name, name
                    ),
                ],
            });
        }
    }

    for body_line in &lines {
        if body_line.in_loop && body_line.text.starts_with("defer ") {
            findings.push(Finding {
                rule_id: "defer_in_loop_resource_growth".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} defers cleanup inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "defer statement observed inside a loop at line {}",
                        body_line.line
                    ),
                    "loop-local defer calls can accumulate resources until function exit"
                        .to_string(),
                ],
            });
        }
    }

    findings
}

fn init_side_effect_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.fingerprint.name != "init" {
        return Vec::new();
    }

    let import_aliases = import_alias_lookup(&file.imports);
    for call in &function.calls {
        let side_effect = match call
            .receiver
            .as_deref()
            .and_then(|receiver| import_aliases.get(receiver))
        {
            Some(path)
                if path == "net/http"
                    && matches!(call.name.as_str(), "Get" | "Post" | "PostForm" | "Head") =>
            {
                Some(format!(
                    "network call via {}.{}",
                    call.receiver.as_deref().unwrap_or("<unknown>"),
                    call.name
                ))
            }
            Some(path)
                if path == "os"
                    && matches!(
                        call.name.as_str(),
                        "Open" | "OpenFile" | "Create" | "ReadFile"
                    ) =>
            {
                Some(format!(
                    "file-system call via {}.{}",
                    call.receiver.as_deref().unwrap_or("<unknown>"),
                    call.name
                ))
            }
            Some(path) if path == "os/exec" => Some(format!(
                "subprocess setup via {}.{}",
                call.receiver.as_deref().unwrap_or("<unknown>"),
                call.name
            )),
            _ => None,
        };

        if let Some(side_effect) = side_effect {
            return vec![Finding {
                rule_id: "init_side_effect".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: "init() performs external side effects at import/startup time".to_string(),
                evidence: vec![side_effect],
            }];
        }
    }

    Vec::new()
}

fn public_bool_parameter_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !is_exported_name(&function.fingerprint.name) {
        return Vec::new();
    }

    let bool_count = bool_parameter_count(&function.signature_text);
    if bool_count == 0 {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "public_bool_parameter_api".to_string(),
        severity: if bool_count > 1 {
            Severity::Warning
        } else {
            Severity::Info
        },
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.start_line,
        message: format!(
            "public function {} uses boolean parameter{} to control behavior",
            function.fingerprint.name,
            if bool_count == 1 { "" } else { "s" }
        ),
        evidence: vec![
            format!(
                "signature: {}",
                compact_whitespace(&function.signature_text)
            ),
            format!(
                "observed {} bool parameter{} in the public API",
                bool_count,
                if bool_count == 1 { "" } else { "s" }
            ),
        ],
    }]
}

fn mutable_package_global_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for package_var in &file.package_vars {
        let mut mutation_lines = Vec::new();
        for function in &file.functions {
            if function.is_test_function {
                continue;
            }
            if let Some(line) = mutation_line(function, &package_var.name) {
                mutation_lines.push(line);
            }
        }

        if mutation_lines.is_empty() {
            continue;
        }

        findings.push(Finding {
            rule_id: "mutable_package_global".to_string(),
            severity: if package_var.is_pub {
                Severity::Warning
            } else {
                Severity::Info
            },
            path: file.path.clone(),
            function_name: None,
            start_line: package_var.line,
            end_line: package_var.line,
            message: format!(
                "package variable {} is mutated across function bodies",
                package_var.name
            ),
            evidence: {
                let mut evidence = vec![
                    format!("package variable declared at line {}", package_var.line),
                    format!(
                        "mutation observed at line{} {}",
                        if mutation_lines.len() == 1 { "" } else { "s" },
                        join_lines(&mutation_lines)
                    ),
                ];
                if let Some(type_text) = &package_var.type_text {
                    evidence.push(format!("declared type: {type_text}"));
                }
                if let Some(value_text) = &package_var.value_text {
                    evidence.push(format!("initial value text: {value_text}"));
                }
                evidence
            },
        });
    }

    findings
}

fn single_impl_interface_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (_, package_files) in group_by_package(files) {
        let mut receiver_methods = BTreeMap::<String, BTreeSet<String>>::new();
        let mut interfaces = Vec::<(&ParsedFile, &InterfaceSummary)>::new();
        let mut consumer_counts = BTreeMap::<String, usize>::new();

        for file in &package_files {
            for symbol in &file.symbols {
                if matches!(symbol.kind, SymbolKind::Method)
                    && let (Some(receiver), Some(_)) =
                        (&symbol.receiver_type, symbol.receiver_is_pointer)
                {
                    receiver_methods
                        .entry(receiver.clone())
                        .or_default()
                        .insert(symbol.name.clone());
                }
            }

            for interface in &file.interfaces {
                interfaces.push((file, interface));
            }
        }

        for file in &package_files {
            for function in &file.functions {
                for (_, interface) in &interfaces {
                    if function.signature_text.contains(&interface.name) {
                        *consumer_counts.entry(interface.name.clone()).or_default() += 1;
                    }
                }
            }

            for go_struct in &file.go_structs {
                for field in &go_struct.fields {
                    *consumer_counts.entry(field.type_text.clone()).or_default() += 1;
                }
            }
        }

        for (file, interface) in interfaces {
            if interface.methods.is_empty() {
                continue;
            }

            let impl_candidates = receiver_methods
                .iter()
                .filter(|(receiver, methods)| {
                    *receiver != &interface.name
                        && interface
                            .methods
                            .iter()
                            .all(|method| methods.contains(method))
                })
                .map(|(receiver, _)| receiver.clone())
                .collect::<Vec<_>>();
            let consumer_count = consumer_counts.get(&interface.name).copied().unwrap_or(0);

            if impl_candidates.len() == 1 && consumer_count <= 2 {
                findings.push(Finding {
                    rule_id: "single_impl_interface".to_string(),
                    severity: if interface.is_pub {
                        Severity::Warning
                    } else {
                        Severity::Info
                    },
                    path: file.path.clone(),
                    function_name: None,
                    start_line: interface.line,
                    end_line: interface.line,
                    message: format!(
                        "interface {} currently has one obvious repository-local implementation",
                        interface.name
                    ),
                    evidence: vec![
                        format!("interface methods: {}", interface.methods.join(", ")),
                        format!("implementation candidate: {}", impl_candidates[0]),
                        format!(
                            "observed consumer count for {}: {}",
                            interface.name, consumer_count
                        ),
                    ],
                });
            }
        }
    }

    findings
}

fn passthrough_wrapper_interface_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (_, package_files) in group_by_package(files) {
        let interface_names = package_files
            .iter()
            .flat_map(|file| {
                file.interfaces
                    .iter()
                    .map(|interface| interface.name.clone())
            })
            .collect::<BTreeSet<_>>();

        for file in &package_files {
            for go_struct in &file.go_structs {
                let interface_fields = go_struct
                    .fields
                    .iter()
                    .filter(|field| interface_names.contains(&field.type_text))
                    .collect::<Vec<_>>();
                if interface_fields.is_empty() {
                    continue;
                }

                let forwarding_methods = file
                    .functions
                    .iter()
                    .filter(|function| {
                        function.fingerprint.receiver_type.as_deref()
                            == Some(go_struct.name.as_str())
                            && function.fingerprint.line_count <= 6
                            && function.fingerprint.call_count <= 2
                            && interface_fields.iter().any(|field| {
                                function.body_text.contains(&format!(".{}.", field.name))
                            })
                    })
                    .collect::<Vec<_>>();

                if forwarding_methods.len() >= 2 {
                    findings.push(Finding {
                        rule_id: "passthrough_wrapper_interface".to_string(),
                        severity: if go_struct.is_pub {
                            Severity::Warning
                        } else {
                            Severity::Info
                        },
                        path: file.path.clone(),
                        function_name: None,
                        start_line: go_struct.line,
                        end_line: go_struct.line,
                        message: format!(
                            "struct {} mostly forwards to an interface field with little added policy",
                            go_struct.name
                        ),
                        evidence: vec![
                            format!(
                                "interface-like field{}: {}",
                                if interface_fields.len() == 1 { "" } else { "s" },
                                interface_fields
                                    .iter()
                                    .map(|field| {
                                        format!(
                                            "{} {} (line {}, exported={})",
                                            field.name, field.type_text, field.line, field.is_pub
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            format!(
                                "forwarding-style methods: {}",
                                forwarding_methods
                                    .iter()
                                    .map(|function| function.fingerprint.name.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        ],
                    });
                }
            }
        }
    }

    findings
}

fn write_header_order_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for body_line in lines {
        let Some((receiver, _)) = body_line.text.split_once(".WriteHeader(") else {
            continue;
        };
        let receiver = receiver.trim();
        if receiver.is_empty() {
            continue;
        }

        let prior_write_line = lines
            .iter()
            .find(|candidate| {
                candidate.line < body_line.line
                    && (candidate.text.contains(&format!("{receiver}.Write("))
                        || candidate
                            .text
                            .contains(&format!("NewEncoder({receiver}).Encode("))
                        || candidate.text.contains(&format!("Fprint({receiver},"))
                        || candidate.text.contains(&format!("Fprintf({receiver},"))
                        || candidate.text.contains(&format!("WriteString({receiver},")))
            })
            .map(|candidate| candidate.line);

        if let Some(prior_write_line) = prior_write_line {
            findings.push(Finding {
                rule_id: "http_writeheader_after_write".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} calls {}.WriteHeader after writing the response body",
                    function.fingerprint.name, receiver
                ),
                evidence: vec![
                    format!("response body write observed first at line {prior_write_line}"),
                    format!(
                        "{}.WriteHeader(...) observed later at line {}",
                        receiver, body_line.line
                    ),
                ],
            });
        }
    }

    findings
}

fn mutation_line(function: &ParsedFunction, name: &str) -> Option<usize> {
    body_lines(function)
        .into_iter()
        .find(|body_line| {
            body_line.text.starts_with(&format!("{name} ="))
                || body_line.text.starts_with(&format!("{name} :="))
                || body_line.text.starts_with(&format!("{name} +="))
                || body_line.text.starts_with(&format!("{name} -="))
                || body_line.text.starts_with(&format!("{name}++"))
                || body_line.text.starts_with(&format!("{name}--"))
                || body_line.text.contains(&format!("{name}[")) && body_line.text.contains('=')
        })
        .map(|body_line| body_line.line)
}

fn http_response_bindings(
    lines: &[BodyLine],
    http_aliases: &[String],
) -> Vec<(String, usize, String)> {
    let mut patterns = http_aliases
        .iter()
        .flat_map(|alias| {
            [
                format!("{alias}.Get("),
                format!("{alias}.Head("),
                format!("{alias}.Post("),
                format!("{alias}.PostForm("),
            ]
        })
        .collect::<Vec<_>>();
    patterns.push(".Do(".to_string());

    let string_patterns = patterns.iter().map(String::as_str).collect::<Vec<_>>();
    binding_matches(lines, &string_patterns)
}

fn binding_matches(lines: &[BodyLine], patterns: &[&str]) -> Vec<(String, usize, String)> {
    let mut matches = Vec::new();

    for body_line in lines {
        if let Some((name, target)) = binding_for_patterns(&body_line.text, patterns) {
            matches.push((name, body_line.line, target));
        }
    }

    matches
}

fn binding_for_patterns(text: &str, patterns: &[&str]) -> Option<(String, String)> {
    let (left, right) = split_assignment(text)?;
    let target = patterns
        .iter()
        .find(|pattern| right.contains(**pattern))?
        .to_string();
    let binding = left
        .trim()
        .trim_start_matches("var ")
        .split(',')
        .next()?
        .split_whitespace()
        .next()?
        .trim();
    is_identifier_name(binding).then(|| (binding.to_string(), target))
}

fn split_assignment(text: &str) -> Option<(&str, &str)> {
    if let Some((left, right)) = text.split_once(":=") {
        return Some((left, right));
    }

    if text.contains("==") || text.contains("!=") || text.contains("<=") || text.contains(">=") {
        return None;
    }

    text.split_once(" = ")
        .or_else(|| text.split_once('='))
        .filter(|(left, _)| !left.trim_start().starts_with("if "))
}

fn body_lines(function: &ParsedFunction) -> Vec<BodyLine> {
    let mut brace_depth = 0usize;
    let mut loop_exit_depths = Vec::new();
    let mut lines = Vec::new();

    for (offset, raw_line) in function.body_text.lines().enumerate() {
        let line_no = function.body_start_line + offset;
        let stripped = strip_line_comment(raw_line).trim().to_string();
        let closing_braces = stripped
            .chars()
            .filter(|character| *character == '}')
            .count();
        for _ in 0..closing_braces {
            brace_depth = brace_depth.saturating_sub(1);
            while loop_exit_depths
                .last()
                .is_some_and(|exit_depth| *exit_depth > brace_depth)
            {
                loop_exit_depths.pop();
            }
        }

        let starts_loop = contains_keyword(&stripped, "for");
        let in_loop = !loop_exit_depths.is_empty() || starts_loop;
        let opening_braces = stripped
            .chars()
            .filter(|character| *character == '{')
            .count();
        if starts_loop {
            loop_exit_depths.push(brace_depth + opening_braces.max(1));
        }

        brace_depth += opening_braces;
        lines.push(BodyLine {
            line: line_no,
            text: stripped,
            in_loop,
        });
    }

    lines
}

fn composite_literal_blocks(function: &ParsedFunction, marker: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let lines = function.body_text.lines().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < lines.len() {
        let line = strip_line_comment(lines[index]);
        if !line.contains(marker) && !line.contains(&format!("&{marker}")) {
            index += 1;
            continue;
        }

        let start_line = function.body_start_line + index;
        let mut text = String::new();
        let mut brace_balance = 0isize;
        let mut started = false;

        while index < lines.len() {
            let candidate = strip_line_comment(lines[index]);
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(candidate.trim());
            for character in candidate.chars() {
                match character {
                    '{' => {
                        brace_balance += 1;
                        started = true;
                    }
                    '}' => brace_balance -= 1,
                    _ => {}
                }
            }
            index += 1;
            if started && brace_balance <= 0 {
                break;
            }
        }

        blocks.push((start_line, text));
    }

    blocks
}

fn import_aliases_for(file: &ParsedFile, import_path: &str) -> Vec<String> {
    file.imports
        .iter()
        .filter(|import| import.path == import_path)
        .map(|import| import.alias.clone())
        .collect()
}

fn contains_text(lines: &[BodyLine], needle: &str) -> bool {
    lines
        .iter()
        .any(|body_line| body_line.text.contains(needle))
}

fn lines_for(lines: &[BodyLine], needle: &str) -> Vec<usize> {
    lines
        .iter()
        .filter(|body_line| body_line.text.contains(needle))
        .map(|body_line| body_line.line)
        .collect()
}

fn returns_binding(function: &ParsedFunction, name: &str) -> bool {
    function
        .body_text
        .lines()
        .any(|line| strip_line_comment(line).contains(&format!("return {name}")))
}

fn bool_parameter_count(signature_text: &str) -> usize {
    let Some(parameters_start) = signature_text.find('(') else {
        return 0;
    };
    let Some(parameters_end) = signature_text.rfind(')') else {
        return 0;
    };
    let parameters = &signature_text[parameters_start + 1..parameters_end];
    parameters
        .split(',')
        .map(str::trim)
        .filter(|segment| !segment.is_empty() && segment.ends_with(" bool"))
        .count()
}

fn is_identifier_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && text
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
}

fn is_exported_name(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
}

fn compact_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn join_lines(lines: &[usize]) -> String {
    lines
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn contains_keyword(line: &str, keyword: &str) -> bool {
    let bytes = line.as_bytes();
    let keyword_bytes = keyword.as_bytes();

    if keyword_bytes.is_empty() || bytes.len() < keyword_bytes.len() {
        return false;
    }

    for start in 0..=bytes.len() - keyword_bytes.len() {
        if &bytes[start..start + keyword_bytes.len()] != keyword_bytes {
            continue;
        }

        let left_ok =
            start == 0 || (!bytes[start - 1].is_ascii_alphanumeric() && bytes[start - 1] != b'_');
        let right_index = start + keyword_bytes.len();
        let right_ok = right_index == bytes.len()
            || (!bytes[right_index].is_ascii_alphanumeric() && bytes[right_index] != b'_');

        if left_ok && right_ok {
            return true;
        }
    }

    false
}

fn strip_line_comment(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

fn group_by_package<'a>(
    files: &'a [&'a ParsedFile],
) -> BTreeMap<(PathBuf, String), Vec<&'a ParsedFile>> {
    let mut groups = BTreeMap::<(PathBuf, String), Vec<&ParsedFile>>::new();
    for file in files {
        let package_name = file
            .package_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let directory = file.path.parent().map(PathBuf::from).unwrap_or_default();
        groups
            .entry((directory, package_name))
            .or_default()
            .push(*file);
    }
    groups
}
