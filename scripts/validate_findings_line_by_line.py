#!/usr/bin/env python3
"""Line-by-line validation pass for deslop findings.

Reads a findings file (default: temp_gopdfsuit.txt), opens each referenced file,
inspects the referenced line with nearby context, and emits:

1) A CSV with one row per finding and a validation verdict.
2) A Markdown summary with coverage + verdict statistics.
"""

from __future__ import annotations

import argparse
import csv
import json
import re
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_INPUT = ROOT / "temp_gopdfsuit.txt"
DEFAULT_CSV = ROOT / "reports" / "gopdfsuit_line_by_line_validation.csv"
DEFAULT_SUMMARY = ROOT / "reports" / "gopdfsuit_line_by_line_validation_summary.md"
REGISTRY_PATH = ROOT / "rules" / "registry.json"

FINDING_RE = re.compile(r"^\s*-\s+(.*?):(\d+)\s+(.*?)\s+\[([^\]]+)\]\s*$")

LANGUAGE_BY_SUFFIX = {
    ".go": "go",
    ".py": "python",
    ".rs": "rust",
}

SUBJECTIVE_FAMILIES = {
    "ai_smells",
    "api_design",
    "comments",
    "domain_modeling",
    "duplication",
    "maintainability",
    "mod",
    "module_surface",
    "naming",
    "packaging",
    "quality",
    "structure",
    "style",
    "test_quality",
}

CONTEXT_FAMILIES = {
    "async_patterns",
    "boundary",
    "concurrency",
    "consistency",
    "context",
    "data_access",
    "framework",
    "gin",
    "hot_path",
    "hot_path_ext",
    "idioms",
    "library",
    "mlops",
    "performance",
    "runtime_boundary",
    "runtime_ownership",
}

RISK_FAMILIES = {
    "errors",
    "hallucination",
    "hygiene",
    "security",
    "security_footguns",
    "unsafe_soundness",
}

ASSERT_MARKERS = (
    "require.",
    "assert.",
    "s.noerror(",
    "s.equal(",
    "s.error(",
    "s.lessorequal(",
    "s.greater(",
    "s.true(",
    "s.false(",
    "s.notnil(",
    "s.nil(",
    "self.assert",
    "pytest.raises",
    "t.error(",
    "t.fatal(",
)


@dataclass(frozen=True)
class Finding:
    index: int
    file_path: Path
    line_no: int
    message: str
    rule_id: str
    raw_line: str


@dataclass(frozen=True)
class RuleMetadata:
    rule_id: str
    language: str
    family: str
    default_severity: str
    status: str
    description: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input_file", nargs="?", default=str(DEFAULT_INPUT))
    parser.add_argument("--csv", default=str(DEFAULT_CSV))
    parser.add_argument("--summary", default=str(DEFAULT_SUMMARY))
    parser.add_argument(
        "--context-lines",
        type=int,
        default=3,
        help="Context lines above and below the finding line used for reasoning.",
    )
    return parser.parse_args()


def parse_findings(path: Path) -> list[Finding]:
    findings: list[Finding] = []
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        match = FINDING_RE.match(raw)
        if not match:
            continue
        findings.append(
            Finding(
                index=len(findings) + 1,
                file_path=Path(match.group(1)),
                line_no=int(match.group(2)),
                message=match.group(3).strip(),
                rule_id=match.group(4).strip(),
                raw_line=raw.strip(),
            )
        )
    return findings


def load_registry() -> dict[str, list[RuleMetadata]]:
    raw = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    out: dict[str, list[RuleMetadata]] = defaultdict(list)
    for item in raw:
        out[item["id"]].append(
            RuleMetadata(
                rule_id=item["id"],
                language=item["language"],
                family=item["family"],
                default_severity=item["default_severity"],
                status=item["status"],
                description=item["description"],
            )
        )
    return dict(out)


def infer_language(path: Path) -> str | None:
    return LANGUAGE_BY_SUFFIX.get(path.suffix.lower())


