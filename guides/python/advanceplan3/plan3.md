# Plan 3 - AI/ML Codebase Anti-Patterns, Data Pipeline Waste, And Bad Practices (Python)

Date: 2026-04-01

## Status

- [x] Initial slice not yet implemented.
- [x] This plan targets performance bottlenecks and bad practices specifically common in AI/ML codebases, data pipelines, and LLM application code that are currently outside the shipped Python heuristics.
- [x] The emphasis is on patterns that AI code generators frequently produce: naive data processing, unoptimized model inference paths, wasteful tensor operations, prompt construction anti-patterns, and common pandas/numpy misuse that is invisible to general-purpose linters.

## Already Covered And Excluded From This Plan

- [x] `full_dataset_load` (covers coarse whole-payload reads but not ML-specific data pipeline shapes)
- [x] `blocking_sync_io_in_async`
- [x] `unrelated_heavy_import`
- [x] `temporary_collection_in_loop`
- [x] `string_concat_in_loop`
- [x] `exception_swallowed`
- [x] `broad_exception_handler`
- [x] `mutable_default_argument`
- [x] `eval_exec_usage`
- [x] `unsafe_yaml_loader`
- [x] `pickle_deserialization_boundary`

## Objective

Add an AI/ML-aware performance and bad-practice pack that understands common pandas anti-patterns, numpy misuse, model inference waste, LLM/prompt engineering mistakes, data pipeline inefficiency, and ML experiment hygiene issues. These rules target patterns that are overwhelmingly produced by AI code generators and rarely caught by traditional Python linters.

## Candidate Scenario Backlog (44 scenarios)

### Pandas Performance Anti-Patterns

- [x] `pandas_iterrows_in_loop`: detect `df.iterrows()` usage in loops when vectorized operations, `.apply()`, or `.itertuples()` would be significantly faster.
- [x] `pandas_apply_with_simple_vectorizable_op`: detect `.apply(lambda x: x + 1)`, `.apply(lambda x: x.lower())`, or similar simple operations that have direct vectorized equivalents.
- [x] `pandas_concat_in_loop`: detect `pd.concat([...])` or `df.append(...)` inside loops instead of collecting all frames and concatenating once.
- [x] `pandas_copy_in_loop`: detect `df.copy()` inside loops on the same unchanged source DataFrame.
- [x] `pandas_chain_assignment_warning`: detect chained indexing like `df['a']['b'] = value` which may produce `SettingWithCopyWarning` and silently fail.
- [x] `pandas_read_csv_without_dtypes`: detect `pd.read_csv(...)` without `dtype` parameter on large files, causing pandas to infer types by reading the entire file twice.
- [x] `pandas_merge_without_validation`: detect `pd.merge(...)` without `validate` parameter on critical joins, risking silent row duplication on unexpected many-to-many.
- [x] `pandas_inplace_false_reassignment_missing`: detect `.drop(columns=[...], inplace=False)` or `.rename(inplace=False)` where the result is not assigned, silently discarding the operation.
- [x] `pandas_eval_string_manipulation`: detect `df.eval('column + 1')` or `df.query(f'column == {var}')` with f-string injection instead of parameterized operations.
- [x] `pandas_to_dict_records_in_loop`: detect `df.to_dict('records')` inside loops when row-by-row access through `.itertuples()` or vectorized operations would be faster.
- [x] `pandas_full_dataframe_print_in_production`: detect `print(df)`, `display(df)`, or `df.to_string()` on potentially large DataFrames in non-notebook production code.

### NumPy Performance Anti-Patterns

- [x] `numpy_python_loop_over_array`: detect `for x in numpy_array:` or `for i in range(len(array)):` style Python loops over NumPy arrays when vectorized operations would be vastly faster.
- [x] `numpy_append_in_loop`: detect `np.append(arr, value)` inside loops instead of pre-allocating with `np.zeros(n)` or `np.empty(n)` and filling.
- [x] `numpy_vstack_hstack_in_loop`: detect `np.vstack(...)`, `np.hstack(...)`, or `np.concatenate(...)` inside loops instead of collecting and stacking once.
- [x] `numpy_tolist_in_hot_path`: detect `.tolist()` on large arrays in hot paths, which copies the entire array into Python objects.
- [x] `numpy_dtype_mismatch_implicit_cast`: detect operations between arrays of different dtypes (e.g., float64 and float32) in tight loops causing implicit upcasting.

