#!/usr/bin/env python3
"""False-positive regression checker for chunk_analysis CSV outputs.

Usage:
  python scripts/fp_regression_check.py --baseline reports/chunk_analysis_all_baseline.csv --current reports/chunk_analysis_all.csv
"""

from __future__ import annotations

import argparse
import csv
import sys
from collections import Counter
from pathlib import Path


def load(csv_path: Path) -> tuple[int, Counter[str], int]:
    rows = 0
    fp_by_rule: Counter[str] = Counter()
    invalid = 0
    with csv_path.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        required = {
            "chunk_file_path",
            "subchunk_number",
            "source_file_path",
            "source_line",
            "rule_id",
            "decision",
            "rationale",
        }
        if not reader.fieldnames or set(reader.fieldnames) != required:
            raise ValueError(f"Unexpected header in {csv_path}: {reader.fieldnames}")

        for row in reader:
            rows += 1
            d = (row.get("decision") or "").strip()
            if d not in {"true_positive", "false_positive"}:
                invalid += 1
                continue
            if d == "false_positive":
                fp_by_rule[row["rule_id"].strip()] += 1
    return rows, fp_by_rule, invalid


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--baseline", type=Path, required=True)
    ap.add_argument("--current", type=Path, required=True)
    ap.add_argument("--max-global-fp-delta", type=int, default=0)
    ap.add_argument("--max-per-rule-fp-delta", type=int, default=0)
    args = ap.parse_args()

    b_rows, b_fp, b_invalid = load(args.baseline)
    c_rows, c_fp, c_invalid = load(args.current)

    failures: list[str] = []
    if b_invalid or c_invalid:
        failures.append(
            f"invalid decision rows baseline={b_invalid} current={c_invalid}"
        )

    b_total = sum(b_fp.values())
    c_total = sum(c_fp.values())
    global_delta = c_total - b_total
    if global_delta > args.max_global_fp_delta:
        failures.append(
            f"global FP delta {global_delta} exceeds {args.max_global_fp_delta}"
        )

    worst_rule = None
    worst_delta = -10**9
    all_rules = set(b_fp) | set(c_fp)
    for rule in all_rules:
        delta = c_fp.get(rule, 0) - b_fp.get(rule, 0)
        if delta > worst_delta:
            worst_delta = delta
            worst_rule = rule
        if delta > args.max_per_rule_fp_delta:
            failures.append(
                f"rule {rule} FP delta {delta} exceeds {args.max_per_rule_fp_delta}"
            )

    print("baseline_rows=", b_rows)
    print("current_rows=", c_rows)
    print("baseline_fp=", b_total)
    print("current_fp=", c_total)
    print("global_fp_delta=", global_delta)
    if worst_rule is not None:
        print("worst_rule_delta=", worst_rule, worst_delta)

    if failures:
        print("FAILED:")
        for msg in failures:
            print(" -", msg)
        return 1

    print("OK: no FP regression thresholds violated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
