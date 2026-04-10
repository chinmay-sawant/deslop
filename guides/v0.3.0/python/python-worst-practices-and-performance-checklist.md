# Python Worst Practices and Performance Bottleneck Detection Checklist

Date: 2026-04-10

## Status

- [x] All 200 rule candidates below are net-new relative to the current `rules/registry.json` Python inventory.
- [x] None of the items below duplicate the existing `ai_smells`, `duplication`, `framework`, `hot_path`, `hot_path_ext`, `maintainability`, `mlops`, `packaging`, `performance`, `quality`, or `structure` Python rules already shipped.
- [x] This document is now fully implemented; all rule candidates have been shipped in `src/heuristics/python/` and registered in `src/rules/catalog/python/`.

## Objective

- [x] Extend Python detection coverage with 200 new rule candidates across architecture and layer boundaries, async and concurrency correctness, error handling discipline, type system contracts, testing anti-patterns, security boundaries, memory and resource management, configuration and secrets hygiene, logging and observability, module and package design, data structure and algorithm choices, and web API design anti-patterns.
- [x] Keep every candidate orthogonal to existing shipped rules so implementation does not create duplicate signals.
- [x] Prefer signals detectable from local AST traversal, import shapes, call patterns, and decorator usage before requiring cross-file or repo-wide correlation.

## Hard Exclusions

- [x] Do not restate current performance, hot-path, mlops, framework, quality, or structure rules that already exist in `rules/registry.json`.
- [x] Do not restate generic comment, naming-convention, or import-grouping rules from `ai_smells` or `maintainability`.
- [x] Do not force mandatory architectural styles on small scripts, CLIs, or single-file utilities.
- [x] Only fire layer-ownership rules when the repository already shows clear role-package naming such as `service`, `repository`, `model`, `schema`, `handler`, `router`, `middleware`, `domain`, or `api`.

## Detection Principles

- [x] Prefer call-site evidence: library imports, decorator shapes, method call chains, and argument patterns visible in the local AST.
- [x] Keep most rules function-local or class-local first; escalate to repo-wide correlation only when it materially reduces false positives.
- [x] Treat `service`, `repository`, `repo`, `model`, `models`, `schema`, `schemas`, `handler`, `handlers`, `domain`, `api`, `router`, `middleware`, `view`, `views`, `serializer`, and `dto` as role hints, not mandatory patterns in every repo.
- [x] Allow legitimate exceptions for migrations, generated code, test support, adapters, and framework glue.

## Section Summary

| Section | Theme | Rules |
| --- | --- | ---: |
| 1 | Architecture and layer boundaries | 20 |
| 2 | Async and concurrency correctness | 20 |
| 3 | Error handling discipline | 20 |
| 4 | Type system and API contracts | 15 |
| 5 | Testing anti-patterns | 20 |
| 6 | Security boundaries | 20 |
| 7 | Memory and resource management | 15 |
| 8 | Configuration and secrets hygiene | 15 |
| 9 | Logging and observability | 15 |
| 10 | Module and package design | 15 |
| 11 | Data structure and algorithm choices | 15 |
| 12 | Web API design anti-patterns | 10 |
| **Total** | | **200** |

## Implementation Fit With The Existing Repository

- [x] Add new Python heuristic modules under `src/heuristics/python` instead of extending one oversized file.
- [x] Mirror new families under `src/rules/catalog/python` so each section maps cleanly to a rule registration module.
- [x] Keep fixture coverage in `tests/fixtures/python` using the current positive and clean text-fixture style.
- [x] Keep integration coverage in `tests/integration_scan/python` so families can be exercised independently.
- [x] Reuse existing parser summaries and import-aware evidence before introducing heavier secondary analysis.

---

## 1. Architecture and Layer Boundaries (20 rules)

Use these rules only when the repository already contains more than one clear role layer such as `service`, `repository`, `model`, `view`, `handler`, or `domain`.

- [x] `service_method_accepts_http_request_object`: flag service methods that accept `flask.Request`, `django.HttpRequest`, or `fastapi.Request` objects directly instead of extracted, transport-neutral domain values.
- [x] `repository_returns_unexecuted_orm_query`: flag repository methods that return ORM query objects such as `QuerySet`, `Query`, or `Select` to callers instead of fully evaluated results or domain values.
- [x] `view_or_handler_constructs_orm_query_directly`: flag view or handler functions that build ORM filter chains, annotations, aggregates, or raw query expressions directly instead of delegating to a repository or query object.
- [x] `domain_model_class_imports_http_library`: flag domain entity or value-object classes that import `requests`, `httpx`, `http.client`, `flask`, `django.http`, or `fastapi` modules at the class definition site.
- [x] `service_raises_or_catches_http_exception_type`: flag service packages that catch or raise `werkzeug.exceptions.HTTPException`, `fastapi.HTTPException`, or framework-specific HTTP error types instead of domain-neutral error types.
- [x] `handler_or_view_builds_raw_sql`: flag handler or view functions that construct raw SQL strings with formatting or concatenation instead of delegating to a repository or query helper.
- [x] `domain_entity_carries_http_status_field`: flag domain entity classes that embed HTTP status codes, response envelope fields, or transport headers as model fields.
- [x] `business_logic_inside_middleware`: flag middleware that implements feature-flag branching, pricing rules, access-control decisions, or other domain workflows instead of pure cross-cutting concerns such as auth enrichment, tracing, or request normalization.
- [x] `service_method_returns_http_response_object`: flag service methods that return `Response`, `JsonResponse`, `HTMLResponse`, or framework `abort()` results instead of domain results or typed errors.
- [x] `repository_method_accepts_pydantic_request_schema`: flag repository constructors or methods that accept Pydantic request schemas or FastAPI dependency-injected inputs instead of domain-level inputs.
- [x] `handler_or_view_owns_transaction_lifecycle`: flag handler or view code that calls `session.begin()`, `transaction.atomic()`, `db.begin()`, or similar transaction-start primitives directly instead of delegating transactional work to a service or unit-of-work helper.
- [x] `celery_or_rq_task_imports_web_framework_app`: flag Celery, arq, or rq task functions that import the Flask `app` factory, FastAPI `app` instance, or Django `wsgi.application` directly.
- [x] `validation_rules_duplicated_at_dto_and_domain_layer`: flag cases where the same field constraint or business rule is validated both at the request schema layer and again inside the domain model or service, without a clear delegation policy.
- [x] `persistent_model_field_encodes_transport_concern`: flag ORM model fields whose names or values are clearly transport-specific such as `status_code`, `http_method`, `response_body`, or `request_id` that belong in transport-layer types.
- [x] `auth_extraction_duplicated_across_views`: flag repeated calls to `request.user`, `get_current_user()`, `request.state.user`, or `g.user` scattered directly across three or more view functions instead of centralized middleware or a shared dependency.
- [x] `service_reads_settings_inline_instead_of_injected`: flag service methods that call `settings.SOME_VALUE`, `config.get(...)`, or `os.getenv(...)` inline during method execution instead of receiving normalized configuration at construction time.
- [x] `background_job_depends_on_request_context_object`: flag background worker functions that import or reference Flask `g`, FastAPI `request`, Django `request`, or thread-local request context objects.
- [x] `orm_model_mixes_domain_logic_and_persistence_mapping`: flag ORM model classes that accumulate domain-behavior methods alongside persistence-mapping tags and API serialization annotations without a clear architectural exception.
- [x] `view_or_handler_performs_direct_file_system_io`: flag view or handler functions that call `open(...)`, `os.rename(...)`, `shutil.copy(...)`, or similar directly instead of delegating to a storage service or file helper.
- [x] `dependency_injection_bypassed_via_global_singleton`: flag code that imports and calls a module-level singleton client, session factory, or service instance directly instead of using constructor injection or a DI container resolving the dependency.

