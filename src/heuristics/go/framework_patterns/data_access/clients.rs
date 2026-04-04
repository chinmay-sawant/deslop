use super::*;

pub(crate) const BINDING_LOCATION: &str = super::BINDING_LOCATION;

pub(super) fn pool_lifecycle_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    request_path: bool,
) -> Vec<Finding> {
    if !request_path {
        return Vec::new();
    }

    let mut findings = Vec::new();

    if has_sql_like_import(file) {
        for call in &function.calls {
            if matches!(
                call.name.as_str(),
                "SetMaxOpenConns" | "SetMaxIdleConns" | "SetConnMaxLifetime" | "SetConnMaxIdleTime"
            ) {
                findings.push(Finding {
                    rule_id: "connection_pool_reconfigured_per_request".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: call.line,
                    end_line: call.line,
                    message: format!(
                        "function {} reconfigures a DB pool on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!(
                            "{}{} observed at line {}",
                            call.receiver
                                .as_deref()
                                .map(|receiver| format!("{receiver}."))
                                .unwrap_or_default(),
                            call.name,
                            call.line
                        ),
                        "connection-pool sizing and lifetime settings are usually process-level configuration"
                            .to_string(),
                    ],
                });
            }
        }
    }

    for alias in import_aliases_for(file, "gorm.io/gorm") {
        for body_line in lines {
            if body_line.text.contains(&format!("{alias}.Open(")) {
                findings.push(Finding {
                    rule_id: "gorm_open_per_request".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} opens a GORM handle on a request path",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Open(...) observed at line {}", alias, body_line.line),
                        "ORM handles and underlying pools are usually reused instead of created per request"
                            .to_string(),
                    ],
                });
            }
        }
    }

    findings
}

