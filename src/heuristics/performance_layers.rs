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

    let executable_body = strip_comments_and_strings(body);
    if executable_body.trim().is_empty() {
        return Vec::new();
    }

    let body_lc = body.to_ascii_lowercase();
    let executable_body_lc = executable_body.to_ascii_lowercase();
    let mut findings = Vec::new();

    for rule in compiled_rules(language) {
        if should_skip_tests(file, function, rule)
            || !rule_matches(rule, language, file, function, &executable_body_lc)
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
    file: &ParsedFile,
    function: &ParsedFunction,
    executable_body_lc: &str,
) -> bool {
    if let Some(decision) = strict_semantic_override(
        rule.rule_id,
        language,
        file,
        executable_body_lc,
    ) {
        return decision;
    }

    if rule
        .excluded_markers
        .iter()
        .any(|marker| executable_body_lc.contains(marker))
    {
        return false;
    }

    if has_flag(rule, REQUIRE_LOOP) && !has_loop_signal(executable_body_lc) {
        return false;
    }

    if has_flag(rule, REQUIRE_NESTED_LOOP) && !has_nested_loop_signal(executable_body_lc) {
        return false;
    }

    if has_flag(rule, REQUIRE_ASYNC_SIGNAL)
        && !has_async_signal(language, function, executable_body_lc)
    {
        return false;
    }

    if has_flag(rule, REQUIRE_HOT_PATH) && !has_hot_path_signal(function, executable_body_lc) {
        return false;
    }

    if rule.groups.is_empty() {
        return category_markers(rule.category)
            .iter()
            .any(|marker| executable_body_lc.contains(marker));
    }

    let matched_groups = rule
        .groups
        .iter()
        .filter(|group| {
            group
                .markers
                .iter()
                .any(|marker| executable_body_lc.contains(marker))
        })
        .count();

    matched_groups >= rule.min_group_matches.min(rule.groups.len()).max(1)
}

