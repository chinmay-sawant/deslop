use std::sync::OnceLock;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};
use crate::rules::{RuleLanguage, rule_registry};

const REQUIRE_LOOP: u32 = 1 << 0;
const REQUIRE_NESTED_LOOP: u32 = 1 << 1;
const REQUIRE_ASYNC_SIGNAL: u32 = 1 << 2;
const REQUIRE_HOT_PATH: u32 = 1 << 3;
const ALLOW_TESTS: u32 = 1 << 4;

#[derive(Debug, Clone, Copy)]
pub(crate) enum PerfLayerLanguage {
    Go,
    Python,
    Rust,
}

#[derive(Debug)]
struct MarkerGroup {
    token: &'static str,
    markers: Vec<&'static str>,
}

#[derive(Debug)]
struct CompiledPerfLayerRule {
    rule_id: &'static str,
    description: &'static str,
    severity: Severity,
    category: &'static str,
    groups: Vec<MarkerGroup>,
    excluded_markers: Vec<&'static str>,
    flags: u32,
    min_group_matches: usize,
}

pub(crate) fn performance_layer_findings(
    language: PerfLayerLanguage,
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let body = function.body_text.as_str();
    if body.trim().is_empty() {
        return Vec::new();
    }

    let body_lc = body.to_ascii_lowercase();
    let mut findings = Vec::new();

    for rule in compiled_rules(language) {
        if should_skip_tests(file, function, rule)
            || !rule_matches(rule, language, function, &body_lc)
        {
            continue;
        }

        let line = first_matching_line(body, function.body_start_line, rule)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(Finding {
            rule_id: rule.rule_id.to_string(),
            severity: rule.severity.clone(),
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "function {} matches performance-layer rule {}",
                function.fingerprint.name, rule.rule_id
            ),
            evidence: finding_evidence(rule, &body_lc),
        });
    }

    findings
}

#[cfg(test)]
pub(crate) fn compiled_rule_count(language: PerfLayerLanguage) -> usize {
    compiled_rules(language).len()
}

fn should_skip_tests(
    file: &ParsedFile,
    function: &ParsedFunction,
    rule: &CompiledPerfLayerRule,
) -> bool {
    (file.is_test_file || function.is_test_function) && !has_flag(rule, ALLOW_TESTS)
}

fn rule_matches(
    rule: &CompiledPerfLayerRule,
    language: PerfLayerLanguage,
    function: &ParsedFunction,
    body_lc: &str,
) -> bool {
    if rule
        .excluded_markers
        .iter()
        .any(|marker| body_lc.contains(marker))
    {
        return false;
    }

    if has_flag(rule, REQUIRE_LOOP) && !has_loop_signal(body_lc) {
        return false;
    }

    if has_flag(rule, REQUIRE_NESTED_LOOP) && !has_nested_loop_signal(body_lc) {
        return false;
    }

    if has_flag(rule, REQUIRE_ASYNC_SIGNAL) && !has_async_signal(language, function, body_lc) {
        return false;
    }

    if has_flag(rule, REQUIRE_HOT_PATH) && !has_hot_path_signal(function, body_lc) {
        return false;
    }

    if rule.groups.is_empty() {
        return category_markers(rule.category)
            .iter()
            .any(|marker| body_lc.contains(marker));
    }

    let matched_groups = rule
        .groups
        .iter()
        .filter(|group| group.markers.iter().any(|marker| body_lc.contains(marker)))
        .count();

    matched_groups >= rule.min_group_matches.min(rule.groups.len()).max(1)
}

fn finding_evidence(rule: &CompiledPerfLayerRule, body_lc: &str) -> Vec<String> {
    let mut evidence = vec![
        format!("category={}", rule.category),
        rule.description.to_string(),
    ];

    let matched_tokens = rule
        .groups
        .iter()
        .filter(|group| group.markers.iter().any(|marker| body_lc.contains(marker)))
        .map(|group| group.token)
        .collect::<Vec<_>>();

    if !matched_tokens.is_empty() {
        evidence.push(format!("matched_tokens={}", matched_tokens.join(",")));
    }

    if !rule.excluded_markers.is_empty() {
        evidence.push(format!(
            "negative_markers_absent={}",
            rule.excluded_markers.join(",")
        ));
    }

    evidence
}

fn first_matching_line(
    body: &str,
    body_start_line: usize,
    rule: &CompiledPerfLayerRule,
) -> Option<usize> {
    body.lines().enumerate().find_map(|(offset, line)| {
        let line_lc = line.to_ascii_lowercase();
        let matches_group = rule
            .groups
            .iter()
            .any(|group| group.markers.iter().any(|marker| line_lc.contains(marker)));
        let matches_category = category_markers(rule.category)
            .iter()
            .any(|marker| line_lc.contains(marker));

        (matches_group || matches_category).then_some(body_start_line + offset)
    })
}

fn compiled_rules(language: PerfLayerLanguage) -> &'static [CompiledPerfLayerRule] {
    static GO_RULES: OnceLock<Vec<CompiledPerfLayerRule>> = OnceLock::new();
    static PYTHON_RULES: OnceLock<Vec<CompiledPerfLayerRule>> = OnceLock::new();
    static RUST_RULES: OnceLock<Vec<CompiledPerfLayerRule>> = OnceLock::new();

    match language {
        PerfLayerLanguage::Go => GO_RULES.get_or_init(|| compile_rules(language)).as_slice(),
        PerfLayerLanguage::Python => PYTHON_RULES
            .get_or_init(|| compile_rules(language))
            .as_slice(),
        PerfLayerLanguage::Rust => RUST_RULES
            .get_or_init(|| compile_rules(language))
            .as_slice(),
    }
}

