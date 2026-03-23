# Python Benchmark Note

## Purpose

This note records the current Python rollout benchmark snapshot for deslop's first Python backend release. It is a fixture-scale sanity check, not a performance gate for large real-world repositories.

## Command

```bash
target/release/deslop bench --warmups 2 --repeats 5 <python-fixture-workspace>
```

The recorded local snapshot used a temporary workspace with these files:

- `app.py` from `tests/fixtures/python/simple.txt`
- `service.py` from `tests/fixtures/python/rule_pack_positive.txt`

## Current Snapshot

- root: temporary Python-only fixture workspace
- warmups: `2`
- repeats: `5`
- discovered files: `2`
- analyzed files: `2`
- functions: `3`
- findings: `11`
- parse failures: `0`
- total ms: min=`0` max=`0` mean=`0.00` median=`0.00`
- parse ms: min=`0` max=`0` mean=`0.00` median=`0.00`
- index ms: min=`0` max=`0` mean=`0.00` median=`0.00`
- heuristics ms: min=`0` max=`0` mean=`0.00` median=`0.00`

## Interpretation Notes

- This benchmark runs against tiny fixture files, so the measured timings are below millisecond resolution on the current machine.
- The useful part of this snapshot is the count stability: files, functions, findings, and parse failures.
- Treat this note as a regression snapshot for the Python fixture workspace, not as a statement about performance on large Python repositories.

## Refresh Policy

Re-record this note when Python parser evidence, Python heuristic routing, or mixed-language index behavior changes in a way that could affect counts or runtime.