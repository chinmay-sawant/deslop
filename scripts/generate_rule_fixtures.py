#!/usr/bin/env python3

"""Generate rule fixture files from rules/registry.json.

For each rule matching the specified language, creates:
  - <rule_id>_positive.txt (empty file)
  - <rule_id>_negative.txt (empty file)
  - <rule_id>.json (rule metadata)

Usage:
  python generate_rule_fixtures.py --language go
  python generate_rule_fixtures.py --language python
  python generate_rule_fixtures.py --language rust
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RULES_PATH = ROOT / "rules" / "registry.json"


def load_rules(language: str) -> list[dict]:
    data = json.loads(RULES_PATH.read_text())
    return [
        item for item in data
        if isinstance(item, dict) and str(item.get("language", "")).lower() == language.lower()
    ]


def generate_fixtures(language: str) -> None:
    fixture_dir = ROOT / "tests" / "fixtures" / language / "rules_fixtures"
    fixture_dir.mkdir(parents=True, exist_ok=True)

    rules = load_rules(language)
    created = 0

    for rule in rules:
        rule_id = rule["id"]
        rule_dir = fixture_dir / rule_id
        rule_dir.mkdir(parents=True, exist_ok=True)

        for polarity in ["positive", "negative"]:
            txt_path = rule_dir / f"{rule_id}_{polarity}.txt"
            if not txt_path.exists():
                txt_path.write_text("")
                created += 1

        json_path = rule_dir / f"{rule_id}.json"
        if not json_path.exists():
            json_path.write_text(json.dumps(rule, indent=2) + "\n")
            created += 1

    print(f"Fixture directory: {fixture_dir}")
    print(f"Rules processed: {len(rules)}")
    print(f"Files created: {created}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate rule fixture files from registry.json")
    parser.add_argument("--language", required=True, help="Language to filter rules (e.g., go, python, rust)")
    args = parser.parse_args()

    generate_fixtures(args.language)


if __name__ == "__main__":
    main()
