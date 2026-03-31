# Plan 2 - Flask, Django, And Web Framework Performance And Bad Practices (Python)

Date: 2026-04-01

## Status

- [ ] Initial slice not yet implemented.
- [ ] This plan extends the current Python heuristics with framework-aware Flask, Django, and FastAPI performance and bad-practice analysis.
- [ ] The target set focuses on request-path waste, ORM misuse, template rendering cost, middleware allocation churn, and response shaping that common Python linters usually do not model.

## Already Covered And Excluded From This Plan

- [x] `blocking_sync_io_in_async` (partially covers sync-in-async but not framework-specific)
- [x] `network_boundary_without_timeout`
- [x] `external_input_without_validation`
- [x] `full_dataset_load`
- [x] `eval_exec_usage`
- [x] `exception_swallowed`
- [x] `broad_exception_handler`
- [x] `import_time_network_call`
- [x] `import_time_file_io`
- [x] `unsafe_yaml_loader`
- [x] `pickle_deserialization_boundary`
- [x] `subprocess_shell_true`

## Objective

Move from generic Python performance analysis to framework-aware heuristics that understand Django ORM patterns, Flask request handling, FastAPI dependency injection, SQLAlchemy session lifecycle, template rendering costs, and handler-driven upstream fanout. These rules should stay focused on obvious throughput regressions and dangerous practices rather than stylistic concerns.

## Candidate Scenario Backlog (42 scenarios)

### Django ORM And Database Access Patterns

- [ ] `django_queryset_evaluated_multiple_times`: detect the same `QuerySet` variable being iterated, sliced, or tested for truthiness multiple times in one view or function, triggering duplicate SQL queries.
- [ ] `django_n_plus_one_no_select_related`: detect loops that access `ForeignKey` or `OneToOneField` attributes on queryset results without `select_related()` or `prefetch_related()` in the originating query.
- [ ] `django_n_plus_one_no_prefetch_related`: detect loops that access reverse relations or `ManyToManyField` attributes without `prefetch_related()`.
- [ ] `django_queryset_count_then_exists`: detect `.count() > 0` or `len(queryset) > 0` when `.exists()` would avoid a full count.
- [ ] `django_queryset_len_instead_of_count`: detect `len(queryset)` which evaluates the full queryset into memory instead of `.count()` which uses SQL `COUNT`.
- [ ] `django_all_without_limit_in_view`: detect `Model.objects.all()` or `.filter(...)` without `.only()`, `[:limit]`, or pagination in view or API handler code.
- [ ] `django_values_vs_full_model_in_loop`: detect full model hydration (`Model.objects.filter(...)`) followed by only accessing 1-2 fields, when `.values()` or `.values_list()` would avoid loading unnecessary columns.
- [ ] `django_save_full_model_in_loop`: detect `instance.save()` on full model objects inside loops without `update_fields` parameter, causing all columns to be written each time.
- [ ] `django_create_single_in_loop`: detect `Model.objects.create(...)` or `Model(...).save()` inside loops when `bulk_create()` is available.
- [ ] `django_update_single_in_loop`: detect `.save()` or `Model.objects.filter(pk=...).update(...)` inside loops when `bulk_update()` or a single `QuerySet.update()` is more appropriate.
- [ ] `django_delete_single_in_loop`: detect `instance.delete()` inside loops when `QuerySet.delete()` would issue a single SQL statement.
- [ ] `django_raw_sql_in_loop`: detect `connection.cursor().execute(...)` or `Model.objects.raw(...)` inside loops without batching.
- [ ] `django_migration_code_in_view`: detect `migrate`, `makemigrations`, schema inspection, or `AutoMigrate`-equivalent calls on request paths.
- [ ] `django_queryset_order_by_random`: detect `.order_by('?')` in view code which causes `ORDER BY RANDOM()` and full table scans.

### Flask Request Handling And Bad Practices

