use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::super::framework_patterns::{
    BodyLine, has_import_path, import_aliases_for, is_request_path_function,
};

pub(super) fn library_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(redis_findings(file, function, lines));
    findings.extend(grpc_findings(file, function, lines));
    findings.extend(logging_findings(file, function, lines));
    findings.extend(config_cli_findings(file, function, lines));
    findings.extend(prometheus_findings(file, function, lines));
    findings.extend(aws_findings(file, function, lines));
    findings
}

// ── Section A — Redis ──

fn redis_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let redis_paths = [
        "github.com/go-redis/redis",
        "github.com/go-redis/redis/v8",
        "github.com/go-redis/redis/v9",
        "github.com/redis/go-redis/v9",
    ];
    let mut aliases: Vec<String> = Vec::new();
    for path in &redis_paths {
        aliases.extend(import_aliases_for(file, path));
    }
    if aliases.is_empty() {
        return findings;
    }

    // A1 redis_ping_per_request
    if is_request_path_function(file, function) {
        for alias in &aliases {
            for bl in lines {
                if bl.text.contains(&format!("{alias}.Ping("))
                    || bl.text.contains(".Ping(ctx)")
                    || bl.text.contains(".Ping(c)")
                {
                    findings.push(Finding {
                        rule_id: "redis_ping_per_request".into(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} pings Redis on every request",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("Redis PING in handler at line {}", bl.line),
                            "connection pool validates connections; ping only at startup".into(),
                        ],
                    });
                }
            }
        }
    }

    // A2 redis_get_set_without_pipeline
    let mut redis_cmd_lines: Vec<usize> = Vec::new();
    for bl in lines {
        let t = &bl.text;
        let is_redis_cmd = aliases.iter().any(|alias| {
            t.contains(&format!("{alias}.Get(")) || t.contains(&format!("{alias}.Set("))
        }) || t.contains(".Get(r.Context()")
            || t.contains(".Set(r.Context()")
            || t.contains(".Get(ctx,")
            || t.contains(".Set(ctx,")
            || t.contains(".Get(c,")
            || t.contains(".Set(c,");
        if is_redis_cmd {
            redis_cmd_lines.push(bl.line);
        }
    }
    if redis_cmd_lines.len() >= 3 {
        let has_pipeline = lines.iter().any(|l| {
            l.text.contains("Pipeline()")
                || l.text.contains("Pipelined(")
                || l.text.contains("TxPipelined(")
        });
        if !has_pipeline {
            findings.push(Finding {
                rule_id: "redis_get_set_without_pipeline".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: redis_cmd_lines[0],
                end_line: redis_cmd_lines[redis_cmd_lines.len() - 1],
                message: format!(
                    "function {} makes multiple sequential Redis calls without pipeline",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("{} Redis commands without pipeline", redis_cmd_lines.len()),
                    "pipelining batches into 1 RTT".into(),
                ],
            });
        }
    }

    // A3 redis_keys_command
    for bl in lines {
        if bl.text.contains(".Keys(") {
            findings.push(Finding {
                rule_id: "redis_keys_command_in_handler".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses Redis KEYS command",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("KEYS at line {}", bl.line),
                    "KEYS blocks Redis for O(n); use SCAN instead".into(),
                ],
            });
        }
    }

    // A4 redis_connection_per_request
    if is_request_path_function(file, function) {
        for bl in lines {
            if bl.text.contains("redis.NewClient(") || bl.text.contains("redis.NewClusterClient(") {
                findings.push(Finding {
                    rule_id: "redis_connection_per_request".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} creates Redis client per request",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("NewClient in handler at line {}", bl.line),
                        "creates new connection pool per request".into(),
                    ],
                });
            }
        }
    }

    // A6 redis_no_ttl
    for bl in lines {
        if bl.text.contains(".Set(") && (bl.text.contains(", 0)") || bl.text.contains("KeepTTL")) {
            findings.push(Finding {
                rule_id: "redis_no_ttl_on_cache_keys".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} sets Redis key without TTL",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("Set with 0/KeepTTL at line {}", bl.line),
                    "cache keys without TTL grow indefinitely".into(),
                ],
            });
        }
    }

    findings.extend(redis_large_value_without_compression(file, function, lines));

    findings
}

