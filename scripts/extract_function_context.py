#!/usr/bin/env python3
"""Expand deslop findings into function-context review blocks.

This script reads a findings file such as ``temp_gopdfsuit.txt`` and writes one
text file per finding under ``scripts/findings/functions`` by default. Each
finding block includes the file path, rule description,
auto-triage note, and the full enclosing function when one can be resolved.
Pass ``--details`` to emit the full metadata-rich block instead.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_INPUT = ROOT / "temp_gopdfsuit.txt"
DEFAULT_OUTPUT_DIR = Path(__file__).resolve().parent / "findings" / "functions"
REGISTRY_PATH = ROOT / "rules" / "registry.json"
FINDING_RE = re.compile(r"^\s*-\s+(.*?):(\d+)\s+(.*?)\s+\[([^\]]+)\]\s*$")
LANGUAGE_BY_SUFFIX = {
    ".go": "go",
    ".py": "python",
    ".rs": "rust",
}
RUST_FN_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:default\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+[A-Za-z_][A-Za-z0-9_]*\b"
)
GO_FN_RE = re.compile(r"^\s*func(?:\s*\([^)]*\))?\s+[A-Za-z_][A-Za-z0-9_]*\b")
PY_FN_RE = re.compile(r"^\s*(?:async\s+def|def)\s+[A-Za-z_][A-Za-z0-9_]*\s*\(")
GENERIC_BRACE_FN_RE = re.compile(
    r"^\s*[A-Za-z_][\w:\<\>\[\]\*&\s,]*\s+[A-Za-z_][A-Za-z0-9_]*\s*\([^;]*\)\s*(?:->\s*[^\{]+)?\s*\{?\s*$"
)
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
        "--output-dir",
        default=str(DEFAULT_OUTPUT_DIR),
        help="Directory where per-finding text files are written (1.txt, 2.txt, ...).",
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


def _iter_txt_files(path: Path) -> Iterable[Path]:
    for child in path.iterdir():
        if child.is_file() and child.suffix.lower() == ".txt":
            yield child


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


def _leading_spaces(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _starts_char_literal(line: str, idx: int) -> bool:
    if idx + 2 >= len(line):
        return False
    if line[idx + 1] == "\\":
        return line.find("'", idx + 2, min(len(line), idx + 8)) != -1
    return line[idx + 2] == "'"


def _brace_scan_line(line: str, in_block_comment: bool) -> tuple[int, bool, bool]:
    delta = 0
    saw_open = False
    i = 0
    in_single = False
    in_double = False
    in_backtick = False
    escape = False

    while i < len(line):
        ch = line[i]
        nxt = line[i + 1] if i + 1 < len(line) else ""

        if in_block_comment:
            if ch == "*" and nxt == "/":
                in_block_comment = False
                i += 2
                continue
            i += 1
            continue

        if in_single:
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == "'":
                in_single = False
            i += 1
            continue

        if in_double:
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == '"':
                in_double = False
            i += 1
            continue

        if in_backtick:
            if ch == "`":
                in_backtick = False
            i += 1
            continue

        if ch == "/" and nxt == "*":
            in_block_comment = True
            i += 2
            continue
        if ch == "/" and nxt == "/":
            break
        if ch == "'" and _starts_char_literal(line, i):
            in_single = True
            i += 1
            continue
        if ch == '"':
            in_double = True
            i += 1
            continue
        if ch == "`":
            in_backtick = True
            i += 1
            continue
        if ch == "{":
            delta += 1
            saw_open = True
        elif ch == "}":
            delta -= 1

        i += 1

    return delta, saw_open, in_block_comment


def _find_opening_brace_line(lines: list[str], start_idx: int, max_lookahead: int = 120) -> int | None:
    in_block_comment = False
    stop_idx = min(len(lines), start_idx + max_lookahead)

    for line_idx in range(start_idx, stop_idx):
        _, saw_open, in_block_comment = _brace_scan_line(lines[line_idx], in_block_comment)
        if saw_open:
            return line_idx
    return None


def _find_matching_brace_line(lines: list[str], open_idx: int) -> int | None:
    in_block_comment = False
    depth = 0
    seen_open = False

    for line_idx in range(open_idx, len(lines)):
        delta, saw_open, in_block_comment = _brace_scan_line(lines[line_idx], in_block_comment)
        if saw_open:
            seen_open = True
        if seen_open:
            depth += delta
            if depth == 0:
                return line_idx

    return None


def _expand_rust_leading_annotations(lines: list[str], start_idx: int) -> int:
    while start_idx > 0:
        candidate = lines[start_idx - 1].lstrip()
        if candidate.startswith("#[") or candidate.startswith("///") or candidate.startswith("//!"):
            start_idx -= 1
            continue
        break
    return start_idx


def _expand_go_leading_comments(lines: list[str], start_idx: int) -> int:
    while start_idx > 0:
        candidate = lines[start_idx - 1].lstrip()
        if candidate.startswith("//"):
            start_idx -= 1
            continue
        break
    return start_idx


def _extract_brace_function(lines: list[str], target_idx: int, suffix: str) -> tuple[int, int] | None:
    if suffix == ".rs":
        start_re = RUST_FN_RE
    elif suffix == ".go":
        start_re = GO_FN_RE
    else:
        start_re = GENERIC_BRACE_FN_RE

    for start_idx in range(min(target_idx, len(lines) - 1), -1, -1):
        if not start_re.match(lines[start_idx]):
            continue

        open_idx = _find_opening_brace_line(lines, start_idx)
        if open_idx is None:
            continue

        end_idx = _find_matching_brace_line(lines, open_idx)
        if end_idx is None:
            continue

        resolved_start_idx = start_idx
        if suffix == ".rs":
            resolved_start_idx = _expand_rust_leading_annotations(lines, start_idx)
        elif suffix == ".go":
            resolved_start_idx = _expand_go_leading_comments(lines, start_idx)

        if resolved_start_idx <= target_idx <= end_idx:
            return resolved_start_idx, end_idx

    return None


def _extract_python_function(lines: list[str], target_idx: int) -> tuple[int, int] | None:
    for start_idx in range(min(target_idx, len(lines) - 1), -1, -1):
        if not PY_FN_RE.match(lines[start_idx]):
            continue

        base_indent = _leading_spaces(lines[start_idx])
        end_idx = len(lines) - 1

        for line_idx in range(start_idx + 1, len(lines)):
            stripped = lines[line_idx].strip()
            if not stripped:
                continue

            indent = _leading_spaces(lines[line_idx])
            if indent <= base_indent:
                end_idx = line_idx - 1
                break

        decorator_start = start_idx
        while decorator_start > 0 and lines[decorator_start - 1].lstrip().startswith("@"):
            decorator_start -= 1

        if decorator_start <= target_idx <= end_idx:
            return decorator_start, end_idx

    return None


def extract_enclosing_function(lines: list[str], file_path: Path, line_no: int) -> tuple[int, int, list[str]] | None:
    if not lines or line_no < 1 or line_no > len(lines):
        return None

    target_idx = line_no - 1
    suffix = file_path.suffix.lower()

    if suffix == ".py":
        span = _extract_python_function(lines, target_idx)
    else:
        span = _extract_brace_function(lines, target_idx, suffix)

    if span is None:
        return None

    start_idx, end_idx = span
    return start_idx + 1, end_idx + 1, lines[start_idx : end_idx + 1]


def _build_function_block(
    finding: Finding,
    lines: list[str],
    rule_metadata_by_id: RuleMetadataById,
    *,
    index: int,
    total: int,
    review_placeholder: str,
    details: bool,
) -> str:
    rule_metadata = _resolve_rule_metadata(finding, rule_metadata_by_id)
    function_span = extract_enclosing_function(lines, finding.file_path, finding.line_no)

    if function_span is None:
        function_start = finding.line_no
        function_end = finding.line_no
        function_lines: list[str] = []
    else:
        function_start, function_end, function_lines = function_span

    auto_triage, auto_reason = _triage_finding(
        finding,
        lines,
        function_start,
        function_end,
        rule_metadata,
    )
    family, severity, status, languages, description = _summarize_rule_metadata(
        finding.rule_id,
        rule_metadata_by_id,
    )

    block_lines = [f"Finding {index}/{total}"]
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
                f"Function range: [{function_start}-{function_end}]",
                f"Auto triage: [{auto_triage}]",
                f"Auto triage note: {auto_reason}",
                f"False positive: [{review_placeholder}]",
                f"Original finding: {finding.raw_line}",
                "Function:",
            ]
        )
    else:
        block_lines.extend(
            [
                f"Source: {finding.file_path}:{finding.line_no}",
                f"Rule: [{finding.rule_id}]",
                f"Rule description: {description}",
                f"Auto triage note: {auto_reason}",
                "Function:",
            ]
        )

    if function_lines:
        for line in function_lines:
            block_lines.append(f"    {line}" if line else "")
    else:
        block_lines.append("    [FUNCTION_NOT_FOUND]")

    return "\n".join(block_lines) + "\n"


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
    block_lines = [f"Finding {index}/{total}"]
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
                "Function: [FILE_NOT_FOUND]",
            ]
        )
    else:
        block_lines.extend(
            [
                f"Source: {finding.file_path}:{finding.line_no}",
                f"Rule: [{finding.rule_id}]",
                f"Rule description: {description}",
                f"Auto triage note: {auto_reason}",
                "Function: [FILE_NOT_FOUND]",
            ]
        )
    return "\n".join(block_lines) + "\n"


def write_outputs(
    findings: list[Finding],
    input_path: Path,
    output_dir: Path,
    *,
    review_placeholder: str,
    details: bool,
) -> None:
    total_findings = len(findings)
    rule_metadata_by_id = _read_rule_metadata()
    cached_files: dict[Path, list[str]] = {}
    _ = input_path  # retained for signature parity and future metadata expansion

    output_dir.mkdir(parents=True, exist_ok=True)
    for txt_file in _iter_txt_files(output_dir):
        txt_file.unlink()

    for index, finding in enumerate(findings, start=1):
        output_path = output_dir / f"{index}.txt"
        if not finding.file_path.exists():
            output_path.write_text(
                _build_missing_file_block(
                    finding,
                    rule_metadata_by_id,
                    index=index,
                    total=total_findings,
                    review_placeholder=review_placeholder,
                    details=details,
                ),
                encoding="utf-8",
            )
            continue

        if finding.file_path not in cached_files:
            cached_files[finding.file_path] = load_lines(finding.file_path)

        output_path.write_text(
            _build_function_block(
                finding,
                cached_files[finding.file_path],
                rule_metadata_by_id,
                index=index,
                total=total_findings,
                review_placeholder=review_placeholder,
                details=details,
            ),
            encoding="utf-8",
        )


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
    output_dir = Path(args.output_dir).expanduser().resolve()

    findings = parse_findings(input_path)
    if not findings:
        raise SystemExit(f"no findings matched the expected format in {input_path}")

    write_outputs(
        findings,
        input_path,
        output_dir,
        review_placeholder=args.review_placeholder,
        details=args.details,
    )

    print(f"Wrote {len(findings)} finding blocks to {output_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