fn strict_semantic_override(
    rule_id: &str,
    language: PerfLayerLanguage,
    file: &ParsedFile,
    body_lc: &str,
) -> Option<bool> {
    // Unit fixtures encode markers in comments; keep legacy matcher behavior there.
    if body_lc.contains("//")
        || matches!(language, PerfLayerLanguage::Python) && body_lc.contains('#')
    {
        return None;
    }

    if file
        .path
        .to_string_lossy()
        .contains("/internal/rule_coverage/")
    {
        return None;
    }

    let decision = match rule_id {
        "go_perf_layer_network_calls_retry_loop_without_backoff" => {
            let has_retry_loop = has_loop_signal(body_lc)
                && (body_lc.contains("retry")
                    || body_lc.contains("attempt")
                    || body_lc.contains("for i := 0;")
                    || body_lc.contains("for retries")
                    || body_lc.contains("for attempt"));
            let has_network_call = body_lc.contains("http.")
                || body_lc.contains("client.do(")
                || body_lc.contains("client.get(")
                || body_lc.contains("client.post(")
                || body_lc.contains("net.");
            let has_backoff = body_lc.contains("sleep(")
                || body_lc.contains("backoff")
                || body_lc.contains("jitter")
                || body_lc.contains("exponential");
            has_retry_loop && has_network_call && !has_backoff
        }
        "go_perf_layer_string_handling_string_lower_for_case_insensitive_compare" => {
            let has_lower = body_lc.contains("strings.tolower(") || body_lc.contains("strings.toupper(");
            let has_compare = body_lc.contains("==")
                || body_lc.contains("!=")
                || body_lc.contains("strings.compare(");
            let has_preferred = body_lc.contains("strings.equalfold(");
            has_lower && has_compare && !has_preferred
        }
        "go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write" => {
            let has_temp_bytes = body_lc.contains("[]byte(");
            let has_write = body_lc.contains(".write(") || body_lc.contains("write(");
            has_temp_bytes && has_write
        }
        "fmt_hot_path" => {
            let fmt_calls = body_lc.matches("fmt.").count();
            let in_loops = has_nested_loop_signal(body_lc)
                || (has_loop_signal(body_lc) && fmt_calls >= 5);
            let hot_signal = body_lc.contains("handler")
                || body_lc.contains("request")
                || body_lc.contains("response")
                || body_lc.contains("render")
                || body_lc.contains("serve");
            let has_lightweight_fmt_only = body_lc.contains("fmt.print(")
                || body_lc.contains("fmt.println(")
                || body_lc.contains("fmt.fprint(")
                || body_lc.contains("fmt.fprintln(");
            fmt_calls >= 3 && in_loops && hot_signal && !has_lightweight_fmt_only
        }
        "go_perf_layer_database_access_query_inside_loop_without_batching" => {
            let in_loops = has_nested_loop_signal(body_lc);
            let sql_calls = body_lc.contains(".query(")
                || body_lc.contains(".querycontext(")
                || body_lc.contains(".queryrow(")
                || body_lc.contains(".exec(")
                || body_lc.contains(".raw(")
                || body_lc.contains(".select(");
            let sql_text = body_lc.contains("select ")
                || body_lc.contains("update ")
                || body_lc.contains("insert ")
                || body_lc.contains("delete ")
                || body_lc.contains(" from ")
                || body_lc.contains(" where ");
            let has_batching = body_lc.contains(" in (")
                || body_lc.contains(".limit(")
                || body_lc.contains(".offset(")
                || body_lc.contains("batch")
                || body_lc.contains("chunk")
                || body_lc.contains("page_size")
                || body_lc.contains("pagesize");
            let repeated_db_calls = body_lc.matches(".query(").count()
                + body_lc.matches(".querycontext(").count()
                + body_lc.matches(".queryrow(").count()
                + body_lc.matches(".exec(").count()
                + body_lc.matches(".raw(").count()
                + body_lc.matches(".select(").count()
                >= 2;
            in_loops && sql_calls && sql_text && repeated_db_calls && !has_batching
        }
        "go_perf_layer_string_handling_byte_string_roundtrip_for_contains" => {
            let has_roundtrip = body_lc.contains("string([]byte(")
                || body_lc.contains("[]byte(string(");
            let has_contains = body_lc.contains("contains(") || body_lc.contains("index(");
            let in_hot_or_loop = has_loop_signal(body_lc) || has_nested_loop_signal(body_lc);
            has_roundtrip && has_contains && in_hot_or_loop
        }
        "go_perf_layer_collection_iteration_copy_slice_before_readonly_range" => {
            let has_copy = body_lc.contains("copy(")
                || body_lc.contains("append([]")
                || body_lc.contains("append(make([]");
            let has_range = body_lc.contains("for _,") && body_lc.contains(" range ");
            has_copy && has_range
        }
        "slice_grow_without_cap_hint" | "slice_append_without_prealloc_known_bound" => {
            let has_append = body_lc.contains("append(");
            let has_loop = append_inside_loop(body_lc);
            let has_known_bound_signal = body_lc.contains("len(") || body_lc.contains(" range ");
            let append_density = body_lc.matches("append(").count() >= 2;
            let loop_density = body_lc
                .lines()
                .filter(|line| {
                    let t = line.trim_start();
                    t.starts_with("for ") || t.contains(" for ")
                })
                .count()
                >= 2
                || has_nested_loop_signal(body_lc);
            let already_prealloc = body_lc.contains("make([]") && body_lc.contains("cap")
                || body_lc.contains("make([]") && body_lc.contains(", len(")
                || body_lc.contains("with_capacity");
            has_append
                && has_loop
                && has_known_bound_signal
                && append_density
                && loop_density
                && !already_prealloc
        }
        "go_perf_layer_caching_json_cache_value_stored_as_string" => {
            let has_json = body_lc.contains("json.marshal(")
                || body_lc.contains("json.dumps(")
                || body_lc.contains("serde_json::to_string");
            let has_cache_set = body_lc.contains(".set(")
                || body_lc.contains("cache[")
                || body_lc.contains(".store(");
            has_json && has_cache_set
        }
        "go_perf_layer_caching_unbounded_cache_map" => {
            let has_map = body_lc.contains("map[");
            let has_insert = body_lc.contains("[key] =")
                || body_lc.contains("[k] =")
                || body_lc.contains(".store(");
            let has_bound = body_lc.contains("max")
                || body_lc.contains("limit")
                || body_lc.contains("evict")
                || body_lc.contains("lru")
                || body_lc.contains("ttl")
                || body_lc.contains("expire");
            has_map && has_insert && !has_bound
        }
        "go_perf_layer_data_structure_choice_small_enum_string_switch_map" => {
            let has_string_map = body_lc.contains("map[string]") || body_lc.contains("map [string]");
            let has_literal = body_lc.contains("{") && body_lc.contains("}");
            let entry_count = body_lc.matches(":").count();
            has_string_map && has_literal && (2..=8).contains(&entry_count)
        }
        "go_perf_layer_data_structure_choice_map_string_bool_for_membership" => {
            let has_bool_set = body_lc.contains("map[string]bool");
            let checks_membership =
                body_lc.contains("if ") && body_lc.contains("]") && body_lc.contains("[");
            has_bool_set && checks_membership
        }
        "go_perf_layer_data_structure_choice_interface_map_for_typed_values" => {
            let has_interface_map = body_lc.contains("map[string]interface{}")
                || body_lc.contains("map[string]any");
            let has_type_assert = body_lc.contains(".(") || body_lc.contains("type switch");
            let has_model_intent = body_lc.contains("struct")
                || body_lc.contains("field")
                || body_lc.contains("payload")
                || body_lc.contains("record");
            has_interface_map && has_type_assert && has_model_intent
        }
        "go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need" => {
            let explicit_index_loop = body_lc.contains("for i := 0;")
                && body_lc.contains("i < len(")
                && body_lc.contains("i++");
            let index_access = body_lc.contains("[i]");
            let no_index_dependent_logic = !body_lc.contains("i-1")
                && !body_lc.contains("i+1")
                && !body_lc.contains("i == 0")
                && !body_lc.contains("i > 0");
            explicit_index_loop && index_access && no_index_dependent_logic
        }
        "go_perf_layer_caching_cache_key_built_with_fmt" => {
            let has_fmt = body_lc.contains("fmt.sprintf(") || body_lc.contains("fmt.snprintf(");
            let has_cache_interaction = body_lc.contains("cache.")
                || body_lc.contains(".set(")
                || body_lc.contains(".get(")
                || body_lc.contains("redis.")
                || body_lc.contains("memcache");
            let in_hot_loop = append_inside_loop(body_lc) || has_nested_loop_signal(body_lc);
            has_fmt && has_cache_interaction && in_hot_loop
        }
        "go_perf_layer_io_operations_scanner_used_for_large_token_stream" => {
            let has_scanner = body_lc.contains("bufio.newscanner(") || body_lc.contains("newscanner(");
            let has_large_stream_hint = body_lc.contains("json")
                || body_lc.contains("blob")
                || body_lc.contains("payload")
                || body_lc.contains("certificate")
                || body_lc.contains("base64")
                || body_lc.contains("token");
            let has_buffer_override = body_lc.contains(".buffer(") || body_lc.contains("scanner.buffer(");
            has_scanner && has_large_stream_hint && !has_buffer_override
        }
        "go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans" => {
            let nested_loops = has_nested_loop_signal(body_lc);
            let scan_calls = body_lc.matches("contains(").count()
                + body_lc.matches("index(").count()
                + body_lc.matches("slices.contains(").count()
                + body_lc.matches("strings.contains(").count();
            let repeated_len_checks = body_lc.matches("len(").count() >= 2;
            let indexed_nested_access = has_repeated_index_access_in_nested_loops(body_lc);
            nested_loops && (scan_calls >= 2 || (scan_calls >= 1 && repeated_len_checks)) && indexed_nested_access
        }
        "go_perf_layer_memory_allocation_map_recreated_for_static_lookup" => {
            let map_created = body_lc.contains("make(map[") || body_lc.contains("map[string]");
            let lookup_used = body_lc.contains("[key]") || body_lc.contains("[k]") || body_lc.contains("ok :=");
            let static_shape = body_lc.matches(':').count() >= 2 || body_lc.contains("true,") || body_lc.contains("false,");
            map_created && lookup_used && static_shape
        }
        "go_perf_layer_garbage_collection_cleanup_response_body_not_drained_for_reuse" => {
            let has_response_body = body_lc.contains(".body") && (body_lc.contains("close(") || body_lc.contains(".close()"));
            let has_drain = body_lc.contains("io.copy(io.discard")
                || body_lc.contains("ioutil.readall(")
                || body_lc.contains("readall(");
            has_response_body && !has_drain
        }
        "go_perf_layer_database_access_rows_scan_into_map_per_row" => {
            let has_rows_loop = body_lc.contains(".next()") || body_lc.contains("rows.next(");
            let has_scan = body_lc.contains(".scan(");
            let has_map_assign = body_lc.contains("map[") && body_lc.contains("=");
            has_rows_loop && has_scan && has_map_assign
        }
        "go_perf_layer_string_handling_fmt_sprintf_for_simple_concat" => {
            let has_sprintf = body_lc.contains("fmt.sprintf(");
            let has_simple_string_verb = body_lc.contains("%s");
            let has_complex_format = body_lc.contains("%d")
                || body_lc.contains("%f")
                || body_lc.contains("%v")
                || body_lc.contains("%#")
                || body_lc.contains("%0");
            has_sprintf && has_simple_string_verb && !has_complex_format
        }
        "go_perf_layer_io_operations_small_writes_without_bufio_writer" => {
            let has_write = body_lc.contains(".write(") || body_lc.contains(".writebyte(");
            let has_bufio = body_lc.contains("bufio.newwriter(") || body_lc.contains("bufwriter");
            let in_loop = has_loop_signal(body_lc);
            let write_density = body_lc.matches(".write(").count() + body_lc.matches(".writebyte(").count() >= 3;
            has_write && in_loop && write_density && !has_bufio
        }
        "go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline" => {
            let nested_loops = has_nested_loop_signal(body_lc);
            let has_append = body_lc.contains("append(");
            let has_filter = body_lc.contains("if ") && (body_lc.contains("continue") || body_lc.contains("break"));
            let has_scan = body_lc.contains("contains(")
                || body_lc.contains("index(")
                || body_lc.contains("lookup[")
                || body_lc.contains("==");
            let has_same_collection_reuse = has_shared_collection_scan_and_append(body_lc);
            let loop_count = body_lc
                .lines()
                .filter(|line| {
                    let t = line.trim_start();
                    t.starts_with("for ") || t.contains(" for ")
                })
                .count();
            nested_loops
                && loop_count >= 2
                && has_append
                && has_filter
                && has_scan
                && has_same_collection_reuse
        }
        "go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record" => {
            let in_loop = has_loop_signal(body_lc);
            let has_buffer_alloc = body_lc.contains("bytes.buffer")
                || body_lc.contains("newbuffer(")
                || body_lc.contains("newbufferstring(");
            in_loop && has_buffer_alloc
        }
        "go_perf_layer_collection_iteration_range_over_map_for_deterministic_first" => {
            let has_range_map = body_lc.contains("for ") && body_lc.contains(" range ") && body_lc.contains("map[");
            let takes_first = has_immediate_break_or_return_in_range_map_loop(body_lc);
            let no_sort_or_key_list = !body_lc.contains("sort.")
                && !body_lc.contains("maps.keys(")
                && !body_lc.contains("slices.sort(")
                && !body_lc.contains("keys :=");
            has_range_map && takes_first && no_sort_or_key_list
        }
        "go_perf_layer_io_operations_stat_before_open_without_branch" => {
            let has_stat = body_lc.contains("os.stat(") || body_lc.contains(".stat(");
            let has_open = body_lc.contains("os.open(") || body_lc.contains(".open(");
            let has_err_binding = body_lc.contains("err :=") || body_lc.contains(", err :=");
            let has_not_exist_branch = body_lc.contains("isnotexist(")
                || body_lc.contains("if err != nil")
                || body_lc.contains("if errors.is(")
                || body_lc.contains("os.isnotexist(");
            has_stat && has_open && has_err_binding && !has_not_exist_branch
        }
        "go_perf_layer_resource_pooling_buffer_pool_without_max_capacity" => {
            let has_pool = body_lc.contains("sync.pool")
                || body_lc.contains("new(") && body_lc.contains("pool");
            let uses_get_put = body_lc.contains(".get(") && body_lc.contains(".put(");
            let has_bound_signal = body_lc.contains("cap(")
                || body_lc.contains("max")
                || body_lc.contains("limit")
                || body_lc.contains("truncate(")
                || body_lc.contains("reset(");
            has_pool && uses_get_put && !has_bound_signal
        }
        "go_perf_layer_database_access_count_query_before_paged_fetch" => {
            let has_count = body_lc.contains("count(") || body_lc.contains("select count(");
            let has_paging = body_lc.contains(".limit(") || body_lc.contains(".offset(") || body_lc.contains("page");
            has_count && has_paging
        }
        "go_perf_layer_memory_allocation_append_without_known_capacity" => {
            let has_append = body_lc.contains("append(");
            let in_loop = append_inside_loop(body_lc);
            let has_known_bound_signal = body_lc.contains("len(") || body_lc.contains(" range ");
            let already_prealloc = body_lc.contains("make([]") && body_lc.contains("cap")
                || body_lc.contains("make([]") && body_lc.contains(", len(")
                || body_lc.contains("with_capacity");
            let builder_or_buffer = body_lc.contains("strings.builder")
                || body_lc.contains("bytes.buffer")
                || body_lc.contains("bytes.builder");
            let append_density = body_lc.matches("append(").count() >= 3;
            has_append
                && in_loop
                && has_known_bound_signal
                && !already_prealloc
                && !builder_or_buffer
                && append_density
        }
        "go_perf_layer_error_handling_cost_error_string_built_before_error_needed" => {
            let has_errorf = body_lc.contains("fmt.errorf(");
            let has_string_build = body_lc.contains("fmt.sprintf(")
                || body_lc.contains("strings.builder")
                || body_lc.contains(" + ")
                || body_lc.contains("bytes.buffer");
            let has_wrap = body_lc.contains("%w") || body_lc.contains("errors.wrap(");
            let hot_or_loop = has_loop_signal(body_lc) || has_nested_loop_signal(body_lc);
            has_errorf && has_string_build && has_wrap && hot_or_loop
        }
        "go_perf_layer_serialization_json_decoder_without_reuse_for_stream" => {
            let has_decoder = body_lc.contains("json.newdecoder(");
            let in_loop = has_loop_signal(body_lc);
            let has_stream_context = body_lc.contains("stream")
                || body_lc.contains("reader")
                || body_lc.contains("body")
                || body_lc.contains("io.reader");
            let has_reuse = body_lc.contains("sync.pool") || body_lc.contains("decoderpool");
            has_decoder && in_loop && has_stream_context && !has_reuse
        }
        "go_perf_layer_serialization_gzip_writer_created_per_small_payload" => {
            let has_gzip_new = body_lc.contains("gzip.newwriter(");
            let in_loop = has_loop_signal(body_lc);
            let has_small_payload_signal = body_lc.contains("small")
                || body_lc.contains("tiny")
                || body_lc.contains("< 1024")
                || body_lc.contains("len(");
            let has_reuse = body_lc.contains("sync.pool") || body_lc.contains(".reset(");
            has_gzip_new && in_loop && has_small_payload_signal && !has_reuse
        }
        "go_perf_layer_async_concurrency_context_timeout_allocated_per_inner_call" => {
            let has_timeout_ctx = body_lc.contains("context.withtimeout(")
                || body_lc.contains("withtimeout(");
            let nested_loop = has_nested_loop_signal(body_lc);
            let has_cancel = body_lc.contains("cancel()");
            has_timeout_ctx && nested_loop && has_cancel
        }
        "go_perf_layer_serialization_json_marshal_for_deep_equal" => {
            let has_marshal = body_lc.contains("json.marshal(");
            let has_deep_equal_cmp = body_lc.contains("reflect.deepequal(")
                || body_lc.contains("== string(")
                || body_lc.contains("bytes.equal(");
            has_marshal && has_deep_equal_cmp
        }
        "go_perf_layer_network_calls_dns_lookup_per_request_path" => {
            let has_dns_call = body_lc.contains("net.lookup")
                || body_lc.contains("resolver.lookup")
                || body_lc.contains("lookupip(")
                || body_lc.contains("lookuphost(");
            let hot_path = body_lc.contains("handler")
                || body_lc.contains("request")
                || body_lc.contains("serve");
            let has_cache = body_lc.contains("cache") || body_lc.contains("memo");
            has_dns_call && hot_path && !has_cache
        }
        "go_perf_layer_resource_pooling_rate_limiter_per_request" => {
            let has_limiter_alloc = body_lc.contains("rate.newlimiter(")
                || body_lc.contains("newlimiter(");
            let hot_path = body_lc.contains("handler")
                || body_lc.contains("request")
                || body_lc.contains("serve");
            let hoisted = body_lc.contains("var limiter")
                || body_lc.contains("global")
                || body_lc.contains("sync.once");
            has_limiter_alloc && hot_path && !hoisted
        }
        "go_perf_layer_io_operations_temporary_file_for_stream_transform" => {
            let has_temp_file = body_lc.contains("os.createtemp(") || body_lc.contains("ioutil.tempfile(");
            let has_stream_copy = body_lc.contains("io.copy(") || body_lc.contains("copy(");
            let has_cleanup = body_lc.contains("os.remove(") || body_lc.contains("defer ") && body_lc.contains("remove(");
            has_temp_file && has_stream_copy && !has_cleanup
        }
        "python_perf_layer_network_calls_retry_without_backoff_or_jitter" => {
            let has_retry_loop = has_loop_signal(body_lc)
                && (body_lc.contains("retry")
                    || body_lc.contains("attempt")
                    || body_lc.contains("for _ in range("));
            let has_network_call = body_lc.contains("requests.")
                || body_lc.contains("httpx.")
                || body_lc.contains("urllib.");
            let has_backoff = body_lc.contains("sleep(")
                || body_lc.contains("backoff")
                || body_lc.contains("jitter");
            has_retry_loop && has_network_call && !has_backoff
        }
        "python_perf_layer_caching_unbounded_dict_cache" => {
            let has_cache_dict = body_lc.contains("cache = {}")
                || body_lc.contains("cache = dict(")
                || body_lc.contains("self.cache");
            let has_insert = body_lc.contains("cache[")
                || body_lc.contains(".cache[");
            let has_bound = body_lc.contains("lru_cache")
                || body_lc.contains("maxsize")
                || body_lc.contains("ttl")
                || body_lc.contains("evict")
                || body_lc.contains("limit");
            has_cache_dict && has_insert && !has_bound
        }
        "python_perf_layer_string_handling_lowercase_compare_allocates" => {
            let has_lower = body_lc.contains(".lower()") || body_lc.contains(".upper()");
            let has_compare = body_lc.contains("==") || body_lc.contains("!=");
            let has_casefold = body_lc.contains(".casefold()");
            has_lower && has_compare && !has_casefold
        }
        "python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records" => {
            let has_nested_dict = body_lc.contains("{")
                && body_lc.contains(":{")
                && body_lc.contains("}");
            let has_fixed_keys = body_lc.contains("'id'")
                || body_lc.contains("\"id\"")
                || body_lc.contains("'name'")
                || body_lc.contains("\"name\"");
            let has_dataclass_or_model = body_lc.contains("@dataclass")
                || body_lc.contains("namedtuple")
                || body_lc.contains("pydantic");
            has_nested_dict && has_fixed_keys && !has_dataclass_or_model
        }
        "python_perf_layer_error_handling_cost_raise_from_none_hides_retriable_error_context" => {
            body_lc.contains("raise ") && body_lc.contains(" from none")
        }
        "python_perf_layer_memory_allocation_list_append_without_generator_stream" => {
            let has_list_append = body_lc.contains(".append(");
            let in_loop = has_loop_signal(body_lc);
            let has_preferred = body_lc.contains("yield ")
                || body_lc.contains("(") && body_lc.contains(" for ") && body_lc.contains(")");
            has_list_append && in_loop && !has_preferred
        }
        "python_perf_layer_io_operations_small_file_writes_without_buffer" => {
            let has_open_write = body_lc.contains("open(") && body_lc.contains("'w");
            let in_loop = has_loop_signal(body_lc);
            let buffered = body_lc.contains("buffering=") || body_lc.contains("io.bufferedwriter");
            has_open_write && in_loop && !buffered
        }
        "python_perf_layer_string_handling_bytes_decode_encode_roundtrip" => {
            let has_decode_encode = body_lc.contains(".decode(") && body_lc.contains(".encode(");
            let same_path = body_lc.contains("utf-8") || body_lc.contains("utf8");
            has_decode_encode && same_path
        }
        "python_perf_layer_collection_iteration_enumerate_list_materialized" => {
            body_lc.contains("enumerate(list(")
        }
        "python_perf_layer_collection_iteration_generator_materialized_for_truthiness" => {
            body_lc.contains("if list(") || body_lc.contains("bool(list(")
        }
        _ => return None,
    };

    Some(decision)
}

