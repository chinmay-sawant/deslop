# Inventory Regression Guards

The inventory-count assertions in the Rust test suite are intentional maintenance guards.
They should only change when the underlying inventories change on purpose.

## Source Rule IDs

The source rule-id guard lives in `src/rules.rs` as `EXPECTED_SOURCE_RULE_ID_COUNT`.

Current intentional source rule-id count:

- `455`

When it fails:
1. Review the diff under `src/` and confirm that rule-id additions or removals are intentional.
2. Re-run `cargo test --quiet` to make sure the failure is only the expected inventory delta.
3. Update `EXPECTED_SOURCE_RULE_ID_COUNT` in `src/rules.rs` in the same change that adds or removes the rule ids.

## Registry Counts By Language

The grouped registry-count guard lives in `src/rules.rs` as `EXPECTED_RULE_COUNTS_BY_LANGUAGE`.

Current intentional grouped counts:

- `common`: 11
- `go`: 633
- `python`: 212
- `rust`: 100

When it fails:
1. Review the added or removed rule definitions and confirm the language delta is intentional.
2. Re-run `cargo test --quiet` to make sure the failure is only the expected grouped inventory change.
3. Update `EXPECTED_RULE_COUNTS_BY_LANGUAGE` in `src/rules.rs` in the same change.

## Corpus Regression Entries

The corpus entry guard lives in `tests/parser_corpus_regression.rs` as `EXPECTED_CORPUS_ENTRY_COUNT`.

When it fails:
1. Review the added or removed `.txt` fixtures under `corpus/regressions/`.
2. Confirm the new fixture belongs in the regression inventory.
3. Update `EXPECTED_CORPUS_ENTRY_COUNT` in `tests/parser_corpus_regression.rs` in the same change.

## Why This Exists

These counts are meant to fail closed.
They make silent coverage shrinkage obvious and force inventory changes to be reviewed explicitly.
Where it helps, the guards also keep grouped breakdowns visible by language or corpus category so drift is easier to localize.

## Informational Scan Workflows

The makefile also exposes non-failing scan targets for review and exploration:

- `make scan-info`
- `make scan-gopdfsuit-info`
- `make scan-snapback-info`
- `make scan-claw-info`

Use these when findings are expected and you want to keep the output without turning the shell session red.
Keep the strict `scan`, `scan-gopdfsuit`, `scan-snapback`, and `scan-claw` targets for policy enforcement and CI-style checks.