fn redis_large_value_without_compression(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let Some(payload_line) = lines
        .iter()
        .find(|line| line.text.contains("json.Marshal("))
    else {
        return findings;
    };
    let has_compression = lines.iter().any(|line| {
        line.text.contains("gzip.NewWriter(")
            || line.text.contains("snappy.")
            || line.text.contains("zstd.")
    });
    if has_compression {
        return findings;
    }
    for bl in lines {
        if bl.text.contains(".Set(")
            && (bl.text.contains("payload") || bl.text.contains("jsonData"))
        {
            findings.push(Finding {
                rule_id: "redis_large_value_without_compression".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: payload_line.line,
                end_line: bl.line,
                message: format!(
                    "function {} stores marshaled payloads in Redis without compression",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("json.Marshal at line {}, Redis Set at line {}", payload_line.line, bl.line),
                    "large JSON cache values are often far smaller after compression, saving Redis memory and bandwidth"
                        .into(),
                ],
            });
        }
    }
    findings
}

// ── Section B — gRPC ──

fn grpc_findings(file: &ParsedFile, function: &ParsedFunction, lines: &[BodyLine]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let grpc_paths = ["google.golang.org/grpc"];
    let mut aliases: Vec<String> = Vec::new();
    for path in &grpc_paths {
        aliases.extend(import_aliases_for(file, path));
    }

    // B1 grpc_dial_per_request
    if is_request_path_function(file, function) {
        for alias in &aliases {
            for bl in lines {
                if bl.text.contains(&format!("{alias}.Dial("))
                    || bl.text.contains(&format!("{alias}.NewClient("))
                {
                    findings.push(Finding {
                        rule_id: "grpc_dial_per_request".into(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} creates gRPC connection per request",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("grpc.Dial in handler at line {}", bl.line),
                            "connection setup costs ~10-50ms".into(),
                        ],
                    });
                }
            }
        }
    }

    // B3 grpc_context_not_propagated
    let sig = &function.signature_text;
    let is_grpc_handler =
        sig.contains("context.Context") && (sig.contains("Request") || sig.contains("Stream"));
    if is_grpc_handler {
        for bl in lines {
            if bl.text.contains("context.Background()") || bl.text.contains("context.TODO()") {
                findings.push(Finding {
                    rule_id: "grpc_context_not_propagated".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} creates new context instead of propagating gRPC context",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("context.Background() in gRPC handler at line {}", bl.line),
                        "drops client deadline and cancellation".into(),
                    ],
                });
            }
        }
    }

    findings.extend(grpc_large_message_without_streaming(file, function, lines));
    findings.extend(grpc_no_keepalive_config(file, function, lines));
    findings.extend(grpc_unary_interceptor_per_rpc(file, function, lines));

    findings
}