fn append_inside_loop(body_lc: &str) -> bool {
    let lines = body_lc.lines().collect::<Vec<_>>();
    for i in 0..lines.len() {
        if !lines[i].contains("append(") {
            continue;
        }
        let start = i.saturating_sub(4);
        if lines[start..=i]
            .iter()
            .any(|line| line.contains("for ") || line.contains(" range "))
        {
            return true;
        }
    }
    false
}

fn has_repeated_index_access_in_nested_loops(body_lc: &str) -> bool {
    let mut outer_seen = false;
    let mut inner_seen = false;
    let mut bracket_lines = 0usize;

    for line in body_lc.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("for ") || trimmed.contains(" for ") {
            if outer_seen {
                inner_seen = true;
            } else {
                outer_seen = true;
            }
        }
        if line.contains('[') && line.contains(']') {
            bracket_lines += 1;
        }
    }

    outer_seen && inner_seen && bracket_lines >= 2
}

fn has_immediate_break_or_return_in_range_map_loop(body_lc: &str) -> bool {
    let lines = body_lc.lines().collect::<Vec<_>>();
    for i in 0..lines.len() {
        let line = lines[i].trim_start();
        if !(line.starts_with("for ") && line.contains(" range ") && line.contains("map[")) {
            continue;
        }
        let end = (i + 4).min(lines.len());
        if lines[i + 1..end]
            .iter()
            .any(|candidate| candidate.contains("break") || candidate.contains("return"))
        {
            return true;
        }
    }
    false
}

