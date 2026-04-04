use super::*;

pub(crate) const BINDING_LOCATION: &str = super::BINDING_LOCATION;

pub(super) fn gorm_chain_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    request_path: bool,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    if !has_import_path(file, "gorm.io/gorm") {
        return findings;
    }

    let go = function.go_evidence();

    if request_path {
        for chain in go.gorm_query_chains {
            if let Some(debug_step) = gorm_chain_step(chain, "Debug") {
                findings.push(Finding {
                    rule_id: "gorm_debug_enabled_in_request_path".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: debug_step.line,
                    end_line: debug_step.line,
                    message: format!(
                        "function {} enables GORM debug logging on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("Debug() observed in GORM chain at line {}", debug_step.line),
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                    ],
                });
            }

            let suppress_limit_rules = is_likely_non_request_workload(file, function);

            if chain.terminal_method == "Find"
                && !gorm_chain_has_step(chain, "Limit")
                && !suppress_limit_rules
            {
                findings.push(Finding {
                    rule_id: "gorm_find_without_limit_on_handler_path".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: chain.line,
                    end_line: chain.line,
                    message: format!(
                        "function {} runs a GORM Find chain on a request path without an observed Limit",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                        "no Limit(...) step was observed before Find(...)".to_string(),
                    ],
                });
            }

            if chain.terminal_method == "Find"
                && !suppress_limit_rules
                && let Some(offset_step) = gorm_chain_step(chain, "Offset")
                && offset_step
                    .argument_texts
                    .first()
                    .is_none_or(|argument| argument != "0")
            {
                findings.push(Finding {
                    rule_id: "offset_pagination_on_large_table".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: offset_step.line,
                    end_line: offset_step.line,
                    message: format!(
                        "function {} uses offset pagination in a GORM Find chain",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("Offset step observed at line {}", offset_step.line),
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                    ],
                });
            }

            if chain.terminal_method == "Find" && is_wide_preload_chain(chain) {
                findings.push(Finding {
                    rule_id: "gorm_preload_clause_associations_on_wide_graph".to_string(),
                    severity: if gorm_chain_has_step(chain, "Limit") {
                        Severity::Info
                    } else {
                        Severity::Warning
                    },
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: chain.line,
                    end_line: chain.line,
                    message: format!(
                        "function {} uses a broad GORM preload graph on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                        "broad Preload(...) chains can widen result sets and round trips significantly"
                            .to_string(),
                    ],
                });
            }
        }

        findings.extend(count_then_find_same_filter_findings(file, function));
    }

    let has_batch_create = go
        .gorm_query_chains
        .iter()
        .any(|chain| chain.terminal_method == "CreateInBatches");
    if !has_batch_create {
        for chain in go
            .gorm_query_chains
            .iter()
            .filter(|chain| chain.in_loop && chain.terminal_method == "Create")
        {
            findings.push(Finding {
                rule_id: "create_single_in_loop_instead_of_batches".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} inserts single GORM rows inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "no CreateInBatches(...) chain was observed in the same function".to_string(),
                ],
            });
        }
    }

    findings
}