fn compile_rules(language: PerfLayerLanguage) -> Vec<CompiledPerfLayerRule> {
    let prefix = rule_prefix(language);
    let mut rules = rule_registry()
        .iter()
        .filter(|metadata| metadata.language == rule_language(language))
        .filter(|metadata| metadata.id.starts_with(prefix))
        .map(|metadata| compile_rule(language, metadata.id, metadata.description))
        .collect::<Vec<_>>();
    rules.sort_by(|left, right| left.rule_id.cmp(right.rule_id));
    rules
}

fn compile_rule(
    language: PerfLayerLanguage,
    rule_id: &'static str,
    description: &'static str,
) -> CompiledPerfLayerRule {
    let suffix = rule_id
        .strip_prefix(rule_prefix(language))
        .unwrap_or(rule_id);
    let (category, remainder) = split_category(suffix);
    let mut groups = Vec::new();
    let mut excluded_markers = Vec::new();
    let mut negative_mode = false;
    let mut positive_tokens = Vec::new();

    for token in remainder.split('_') {
        if token.is_empty() || ignored_token(token) {
            continue;
        }
        if negative_token(token) {
            negative_mode = true;
            continue;
        }

        if negative_mode {
            if !positive_tokens.contains(&token) && !excluded_markers.contains(&token) {
                excluded_markers.push(token);
            }
            continue;
        }

        if !positive_tokens.contains(&token) {
            positive_tokens.push(token);
        }

        let markers = token_markers(language, token)
            .map(|markers| markers.to_vec())
            .unwrap_or_else(|| vec![token]);

        if !groups
            .iter()
            .any(|group: &MarkerGroup| group.token == token)
        {
            groups.push(MarkerGroup { token, markers });
        }
    }

    extend_negative_markers(language, rule_id, &mut excluded_markers);

    // A marker cannot be both a required positive signal and an exclusion guard.
    // Remove any extended negative markers that conflict with group markers.
    let group_markers: Vec<&'static str> = groups
        .iter()
        .flat_map(|group| group.markers.iter().copied())
        .collect();
    excluded_markers.retain(|excluded| !group_markers.contains(excluded));

    if groups.is_empty() {
        groups.push(MarkerGroup {
            token: category,
            markers: category_markers(category).to_vec(),
        });
    }

    CompiledPerfLayerRule {
        rule_id,
        description,
        severity: severity_for_rule(rule_id, category),
        category,
        min_group_matches: min_group_matches(&groups, rule_id, category),
        groups,
        excluded_markers,
        flags: flags_for_rule(language, rule_id, category),
    }
}

fn split_category(suffix: &'static str) -> (&'static str, &'static str) {
    for category in CATEGORY_PREFIXES {
        if let Some(remainder) = suffix.strip_prefix(category) {
            return (*category, remainder.trim_start_matches('_'));
        }
    }
    ("performance", suffix)
}

fn min_group_matches(groups: &[MarkerGroup], rule_id: &str, category: &str) -> usize {
    if rule_id.contains("roundtrip")
        || rule_id.contains("count_then")
        || rule_id.contains("read_to_string")
        || rule_id.contains("read_entire")
        || rule_id.contains("created_per")
    {
        return 2.min(groups.len()).max(1);
    }

    if matches!(
        category,
        "error_handling_cost"
            | "logging_overhead"
            | "runtime_configuration"
            | "profiling_benchmarking"
    ) && groups.len() >= 2
    {
        return 2;
    }

    if groups.len() >= 4 { 2 } else { 1 }
}

fn flags_for_rule(language: PerfLayerLanguage, rule_id: &str, category: &str) -> u32 {
    let mut flags = 0;

    if rule_id.contains("nested")
        || rule_id.contains("inside_loop")
        || rule_id.contains("_in_loop")
        || rule_id.contains("_loop")
        || rule_id.contains("loop_")
        || rule_id.contains("per_item")
        || rule_id.contains("per_record")
        || rule_id.contains("per_row")
        || rule_id.contains("row_by_row")
        || rule_id.contains("each_iteration")
        || rule_id.contains("iterrows")
    {
        flags |= REQUIRE_LOOP;
    }

    if rule_id.contains("nested")
        || rule_id.contains("cartesian")
        || rule_id.contains("quadratic")
        || rule_id.contains("n_squared")
    {
        flags |= REQUIRE_NESTED_LOOP;
    }

    if matches!(
        category,
        "hot_path_optimization" | "framework_performance" | "logging_overhead"
    ) || rule_id.contains("hot_path")
        || rule_id.contains("request")
        || rule_id.contains("handler")
        || rule_id.contains("route")
        || rule_id.contains("per_call")
        || rule_id.contains("per_request")
    {
        flags |= REQUIRE_HOT_PATH;
    }

    if matches!(
        language,
        PerfLayerLanguage::Python | PerfLayerLanguage::Rust
    ) && (rule_id.contains("async")
        || rule_id.contains("await")
        || rule_id.contains("tokio")
        || rule_id.contains("future"))
    {
        flags |= REQUIRE_ASYNC_SIGNAL;
    }

    if category == "profiling_benchmarking" || rule_id.contains("benchmark") {
        flags |= ALLOW_TESTS;
    }

    flags
}

fn severity_for_rule(rule_id: &str, category: &str) -> Severity {
    if matches!(
        category,
        "async_concurrency"
            | "database_access"
            | "network_calls"
            | "resource_pooling"
            | "runtime_configuration"
    ) || rule_id.contains("blocking")
        || rule_id.contains("unbounded")
        || rule_id.contains("duplicate_concurrent")
        || rule_id.contains("control_flow")
        || rule_id.contains("not_stopped")
        || rule_id.contains("without_shutdown")
    {
        Severity::Warning
    } else {
        Severity::Info
    }
}