fn grpc_large_message_without_streaming(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let sig = &function.signature_text;
    let is_grpc_handler =
        sig.contains("context.Context") && (sig.contains("Request") || sig.contains("Stream"));
    if !is_grpc_handler || lines.iter().any(|line| line.text.contains(".Send(")) {
        return findings;
    }
    for bl in lines {
        if bl.text.contains("make([]")
            && (bl.text.contains("10000") || bl.text.contains("20000") || bl.text.contains("50000"))
        {
            findings.push(Finding {
                rule_id: "grpc_large_message_without_streaming".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "gRPC handler {} appears to build a very large unary payload",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("large slice allocation at line {}", bl.line),
                    "large result sets are usually safer and lighter as streaming RPCs than as one big unary message"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn grpc_no_keepalive_config(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("grpc.NewServer(")
            && !bl.text.contains("KeepaliveParams")
            && !lines
                .iter()
                .any(|line| line.text.contains("KeepaliveParams("))
        {
            findings.push(Finding {
                rule_id: "grpc_no_keepalive_config".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} creates a gRPC server without keepalive settings",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("grpc.NewServer at line {}", bl.line),
                    "keepalive parameters help detect dead clients and load-balancer drops earlier"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn grpc_unary_interceptor_per_rpc(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let sig = &function.signature_text;
    let is_grpc_handler =
        sig.contains("context.Context") && (sig.contains("Request") || sig.contains("Stream"));
    if !is_grpc_handler {
        return findings;
    }
    for bl in lines {
        if bl.text.contains("grpc.UnaryInterceptor(")
            || bl.text.contains("grpc.ChainUnaryInterceptor(")
            || bl.text.contains("grpc.ChainStreamInterceptor(")
        {
            findings.push(Finding {
                rule_id: "grpc_unary_interceptor_per_rpc".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "gRPC handler {} constructs interceptors on the request path",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("interceptor construction at line {}", bl.line),
                    "register interceptors once at server setup rather than rebuilding them per RPC".into(),
                ],
            });
        }
    }
    findings
}

// ── Section C — Logging ──

fn logging_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    // C1 log_level_check_after_format
    for bl in lines {
        if (bl.text.contains(".Debugf(") || bl.text.contains(".Debugw(")) && bl.text.contains("()")
        {
            let has_func_call = bl
                .text
                .split("Debugf(")
                .nth(1)
                .or_else(|| bl.text.split("Debugw(").nth(1))
                .map(|s| s.contains("()"))
                .unwrap_or(false);
            if has_func_call {
                findings.push(Finding {
                    rule_id: "log_level_check_after_format".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} evaluates expensive arguments for debug logging",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("function call in debug log at line {}", bl.line),
                        "arguments evaluated even when debug is disabled".into(),
                    ],
                });
            }
        }
    }

    // C2 logger_created_per_request
    if is_request_path_function(file, function) {
        for bl in lines {
            if bl.text.contains("zap.NewProduction()")
                || bl.text.contains("zap.NewDevelopment()")
                || bl.text.contains("logrus.New()")
                || bl.text.contains("zerolog.New(")
            {
                findings.push(Finding {
                    rule_id: "logger_created_per_request".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} creates logger per request",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("logger construction at line {}", bl.line),
                        "create once at startup; use .With() for request-scoped fields".into(),
                    ],
                });
            }
        }
    }

    // C3 string_format_in_structured_logger
    for bl in lines {
        if (bl.text.contains("logger.Info(")
            || bl.text.contains("logger.Error(")
            || bl.text.contains("logger.Warn(")
            || bl.text.contains("logger.Debug("))
            && bl.text.contains("fmt.Sprintf(")
        {
            findings.push(Finding {
                rule_id: "string_format_in_structured_logger".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} formats strings for structured logger",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("fmt.Sprintf in structured log at line {}", bl.line),
                    "use structured fields instead of formatted strings".into(),
                ],
            });
        }
    }

    // C4 log_printf_for_production
    if is_request_path_function(file, function) {
        for alias in import_aliases_for(file, "log") {
            for bl in lines {
                if bl.text.contains(&format!("{alias}.Printf("))
                    || bl.text.contains(&format!("{alias}.Println("))
                {
                    findings.push(Finding {
                        rule_id: "log_printf_for_production".into(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} uses stdlib log in handler",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("log.Printf in handler at line {}", bl.line),
                            "stdlib log uses global mutex; use slog or zap".into(),
                        ],
                    });
                }
            }
        }
    }

    // C5 error_logged_and_returned
    for (i, bl) in lines.iter().enumerate() {
        if (bl.text.contains("log.Error(")
            || bl.text.contains("logger.Error(")
            || bl.text.contains(".Errorf("))
            && bl.text.contains("err")
        {
            for next in lines.iter().skip(i + 1).take(3) {
                if next.text.starts_with("return") && next.text.contains("err") {
                    findings.push(Finding {
                        rule_id: "error_logged_and_returned".into(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: next.line,
                        message: format!(
                            "function {} logs error then returns it",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("log at line {}, return at line {}", bl.line, next.line),
                            "caller will also log; produces duplicate entries".into(),
                        ],
                    });
                    break;
                }
            }
        }
    }

    findings
}

// ── Section D — Config And CLI ──