pub(super) fn gorm_loop_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !has_import_path(file, "gorm.io/gorm") {
        return Vec::new();
    }

    let go = function.go_evidence();
    let mut findings = Vec::new();

    for chain in go
        .gorm_query_chains
        .iter()
        .filter(|chain| chain.in_loop && chain.terminal_method == "Count")
    {
        findings.push(Finding {
            rule_id: "count_inside_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: chain.line,
            end_line: chain.line,
            message: format!(
                "function {} runs a GORM Count chain inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
        });
    }

    for chain in go
        .gorm_query_chains
        .iter()
        .filter(|chain| chain.in_loop && gorm_chain_has_step(chain, "Session"))
    {
        findings.push(Finding {
            rule_id: "gorm_session_allocated_per_item".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: chain.line,
            end_line: chain.line,
            message: format!(
                "function {} allocates GORM sessions inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
        });
    }

    for chain in go
        .gorm_query_chains
        .iter()
        .filter(|chain| chain.in_loop && gorm_chain_has_step(chain, "Preload"))
    {
        findings.push(Finding {
            rule_id: "preload_inside_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: chain.line,
            end_line: chain.line,
            message: format!(
                "function {} issues preloaded GORM queries inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
        });
    }

    for chain in go.gorm_query_chains.iter().filter(|chain| {
        chain.in_loop && chain.terminal_method == "Scan" && gorm_chain_has_step(chain, "Raw")
    }) {
        findings.push(Finding {
            rule_id: "raw_scan_inside_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: chain.line,
            end_line: chain.line,
            message: format!(
                "function {} runs raw SQL scans inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
        });
    }

    for chain in go.gorm_query_chains.iter().filter(|chain| {
        chain.in_loop
            && chain.terminal_method == "Find"
            && gorm_chain_has_step(chain, "Association")
    }) {
        findings.push(Finding {
            rule_id: "association_find_inside_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: chain.line,
            end_line: chain.line,
            message: format!(
                "function {} loads GORM associations inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
        });
    }

    for chain in go
        .gorm_query_chains
        .iter()
        .filter(|chain| chain.in_loop && chain.terminal_method == "FirstOrCreate")
    {
        findings.push(Finding {
            rule_id: "first_or_create_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: chain.line,
            end_line: chain.line,
            message: format!(
                "function {} runs FirstOrCreate inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
        });
    }

    for chain in go
        .gorm_query_chains
        .iter()
        .filter(|chain| chain.in_loop && chain.terminal_method == "Save")
    {
        findings.push(Finding {
            rule_id: "save_in_loop_full_model".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: chain.line,
            end_line: chain.line,
            message: format!(
                "function {} saves full GORM models inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
        });
    }

    for chain in go.gorm_query_chains.iter().filter(|chain| {
        chain.in_loop
            && matches!(
                chain.terminal_method.as_str(),
                "Update" | "UpdateColumn" | "Updates"
            )
    }) {
        findings.push(Finding {
            rule_id: "update_single_row_in_loop_without_batch".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: chain.line,
            end_line: chain.line,
            message: format!(
                "function {} updates rows one at a time inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
        });
    }

    for chain in go
        .gorm_query_chains
        .iter()
        .filter(|chain| chain.in_loop && chain.terminal_method == "Delete")
    {
        findings.push(Finding {
            rule_id: "delete_single_row_in_loop_without_batch".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: chain.line,
            end_line: chain.line,
            message: format!(
                "function {} deletes rows one at a time inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![format!("chain shape: {}", gorm_chain_shape(chain))],
        });
    }

    findings
}

fn count_then_find_same_filter_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let count_chains = go
        .gorm_query_chains
        .iter()
        .filter(|chain| !chain.in_loop && chain.terminal_method == "Count")
        .collect::<Vec<_>>();
    let find_chains = go
        .gorm_query_chains
        .iter()
        .filter(|chain| {
            !chain.in_loop
                && chain.terminal_method == "Find"
                && !gorm_chain_has_step(chain, "Limit")
                && !gorm_chain_has_step(chain, "Offset")
        })
        .collect::<Vec<_>>();

    let mut findings = Vec::new();
    for count_chain in count_chains {
        let count_shape = gorm_shape_key(count_chain);
        if count_shape.is_empty() {
            continue;
        }

        let Some(find_chain) = find_chains.iter().copied().find(|find_chain| {
            find_chain.line > count_chain.line
                && find_chain.root_text == count_chain.root_text
                && gorm_shape_key(find_chain) == count_shape
        }) else {
            continue;
        };

        findings.push(Finding {
            rule_id: "count_then_find_same_filter".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: find_chain.line,
            end_line: find_chain.line,
            message: format!(
                "function {} runs Count and Find with the same broad GORM filter chain",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "Count chain at line {}: {}",
                    count_chain.line,
                    gorm_chain_shape(count_chain)
                ),
                format!(
                    "Find chain at line {}: {}",
                    find_chain.line,
                    gorm_chain_shape(find_chain)
                ),
            ],
        });
    }

    findings
}

fn is_wide_preload_chain(chain: &GormQueryChain) -> bool {
    let preload_steps = chain
        .steps
        .iter()
        .filter(|step| step.method_name == "Preload")
        .collect::<Vec<_>>();

    preload_steps.len() >= 3
        || preload_steps.iter().any(|step| {
            step.first_string_arg.as_deref() == Some("clause.Associations")
                || step
                    .argument_texts
                    .first()
                    .is_some_and(|argument| argument.contains("clause.Associations"))
        })
}

fn gorm_shape_key(chain: &GormQueryChain) -> String {
    chain
        .steps
        .iter()
        .filter(|step| {
            !matches!(
                step.method_name.as_str(),
                "Count" | "Find" | "Limit" | "Offset" | "Order"
            )
        })
        .map(|step| {
            let first_argument = step.argument_texts.first().cloned().unwrap_or_default();
            format!("{}({})", step.method_name, first_argument)
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}

pub(super) fn gorm_chain_shape(chain: &GormQueryChain) -> String {
    chain
        .steps
        .iter()
        .map(|step| {
            if step.argument_texts.is_empty() {
                step.method_name.clone()
            } else {
                format!("{}({})", step.method_name, step.argument_texts.join(", "))
            }
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}

pub(super) fn gorm_chain_has_step(chain: &GormQueryChain, method_name: &str) -> bool {
    chain
        .steps
        .iter()
        .any(|step| step.method_name == method_name)
}

pub(super) fn gorm_chain_step<'a>(
    chain: &'a GormQueryChain,
    method_name: &str,
) -> Option<&'a GormChainStep> {
    chain
        .steps
        .iter()
        .find(|step| step.method_name == method_name)
}

pub(super) fn find_all_then_manual_paginate_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    for chain in go.gorm_query_chains.iter().filter(|c| {
        c.terminal_method == "Find"
            && !gorm_chain_has_step(c, "Limit")
            && !gorm_chain_has_step(c, "Offset")
    }) {
        let has_manual_slice = lines.iter().any(|bl| {
            bl.line > chain.line
                && (bl.text.contains("[:")
                    || bl.text.contains("[offset:")
                    || bl.text.contains("[start:"))
        });

        if has_manual_slice {
            findings.push(Finding {
                rule_id: "find_all_then_manual_paginate_in_go".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} fetches all rows and then slices them in Go",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "database-level Limit/Offset or keyset pagination is usually cheaper than fetching all rows and slicing in Go".to_string(),
                ],
            });
        }
    }

    findings
}

pub(super) fn duplicate_find_then_preload_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let go = function.go_evidence();
    let mut findings = Vec::new();

    let plain_finds: Vec<_> = go
        .gorm_query_chains
        .iter()
        .filter(|c| c.terminal_method == "Find" && !gorm_chain_has_step(c, "Preload"))
        .collect();

    let preload_finds: Vec<_> = go
        .gorm_query_chains
        .iter()
        .filter(|c| {
            (c.terminal_method == "Find" || c.terminal_method == "First")
                && gorm_chain_has_step(c, "Preload")
        })
        .collect();

    for plain in &plain_finds {
        for preload in &preload_finds {
            if preload.line > plain.line && preload.root_text == plain.root_text {
                findings.push(Finding {
                    rule_id: "duplicate_find_then_preload_followup".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: preload.line,
                    end_line: preload.line,
                    message: format!(
                        "function {} fetches rows and then follows up with a separate preload query",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("initial Find at line {}: {}", plain.line, gorm_chain_shape(plain)),
                        format!("follow-up Preload+Find at line {}: {}", preload.line, gorm_chain_shape(preload)),
                        "folding the preload into the initial query usually reduces round trips".to_string(),
                    ],
                });
            }
        }
    }

    findings
}

pub(super) fn gorm_query_shape_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    request_path: bool,
) -> Vec<Finding> {
    if !has_import_path(file, "gorm.io/gorm") || !request_path {
        return Vec::new();
    }

    let go = function.go_evidence();
    let mut findings = Vec::new();

    for chain in go.gorm_query_chains.iter() {
        if chain.terminal_method == "Find"
            && !gorm_chain_has_step(chain, "Select")
            && !gorm_chain_has_step(chain, "Omit")
            && gorm_chain_has_step(chain, "Where")
        {
            let has_many_fields = chain.steps.len() >= 3;
            if has_many_fields {
                findings.push(Finding {
                    rule_id: "gorm_select_missing_projection_on_wide_model".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: chain.line,
                    end_line: chain.line,
                    message: format!(
                        "function {} fetches wide models without Select or Omit projection",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("chain shape: {}", gorm_chain_shape(chain)),
                        "adding Select(...) or Omit(...) to narrow the result set can reduce transfer and allocation cost".to_string(),
                    ],
                });
            }
        }

        if chain.terminal_method == "Find"
            && gorm_chain_has_step(chain, "Joins")
            && gorm_chain_has_step(chain, "Preload")
            && !gorm_chain_has_step(chain, "Limit")
        {
            findings.push(Finding {
                rule_id: "gorm_joins_plus_preload_plus_find_without_limit".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} combines Joins, Preload, and unbounded Find on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "Joins + Preload + Find without Limit can produce very large result sets and round trips".to_string(),
                ],
            });
        }

        if gorm_chain_has_step(chain, "Order")
            && !gorm_chain_has_step(chain, "Limit")
            && matches!(chain.terminal_method.as_str(), "Find" | "Scan")
        {
            findings.push(Finding {
                rule_id: "order_by_without_limit_orm_chain".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} orders results without a Limit on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "ordering without a limit can cause expensive full-table sorts on large datasets".to_string(),
                ],
            });
        }

        if let Some(order_step) = gorm_chain_step(chain, "Order") {
            let order_text = order_step
                .argument_texts
                .first()
                .cloned()
                .unwrap_or_default()
                .to_ascii_uppercase();
            if order_text.contains("RAND()") || order_text.contains("RANDOM()") {
                findings.push(Finding {
                    rule_id: "order_by_random_request_path".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: order_step.line,
                    end_line: order_step.line,
                    message: format!(
                        "function {} orders by random on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("ORDER BY RAND()/RANDOM() observed at line {}", order_step.line),
                        "random ordering requires a full table scan and sort, which is expensive on large tables".to_string(),
                    ],
                });
            }
        }

        if gorm_chain_has_step(chain, "Distinct")
            && !gorm_chain_has_step(chain, "Select")
            && matches!(chain.terminal_method.as_str(), "Find" | "Scan")
        {
            findings.push(Finding {
                rule_id: "distinct_wide_row_request_path".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} uses Distinct on wide rows without Select projection",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "Distinct on wide rows is expensive; a key-only subquery is usually cheaper"
                        .to_string(),
                ],
            });
        }
    }

    findings
}

