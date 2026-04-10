# gopdfsuit Line-by-Line Validation Summary

- Input file: `/tmp/temp_gopdfsuit_after_fp_fixes3.txt`
- Findings processed: **1854**
- Findings with readable source path: **1854/1854**

## Binary False-Positive Result

- `false_positive = true`: **0**
- `false_positive = false`: **1854**

## Verdict Totals

- `FALSE_POSITIVE`: **0**
- `NOT_FALSE_POSITIVE`: **349**
- `NEEDS_MANUAL_REVIEW`: **1505**

## Top False-Positive Rules

- none

## Top Not-False-Positive Rules

- `slice_grow_without_cap_hint`: 192
- `error_detail_leaked_to_client`: 34
- `weak_crypto`: 16
- `weak_hash_for_integrity`: 15
- `handler_switches_on_error_strings`: 13
- `raw_db_error_exposed_to_client`: 13
- `python_public_api_any_contract`: 10
- `three_index_slice_for_append_safety`: 9
- `panic_on_error`: 7
- `error_wrapping_misuse`: 6
- `fmt_errorf_without_wrap_verb`: 6
- `type_assertion_without_comma_ok`: 5
- `shared_slice_append_race`: 4
- `map_lookup_double_access`: 4
- `struct_field_exposed_in_json`: 4
- `filepath_join_with_user_path`: 2
- `temp_file_predictable_name`: 1
- `debug_endpoint_in_production`: 1
- `missing_rate_limiting_on_auth_endpoint`: 1
- `gin_context_passed_beyond_request_boundary`: 1

## Top Needs-Manual-Review Rules

- `slice_append_without_prealloc_known_bound`: 178
- `fmt_hot_path`: 99
- `binary_read_for_single_field`: 48
- `full_dataset_load`: 44
- `test_imports_private_production_module`: 39
- `single_feature_requires_edits_in_many_unrelated_modules_due_to_scattered_policy`: 38
- `regexp_compile_in_hot_path`: 32
- `happy_path_only_test`: 30
- `public_api_missing_type_hints`: 28
- `builder_write_string_single_byte_literal`: 28
- `map_growth_without_size_hint`: 28
- `invariant_template_or_prefix_string_reformatted_inside_loop`: 27
- `sprintf_for_simple_string_format`: 27
- `weak_typing`: 26
- `error_logged_and_returned`: 25
- `batchable_writes_executed_one_at_a_time`: 22
- `copy_of_mapping_created_only_to_read_values`: 22
- `http_response_body_not_closed`: 19
- `bytes_buffer_without_grow_known_bound`: 18
- `filter_then_count_then_iterate`: 18
