# Plan 1 - Core Hot-Path And Allocation Backlog (Go)

Date: 2026-03-31

## Status

- [ ] Drafted only; not implemented yet.
- [ ] This plan is intentionally scoped to performance patterns that are not already covered by the current Go heuristics.
- [ ] The target set is biased toward signals that usually sit outside common `golangci-lint` bundles and style linters because they require loop-shape, binding-reuse, or hot-path reasoning.

## Already Covered And Excluded From This Plan

- [x] `allocation_churn_in_loop`
- [x] `fmt_hot_path`
- [x] `reflection_hot_path`
- [x] `string_concat_in_loop`
- [x] `repeated_json_marshaling`
- [x] `likely_n_squared_allocation`
- [x] `likely_n_squared_string_concat`
- [x] `full_dataset_load`
- [x] The coarse DB-query family already exists separately and is expanded in `advanceplan3/plan2.md`.

## Objective

Build the next generic Go performance pack around hot-path allocation shape, repeated parsing work, duplicate materialization, and algorithmic waste that looks plausible in generated code but normally escapes mainstream linting. The emphasis is on conservative, explainable findings that can be supported by parser evidence already close to what `ParsedFunction` exposes today.

## Candidate Scenario Backlog (37 scenarios)

### Capacity And Growth Planning

- [ ] `slice_append_without_prealloc_known_bound`: detect `append` into nil or zero-capacity slices when the enclosing loop clearly ranges over a source with a visible length and no capacity hint is present.
- [ ] `nested_append_without_outer_capacity`: detect nested loops that build an output slice without preallocating even though both outer and inner bounds are visible locally.
- [ ] `map_growth_without_size_hint`: detect steady-state inserts into a map in hot paths without `make(map[K]V, hint)` when the candidate entry count is locally inferable.
- [ ] `strings_builder_without_grow_known_bound`: detect `strings.Builder` use that still skips `Grow` even when approximate output size is easy to infer.
- [ ] `bytes_buffer_without_grow_known_bound`: detect `bytes.Buffer` use that repeatedly expands without a visible `Grow` or initial capacity plan.
- [ ] `builder_or_buffer_recreated_per_iteration`: detect `strings.Builder`, `bytes.Buffer`, or `bytes.NewBuffer(nil)` being constructed once per item in a loop instead of being reset or reused.
- [ ] `make_slice_inside_hot_loop_same_shape`: detect short-lived scratch slices recreated on every iteration with the same shape and no sign that they must escape.
- [ ] `make_map_inside_hot_loop_same_shape`: detect temporary maps recreated in inner loops for normalization, deduplication, or metadata collection.
- [ ] `repeated_slice_clone_in_loop`: detect `append([]T(nil), src...)`, `slices.Clone`, or `copy`-based cloning inside loops where the cloned slice is used only transiently.
- [ ] `repeated_map_clone_in_loop`: detect whole-map copies inside loops where a clear read-only source map is being re-cloned for every iteration.
- [ ] `append_then_trim_each_iteration`: detect buffers or slices that grow and are immediately resliced back down in steady-state loops, which usually indicates a reusable scratch buffer candidate.
- [ ] `byte_string_conversion_in_loop`: detect `string([]byte)` or `[]byte(string)` conversions inside loops when the converted value is immediately used as a map key, join fragment, or short-lived lookup token.

### Repeated Parse, Compile, And Normalize Work

