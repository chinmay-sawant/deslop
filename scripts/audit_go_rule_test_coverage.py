#!/usr/bin/env python3

import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
RULES_PATH = ROOT / "rules" / "registry.json"
TESTS_ROOT = ROOT / "tests"
GO_RULE_FIXTURE_ROOT = TESTS_ROOT / "fixtures" / "go" / "rule_coverage"


def load_go_rules() -> list[str]:
    data = json.loads(RULES_PATH.read_text())
    items = data if isinstance(data, list) else data.get("rules", data)
    return sorted(
        {
            item["id"]
            for item in items
            if isinstance(item, dict) and str(item.get("language", "")).lower() == "go"
        }
    )


def collect_array_constants(text: str, valid_ids: set[str]) -> dict[str, set[str]]:
    constants: dict[str, set[str]] = {}
    for name, body in re.findall(r"const\s+(\w+)\s*:\s*&\[&str\]\s*=\s*&\[(.*?)\];", text, re.S):
        constants[name] = {rule_id for rule_id in re.findall(r'"([^"]+)"', body) if rule_id in valid_ids}
    return constants


def update_from_assert_calls(
    text: str,
    constants: dict[str, set[str]],
    valid_ids: set[str],
    positive: set[str],
    negative: set[str],
) -> None:
    for body in re.findall(r"assert_rules_present\s*\(.*?&\[(.*?)\]\s*\)", text, re.S):
        positive.update(rule_id for rule_id in re.findall(r'"([^"]+)"', body) if rule_id in valid_ids)
    for body in re.findall(r"assert_rules_absent\s*\(.*?&\[(.*?)\]\s*\)", text, re.S):
        negative.update(rule_id for rule_id in re.findall(r'"([^"]+)"', body) if rule_id in valid_ids)

    for name, rule_ids in constants.items():
        if re.search(rf"assert_rules_present\s*\(.*?&\s*{name}\b", text, re.S):
            positive.update(rule_ids)
        if re.search(rf"assert_rules_absent\s*\(.*?&\s*{name}\b", text, re.S):
            negative.update(rule_ids)

    for rule_id in re.findall(r'assert_go_perf_layer_pair\s*\(\s*"([^"]+)"', text):
        if rule_id in valid_ids:
            positive.add(rule_id)
            negative.add(rule_id)


def update_from_direct_asserts(text: str, valid_ids: set[str], positive: set[str], negative: set[str]) -> None:
    for rule_id in re.findall(r'finding\.rule_id\s*==\s*"([^"]+)"', text):
        if rule_id in valid_ids:
            positive.add(rule_id)
    for rule_id in re.findall(r'!\s*(?:has_rule|report_has_rule)\([^\n]*"([^"]+)"', text):
        if rule_id in valid_ids:
            negative.add(rule_id)
    for rule_id in re.findall(r'(?<![!\w])(?:has_rule|report_has_rule)\([^\n]*"([^"]+)"', text):
        if rule_id in valid_ids:
            positive.add(rule_id)


def collect_fixture_pairs(go_rules: list[str]) -> tuple[set[str], set[str]]:
    positive = set()
    negative = set()
    for rule_id in go_rules:
        positive_matches = list(GO_RULE_FIXTURE_ROOT.glob(f"*/{rule_id}_positive.txt"))
        negative_matches = list(GO_RULE_FIXTURE_ROOT.glob(f"*/{rule_id}_negative.txt"))
        if positive_matches:
            positive.add(rule_id)
        if negative_matches:
            negative.add(rule_id)
    return positive, negative


def main() -> None:
    go_rules = load_go_rules()
    valid_ids = set(go_rules)
    positive: set[str] = set()
    negative: set[str] = set()

    for path in TESTS_ROOT.rglob("*.rs"):
        text = path.read_text(errors="ignore")
        constants = collect_array_constants(text, valid_ids)
        update_from_assert_calls(text, constants, valid_ids, positive, negative)
        update_from_direct_asserts(text, valid_ids, positive, negative)

    both = positive & negative
    go_perf_layer = {rule_id for rule_id in valid_ids if rule_id.startswith("go_perf_layer_")}
    go_perf_layer_both = both & go_perf_layer
    fixture_positive, fixture_negative = collect_fixture_pairs(go_rules)
    fixture_both = fixture_positive & fixture_negative

    print(f"go_total {len(valid_ids)}")
    print(f"positive_rules {len(positive)}")
    print(f"negative_rules {len(negative)}")
    print(f"both_positive_and_negative {len(both)}")
    print(f"missing_positive {len(valid_ids - positive)}")
    print(f"missing_negative {len(valid_ids - negative)}")
    print(f"missing_both {len(valid_ids - both)}")
    print(f"go_perf_layer_total {len(go_perf_layer)}")
    print(f"go_perf_layer_both {len(go_perf_layer_both)}")
    print(f"go_perf_layer_missing_both {len(go_perf_layer - go_perf_layer_both)}")
    print(f"fixture_positive_files {len(fixture_positive)}")
    print(f"fixture_negative_files {len(fixture_negative)}")
    print(f"fixture_both_positive_and_negative {len(fixture_both)}")
    print(f"fixture_missing_both {len(valid_ids - fixture_both)}")


if __name__ == "__main__":
    main()
