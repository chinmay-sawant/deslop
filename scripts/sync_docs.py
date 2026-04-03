#!/usr/bin/env python3
"""Synchronize machine-managed docs content from the central rule registry."""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = ROOT / "rules" / "registry.json"
README_PATH = ROOT / "README.md"
FEATURES_PATH = ROOT / "guides" / "features-and-detections.md"
DOCS_CONTENT_PATH = ROOT / "frontend" / "src" / "features" / "docs" / "docs-content.ts"
ACTION_PATH = ROOT / "action.yml"
CARGO_TOML_PATH = ROOT / "Cargo.toml"

LANGUAGE_ORDER = ["common", "go", "python", "rust"]
STATUS_ORDER = ["stable", "experimental", "research"]
README_ACTION_REF = "chinmay-sawant/deslop@v{version}"
FRONTEND_ACTION_REF = "${currentRelease.actionRef}"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="validate without writing files")
    args = parser.parse_args()

    registry = load_registry()
    validate_registry(registry)

    cargo_version = load_cargo_version()
    action_inputs = parse_action_inputs(ACTION_PATH.read_text(encoding="utf-8"))

    changed = []
    changed.extend(
        sync_marked_block(
            README_PATH,
            "<!-- GENERATED_RULE_SUMMARY_START -->",
            "<!-- GENERATED_RULE_SUMMARY_END -->",
            build_readme_rule_summary(registry),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            README_PATH,
            "<!-- GENERATED_ACTION_INPUTS_START -->",
            "<!-- GENERATED_ACTION_INPUTS_END -->",
            build_action_inputs_markdown(action_inputs),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            README_PATH,
            "<!-- GENERATED_ACTION_SCAN_EXAMPLE_START -->",
            "<!-- GENERATED_ACTION_SCAN_EXAMPLE_END -->",
            build_readme_action_example(
                build_action_scan_example(README_ACTION_REF.format(version=cargo_version))
            ),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            README_PATH,
            "<!-- GENERATED_ACTION_JSON_EXAMPLE_START -->",
            "<!-- GENERATED_ACTION_JSON_EXAMPLE_END -->",
            build_readme_action_example(
                build_action_json_example(README_ACTION_REF.format(version=cargo_version))
            ),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            README_PATH,
            "<!-- GENERATED_ACTION_BENCH_EXAMPLE_START -->",
            "<!-- GENERATED_ACTION_BENCH_EXAMPLE_END -->",
            build_readme_action_example(
                build_action_bench_example(README_ACTION_REF.format(version=cargo_version))
            ),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            DOCS_CONTENT_PATH,
            "// GENERATED_RULES_START",
            "// GENERATED_RULES_END",
            build_frontend_rules_block(registry),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            DOCS_CONTENT_PATH,
            "// GENERATED_ACTION_INPUTS_START",
            "// GENERATED_ACTION_INPUTS_END",
            build_frontend_action_inputs_block(action_inputs),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            DOCS_CONTENT_PATH,
            "// GENERATED_ACTION_EXAMPLES_START",
            "// GENERATED_ACTION_EXAMPLES_END",
            build_frontend_action_examples_block(),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_features_inventory(
            registry,
            cargo_version,
            check_only=args.check,
        )
    )

    if args.check:
        if changed:
            for path in changed:
                print(f"out of date: {path}")
            return 1
        print("docs are in sync")
        return 0

    for path in changed:
        print(f"updated: {path}")
    return 0


def load_registry() -> list[dict]:
    return json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))


def validate_registry(registry: list[dict]) -> None:
    pairs = [(item["language"], item["id"]) for item in registry]
    if len(pairs) != len(set(pairs)):
        duplicates = [
            f"{language}:{rule_id}"
            for (language, rule_id), count in Counter(pairs).items()
            if count > 1
        ]
        raise SystemExit(f"duplicate language-scoped rule ids in registry: {duplicates}")

    for item in registry:
        if item["language"] not in LANGUAGE_ORDER:
            raise SystemExit(f"unknown registry language: {item['language']}")
        if item["status"] not in STATUS_ORDER:
            raise SystemExit(f"unknown registry status: {item['status']}")
        if not item["family"] or not item["description"]:
            raise SystemExit(f"registry entry is missing family/description: {item['id']}")

    expected = sorted(
        registry,
        key=lambda item: (
            LANGUAGE_ORDER.index(item["language"]),
            item["family"],
            item["id"],
        ),
    )
    if registry != expected:
        raise SystemExit("rules/registry.json must stay sorted by language, family, then id")


def load_cargo_version() -> str:
    cargo_text = CARGO_TOML_PATH.read_text(encoding="utf-8")
    match = re.search(r'^version = "([^"]+)"$', cargo_text, re.M)
    if not match:
        raise SystemExit("failed to parse Cargo.toml version")
    return match.group(1)


