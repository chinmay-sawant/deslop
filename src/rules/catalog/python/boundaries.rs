use super::{
    RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus, bindings,
};

macro_rules! bound_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "boundaries",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: bindings::PYTHON_BOUNDARIES,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    // ── Section 6 · Security Boundaries ────────────────────────────────────
    bound_rule!(
        "sql_query_built_with_string_formatting_instead_of_parameters",
        "SQL query string built via f-string or % formatting instead of parameterised query binding."
    ),
    bound_rule!(
        "file_path_from_user_input_without_normalization_or_anchor_check",
        "File path derived from user input without Path.resolve() or anchor containment check."
    ),
    bound_rule!(
        "xml_parsing_with_external_dtd_or_entity_processing_enabled",
        "XML parsed with a parser that processes external DTDs or entities, enabling XXE attacks."
    ),
    bound_rule!(
        "http_client_url_built_from_user_input_without_allowlist",
        "HTTP client URL constructed from user-controlled input without an allowlist or URL validation."
    ),
    bound_rule!(
        "subprocess_invoked_with_shell_true_and_user_derived_input",
        "subprocess called with shell=True and a command string that includes user-controlled data."
    ),
    bound_rule!(
        "jinja2_environment_created_with_autoescape_disabled",
        "Jinja2 Environment created without autoescape=True or with autoescape=False, enabling XSS."
    ),
    bound_rule!(
        "jwt_decode_allows_none_algorithm_or_no_algorithm_restriction",
        "JWT decoded without restricting the allowed algorithm list, permitting the none algorithm bypass."
    ),
    bound_rule!(
        "insecure_hash_algorithm_used_for_security_sensitive_purpose",
        "MD5 or SHA-1 used for password hashing, token generation, or a security-sensitive digest."
    ),
    bound_rule!(
        "deserialization_from_external_or_user_controlled_source_with_pickle",
        "pickle.loads or pickle.load called on data from an external or user-controlled source."
    ),
    bound_rule!(
        "debug_or_admin_endpoint_registered_without_environment_guard",
        "Debug, admin, or diagnostics route registered without an environment variable guard."
    ),
    bound_rule!(
        "weak_random_function_used_for_security_token_generation",
        "random.random or random.choice used to generate a security token, CSRF value, or password reset token."
    ),
    bound_rule!(
        "open_redirect_via_user_supplied_url_without_allowlist",
        "HTTP redirect target constructed from user-supplied URL without allowlist validation."
    ),
    bound_rule!(
        "arbitrary_file_write_via_user_controlled_path",
        "File write operation uses a path derived from user input without containment check."
    ),
    bound_rule!(
        "cors_allow_all_origins_set_without_production_environment_check",
        "CORS configuration allows all origins (*) without a conditional check for the production environment."
    ),
    bound_rule!(
        "server_side_template_injection_via_user_input_in_template_source",
        "User-supplied data used as the template source string for Jinja2 or Mako, enabling SSTI."
    ),
    bound_rule!(
        "regex_pattern_with_catastrophic_backtracking_applied_to_unbounded_input",
        "Regular expression containing nested quantifiers applied to user-supplied unbounded input."
    ),
    bound_rule!(
        "ldap_search_filter_built_from_user_input_without_escaping",
        "LDAP search filter string built from user-controlled input without ldap3 or python-ldap escaping."
    ),
    bound_rule!(
        "state_changing_endpoint_missing_csrf_protection",
        "State-changing POST/PUT/DELETE endpoint lacks CSRF token validation or SameSite enforcement."
    ),
    bound_rule!(
        "cryptographic_secret_hardcoded_in_test_fixture_or_seed",
        "Real-looking cryptographic secret, token, or private key hardcoded in a test fixture or seed file."
    ),
    // ── Section 7 · Memory and Resource Boundaries ─────────────────────────
    bound_rule!(
        "unbounded_list_accumulation_inside_long_running_function",
        "List grows unboundedly inside a long-running function or loop without a cap or periodic flush."
    ),
    bound_rule!(
        "generator_consumed_twice_without_recreation",
        "Same generator object iterated a second time after it has already been exhausted."
    ),
    bound_rule!(
        "file_object_returned_or_stored_without_clear_close_path",
        "File object returned from a function or stored in an attribute without a guaranteed close path."
    ),
    bound_rule!(
        "weakref_dereferenced_without_live_check",
        "weakref.ref() called and result used without checking for None, potential AttributeError."
    ),
    bound_rule!(
        "functools_lru_cache_applied_to_instance_method",
        "@functools.lru_cache applied to an instance method, holding a reference to self and preventing GC."
    ),
    bound_rule!(
        "subprocess_pipe_without_communicate_for_large_output",
        "subprocess.Popen with PIPE but .communicate() not called, risking deadlock on large output."
    ),
    bound_rule!(
        "socket_opened_without_context_manager_or_guaranteed_close",
        "socket.socket opened without a context manager or explicit .close() in a finally block."
    ),
    bound_rule!(
        "repeated_deepcopy_in_loop_on_same_source_object",
        "copy.deepcopy called on the same large object on every iteration of a loop."
    ),
    bound_rule!(
        "redis_commands_issued_individually_in_loop_without_pipeline",
        "Redis SETEX/HSET/GET commands issued one per loop iteration without using a pipeline."
    ),
    bound_rule!(
        "unclosed_tempfile_or_tmp_directory_from_tempfile_module",
        "tempfile.NamedTemporaryFile or TemporaryDirectory created without delete=False or a context manager."
    ),
    bound_rule!(
        "db_connection_pool_size_exceeds_server_max_connections",
        "Database connection pool max_overflow or pool_size configured above the database server's max_connections."
    ),
    bound_rule!(
        "closure_captures_large_object_after_producing_function_returns",
        "Inner function or lambda captures a large local variable after the producer returns, preventing GC."
    ),
    bound_rule!(
        "object_allocated_in_tight_loop_expected_to_be_pooled",
        "Heavyweight object such as HTTPSession or DB connection allocated fresh on every loop iteration."
    ),
    // ── Section 8 · Configuration Hygiene ──────────────────────────────────
    bound_rule!(
        "dotenv_load_dotenv_called_from_multiple_modules",
        "dotenv.load_dotenv() called from more than one module, risking silent config override."
    ),
    bound_rule!(
        "pydantic_settings_model_allows_post_init_mutation",
        "Pydantic Settings model does not set frozen=True, allowing accidental mutation of config values."
    ),
    bound_rule!(
        "feature_flag_checked_via_inline_env_lookup_across_handlers",
        "Feature flag checked via repeated inline os.getenv in multiple handlers instead of a single flag service."
    ),
    bound_rule!(
        "secrets_manager_client_created_per_function_call",
        "AWS Secrets Manager or Vault client instantiated on every function call instead of being cached."
    ),
    bound_rule!(
        "toml_or_ini_config_file_parsed_on_request_path",
        "TOML, INI, or YAML config file parsed inside a request handler on every request."
    ),
    bound_rule!(
        "startup_log_statement_includes_raw_secret_value",
        "Startup or health-check log statement interpolates a raw secret, key, or password value."
    ),
    bound_rule!(
        "pydantic_settings_model_does_not_forbid_extra_fields",
        "Pydantic Settings model does not set model_config = ConfigDict(extra='forbid'), allowing silent typos."
    ),
    bound_rule!(
        "yaml_config_loaded_without_safe_loader",
        "YAML configuration file loaded with yaml.load instead of yaml.safe_load or yaml.load + Loader=SafeLoader."
    ),
    bound_rule!(
        "application_config_values_validated_lazily_on_first_use",
        "Config values validated inside request handlers rather than at startup, deferring misconfiguration errors."
    ),
    bound_rule!(
        "sensitive_config_key_included_in_debug_level_log_dict_dump",
        "Debug log statement dumps the full config dict including keys that may hold secrets."
    ),
    bound_rule!(
        "multiple_config_sources_merged_without_documented_precedence_order",
        "Application merges environment variables, config files, and defaults without explicit precedence documentation."
    ),
    bound_rule!(
        "pydantic_settings_model_missing_env_prefix_isolation",
        "Pydantic BaseSettings model does not define env_prefix, mixing its env vars with system environment."
    ),
    bound_rule!(
        "mutable_default_argument_leaks_state_across_calls",
        "Flag function signatures that use mutable defaults such as [], {}, or set() and allow state sharing between calls."
    ),
    bound_rule!(
        "helper_returns_live_internal_collection_reference",
        "Flag helpers that hand out direct references to mutable internal lists, dicts, or sets instead of copies or read-only views."
    ),
    bound_rule!(
        "public_api_mutates_argument_in_place_without_signal",
        "Flag public APIs that rewrite caller-owned containers or objects without naming or documentation that makes the mutation obvious."
    ),
    bound_rule!(
        "dataclass_mutable_default_without_default_factory",
        "Flag dataclass fields that use mutable defaults without default_factory."
    ),
    bound_rule!(
        "context_manager_yields_global_mutable_resource",
        "Flag context managers that expose a shared global mutable resource while implying call-local ownership."
    ),
    bound_rule!(
        "module_cache_exposed_without_invalidation_boundary",
        "Flag modules that expose cache internals directly with no clear invalidation or ownership boundary."
    ),
    bound_rule!(
        "closure_captures_loop_variable_without_binding",
        "Flag closures created in loops that capture the loop variable without binding the current value."
    ),
    bound_rule!(
        "function_accepts_mapping_protocol_but_mutates_input",
        "Flag functions that advertise broad mapping or sequence inputs and then mutate the received object in place."
    ),
    bound_rule!(
        "iterator_argument_consumed_then_reused_later",
        "Flag functions that exhaust an iterator and later treat it as if it were reusable data."
    ),
    bound_rule!(
        "public_api_forwards_library_specific_exception_shape",
        "Flag public interfaces that require callers to understand raw exceptions from underlying libraries."
    ),
    bound_rule!(
        "datetime_boundary_mixes_naive_and_aware_values",
        "Flag APIs that accept or combine timezone-aware and timezone-naive datetimes without normalization."
    ),
    bound_rule!(
        "text_bytes_boundary_relies_on_implicit_default_encoding",
        "Flag code that crosses text and bytes boundaries without an explicit encoding contract."
    ),
    bound_rule!(
        "path_boundary_accepts_unexpanded_or_relative_input_without_normalization",
        "Flag filesystem-facing APIs that accept relative paths or ~ forms without normalizing them before downstream use."
    ),
    bound_rule!(
        "sentinel_default_value_overlaps_valid_business_value",
        "Flag APIs whose sentinel or fallback value can also be a valid domain value."
    ),
    bound_rule!(
        "sync_api_accepts_coroutine_object_as_regular_value",
        "Flag synchronous interfaces that can accidentally receive coroutine objects and treat them as plain values."
    ),
    bound_rule!(
        "async_api_returns_plain_iterator_with_blocking_iteration",
        "Flag async-facing APIs that return plain iterators whose consumption can still block."
    ),
    bound_rule!(
        "property_returns_live_internal_cache_object",
        "Flag properties that expose live mutable cache objects directly to callers."
    ),
    bound_rule!(
        "lock_acquire_and_release_owned_by_different_callers",
        "Flag APIs that require one caller to acquire a lock and another caller to release it."
    ),
    bound_rule!(
        "module_constant_rebound_after_public_import",
        "Flag modules that expose apparent constants and later rebind them during normal runtime flow."
    ),
    bound_rule!(
        "helper_requires_caller_to_know_hidden_ordering_constraints",
        "Flag helpers whose correctness depends on the caller invoking them in a specific undocumented order."
    ),
];
