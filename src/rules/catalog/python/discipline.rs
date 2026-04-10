use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus, bindings};

macro_rules! disc_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "discipline",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: bindings::PYTHON_DISCIPLINE,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    // ── Section 3 · Error Handling Discipline ──────────────────────────────
    disc_rule!(
        "exception_raised_without_chaining_original_cause",
        "New exception raised inside except block without `from e`, discarding the original cause."
    ),
    disc_rule!(
        "exception_handler_branches_on_error_message_string",
        "Exception handler inspects str(e) or e.message in a condition instead of matching on exception type."
    ),
    disc_rule!(
        "bare_except_clause_catches_system_exit",
        "Bare `except:` or `except BaseException` catches SystemExit and KeyboardInterrupt unintentionally."
    ),
    disc_rule!(
        "exception_logged_and_then_re_raised_redundantly",
        "Exception logged at ERROR level and then re-raised, producing duplicate error log entries."
    ),
    disc_rule!(
        "validation_or_parse_error_mapped_to_500_status",
        "ValueError or ValidationError caught and mapped to HTTP 500 instead of a 4xx client-error status."
    ),
    disc_rule!(
        "exception_silenced_in_cleanup_or_finally_block",
        "Exception or return statement inside a finally block silences an in-flight exception from the try body."
    ),
    disc_rule!(
        "project_exception_class_not_inheriting_shared_base",
        "Custom exception class defined without inheriting from the project-wide base exception class."
    ),
    disc_rule!(
        "exception_raised_and_caught_for_control_flow_within_same_function",
        "Exception raised and immediately caught within the same function as an alternative to a conditional return."
    ),
    disc_rule!(
        "error_message_embeds_sensitive_data",
        "Exception message or error response string interpolated with credentials, keys, or PII."
    ),
    disc_rule!(
        "retry_loop_catches_broad_base_exception",
        "Retry loop catches Exception or BaseException instead of the specific transient errors that warrant a retry."
    ),
    disc_rule!(
        "transaction_block_missing_rollback_on_exception",
        "Database transaction started manually without a corresponding rollback in an except or finally clause."
    ),
    disc_rule!(
        "assert_used_for_runtime_input_validation_in_production",
        "assert statement used to validate user-provided input or production logic, which is stripped by -O."
    ),
    disc_rule!(
        "warning_issued_instead_of_exception_for_invalid_state",
        "warnings.warn() issued for an invalid state that should raise an exception instead."
    ),
    disc_rule!(
        "exception_handler_returns_default_without_any_logging",
        "Broad except block returns a default value without logging the caught exception."
    ),
    disc_rule!(
        "deeply_nested_try_except_beyond_two_levels",
        "try/except blocks nested more than two levels deep, obscuring error propagation."
    ),
    disc_rule!(
        "contextlib_suppress_applied_with_exception_base_class",
        "contextlib.suppress used with Exception or BaseException, silently swallowing unexpected errors."
    ),
    disc_rule!(
        "oserror_caught_without_errno_inspection",
        "OSError caught as a broad class without inspecting errno to distinguish ENOENT, EACCES, and others."
    ),
    disc_rule!(
        "custom_exception_encodes_identity_as_string_code_attribute",
        "Custom exception stores its error identity as a plain string code attribute instead of sub-classing."
    ),
    disc_rule!(
        "generator_close_exception_not_handled_when_cleanup_required",
        "Generator or context manager performs resource acquisition in a finally block without a try/finally guard."
    ),
    // ── Section 4 · Type System Discipline ─────────────────────────────────
    disc_rule!(
        "overloaded_dispatch_without_typing_overload_decorator",
        "Function branches on isinstance to dispatch overloads but does not use @typing.overload decorators."
    ),
    disc_rule!(
        "protocol_used_in_isinstance_without_runtime_checkable",
        "typing.Protocol used in isinstance() call without @runtime_checkable decorator."
    ),
    disc_rule!(
        "optional_parameter_used_without_none_guard",
        "Optional[T] parameter accessed via attribute or subscript without a preceding None guard."
    ),
    disc_rule!(
        "callable_annotation_without_parameter_types",
        "Callable annotation used without explicit parameter types, losing type information downstream."
    ),
    disc_rule!(
        "cast_applied_without_preceding_type_narrowing_guard",
        "typing.cast applied to a value without a preceding isinstance or type narrowing check."
    ),
    disc_rule!(
        "type_alias_shadows_builtin_name",
        "Type alias variable assigned a name that shadows a Python builtin such as list, dict, or type."
    ),
    disc_rule!(
        "namedtuple_used_where_dataclass_better_fits",
        "collections.namedtuple used for a record that needs default values, mutation, or methods."
    ),
    disc_rule!(
        "public_function_return_type_annotated_as_union_of_many_unrelated_types",
        "Public function return annotation is Union of four or more unrelated types, indicating mixed concerns."
    ),
    disc_rule!(
        "typevar_defined_without_bound_or_constraints_for_narrow_use",
        "TypeVar defined without a bound or constraints for a generic that is only used with one or two concrete types."
    ),
    disc_rule!(
        "generic_class_used_without_type_parameter_application",
        "Generic class subclassed or instantiated without supplying the required type parameters."
    ),
    disc_rule!(
        "protocol_method_lacks_type_annotations",
        "Method in a Protocol class body lacks parameter or return type annotations."
    ),
    disc_rule!(
        "typed_dict_key_access_without_get_or_guard",
        "TypedDict with total=False has a key accessed via direct subscript without .get() or a guard."
    ),
    disc_rule!(
        "typed_dict_total_false_without_docstring_noting_optional_keys",
        "TypedDict(total=False) class has no docstring indicating which keys are optional."
    ),
    disc_rule!(
        "string_forward_reference_in_annotation_not_under_type_checking_guard",
        "String forward reference annotation not placed under `if TYPE_CHECKING:`, causing import at runtime."
    ),
    // ── Section 5 · Testing Anti-patterns ──────────────────────────────────
    disc_rule!(
        "test_function_stacks_too_many_mock_patch_decorators",
        "Test function has five or more @mock.patch decorators, indicating over-specified mocking."
    ),
    disc_rule!(
        "test_calls_time_sleep_for_coordination",
        "Test calls time.sleep to coordinate with async or threaded code instead of events or monkeypatching."
    ),
    disc_rule!(
        "test_mutates_module_global_without_restore",
        "Test mutates a module-level global variable without restoring it in teardown, leaking state."
    ),
    disc_rule!(
        "test_asserts_private_attribute_value_instead_of_behavior",
        "Test asserts the value of a private attribute (_name) instead of observable public behavior."
    ),
    disc_rule!(
        "test_fixture_calls_datetime_now_without_freezing",
        "Test fixture or test body calls datetime.now() or datetime.today() without freezegun or similar."
    ),
    disc_rule!(
        "test_wraps_sut_in_try_except_hiding_exception_detail",
        "Test wraps the system-under-test call in try/except, hiding the raised exception from pytest output."
    ),
    disc_rule!(
        "pytest_parametrize_with_single_test_case",
        "@pytest.mark.parametrize applied with only one parameter set; use a plain test function instead."
    ),
    disc_rule!(
        "test_skipped_with_no_reason_string",
        "@pytest.mark.skip used without a reason= string explaining why the test is disabled."
    ),
    disc_rule!(
        "test_compares_float_with_equality_operator",
        "Test uses == to compare floating-point values instead of pytest.approx or math.isclose."
    ),
    disc_rule!(
        "test_loads_real_application_config_or_secrets",
        "Test reads real application config files or environment secrets rather than injecting test doubles."
    ),
    disc_rule!(
        "test_makes_real_outbound_http_call_without_mock_or_vcr",
        "Test makes a real outbound HTTP request without mocking or a VCR cassette."
    ),
    disc_rule!(
        "test_function_covers_multiple_unrelated_scenarios",
        "Integration or unit test body covers multiple unrelated scenarios without parameterization."
    ),
    disc_rule!(
        "integration_test_writes_state_without_cleanup",
        "Integration test creates database rows or files without a teardown or transactional rollback."
    ),
    disc_rule!(
        "pytest_raises_without_match_parameter_on_broad_exception",
        "pytest.raises used with a broad exception type without a match= pattern to verify the message."
    ),
    disc_rule!(
        "test_re_implements_production_validation_logic",
        "Test contains a hard-coded reimplementation of production validation logic instead of invoking it."
    ),
    disc_rule!(
        "test_imports_private_production_module",
        "Test file imports a private production module (._module) instead of going through the public API."
    ),
    disc_rule!(
        "mock_return_value_is_incompatible_type_with_real_signature",
        "Mock return value set to a type that is incompatible with the real object's declared return type."
    ),
    disc_rule!(
        "unittest_test_class_duplicates_setup_without_base_class",
        "Multiple unittest.TestCase subclasses in the same file duplicate setUp logic without extracting a base class."
    ),
    disc_rule!(
        "test_depends_on_sibling_test_side_effects",
        "Test function depends on state left behind by a sibling test in the same module."
    ),
];