fn extend_negative_markers(
    language: PerfLayerLanguage,
    rule_id: &str,
    excluded_markers: &mut Vec<&'static str>,
) {
    if rule_id.contains("without_backoff") || rule_id.contains("retry_loop_without") {
        extend_unique(
            excluded_markers,
            &["backoff", "jitter", "exponential", "sleep("],
        );
    }
    if rule_id.contains("without_known_capacity")
        || rule_id.contains("without_capacity")
        || rule_id.contains("missing_capacity")
    {
        extend_unique(
            excluded_markers,
            match language {
                PerfLayerLanguage::Go => &["make([]", "make(map", "cap:", "capacity"],
                PerfLayerLanguage::Python => &["capacity", "deque(", "array("],
                PerfLayerLanguage::Rust => &["with_capacity", "reserve(", "capacity"],
            },
        );
    }
    if rule_id.contains("without_buffer") || rule_id.contains("unbuffered") {
        extend_unique(
            excluded_markers,
            match language {
                PerfLayerLanguage::Go => &["bufio.", "buffer", "strings.builder", "bytes.buffer"],
                PerfLayerLanguage::Python => &["buffer", "writelines(", "join("],
                PerfLayerLanguage::Rust => &["bufwriter", "buffer", "write_all_vectored"],
            },
        );
    }
    if rule_id.contains("without_shutdown") {
        extend_unique(
            excluded_markers,
            &["shutdown(", "close(", "stop(", "defer "],
        );
    }
    if rule_id.contains("not_stopped") {
        extend_unique(excluded_markers, &[".stop(", "stop()"]);
    }
    if rule_id.contains("not_flushed") {
        extend_unique(excluded_markers, &["flush("]);
    }
    if rule_id.contains("without_seen") || rule_id.contains("without_iterative_guard") {
        extend_unique(
            excluded_markers,
            &["seen", "visited", "depth", "limit", "guard"],
        );
    }
    if rule_id.contains("without_limit")
        || rule_id.contains("unbounded")
        || rule_id.contains("too_small_for_known_burst")
    {
        extend_unique(
            excluded_markers,
            &[
                "limit",
                "maxsize",
                "max_size",
                "bounded",
                "semaphore",
                "buffered",
                "capacity",
            ],
        );
    }
    if rule_id.contains("without_measurement")
        || rule_id.contains("without_profile")
        || rule_id.contains("without_benchmark")
    {
        extend_unique(
            excluded_markers,
            &["benchmark", "profile", "pprof", "criterion", "trace"],
        );
    }
    if rule_id.contains("missing_warmup") || rule_id.contains("includes_setup") {
        extend_unique(
            excluded_markers,
            &["warmup", "reset_timer", "resettimer", "b.stop_timer"],
        );
    }
    if rule_id.contains("missing_black_box") || rule_id.contains("dead_code_eliminated") {
        extend_unique(
            excluded_markers,
            &["black_box", "keepalive", "runtime.keepalive"],
        );
    }
}

fn extend_unique(target: &mut Vec<&'static str>, markers: &'static [&'static str]) {
    for marker in markers {
        if !target.contains(marker) {
            target.push(marker);
        }
    }
}

fn has_flag(rule: &CompiledPerfLayerRule, flag: u32) -> bool {
    rule.flags & flag != 0
}

fn has_loop_signal(body_lc: &str) -> bool {
    body_lc.contains("for ")
        || body_lc.contains("while ")
        || body_lc.contains("range ")
        || body_lc.contains(".iter()")
        || body_lc.contains(".iter_mut()")
        || body_lc.contains(" loop ")
}

fn has_nested_loop_signal(body_lc: &str) -> bool {
    let loop_lines = body_lc
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("loop ")
                || trimmed.contains(" for ")
        })
        .count();
    loop_lines >= 2
}

fn has_async_signal(language: PerfLayerLanguage, function: &ParsedFunction, body_lc: &str) -> bool {
    match language {
        PerfLayerLanguage::Go => true,
        PerfLayerLanguage::Python => {
            function.python_evidence().is_async
                || function
                    .signature_text
                    .to_ascii_lowercase()
                    .contains("async ")
                || body_lc.contains("await ")
                || body_lc.contains("asyncio.")
        }
        PerfLayerLanguage::Rust => {
            function.rust_evidence().is_async
                || function
                    .signature_text
                    .to_ascii_lowercase()
                    .contains("async ")
                || body_lc.contains(".await")
                || body_lc.contains("tokio::")
        }
    }
}

fn has_hot_path_signal(function: &ParsedFunction, body_lc: &str) -> bool {
    let name = function.fingerprint.name.to_ascii_lowercase();
    [
        "handler",
        "request",
        "route",
        "serve",
        "process",
        "batch",
        "ingest",
        "worker",
        "middleware",
        "interceptor",
        "serialize",
        "render",
        "repository",
        "service",
        "benchmark",
        "transform",
    ]
    .iter()
    .any(|marker| name.contains(marker) || body_lc.contains(marker))
}

fn ignored_token(token: &str) -> bool {
    matches!(
        token,
        "a" | "all"
            | "and"
            | "as"
            | "at"
            | "before"
            | "by"
            | "choice"
            | "cost"
            | "each"
            | "every"
            | "for"
            | "from"
            | "in"
            | "inside"
            | "into"
            | "of"
            | "on"
            | "or"
            | "over"
            | "path"
            | "per"
            | "same"
            | "sensitive"
            | "specific"
            | "the"
            | "to"
            | "too"
            | "use"
            | "used"
            | "uses"
            | "with"
    )
}

