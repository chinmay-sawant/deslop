**Wave 1 Improvement Plan (420 FP Files, 6x70 Batches)**

1. **Primary root cause to fix first**
- Most false positives are from over-broad lexical matching in [`/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/performance_layers.rs`](/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/performance_layers.rs).
- Rules fire from comments/generic tokens (`for`, `map`, `body`, `cache`, `retry`) without semantic anchors.
- This affects many `go_perf_layer_*` rules across `generator.go`, `helpers.go`, `image.go`, `links.go`, `merge/annotations.go`.

2. **Priority code-change workstreams (Rust)**
- **P0: Perf-layer matcher hardening**
  - Update [`/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/performance_layers.rs`](/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/performance_layers.rs)
  - Strip comments/string literals before token matching.
  - Raise `min_group_matches` for noisy categories (DB/network/caching/GC/io).
  - Require category anchors:
    - DB rules: `database/sql`, `gorm`, query/scan call evidence
    - Network rules: `http.Client`/`Do`/`Get`/`RoundTrip`/`net` evidence
    - Scanner rules: explicit `bufio.Scanner`
    - Response-body rules: real `http.Response` flow, not variable named `body`
- **P1: Hot-path and repeated-work precision**
  - Update [`/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/go/framework_patterns/hot_path/repeated_work/repeated_work_primary.rs`](/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/go/framework_patterns/hot_path/repeated_work/repeated_work_primary.rs)
  - Update [`/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/go/framework_patterns/hot_path/repeated_work/repeated_work_secondary.rs`](/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/go/framework_patterns/hot_path/repeated_work/repeated_work_secondary.rs)
  - `filter_then_count_then_iterate`: require repeated passes over same collection symbol.
  - `strconv_repeat_on_same_binding`: require same binding reused, not just repeated `Atoi`.
- **P1: Go idioms/security context guards**
  - Update [`/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/go/idioms.rs`](/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/go/idioms.rs)
  - `http_response_body_not_closed`: only when actual HTTP response source exists.
  - `mutable_package_global`: suppress static map-literal tables never mutated.
  - Update weak-hash context in:
    - [`/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/security.rs`](/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/security.rs)
    - [`/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/go/library_misuse/security/crypto.rs`](/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/go/library_misuse/security/crypto.rs)
  - Keep rule, but require security-sensitive context for strong findings.
- **P2: Python structural false positives**
  - Update [`/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/python/structure.rs`](/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/python/structure.rs)
  - Guard `sibling_modules_depend_on_private_helpers_from_each_other` for `_bindings` and tests.
  - Update `option_bag_model` logic in python quality module to avoid schema-only DTO false positives.
- **P2: Naming rule refinement**
  - Update [`/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/naming.rs`](/home/chinmay/ChinmayPersonalProjects/deslop/src/heuristics/naming.rs)
  - Allow domain-precise long names where clarity is high.

3. **Fixture update plan (required)**
- Add/extend negative fixtures under:
  - [`/home/chinmay/ChinmayPersonalProjects/deslop/tests/fixtures/go/rules_fixtures`](/home/chinmay/ChinmayPersonalProjects/deslop/tests/fixtures/go/rules_fixtures)
  - [`/home/chinmay/ChinmayPersonalProjects/deslop/tests/fixtures/python/rules_fixtures`](/home/chinmay/ChinmayPersonalProjects/deslop/tests/fixtures/python/rules_fixtures)
- Focus fixture snippets:
  - comment-only keyword noise should not trigger perf-layer rules
  - map/set/regex parsing helpers without DB/network/http/scanner should stay negative
  - `sync.Pool` type assertions should not trigger HTTP-body or unsafe-assertion rules
  - local dedupe maps should not trigger unbounded-cache rules
  - structured `fmt.Sprintf` for protocol/document emit should not trigger simple-concat smell
  - `_bindings` imports and DTO dataclasses should not trigger structure/option-bag smells
- Add targeted positives for each tightened rule to preserve recall.

4. **Execution sequencing for this wave**
1. Implement P0 matcher hardening first.
2. Update fixtures for top noisy `go_perf_layer_*` rules.
3. Re-run targeted tests for touched rules.
4. Implement P1/P2 guards.
5. Re-run and compare FP delta on the same 420-file wave.

5. **Batching strategy for remaining 2,557 FP files**
- Continue in waves of **6 subagents x 70 files = 420 per wave**.
- Expected remaining waves: **7** (6 full + 1 partial).
- Reuse this same output format each wave so we can apply incremental rule hardening safely.

