use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus, bindings};

macro_rules! obs_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "observability",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: bindings::PYTHON_OBSERVABILITY,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    // ── Section 9 · Logging and Observability ──────────────────────────────
    obs_rule!(
        "logging_basic_config_called_from_library_package",
        "Library module calls logging.basicConfig or addHandler at import time, overriding application log config."
    ),
    obs_rule!(
        "logging_set_level_hardcoded_at_module_scope",
        "logger.setLevel(logging.DEBUG) or similar hardcoded at module scope instead of via application config."
    ),
    obs_rule!(
        "f_string_evaluated_eagerly_inside_logging_call",
        "f-string passed directly to logger.debug/info/warning, eagerly evaluating even when the level is disabled."
    ),
    obs_rule!(
        "logger_error_inside_except_without_exc_info",
        "logger.error or logger.critical called inside an except block without exc_info=True, discarding the stack trace."
    ),
    obs_rule!(
        "distributed_trace_span_created_without_parent_context_propagation",
        "OpenTelemetry or custom trace span created without extracting the parent context from the incoming request."
    ),
    obs_rule!(
        "health_check_handler_queries_slow_database_table",
        "Health-check endpoint executes a full ORM query instead of a lightweight SELECT 1 probe."
    ),
    obs_rule!(
        "exception_swallowed_before_sentry_or_error_tracker_capture",
        "Exception re-wrapped before capture_exception, causing Sentry or similar to lose the original traceback."
    ),
    obs_rule!(
        "high_frequency_code_path_logs_without_sampling_or_rate_limit",
        "logger.info or logger.debug called inside a loop without a sampling guard, risking log flood."
    ),
    obs_rule!(
        "opentelemetry_span_attribute_attaches_pii_fields",
        "OpenTelemetry span attribute set to a key that likely carries PII such as email, phone, or IP address."
    ),
    obs_rule!(
        "structured_log_record_missing_trace_or_correlation_id",
        "Structured log record in a request handler lacks trace_id or request_id for distributed tracing."
    ),
    obs_rule!(
        "logging_call_inside_signal_handler_function",
        "logging call inside a signal handler; logging uses locks and is not async-signal-safe."
    ),
    obs_rule!(
        "alert_or_slo_threshold_hardcoded_inside_application_logic",
        "SLO error rate or latency threshold hardcoded in application code instead of external configuration."
    ),
    obs_rule!(
        "prometheus_or_statsd_metric_emitted_inside_db_result_loop",
        "Prometheus counter or statsd metric emitted on every DB row in a result loop instead of once after."
    ),
    obs_rule!(
        "observability_metric_names_use_inconsistent_separators",
        "Metric names in the same codebase mix dot and underscore separators."
    ),
    // ── Section 10 · Module and Package Design ─────────────────────────────
    obs_rule!(
        "star_import_used_in_non_init_production_module",
        "Star import (from x import *) used in a production module other than __init__.py."
    ),
    obs_rule!(
        "importlib_import_module_called_inside_request_handler",
        "importlib.import_module called inside a request handler instead of at application startup."
    ),
    obs_rule!(
        "optional_library_import_checked_on_hot_code_path",
        "Optional-dependency import guarded by try/except ImportError checked on every request."
    ),
    obs_rule!(
        "module_level_side_effect_outside_main_guard",
        "Module-level side effect such as signal.signal or threading.Thread outside an if __name__ guard."
    ),
    obs_rule!(
        "test_support_helpers_located_inside_production_package",
        "Test helper or factory file with _test_helpers.py or _factories.py suffix is inside the production package."
    ),
    obs_rule!(
        "relative_import_crossing_sibling_package_boundary",
        "Relative import uses three or more dots, crossing into a sibling package boundary."
    ),
    obs_rule!(
        "init_file_re_exports_private_module_symbols",
        "__init__.py re-exports a symbol with a leading underscore, leaking private implementation details."
    ),
    obs_rule!(
        "public_package_missing_all_list",
        "Non-trivial package __init__.py defines public symbols without an __all__ list."
    ),
    obs_rule!(
        "dynamic_plugin_loaded_from_config_without_registry_allowlist",
        "Plugin loaded via importlib.import_module from config without validating against an allowlist."
    ),
    obs_rule!(
        "importlib_metadata_version_queried_inside_request_loop",
        "importlib.metadata.version() called inside a request handler; cache the result at startup."
    ),
    obs_rule!(
        "pkg_resources_used_for_runtime_version_lookup",
        "pkg_resources.get_distribution used at runtime; use importlib.metadata.version() instead."
    ),
    // ── Section 11 · Data Structure and Algorithm Choices ──────────────────
    obs_rule!(
        "sorted_full_collection_to_extract_top_n_elements",
        "sorted() called on a full collection just to slice the first N elements; use heapq.nsmallest/nlargest."
    ),
    obs_rule!(
        "linear_membership_test_in_loop_over_large_static_list",
        "Membership test `in list` used inside a loop; convert to a set for O(1) lookup."
    ),
    obs_rule!(
        "manual_dict_increment_instead_of_counter_or_defaultdict",
        "Frequency counter maintained with manual if-key-in-dict increment; use collections.Counter."
    ),
    obs_rule!(
        "list_pop_zero_used_as_queue_operation",
        "list.pop(0) or list.insert(0, ...) used as a FIFO queue; use collections.deque.popleft()."
    ),
    obs_rule!(
        "zip_range_len_used_instead_of_enumerate",
        "zip(range(len(x)), x) pattern used; replace with enumerate(x)."
    ),
    obs_rule!(
        "defaultdict_created_with_lambda_instead_of_builtin_factory",
        "defaultdict(lambda: []) or similar lambda factory; use defaultdict(list) or defaultdict(int)."
    ),
    obs_rule!(
        "filter_and_map_results_materialized_to_list_at_each_step",
        "list(filter(...)) followed by list(map(...)), materializing intermediate collections; use a generator pipeline."
    ),
    obs_rule!(
        "frozenset_not_used_for_constant_membership_set_rebuilt_per_call",
        "Constant set literal rebuilt on every function call; hoist as frozenset at module scope."
    ),
    obs_rule!(
        "sorted_list_maintained_with_insert_instead_of_bisect_insort",
        "Sorted list maintained by appending then calling .sort(); use bisect.insort() instead."
    ),
    obs_rule!(
        "namedtuple_fields_accessed_by_integer_index",
        "namedtuple fields accessed by integer index instead of by named attribute."
    ),
    obs_rule!(
        "counter_most_common_all_items_retrieved_for_top_one",
        "Counter.most_common() called without an argument and subscripted with [0]; pass n=1 or use max()."
    ),
    obs_rule!(
        "repeated_key_hash_via_dict_lookup_in_tight_loop",
        "Same dict key looked up repeatedly inside a loop; cache the value in a local variable."
    ),
    obs_rule!(
        "chain_of_boolean_or_conditions_over_same_value_not_using_in_operator",
        "Three or more `x == 'a' or x == 'b' or ...` conditions chained; use `x in {'a', 'b', ...}`."
    ),
    obs_rule!(
        "ordered_dict_used_in_python_37_plus_where_dict_suffices",
        "OrderedDict used in Python 3.7+ code where a plain dict preserves insertion order."
    ),
    // ── Section 12 · Web API Design Anti-patterns ──────────────────────────
    obs_rule!(
        "api_endpoint_returns_json_without_documented_response_schema",
        "API endpoint returns JSON without a Pydantic response_model or documented response schema."
    ),
    obs_rule!(
        "cursor_based_pagination_missing_stable_sort_tiebreaker",
        "Cursor-based pagination implemented without a stable unique sort key tiebreaker."
    ),
    obs_rule!(
        "bulk_endpoint_partial_failure_contract_ambiguous",
        "Bulk endpoint processes items in a loop without a documented all-or-nothing or partial-failure contract."
    ),
    obs_rule!(
        "rate_limit_429_response_missing_retry_after_header_or_stable_body",
        "HTTP 429 response returned without a Retry-After header or stable body field."
    ),
    obs_rule!(
        "pydantic_validation_error_detail_forwarded_with_internal_field_aliases",
        "Raw Pydantic .errors() detail forwarded to the API client, potentially exposing internal field aliases."
    ),
    obs_rule!(
        "state_changing_endpoint_returns_200_with_empty_body",
        "State-changing POST/PUT endpoint returns HTTP 200 with an empty body; prefer 201/202/204."
    ),
    obs_rule!(
        "binary_or_multipart_response_missing_explicit_content_type",
        "Binary or streaming response returned without an explicit Content-Type header."
    ),
    obs_rule!(
        "large_response_body_fully_buffered_in_memory_before_send",
        "Large response body buffered completely in memory before writing; use StreamingResponse."
    ),
    obs_rule!(
        "api_versioning_in_url_without_matching_router_group",
        "URL path contains a version segment (/v1/) without a corresponding versioned router or blueprint."
    ),
    obs_rule!(
        "response_envelope_shape_inconsistent_across_siblings_in_same_router",
        "Response envelope shape inconsistent across sibling endpoints in the same router file."
    ),
];