fn negative_token(token: &str) -> bool {
    matches!(token, "without" | "missing" | "not" | "no" | "lacks")
}

fn rule_prefix(language: PerfLayerLanguage) -> &'static str {
    match language {
        PerfLayerLanguage::Go => "go_perf_layer_",
        PerfLayerLanguage::Python => "python_perf_layer_",
        PerfLayerLanguage::Rust => "rust_perf_layer_",
    }
}

fn rule_language(language: PerfLayerLanguage) -> RuleLanguage {
    match language {
        PerfLayerLanguage::Go => RuleLanguage::Go,
        PerfLayerLanguage::Python => RuleLanguage::Python,
        PerfLayerLanguage::Rust => RuleLanguage::Rust,
    }
}

fn token_markers(language: PerfLayerLanguage, token: &str) -> Option<&'static [&'static str]> {
    language_token_markers(language, token).or_else(|| generic_token_markers(token))
}

fn language_token_markers(
    language: PerfLayerLanguage,
    token: &str,
) -> Option<&'static [&'static str]> {
    match language {
        PerfLayerLanguage::Go => go_token_markers(token),
        PerfLayerLanguage::Python => python_token_markers(token),
        PerfLayerLanguage::Rust => rust_token_markers(token),
    }
}

fn generic_token_markers(token: &str) -> Option<&'static [&'static str]> {
    match token {
        "alloc" | "allocates" | "allocated" | "allocation" => Some(&[
            "alloc",
            "new(",
            "make(",
            "vec!",
            "string::new",
            "list(",
            "dict(",
        ]),
        "array" | "arrays" => Some(&["array", "ndarray", "[]", "vec<"]),
        "backoff" => Some(&["backoff", "jitter", "sleep("]),
        "base64" => Some(&["base64"]),
        "batch" | "batching" => Some(&["batch", "bulk", "executemany"]),
        "benchmark" | "benchmarks" | "microbenchmark" => {
            Some(&["benchmark", "bench_", "criterion", "timeit", "testing.b"])
        }
        "buffer" | "buffered" | "buffers" => {
            Some(&["buffer", "bufio", "bufwriter", "bytes.buffer", "stringio"])
        }
        "cache" | "cached" | "caching" => {
            Some(&["cache", "lru_cache", "hashmap", "map[", "dict", "once"])
        }
        "client" | "clients" => Some(&["client", "http.client", "reqwest::client", "http.client{"]),
        "clone" | "cloned" | "copy" | "copied" => {
            Some(&["clone(", "copy(", ".copy", "deepcopy", "to_owned"])
        }
        "collection" | "collections" => Some(&["list", "vec", "slice", "map", "dict", "set"]),
        "compare" | "comparison" | "equality" | "equal" => Some(&["==", "equal", "deepequal"]),
        "compress" | "compression" | "gzip" => Some(&["gzip", "compress", "flate2"]),
        "config" | "configuration" | "settings" => {
            Some(&["config", "settings", "env", "getenv", "var("])
        }
        "count" | "counts" => Some(&["count(", ".count", "select count", "count(*)"]),
        "debug" => Some(&["debug", "dbg!", "debug_assert", "debugf"]),
        "decode" | "decoded" => Some(&["decode", "unmarshal", "from_utf8", "serde_json::from"]),
        "dedup" | "deduplication" => Some(&["dedup", "unique", "distinct"]),
        "deserialize" | "deserialized" | "deserialization" => Some(&[
            "deserialize",
            "loads(",
            "unmarshal",
            "from_str",
            "json.loads",
        ]),
        "dns" => Some(&[
            "dns",
            "lookuphost",
            "lookupaddr",
            "to_socket_addrs",
            "urlparse",
        ]),
        "encode" | "encoding" => Some(&["encode", "marshal", "to_vec", "to_string"]),
        "env" | "environment" => Some(&["getenv", "os.environ", "env::var", "lookupenv"]),
        "error" | "errors" => Some(&["error", "exception", "anyhow", "multierror"]),
        "executor" | "pool" | "threadpool" | "processpool" | "threadpools" => Some(&[
            "executor",
            "threadpool",
            "processpool",
            "pool",
            "spawn_blocking",
        ]),
        "fetch" | "download" | "response" => Some(&["fetch", "get(", "response", "resp", "body"]),
        "file" | "filesystem" => Some(&["file", "open(", "readfile", "read_to", "path::"]),
        "filter" | "filters" => Some(&["filter(", ".filter", "where(", "if "]),
        "format" | "formatted" | "formatting" => Some(&["format!", "format(", "sprintf", "f\""]),
        "full" => Some(&["all(", "readall", "read(", "fetch_all", "select *", "sort("]),
        "hash" | "hashing" => Some(&["hash", "hashmap", "hashset", "sha", "md5"]),
        "http" | "https" => Some(&["http", "requests.", "reqwest", "net/http"]),
        "json" => Some(&["json", "serde_json", "encoding/json"]),
        "large" => Some(&["large", "all(", "readall", "fetch_all", "vec!", "list("]),
        "lazy" => Some(&["lazy", "once", "oncelock", "lazylock", "lazy_static"]),
        "limit" | "limits" => Some(&["limit", "take(", "max", "bounded"]),
        "lock" | "locks" | "mutex" => Some(&["lock(", "mutex", "rwlock", ".lock"]),
        "log" | "logger" | "logging" => Some(&["log.", "logger", "tracing", "debug!", "info!"]),
        "map" | "mapping" => Some(&["map[", "hashmap", "btreemap", "dict", ".map("]),
        "materialized" | "materializing" => {
            Some(&["collect", "list(", "tuple(", "readall", "fetch_all"])
        }
        "metadata" => Some(&["metadata", "stat(", "headers", "metadata()"]),
        "network" => Some(&["http", "requests", "reqwest", "net.", "socket"]),
        "payload" | "payloads" => Some(&["payload", "body", "request", "response"]),
        "query" | "queries" | "queryset" => Some(&["query", "select", "where", "filter", "find"]),
        "read" | "readonly" => Some(&["read", "readall", "read_to", "readonly"]),
        "record" | "records" | "row" | "rows" => Some(&["record", "row", "rows", "iterrows"]),
        "retry" | "retries" => Some(&["retry", "for ", "while ", "backoff"]),
        "schema" | "schemas" => Some(&["schema", "compile", "parse"]),
        "serialize" | "serialization" | "serialized" => {
            Some(&["serialize", "dumps", "marshal", "to_string", "to_vec"])
        }
        "sort" | "sorted" => Some(&["sort(", ".sort", "sorted(", "sort_by", "sort_values"]),
        "startup" => Some(&["startup", "init", "main(", "__init__", "lazy_static"]),
        "stream" | "streaming" | "streams" => Some(&["stream", "reader", "iter", "yield"]),
        "string" | "strings" => Some(&["string", "strings.", "sprintf", "format!", "builder"]),
        "temporary" | "tempfile" => {
            Some(&["tempfile", "temporary", "mktemp", "namedtemporaryfile"])
        }
        "time" | "timer" | "timeout" => Some(&["time.", "instant::now", "datetime.now", "timeout"]),
        "tls" => Some(&["tls", "ssl", "rustls", "native_tls"]),
        "typed" => Some(&["struct", "dataclass", "typed", "type "]),
        "url" => Some(&["url", "urlparse", "parse_url"]),
        "value" | "values" => Some(&["value", "values", "serde_json::value", "map[string]any"]),
        "write" | "writes" | "writer" => Some(&["write(", "writeln!", "writer", "write_all"]),
        _ => None,
    }
}

