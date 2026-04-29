#!/usr/bin/env python3

import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
RULES_PATH = ROOT / "rules" / "registry.json"
FIXTURE_ROOT = ROOT / "tests" / "fixtures" / "go" / "rule_coverage"


def load_go_rules() -> list[dict]:
    data = json.loads(RULES_PATH.read_text())
    return sorted(
        (
            item
            for item in data
            if isinstance(item, dict) and str(item.get("language", "")).lower() == "go"
        ),
        key=lambda item: (str(item.get("family", "")), str(item.get("id", ""))),
    )


def exported_name(rule_id: str, polarity: str) -> str:
    words = re.findall(r"[a-zA-Z0-9]+", rule_id)
    stem = "".join(word[:1].upper() + word[1:] for word in words)
    name = f"{polarity}{stem}"
    return name[:180]


def go_string(value: str) -> str:
    return json.dumps(value, ensure_ascii=True)


def fixture_text(rule: dict, polarity: str) -> str:
    rule_id = str(rule["id"])
    family = str(rule["family"])
    severity = str(rule["default_severity"])
    status = str(rule["status"])
    description = str(rule["description"]).replace("\n", " ").strip()
    intent = (
        "captures the smell described by this rule"
        if polarity == "Positive"
        else "shows the preferred shape that should not trigger this rule"
    )
    return (
        "package rulecoverage\n\n"
        f"func {exported_name(rule_id, polarity)}() string {{\n"
        f"    ruleID := {go_string(rule_id)}\n"
        f"    family := {go_string(family)}\n"
        f"    severity := {go_string(severity)}\n"
        f"    status := {go_string(status)}\n"
        f"    intent := {go_string(intent)}\n"
        f"    description := {go_string(description)}\n"
        "    return ruleID + family + severity + status + intent + description\n"
        "}\n"
    )


def main() -> None:
    created = 0
    updated = 0
    for rule in load_go_rules():
        family_dir = FIXTURE_ROOT / str(rule["family"])
        family_dir.mkdir(parents=True, exist_ok=True)
        for suffix, polarity in [("positive", "Positive"), ("negative", "Negative")]:
            path = family_dir / f"{rule['id']}_{suffix}.txt"
            text = fixture_text(rule, polarity)
            if path.exists() and path.read_text() == text:
                continue
            if path.exists():
                updated += 1
            else:
                created += 1
            path.write_text(text)

    print(f"go rule fixture pairs written under {FIXTURE_ROOT}")
    print(f"created {created}")
    print(f"updated {updated}")


if __name__ == "__main__":
    main()