- [ ] `regexp_compile_in_hot_path`: detect `regexp.Compile` or `regexp.MustCompile` inside handlers, middleware, or obvious iterative paths.
- [ ] `template_parse_in_hot_path`: detect `html/template` or `text/template` parse calls inside request or export paths instead of startup-time caching.
- [ ] `url_parse_in_loop_on_invariant_base`: detect repeated `url.Parse` or reference-resolution work on the same invariant base value inside loops.
- [ ] `time_parse_layout_in_loop`: detect `time.Parse` or `ParseInLocation` in hot loops when the layout and input family are clearly repetitive.
- [ ] `json_unmarshal_same_payload_multiple_times`: detect the same local payload binding being unmarshaled into multiple targets in one function.
- [ ] `xml_unmarshal_same_payload_multiple_times`: detect duplicate XML decoding against the same local payload.
- [ ] `yaml_unmarshal_same_payload_multiple_times`: detect repeated YAML or TOML decoding on the same raw bytes or string binding.
- [ ] `proto_unmarshal_same_payload_multiple_times`: detect protobuf payloads that are decoded multiple times in one request path.
- [ ] `strings_split_same_input_multiple_times`: detect repeated `strings.Split`, `SplitN`, or `Fields` calls against the same unchanged binding.
- [ ] `bytes_split_same_input_multiple_times`: detect repeated `bytes.Split*` calls against the same unchanged byte slice.
- [ ] `stable_value_normalization_in_inner_loop`: detect repeated `strings.ToLower`, `TrimSpace`, `ReplaceAll`, `path.Clean`, or similar normalization calls on invariant values inside nested loops.
- [ ] `strconv_repeat_on_same_binding`: detect repeated `strconv` conversions on the same unchanged binding within a single function body.
- [ ] `uuid_hash_formatting_only_for_logs`: detect `uuid.String()`, `hex.EncodeToString`, or `base64` formatting inside tight loops when the formatted value only feeds logging or debug strings.

### Serialization, Compression, And Stream Shaping

- [ ] `json_encoder_recreated_per_item`: detect `json.NewEncoder` being created for each item or sub-loop on the same writer instead of reusing a stable encoder.
- [ ] `json_decoder_recreated_per_item`: detect `json.NewDecoder` being rebuilt repeatedly on short-lived readers inside inner loops.
- [ ] `gzip_reader_writer_recreated_per_item`: detect `gzip.NewReader` or `gzip.NewWriter` per element instead of per stream or pooled reuse.
- [ ] `csv_writer_flush_per_row`: detect `csv.Writer.Flush()` or equivalent buffer flushes inside per-row export loops.
- [ ] `bufio_writer_missing_in_bulk_export`: detect large write loops to files or sockets without a visible buffered writer.
- [ ] `bufio_reader_missing_for_small_read_loop`: detect repeated tiny reads from files or sockets in loops without `bufio.Reader` style buffering.
- [ ] `read_then_decode_duplicate_materialization`: detect `io.ReadAll` plus a second decode/materialization stage when a streaming decoder could serve the same path.

### Algorithmic Waste And Container Shape

- [ ] `slice_membership_in_loop_map_candidate`: detect `slices.Contains` or manual linear membership checks inside loops against a stable slice that could be indexed once.
- [ ] `nested_linear_join_map_candidate`: detect nested-loop lookup joins between two collections when one side is effectively being searched by key each time.
- [ ] `append_then_sort_each_iteration`: detect result slices that are re-sorted after each append instead of once at the end or through a bounded insertion strategy.
- [ ] `sort_before_first_or_membership_only`: detect full sorts when the code only uses the first element, min/max, or a yes/no membership outcome afterward.
- [ ] `filter_then_count_then_iterate`: detect repeated full traversals over the same slice or array for filter, count, and process phases inside one function.

## Shared Implementation Checklist

- [ ] Extend Go parser evidence so append targets, `make` capacity hints, builder writes, flush sites, and repeated decode targets can be summarized instead of re-derived from raw `body_text` each time.
- [ ] Add import-aware alias helpers for `strings`, `bytes`, `regexp`, `encoding/json`, `encoding/xml`, `compress/gzip`, `bufio`, `encoding/csv`, `strconv`, `net/url`, and `time`.
- [ ] Prefer `Info` severity for micro-optimization candidates and require multiple corroborating signals before escalating to `Warning`.
- [ ] Add one positive and one clean fixture for every scenario family before enabling any new rule by default.
- [ ] Benchmark against at least one generic CLI-style Go repository and one web-service repository to ensure parser enrichment stays cheap.

## Acceptance Criteria

- [ ] Every shipped rule anchors to a concrete line and points at the expensive operation, collection growth site, or repeated parse target.
- [ ] Clean fixtures for deliberate buffering, preallocation, or cached parsing stay quiet.
- [ ] The first wave remains function-local unless repo-aware correlation clearly improves precision.