fn python_token_markers(token: &str) -> Option<&'static [&'static str]> {
    match token {
        "apply" => Some(&[".apply("]),
        "asdict" => Some(&["asdict("]),
        "asyncio" | "async" => Some(&["asyncio.", "async def", "await ", "create_task("]),
        "attribute" | "getattr" => Some(&["getattr(", "."]),
        "celery" => Some(&["celery", "@app.task", ".delay("]),
        "context" => Some(&["ssl.create_default_context", "context", "with "]),
        "dataframe" | "pandas" => Some(&["dataframe", "pd.", ".to_sql(", ".iterrows(", ".apply("]),
        "dataclass" | "dataclasses" => Some(&["dataclass", "asdict("]),
        "deepcopy" => Some(&["deepcopy("]),
        "dict" | "dicts" => Some(&["dict(", "{}", "defaultdict", "json.dumps("]),
        "django" => Some(&["django", "queryset", "select_related", "prefetch_related"]),
        "dtype" => Some(&["dtype", "object"]),
        "fastapi" => Some(&["fastapi", "depends(", "apirouter", "@app."]),
        "fifo" | "queue" => Some(&["pop(0)", "insert(0", "queue", "deque"]),
        "generator" => Some(&["generator", "yield", "list(", "tuple("]),
        "heapq" | "heap" => Some(&["heapq", "sorted("]),
        "iterrows" => Some(&[".iterrows("]),
        "lambda" | "closure" => Some(&["lambda ", "def "]),
        "locale" => Some(&["locale.", "zoneinfo", "timezone"]),
        "lowercase" | "lower" | "casefold" => Some(&[".lower(", ".upper(", ".casefold("]),
        "lru" => Some(&["lru_cache", "cache"]),
        "numpy" => Some(&["numpy", "np.", "ndarray"]),
        "object" => Some(&["object", "dtype=object"]),
        "pickle" => Some(&["pickle."]),
        "pydantic" => Some(&["pydantic", "basemodel", "model_validate"]),
        "re" | "regex" => Some(&["re.compile", "re.", "regex"]),
        "requests" | "session" => Some(&["requests.", "session(", "httpx."]),
        "subprocess" => Some(&["subprocess.", "popen(", "run("]),
        "template" | "templates" => Some(&["template", "jinja", "render_template"]),
        "threadpool" => Some(&["threadpoolexecutor", "processpoolexecutor"]),
        "truthiness" => Some(&["if list(", "if tuple(", "len("]),
        _ => None,
    }
}

fn go_token_markers(token: &str) -> Option<&'static [&'static str]> {
    match token {
        "bool" => Some(&["map[string]bool", "bool"]),
        "bufio" => Some(&["bufio.", "newwriter", "newreader"]),
        "burst" | "channel" => Some(&["chan ", "make(chan", "select "]),
        "client" => Some(&["http.client", "&http.client", "http.client{"]),
        "context" | "withtimeout" => Some(&["context.withtimeout", "context.withdeadline"]),
        "defer" => Some(&["defer "]),
        "finalizer" => Some(&["runtime.setfinalizer"]),
        "fmt" | "sprintf" => Some(&["fmt.sprintf", "fmt.format"]),
        "gin" => Some(&["gin.", "*gin.context", ".copy("]),
        "gomaxprocs" => Some(&["gomaxprocs", "runtime.gomaxprocs"]),
        "gogc" => Some(&["gogc", "setgcpercent"]),
        "goroutine" => Some(&["go ", "goroutine"]),
        "gorm" => Some(&["gorm", ".preload(", ".find("]),
        "grpc" => Some(&["grpc", "metadata.fromincomingcontext"]),
        "interface" => Some(&["interface{}", "any", "map[string]any"]),
        "join" => Some(&["strings.join"]),
        "lookup" => Some(&["lookuphost", "lookupaddr", "lookupip", "map["]),
        "map" => Some(&["map[", "make(map"]),
        "multierror" => Some(&["multierror", "errors.join"]),
        "panic" | "recover" => Some(&["panic(", "recover("]),
        "pprof" => Some(&["pprof", "net/http/pprof"]),
        "regexp" => Some(&["regexp.compile", "regexp.mustcompile"]),
        "scan" | "scans" | "scanner" => Some(&["scan(", "scanner", "for "]),
        "select" => Some(&["select {", "select *", ".select("]),
        "slice" | "slices" => Some(&["[]", "slice", "append("]),
        "sqlx" => Some(&["sqlx", ".select("]),
        "sync" => Some(&["sync.", "sync.pool", "sync.once"]),
        "template" => Some(&["template.", "execute", "executetemplate"]),
        "ticker" => Some(&["time.newticker", "ticker"]),
        "tolower" => Some(&["strings.tolower", "strings.toupper"]),
        "transaction" => Some(&["begin(", "commit(", "transaction"]),
        "transport" => Some(&["http.transport", "transport"]),
        _ => None,
    }
}

