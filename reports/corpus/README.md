# Corpus Reports

This directory stores repeatable real-repository evaluation artifacts for deslop.

Expected layout:

- `promotion-note-template.md`: template used for per-target false-positive and promotion notes
- `<target>/latest-scan.txt`: latest captured scan output from `scripts/corpus_harness.py`
- `<target>/latest-bench.json`: latest captured benchmark JSON from `scripts/corpus_harness.py`
- `<target>/comparisons/`: timestamped Markdown finding-diff reports
- `<target>/promotion-notes.md`: reviewer notes used before severity promotion

The authoritative target list lives in `corpus/manifest.json`.
