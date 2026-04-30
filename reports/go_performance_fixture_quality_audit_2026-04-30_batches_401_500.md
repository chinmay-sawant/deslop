# Go Performance Fixture Quality Audit - 2026-04-30 (Batch 401-500)

## Scope

- Audited the next 100 sorted Go performance rule-coverage fixtures, covering sorted indices 401-500.
- Audit execution used a GPT-5.4 subagent.
- Audit standard:
  - `_positive.txt` must clearly contain the named rule behavior.
  - `_negative.txt` must clearly avoid the named rule behavior while staying semantically close.
- This report now includes the first direct-fix pass applied after the audit.

## Confirmed Findings

- Confirmed mismatches across sorted performance fixtures 401-500: 39.
- All 39 confirmed mismatches are positive-side fixture defects.
- No high-confidence negative-side mismatches were reported in this 100-file window.

## High-Yield Direct Cases

The strongest immediately patchable files from this audit are concentrated in the string and time families:

- `tests/fixtures/go/rule_coverage/performance/sha1_sum_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/sha512_sum_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/sort_slice_vs_sort_sort_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_compare_not_equal_zero_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_hasprefix_then_trimprefix_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_hassuffix_then_trimsuffix_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_split_two_index_zero_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitafter_two_index_one_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitafter_two_index_zero_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitaftern_two_index_one_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitaftern_two_index_zero_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitn_two_index_one_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitn_two_index_zero_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_trimleft_space_trimspace_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_trimright_space_trimspace_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/time_fixedzone_per_call_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/time_loadlocation_per_call_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/time_since_candidate_via_now_sub_positive.txt`

## Repairs Applied In First Direct Pass

- `tests/fixtures/go/rule_coverage/performance/sha1_sum_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/sha512_sum_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/sort_slice_vs_sort_sort_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_compare_not_equal_zero_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_hasprefix_then_trimprefix_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_hassuffix_then_trimsuffix_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_split_two_index_zero_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitafter_two_index_one_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitafter_two_index_zero_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitaftern_two_index_one_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitaftern_two_index_zero_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitn_two_index_one_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_splitn_two_index_zero_cut_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_trimleft_space_trimspace_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/strings_trimright_space_trimspace_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/time_fixedzone_per_call_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/time_loadlocation_per_call_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/time_since_candidate_via_now_sub_positive.txt`

Total repaired so far in the 401-500 window: 18 files.

## Validation

Passed after the first direct-fix pass:

- `cargo test go_rule_fixture_batch_400_499_is_parseable_scenario_code`

## Queue Status

- Remaining confirmed mismatches from sorted performance fixtures 401-500: 21.
- The first direct-fix pass removed the highest-confidence checksum/string/time positives, leaving a smaller remainder of more varied positive-side defects.
- The still-unresolved remainder from the 201-400 audit remains the older queue and should stay ahead of any broader expansion beyond 500.
