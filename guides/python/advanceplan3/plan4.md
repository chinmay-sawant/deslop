# Plan 4 - Implementation Roadmap, Parser Work, And Validation (Python)

Date: 2026-04-01

## Status

- [ ] Initial implementation not yet started.
- [ ] This file is the execution roadmap for Python `advanceplan3` and ties the scenario backlog to the current Rust parser and heuristics architecture.

## Backlog Summary

- [ ] `advanceplan3/plan1.md` defines 38 generic hot-path, allocation, and computation waste scenarios.
- [ ] `advanceplan3/plan2.md` defines 42 Flask, Django, FastAPI, and SQLAlchemy framework scenarios.
- [ ] `advanceplan3/plan3.md` defines 44 AI/ML codebase, data pipeline, and LLM application scenarios.
- [ ] Total backlog in `advanceplan3`: **124 candidate scenarios**.

## Currently Shipped Python Rules (75 rules)

### Phase 1-3 Baseline (6 rules)

- `string_concat_in_loop`
- `blocking_sync_io_in_async`
- `exception_swallowed`
- `eval_exec_usage`
- `print_debugging_leftover`
- `full_dataset_load`

### Phase 4 Performance (9 rules)

- `list_materialization_first_element`
- `deque_candidate_queue`
- `temporary_collection_in_loop`
- `recursive_traversal_risk`
- `list_membership_in_loop`
- `repeated_len_in_loop`
- `builtin_reduction_candidate`
- `none_comparison`
- `mixed_sync_async_module`

### Phase 4 Maintainability (13 rules)

- `side_effect_comprehension`
- `redundant_return_none`
- `hardcoded_path_string`
- `hardcoded_business_rule`
- `magic_value_branching`
- `reinvented_utility`
- `variadic_public_api`
- `broad_exception_handler`
- `missing_context_manager`
- `public_api_missing_type_hints`
- `commented_out_code`
- `network_boundary_without_timeout`
- `environment_boundary_without_fallback`
- `external_input_without_validation`

### Phase 4 Structure (11 rules)

- `god_function`
- `god_class`
- `monolithic_init_module`
- `monolithic_module`
- `too_many_instance_attributes`
- `eager_constructor_collaborators`
- `over_abstracted_wrapper`
- `mixed_concerns_function`
- `name_responsibility_mismatch`
- `deep_inheritance_hierarchy`
- `tight_module_coupling`

### Phase 4 AI-Smells (5 rules)

- `textbook_docstring_small_helper`
- `mixed_naming_conventions`
- `unrelated_heavy_import`
- `obvious_commentary`
- `enthusiastic_commentary`

### Phase 4 Duplication (8 rules)

- `repeated_string_literal`
- `duplicate_error_handler_block`
- `duplicate_validation_pipeline`
- `duplicate_test_utility_logic`
- `cross_file_copy_paste_function`
- `duplicate_transformation_pipeline`
- `cross_file_repeated_literal`
- `duplicate_query_fragment`

### Advanceplan2 Async And Task Lifecycle (4 rules)

- `untracked_asyncio_task`
- `background_task_exception_unobserved`
- `async_lock_held_across_await`
- `async_retry_sleep_without_backoff`

### Advanceplan2 Mutable Defaults And Model Contracts (6 rules)

- `mutable_default_argument`
- `dataclass_mutable_default`
- `dataclass_heavy_post_init`
- `option_bag_model`
- `public_any_type_leak`
- `typeddict_unchecked_access`

### Advanceplan2 Import-Time Side Effects (6 rules)

- `import_time_network_call`
- `import_time_file_io`
- `import_time_subprocess`
- `module_singleton_client_side_effect`
- `mutable_module_global_state`
- `import_time_config_load`

### Advanceplan2 Boundary Safety (5 rules)

- `unsafe_yaml_loader`
- `pickle_deserialization_boundary`
- `subprocess_shell_true`
- `tar_extractall_unfiltered`
- `tempfile_without_cleanup`

### Shared Rules (2+ rules from common heuristics)

- `hardcoded_secret`
- `generic_name`
- `overlong_name`

## Shared Parser And Evidence Work

### Parser Extensions Required For Plan 1 (Core Hot-Path)