---

## 2. Async and Concurrency Correctness (20 rules)

- [x] `asyncio_gather_without_return_exceptions_on_partial_failure_path`: flag `asyncio.gather(*tasks)` calls on code paths where individual task failures should be isolated but `return_exceptions=True` is not passed.
- [x] `thread_local_storage_read_from_async_function`: flag `threading.local()` attribute accesses inside `async def` functions, where task scheduling means thread identity is not stable across await points.
- [x] `loop_run_until_complete_inside_running_loop`: flag `loop.run_until_complete(...)` called from inside an already running event loop, which raises `RuntimeError` at runtime.
- [x] `asyncio_sleep_zero_busy_wait_pattern`: flag tight retry or polling loops that use `await asyncio.sleep(0)` as a yield point instead of a structured condition variable or event.
- [x] `threading_thread_without_daemon_true_in_server_code`: flag `threading.Thread(...)` instances started without `daemon=True` in server or WSGI application code where non-daemon threads block clean shutdown.
- [x] `shared_mutable_collection_mutated_across_threads_without_lock`: flag module-scope mutable `list`, `dict`, or `set` objects appended or mutated from spawned threads without a visible lock guarding the mutation.
- [x] `multiprocessing_pool_created_without_context_manager_or_terminate`: flag `multiprocessing.Pool(...)` objects created without wrapping in a `with` block or calling `.terminate()` and `.join()` in a finally clause.
- [x] `concurrent_futures_executor_not_shut_down`: flag `ThreadPoolExecutor` or `ProcessPoolExecutor` objects created without explicit `.shutdown(wait=True)` or context-manager usage.
- [x] `asyncio_get_event_loop_at_module_scope`: flag `asyncio.get_event_loop()` or `asyncio.get_running_loop()` called at module scope during import instead of inside an async entry point or startup routine.
- [x] `threading_lock_acquired_blocking_inside_async_def`: flag `threading.Lock().acquire()` or `threading.RLock().acquire()` called inside `async def` without using `asyncio.Lock` or an executor, risking event-loop blocking.
- [x] `asyncio_queue_created_without_maxsize_in_producer_path`: flag `asyncio.Queue()` or `asyncio.LifoQueue()` instantiated without a `maxsize` argument in producer-heavy code that may outrun consumers.
- [x] `task_group_aggregated_exception_not_handled`: flag `asyncio.TaskGroup` usage or `trio.Nursery` scope without visible handling of `ExceptionGroup` or aggregated exceptions from failed subtasks.
- [x] `coroutine_result_discarded_without_await`: flag coroutine objects returned from helper function calls that are assigned or ignored without `await`, silently producing no-ops.
- [x] `thread_pool_max_workers_exceeds_cpu_count_for_cpu_bound_work`: flag `ThreadPoolExecutor(max_workers=N)` where N is notably larger than `os.cpu_count()` inside code paths described as CPU-bound computation.
- [x] `semaphore_acquired_without_async_with_context_manager`: flag manual `await semaphore.acquire()` and `semaphore.release()` outside an `async with` block, risking release being skipped on exception.
- [x] `gevent_monkey_patch_selective_but_uses_unpatched_stdlib`: flag `gevent.monkey.patch_select()` or other selective patches without patching `ssl`, `socket`, or `thread` when those stdlib modules are also used.
- [x] `asyncio_shield_outer_coroutine_cancelled_without_cleanup`: flag `asyncio.shield(coro)` patterns where the outer awaiting coroutine is cancelled without visible cleanup or fallback handling.
- [x] `sync_function_called_from_async_without_executor`: flag calls to known blocking I/O functions such as `time.sleep`, `open(...)`, `requests.get(...)`, or `subprocess.run(...)` from inside `async def` functions without `run_in_executor` offloading.
- [x] `async_context_manager_not_awaited_on_entry`: flag `with` instead of `async with` used on objects that implement `__aenter__` and `__aexit__`, silently bypassing the async protocol.
- [x] `untracked_create_task_result_may_hide_exception`: flag `asyncio.create_task(coro)` whose return value is immediately discarded without saving a reference, preventing exception observation and risking silent task failure.

