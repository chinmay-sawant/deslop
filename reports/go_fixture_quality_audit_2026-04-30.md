# Go Fixture Quality Audit - 2026-04-30

## Scope

- Audited the first 200 sorted Go rule-coverage positive/negative fixtures using 4 parallel GPT-5.4 subagents, in 50-file slices.
- The first 200 files were all under `tests/fixtures/go/rule_coverage/architecture`.
- High-confidence result: the architecture family contains widespread generic scaffold fixtures whose function bodies do not concretely express the named rule behavior.

## Architecture Findings

- Batch 1 findings: 19 confirmed mismatches.
- Batch 2 findings: 23 confirmed mismatches.
- Batch 3 findings: 20 confirmed mismatches.
- Batch 4 findings: 18 confirmed mismatches.
- Total confirmed mismatches in the first 200 sorted Go rule-coverage fixtures: 80.

Common pattern:

- Positive fixtures often contain a generic handler or service scaffold plus token maps, but no concrete rule-specific scenario.
- Some negative fixtures still directly perform the prohibited behavior named by the rule.
- This looks systemic rather than incidental and likely needs broader regeneration or deliberate rule-by-rule rewrites for the architecture family.

## Concrete Repairs Applied

Fixed these high-confidence performance false positives so the positive file now actually contains the named behavior:

- `tests/fixtures/go/rule_coverage/performance/base64_decode_string_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/hex_decode_string_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/json_marshalindent_in_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/json_valid_then_unmarshal_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/time_newticker_inside_loop_positive.txt`
- `tests/fixtures/go/rule_coverage/performance/time_newtimer_inside_loop_positive.txt`

## Validation

Executed and passed:

- `cargo test go_rule_fixture_batch_400_499_is_parseable_scenario_code`
- `cargo test go_rule_fixture_batch_600_699_is_parseable_scenario_code`

## Recommended Next Slice

- Continue auditing the Go `performance` family in 50-file batches.
- Defer large-scale architecture rewrites until deciding whether to regenerate or hand-author that family.
