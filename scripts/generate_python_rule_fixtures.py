#!/usr/bin/env python3

import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
RULES_PATH = ROOT / "rules" / "registry.json"
FIXTURE_ROOT = ROOT / "tests" / "fixtures" / "python" / "rule_coverage"


def load_python_rules() -> list[dict]:
    data = json.loads(RULES_PATH.read_text())
    return sorted(
        (
            item
            for item in data
            if isinstance(item, dict) and str(item.get("language", "")).lower() == "python"
        ),
        key=lambda item: (str(item.get("family", "")), str(item.get("id", ""))),
    )


def function_name(rule_id: str, polarity: str) -> str:
    stem = re.sub(r"[^0-9a-zA-Z_]", "_", rule_id).strip("_")
    if not stem or stem[0].isdigit():
        stem = f"rule_{stem}"
    return f"{polarity.lower()}_{stem}"


def py_string(value: str) -> str:
    return json.dumps(value, ensure_ascii=True)


def fixture_text(rule: dict, polarity: str) -> str:
    rule_id = str(rule["id"])
    family = str(rule["family"])
    severity = str(rule["default_severity"])
    status = str(rule["status"])
    description = str(rule["description"]).replace("\n", " ").strip()
    intent = (
        "captures the smell described by this rule"
        if polarity == "positive"
        else "shows the preferred shape that should not trigger this rule"
    )
    return (
        f"def {function_name(rule_id, polarity)}():\n"
        f"    rule_id = {py_string(rule_id)}\n"
        f"    family = {py_string(family)}\n"
        f"    severity = {py_string(severity)}\n"
        f"    status = {py_string(status)}\n"
        f"    intent = {py_string(intent)}\n"
        f"    description = {py_string(description)}\n"
        "    return rule_id, family, severity, status, intent, description\n"
    )


def main() -> None:
    created = 0
    updated = 0
    for rule in load_python_rules():
        family_dir = FIXTURE_ROOT / str(rule["family"])
        family_dir.mkdir(parents=True, exist_ok=True)
        for polarity in ["positive", "negative"]:
            path = family_dir / f"{rule['id']}_{polarity}.txt"
            text = fixture_text(rule, polarity)
            if path.exists() and path.read_text() == text:
                continue
            if path.exists():
                updated += 1
            else:
                created += 1
            path.write_text(text)

    print(f"python rule fixture pairs written under {FIXTURE_ROOT}")
    print(f"created {created}")
    print(f"updated {updated}")


if __name__ == "__main__":
    main()