---

## 3. Error Handling Discipline (20 rules)

- [x] `exception_raised_without_chaining_original_cause`: flag `raise NewError(...)` inside an `except` block that does not use `from e` or `from None`, silently losing the original exception cause.
- [x] `exception_handler_branches_on_error_message_string`: flag code that inspects `str(e)`, `e.args[0]`, or `e.message` text content to drive branching logic instead of matching on exception type or typed attributes.
- [x] `bare_except_clause_catches_system_exit`: flag `except:` clauses with no exception type specified, which catch `SystemExit`, `KeyboardInterrupt`, and `GeneratorExit` as unintended side effects.
- [x] `exception_logged_and_then_re_raised_redundantly`: flag patterns where an exception is logged at one layer and then re-raised to be logged again at a higher layer, producing duplicate log entries for the same failure.
- [x] `validation_or_parse_error_mapped_to_500_status`: flag exception handlers that translate input validation errors, `ValueError`, or `ParseError` into HTTP 500 responses instead of 400-class client error responses.
- [x] `exception_silenced_in_cleanup_or_finally_block`: flag `except` clauses inside `finally` or cleanup methods that silently discard exceptions via `pass` or `return`, hiding failures from the caller.
- [x] `project_exception_class_not_inheriting_shared_base`: flag custom exception classes within the project that inherit directly from `Exception` rather than a project-defined base exception hierarchy, making blanket handling impossible.
- [x] `exception_raised_and_caught_for_control_flow_within_same_function`: flag patterns where an exception is raised and caught within the same function body to direct normal code flow, which is better expressed with early returns or conditional branches.
- [x] `error_message_embeds_sensitive_data`: flag error message strings that interpolate raw SQL, database credentials, user-provided input, or environment variable values that should not appear in error output.
- [x] `retry_loop_catches_broad_base_exception`: flag retry loops that catch `Exception` or `BaseException` without also filtering to a declared set of retryable exception types.
- [x] `transaction_block_missing_rollback_on_exception`: flag SQLAlchemy, Django ORM, or raw DB transaction blocks that do not explicitly call `.rollback()` or re-raise on exception, leaving the session in an uncommitted state.
- [x] `assert_used_for_runtime_input_validation_in_production`: flag `assert` statements used to validate function inputs or external data in non-test production code, which can be silently removed by the `-O` interpreter flag.
- [x] `warning_issued_instead_of_exception_for_invalid_state`: flag `warnings.warn(...)` calls used to signal genuinely invalid preconditions or contract violations where raising an exception would be more appropriate.
- [x] `exception_handler_returns_default_without_any_logging`: flag broad `except` clauses that return a default value or reset state without logging or emitting any observable signal about the suppressed failure.
- [x] `deeply_nested_try_except_beyond_two_levels`: flag `try/except` blocks nested more than two levels deep within a single function, which typically signals missing decomposition.
- [x] `contextlib_suppress_applied_with_exception_base_class`: flag `contextlib.suppress(Exception)` or `contextlib.suppress(BaseException)` usage that suppresses all exceptions rather than a specific expected subset.
- [x] `custom_exception_encodes_identity_as_string_code_attribute`: flag custom exception classes that carry a string `code`, `error_code`, or `type` attribute as the primary identity discriminator instead of expressing identity through the class hierarchy.
- [x] `oserror_caught_without_errno_inspection`: flag broad `except OSError`, `except IOError`, or `except PermissionError` clauses that do not inspect `e.errno` or `e.strerror` where the specific OS failure kind determines recovery behavior.
- [x] `exception_group_raised_without_except_star_handlers`: flag code that raises or propagates `ExceptionGroup` in Python 3.11 or later without providing `except*` handlers that address the individual exception types within the group.
- [x] `generator_close_exception_not_handled_when_cleanup_required`: flag generator functions that perform teardown in their body but do not handle `GeneratorExit` or use a `try/finally` guard, risking incomplete teardown when the generator is closed before exhaustion.

---

## 4. Type System and API Contracts (15 rules)

- [x] `overloaded_dispatch_without_typing_overload_decorator`: flag functions that manually dispatch on `isinstance` checks for multiple input types but do not use `@typing.overload` to declare separate call signatures, leaving the API contract opaque to callers and type checkers.
- [x] `protocol_used_in_isinstance_without_runtime_checkable`: flag `Protocol` subclasses used as `isinstance(obj, SomeProtocol)` checks without the `@runtime_checkable` decorator, which raises `TypeError` at runtime.
- [x] `typevar_defined_without_bound_or_constraints_for_narrow_use`: flag `TypeVar` definitions without `bound=` or `constraints=` in call sites where the intended type set is clearly narrower than any type.
- [x] `generic_class_used_without_type_parameter_application`: flag uses of generic container classes such as `list`, `dict`, `Optional`, or project-defined `Generic[T]` subclasses in annotations without type parameter application, leaving the contract as an unparameterized raw type.
- [x] `optional_parameter_used_without_none_guard`: flag function bodies where a parameter annotated `Optional[T]` or `T | None` is dereferenced or called directly without an `is None` guard or a default override.
- [x] `callable_annotation_without_parameter_types`: flag `Callable` type hints used without specifying parameter types and return type, such as bare `Callable` or `Callable[..., Any]` on public API boundaries.
- [x] `typed_dict_total_false_without_docstring_noting_optional_keys`: flag `TypedDict` classes declared with `total=False` without a docstring or inline comment identifying which specific keys are optional and under what conditions.
- [x] `public_function_return_type_annotated_as_union_of_many_unrelated_types`: flag public functions annotated as returning `Union` with more than three unrelated member types where a more specific sum type or protocol would convey clearer intent.
- [x] `cast_applied_without_preceding_type_narrowing_guard`: flag `typing.cast(TargetType, value)` calls applied without a preceding `isinstance`, `assert`, or conditional guard that actually narrows the type, silencing type errors with no runtime safety.
- [x] `literal_type_annotation_enforced_only_by_runtime_string_equality`: flag `Literal[...]` annotated parameters where the value is validated inside the function by string equality only, without a guard that short-circuits on unrecognized values.
- [x] `string_forward_reference_in_annotation_not_under_type_checking_guard`: flag string-form forward references in annotations that are unresolvable at class definition time and that are not isolated under `if TYPE_CHECKING:`, preventing `get_type_hints()` from resolving them at the standard evaluation site.
- [x] `type_alias_shadows_builtin_name`: flag type alias assignments whose names are `list`, `dict`, `set`, `type`, `id`, `filter`, `map`, `input`, `bytes`, or other built-in names, shadowing the built-in for the rest of the module.
- [x] `protocol_method_lacks_type_annotations`: flag public `Protocol` method or property definitions that omit parameter or return type annotations, making the structural contract incomplete for static analyses.
- [x] `namedtuple_used_where_dataclass_better_fits`: flag `collections.namedtuple` used for data containers that need mutable fields, default values, post-init logic, or method extensions where `@dataclass` would be cleaner.
- [x] `typed_dict_key_access_without_get_or_guard`: flag direct `typed_dict_instance["key"]` indexing of `TypedDict` keys declared optional (`total=False`) without a `.get()` call or `"key" in d` guard.

