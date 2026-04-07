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

5. Build the release crate artifact:

```bash
cargo build --release
```

6. Validate the crates.io package before upload:

```bash
cargo publish --dry-run
```

If the release edits are intentionally not committed yet, use `cargo publish --dry-run --allow-dirty` instead. A clean committed tree is preferred for the final publish.

7. Publish the crate to crates.io:

```bash
cargo publish
```

If the release edits are intentionally not committed yet, use `cargo publish --allow-dirty` instead. Make sure the version in `Cargo.toml` has been bumped first because crates.io will reject a duplicate version.

## Exit Criteria

- All commands above exit successfully.
- `make test` completes with zero failing tests.
- Generated changes from `python3 scripts/sync_docs.py` are reviewed and included.
- `reports/rust-security-baseline/latest.txt` is reviewed if `check-rust-security.sh` reports new matches.
- `cargo publish --dry-run` succeeds before the final upload.
- `cargo publish` completes successfully with a valid crates.io login or `CARGO_REGISTRY_TOKEN`.
- Do not cut the release or merge the release PR until every item above is complete.
