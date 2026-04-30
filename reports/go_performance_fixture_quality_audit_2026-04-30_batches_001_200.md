# Go Performance Fixture Quality Audit - 2026-04-30

## Scope

- Audited the first 200 sorted Go performance rule-coverage fixtures in 4 parallel 50-file batches using GPT-5.4 subagents.
- Audit standard:
  - `_positive.txt` must clearly contain the named rule behavior.
  - `_negative.txt` must clearly avoid the named rule behavior while staying semantically close.
- Focused repairs were limited to high-confidence direct false positives and false negatives.

## Confirmed Findings

- Batch 1 (files 1-50): 17 confirmed mismatches.
- Batch 2 (files 51-100): 23 confirmed mismatches.
- Batch 3 (files 101-150): 24 confirmed mismatches.
- Batch 4 (files 151-200): 23 confirmed mismatches.
- Total confirmed mismatches across the first 200 sorted Go performance fixtures: 87.

## Repairs Applied In This Pass

Previously repaired before this continuation:

- `tests/fixtures/go/rule_coverage/performance/base64_decode_string_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/hex_decode_string_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/json_marshalindent_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/json_valid_then_unmarshal_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/time_newticker_inside_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/time_newtimer_inside_loop_positive.txt`

Repaired in this continuation:

- `tests/fixtures/go/rule_coverage/performance/adler32_checksum_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/allocation_churn_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/binary_read_for_single_field_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/buffer_write_rune_ascii_literal_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/buffer_write_string_single_byte_literal_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bufio_scanner_small_buffer_for_large_lines_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/builder_write_rune_ascii_literal_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/builder_write_string_single_byte_literal_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_buffer_string_len_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_buffer_truncate_zero_reset_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_compare_not_equal_zero_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_index_any_not_minus_one_contains_any_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_newbufferstring_on_string_conversion_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_newreader_on_string_conversion_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_replace_neg_one_replaceall_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_split_two_index_zero_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_splitafter_two_index_one_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_splitafter_two_index_zero_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_splitaftern_two_index_one_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/bytes_splitn_two_index_one_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/clear_map_go121_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/context_withtimeout_inside_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/copy_append_idiom_waste_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/crc64_checksum_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/csv_reader_reuse_record_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/defer_in_tight_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/duration_nanoseconds_zero_check_positive.txt`

Total repaired so far within the performance family on 2026-04-30: 33 files.

## Validation

Passed after repairs:

- `cargo test go_rule_fixture_batch_400_499_is_parseable_scenario_code`
- `cargo test go_rule_fixture_batch_400_499_is_parseable_scenario_code` (rerun after second repair burst)
- `cargo test go_rule_fixture_batch_600_699_is_parseable_scenario_code`

## Remaining Work From This 200-File Audit

- 60 confirmed mismatches remain from the audited first 200 performance fixtures.
- The remaining confirmed cases are still mostly direct positive-file API mismatches and can be continued in the same batch-by-batch workflow.
- Next practical target is the rest of batch 2, then batches 3 and 4.
