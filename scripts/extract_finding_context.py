#!/usr/bin/env python3
"""Expand deslop findings into code-context review blocks.

This script reads a findings file such as ``temp_gopdfsuit.txt`` and writes a
single consolidated text file to ``scripts/temp.txt``. Each finding block
includes the original finding, the exact extracted line range, a 10-line
context window above and below the flagged line, and a placeholder review field
for marking false positives during later triage.
"""

from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_INPUT = ROOT / "temp_gopdfsuit.txt"
DEFAULT_OUTPUT = Path(__file__).resolve().parent / "temp.txt"
FINDING_RE = re.compile(r"^\s*-\s+(.*?):(\d+)\s+(.*?)\s+\[([^\]]+)\]\s*$")


@dataclass(frozen=True)
class Finding:
    file_path: Path
    line_no: int
    message: str
    rule_id: str
    raw_line: str


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
        default=10,
        help="How many lines to include above and below the flagged line.",
    )
    parser.add_argument(
        "--review-placeholder",
        default="REVIEW_NEEDED",
        help="Initial value written into the false-positive review field.",
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


def build_context_block(
    finding: Finding,
    lines: list[str],
    *,
    context: int,
    index: int,
    total: int,
    review_placeholder: str,
) -> str:
    start_line = max(1, finding.line_no - context)
    end_line = min(len(lines), finding.line_no + context)
    width = max(4, len(str(end_line)))
    auto_triage, auto_reason = triage_finding(finding, lines, start_line, end_line)

    block_lines = [
        "=" * 100,
        f"Finding {index}/{total}",
        f"Source: {finding.file_path}:{finding.line_no}",
        f"Rule: [{finding.rule_id}]",
        f"Message: {finding.message}",
        f"Suspect range: [{start_line}-{end_line}]",
        f"Auto triage: [{auto_triage}]",
        f"Auto triage note: {auto_reason}",
        f"False positive: [{review_placeholder}]",
        f"Original finding: {finding.raw_line}",
        "Code:",
    ]

    for line_number in range(start_line, end_line + 1):
        marker = ">>" if line_number == finding.line_no else "  "
        code_line = lines[line_number - 1]
        block_lines.append(f"{marker} {line_number:>{width}} | {code_line}")

    block_lines.append("")
    return "\n".join(block_lines)


def build_missing_file_block(
    finding: Finding,
    *,
    index: int,
    total: int,
    review_placeholder: str,
) -> str:
    auto_triage, auto_reason = triage_finding(finding, [], finding.line_no, finding.line_no)
    return "\n".join(
        [
            "=" * 100,
            f"Finding {index}/{total}",
            f"Source: {finding.file_path}:{finding.line_no}",
            f"Rule: [{finding.rule_id}]",
            f"Message: {finding.message}",
            f"Auto triage: [{auto_triage}]",
            f"Auto triage note: {auto_reason}",
            f"False positive: [{review_placeholder}]",
            f"Original finding: {finding.raw_line}",
            "Code: [FILE_NOT_FOUND]",
            "",
        ]
    )


def write_output(
    findings: list[Finding],
    input_path: Path,
    output_path: Path,
    *,
    context: int,
    review_placeholder: str,
) -> None:
    cached_files: dict[Path, list[str]] = {}
    blocks = [
        f"Input file: {input_path}",
        f"Output file: {output_path}",
        f"Total findings parsed: {len(findings)}",
        f"Context window: +/- {context} lines",
        "",
    ]

    for index, finding in enumerate(findings, start=1):
        if not finding.file_path.exists():
            blocks.append(
                build_missing_file_block(
                    finding,
                    index=index,
                    total=len(findings),
                    review_placeholder=review_placeholder,
                )
            )
            continue

        if finding.file_path not in cached_files:
            cached_files[finding.file_path] = load_lines(finding.file_path)

        blocks.append(
            build_context_block(
                finding,
                cached_files[finding.file_path],
                context=context,
                index=index,
                total=len(findings),
                review_placeholder=review_placeholder,
            )
        )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    if output_path.exists():
        output_path.unlink()
    output_path.write_text("\n".join(blocks), encoding="utf-8")


def triage_finding(
    finding: Finding,
    lines: list[str],
    start_line: int,
    end_line: int,
) -> tuple[str, str]:
    style_rules = {
        "comment_style_tutorial",
        "over_abstracted_wrapper",
        "option_bag_model",
        "overlong_name",
        "public_api_missing_type_hints",
        "public_any_type_leak",
        "python_public_api_any_contract",
        "weak_typing",
        "redundant_return_none",
        "variadic_public_api",
        "tight_module_coupling",
    }
    perf_rules = {
        "slice_append_without_prealloc_known_bound",
        "slice_grow_without_cap_hint",
        "fmt_hot_path",
        "three_index_slice_for_append_safety",
        "binary_read_for_single_field",
        "full_dataset_load",
        "regexp_compile_in_hot_path",
        "map_growth_without_size_hint",
        "sprintf_for_simple_string_format",
        "bytes_buffer_without_grow_known_bound",
        "filter_then_count_then_iterate",
        "string_concat_in_loop",
        "strings_builder_without_grow_known_bound",
    }
    risk_rules = {
        "weak_crypto",
        "error_detail_leaked_to_client",
        "error_logged_and_returned",
    }

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

    if finding.rule_id in style_rules:
        return (
            "LIKELY_SUBJECTIVE",
            "This rule is mostly style or API-shape guidance, so whether it matters depends on project conventions.",
        )

    if finding.rule_id in perf_rules:
        return (
            "CONTEXT_DEPENDENT",
            "This looks like a performance-focused suggestion; confirm with hot-path context or profiling before treating it as actionable.",
        )

    if finding.rule_id in risk_rules:
        return (
            "LIKELY_REAL",
            "This category usually maps to runtime, security, or user-visible behavior rather than a style preference.",
        )

    return (
        "REVIEW_NEEDED",
        "No safe automatic classification was inferred from the local code context alone.",
    )


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
    )

    print(f"Wrote {len(findings)} finding blocks to {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
