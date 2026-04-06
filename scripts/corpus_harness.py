#!/usr/bin/env python3
"""Run and validate deslop corpus targets from a central manifest."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

import compare_findings


ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "corpus" / "manifest.json"
PROMOTION_TEMPLATE_PATH = ROOT / "reports" / "corpus" / "promotion-note-template.md"
VALID_LANGUAGES = {"go", "python", "rust", "common"}
VALID_AVAILABILITY = {"active", "planned"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    validate_parser = subparsers.add_parser("validate", help="Validate the manifest structure")
    validate_parser.add_argument(
        "--strict-paths",
        action="store_true",
        help="Fail when active targets resolve to missing repository paths",
    )

    list_parser = subparsers.add_parser("list", help="List configured corpus targets")
    list_parser.add_argument(
        "--include-planned",
        action="store_true",
        help="Include planned slots as well as active targets",
    )

    run_parser = subparsers.add_parser("run", help="Run scan and or bench against corpus targets")
    run_parser.add_argument(
        "--target",
        action="append",
        default=[],
        help="Run only the named corpus target. Repeat to select multiple targets.",
    )
    run_parser.add_argument(
        "--include-planned",
        action="store_true",
        help="Allow explicitly selected planned slots to run if they later gain a path.",
    )
    run_parser.add_argument(
        "--strict-paths",
        action="store_true",
        help="Fail when a selected target path is missing instead of skipping it.",
    )
    run_parser.add_argument(
        "--repo-root",
        help="Optional root directory used to resolve relative corpus target paths.",
    )
    run_parser.add_argument(
        "--scan",
        action="store_true",
        help="Run scans for the selected targets.",
    )
    run_parser.add_argument(
        "--bench",
        action="store_true",
        help="Run benchmarks for the selected targets.",
    )
    run_parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print commands and resolved paths without executing them.",
    )
    run_parser.add_argument(
        "--update-baseline",
        action="store_true",
        help="Replace the configured scan baseline with the latest scan output.",
    )
    run_parser.add_argument(
        "--no-compare",
        action="store_true",
        help="Skip finding-diff report generation even when a baseline exists.",
    )

    args = parser.parse_args()
    manifest = load_manifest()
    validate_manifest(manifest, strict_paths=getattr(args, "strict_paths", False), repo_root=getattr(args, "repo_root", None))

    if args.command == "validate":
        print("corpus manifest is valid")
        return 0

    if args.command == "list":
        print(render_target_list(manifest["targets"], include_planned=args.include_planned))
        return 0

    should_scan = args.scan or not args.bench
    should_bench = args.bench
    selected = select_targets(
        manifest["targets"],
        requested_ids=args.target,
        include_planned=args.include_planned,
    )
    return run_targets(
        selected,
        repo_root=args.repo_root,
        should_scan=should_scan,
        should_bench=should_bench,
        strict_paths=args.strict_paths,
        dry_run=args.dry_run,
        compare=not args.no_compare,
        update_baseline=args.update_baseline,
    )


def load_manifest() -> dict:
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


def validate_manifest(manifest: dict, *, strict_paths: bool, repo_root: str | None) -> None:
    if manifest.get("version") != 1:
        raise SystemExit("corpus manifest version must be 1")

    targets = manifest.get("targets")
    if not isinstance(targets, list) or not targets:
        raise SystemExit("corpus manifest must define a non-empty targets list")

    ids = set()
    for target in targets:
        validate_target(target)
        target_id = target["id"]
        if target_id in ids:
            raise SystemExit(f"duplicate corpus target id: {target_id}")
        ids.add(target_id)

        if strict_paths and target["availability"] == "active":
            repo_path = resolve_repo_path(target, repo_root=repo_root)
            if repo_path is None or not repo_path.exists():
                raise SystemExit(f"active corpus target path is missing: {target_id}")


def validate_target(target: dict) -> None:
    required = {
        "id",
        "language",
        "category",
        "availability",
        "description",
        "relative_path",
        "scan_baseline",
        "reports_dir",
        "promotion_note",
        "scan",
        "bench",
    }
    missing = sorted(required - set(target))
    if missing:
        raise SystemExit(f"corpus target {target.get('id', '<unknown>')} is missing keys: {missing}")

    if target["language"] not in VALID_LANGUAGES:
        raise SystemExit(f"invalid target language for {target['id']}: {target['language']}")
    if target["availability"] not in VALID_AVAILABILITY:
        raise SystemExit(f"invalid target availability for {target['id']}: {target['availability']}")
    if not target["description"].strip():
        raise SystemExit(f"corpus target {target['id']} must include a description")

    for key in ("reports_dir", "promotion_note", "scan_baseline"):
        value = target[key]
        if value is not None and Path(value).is_absolute():
            raise SystemExit(f"corpus target {target['id']} must use repo-relative {key}")

    if target["relative_path"] is not None and Path(target["relative_path"]).is_absolute():
        raise SystemExit(f"corpus target {target['id']} must use a repo-relative path")

    scan = target["scan"]
    bench = target["bench"]
    if not isinstance(scan, dict) or not isinstance(bench, dict):
        raise SystemExit(f"corpus target {target['id']} must define scan and bench objects")
    if not isinstance(scan.get("enable_semantic"), bool):
        raise SystemExit(f"corpus target {target['id']} scan.enable_semantic must be a boolean")
    if not isinstance(bench.get("warmups"), int) or bench["warmups"] < 0:
        raise SystemExit(f"corpus target {target['id']} bench.warmups must be a non-negative integer")
    if not isinstance(bench.get("repeats"), int) or bench["repeats"] <= 0:
        raise SystemExit(f"corpus target {target['id']} bench.repeats must be a positive integer")


def render_target_list(targets: list[dict], *, include_planned: bool) -> str:
    lines = []
    for target in targets:
        if target["availability"] != "active" and not include_planned:
            continue
        path_hint = target["relative_path"] or "<not assigned>"
        lines.append(
            f"- {target['id']} [{target['language']} / {target['availability']} / {target['category']}]"
        )
        lines.append(f"  {target['description']}")
        lines.append(f"  path: {path_hint}")
    return "\n".join(lines)


def select_targets(targets: list[dict], *, requested_ids: list[str], include_planned: bool) -> list[dict]:
    by_id = {target["id"]: target for target in targets}
    if requested_ids:
        missing = [target_id for target_id in requested_ids if target_id not in by_id]
        if missing:
            raise SystemExit(f"unknown corpus targets: {', '.join(missing)}")
        selected = [by_id[target_id] for target_id in requested_ids]
    else:
        selected = [target for target in targets if target["availability"] == "active"]

    if not include_planned:
        selected = [target for target in selected if target["availability"] == "active"]

    if not selected:
        raise SystemExit("no corpus targets selected")
    return selected


def run_targets(
    targets: list[dict],
    *,
    repo_root: str | None,
    should_scan: bool,
    should_bench: bool,
    strict_paths: bool,
    dry_run: bool,
    compare: bool,
    update_baseline: bool,
) -> int:
    failures = 0
    ran = 0
    skipped = 0

    for target in targets:
        repo_path = resolve_repo_path(target, repo_root=repo_root)
        if repo_path is None or not repo_path.exists():
            if strict_paths:
                print(f"missing repository path for {target['id']}", file=sys.stderr)
                failures += 1
                continue
            print(f"skipping {target['id']}: repository path is not available")
            skipped += 1
            continue

        if dry_run:
            print(f"{target['id']}: {repo_path}")
            if should_scan:
                print("  scan  " + shell_join(build_scan_command(target, repo_path)))
            if should_bench:
                print("  bench " + shell_join(build_bench_command(target, repo_path)))
            ran += 1
            continue

        ensure_promotion_note(target, repo_path)

        try:
            if should_scan:
                run_scan(target, repo_path, compare=compare, update_baseline=update_baseline)
            if should_bench:
                run_bench(target, repo_path)
            ran += 1
        except subprocess.CalledProcessError as error:
            failures += 1
            print(f"{target['id']} failed with exit code {error.returncode}", file=sys.stderr)

    print(f"corpus run summary: ran={ran} skipped={skipped} failed={failures}")
    return 1 if failures else 0


def resolve_repo_path(target: dict, *, repo_root: str | None) -> Path | None:
    relative_path = target["relative_path"]
    if relative_path is None:
        return

    base = Path(repo_root).resolve() if repo_root else ROOT
    return (base / relative_path).resolve()


def build_scan_command(target: dict, repo_path: Path) -> list[str]:
    command = ["cargo", "run", "--quiet", "--", "scan", "--no-fail"]
    if target["scan"]["enable_semantic"]:
        command.append("--enable-semantic")
    command.append(str(repo_path))
    return command


def build_bench_command(target: dict, repo_path: Path) -> list[str]:
    bench = target["bench"]
    command = [
        "cargo",
        "run",
        "--quiet",
        "--",
        "bench",
        "--json",
        "--warmups",
        str(bench["warmups"]),
        "--repeats",
        str(bench["repeats"]),
    ]
    if target["scan"]["enable_semantic"]:
        command.append("--enable-semantic")
    command.append(str(repo_path))
    return command


def run_scan(target: dict, repo_path: Path, *, compare: bool, update_baseline: bool) -> None:
    output_dir = ROOT / target["reports_dir"]
    output_dir.mkdir(parents=True, exist_ok=True)

    latest_scan = output_dir / "latest-scan.txt"
    latest_stderr = output_dir / "latest-scan.stderr.txt"
    result = subprocess.run(
        build_scan_command(target, repo_path),
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    latest_scan.write_text(result.stdout, encoding="utf-8")
    if result.stderr.strip():
        latest_stderr.write_text(result.stderr, encoding="utf-8")
    elif latest_stderr.exists():
        latest_stderr.unlink()

    baseline_path = ROOT / target["scan_baseline"] if target["scan_baseline"] else None
    if update_baseline and baseline_path is not None:
        baseline_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(latest_scan, baseline_path)

    if compare and baseline_path is not None and baseline_path.exists():
        comparison_dir = output_dir / "comparisons"
        baseline = compare_findings.parse_findings(baseline_path)
        latest = compare_findings.parse_findings(latest_scan)
        compare_findings.summarize(
            baseline,
            latest,
            str(baseline_path),
            str(latest_scan),
            str(comparison_dir),
        )


def run_bench(target: dict, repo_path: Path) -> None:
    output_dir = ROOT / target["reports_dir"]
    output_dir.mkdir(parents=True, exist_ok=True)

    latest_bench = output_dir / "latest-bench.json"
    latest_stderr = output_dir / "latest-bench.stderr.txt"
    result = subprocess.run(
        build_bench_command(target, repo_path),
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    latest_bench.write_text(result.stdout, encoding="utf-8")
    if result.stderr.strip():
        latest_stderr.write_text(result.stderr, encoding="utf-8")
    elif latest_stderr.exists():
        latest_stderr.unlink()


def ensure_promotion_note(target: dict, repo_path: Path) -> None:
    note_path = ROOT / target["promotion_note"]
    if note_path.exists():
        return

    note_path.parent.mkdir(parents=True, exist_ok=True)
    template = PROMOTION_TEMPLATE_PATH.read_text(encoding="utf-8")
    content = (
        template.replace("{{target_id}}", target["id"])
        .replace("{{language}}", target["language"])
        .replace("{{category}}", target["category"])
        .replace("{{availability}}", target["availability"])
        .replace("{{repo_path}}", str(repo_path))
    )
    note_path.write_text(content, encoding="utf-8")


def shell_join(parts: list[str]) -> str:
    return " ".join(json.dumps(part) if " " in part else part for part in parts)


if __name__ == "__main__":
    sys.exit(main())