pub(super) fn orm_tuning_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    request_path: bool,
) -> Vec<Finding> {
    if !has_import_path(file, "gorm.io/gorm") {
        return Vec::new();
    }

    let go = function.go_evidence();
    let mut findings = Vec::new();

    let has_bulk_create = go.gorm_query_chains.iter().any(|c| {
        c.terminal_method == "Create"
            && c.steps.iter().any(|s| {
                s.argument_texts
                    .first()
                    .is_some_and(|a| a.contains("[]") || a.contains("..."))
            })
    });
    if has_bulk_create {
        let uses_default_tx = !function.body_text.contains("SkipDefaultTransaction")
            && !function.body_text.contains("Session(");
        if uses_default_tx {
            let anchor = go
                .gorm_query_chains
                .iter()
                .find(|c| c.terminal_method == "Create")
                .map(|c| c.line)
                .unwrap_or(function.body_start_line);

            findings.push(Finding {
                rule_id: "default_transaction_enabled_for_bulk_create".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: anchor,
                end_line: anchor,
                message: format!(
                    "function {} performs bulk creates with default transaction enabled",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("bulk Create observed at line {anchor}"),
                    "disabling the default transaction via Session(&gorm.Session{{SkipDefaultTransaction: true}}) can improve bulk insert throughput".to_string(),
                ],
            });
        }
    }

    for chain in go
        .gorm_query_chains
        .iter()
        .filter(|c| c.terminal_method == "Save")
    {
        let update_count = function
            .body_text
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.contains(". =") || trimmed.contains(" = ")
            })
            .count();

        if update_count <= 2 && update_count > 0 {
            findings.push(Finding {
                rule_id: "save_for_single_column_change".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: chain.line,
                end_line: chain.line,
                message: format!(
                    "function {} uses Save for a single-column update",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("chain shape: {}", gorm_chain_shape(chain)),
                    "Update or Updates for specific columns is cheaper than Save which writes the full model".to_string(),
                ],
            });
        }
    }

    for body_line in lines.iter().filter(|bl| bl.in_loop) {
        if body_line.text.contains(".Updates(map[string]") {
            findings.push(Finding {
                rule_id: "updates_map_allocated_per_row".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} allocates update maps inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("Updates(map[string]...) inside a loop at line {}", body_line.line),
                    "reusing an update map or using struct updates reduces per-row allocation churn".to_string(),
                ],
            });
            break;
        }
    }

    if request_path {
        let has_findinbatches = go
            .gorm_query_chains
            .iter()
            .any(|c| c.terminal_method == "FindInBatches");
        if !has_findinbatches {
            for chain in go.gorm_query_chains.iter().filter(|c| {
                c.terminal_method == "Find"
                    && !gorm_chain_has_step(c, "Limit")
                    && gorm_chain_has_step(c, "Where")
            }) {
                let step_count = chain.steps.len();
                if step_count >= 2 {
                    findings.push(Finding {
                        rule_id: "findinbatches_candidate_for_large_scan".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: chain.line,
                        end_line: chain.line,
                        message: format!(
                            "function {} fetches unbounded results that could use FindInBatches",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("chain shape: {}", gorm_chain_shape(chain)),
                            "FindInBatches or cursor iteration avoids materializing the full result set at once".to_string(),
                        ],
                    });
                }
            }
        }
    }

    if request_path {
        for body_line in lines {
            if body_line.text.contains(".AutoMigrate(") || body_line.text.contains(".Migrator()") {
                findings.push(Finding {
                    rule_id: "automigrate_or_schema_probe_in_request_path".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} runs AutoMigrate or schema probes on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("migration/schema probe observed at line {}", body_line.line),
                        "schema migrations and introspection belong at startup, not on request paths".to_string(),
                    ],
                });
                break;
            }
        }
    }

    findings
}