def resolve_rule_metadata(
    finding: Finding, registry: dict[str, list[RuleMetadata]]
) -> RuleMetadata | None:
    variants = registry.get(finding.rule_id, [])
    if not variants:
        return None
    inferred = infer_language(finding.file_path)
    if inferred:
        for variant in variants:
            if variant.language == inferred:
                return variant
    return variants[0]


def extract_context(lines: list[str], line_no: int, context: int) -> tuple[int, int, str]:
    if not lines:
        return (line_no, line_no, "")
    start = max(1, line_no - context)
    end = min(len(lines), line_no + context)
    block = "\n".join(lines[start - 1 : end])
    return (start, end, block)


def count_literal_keys_in_block(lines: list[str], start_line: int) -> int:
    key_count = 0
    brace_balance = 0
    started = False
    for raw in lines[start_line - 1 :]:
        text = raw.split("//", 1)[0].strip()
        if not text:
            continue
        if not started:
            if "{" not in text:
                continue
            started = True
        key_count += text.count(":")
        brace_balance += text.count("{")
        brace_balance -= text.count("}")
        if started and brace_balance <= 0:
            break
    return key_count


def classify(
    finding: Finding,
    lines: list[str],
    metadata: RuleMetadata | None,
    context_lines: int,
) -> tuple[str, str, str]:
    # verdict: FALSE_POSITIVE | NOT_FALSE_POSITIVE | NEEDS_MANUAL_REVIEW
    if not lines:
        return (
            "NEEDS_MANUAL_REVIEW",
            "low",
            "Referenced file could not be read; no line-level validation possible.",
        )

    if finding.line_no < 1 or finding.line_no > len(lines):
        return (
            "NEEDS_MANUAL_REVIEW",
            "low",
            "Referenced line number is outside file bounds; cannot validate from source.",
        )

    line = lines[finding.line_no - 1]
    line_lower = line.lower()
    _, _, context_block = extract_context(lines, finding.line_no, context_lines)
    context_lower = context_block.lower()

    # Rule-specific high-confidence checks.
    if finding.rule_id == "rows_without_close":
        if "c.Query(" in line or "ctx.Query(" in line:
            return (
                "FALSE_POSITIVE",
                "high",
                "Looks like HTTP query parameter access, not a DB rows handle.",
            )
        if ".Query(" in line:
            return (
                "NOT_FALSE_POSITIVE",
                "medium",
                "Query-like handle assignment detected; missing Close() can be a real hygiene issue.",
            )

    if finding.rule_id == "world_readable_file_permissions":
        mode_match = re.search(r"\b0[0-7]{3}\b", line)
        if mode_match:
            mode = int(mode_match.group(0), 8)
            if mode in (0o666, 0o777):
                return (
                    "NOT_FALSE_POSITIVE",
                    "high",
                    f"World-readable or world-writable mode {mode_match.group(0)} is present.",
                )
            return (
                "FALSE_POSITIVE",
                "high",
                f"Mode {mode_match.group(0)} is not world-readable/writable as described by the rule.",
            )

    if finding.rule_id == "map_lookup_double_access":
        if "ok :=" in line and "[" in line and "]" in line:
            nearby = "\n".join(lines[max(0, finding.line_no - 1) : min(len(lines), finding.line_no + 3)])
            if re.search(r"\w+\[[^\]]+\]", nearby):
                return (
                    "NOT_FALSE_POSITIVE",
                    "medium",
                    "Nearby code appears to perform repeated map-key access.",
                )
            return (
                "FALSE_POSITIVE",
                "medium",
                "Second lookup for the same key is not visible in nearby code.",
            )

    if finding.rule_id == "repeated_json_dumps_same_object":
        # Conservative check: if only one dumps call appears in nearby context, likely FP.
        if context_lower.count("json.dumps(") <= 1:
            return (
                "FALSE_POSITIVE",
                "medium",
                "Only one json.dumps(...) call is visible near the flagged line.",
            )

    if finding.rule_id == "test_without_assertion_signal":
        scan_start = max(0, finding.line_no - 80)
        scan_end = min(len(lines), finding.line_no + 80)
        test_window = "\n".join(lines[scan_start:scan_end]).lower()
        if any(marker in test_window for marker in ASSERT_MARKERS):
            return (
                "FALSE_POSITIVE",
                "medium",
                "Assertion-like calls are present in the surrounding test body/window.",
            )
        return (
            "NOT_FALSE_POSITIVE",
            "medium",
            "No assertion-like markers were seen in the local test window.",
        )

    if finding.rule_id == "large_h_payload_built_only_for_json_response":
        key_count = count_literal_keys_in_block(lines, finding.line_no)
        if key_count < 5:
            return (
                "FALSE_POSITIVE",
                "medium",
                f"Map literal appears small ({key_count} keys) near the flagged location.",
            )
        return (
            "NOT_FALSE_POSITIVE",
            "medium",
            f"Map literal appears large ({key_count} keys) near the flagged location.",
        )

    if "login/authentication handler functions" in finding.message.lower():
        if "logauth" in finding.message.lower():
            return (
                "FALSE_POSITIVE",
                "high",
                "Flagged symbol appears to be logging/auth-info utility, not a login entrypoint.",
            )

    # Metadata-based fallback.
    if metadata is None:
        return (
            "NEEDS_MANUAL_REVIEW",
            "low",
            "Rule metadata missing; cannot classify reliably from generic fallback.",
        )

    if metadata.family in RISK_FAMILIES or metadata.default_severity in {"warning", "error"}:
        return (
            "NOT_FALSE_POSITIVE",
            "low",
            "Risk-oriented family/severity suggests potentially real issue; needs deeper semantic review.",
        )

    if metadata.family in SUBJECTIVE_FAMILIES:
        return (
            "NEEDS_MANUAL_REVIEW",
            "low",
            "Style/maintainability guidance is often project-convention dependent.",
        )

    if metadata.family in CONTEXT_FAMILIES or metadata.default_severity == "contextual":
        return (
            "NEEDS_MANUAL_REVIEW",
            "low",
            "Context/performance rule requires workload and architectural context to validate.",
        )

    if metadata.default_severity == "info":
        return (
            "NEEDS_MANUAL_REVIEW",
            "low",
            "Info-level guidance is not a reliable binary false-positive signal.",
        )

    return (
        "NEEDS_MANUAL_REVIEW",
        "low",
        "No high-confidence classifier matched; manual review required.",
    )