### Model Inference And ML Pipeline Waste

- [x] `model_loaded_per_request`: detect `torch.load(...)`, `tf.keras.models.load_model(...)`, `joblib.load(...)`, `pickle.load(...)` for models, `AutoModel.from_pretrained(...)`, or `pipeline(...)` instantiation inside request handlers, loops, or frequently-called functions.
- [x] `tokenizer_loaded_per_request`: detect `AutoTokenizer.from_pretrained(...)` or similar tokenizer loading inside request handlers or loops instead of startup-time initialization.
- [x] `model_eval_mode_missing`: detect `model(input)` or `model.forward(input)` calls without a preceding `model.eval()` or within a `torch.no_grad()` context in inference code, wasting memory on gradient computation.
- [x] `torch_no_grad_missing_in_inference`: detect model inference code that does not use `torch.no_grad()` or `torch.inference_mode()` context manager, causing unnecessary gradient tracking.
- [x] `model_to_device_in_loop`: detect `model.to(device)` or `tensor.to(device)` inside loops instead of moving once before the loop.
- [x] `training_loop_without_zero_grad`: detect `optimizer.step()` without a preceding `optimizer.zero_grad()` in training loops, causing gradient accumulation bugs.
- [x] `dataset_not_using_dataloader`: detect manual batching of datasets with list slicing inside training loops when `torch.utils.data.DataLoader` would handle batching, shuffling, and prefetching.
- [x] `embedding_computed_per_request`: detect embedding computation (`model.encode(...)`, `openai.Embedding.create(...)`) on the same static text inside request handlers instead of pre-computing and caching.

### LLM And Prompt Engineering Anti-Patterns

- [x] `llm_api_call_in_loop_without_batching`: detect OpenAI, Anthropic, or similar LLM API calls inside loops without batching, running up cost and latency.
- [x] `prompt_template_string_concat_in_loop`: detect prompt construction using string concatenation (`+`, `+=`, or `.format(...)`) inside loops when a template should be built once.
- [x] `llm_response_not_cached_same_input`: detect identical LLM API calls (same prompt/parameters) being made multiple times in one function without caching.
- [x] `llm_full_response_loaded_into_memory`: detect `response.choices[0].message.content` patterns that load full streaming responses into memory instead of using streaming iterators for large outputs.
- [x] `embedding_dimension_mismatch_silent`: detect vector operations (cosine similarity, dot product) between embeddings of different dimensions or sources without explicit dimension validation.
- [x] `token_count_not_checked_before_api_call`: detect LLM API calls that send prompts without any preceding token counting or length validation, risking context window overflow.
- [x] `hardcoded_api_key_in_source`: detect OpenAI, Anthropic, or other LLM API keys hardcoded as string literals instead of environment variables or secret management.
- [x] `retry_on_rate_limit_without_backoff`: detect retry logic around LLM API calls that does not implement exponential backoff or respect `Retry-After` headers.

### Data Pipeline And ETL Bad Practices

- [x] `pandas_read_without_chunksize_large_file`: detect `pd.read_csv(...)`, `pd.read_json(...)`, or `pd.read_parquet(...)` on large files without `chunksize` or `nrows` parameter in data pipeline code.
- [x] `entire_dataframe_copied_for_transform`: detect `df_copy = df.copy()` followed by in-place operations that could have been done on a view or with method chaining.
- [x] `no_schema_validation_on_external_data`: detect data ingestion from external sources (API responses, file reads) without schema validation using `pydantic`, `marshmallow`, `cerberus`, or explicit type/null checks.
- [x] `global_state_in_data_pipeline`: detect shared mutable global state (module-level lists, dicts, or counters) being modified inside data processing functions, causing non-deterministic behavior in parallel execution.
- [x] `data_pipeline_no_error_handling`: detect data processing functions that read from external sources and perform transformations without any exception handling, causing entire pipeline failures on single-record errors.
- [x] `intermediate_dataframe_not_freed`: detect large intermediate DataFrames that are assigned but never `del`'d or reassigned before subsequent memory-heavy operations in linear pipeline code.