pub(super) fn service_client_lifecycle_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    request_path: bool,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let pgxpool_aliases = pgxpool_import_aliases(file);
    if request_path {
        for alias in &pgxpool_aliases {
            for body_line in lines {
                if body_line.text.contains(&format!("{alias}.New("))
                    || body_line.text.contains(&format!("{alias}.NewWithConfig("))
                {
                    findings.push(Finding {
                        rule_id: "pgxpool_new_per_request".to_string(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} creates a pgxpool pool on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "{}.New-like call observed at line {}",
                                alias, body_line.line
                            ),
                            "pgxpool pools are usually initialized once and reused across requests"
                                .to_string(),
                        ],
                    });
                }
            }
        }

        if !pgxpool_aliases.is_empty() {
            for body_line in lines {
                if body_line.text.contains(".Ping(") {
                    findings.push(Finding {
                        rule_id: "pgxpool_ping_per_request".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} checks pgxpool connectivity on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("Ping-like pool connectivity check observed at line {}", body_line.line),
                            "connectivity probes are usually better handled at startup or on health-check endpoints"
                                .to_string(),
                        ],
                    });
                    break;
                }
            }
        }
    }

    for body_line in lines.iter().filter(|line| line.in_loop) {
        if !pgxpool_aliases.is_empty() && body_line.text.contains(".Acquire(") {
            findings.push(Finding {
                rule_id: "pgxpool_acquire_in_loop".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} acquires pgxpool resources inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("Acquire observed inside a loop at line {}", body_line.line),
                    "batching or hoisting the acquire outside the loop often reduces pool churn"
                        .to_string(),
                ],
            });
            break;
        }
    }

    let redis_aliases = redis_import_aliases(file);
    if request_path {
        for alias in &redis_aliases {
            for body_line in lines {
                if body_line.text.contains(&format!("{alias}.NewClient("))
                    || body_line
                        .text
                        .contains(&format!("{alias}.NewClusterClient("))
                    || body_line
                        .text
                        .contains(&format!("{alias}.NewFailoverClient("))
                    || body_line.text.contains(&format!("{alias}.NewRing("))
                {
                    findings.push(Finding {
                        rule_id: "redis_client_created_per_request".to_string(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} creates a Redis client on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("{}.NewClient-like call observed at line {}", alias, body_line.line),
                            "Redis clients are usually process-level singletons reused across requests"
                                .to_string(),
                        ],
                    });
                }
            }
        }

        if !redis_aliases.is_empty() {
            for body_line in lines {
                if body_line.text.contains(".Ping(") {
                    findings.push(Finding {
                        rule_id: "redis_ping_per_request".to_string(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} pings Redis on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("Redis Ping observed at line {}", body_line.line),
                            "per-request Redis health checks add latency and are usually better handled elsewhere"
                                .to_string(),
                        ],
                    });
                    break;
                }
            }
        }
    }

    if !redis_aliases.is_empty()
        && !function.body_text.contains("Pipeline(")
        && !function.body_text.contains("Pipelined(")
        && !function.body_text.contains("TxPipeline(")
        && !function.body_text.contains("TxPipelined(")
    {
        for body_line in lines.iter().filter(|line| line.in_loop) {
            if [
                ".Get(", ".Set(", ".Del(", ".HGet(", ".HSet(", ".Incr(", ".Exists(",
            ]
            .iter()
            .any(|marker| body_line.text.contains(marker))
            {
                findings.push(Finding {
                    rule_id: "redis_command_loop_without_pipeline".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} issues Redis commands inside a loop without pipeline usage",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("Redis command observed inside a loop at line {}", body_line.line),
                        "pipelining usually reduces round-trips when many commands are issued in one batch"
                            .to_string(),
                    ],
                });
                break;
            }
        }
    }

    if request_path {
        for alias in import_aliases_for(file, "github.com/uptrace/bun") {
            for body_line in lines {
                if body_line.text.contains(&format!("{alias}.NewDB(")) {
                    findings.push(Finding {
                        rule_id: "bun_newdb_per_request".to_string(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} creates a Bun DB handle on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("{}.NewDB observed at line {}", alias, body_line.line),
                            "Bun DB handles and underlying pools are usually reused rather than created per request"
                                .to_string(),
                        ],
                    });
                }
            }
        }

        if has_import_path(file, "github.com/uptrace/bun")
            && function.body_text.contains("NewSelect(")
            && function.body_text.contains(".Scan(")
            && !function.body_text.contains(".Limit(")
            && let Some(line) = lines
                .iter()
                .find(|line| line.text.contains("NewSelect("))
                .map(|line| line.line)
        {
            findings.push(Finding {
                rule_id: "bun_select_scan_without_limit".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: line,
                end_line: line,
                message: format!(
                    "function {} scans a Bun select without a visible limit on a request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("NewSelect/Scan chain observed at line {}", line),
                    "request-path scans usually benefit from LIMIT, pagination, or cursor iteration"
                        .to_string(),
                ],
            });
        }

        for alias in ent_import_aliases(file) {
            for body_line in lines {
                if body_line.text.contains(&format!("{alias}.Open(")) {
                    findings.push(Finding {
                        rule_id: "ent_open_per_request".to_string(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} opens an ent client on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("{}.Open observed at line {}", alias, body_line.line),
                            "ent clients are usually initialized once and reused with the process-level DB pool"
                                .to_string(),
                        ],
                    });
                }
            }
        }
    }

    findings
}

fn pgxpool_import_aliases(file: &ParsedFile) -> Vec<String> {
    file.imports
        .iter()
        .filter(|import| {
            matches!(
                import.path.as_str(),
                "github.com/jackc/pgx/v5/pgxpool" | "github.com/jackc/pgx/v4/pgxpool"
            )
        })
        .map(|import| import.alias.clone())
        .collect()
}

fn redis_import_aliases(file: &ParsedFile) -> Vec<String> {
    file.imports
        .iter()
        .filter(|import| {
            matches!(
                import.path.as_str(),
                "github.com/redis/go-redis/v9"
                    | "github.com/go-redis/redis/v8"
                    | "github.com/go-redis/redis/v9"
            )
        })
        .map(|import| import.alias.clone())
        .collect()
}

fn ent_import_aliases(file: &ParsedFile) -> Vec<String> {
    file.imports
        .iter()
        .filter(|import| import.path == "entgo.io/ent" || import.path.ends_with("/ent"))
        .map(|import| import.alias.clone())
        .collect()
}