def write_summary(
    summary_path: Path,
    *,
    input_file: Path,
    total: int,
    file_accessed_count: int,
    verdict_counts: Counter,
    false_positive_true: int,
    false_positive_false: int,
    top_fp_rules: list[tuple[str, int]],
    top_manual_rules: list[tuple[str, int]],
    top_not_fp_rules: list[tuple[str, int]],
) -> None:
    lines = [
        "# gopdfsuit Line-by-Line Validation Summary",
        "",
        f"- Input file: `{input_file}`",
        f"- Findings processed: **{total}**",
        f"- Findings with readable source path: **{file_accessed_count}/{total}**",
        "",
        "## Binary False-Positive Result",
        "",
        f"- `false_positive = true`: **{false_positive_true}**",
        f"- `false_positive = false`: **{false_positive_false}**",
        "",
        "## Verdict Totals",
        "",
        f"- `FALSE_POSITIVE`: **{verdict_counts.get('FALSE_POSITIVE', 0)}**",
        f"- `NOT_FALSE_POSITIVE`: **{verdict_counts.get('NOT_FALSE_POSITIVE', 0)}**",
        f"- `NEEDS_MANUAL_REVIEW`: **{verdict_counts.get('NEEDS_MANUAL_REVIEW', 0)}**",
        "",
        "## Top False-Positive Rules",
        "",
    ]
    if top_fp_rules:
        lines.extend(f"- `{rule}`: {count}" for rule, count in top_fp_rules)
    else:
        lines.append("- none")

    lines.extend(["", "## Top Not-False-Positive Rules", ""])
    if top_not_fp_rules:
        lines.extend(f"- `{rule}`: {count}" for rule, count in top_not_fp_rules)
    else:
        lines.append("- none")

    lines.extend(["", "## Top Needs-Manual-Review Rules", ""])
    if top_manual_rules:
        lines.extend(f"- `{rule}`: {count}" for rule, count in top_manual_rules)
    else:
        lines.append("- none")

    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    input_file = Path(args.input_file).expanduser().resolve()
    csv_path = Path(args.csv).expanduser().resolve()
    summary_path = Path(args.summary).expanduser().resolve()

    findings = parse_findings(input_file)
    if not findings:
        raise SystemExit(f"No findings matched expected format in {input_file}")

    registry = load_registry()
    file_cache: dict[Path, list[str]] = {}
    verdict_counts: Counter = Counter()
    by_verdict_rule: dict[str, Counter] = {
        "FALSE_POSITIVE": Counter(),
        "NOT_FALSE_POSITIVE": Counter(),
        "NEEDS_MANUAL_REVIEW": Counter(),
    }
    file_accessed_count = 0

    csv_path.parent.mkdir(parents=True, exist_ok=True)
    with csv_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow(
            [
                "index",
                "path",
                "line",
                "rule_id",
                "message",
                "false_positive",
                "verdict",
                "confidence",
                "file_accessed",
                "reason",
                "line_text",
            ]
        )

        for finding in findings:
            if finding.file_path not in file_cache:
                if finding.file_path.exists():
                    file_cache[finding.file_path] = finding.file_path.read_text(
                        encoding="utf-8", errors="replace"
                    ).splitlines()
                else:
                    file_cache[finding.file_path] = []

            lines = file_cache[finding.file_path]
            file_accessed = bool(lines)
            if file_accessed:
                file_accessed_count += 1

            metadata = resolve_rule_metadata(finding, registry)
            verdict, confidence, reason = classify(
                finding,
                lines,
                metadata,
                args.context_lines,
            )
            verdict_counts[verdict] += 1
            by_verdict_rule.setdefault(verdict, Counter())[finding.rule_id] += 1

            line_text = (
                lines[finding.line_no - 1].strip()
                if lines and 1 <= finding.line_no <= len(lines)
                else ""
            )

            writer.writerow(
                [
                    finding.index,
                    str(finding.file_path),
                    finding.line_no,
                    finding.rule_id,
                    finding.message,
                    "true" if verdict == "FALSE_POSITIVE" else "false",
                    verdict,
                    confidence,
                    "true" if file_accessed else "false",
                    reason,
                    line_text,
                ]
            )

    write_summary(
        summary_path,
        input_file=input_file,
        total=len(findings),
        file_accessed_count=file_accessed_count,
        verdict_counts=verdict_counts,
        false_positive_true=verdict_counts.get("FALSE_POSITIVE", 0),
        false_positive_false=len(findings) - verdict_counts.get("FALSE_POSITIVE", 0),
        top_fp_rules=by_verdict_rule.get("FALSE_POSITIVE", Counter()).most_common(20),
        top_manual_rules=by_verdict_rule.get("NEEDS_MANUAL_REVIEW", Counter()).most_common(20),
        top_not_fp_rules=by_verdict_rule.get("NOT_FALSE_POSITIVE", Counter()).most_common(20),
    )

    print(f"Processed {len(findings)} findings")
    print(f"Wrote CSV: {csv_path}")
    print(f"Wrote summary: {summary_path}")
    print(f"Readable source paths: {file_accessed_count}/{len(findings)}")
    print(
        "Verdicts:",
        ", ".join(f"{k}={v}" for k, v in sorted(verdict_counts.items())),
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