def parse_action_inputs(text: str) -> list[dict]:
    inputs: list[dict] = []
    lines = text.splitlines()
    in_inputs = False
    current: dict | None = None

    for raw_line in lines:
        line = raw_line.rstrip()
        stripped = line.strip()
        indent = len(line) - len(line.lstrip(" "))

        if stripped == "inputs:":
            in_inputs = True
            current = None
            continue

        if in_inputs and indent == 0 and stripped.endswith(":") and stripped != "inputs:":
            break

        if not in_inputs or not stripped:
            continue

        if indent == 2 and stripped.endswith(":"):
            if current is not None:
                inputs.append(current)
            current = {"name": stripped[:-1]}
            continue

        if current is None or indent < 4 or ":" not in stripped:
            continue

        key, value = stripped.split(":", 1)
        current[key.strip()] = value.strip().strip("'\"")

    if current is not None:
        inputs.append(current)

    return inputs


def sync_marked_block(
    path: Path,
    start_marker: str,
    end_marker: str,
    generated_body: str,
    *,
    check_only: bool,
) -> list[Path]:
    original = path.read_text(encoding="utf-8")
    start_token = f"{start_marker}\n"
    start_index = original.find(start_token)
    if start_index < 0:
        raise SystemExit(f"failed to locate generated block in {path}")

    content_start = start_index + len(start_token)
    end_index = original.find(end_marker, content_start)
    if end_index < 0:
        raise SystemExit(f"failed to locate generated block in {path}")

    replacement_body = generated_body
    if replacement_body:
        replacement_body += "\n"
    updated = original[:content_start] + replacement_body + original[end_index:]

    if updated == original:
        return []

    if not check_only:
        path.write_text(updated, encoding="utf-8")
    return [path]


def sync_features_inventory(
    registry: list[dict],
    cargo_version: str,
    *,
    check_only: bool,
) -> list[Path]:
    original = FEATURES_PATH.read_text(encoding="utf-8")
    generated = build_features_inventory(registry, cargo_version)
    pattern = re.compile(
        r"(## What deslop detects today\n\n)(.*?)(\n## Detection philosophy)",
        re.S,
    )
    updated, count = pattern.subn(
        lambda match: f"{match.group(1)}{generated}{match.group(3)}",
        original,
    )
    if count != 1:
        raise SystemExit("failed to locate inventory section in guides/features-and-detections.md")

    if updated == original:
        return []

    if not check_only:
        FEATURES_PATH.write_text(updated, encoding="utf-8")
    return [FEATURES_PATH]


def build_readme_rule_summary(registry: list[dict]) -> str:
    counts = summarize_counts(registry)
    lines = [
        "deslop now publishes a central rule registry that drives the CLI and the synced docs surfaces.",
        "",
        "| Language | Stable | Experimental | Research | Total |",
        "| --- | ---: | ---: | ---: | ---: |",
    ]
    total_stable = total_experimental = total_research = total_rules = 0
    for language in LANGUAGE_ORDER:
        stable = counts[language]["stable"]
        experimental = counts[language]["experimental"]
        research = counts[language]["research"]
        total = stable + experimental + research
        total_stable += stable
        total_experimental += experimental
        total_research += research
        total_rules += total
        lines.append(
            f"| {language} | {stable} | {experimental} | {research} | {total} |"
        )
    lines.append(
        f"| total | {total_stable} | {total_experimental} | {total_research} | {total_rules} |"
    )
    lines.append("")
    lines.append(
        "The totals above are language-scoped rule entries, so a shared rule ID implemented in more than one backend appears in each relevant language bucket."
    )
    lines.append(
        "The registry is now the source of truth for `deslop rules`, the frontend rule catalog, and the generated detection inventory guide."
    )
    return "\n".join(lines)


def build_action_inputs_markdown(inputs: list[dict]) -> str:
    lines = []
    for item in inputs:
        description = item.get("description", "")
        default = item.get("default", "")
        required = item.get("required", "")
        extra = []
        if default:
            extra.append(f"Defaults to `{default}`.")
        if required:
            extra.append("Required." if required == "true" else "Optional.")
        suffix = f" {' '.join(extra)}" if extra else ""
        lines.append(f"- `{item['name']}`: {description}{suffix}")
    return "\n".join(lines)


def build_readme_action_example(body: str) -> str:
    return f"```yaml\n{body}\n```"


def build_frontend_rules_block(registry: list[dict]) -> str:
    grouped: dict[str, list[dict]] = defaultdict(list)
    for item in registry:
        grouped[item["language"]].append(item)

    lines = []
    for language in LANGUAGE_ORDER:
        name = {
            "common": "commonRules",
            "go": "goRules",
            "python": "pythonRules",
            "rust": "rustRules",
        }[language]
        lines.append(f"const {name}: Rule[] = [")
        for item in grouped[language]:
            lines.append(
                f"  {{ id: '{item['id']}', description: '{ts_string(item['description'])}' }},"
            )
        lines.append("]")
        lines.append("")
    return "\n".join(lines).rstrip()