fn config_cli_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    // D1 viper_get_in_hot_path
    if is_request_path_function(file, function) {
        for bl in lines {
            if bl.text.contains("viper.Get") {
                findings.push(Finding {
                    rule_id: "viper_get_in_hot_path".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} reads viper config per request",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("viper.Get in handler at line {}", bl.line),
                        "cache config at startup; viper uses global mutex".into(),
                    ],
                });
            }
        }
    }

    // D2 os_getenv_in_hot_path
    if is_request_path_function(file, function) {
        for bl in lines {
            if bl.text.contains("os.Getenv(") {
                findings.push(Finding {
                    rule_id: "os_getenv_in_hot_path".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} reads env var per request",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("os.Getenv in handler at line {}", bl.line),
                        "env doesn't change at runtime; cache at startup".into(),
                    ],
                });
            }
        }
    }

    // D3 config_file_read_per_request
    if is_request_path_function(file, function) {
        for bl in lines {
            if bl.text.contains("os.ReadFile(")
                && (bl.text.contains("config") || bl.text.contains("Config"))
                || bl.text.contains("viper.ReadInConfig()")
            {
                findings.push(Finding {
                    rule_id: "config_file_read_per_request".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} reads config file per request",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("config file I/O at line {}", bl.line),
                        "read config at startup; use fsnotify for reload".into(),
                    ],
                });
            }
        }
    }

    // D4 cobra_flag_lookup
    for bl in lines {
        if bl.text.contains("cmd.Flags().GetString(")
            || bl.text.contains("cmd.Flags().GetInt(")
            || bl.text.contains("cmd.Flags().GetBool(")
        {
            findings.push(Finding {
                rule_id: "cobra_flag_lookup_in_run".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses Flags().Get* instead of StringVar binding",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("flag lookup at line {}", bl.line),
                    "StringVar binds directly to pointer; ~100× faster access".into(),
                ],
            });
        }
    }

    // D5 env_parsing_repeated
    let env_parse_count = lines
        .iter()
        .filter(|bl| {
            bl.text.contains("os.Getenv(")
                && (bl.text.contains("strconv.")
                    || bl.text.contains("Atoi")
                    || bl.text.contains("ParseBool")
                    || bl.text.contains("ParseInt"))
        })
        .count();
    if env_parse_count >= 4
        && let Some(first) = lines.iter().find(|bl| bl.text.contains("os.Getenv("))
    {
        findings.push(Finding {
            rule_id: "env_parsing_repeated_in_init".into(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: first.line,
            end_line: first.line,
            message: format!(
                "function {} has {} manual env parsing chains",
                function.fingerprint.name, env_parse_count
            ),
            evidence: vec![
                format!("{} os.Getenv+strconv chains", env_parse_count),
                "use envconfig.Process for validated config struct".into(),
            ],
        });
    }

    findings
}

// ── Section E — Prometheus And Metrics ──

fn prometheus_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_prom = has_import_path(file, "github.com/prometheus/client_golang/prometheus")
        || has_import_path(
            file,
            "github.com/prometheus/client_golang/prometheus/promauto",
        );
    if !has_prom {
        return findings;
    }

    // E1 prometheus_counter_created_per_request
    if is_request_path_function(file, function) {
        for bl in lines {
            if bl.text.contains("prometheus.NewCounter(")
                || bl.text.contains("prometheus.NewHistogram(")
                || bl.text.contains("prometheus.NewGauge(")
            {
                findings.push(Finding {
                    rule_id: "prometheus_counter_created_per_request".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} creates Prometheus metric per request",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("metric construction at line {}", bl.line),
                        "register metrics once at init; per-request panics on duplicate".into(),
                    ],
                });
            }
        }
    }

    // E2 prometheus_high_cardinality_labels
    for bl in lines {
        if bl.text.contains(".WithLabelValues(") {
            let lower = bl.text.to_lowercase();
            if lower.contains("userid")
                || lower.contains("user_id")
                || lower.contains("requestid")
                || lower.contains("request_id")
                || lower.contains("path")
                || lower.contains("url")
            {
                findings.push(Finding {
                    rule_id: "prometheus_high_cardinality_labels".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses high-cardinality Prometheus labels",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("high-cardinality label at line {}", bl.line),
                        "unbounded labels cause Prometheus OOM".into(),
                    ],
                });
            }
        }
    }

    findings.extend(prometheus_observe_without_timer(file, function, lines));

    // E4 prometheus_unregistered_metric
    for bl in lines {
        if (bl.text.contains("prometheus.NewCounter(")
            || bl.text.contains("prometheus.NewHistogram(")
            || bl.text.contains("prometheus.NewGauge("))
            && !bl.text.contains("promauto.")
        {
            let has_register = lines
                .iter()
                .any(|l| l.text.contains("MustRegister(") || l.text.contains("Register("));
            if !has_register {
                findings.push(Finding {
                    rule_id: "prometheus_unregistered_metric".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} creates metric without registering",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("unregistered metric at line {}", bl.line),
                        "unregistered metrics are never scraped".into(),
                    ],
                });
            }
        }
    }

    findings
}

