## Rule Merge Contract

- [ ] Updated rule metadata in `rules/registry.json` when rule inventory changed
- [ ] Added one positive fixture for each new or materially changed detector
- [ ] Added one negative fixture for each new or materially changed detector
- [ ] Added suppression or config coverage when rollout or suppression behavior changed
- [ ] Added a mixed-language regression when parser or index behavior changed
- [ ] Ran `cargo test --locked`
- [ ] Ran `python3 scripts/sync_docs.py --check`
- [ ] Captured corpus evidence or documented why real-repo validation is deferred
- [ ] Added or updated promotion notes under `reports/corpus/<target>/promotion-notes.md` before moving a rule to `stable`

## Summary

Describe the user-visible change and the rule families or parser areas affected.

## Corpus Notes

Link the comparison report or promotion note that informed rollout decisions.
