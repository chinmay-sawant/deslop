# Plan 1 - Core Hot-Path, Allocation, And Computation Waste (Python)

Date: 2026-04-01

## Status

- [ ] Initial slice not yet implemented.
- [ ] This plan is intentionally scoped to performance patterns that are not already covered by the current Python heuristics.
- [ ] The target set is biased toward signals that usually sit outside common `pylint`, `flake8`, and `ruff` bundles because they require loop-shape, binding-reuse, hot-path, or allocation reasoning.

## Already Covered And Excluded From This Plan

- [x] `string_concat_in_loop`
- [x] `blocking_sync_io_in_async`
- [x] `full_dataset_load`
- [x] `list_materialization_first_element`
- [x] `deque_candidate_queue`
- [x] `temporary_collection_in_loop`
- [x] `recursive_traversal_risk`
- [x] `list_membership_in_loop`
- [x] `repeated_len_in_loop`
- [x] `builtin_reduction_candidate`
- [x] `missing_context_manager`
- [x] `mutable_default_argument`

## Objective

Build the next generic Python performance pack around hot-path allocation shape, repeated parsing or compilation work, duplicate materialization, serialization waste, and algorithmic inefficiency that looks plausible in AI-generated code but normally escapes mainstream linting. The emphasis is on conservative, explainable findings that can be supported by parser evidence already close to what `ParsedFunction` exposes today.

## Candidate Scenario Backlog (38 scenarios)

### Repeated Parse, Compile, And Normalize Work

- [ ] `regex_compile_in_hot_path`: detect `re.compile(...)` inside loops, request handlers, or repeatedly-called functions instead of module-level precompilation.
- [ ] `json_loads_same_payload_multiple_times`: detect the same local variable or parameter being passed to `json.loads(...)` or `json.load(...)` multiple times within one function.
- [ ] `yaml_load_same_payload_multiple_times`: detect repeated `yaml.safe_load(...)` or `yaml.load(...)` on the same unchanged binding in one function.
- [ ] `xml_parse_same_payload_multiple_times`: detect repeated `ET.fromstring(...)`, `ET.parse(...)`, or `minidom.parseString(...)` on the same binding.
- [ ] `repeated_json_dumps_same_object`: detect `json.dumps(obj)` called multiple times on the same unchanged object in one function body.
- [ ] `repeated_datetime_strptime_same_format`: detect `datetime.strptime(...)` with the same format string inside loops or repeated calls instead of caching the compiled format.
- [ ] `repeated_string_format_invariant_template`: detect `.format(...)` or f-string construction on invariant template parts inside loops when the template itself does not change per iteration.
- [ ] `urlparse_in_loop_on_invariant_base`: detect `urllib.parse.urlparse(...)` or `urlsplit(...)` on the same invariant base URL inside loops.
- [ ] `path_resolve_or_expanduser_in_loop`: detect `Path.resolve()`, `Path.expanduser()`, or `os.path.abspath(...)` on invariant path values inside loops.
- [ ] `repeated_hashlib_new_same_algorithm`: detect `hashlib.new(algo)` or `hashlib.sha256()` being reconstructed per iteration instead of using `.copy()` or updating a single digest.
- [ ] `repeated_locale_or_codec_lookup_in_loop`: detect `codecs.lookup(...)`, `locale.getlocale()`, or similar locale/codec resolution inside loops.

### Serialization, Compression, And I/O Shaping

- [ ] `json_encoder_recreated_per_item`: detect `json.JSONEncoder(...)` or `json.JSONDecoder(...)` being instantiated per loop iteration instead of reusing one encoder.
- [ ] `csv_writer_flush_per_row`: detect `csv.writer(f).writerow(...)` or explicit flush calls inside per-row loops instead of batched writes.
- [ ] `gzip_open_per_chunk`: detect `gzip.open(...)` or `gzip.GzipFile(...)` being created repeatedly inside loops for small chunks instead of one streaming writer.
- [ ] `pickle_dumps_in_loop_same_structure`: detect `pickle.dumps(...)` inside loops when the structure shape is clearly repetitive and a single serialization pass would suffice.
- [ ] `write_without_buffering_in_loop`: detect unbuffered `file.write(...)` calls inside tight loops without a `BufferedWriter` or batched accumulation.
- [ ] `readlines_then_iterate`: detect `file.readlines()` followed by iteration when direct `for line in file:` would avoid full materialization.
- [ ] `read_then_splitlines`: detect `file.read().splitlines()` or `.split('\n')` when line-by-line iteration would be more memory-efficient.
- [ ] `repeated_open_same_file_in_function`: detect multiple `open(same_path, ...)` calls within one function body instead of reading once and reusing.

### Allocation Churn And Container Waste