def build_frontend_action_inputs_block(inputs: list[dict]) -> str:
    lines = ["const githubActionInputs: GitHubActionInput[] = ["]
    for item in inputs:
        description = item.get("description", "")
        default = item.get("default", "")
        required = item.get("required", "")
        extra = []
        if default:
            extra.append(f"Defaults to {default}.")
        if required:
            extra.append("Required." if required == "true" else "Optional.")
        final_description = " ".join([description, *extra]).strip()
        lines.append(
            f"  {{ name: '{item['name']}', description: '{ts_string(final_description)}' }},"
        )
    lines.append("]")
    return "\n".join(lines)


def build_frontend_action_examples_block() -> str:
    workflow = ts_template(build_action_scan_example(FRONTEND_ACTION_REF))
    json_example = ts_template(build_action_json_example(FRONTEND_ACTION_REF))
    bench_example = ts_template(build_action_bench_example(FRONTEND_ACTION_REF))
    return "\n".join(
        [
            f"const githubActionWorkflow = `{workflow}`",
            "",
            f"const githubActionJsonExample = `{json_example}`",
            "",
            f"const githubActionBenchExample = `{bench_example}`",
        ]
    )


def build_features_inventory(registry: list[dict], cargo_version: str) -> str:
    counts = summarize_counts(registry)
    total_rules = len(registry)
    lines = [
        f"The shipped registry currently tracks **{total_rules} language-scoped rule entries** in deslop `{cargo_version}`.",
        "",
        "| Language | Stable | Experimental | Research | Total |",
        "| --- | ---: | ---: | ---: | ---: |",
    ]
    for language in LANGUAGE_ORDER:
        stable = counts[language]["stable"]
        experimental = counts[language]["experimental"]
        research = counts[language]["research"]
        total = stable + experimental + research
        lines.append(
            f"| {language} | {stable} | {experimental} | {research} | {total} |"
        )

    lines.append("")
    lines.append(
        "The sections below are generated from the rule registry and grouped by language and family."
    )
    lines.append(
        "When the same rule ID is implemented in more than one backend, it appears once in each relevant language section."
    )
    lines.append("")

    by_language: dict[str, dict[str, list[dict]]] = defaultdict(lambda: defaultdict(list))
    for item in registry:
        by_language[item["language"]][item["family"]].append(item)

    for language in LANGUAGE_ORDER:
        language_rules = by_language[language]
        total = sum(len(items) for items in language_rules.values())
        lines.append(f"### {language.title()} rules ({total})")
        lines.append("")
        for family in sorted(language_rules):
            lines.append(f"#### {family.replace('_', ' ').title()} ({len(language_rules[family])})")
            for item in language_rules[family]:
                status_suffix = (
                    f" *(status: {item['status']})*"
                    if item["status"] != "stable"
                    else ""
                )
                lines.append(
                    f"- `{item['id']}`: {item['description']}{status_suffix}"
                )
            lines.append("")

    return "\n".join(lines).rstrip()


def summarize_counts(registry: list[dict]) -> dict[str, Counter]:
    counts = {language: Counter() for language in LANGUAGE_ORDER}
    for item in registry:
        counts[item["language"]][item["status"]] += 1
    return counts


def ts_string(value: str) -> str:
    return value.replace("\\", "\\\\").replace("'", "\\'")


def ts_template(value: str) -> str:
    return value.replace("\\", "\\\\").replace("`", "\\`")


def build_action_scan_example(action_ref: str) -> str:
    return "\n".join(
        [
            "name: Deslop",
            "",
            "on:",
            "  pull_request:",
            "  push:",
            "    branches:",
            "      - main",
            "",
            "jobs:",
            "  scan:",
            "    runs-on: ubuntu-latest",
            "    steps:",
            "      - uses: actions/checkout@v4",
            f"      - uses: {action_ref}",
            "        with:",
            "          path: .",
        ]
    )


def build_action_json_example(action_ref: str) -> str:
    return "\n".join(
        [
            "- uses: actions/checkout@v4",
            f"- uses: {action_ref}",
            "  with:",
            "    path: .",
            "    json: 'true'",
            "    details: 'true'",
            "    fail-on-findings: 'false'",
        ]
    )


def build_action_bench_example(action_ref: str) -> str:
    return "\n".join(
        [
            "- uses: actions/checkout@v4",
            f"- uses: {action_ref}",
            "  with:",
            "    command: bench",
            "    path: .",
            "    repeats: '10'",
            "    warmups: '2'",
        ]
    )


if __name__ == "__main__":
    sys.exit(main())
