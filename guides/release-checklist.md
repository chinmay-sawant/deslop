# Release Checklist

This checklist is the shared release gate for deslop. Use it before cutting a release and before marking a release PR ready to merge.

## Required Commands

1. Run the release-related validation scripts from `scripts/`:

```bash
python3 scripts/corpus_harness.py validate
```

2. Run the full test suite:

```bash
make test
```

No tests should be failing.

3. Regenerate the frontend and README content:

```bash
python3 scripts/sync_docs.py
```

Review and include the generated changes before releasing.

4. Run the security and hygiene checks:

```bash
bash scripts/check-rust-security.sh
bash scripts/check_rust_hygiene.sh
```

## Exit Criteria

- All commands above exit successfully.
- `make test` completes with zero failing tests.
- Generated changes from `python3 scripts/sync_docs.py` are reviewed and included.
- `reports/rust-security-baseline/latest.txt` is reviewed if `check-rust-security.sh` reports new matches.
- Do not cut the release or merge the release PR until every item above is complete.