---

## 5. Testing Anti-patterns (20 rules)

- [x] `test_function_stacks_too_many_mock_patch_decorators`: flag test functions with five or more `mock.patch(...)`, `mocker.patch(...)`, or `unittest.mock.patch` decorator applications, indicating the production code likely needs better dependency injection rather than more mocking.
- [x] `test_mutates_module_global_without_restore`: flag tests that assign to module-level globals, class variables, or singleton state without a corresponding teardown, fixture yield, or `monkeypatch` reset, risking test-ordering pollution.
- [x] `test_asserts_private_attribute_value_instead_of_behavior`: flag `assert obj._some_private_attribute == expected` patterns in tests that verify implementation internals rather than observable public behavior.
- [x] `test_function_covers_multiple_unrelated_scenarios`: flag single test functions that make three or more groups of unrelated assertions or set up multiple distinct scenarios, indicating the test should be split into focused cases.
- [x] `test_fixture_calls_datetime_now_without_freezing`: flag test fixtures or setup code that call `datetime.now()`, `datetime.utcnow()`, `time.time()`, or `date.today()` without time freezing via `freezegun`, `time-machine`, or monkeypatching.
- [x] `test_calls_time_sleep_for_coordination`: flag test code that calls `time.sleep(...)` or `asyncio.sleep(...)` with a non-negligible argument as a coordination mechanism instead of events, condition variables, or mock timers.
- [x] `integration_test_writes_state_without_cleanup`: flag integration or functional tests that write to real databases, filesystems, queues, or external services without visible teardown in a `finally`, `addCleanup`, or `yield`-fixture pattern.
- [x] `test_wraps_sut_in_try_except_hiding_exception_detail`: flag test code that wraps the system under test in `try/except` without asserting the exception type or message, hiding unexpected exceptions from the test result.
- [x] `pytest_parametrize_with_single_test_case`: flag `@pytest.mark.parametrize` decorators with only one parameter tuple, which is functionally identical to a plain test and adds ceremony without multiple coverage.
- [x] `test_imports_private_production_module`: flag test files that access private submodules (`_impl.py`, `_internal/`) from production packages directly instead of testing only through the package's public API.
- [x] `mock_return_value_is_incompatible_type_with_real_signature`: flag mock setup where `return_value` or `side_effect` produces a type that clearly cannot satisfy the real function's return annotation, making the test exercise incorrect assumptions.
- [x] `test_depends_on_sibling_test_side_effects`: flag test functions that read or rely on state established by a sibling test in the same module through shared mutable globals or file system state, violating test isolation.
- [x] `test_re_implements_production_validation_logic`: flag test files that manually re-implement business validation rules already present in the production code, creating a duplicate that can diverge silently.
- [x] `unittest_test_class_duplicates_setup_without_base_class`: flag `unittest.TestCase` subclasses that duplicate setUp or helper methods already present in other test classes without a shared base class or mixin.
- [x] `test_skipped_with_no_reason_string`: flag `@pytest.mark.skip`, `@pytest.mark.xfail`, or `unittest.skip(...)` usage without a reason string explaining why the test is disabled.
- [x] `test_compares_float_with_equality_operator`: flag tests asserting float equality via `==` without `pytest.approx(...)`, `math.isclose(...)`, or a tolerance argument, which is unreliable due to floating-point representation.
- [x] `stub_or_fake_class_duplicated_across_test_modules`: flag near-identical stub, fake, or test-double class definitions repeated across two or more test files that should be extracted into a shared `conftest.py` or test support module.
- [x] `test_loads_real_application_config_or_secrets`: flag tests that read `.env` files, call `load_dotenv()`, read `settings.py`, or access live secret stores instead of using isolated test configuration.
- [x] `pytest_raises_without_match_parameter_on_broad_exception`: flag `pytest.raises(SomeException)` without a `match=` parameter when `SomeException` is broad enough that any exception message would satisfy the assertion.
- [x] `test_makes_real_outbound_http_call_without_mock_or_vcr`: flag integration tests that perform real network requests to external services without a VCR cassette, `respx` mock, `responses` mock, or similar request isolation.

---

## 6. Security Boundaries (20 rules)