If you want, next step I’ll start implementing **P0 + fixture updates** immediately for this first wave.


**Reducing False Positives: Progress Summary**

- Source report: `gopdfsuit_final_singular_report.html`
- Total report FPs: `2977`
- Work style: wave-based (`420` files/wave, split via subagents), patch rules + update fixture `.txt` files + rerun tests.

**What We Changed**

- Tightened perf-layer matching in:
  - `src/heuristics/performance_layers.rs`
- Reduced noisy Go rule behavior in:
  - `src/heuristics/go/performance.rs` (`fmt_hot_path`)
  - `src/heuristics/go/idioms.rs` (`http_response_body_not_closed`)
  - `src/heuristics/go/library_misuse/performance/error_api.rs` (`type_assertion_without_comma_ok`)
  - `src/heuristics/go/framework_patterns/hot_path/repeated_work/repeated_work_secondary.rs` (`filter_then_count_then_iterate`)
- Tightened Python FP behavior in:
  - `src/heuristics/python/structure.rs`
  - `src/heuristics/python/quality/module_state.rs`
- Updated corresponding fixture files under:
  - `tests/fixtures/go/rules_fixtures/*`
  - `tests/fixtures/python/rules_fixtures/*`

**Validation**

- `cargo test -q performance_layers -- --nocapture` passed
- `cargo test -q integration_scan::go::rule_fixture_coverage -- --nocapture` passed
- `cargo test -q integration_scan::python::rule_fixture_coverage -- --nocapture` passed

**Wave Metrics (measured against report FP pairs: `source + rule_id`)**

- Wave 2 (`420` files): `312 -> 229` (`-83`)
- Wave 3 (`420` files): `326 -> 217` (`-109`)
- Wave 4 (`420` files): `342 -> 248` (`-94`)

**Coverage Status vs Report**

- Processed so far: Waves `1..4` = `1680` FP files reviewed from the report list.
- Remaining: `2977 - 1680 = 1297` FP files (Waves `5..8`, final wave partial).
- Conclusion: not all report false positives are covered yet; coverage is partial and in progress.

**Next Steps**

1. Continue wave-by-wave on remaining `1297` files.
2. Prioritize top remaining noisy rules each wave.
3. Keep rule changes paired with fixture `.txt` updates and rerun coverage tests each iteration.

---

## Latest Iteration Summary (Appended)

### Current Status

- We are **not done yet** for the full `gopdfsuit_final_singular_report` FP list.
- Full report FPs: `2977`
- Waves processed so far: `1..6` (`2520` files)
- Remaining not yet fully iterated: `457` files (wave 7 + final partial wave)

### What Improved

Recent wave deltas after rule + fixture updates:
- Wave 2: `312 -> 229` (`-83`)
- Wave 3: `326 -> 217` (`-109`)
- Wave 4: `342 -> 248` (`-94`)
- Wave 5: `349 -> 267` (`-82`)
- Wave 6: `349 -> 310` (`-39`)

This confirms material reduction, but more FP cleanup is still pending.

### Key Rule Tightening Snippets

`src/heuristics/go/performance.rs` (`fmt_hot_path` tighter gating):

```rust
let hot_signal = ["handler", "serve", "http", "api", "request", "process", "render"]
    .iter()
    .any(|marker| name_lc.contains(marker) || path_lc.contains(marker));
if go.fmt_loops.is_empty() || (!hot_signal && go.fmt_loops.len() < 3) {
    return Vec::new();
}
```

`src/heuristics/go/idioms.rs` (`http_response_body_not_closed` avoids `.Get(` false matches like `sync.Pool.Get`):

```rust
binding_matches(lines, &string_patterns)
    .into_iter()
    .filter(|(_, _, target)| {
        target.contains("http.")
            || target.contains("net/http")
            || target.contains("client.Do(")
            || target == ".Do("
    })
    .collect()
```

`src/heuristics/go/framework_patterns/hot_path/repeated_work/repeated_work_secondary.rs`
(`filter_then_count_then_iterate` now requires repeated ranges over same target):

```rust
let repeated_target = !t1.is_empty() && (t1 == t2 || t1 == t3 || t2 == t3);
if !repeated_target {
    return false;
}
```

`src/heuristics/performance_layers.rs` (rule-specific guards added):

```rust
if rule.rule_id == "go_perf_layer_data_structure_choice_interface_map_for_typed_values"
    && !body_lc.contains("_ = 1")
    && !(body_lc.contains("map[string]any")
        || body_lc.contains("map[string]interface{}")
        || body_lc.contains("interface{}"))
{
    return false;
}
```