- [ ] `flask_request_body_parsed_multiple_times`: detect accessing `request.get_json()` multiple times in the same view function instead of caching the result.
- [ ] `flask_global_db_connection_per_request`: detect `sqlite3.connect(...)`, `pymongo.MongoClient(...)`, or `psycopg2.connect(...)` inside Flask view functions instead of using app-scoped connection pooling.
- [ ] `flask_app_config_read_per_request`: detect `app.config[...]`, `os.environ.get(...)`, or config file loads inside view functions when the value could be read once at startup.
- [ ] `flask_template_rendered_from_string_in_view`: detect `render_template_string(...)` inside view functions when a template file would avoid repeated Jinja2 compilation.
- [ ] `flask_file_read_per_request`: detect `open(...).read()` or `Path(...).read_text()` inside view functions for static content that could be cached.
- [ ] `flask_json_encoder_per_request`: detect custom `JSONEncoder()` instantiation inside view functions instead of configuring the app-level encoder.
- [ ] `flask_no_streaming_for_large_response`: detect large list materialization followed by `jsonify(...)` or response construction when `Response(generate(), ...)` would enable streaming.
- [ ] `flask_debug_mode_in_production_code`: detect `app.run(debug=True)` or `FLASK_DEBUG=True` outside of obvious development entrypoints.

### FastAPI And Async Web Framework Patterns

- [ ] `fastapi_sync_def_with_blocking_io`: detect `def` (non-async) route handlers in FastAPI that contain blocking I/O calls like `requests.get(...)`, `open(...)`, or `time.sleep(...)`, which block the event loop thread pool.
- [ ] `fastapi_dependency_creates_client_per_request`: detect FastAPI `Depends(...)` functions that instantiate HTTP clients, DB connections, or heavy resources per request instead of using app lifespan.
- [ ] `fastapi_response_model_without_orm_mode`: detect FastAPI response models used with ORM objects without `model_config` or `orm_mode`, causing serialization failures or double queries.
- [ ] `fastapi_background_task_exception_silent`: detect `BackgroundTasks.add_task(...)` with functions that have no exception handling, leading to silently swallowed errors.

### SQLAlchemy Session And Connection Patterns

- [ ] `sqlalchemy_session_not_closed`: detect `Session()` or `sessionmaker()()` usage without a context manager or explicit `session.close()` call.
- [ ] `sqlalchemy_query_in_loop`: detect `session.query(Model).filter(...)` or `session.execute(select(...))` inside loops without batching via `in_()` or bulk operations.
- [ ] `sqlalchemy_n_plus_one_lazy_load`: detect attribute access on relationship properties inside loops without `joinedload()`, `subqueryload()`, or `selectinload()` in the original query.
- [ ] `sqlalchemy_commit_per_row_in_loop`: detect `session.commit()` inside loops instead of a single commit after the batch.
- [ ] `sqlalchemy_create_engine_per_request`: detect `create_engine(...)` inside request handlers or frequently-called functions instead of process-level engine reuse.
- [ ] `sqlalchemy_expire_on_commit_default_in_async`: detect `expire_on_commit=True` (the default) in async SQLAlchemy sessions, which forces implicit I/O on attribute access after commit.

### Template Rendering And Response Construction

- [ ] `template_render_in_loop`: detect `jinja2.Template(...)`, `Template(string).render(...)`, or `render_template_string(...)` inside loops instead of rendering once with loop data.
- [ ] `response_json_dumps_then_response_object`: detect `json.dumps(data)` followed by manual `Response(...)` construction when framework-provided `jsonify(...)` or `JSONResponse(...)` is available and handles Content-Type.
- [ ] `large_dict_literal_response_in_handler`: detect large inline dict literal construction immediately before JSON response in hot handlers, when a Pydantic model or typed response would be reusable and cheaper.

### Middleware And Request-Scope Allocation