- [ ] Add loop-body call-site grouping keyed by callee module, function name, and stable argument bindings so "same input parsed twice" rules work without raw body-text matching.
- [ ] Add repeated-callee detection for `re.compile`, `json.loads`, `json.dumps`, `yaml.safe_load`, `ET.fromstring`, `datetime.strptime`, `urlparse`, `hashlib.new`, and `codecs.lookup`.
- [ ] Add allocation-shape summaries for list/dict/set construction inside loops: `list(source)`, `dict(source)`, `source.copy()`, `source[:]`, `set(source)`, `{**source}`.
- [ ] Add import-aware alias resolution for `re`, `json`, `yaml`, `xml.etree.ElementTree`, `csv`, `gzip`, `pickle`, `hashlib`, `datetime`, `urllib.parse`, `codecs`, `pathlib`, and `collections`.
- [ ] Add materialization-shape detection for `sorted(...)[0]`, `len([comprehension])`, and `list(dict.keys())` patterns.

### Parser Extensions Required For Plan 2 (Web Frameworks)

- [ ] Add decorator-based handler detection for Flask (`@app.route`, `@bp.route`), Django (`@api_view`, `@action`, class-based views inheriting `View`, `APIView`, `ViewSet`, `ModelViewSet`), and FastAPI (`@router.get`, `@router.post`, `@app.get`).
- [ ] Add Django ORM call-chain summaries: detect chained queryset operations and track presence/absence of `select_related`, `prefetch_related`, `only`, `defer`, `values`, ordering, and limiting clauses.
- [ ] Add SQLAlchemy session lifecycle tracking: detect `Session()` creation, `session.query(...)`, `session.execute(...)`, `session.commit()`, `session.close()`, and relationship loading strategy calls.
- [ ] Add Flask/FastAPI request-body access tracking: detect multiple `request.get_json()`, `request.form`, `request.data` accesses in the same handler.
- [ ] Add middleware/hook detection for Flask (`@app.before_request`, `@app.after_request`), Django (`MIDDLEWARE` class methods), and FastAPI (`@app.middleware`).

### Parser Extensions Required For Plan 3 (AI/ML)

- [ ] Add pandas call-chain summaries: detect `df.iterrows()`, `df.apply(...)`, `pd.concat(...)`, `df.copy()`, `df.merge(...)`, `df.to_dict(...)`, `pd.read_csv(...)` and track context (inside loop vs. top-level).
- [ ] Add NumPy operation classification: detect `np.append(...)`, `np.vstack(...)`, `np.hstack(...)`, `np.concatenate(...)` inside loops vs. one-shot operations.
- [ ] Add ML model lifecycle detection: detect `torch.load(...)`, `tf.keras.models.load_model(...)`, `AutoModel.from_pretrained(...)`, `model.eval()`, `model.train()`, `torch.no_grad()`, `optimizer.zero_grad()`, `optimizer.step()` and track their relative positions.
- [ ] Add LLM API call detection: detect `openai.ChatCompletion.create(...)`, `openai.chat.completions.create(...)`, `anthropic.Client().messages.create(...)`, `langchain` chain invocations, and track whether they occur inside loops.
- [ ] Add experiment tracking detection: detect `wandb.log(...)`, `mlflow.log_metric(...)`, `mlflow.log_param(...)` and determine if they are inside inner training loops vs. epoch-level logging.

## Heuristic Wave Plan

### Wave 1: Core Hot-Path (Plan 1, highest confidence)

Ship the highest-confidence generic hot-path rules that only require function-local evidence:

- [ ] `regex_compile_in_hot_path`
- [ ] `json_loads_same_payload_multiple_times`
- [ ] `repeated_json_dumps_same_object`
- [ ] `sorted_only_for_first_element`
- [ ] `list_comprehension_only_for_length`
- [ ] `readlines_then_iterate`
- [ ] `read_then_splitlines`
- [ ] `in_check_on_list_literal`
- [ ] `string_startswith_endswith_chain`
- [ ] `enumerate_on_range_len`
- [ ] `csv_writer_flush_per_row`
- [ ] `write_without_buffering_in_loop`
- [ ] `repeated_open_same_file_in_function`
- [ ] `dict_items_or_keys_materialized_in_loop`