### Fixture Discipline (kept in sync)

For each updated rule, matching `tests/fixtures/*/rules_fixtures/*_positive.txt` and `*_negative.txt` files were updated and then validated with:

- `cargo test -q performance_layers -- --nocapture`
- `cargo test -q integration_scan::go::rule_fixture_coverage -- --nocapture`
- `cargo test -q integration_scan::python::rule_fixture_coverage -- --nocapture`

### Remaining Work

- Continue wave-by-wave for the remaining FP files.
- Prioritize remaining top noisy IDs in wave 6+ (mix of Go + Python perf-layer style rules).
- Keep same process: patch rules -> update fixtures -> rerun coverage tests -> measure wave delta.

## Remaining Waves Update (Completed Run)

Processed the remaining report batches:
- `wave7_420.txt` (420 files)
- `wave8_tail.txt` (37 files)

Latest measured deltas (against same baseline pairing logic: `source + rule_id`):
- Wave 7: `338 -> 247` (`-91`)
- Wave 8 tail: `31 -> 20` (`-11`)

Overall report progress:
- Total report pairs parsed: `2977`
- Baseline detected pairs: `2380`
- Current detected pairs: `1663`
- Net reduction so far: `-717`

Additional rules adjusted in this pass:
- Go: `full_dataset_load` guard updates in `src/heuristics/go/performance.rs`
- Python: `public_api_missing_type_hints` path-scope guards in `src/heuristics/python/maintainability/function_rules.rs`

Additional fixture `.txt` updates in this pass:
- `tests/fixtures/go/rules_fixtures/full_dataset_load/full_dataset_load_negative.txt`
- `tests/fixtures/python/rules_fixtures/public_api_missing_type_hints/public_api_missing_type_hints_negative.txt`

Validation after these changes:
- `cargo test -q integration_scan::go::rule_fixture_coverage -- --nocapture` passed
- `cargo test -q integration_scan::python::rule_fixture_coverage -- --nocapture` passed

Status:
- Remaining false-positive cleanup is still pending (`1663` currently still detected from the report set), but all waves have now been processed and measured.

## May 21, 2026 Multi-Subagent Wave Update

### What We Changed (This Wave)

Used all 6 subagents in parallel and applied rule + fixture updates across Go and Python:

- Go rules tightened:
  - `fmt_hot_path`
  - `http_response_body_not_closed`
  - `type_assertion_without_comma_ok`
  - `filter_then_count_then_iterate`
  - `go_perf_layer_async_concurrency_context_timeout_allocated_per_inner_call`
  - `go_perf_layer_memory_allocation_append_without_known_capacity`
  - `go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline`
  - `go_perf_layer_memory_allocation_map_recreated_for_static_lookup`
  - `go_perf_layer_string_handling_strings_join_single_element_loop`

- Python rules tightened:
  - `python_perf_layer_error_handling_cost_error_message_formatted_on_success_path`
  - `python_perf_layer_error_handling_cost_raise_from_none_hides_retriable_error_context`
  - `python_perf_layer_memory_allocation_list_append_without_generator_stream`
  - `python_perf_layer_string_handling_lowercase_compare_allocates`
  - `python_perf_layer_collection_iteration_generator_materialized_for_truthiness`
  - `python_perf_layer_io_operations_temporary_file_for_bytes_transform`
  - `python_perf_layer_memory_allocation_deepcopy_before_readonly_transform`
  - `python_perf_layer_collection_iteration_enumerate_list_materialized`
  - `python_perf_layer_network_calls_tls_context_built_per_request`
  - `full_response_or_export_buffered_before_incremental_consumer_use`

- Maintainability/test-quality tightening:
  - `print_debugging_leftover`
  - `broad_exception_handler`
  - `redundant_return_none`
  - `happy_path_only_test`

### Code Snippets

`src/heuristics/performance_layers.rs` (benchmark/sampledata suppression for non-profiling rules):

```rust
if should_skip_benchmark_context(file, function_name, body_lc)
    && !rule.category.eq_ignore_ascii_case("profiling_benchmarking")
{
    return false;
}
```

`src/heuristics/go/performance_extra.rs` (timeout-in-loop rule now requires retry semantics):

```rust
if line.in_loop
    && body_lc.contains("context.withtimeout")
    && (body_lc.contains("retry") || body_lc.contains("attempt") || body_lc.contains("backoff"))
{
    // emit finding
}
```

### Fixture Discipline

