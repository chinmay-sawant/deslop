#!/usr/bin/env python3
"""Expand deslop findings into code-context review blocks.

This script reads a findings file such as ``temp_gopdfsuit.txt`` and writes a
single consolidated text file to ``scripts/temp.txt``. By default each finding
block includes the file path, rule description, auto-triage note, and code
context. Pass ``--details`` to emit the full metadata-rich block instead.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_INPUT = ROOT / "temp_gopdfsuit.txt"
DEFAULT_OUTPUT = Path(__file__).resolve().parent / "temp.txt"
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


@dataclass(frozen=True)
class Finding:
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


RuleMetadataById = dict[str, tuple[RuleMetadata, ...]]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "input_file",
        nargs="?",
        default=str(DEFAULT_INPUT),
        help="Path to the deslop findings text file.",
    )
    parser.add_argument(
        "--output",
        default=str(DEFAULT_OUTPUT),
        help="Path to the consolidated output file.",
    )
    parser.add_argument(
        "--context",
        type=int,
        default=1,
        help="How many lines to include above and below the flagged line.",
    )
    parser.add_argument(
        "--review-placeholder",
        default="REVIEW_NEEDED",
        help="Initial value written into the false-positive review field.",
    )
    parser.add_argument(
        "--details",
        action="store_true",
        help="Emit the full metadata-rich finding blocks instead of the compact default view.",
    )
    return parser.parse_args()


def parse_findings(input_path: Path) -> list[Finding]:
    findings: list[Finding] = []
    for raw_line in input_path.read_text(encoding="utf-8").splitlines():
        if not raw_line.lstrip().startswith("- "):
            continue
        match = FINDING_RE.match(raw_line)
        if not match:
            continue
        findings.append(
            Finding(
                file_path=Path(match.group(1)),
                line_no=int(match.group(2)),
                message=match.group(3).strip(),
                rule_id=match.group(4).strip(),
                raw_line=raw_line.strip(),
            )
        )
    return findings


def load_lines(file_path: Path) -> list[str]:
    return file_path.read_text(encoding="utf-8", errors="replace").splitlines()


def _read_rule_metadata() -> RuleMetadataById:
    registry = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    grouped: dict[str, list[RuleMetadata]] = defaultdict(list)

    for item in registry:
        grouped[item["id"]].append(
            RuleMetadata(
                rule_id=item["id"],
                language=item["language"],
                family=item["family"],
                default_severity=item["default_severity"],
                status=item["status"],
                description=item["description"],
            )
        )

    return {
        rule_id: tuple(sorted(variants, key=lambda variant: variant.language))
        for rule_id, variants in grouped.items()
    }


def infer_language(file_path: Path) -> str | None:
    return LANGUAGE_BY_SUFFIX.get(file_path.suffix.lower())


def _rule_variants(rule_id: str, rule_metadata_by_id: RuleMetadataById) -> tuple[RuleMetadata, ...]:
    return rule_metadata_by_id.get(rule_id, ())


def _resolve_rule_metadata(
    finding: Finding,
    rule_metadata_by_id: RuleMetadataById,
) -> RuleMetadata | None:
    variants = _rule_variants(finding.rule_id, rule_metadata_by_id)
    inferred_language = infer_language(finding.file_path)
    return next(
        (variant for variant in variants if variant.language == inferred_language),
        variants[0] if variants else None,
    )


def _summarize_rule_metadata(
    rule_id: str,
    rule_metadata_by_id: RuleMetadataById,
) -> tuple[str, str, str, str, str]:
    variants = _rule_variants(rule_id, rule_metadata_by_id)
    if not variants:
        return (
            "unknown",
            "unknown",
            "unknown",
            "unknown",
            "Rule metadata not found in rules/registry.json.",
        )

    first = variants[0]
    languages = ", ".join(variant.language for variant in variants)
    return (
        first.family,
        first.default_severity,
        first.status,
        languages,
        first.description,
    )


def _build_rule_inventory_summary(
    findings: list[Finding],
    rule_metadata_by_id: RuleMetadataById,
) -> str:
    rule_counts = Counter(finding.rule_id for finding in findings)
    rule_summaries = {
        rule_id: _summarize_rule_metadata(rule_id, rule_metadata_by_id)
        for rule_id in rule_counts
    }
    families = {summary[0] for summary in rule_summaries.values()}
    family_finding_counts = Counter(
        {
            family: sum(
                count
                for rule_id, count in rule_counts.items()
                if rule_summaries[rule_id][0] == family
            )
            for family in families
        }
    )
    family_rule_counts = Counter(summary[0] for summary in rule_summaries.values())
    missing_rule_ids = sorted(
        rule_id for rule_id, summary in rule_summaries.items() if summary[0] == "unknown"
    )

    lines = [
        "Rule inventory:",
        f"- Registry unique rule ids: {len(rule_metadata_by_id)}",
        f"- Registry language-scoped rules: {sum(len(variants) for variants in rule_metadata_by_id.values())}",
        f"- Rule ids in findings: {len(rule_counts)}",
        f"- Rule ids missing from registry: {len(missing_rule_ids)}",
        "",
        "Family summary:",
    ]

    lines.extend(
        f"- {family} | findings={count} | rules={family_rule_counts[family]}"
        for family, count in sorted(
            family_finding_counts.items(),
            key=lambda item: (-item[1], item[0]),
        )
    )

    if missing_rule_ids:
        lines.extend(
            [
                "",
                "Registry gaps:",
                *[f"- {rule_id}" for rule_id in missing_rule_ids],
            ]
        )

    lines.extend(["", "Rule summary:"])
    lines.extend(
        (
            f"- {rule_id} | findings={count} | family={family} | severity={severity} "
            f"| status={status} | languages={languages} | description={description}"
        )
        for rule_id, count in sorted(rule_counts.items(), key=lambda item: (-item[1], item[0]))
        for family, severity, status, languages, description in [rule_summaries[rule_id]]
    )

    lines.append("")
    return "\n".join(lines)


def _build_context_block(
    finding: Finding,
    lines: list[str],
    rule_metadata_by_id: RuleMetadataById,
    *,
    context: int,
    index: int,
    total: int,
    review_placeholder: str,
    details: bool,
) -> str:
    start_line = max(1, finding.line_no - context)
    end_line = min(len(lines), finding.line_no + context)
    width = max(4, len(str(end_line)))
    rule_metadata = _resolve_rule_metadata(finding, rule_metadata_by_id)
    auto_triage, auto_reason = _triage_finding(
        finding,
        lines,
        start_line,
        end_line,
        rule_metadata,
    )
    family, severity, status, languages, description = _summarize_rule_metadata(
        finding.rule_id,
        rule_metadata_by_id,
    )

    block_lines = ["=" * 100, f"Finding {index}/{total}"]
    if details:
        block_lines.extend(
            [
                f"Source: {finding.file_path}:{finding.line_no}",
                f"Rule: [{finding.rule_id}]",
                f"Rule family: [{family}]",
                f"Rule severity: [{severity}]",
                f"Rule status: [{status}]",
                f"Rule languages: [{languages}]",
                f"Rule description: {description}",
                f"Message: {finding.message}",
                f"Suspect range: [{start_line}-{end_line}]",
                f"Auto triage: [{auto_triage}]",
                f"Auto triage note: {auto_reason}",
                f"False positive: [{review_placeholder}]",
                f"Original finding: {finding.raw_line}",
                "Code:",
            ]
        )
    else:
        block_lines.extend(
            [
                f"Source: {finding.file_path}:{finding.line_no}",
                f"Rule: [{finding.rule_id}]",
                f"Rule description: {description}",
                f"Auto triage note: {auto_reason}",
                "Code:",
            ]
        )

    block_lines.extend(
        f'{">>" if line_number == finding.line_no else "  "} {line_number:>{width}} | {lines[line_number - 1]}'
        for line_number in range(start_line, end_line + 1)
    )

    block_lines.append("")
    return "\n".join(block_lines)


def _build_missing_file_block(
    finding: Finding,
    rule_metadata_by_id: RuleMetadataById,
    *,
    index: int,
    total: int,
    review_placeholder: str,
    details: bool,
) -> str:
    rule_metadata = _resolve_rule_metadata(finding, rule_metadata_by_id)
    auto_triage, auto_reason = _triage_finding(
        finding,
        [],
        finding.line_no,
        finding.line_no,
        rule_metadata,
    )
    family, severity, status, languages, description = _summarize_rule_metadata(
        finding.rule_id,
        rule_metadata_by_id,
    )
    block_lines = ["=" * 100, f"Finding {index}/{total}"]
    if details:
        block_lines.extend(
            [
                f"Source: {finding.file_path}:{finding.line_no}",
                f"Rule: [{finding.rule_id}]",
                f"Rule family: [{family}]",
                f"Rule severity: [{severity}]",
                f"Rule status: [{status}]",
                f"Rule languages: [{languages}]",
                f"Rule description: {description}",
                f"Message: {finding.message}",
                f"Auto triage: [{auto_triage}]",
                f"Auto triage note: {auto_reason}",
                f"False positive: [{review_placeholder}]",
                f"Original finding: {finding.raw_line}",
                "Code: [FILE_NOT_FOUND]",
            ]
        )
    else:
        block_lines.extend(
            [
                f"Rule description: {description}",
                f"Auto triage note: {auto_reason}",
                "Code: [FILE_NOT_FOUND]",
            ]
        )
    block_lines.append("")
    return "\n".join(block_lines)


def write_output(
    findings: list[Finding],
    input_path: Path,
    output_path: Path,
    *,
    context: int,
    review_placeholder: str,
    details: bool,
) -> None:
    total_findings = len(findings)
    rule_metadata_by_id = _read_rule_metadata()
    cached_files: dict[Path, list[str]] = {}
    blocks = [
        f"Input file: {input_path}",
        f"Output file: {output_path}",
        f"Total findings parsed: {total_findings}",
        f"Context window: +/- {context} lines",
    ]
    if details:
        blocks.extend(["", _build_rule_inventory_summary(findings, rule_metadata_by_id)])

    for index, finding in enumerate(findings, start=1):
        if not finding.file_path.exists():
            blocks.append(
                _build_missing_file_block(
                    finding,
                    rule_metadata_by_id,
                    index=index,
                    total=total_findings,
                    review_placeholder=review_placeholder,
                    details=details,
                )
            )
            continue

        if finding.file_path not in cached_files:
            cached_files[finding.file_path] = load_lines(finding.file_path)

        blocks.append(
            _build_context_block(
                finding,
                cached_files[finding.file_path],
                rule_metadata_by_id,
                context=context,
                index=index,
                total=total_findings,
                review_placeholder=review_placeholder,
                details=details,
            )
        )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    if output_path.exists():
        output_path.unlink()
    output_path.write_text("\n".join(blocks), encoding="utf-8")


def _triage_len_empty(current_line: str) -> tuple[str, str]:
    collection_hints = (
        "parts",
        "items",
        "files",
        "rows",
        "entries",
        "fonts",
        "pages",
        "results",
        "matches",
        "tokens",
        "children",
        "values",
    )
    if any(hint in current_line for hint in collection_hints):
        return (
            "LIKELY_FALSE_POSITIVE",
            "The flagged len(...) check appears to target a collection rather than a string empty-check.",
        )
    return (
        "CONTEXT_DEPENDENT",
        "This may be style-only or incorrect depending on the type of the value passed to len(...).",
    )


def _triage_by_metadata(rule_metadata: RuleMetadata | None) -> tuple[str, str]:
    if rule_metadata is None:
        return (
            "REVIEW_NEEDED",
            "No safe automatic classification was inferred from the local code context alone.",
        )

    experimental_note = (
        " The rule is marked experimental in the registry, so keep a slightly higher false-positive bar."
        if rule_metadata.status == "experimental"
        else ""
    )

    if rule_metadata.family in SUBJECTIVE_FAMILIES:
        return (
            "LIKELY_SUBJECTIVE",
            f"Registry metadata classifies this as {rule_metadata.family} guidance with {rule_metadata.default_severity} severity, so whether it matters depends on project conventions.{experimental_note}",
        )

    if rule_metadata.family in CONTEXT_FAMILIES or rule_metadata.default_severity == "contextual":
        return (
            "CONTEXT_DEPENDENT",
            f"Registry metadata classifies this as {rule_metadata.family} with {rule_metadata.default_severity} severity, so runtime path, workload, and surrounding design matter before treating it as actionable.{experimental_note}",
        )

    if rule_metadata.family in RISK_FAMILIES or rule_metadata.default_severity in {"warning", "error"}:
        return (
            "LIKELY_REAL",
            f"Registry metadata classifies this as {rule_metadata.family} with {rule_metadata.default_severity} severity, which usually maps to correctness, security, or production risk.{experimental_note}",
        )

    if rule_metadata.default_severity == "info":
        return (
            "LIKELY_SUBJECTIVE",
            f"Registry metadata marks this as info-level guidance; treat it as a review prompt rather than a clear defect.{experimental_note}",
        )

    return (
        "REVIEW_NEEDED",
        "No safe automatic classification was inferred from the local code context alone.",
    )


def _triage_finding(
    finding: Finding,
    lines: list[str],
    start_line: int,
    end_line: int,
    rule_metadata: RuleMetadata | None,
) -> tuple[str, str]:
    context_lines = lines[start_line - 1 : end_line] if lines else []
    context_text = "\n".join(context_lines).lower()
    search_start = max(0, finding.line_no - 26)
    search_end = min(len(lines), finding.line_no + 5)
    extended_text = "\n".join(lines[search_start:search_end]).lower() if lines else ""
    current_line = lines[finding.line_no - 1].strip() if lines and 0 < finding.line_no <= len(lines) else ""

    if finding.rule_id == "cgo_string_lifetime" and "caller must free" in (context_text + "\n" + extended_text):
        return (
            "LIKELY_FALSE_POSITIVE",
            "Nearby comments suggest the API intentionally transfers ownership of the allocated C string to the caller.",
        )

    if finding.rule_id == "len_string_for_empty_check":
        return _triage_len_empty(current_line)

    return _triage_by_metadata(rule_metadata)


def main() -> int:
    args = parse_args()
    input_path = Path(args.input_file).expanduser().resolve()
    output_path = Path(args.output).expanduser().resolve()

    findings = parse_findings(input_path)
    if not findings:
        raise SystemExit(f"no findings matched the expected format in {input_path}")

    write_output(
        findings,
        input_path,
        output_path,
        context=max(0, args.context),
        review_placeholder=args.review_placeholder,
        details=args.details,
    )

    print(f"Wrote {len(findings)} finding blocks to {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