Target: 14 rules with function-local evidence only.

### Wave 2: Extended Hot-Path And Allocation (Plan 1, medium confidence)

Ship allocation churn and algorithmic waste rules that need loop-body analysis:

- [ ] `yaml_load_same_payload_multiple_times`
- [ ] `xml_parse_same_payload_multiple_times`
- [ ] `repeated_datetime_strptime_same_format`
- [ ] `urlparse_in_loop_on_invariant_base`
- [ ] `path_resolve_or_expanduser_in_loop`
- [ ] `repeated_hashlib_new_same_algorithm`
- [ ] `list_copy_in_loop_same_source`
- [ ] `dict_copy_in_loop_same_source`
- [ ] `set_created_per_iteration_same_elements`
- [ ] `string_join_without_generator`
- [ ] `nested_list_search_map_candidate`
- [ ] `sort_then_first_or_membership_only`
- [ ] `filter_then_count_then_iterate`
- [ ] `repeated_list_index_lookup`
- [ ] `append_then_sort_each_iteration`
- [ ] `repeated_dict_get_same_key_no_cache`

Target: 16 rules with loop-body and invariant-binding evidence.

### Wave 3: Web Framework Rules (Plan 2, framework-gated)

Ship framework-aware rules that require decorator and import evidence:

- [ ] `django_queryset_evaluated_multiple_times`
- [ ] `django_n_plus_one_no_select_related`
- [ ] `django_n_plus_one_no_prefetch_related`
- [ ] `django_queryset_count_then_exists`
- [ ] `django_queryset_len_instead_of_count`
- [ ] `django_all_without_limit_in_view`
- [ ] `django_save_full_model_in_loop`
- [ ] `django_create_single_in_loop`
- [ ] `flask_request_body_parsed_multiple_times`
- [ ] `flask_global_db_connection_per_request`
- [ ] `flask_template_rendered_from_string_in_view`
- [ ] `fastapi_sync_def_with_blocking_io`
- [ ] `fastapi_dependency_creates_client_per_request`
- [ ] `sqlalchemy_session_not_closed`
- [ ] `sqlalchemy_query_in_loop`
- [ ] `sqlalchemy_n_plus_one_lazy_load`
- [ ] `sqlalchemy_commit_per_row_in_loop`
- [ ] `sqlalchemy_create_engine_per_request`
- [ ] `middleware_creates_http_client_per_request`
- [ ] `middleware_compiles_regex_per_request`
- [ ] `upstream_http_call_per_item_in_handler`
- [ ] `upstream_call_without_timeout_in_handler`

Target: 22 rules with framework import and decorator gating.

### Wave 4: AI/ML Codebase Rules (Plan 3, framework-gated)

Ship ML-specific rules that require ML framework imports:

- [ ] `pandas_iterrows_in_loop`
- [ ] `pandas_apply_with_simple_vectorizable_op`
- [ ] `pandas_concat_in_loop`
- [ ] `pandas_read_csv_without_dtypes`
- [ ] `numpy_python_loop_over_array`
- [ ] `numpy_append_in_loop`
- [ ] `numpy_vstack_hstack_in_loop`
- [ ] `model_loaded_per_request`
- [ ] `tokenizer_loaded_per_request`
- [ ] `model_eval_mode_missing`
- [ ] `torch_no_grad_missing_in_inference`
- [ ] `training_loop_without_zero_grad`
- [ ] `llm_api_call_in_loop_without_batching`
- [ ] `prompt_template_string_concat_in_loop`
- [ ] `hardcoded_api_key_in_source`
- [ ] `retry_on_rate_limit_without_backoff`
- [ ] `random_seed_not_set`
- [ ] `embedding_computed_per_request`

Target: 18 rules with ML/AI framework import gating.

### Wave 5: Extended Framework And Data Pipeline (Plans 2 + 3, remaining)

Ship the remaining framework and data pipeline rules after Waves 1-4 have settled:

- [ ] Remaining Django ORM rules from Plan 2
- [ ] Remaining Flask/FastAPI rules from Plan 2
- [ ] Remaining pandas/numpy rules from Plan 3
- [ ] Data pipeline hygiene rules from Plan 3
- [ ] ML experiment tracking rules from Plan 3
- [ ] LLM-specific response and caching rules from Plan 3