- [x] `sql_query_built_with_string_formatting_instead_of_parameters`: flag SQL statements built with `%s % values`, `.format(...)`, or f-strings instead of parameterized query placeholders, risking SQL injection.
- [x] `file_path_from_user_input_without_normalization_or_anchor_check`: flag file paths derived from user-controlled input that are not passed through `os.path.normpath`, `Path.resolve()`, or a check confirming the resolved path stays within an expected base directory.
- [x] `xml_parsing_with_external_dtd_or_entity_processing_enabled`: flag `lxml.etree.parse(...)`, `xml.etree.ElementTree.parse(...)`, or similar calls where external entity resolution or DTD processing is not explicitly disabled.
- [x] `http_client_url_built_from_user_input_without_allowlist`: flag `requests.get(url)`, `httpx.get(url)`, or similar calls where `url` is composed from query parameters, headers, or request body values without scheme validation or a hostname allowlist.
- [x] `subprocess_invoked_with_shell_true_and_user_derived_input`: flag `subprocess.run(cmd, shell=True)` where `cmd` is assembled from user-supplied values, environment variables, or external data sources, enabling command injection.
- [x] `jinja2_environment_created_with_autoescape_disabled`: flag `jinja2.Environment(autoescape=False)` or `Environment()` without `autoescape=select_autoescape(...)` being set, leaving template rendering vulnerable to XSS.
- [x] `jwt_decode_allows_none_algorithm_or_no_algorithm_restriction`: flag `jwt.decode(token, ...)` calls that accept `algorithms=["none"]` or that omit the `algorithms` parameter, allowing algorithm confusion attacks.
- [x] `cryptographic_secret_hardcoded_in_test_fixture_or_seed`: flag API keys, private keys, passwords, or cryptographic secrets hardcoded in test fixtures, seed files, or factory helpers that may be committed to revision control.
- [x] `insecure_hash_algorithm_used_for_security_sensitive_purpose`: flag `hashlib.md5(...)`, `hashlib.sha1(...)`, or `hmac` calls using `md5` or `sha1` for password hashing, token generation, or signature verification, where collision resistance is required.
- [x] `deserialization_from_external_or_user_controlled_source_with_pickle`: flag `pickle.loads(data)`, `marshal.loads(data)`, `shelve.open(...)`, or `dill.loads(data)` applied to data originating from HTTP requests, message queues, or user-uploaded files.
- [x] `debug_or_admin_endpoint_registered_without_environment_guard`: flag debug introspection routes, admin bootstrap endpoints, or test-only handlers registered in application routing without a visible `DEBUG` or environment check.
- [x] `weak_random_function_used_for_security_token_generation`: flag `random.random()`, `random.choice()`, `random.randint()`, or `random.token_hex()` used for session tokens, CSRF nonces, password reset tokens, or any value requiring cryptographic unpredictability instead of `secrets`.
- [x] `state_changing_endpoint_missing_csrf_protection`: flag POST, PUT, PATCH, or DELETE endpoints in Flask or Django that do not apply CSRF middleware or a CSRF token check visible in the route handler.
- [x] `ldap_search_filter_built_from_user_input_without_escaping`: flag LDAP filter strings assembled from user input without escaping using `ldap3.utils.conv.escape_filter_chars` or equivalent.
- [x] `regex_pattern_with_catastrophic_backtracking_applied_to_unbounded_input`: flag regular expression patterns containing nested quantifiers such as `(a+)+`, `(.*)*`, or similar exponential structures applied to strings sourced from user input.
- [x] `open_redirect_via_user_supplied_url_without_allowlist`: flag `redirect(url)` or `Response(302, Location=url)` calls where `url` derives from query parameters without a same-origin check or an allowed-destination allowlist.
- [x] `xml_entity_expansion_not_limited_in_lxml_or_elementtree`: flag XML parsing configurations that do not set `resolve_entities=False` or equivalent, remaining susceptible to exponential entity expansion attacks.
- [x] `arbitrary_file_write_via_user_controlled_path`: flag `open(path, "w")`, `shutil.copy(..., dst)`, or similar write operations where `path` or `dst` derives from user input without a safe-destination check.
- [x] `cors_allow_all_origins_set_without_production_environment_check`: flag CORS middleware or headers that set `Access-Control-Allow-Origin: *` or `allow_origins=["*"]` without a visible production-environment guard restricting it to specific trusted origins.
- [x] `server_side_template_injection_via_user_input_in_template_source`: flag calls that pass user-controlled strings as the template source string to `jinja2.from_string(...)`, `mako.Template(...)`, or similar instead of passing user values only as rendering context.

---

## 7. Memory and Resource Management (15 rules)

- [x] `unbounded_list_accumulation_inside_long_running_function`: flag code that appends to a list or dict inside a loop or long-running function body without a visible upper-bound check, eviction policy, or capacity limit, risking unbounded memory growth.
- [x] `generator_consumed_twice_without_recreation`: flag generator or iterator objects consumed by `for`, `list(...)`, or `next(...)` a second time in the same function without being recreated, silently producing empty results on the second pass.
- [x] `file_object_returned_or_stored_without_clear_close_path`: flag `open(...)` calls in non-context-manager form whose resulting file object is returned to a caller or stored on an instance attribute without a documented close path.
- [x] `weakref_dereferenced_without_live_check`: flag `weakref.ref(obj)()` or `ref = weakref.ref(obj); ref.something` patterns that dereference the weak reference without checking whether the object is still alive.
- [x] `functools_lru_cache_applied_to_instance_method`: flag `@functools.lru_cache` or `@cache` applied directly to instance methods where `self` is included in the cache key, keeping instances alive indefinitely in the cache.
- [x] `bidirectional_object_reference_without_weakref`: flag parent-child or observer-subject relationships where both sides hold strong references to each other without using `weakref` on one side, creating uncollectable cycles.
- [x] `subprocess_pipe_without_communicate_for_large_output`: flag `subprocess.Popen(stdout=subprocess.PIPE)` or `subprocess.run(stdout=subprocess.PIPE)` on commands expected to produce large outputs without calling `communicate()` or reading stdout incrementally, risking deadlock.
- [x] `socket_opened_without_context_manager_or_guaranteed_close`: flag `socket.socket(...)` creation outside a `with` block or `try/finally` that calls `sock.close()`, leaving the socket open on exception paths.
- [x] `large_binary_payload_retained_across_await_in_async_handler`: flag large binary buffers such as image bytes, upload file content, or large JSON blobs stored in local variables across `await` points when they can be processed and released before the yield.
- [x] `db_connection_pool_size_exceeds_server_max_connections`: flag SQLAlchemy `create_engine(pool_size=N, max_overflow=M)` where `N + M` exceeds the known or default `max_connections` of the target database, causing connection failures under load.
- [x] `repeated_deepcopy_in_loop_on_same_source_object`: flag `copy.deepcopy(obj)` called inside a loop on the same source object each iteration when the source is invariant and the copy could be hoisted before the loop.
- [x] `redis_commands_issued_individually_in_loop_without_pipeline`: flag multiple individual Redis `set`, `get`, `hset`, `lpush`, or similar commands issued inside a loop without using a Redis pipeline or transaction block to batch the round trips.
- [x] `unclosed_tempfile_or_tmp_directory_from_tempfile_module`: flag `tempfile.NamedTemporaryFile(...)` or `tempfile.mkdtemp(...)` usage without a `with` block, `delete=True`, or a `shutil.rmtree` call in a `finally` clause.
- [x] `closure_captures_large_object_after_producing_function_returns`: flag closures that capture objects such as DataFrames, large dicts, or model weights into the closure scope when those objects are only needed to compute the callback and could be released after construction.
- [x] `object_allocated_in_tight_loop_expected_to_be_pooled`: flag repeated instantiation of heavyweight objects such as compiled regex patterns, database session objects, HTTP client instances, or connection pools inside tight loops where pooling or hoisting would eliminate the repeated allocation cost.