fn rust_token_markers(token: &str) -> Option<&'static [&'static str]> {
    match token {
        "anyhow" => Some(&["anyhow!", ".context(", "with_context("]),
        "arc" => Some(&["arc<", "arc::new", "weak<"]),
        "askama" => Some(&["askama", "render()"]),
        "async" => Some(&["async fn", ".await", "tokio::", "futures::"]),
        "axum" => Some(&["axum", "extension(", "state("]),
        "binaryheap" => Some(&["binaryheap", "sort_by", "sort_unstable"]),
        "box" | "boxed" => Some(&["box<dyn", "box::new", "vec<box"]),
        "btreemap" => Some(&["btreemap"]),
        "bufwriter" => Some(&["bufwriter", "linewriter"]),
        "bytes" => Some(&["vec<u8>", "bytes", "from_utf8"]),
        "criterion" => Some(&["criterion", "bench_function"]),
        "dispatch" => Some(&["dyn ", "box<dyn", "&dyn"]),
        "enum" => Some(&["enum ", "match "]),
        "fetch" => Some(&["fetch_all", "fetch("]),
        "format" => Some(&["format!", "write!", "writeln!"]),
        "hashmap" => Some(&["hashmap", "hashmap::new", "hashmap::default"]),
        "hashset" => Some(&["hashset", "hashset::new"]),
        "hyper" => Some(&["hyper", "connector"]),
        "instant" => Some(&["instant::now", "systemtime::now"]),
        "iterator" | "iter" => Some(&[".iter()", ".into_iter()", ".filter(", ".map("]),
        "lazy" | "static" => Some(&["lazy_static", "oncelock", "lazylock", "static "]),
        "leaked" | "leak" => Some(&["box::leak", "leak("]),
        "lowercase" => Some(&["to_lowercase", "to_uppercase"]),
        "metadata" => Some(&["fs::metadata", ".metadata("]),
        "oncelock" => Some(&["oncelock", "lazylock"]),
        "rayon" => Some(&["rayon", "threadpoolbuilder"]),
        "regex" => Some(&["regex::new", "regexset::new"]),
        "reqwest" => Some(&["reqwest::client", "client::new"]),
        "rustls" => Some(&["rustls", "native_tls"]),
        "semaphore" => Some(&["semaphore", "rate_limiter"]),
        "serde" => Some(&["serde_json", "serde::"]),
        "sql" | "sqlx" => Some(&["sqlx", "query(", "select *", "fetch_all"]),
        "spawn" | "tokio" => Some(&["tokio::spawn", "joinset", "spawn("]),
        "split" => Some(&[".split(", ".collect::<vec"]),
        "tonic" => Some(&["tonic", "metadata()"]),
        "trait" => Some(&["dyn ", "trait ", "box<dyn"]),
        "vec" => Some(&["vec!", "vec::new", "vec::with_capacity", ".push("]),
        "vecdeque" => Some(&["vecdeque", ".remove(0)"]),
        "weak" => Some(&["weak<", "arc<"]),
        _ => None,
    }
}

fn category_markers(category: &str) -> &'static [&'static str] {
    match category {
        "algorithmic_complexity" => &["for ", "while ", "sort", "filter", "map(", ".iter("],
        "data_structure_choice" => &["map", "dict", "hashmap", "vec", "list", "slice", "set"],
        "memory_allocation" => &["new(", "make(", "vec!", "list(", "append(", "push("],
        "garbage_collection_cleanup" => &["close(", "drop", "clear(", "temp", "pool", "finalizer"],
        "string_handling" => &["string", "str", "format", "sprintf", "regex", "join"],
        "collection_iteration" => &["for ", "while ", ".iter(", "range ", "collect", "list("],
        "async_concurrency" => &["async", "await", "goroutine", "tokio", "mutex", "channel"],
        "io_operations" => &["read", "write", "open(", "file", "scanner", "metadata"],
        "database_access" => &["select", "query", "sql", "rows", "insert", "transaction"],
        "network_calls" => &["http", "request", "response", "tls", "dns", "retry"],
        "caching" => &["cache", "lru", "dict", "map", "hashmap", "once"],
        "serialization" | "serialization_deserialization" => {
            &["json", "pickle", "serde", "marshal", "base64", "gzip"]
        }
        "logging_overhead" => &["log", "logger", "tracing", "debug", "info"],
        "error_handling_cost" => &["error", "err", "exception", "panic", "recover"],
        "runtime_configuration" => &["env", "config", "settings", "profile", "debug"],
        "hot_path_optimization" => &["for ", "while ", "parse", "now(", "reflect", "getattr"],
        "lazy_loading" => &["lazy", "once", "startup", "init", "load", "connect"],
        "resource_pooling" => &["pool", "client", "engine", "transport", "semaphore"],
        "framework_performance" => &["handler", "router", "queryset", "gin", "axum", "sqlx"],
        "profiling_benchmarking" => &["benchmark", "timeit", "criterion", "pprof", "profile"],
        _ => &["for ", "while ", "alloc", "query", "http", "json"],
    }
}