Updated matching `tests/fixtures/go/rules_fixtures/*` and `tests/fixtures/python/rules_fixtures/*` negatives (and selected positives) for each touched rule so the new behavior is locked in.

### Validation

- `cargo test -q performance_layers -- --nocapture` passed (`609 passed`)
- `cargo test -q integration_scan::go::rule_fixture_coverage -- --nocapture` passed
- `cargo test -q integration_scan::python::rule_fixture_coverage -- --nocapture` passed

### Status Answer (Done vs Pending)

- We are **not done yet** with full false-positive cleanup from `gopdfsuit_final_singular_report`.
- This wave is integrated and validated, but final remaining FP count must be recomputed with the same report-pair evaluator used for the earlier `2380 -> 1663` metric line.
- So yes, more false-positive fixes are still pending until that re-evaluation is rerun and remaining hotspots are iterated down.

## FP Revalidation + New Fix Plan (May 21, 2026)

### Revalidation Result (Phase-2 QC)

Performed a full QC pass over all `1401` phase-2-labeled FP findings (6 subagents, full coverage).

- Input FP bucket: `1401`
- `CONFIRMED_FP`: `1106`
- `MISSED_TP` (wrongly marked FP): `295`

Corrected adjudication totals after QC flip:
- `TP`: `1525`
- `FP`: `1106`
- `REVIEW_REQUIRED`: `0`

Artifacts:
- `/tmp/gopdfsuit_retriage_2631/final_adjudication_all_phase2_qc.csv`
- `/tmp/gopdfsuit_retriage_2631/final_adjudication_all_with_text_paths_phase2_qc.csv`

### Key Insight

The FP rate is still high, but lower than phase-2 arbitration output suggested.
Largest remaining false-positive pressure is concentrated in a narrow rule set, mostly performance/hot-path heuristics.

Top remaining FP-heavy rules (post-QC confirmed):
1. `test_imports_private_production_module` (`39`)
2. `go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need` (`38`)
3. `go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record` (`33`)
4. `go_perf_layer_serialization_gzip_writer_created_per_small_payload` (`32`)
5. `go_perf_layer_error_handling_cost_error_string_built_before_error_needed` (`28`)
6. `python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records` (`26`)
7. `python_perf_layer_caching_unbounded_dict_cache` (`25`)
8. `go_perf_layer_collection_iteration_len_called_after_materializing_channel` (`25`)

### New Improvement Plan (Like Earlier, Wave-Based)

Scope to fix now: `1106` confirmed FPs.

Batching:
- 6 subagents x ~`184` files per wave (`~1104` total), plus tail wave for remainder.

Execution waves:
- Wave A (priority rules 1-4)
- Wave B (priority rules 5-8)
- Wave C (next top 8 noisy rules)
- Wave D+ (tail long-tail rules + verification)

For each wave (strict discipline):
1. Patch detection logic in Rust for target rules.
2. Add/update rule fixtures:
   - `tests/fixtures/go/rules_fixtures/<rule>/*_negative.txt`
   - `tests/fixtures/go/rules_fixtures/<rule>/*_positive.txt`
   - `tests/fixtures/python/rules_fixtures/<rule>/*_negative.txt`
   - `tests/fixtures/python/rules_fixtures/<rule>/*_positive.txt`
3. Run validation:
   - `cargo test -q performance_layers -- --nocapture`
   - `cargo test -q integration_scan::go::rule_fixture_coverage -- --nocapture`
   - `cargo test -q integration_scan::python::rule_fixture_coverage -- --nocapture`
4. Re-score wave FP delta against QC-corrected baseline.

### Detection Tightening Themes (Code Changes)

- Add stronger context requirements for perf rules in non-benchmark production code.
- Require cost-amplifier evidence (loop multiplicity, request-path markers, repeated pattern density).
- Suppress style-only patterns when no runtime cost signal exists.
- Expand precise sample/demo/test path guards where rule intent is production hot path.
- Keep profiling/benchmark category rules exempt from broad suppression.

### Target Files for First Wave

- `src/heuristics/performance_layers.rs`
- `src/heuristics/go/performance.rs`
- `src/heuristics/go/performance_extra.rs`
- `src/heuristics/go/framework_patterns/hot_path/repeated_work/*.rs`
- `src/heuristics/python/performance.rs`
- `src/heuristics/python/maintainability/function_rules.rs` (only where noisy overlaps remain)

### Immediate Goal

Reduce confirmed FP from `1106` to below `700` in next two waves while preserving TP recall (no significant TP drop in validated fixtures).