---

## 8. Configuration and Secrets Hygiene (15 rules)

- [x] `dotenv_load_dotenv_called_from_multiple_modules`: flag `dotenv.load_dotenv()` called in two or more source files instead of once at application entry point, risking double-loading and environment key precedence confusion.
- [x] `pydantic_settings_model_allows_post_init_mutation`: flag `pydantic-settings` or `pydantic.BaseModel` configuration classes that do not use `model_config = ConfigDict(frozen=True)` or `allow_mutation = False`, allowing post-initialization field mutation.
- [x] `feature_flag_checked_via_inline_env_lookup_across_handlers`: flag three or more distinct handler or service functions that each call `os.getenv("ENABLE_FEATURE_XYZ")` directly instead of routing those checks through a centralized feature flag interface.
- [x] `secrets_manager_client_created_per_function_call`: flag AWS Secrets Manager, GCP Secret Manager, Azure Key Vault, or HashiCorp Vault client constructors called inside request handler or service function bodies instead of being created once and reused.
- [x] `same_config_key_has_different_defaults_in_multiple_modules`: flag the same configuration key appearing with different fallback values in separate `os.getenv("KEY", default)` calls across the codebase, creating inconsistency risk.
- [x] `pydantic_settings_model_missing_env_prefix_isolation`: flag `pydantic-settings` `BaseSettings` subclasses that do not configure `env_prefix` or a scoped `model_config`, allowing environment variables from one service to collide with another in shared deployment environments.
- [x] `toml_or_ini_config_file_parsed_on_request_path`: flag `tomllib.load(...)`, `tomli.load(...)`, or `configparser.read(...)` calls executed inside request handlers or per-invocation paths instead of at application startup.
- [x] `startup_log_statement_includes_raw_secret_value`: flag `logging.info(...)`, `print(...)`, or similar calls at startup that interpolate raw passwords, API keys, connection string credentials, or token values into the log message.
- [x] `pydantic_settings_model_does_not_forbid_extra_fields`: flag `BaseSettings` or config dataclasses that do not set `extra = "forbid"` or `model_config = ConfigDict(extra="forbid")`, silently swallowing unknown configuration keys and hiding deployment mistakes.
- [x] `database_url_assembled_from_unvalidated_environment_parts`: flag cases where a database URL or DSN is built by concatenating individual environment variables without validating the assembled format before passing to a driver.
- [x] `remote_feature_flag_service_polled_without_local_cache`: flag feature flag values fetched from LaunchDarkly, Split.io, Statsig, or similar services on every request path without a local cache, TTL, or SDK-managed polling interval.
- [x] `yaml_config_loaded_without_safe_loader`: flag `yaml.load(config_data, Loader=yaml.Loader)` or `yaml.full_load(...)` used for application configuration files where `yaml.safe_load(...)` or `Loader=yaml.SafeLoader` would prevent `!!python/object` tag execution.
- [x] `application_config_values_validated_lazily_on_first_use`: flag applications where configuration fields are read and validated on first access during a live request or background task rather than at startup, deferring initialization errors into production traffic.
- [x] `multiple_config_sources_merged_without_documented_precedence_order`: flag applications that merge CLI argument values, environment variables, and config files without a documented and enforced precedence order, creating unpredictable configuration resolution.
- [x] `sensitive_config_key_included_in_debug_level_log_dict_dump`: flag debug-level log statements that call `logger.debug("config: %s", vars(settings))` or `logger.debug(config.__dict__)`, risking accidental exposure of secret values in development log streams.

---

## 9. Logging and Observability (15 rules)