- [ ] `list_comprehension_only_for_length`: detect `len([x for x in iterable if condition])` when `sum(1 for x in iterable if condition)` would avoid allocating the full list.
- [ ] `sorted_only_for_first_element`: detect `sorted(collection)[0]` or `sorted(collection)[-1]` when `min(...)` or `max(...)` would be O(n) instead of O(n log n).
- [ ] `list_copy_in_loop_same_source`: detect `list(source)`, `source[:]`, or `source.copy()` being called per iteration on the same unchanged source inside loops.
- [ ] `dict_copy_in_loop_same_source`: detect `dict(source)`, `source.copy()`, or `{**source}` being called per iteration on the same unchanged mapping inside loops.
- [ ] `set_created_per_iteration_same_elements`: detect `set(...)` construction on the same invariant iterable inside loops.
- [ ] `tuple_unpacking_in_tight_loop`: detect repeated tuple creation and unpacking inside inner loops when the structure is invariant.
- [ ] `string_join_without_generator`: detect `"".join([list comprehension])` when `"".join(generator)` would avoid intermediate list allocation.
- [ ] `dict_items_or_keys_materialized_in_loop`: detect `list(d.keys())`, `list(d.values())`, or `list(d.items())` inside loops when iteration over the view would suffice.
- [ ] `enumerate_on_range_len`: detect `for i, x in enumerate(range(len(collection)))` or equivalent anti-patterns that could use direct `enumerate(collection)`.

### Algorithmic Waste And Quadratic Patterns

- [ ] `nested_list_search_map_candidate`: detect nested-loop lookup joins between two collections when one side is effectively being searched by key each time.
- [ ] `sort_then_first_or_membership_only`: detect full `sorted(...)` or `.sort()` calls when the code only uses the first element, min/max, or a yes/no membership outcome afterward.
- [ ] `filter_then_count_then_iterate`: detect repeated full traversals over the same iterable for filter, count, and process phases inside one function.
- [ ] `repeated_list_index_lookup`: detect `list.index(value)` inside loops when building a reverse-lookup dictionary would be more efficient.
- [ ] `repeated_isinstance_chain_same_object`: detect long `isinstance(obj, Type1) ... isinstance(obj, Type2) ...` chains that should use a single `isinstance(obj, (Type1, Type2, ...))` call or a dispatch dict.
- [ ] `string_startswith_endswith_chain`: detect chains of `.startswith(a) or .startswith(b) or .startswith(c)` when `.startswith((a, b, c))` would be cleaner and faster.
- [ ] `in_check_on_list_literal`: detect `x in [a, b, c, ...]` membership checks on list literals when a set literal `{a, b, c, ...}` would be O(1).
- [ ] `append_then_sort_each_iteration`: detect result lists that are re-sorted after each append instead of once at the end or through `heapq` or `bisect.insort`.
- [ ] `repeated_dict_get_same_key_no_cache`: detect `d.get(key)` or `d[key]` called multiple times for the same key in one function when assigning to a local variable would avoid repeated hash lookups.
- [ ] `concatenation_in_comprehension_body`: detect string concatenation (`+`) inside list/dict/set comprehension bodies that build repeated intermediate strings.

## Shared Implementation Checklist

- [ ] Extend Python parser evidence so loop body call sites, repeated callee targets, invariant binding analysis, and materialization shapes can be summarized instead of re-derived from raw `body_text` each time.
- [ ] Add import-aware alias helpers for `re`, `json`, `yaml`, `xml.etree`, `csv`, `gzip`, `pickle`, `hashlib`, `datetime`, `urllib.parse`, `codecs`, and `pathlib`.
- [ ] Prefer `Info` severity for micro-optimization candidates and require multiple corroborating signals before escalating to `Warning`.
- [ ] Add one positive and one clean fixture for every scenario family before enabling any new rule by default.

## Fixtures And Verification

- [ ] Add `tests/fixtures/python/integration/advanceplan3/core_positive.txt`.
- [ ] Add `tests/fixtures/python/integration/advanceplan3/core_clean.txt`.
- [ ] Add `tests/integration_scan/python/advanceplan3.rs` coverage for the core hot-path family.
- [ ] Verify with `cargo test python_advanceplan3` and the full `cargo test --test integration_scan` suite.

## Acceptance Criteria

- [ ] Every shipped rule anchors to a concrete line and points at the expensive operation, repeated parse target, or allocation site.
- [ ] Clean fixtures for deliberate caching, precompilation, or streaming stay quiet.
- [ ] The first wave remains function-local unless repo-aware correlation clearly improves precision.
- [ ] No rule claims a measured regression; messages stay at the level of likely hot-path risk or avoidable allocation pattern.
