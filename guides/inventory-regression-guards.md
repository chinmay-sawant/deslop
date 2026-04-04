# Inventory Regression Guards

The inventory-count assertions in the Rust test suite are intentional maintenance guards.
They should only change when the underlying inventories change on purpose.

## Source Rule IDs

The source rule-id guard lives in `src/rules.rs` as `EXPECTED_SOURCE_RULE_ID_COUNT`.

When it fails:
1. Review the diff under `src/` and confirm that rule-id additions or removals are intentional.
2. Re-run `cargo test --quiet` to make sure the failure is only the expected inventory delta.
3. Update `EXPECTED_SOURCE_RULE_ID_COUNT` in `src/rules.rs` in the same change that adds or removes the rule ids.

## Corpus Regression Entries

The corpus entry guard lives in `tests/parser_corpus_regression.rs` as `EXPECTED_CORPUS_ENTRY_COUNT`.

When it fails:
1. Review the added or removed `.txt` fixtures under `corpus/regressions/`.
2. Confirm the new fixture belongs in the regression inventory.
3. Update `EXPECTED_CORPUS_ENTRY_COUNT` in `tests/parser_corpus_regression.rs` in the same change.

## Why This Exists

These counts are meant to fail closed.
They make silent coverage shrinkage obvious and force inventory changes to be reviewed explicitly.