- [x] `logging_basic_config_called_from_library_package`: flag `logging.basicConfig(...)` or `logging.getLogger().addHandler(handler)` calls in library code that users import, which override the host application's logging configuration.
- [x] `f_string_evaluated_eagerly_inside_logging_call`: flag `logger.debug(f"computed {value}")` or `logging.info(f"...")` patterns where the f-string is evaluated before the log level is checked, performing unnecessary string formatting when the level is disabled.
- [x] `logger_error_inside_except_without_exc_info`: flag `logger.error(message)`, `logger.warning(message)`, or `logger.critical(message)` calls inside `except` blocks that do not pass `exc_info=True` or `exc_info=sys.exc_info()`, losing the stack trace from the output.
- [x] `logging_set_level_hardcoded_at_module_scope`: flag `logging.getLogger(__name__).setLevel(logging.DEBUG)` or similar hardcoded level assignments at module scope that override the application-level log configuration.
- [x] `distributed_trace_span_created_without_parent_context_propagation`: flag functions that start new spans using `tracer.start_span(...)` or `with tracer.start_as_current_span(...)` without extracting and linking the parent span context from the incoming request.
- [x] `observability_metric_names_use_inconsistent_separators`: flag metric names emitted via Prometheus, statsd, or OpenTelemetry that mix dots, slashes, and underscores in the same service without a clear naming convention.
- [x] `health_check_handler_queries_slow_database_table`: flag `/health`, `/healthz`, or `/readiness` endpoints that execute full ORM queries, `COUNT(*)`, or multi-join selects instead of lightweight `SELECT 1` probes or cached liveness flags.
- [x] `structured_log_uses_inconsistent_field_names_for_same_concept`: flag JSON-structured log records across handlers or services that use different keys for the same concept such as `user_id`, `userId`, `uid`, or `account_id` in the same API module.
- [x] `exception_swallowed_before_sentry_or_error_tracker_capture`: flag exceptions caught and re-wrapped before being passed to `sentry_sdk.capture_exception(...)` or `rollbar.report_exc_info(...)`, sending only the outer wrapper context to the error tracker.
- [x] `high_frequency_code_path_logs_without_sampling_or_rate_limit`: flag `logger.info(...)` or `logger.debug(...)` calls inside tight loops, per-row processing functions, or per-request hot paths without a log sampling guard or rate-limit token check.
- [x] `opentelemetry_span_attribute_attaches_pii_fields`: flag `span.set_attribute("user.email", email)`, `span.set_attribute("ip_address", ip)`, or similar OpenTelemetry span attribute calls that attach PII data without a documented scrubbing or masking policy.
- [x] `structured_log_record_missing_trace_or_correlation_id`: flag JSON-structured log records emitted inside request handling paths that do not include a `trace_id`, `request_id`, or `correlation_id` field, making distributed debugging impossible without log-to-trace correlation.
- [x] `logging_call_inside_signal_handler_function`: flag `logging.info(...)`, `logger.error(...)`, or `print(...)` statements inside Python `signal.signal(...)` handlers, which is not async-signal-safe and can cause deadlocks when interrupted during logging I/O.
- [x] `alert_or_slo_threshold_hardcoded_inside_application_logic`: flag numeric alerting thresholds, SLO error budget percentages, or latency targets hardcoded in application source code instead of managed via external configuration or observability platform rules.
- [x] `prometheus_or_statsd_metric_emitted_inside_db_result_loop`: flag Prometheus `counter.inc()`, `histogram.observe()`, or `statsd.increment(...)` calls made inside per-row database result loops, adding per-result serialization and lock overhead.

---

## 10. Module and Package Design (15 rules)

- [x] `star_import_used_in_non_init_production_module`: flag `from module import *` usage in production Python files that are not `__init__.py` re-export shims, polluting the importing namespace and hiding dependency origins.
- [x] `public_package_missing_all_list`: flag Python packages with public symbols that lack an `__all__` definition, making the public API surface ambiguous for callers and static tools.
- [x] `relative_import_crossing_sibling_package_boundary`: flag relative imports using `..` that cross into sibling packages not intended to be sub-packages of a shared parent, creating coupling across independent modules.
- [x] `optional_library_import_checked_on_hot_code_path`: flag `try: import optional_lib except ImportError: optional_lib = None` patterns where the `optional_lib is None` check happens inside a request handler or tight loop instead of at module initialization.
- [x] `importlib_import_module_called_inside_request_handler`: flag `importlib.import_module(name)` called inside request handlers, task bodies, or tight loops instead of resolving dynamic imports at startup, incurring repeated import machinery overhead.
- [x] `init_file_re_exports_private_module_symbols`: flag `__init__.py` files that explicitly import and re-export symbols whose names begin with `_`, advertising private implementation details as part of the package's public API.
- [x] `implicit_namespace_package_used_where_regular_package_needed`: flag directories that lack `__init__.py` but are treated as a single importable package when they need initialization code, data file discovery, or subpackage consistency.
- [x] `pkg_resources_used_for_runtime_version_lookup`: flag `pkg_resources.get_distribution("package").version` or `pkg_resources.require("package")` calls performed at runtime instead of using `importlib.metadata.version("package")`.
- [x] `module_level_side_effect_outside_main_guard`: flag module-level code that prints output, registers signal handlers, creates files, or starts threads outside of `if __name__ == "__main__":` or an explicit initialization function, causing side effects on import.
- [x] `test_support_helpers_located_inside_production_package`: flag test helpers, factory functions, fake client implementations, or test fixture builders located inside a production package directory instead of the `tests/` tree.
- [x] `conftest_imports_private_submodule_from_production_package`: flag `conftest.py` files that import `_internal` submodules, `_impl.py` modules, or `_private` packages directly from production code, coupling test infrastructure to implementation details.
- [x] `dynamic_plugin_loaded_from_config_without_registry_allowlist`: flag `importlib.import_module(plugin_name)` calls where `plugin_name` is derived from a config file or user input without validation against a known plugin registry or allowlist.
- [x] `importlib_metadata_version_queried_inside_request_loop`: flag `importlib.metadata.version("package")` called inside request handlers or loops when the result is invariant per process and could be cached once at application startup.
- [x] `two_local_packages_expose_modules_with_same_unqualified_name`: flag cases where two separate local packages contain modules with the same unqualified name such that one shadows the other depending on import order.
- [x] `init_file_implicitly_re_exports_submodule_symbols_without_all`: flag `__init__.py` files that import from submodules with bare `from .sub import SomeClass` without declaring `__all__`, making the public boundary implicit and fragile to submodule refactoring.

---

## 11. Data Structure and Algorithm Choices (15 rules)