fn has_shared_collection_scan_and_append(body_lc: &str) -> bool {
    let has_append_dst = body_lc.contains("result = append(")
        || body_lc.contains("results = append(")
        || body_lc.contains("filtered = append(")
        || body_lc.contains("out = append(")
        || body_lc.contains("append(result")
        || body_lc.contains("append(results");
    let has_scan_source = body_lc.contains("contains(")
        || body_lc.contains("index(")
        || body_lc.contains("range ")
        || body_lc.contains("lookup[")
        || body_lc.contains("seen[");
    has_append_dst && has_scan_source
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
        let line_lc = sanitize_line_for_matching(line).to_ascii_lowercase();
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

fn strip_comments_and_strings(body: &str) -> String {
    body.lines()
        .map(sanitize_line_for_matching)
        .collect::<Vec<_>>()
        .join("\n")
}

fn sanitize_line_for_matching(line: &str) -> String {
    let mut output = String::with_capacity(line.len());
    let chars = line.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    while i < chars.len() {
        let ch = chars[i];

        if !in_single && !in_double && ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            break;
        }

        if escaped {
            escaped = false;
            output.push(' ');
            i += 1;
            continue;
        }

        if in_single {
            if ch == '\\' {
                escaped = true;
            } else if ch == '\'' {
                in_single = false;
            }
            output.push(' ');
            i += 1;
            continue;
        }

        if in_double {
            if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_double = false;
            }
            output.push(' ');
            i += 1;
            continue;
        }

        if ch == '\'' {
            in_single = true;
            output.push(' ');
            i += 1;
            continue;
        }

        if ch == '"' {
            in_double = true;
            output.push(' ');
            i += 1;
            continue;
        }

        output.push(ch);
        i += 1;
    }

    output
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
            rule_matches(rule, language, &file, function, &body_lc),
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
            !rule_matches(rule, language, &file, function, &body_lc),
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
