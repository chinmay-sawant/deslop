use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus, bindings};

macro_rules! arch_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "architecture",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: bindings::PYTHON_ARCHITECTURE,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    // ── Section 1 · Architecture and Layer Boundaries ──────────────────────
    arch_rule!(
        "service_method_accepts_http_request_object",
        "Service-layer method parameter list includes an HTTP request type from Flask, Django, or FastAPI."
    ),
    arch_rule!(
        "repository_returns_unexecuted_orm_query",
        "Repository method returns a raw ORM query object instead of executing it and returning domain values."
    ),
    arch_rule!(
        "view_or_handler_constructs_orm_query_directly",
        "View or route handler builds an ORM query rather than delegating to a repository or service."
    ),
    arch_rule!(
        "domain_model_class_imports_http_library",
        "Domain model or entity file imports an HTTP framework module such as Flask, Django, or FastAPI."
    ),
    arch_rule!(
        "service_raises_or_catches_http_exception_type",
        "Service-layer function raises or catches HTTP exception types that belong to the transport layer."
    ),
    arch_rule!(
        "handler_or_view_builds_raw_sql",
        "Route handler or view constructs a raw SQL string instead of delegating to a repository layer."
    ),
    arch_rule!(
        "service_method_returns_http_response_object",
        "Service-layer method returns an HTTP response object that belongs to the transport layer."
    ),
    arch_rule!(
        "handler_or_view_owns_transaction_lifecycle",
        "Route handler or view manually manages database transaction boundaries (begin/commit/rollback)."
    ),
    arch_rule!(
        "service_reads_settings_inline_instead_of_injected",
        "Service reads configuration from os.getenv or settings module inline rather than receiving it via injection."
    ),
    arch_rule!(
        "view_or_handler_performs_direct_file_system_io",
        "Route handler or view calls open(), os.path, or pathlib directly on the request path."
    ),
    arch_rule!(
        "business_logic_inside_middleware",
        "Middleware component encodes domain decision logic beyond cross-cutting concerns such as auth and logging."
    ),
    arch_rule!(
        "dependency_injection_bypassed_via_global_singleton",
        "Function bypasses the DI container and retrieves a collaborator via a global variable or module-level singleton."
    ),
    arch_rule!(
        "auth_extraction_duplicated_across_views",
        "Multiple view functions repeat identical auth or principal-extraction logic instead of sharing via middleware or dependency."
    ),
    arch_rule!(
        "background_job_depends_on_request_context_object",
        "Celery, RQ, or background task function holds a reference to an HTTP request context object across the task boundary."
    ),
    arch_rule!(
        "repository_method_accepts_pydantic_request_schema",
        "Repository method parameter type is a Pydantic request schema that belongs to the presentation layer."
    ),
    arch_rule!(
        "celery_or_rq_task_imports_web_framework_app",
        "Celery or RQ task module imports the Flask app object or FastAPI application at the top level."
    ),
    arch_rule!(
        "persistent_model_field_encodes_transport_concern",
        "SQLAlchemy or Django ORM model carries a field that encodes HTTP or transport metadata such as status codes."
    ),
    arch_rule!(
        "orm_model_mixes_domain_logic_and_persistence_mapping",
        "ORM model class contains domain business logic methods alongside column mapping, coupling persistence and domain concerns."
    ),
    arch_rule!(
        "validation_rules_duplicated_at_dto_and_domain_layer",
        "Identical validation constraint is applied at both the DTO/schema layer and the domain entity layer."
    ),
    // ── Section 2 · Async / Concurrency Correctness ────────────────────────
    arch_rule!(
        "asyncio_gather_without_return_exceptions_on_partial_failure_path",
        "asyncio.gather called without return_exceptions=True on a code path where partial failure should be recoverable."
    ),
    arch_rule!(
        "thread_local_storage_read_from_async_function",
        "Async function reads from threading.local() storage, which is per-thread not per-coroutine."
    ),
    arch_rule!(
        "loop_run_until_complete_inside_running_loop",
        "loop.run_until_complete called from within a running event loop, which raises RuntimeError."
    ),
    arch_rule!(
        "asyncio_sleep_zero_busy_wait_pattern",
        "asyncio.sleep(0) used in a tight loop as a busy-wait yield pattern instead of a proper event-driven wait."
    ),
    arch_rule!(
        "threading_thread_without_daemon_true_in_server_code",
        "threading.Thread created in server-side code without daemon=True, which can block graceful shutdown."
    ),
    arch_rule!(
        "shared_mutable_collection_mutated_across_threads_without_lock",
        "Shared mutable collection such as a list or dict is mutated from multiple threads without a lock."
    ),
    arch_rule!(
        "multiprocessing_pool_created_without_context_manager_or_terminate",
        "multiprocessing.Pool created without a context manager or explicit .terminate() call, leaking worker processes."
    ),
    arch_rule!(
        "concurrent_futures_executor_not_shut_down",
        "ThreadPoolExecutor or ProcessPoolExecutor created without shutdown() or a context manager, leaking threads."
    ),
    arch_rule!(
        "threading_lock_acquired_blocking_inside_async_def",
        "threading.Lock.acquire() called with blocking=True inside an async function, blocking the event loop thread."
    ),
    arch_rule!(
        "asyncio_queue_created_without_maxsize_in_producer_path",
        "asyncio.Queue created without maxsize on a producer code path, allowing unbounded memory growth."
    ),
    arch_rule!(
        "coroutine_result_discarded_without_await",
        "Coroutine object assigned or returned without await, so the coroutine body never executes."
    ),
    arch_rule!(
        "sync_function_called_from_async_without_executor",
        "Blocking I/O or CPU-bound function called directly inside async def without loop.run_in_executor."
    ),
    arch_rule!(
        "untracked_create_task_result_may_hide_exception",
        "asyncio.create_task result discarded without storing a reference; exceptions raised in the task are silenced."
    ),
    arch_rule!(
        "semaphore_acquired_without_async_with_context_manager",
        "asyncio.Semaphore acquired via .acquire() without async with, risking a missed release on exception."
    ),
    // ── Section 2 · File-level (asyncio_get_event_loop_at_module_scope) ────
    arch_rule!(
        "asyncio_get_event_loop_at_module_scope",
        "asyncio.get_event_loop() or get_running_loop() called at module import scope outside any function."
    ),
];