const CATEGORY_PREFIXES: &[&str] = &[
    "algorithmic_complexity",
    "data_structure_choice",
    "memory_allocation",
    "garbage_collection_cleanup",
    "string_handling",
    "collection_iteration",
    "async_concurrency",
    "io_operations",
    "database_access",
    "network_calls",
    "caching",
    "serialization",
    "logging_overhead",
    "error_handling_cost",
    "runtime_configuration",
    "hot_path_optimization",
    "lazy_loading",
    "resource_pooling",
    "framework_performance",
    "profiling_benchmarking",
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use super::{
        CompiledPerfLayerRule, PerfLayerLanguage, REQUIRE_ASYNC_SIGNAL, REQUIRE_LOOP,
        REQUIRE_NESTED_LOOP, compiled_rule_count, compiled_rules, performance_layer_findings,
        rule_language, rule_matches, rule_prefix,
    };
    use crate::analysis::parse_source_file;
    use crate::rules::rule_registry;

    #[test]
    fn compiles_all_performance_layer_catalog_rules() {
        for language in [
            PerfLayerLanguage::Go,
            PerfLayerLanguage::Python,
            PerfLayerLanguage::Rust,
        ] {
            let catalog_ids = rule_registry()
                .iter()
                .filter(|metadata| metadata.language == rule_language(language))
                .filter(|metadata| metadata.id.starts_with(rule_prefix(language)))
                .map(|metadata| metadata.id)
                .collect::<BTreeSet<_>>();
            let compiled_ids = compiled_rules(language)
                .iter()
                .map(|rule| rule.rule_id)
                .collect::<BTreeSet<_>>();

            assert_eq!(compiled_ids, catalog_ids);
            assert_eq!(compiled_rule_count(language), 100);
        }
    }

    #[test]
    fn every_compiled_rule_has_a_matching_signal() {
        for language in [
            PerfLayerLanguage::Go,
            PerfLayerLanguage::Python,
            PerfLayerLanguage::Rust,
        ] {
            for rule in compiled_rules(language) {
                assert!(
                    !rule.groups.is_empty(),
                    "rule {} should have at least one marker group",
                    rule.rule_id
                );
            }
        }
    }

    fn assert_generated_rule_positive(language: PerfLayerLanguage, rule_id: &str) {
        let rule = compiled_rule(language, rule_id);
        let source = generated_source(language, rule, true);
        let body_lc = generated_body(language, rule, true).to_ascii_lowercase();
        let file = parse_source_file(Path::new(generated_path(language)), &source).unwrap_or_else(
            |error| {
                std::panic::panic_any(format!(
                    "generated positive source for {rule_id} should parse: {error}\n{source}"
                ))
            },
        );
        let function = &file.functions[0];

        assert!(
            rule_matches(rule, language, function, &body_lc),
            "expected {rule_id} to match for generated positive {language:?} source.\nsource:\n{source}\nflags={} min_group_matches={} groups={:?} excluded={:?}",
            rule.flags,
            rule.min_group_matches,
            rule.groups,
            rule.excluded_markers
        );
    }

    fn assert_generated_rule_negative(language: PerfLayerLanguage, rule_id: &str) {
        let rule = compiled_rule(language, rule_id);
        let source = generated_source(language, rule, false);
        let body_lc = generated_body(language, rule, false).to_ascii_lowercase();
        let file = parse_source_file(Path::new(generated_path(language)), &source).unwrap_or_else(
            |error| {
                std::panic::panic_any(format!(
                    "generated negative source for {rule_id} should parse: {error}\n{source}"
                ))
            },
        );
        let function = &file.functions[0];

        assert!(
            !rule_matches(rule, language, function, &body_lc),
            "expected {rule_id} to stay silent for generated negative {language:?} source.\nsource:\n{source}"
        );
    }

    #[test]
    fn emits_python_performance_layer_findings() {
        let file = parse_source_file(
            Path::new("service.py"),
            "import requests\n\nasync def handle_route(url):\n    blocking_call = requests.get(url)\n    return blocking_call.json()\n",
        )
        .unwrap_or_else(|error| std::panic::panic_any(format!("python snippet should parse: {error}")));

        let findings =
            performance_layer_findings(PerfLayerLanguage::Python, &file, &file.functions[0]);

        assert!(findings.iter().any(|finding| {
            finding.rule_id
                == "python_perf_layer_async_concurrency_blocking_requests_in_async_route"
        }));
    }

    #[test]
    fn emits_go_performance_layer_findings() {
        let file = parse_source_file(
            Path::new("service.go"),
            "package service\n\nimport \"net/http\"\n\nfunc HandleRequest(url string) error {\n    client := &http.Client{}\n    _, err := client.Get(url)\n    return err\n}\n",
        )
        .unwrap_or_else(|error| std::panic::panic_any(format!("go snippet should parse: {error}")));

        let findings = performance_layer_findings(PerfLayerLanguage::Go, &file, &file.functions[0]);

        assert!(findings.iter().any(|finding| {
            finding.rule_id == "go_perf_layer_network_calls_http_client_created_per_call"
        }));
    }

    #[test]
    fn emits_rust_performance_layer_findings() {
        let file = parse_source_file(
            Path::new("service.rs"),
            "async fn handle_request(url: &str) -> reqwest::Result<()> {\n    let client = reqwest::Client::new();\n    client.get(url).send().await?;\n    Ok(())\n}\n",
        )
        .unwrap_or_else(|error| std::panic::panic_any(format!("rust snippet should parse: {error}")));

        let findings =
            performance_layer_findings(PerfLayerLanguage::Rust, &file, &file.functions[0]);

        assert!(findings.iter().any(|finding| {
            finding.rule_id == "rust_perf_layer_network_calls_reqwest_client_created_per_call"
        }));
    }

    fn compiled_rule(language: PerfLayerLanguage, rule_id: &str) -> &'static CompiledPerfLayerRule {
        compiled_rules(language)
            .iter()
            .find(|rule| rule.rule_id == rule_id)
            .unwrap_or_else(|| std::panic::panic_any(format!("missing compiled rule {rule_id}")))
    }

    fn generated_source(
        language: PerfLayerLanguage,
        rule: &CompiledPerfLayerRule,
        positive: bool,
    ) -> String {
        match language {
            PerfLayerLanguage::Go => generated_go_source(rule, positive),
            PerfLayerLanguage::Python => generated_python_source(rule, positive),
            PerfLayerLanguage::Rust => generated_rust_source(rule, positive),
        }
    }

    fn generated_body(
        language: PerfLayerLanguage,
        rule: &CompiledPerfLayerRule,
        positive: bool,
    ) -> String {
        if positive {
            positive_body_lines(language, rule).join("\n")
        } else {
            base_statement_lines(language).join("\n")
        }
    }

    fn generated_path(language: PerfLayerLanguage) -> &'static str {
        match language {
            PerfLayerLanguage::Go => "generated_rule.go",
            PerfLayerLanguage::Python => "generated_rule.py",
            PerfLayerLanguage::Rust => "generated_rule.rs",
        }
    }

    fn generated_go_source(rule: &CompiledPerfLayerRule, positive: bool) -> String {
        let function_name = if positive { "HandleRequest" } else { "Helper" };
        let body = generated_body(PerfLayerLanguage::Go, rule, positive);

        format!("package sample\n\nfunc {function_name}() {{\n{body}\n}}\n")
    }

    fn generated_python_source(rule: &CompiledPerfLayerRule, positive: bool) -> String {
        let signature = if positive {
            let async_prefix = if rule.flags & REQUIRE_ASYNC_SIGNAL != 0 {
                "async "
            } else {
                ""
            };
            format!("{async_prefix}def handle_request():")
        } else {
            "def helper():".to_string()
        };
        let body = generated_body(PerfLayerLanguage::Python, rule, positive);

        format!("{signature}\n{body}\n")
    }

    fn generated_rust_source(rule: &CompiledPerfLayerRule, positive: bool) -> String {
        let signature = if positive {
            let async_prefix = if rule.flags & REQUIRE_ASYNC_SIGNAL != 0 {
                "async "
            } else {
                ""
            };
            format!("{async_prefix}fn handle_request() {{")
        } else {
            "fn helper() {".to_string()
        };
        let body = generated_body(PerfLayerLanguage::Rust, rule, positive);

        format!("{signature}\n{body}\n}}\n")
    }

    fn positive_body_lines(
        language: PerfLayerLanguage,
        rule: &CompiledPerfLayerRule,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        if rule.flags & REQUIRE_NESTED_LOOP != 0 {
            lines.push(comment_line(language, "for item in items"));
            lines.push(comment_line(language, "for nested in items"));
        } else if rule.flags & REQUIRE_LOOP != 0 {
            lines.push(comment_line(language, "for item in items"));
        }
        for marker in selected_markers(rule) {
            lines.push(comment_line(language, marker));
        }
        lines.extend(base_statement_lines(language));
        lines
    }

    fn base_statement_lines(language: PerfLayerLanguage) -> Vec<String> {
        match language {
            PerfLayerLanguage::Go => vec!["    _ = 1".to_string()],
            PerfLayerLanguage::Python => vec!["    return 1".to_string()],
            PerfLayerLanguage::Rust => vec!["    let _ = 1;".to_string()],
        }
    }

    fn selected_markers(rule: &CompiledPerfLayerRule) -> Vec<&'static str> {
        let mut selected = Vec::new();
        for group in &rule.groups {
            for marker in group.markers.iter().copied() {
                if !marker
                    .chars()
                    .any(|character| character.is_ascii_alphanumeric())
                    || rule
                        .excluded_markers
                        .iter()
                        .any(|excluded| marker.contains(excluded))
                {
                    continue;
                }
                if !selected.contains(&marker) {
                    selected.push(marker);
                }
            }
        }
        selected
    }

    fn comment_line(language: PerfLayerLanguage, marker: &str) -> String {
        let prefix = match language {
            PerfLayerLanguage::Go | PerfLayerLanguage::Rust => "//",
            PerfLayerLanguage::Python => "#",
        };
        format!("    {prefix} {marker}")
    }

    macro_rules! generate_rule_tests {
        ($suite:ident, $language:expr, [$($rule:ident),* $(,)?]) => {
            mod $suite {
                use super::*;

                $(
                    mod $rule {
                        use super::*;

                        #[test]
                        fn positive() {
                            assert_generated_rule_positive($language, stringify!($rule));
                        }

                        #[test]
                        fn negative() {
                            assert_generated_rule_negative($language, stringify!($rule));
                        }
                    }
                )*
            }
        };
    }

    include!("performance_layers/generated_rule_tests.rs");
}