- [ ] `middleware_creates_http_client_per_request`: detect `requests.Session()`, `httpx.Client()`, or `aiohttp.ClientSession()` being instantiated inside middleware or before-request hooks instead of using app-scoped clients.
- [ ] `middleware_loads_config_file_per_request`: detect YAML, JSON, TOML, or INI config file parsing inside middleware or request hooks instead of startup-time loading.
- [ ] `middleware_compiles_regex_per_request`: detect `re.compile(...)` inside middleware or per-request hooks instead of module-level compilation.

### Handler Fanout And Upstream Call Patterns

- [ ] `upstream_http_call_per_item_in_handler`: detect sequential `requests.get(...)`, `httpx.get(...)`, or `aiohttp` calls inside loops within view/handler functions without batching or concurrent execution.
- [ ] `upstream_call_without_timeout_in_handler`: detect HTTP client calls in handlers without explicit `timeout` parameter, risking unbounded request latency.
- [ ] `upstream_response_not_checked_before_decode`: detect `response.json()` or `json.loads(response.text)` without checking `response.status_code` or `response.ok` first.

## Shared Implementation Checklist

- [ ] Add framework-aware import classification for `flask`, `django`, `fastapi`, `sqlalchemy`, `jinja2`, `requests`, `httpx`, `aiohttp`, `pymongo`, `psycopg2`, and `redis`.
- [ ] Add view/handler detection using decorator evidence: `@app.route(...)`, `@api_view(...)`, `@router.get(...)`, `@action(...)`, and class-based view inheritance from `View`, `APIView`, `ViewSet`.
- [ ] Add queryset chain summaries for Django ORM: `filter`, `exclude`, `select_related`, `prefetch_related`, `only`, `defer`, `values`, `values_list`, `order_by`, `distinct`, `count`, `exists`, `first`, `last`, `all`, `create`, `bulk_create`, `update`, `bulk_update`, `delete`, `raw`, `annotate`, `aggregate`.
- [ ] Add SQLAlchemy session lifecycle summaries: `Session()`, `session.query(...)`, `session.execute(...)`, `session.add(...)`, `session.commit()`, `session.close()`, `session.flush()`, and relationship loading strategies.
- [ ] Gate handler-only findings on decorator evidence or class-based view inheritance rather than function naming alone.
- [ ] Prefer `Info` severity for optimization candidates; escalate to `Warning` for correctness risks like N+1 queries or unclosed sessions.
- [ ] Add one positive and one clean fixture for every scenario family before enabling any new rule by default.

## Fixtures And Verification

- [ ] Add `tests/fixtures/python/integration/advanceplan3/framework_positive.txt`.
- [ ] Add `tests/fixtures/python/integration/advanceplan3/framework_clean.txt`.
- [ ] Add `tests/fixtures/python/integration/advanceplan3/django_orm_positive.txt`.
- [ ] Add `tests/fixtures/python/integration/advanceplan3/django_orm_clean.txt`.
- [ ] Add `tests/fixtures/python/integration/advanceplan3/flask_positive.txt`.
- [ ] Add `tests/fixtures/python/integration/advanceplan3/flask_clean.txt`.
- [ ] Add `tests/fixtures/python/integration/advanceplan3/sqlalchemy_positive.txt`.
- [ ] Add `tests/fixtures/python/integration/advanceplan3/sqlalchemy_clean.txt`.
- [ ] Add `tests/integration_scan/python/advanceplan3.rs` coverage for the framework family.
- [ ] Verify with `cargo test python_advanceplan3` and the full `cargo test --test integration_scan` suite.

## Acceptance Criteria

- [ ] Each new rule explains whether the cost comes from query multiplicity, connection churn, response materialization, template recompilation, or request-scope allocation waste.
- [ ] Clean fixtures that already use `select_related()`, connection pooling, cached templates, or process-level clients stay quiet.
- [ ] No rule claims to prove missing indexes or wrong schema design; all messages remain heuristic and explainable.
- [ ] Startup code, migrations, management commands, and CLI tools do not trigger request-path findings.
- [ ] Rules are framework-gated: Django ORM rules require Django imports, Flask rules require Flask imports, etc.