### ML Experiment And Reproducibility Hygiene

- [x] `random_seed_not_set`: detect training or evaluation code that uses random operations (`random`, `numpy.random`, `torch.manual_seed`) without setting seeds at the beginning, making experiments non-reproducible.
- [x] `wandb_mlflow_log_in_tight_loop`: detect `wandb.log(...)`, `mlflow.log_metric(...)`, or equivalent experiment tracking calls inside inner training loops without step-based batching.
- [x] `gpu_memory_not_cleared_between_experiments`: detect model training or inference code that does not call `torch.cuda.empty_cache()` or equivalent between experiments or large model switches.
- [x] `print_metrics_instead_of_logging`: detect `print(f"accuracy: {acc}")` style metric reporting in training code instead of using proper logging or experiment tracking frameworks.

## Shared Implementation Checklist

- [x] Add import-aware classification for `pandas` (`pd`), `numpy` (`np`), `torch`, `tensorflow` (`tf`), `sklearn`, `transformers`, `openai`, `anthropic`, `langchain`, `wandb`, `mlflow`, `joblib`, `pydantic`, `httpx`, `aiohttp`.
- [x] Add ML context detection: distinguish training code (imports `torch.optim`, uses `model.train()`, has loss computation) from inference code (uses `model.eval()`, `torch.no_grad()`, serves predictions).
- [x] Add data pipeline detection: recognize ETL patterns through imports (`pandas`, `polars`, file I/O patterns) and function naming conventions (`process_*`, `transform_*`, `load_*`, `extract_*`).
- [x] Add LLM API detection: recognize common LLM client patterns (`openai.ChatCompletion`, `anthropic.Client`, `langchain.LLM`).
- [x] Gate ML-specific rules on ML framework imports so non-ML codebases stay quiet.
- [x] Gate pandas rules on `pandas` or `pd` imports.
- [x] Prefer `Info` severity for optimization candidates; escalate to `Warning` for correctness risks like missing `zero_grad()` or unvalidated schemas.
- [x] Add one positive and one clean fixture for every scenario family before enabling any new rule by default.

## Fixtures And Verification

- [x] Add `tests/fixtures/python/integration/advanceplan3/ml_positive.txt`.
- [x] Add `tests/fixtures/python/integration/advanceplan3/ml_clean.txt`.
- [x] Add `tests/fixtures/python/integration/advanceplan3/pandas_positive.txt`.
- [x] Add `tests/fixtures/python/integration/advanceplan3/pandas_clean.txt`.
- [x] Add `tests/fixtures/python/integration/advanceplan3/llm_positive.txt`.
- [x] Add `tests/fixtures/python/integration/advanceplan3/llm_clean.txt`.
- [x] Add `tests/fixtures/python/integration/advanceplan3/data_pipeline_positive.txt`.
- [x] Add `tests/fixtures/python/integration/advanceplan3/data_pipeline_clean.txt`.
- [x] Add `tests/integration_scan/python/advanceplan3.rs` coverage for the ML/AI family.
- [x] Verify with `cargo test python_advanceplan3` and the full `cargo test --test integration_scan` suite.

## Acceptance Criteria

- [x] Each new rule explains whether the cost comes from unnecessary computation, memory waste, API cost amplification, data pipeline inefficiency, or reproducibility risk.
- [x] Clean fixtures that use vectorized pandas operations, pre-loaded models, cached embeddings, batched API calls, and proper experiment tracking stay quiet.
- [x] No rule claims to prove optimal model architecture or hyperparameter choices; all messages remain about obvious code-level waste.
- [x] Notebook-style exploratory code and small-data prototyping scripts are not penalized for patterns that only matter at scale.
- [x] Rules are framework-gated: pandas rules require pandas imports, torch rules require torch imports, etc.