- [x] `sorted_full_collection_to_extract_top_n_elements`: flag code that calls `sorted(collection)[0:n]` or `sorted(collection)[-n:]` when `heapq.nsmallest(n, collection)` or `heapq.nlargest(n, collection)` would do the same work in O(k log n) instead of O(k log k).
- [x] `linear_membership_test_in_loop_over_large_static_list`: flag repeated `if x in large_list` membership tests inside loops where the list is built once and never modified, and where a `set` or `frozenset` index would reduce each test from O(n) to O(1).
- [x] `manual_dict_increment_instead_of_counter_or_defaultdict`: flag `if key in d: d[key] += 1; else: d[key] = 1` patterns that are cleaner and less error-prone with `collections.Counter` or `collections.defaultdict(int)`.
- [x] `ordered_dict_used_in_python_37_plus_where_dict_suffices`: flag `collections.OrderedDict` usage in code confirmed to run on Python 3.7 or later where the insertion-order guarantee of built-in `dict` already covers the stated need.
- [x] `sorted_list_maintained_with_insert_instead_of_bisect_insort`: flag code that maintains a sorted list by calling `list.sort()` or linear-scan insertion after each new element instead of using `bisect.insort`, which keeps insertion O(n) but avoids the O(k log k) re-sort.
- [x] `namedtuple_fields_accessed_by_integer_index`: flag `point[0]`, `record[2]`, or similar zero-based index access on `namedtuple` instances where accessing by field name would be clearer and would not break on field reordering.
- [x] `list_pop_zero_used_as_queue_operation`: flag `list.pop(0)` or `list.insert(0, item)` used to simulate a FIFO queue, where `collections.deque.popleft()` and `deque.appendleft()` are O(1) instead of O(n).
- [x] `filter_and_map_results_materialized_to_list_at_each_step`: flag chained `list(filter(..., list(map(..., data))))` patterns that materialize an intermediate list at each transformation step when a single generator pipeline would defer all work to final consumption.
- [x] `defaultdict_created_with_lambda_instead_of_builtin_factory`: flag `collections.defaultdict(lambda: [])` where `defaultdict(list)` is clearer and avoids the closure allocation, and similarly `lambda: {}` vs `dict`, `lambda: 0` vs `int`.
- [x] `zip_range_len_used_instead_of_enumerate`: flag `zip(range(len(x)), x)` patterns where `enumerate(x)` produces the same indexed iteration more idiomatically.
- [x] `frozenset_not_used_for_constant_membership_set_rebuilt_per_call`: flag constant membership-test sets such as `ALLOWED_METHODS = {"GET", "POST", "PUT"}` declared with `set(...)` instead of `frozenset(...)`, incurring a mutable allocation on each call when a module-level constant would be evaluated once.
- [x] `deque_maxlen_behavior_not_accounted_for_by_caller`: flag `collections.deque(maxlen=N)` objects used in contexts where the caller appends items and then consumes all of them, potentially unaware that older items are silently dropped once the deque is full.
- [x] `counter_most_common_all_items_retrieved_for_top_one`: flag `Counter(data).most_common()` or `Counter(data).most_common(len(counter))` when only the single most-common element is needed, where `max(counter, key=counter.get)` is simpler.
- [x] `repeated_key_hash_via_dict_lookup_in_tight_loop`: flag repeated `d[key]` accesses to the same key within a tight loop where caching `value = d[key]` once before the loop would eliminate redundant hash computations.
- [x] `chain_of_boolean_or_conditions_over_same_value_not_using_in_operator`: flag `x == "a" or x == "b" or x == "c"` patterns with three or more equality checks against the same variable where `x in {"a", "b", "c"}` is more concise and measurably faster.

---

## 12. Web API Design Anti-patterns (10 rules)

- [x] `api_endpoint_returns_json_without_documented_response_schema`: flag API route handlers that return `jsonify(...)`, `JSONResponse(...)`, or `dict` payloads without a Pydantic response model, `TypedDict`, or OpenAPI annotation describing the shape, leaving the contract undocumented.
- [x] `response_envelope_shape_inconsistent_across_siblings_in_same_router`: flag sibling endpoints in the same router or blueprint that return different top-level payload structures such as some with a `{"data": ...}` wrapper and some with bare objects, without a shared serializer or response helper.
- [x] `cursor_based_pagination_missing_stable_sort_tiebreaker`: flag cursor-based pagination implementations that derive the cursor from a non-unique column or that do not include a stable tiebreaker in the sort order, producing non-deterministic pages when the primary sort column has duplicate values.
- [x] `bulk_endpoint_partial_failure_contract_ambiguous`: flag bulk create, update, or delete endpoints that do not clearly document or enforce whether they are all-or-nothing transactional or support partial success with per-item error reporting.
- [x] `api_versioning_in_url_without_matching_router_group`: flag URL path segments such as `/v1/`, `/v2/`, or `/api/v3/` without a matching router group, blueprint prefix, or versioned sub-application isolating all routes under that version.
- [x] `rate_limit_429_response_missing_retry_after_header_or_stable_body`: flag rate-limiting middleware or decorators that return HTTP 429 responses without a `Retry-After` header or a stable response body shape, depriving clients of actionable backoff information.
- [x] `pydantic_validation_error_detail_forwarded_with_internal_field_aliases`: flag validation error responses that forward raw Pydantic `ValidationError` detail objects including internal `alias` field names from persistence models, leaking implementation details to API consumers.
- [x] `state_changing_endpoint_returns_200_with_empty_body`: flag endpoints that perform resource creation, update, or deletion operations and return HTTP 200 with an empty or meaningless body instead of 201 with a location header, 202 with a job reference, or 204.
- [x] `binary_or_multipart_response_missing_explicit_content_type`: flag endpoints that stream file bytes, images, or multipart data without setting an explicit `Content-Type` response header, relying on client-side content sniffing which can be suppressed by browser security policies.
- [x] `large_response_body_fully_buffered_in_memory_before_send`: flag endpoints that assemble large JSON arrays, CSV exports, or binary archives fully in memory before sending when `StreamingResponse`, generator-based responses, or chunked transfer encoding would allow early data delivery and reduce peak memory.