fn prometheus_observe_without_timer(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if lines
        .iter()
        .any(|line| line.text.contains("prometheus.NewTimer("))
    {
        return findings;
    }
    for bl in lines {
        if bl.text.contains(".Observe(") && bl.text.contains("time.Since(") {
            findings.push(Finding {
                rule_id: "prometheus_observe_without_timer".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} manually records a Prometheus duration instead of using a timer helper",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("manual Observe(time.Since(...)) at line {}", bl.line),
                    "prometheus.NewTimer(...).ObserveDuration() is safer on early returns and panics".into(),
                ],
            });
        }
    }
    findings
}

// ── Section F — AWS SDK ──

fn aws_findings(file: &ParsedFile, function: &ParsedFunction, lines: &[BodyLine]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let aws_paths = [
        "github.com/aws/aws-sdk-go/aws/session",
        "github.com/aws/aws-sdk-go-v2/config",
        "github.com/aws/aws-sdk-go/aws/credentials",
    ];
    let has_aws = aws_paths.iter().any(|p| has_import_path(file, p));
    if !has_aws {
        return findings;
    }

    // F1 aws_session_per_request
    if is_request_path_function(file, function) {
        for bl in lines {
            if bl.text.contains("session.NewSession(")
                || bl.text.contains("config.LoadDefaultConfig(")
            {
                findings.push(Finding {
                    rule_id: "aws_session_per_request".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} creates AWS session per request",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("session creation at line {}", bl.line),
                        "session setup costs ~5-50ms; create once at startup".into(),
                    ],
                });
            }
        }
    }

    // F3 aws_credential_hardcoded
    for bl in lines {
        if bl.text.contains("NewStaticCredentials(") && bl.text.contains("\"") {
            findings.push(Finding {
                rule_id: "aws_credential_hardcoded".into(),
                severity: Severity::Error,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} has hardcoded AWS credentials",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("static credentials at line {}", bl.line),
                    "leaked keys are exploited within minutes".into(),
                ],
            });
        }
    }

    // F5 dynamodb_scan_in_handler
    if is_request_path_function(file, function) {
        for bl in lines {
            if bl.text.contains(".Scan(") && bl.text.contains("dynamodb") {
                findings.push(Finding {
                    rule_id: "dynamodb_scan_in_handler".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses DynamoDB Scan in request handler",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("DynamoDB Scan at line {}", bl.line),
                        "Scan reads entire table; use Query with partition key".into(),
                    ],
                });
            }
        }
    }

    findings.extend(s3_getobject_without_range(file, function, lines));
    findings.extend(s3_listobjects_without_pagination(file, function, lines));

    findings
}

fn s3_getobject_without_range(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_s3 = [
        "github.com/aws/aws-sdk-go/service/s3",
        "github.com/aws/aws-sdk-go-v2/service/s3",
    ]
    .iter()
    .any(|path| has_import_path(file, path));
    if !has_s3 {
        return findings;
    }
    let has_limit_reader = lines.iter().any(|line| line.text.contains("LimitReader("));
    let has_range = lines
        .iter()
        .any(|line| line.text.contains("Range:") || line.text.contains("bytes="));
    if !has_limit_reader || has_range {
        return findings;
    }
    for bl in lines {
        if bl.text.contains("GetObject(") {
            findings.push(Finding {
                rule_id: "s3_getobject_without_range".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} reads only part of an S3 object after downloading the full object",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("S3 GetObject without Range at line {}", bl.line),
                    "partial consumers should request a byte range instead of fetching the entire object"
                        .into(),
                ],
            });
        }
    }
    findings
}

fn s3_listobjects_without_pagination(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_s3 = [
        "github.com/aws/aws-sdk-go/service/s3",
        "github.com/aws/aws-sdk-go-v2/service/s3",
    ]
    .iter()
    .any(|path| has_import_path(file, path));
    if !has_s3 || !is_request_path_function(file, function) {
        return findings;
    }
    let has_pagination = lines.iter().any(|line| {
        line.text.contains("ListObjectsV2Pages")
            || line.text.contains("NewListObjectsV2Paginator(")
            || line.text.contains("ContinuationToken")
            || line.text.contains("MaxKeys:")
    });
    if has_pagination {
        return findings;
    }
    for bl in lines {
        if bl.text.contains("ListObjectsV2(") {
            findings.push(Finding {
                rule_id: "s3_listobjects_without_pagination".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} lists S3 objects without visible pagination",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("ListObjectsV2 at line {}", bl.line),
                    "single-page list calls silently miss objects beyond the first response page"
                        .into(),
                ],
            });
        }
    }
    findings
}