Target: remaining ~54 scenarios after Waves 1-4.

## False-Positive Controls

- [ ] Gate handler-only findings on decorator evidence (`@app.route`, `@api_view`, `@router.get`) or class-based view inheritance rather than function naming alone.
- [ ] Gate Django ORM findings on `django` imports; Flask findings on `flask` imports; FastAPI findings on `fastapi` imports.
- [ ] Gate pandas findings on `pandas` or `pd` imports; numpy findings on `numpy` or `np` imports.
- [ ] Gate ML findings on `torch`, `tensorflow`, `sklearn`, `transformers`, `openai`, or `anthropic` imports.
- [ ] Keep startup-only setup code, management commands, and migration scripts quiet unless the expensive operation clearly appears inside request or loop paths.
- [ ] Default "candidate optimization" rules to `Info` and escalate to `Warning` only when at least two supporting signals agree.
- [ ] Suppress batch findings when `bulk_create`, `bulk_update`, batched API calls, or explicit chunking helpers are already present.
- [ ] Avoid raw method-name matching when import resolution cannot disambiguate framework-specific symbols from unrelated libraries.
- [ ] Keep messages explicit about uncertainty when the rule infers hot-path intent from naming, handler shape, or repeated-loop evidence rather than a guaranteed profiler trace.
- [ ] Notebook files (`.ipynb` rendered as `.py`) and small exploratory scripts should not trigger scale-dependent rules.
- [ ] Test files, conftest fixtures, and test utilities should remain suppressed for request-path and production-only rules.

## Real-Repo Validation Strategy

- [ ] Validate Plan 1 (core hot-path) rules against at least one generic Python CLI tool and one Python web service.
- [ ] Validate Plan 2 (framework) rules against at least one real Django project and one real Flask project.
- [ ] Validate Plan 3 (AI/ML) rules against at least one real ML training codebase and one LLM application.
- [ ] Track false-positive rates per rule family during validation and disable rules that exceed a 20% false-positive threshold without additional parser evidence.
- [ ] Re-run representative external validation before promoting any family from `Info` to `Warning` severity.

## Priority Order

1. First ship the 14 highest-confidence core hot-path rules from Plan 1 Wave 1 that need only local call and loop evidence.
2. Next ship the 16 extended hot-path rules from Plan 1 Wave 2 with loop-body invariant analysis.
3. Then ship the 22 framework-gated web rules from Plan 2 Wave 3 because they have clearer anchors through decorator and import evidence.
4. Then ship the 18 AI/ML rules from Plan 3 Wave 4 which require ML framework import gating.
5. Finally ship the remaining ~54 scenarios across Plans 2 and 3 in Wave 5 after the earlier waves have settled.
6. Hold speculative rules until at least two real-repo examples justify the signal and clean fixtures prove the false-positive controls.

## Acceptance Criteria

- [ ] Every shipped rule points at a concrete expensive site and explains why it matters in throughput, allocation, API cost, or correctness terms.
- [ ] Every rule family has clean fixtures that demonstrate the intended escape hatch or optimized pattern.
- [ ] The new parser summaries do not materially regress end-to-end Python scan time.
- [ ] Real repositories with startup code, migrations, tests, management commands, and CLI tools do not light up with broad false positives.
- [ ] Framework-specific rules stay silent on codebases that do not import the relevant framework.
- [ ] ML/AI rules stay silent on non-ML codebases.

## Non-Goals

- [ ] Do not claim exact Big-O proofs, precise query plans, or index truth without schema/runtime evidence.
- [ ] Do not turn `advanceplan3` into a generic style pack; keep the focus on measurable or plausibly measurable performance waste and dangerous bad practices.
- [ ] Do not replace specialized tooling: django-debug-toolbar for query analysis, cProfile/py-spy for profiling, mypy for type checking, or bandit for security auditing.
- [ ] Do not couple the first implementation wave to full type inference, data-flow analysis, or inter-procedural call-graph resolution unless a rule demonstrably needs it.
- [ ] Do not penalize idiomatic Python patterns that are only slow at extreme scale with no local evidence of scale.
- [ ] Do not claim to detect all N+1 query variants; only flag the patterns with clear queryset-in-loop evidence.
