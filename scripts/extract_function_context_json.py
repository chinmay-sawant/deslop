#!/usr/bin/env python3
"""Export deslop findings into structured JSON for the React visualizer."""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from extract_function_context import (
    ROOT,
    _read_rule_metadata,
    _resolve_rule_metadata,
    _summarize_rule_metadata,
    _triage_finding,
    extract_enclosing_function,
    infer_language,
    load_lines,
    parse_findings,
)

DEFAULT_INPUT = ROOT / "temp.txt"
DEFAULT_OUTPUT_DIR = ROOT / "frontend" / "public" / "findings"
DEFAULT_MAX_GRAPH_NODES = 160
DEFAULT_SHARD_SIZE = 500


@dataclass(frozen=True)
class DetailShard:
    key: str
    path: str
    start_id: int
    end_id: int
    count: int


SEVERITY_ORDER = {"error": 0, "warning": 1, "contextual": 2, "info": 3, "unknown": 4}
STATUS_ORDER = {"stable": 0, "experimental": 1, "research": 2, "unknown": 3}


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
        help="Directory where the visualizer dataset is written.",
    )
    parser.add_argument(
        "--details",
        action="store_true",
        help="Include verbose metadata-compatible fields in the exported records.",
    )
    parser.add_argument(
        "--shard-size",
        type=int,
        default=DEFAULT_SHARD_SIZE,
        help="Number of finding details per shard. Use 0 to inline details in dataset.json.",
    )
    parser.add_argument(
        "--max-graph-nodes",
        type=int,
        default=DEFAULT_MAX_GRAPH_NODES,
        help="Target number of nodes for the initial graph scope metadata.",
    )
    parser.add_argument(
        "--include-function-text",
        action="store_true",
        help="Include full function text in the output (inline or in detail shards).",
    )
    parser.add_argument(
        "--summary-only",
        action="store_true",
        help="Only write manifest/summary output without the main dataset files.",
    )
    return parser.parse_args()


def _json_dump(path: Path, payload: Any) -> None:
    path.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=False) + "\n",
        encoding="utf-8",
    )


def _normalize_text_block(lines: list[str]) -> str:
    text = "\n".join(lines).rstrip()
    return text


def _function_preview(text: str, *, max_lines: int = 6, max_chars: int = 420) -> str:
    if not text:
        return ""

    lines = text.splitlines()
    preview = "\n".join(lines[:max_lines]).strip()
    if len(preview) > max_chars:
        preview = preview[: max_chars - 1].rstrip() + "…"
    return preview


def _basename(path_text: str) -> str:
    return Path(path_text).name


def _derive_source_repo(file_path: Path) -> str:
    parts = file_path.parts
    if "real-repos" in parts:
        idx = parts.index("real-repos")
        if idx + 1 < len(parts):
            return parts[idx + 1]

    try:
        rel = file_path.relative_to(ROOT)
    except ValueError:
        return parts[0] if parts else "unknown"

    return rel.parts[0] if rel.parts else "unknown"


def _display_path(file_path: Path, source_repo: str) -> str:
    parts = list(file_path.parts)
    if "real-repos" in parts:
        idx = parts.index("real-repos")
        tail = parts[idx + 2 :]
        if tail:
            return "/".join(tail)

    try:
        return str(file_path.relative_to(ROOT))
    except ValueError:
        return str(file_path)



def _counter_entries(counter: Counter[str], *, limit: int | None = None) -> list[dict[str, Any]]:
    items = sorted(counter.items(), key=lambda item: (-item[1], item[0]))
    if limit is not None:
        items = items[:limit]
    return [{"key": key, "count": count} for key, count in items]



def _build_graph(records: list[dict[str, Any]], max_graph_nodes: int) -> dict[str, Any]:
    rule_counter: Counter[str] = Counter()
    file_counter: Counter[str] = Counter()
    pair_counter: Counter[tuple[str, str]] = Counter()

    for record in records:
        rule_counter[record["ruleId"]] += 1
        file_counter[record["sourcePath"]] += 1
        pair_counter[(record["ruleId"], record["sourcePath"])] += 1

    top_rules = [entry["key"] for entry in _counter_entries(rule_counter, limit=max(12, max_graph_nodes // 5))]
    top_files = [entry["key"] for entry in _counter_entries(file_counter, limit=max(24, max_graph_nodes // 2))]
    top_rule_set = set(top_rules)
    top_file_set = set(top_files)

    rule_nodes = [
        {
            "id": f"rule:{rule_id}",
            "nodeType": "rule",
            "label": rule_id,
            "count": rule_counter[rule_id],
        }
        for rule_id in top_rules
    ]
    file_nodes = [
        {
            "id": f"file:{path_text}",
            "nodeType": "file",
            "label": _basename(path_text),
            "count": file_counter[path_text],
            "sourcePath": path_text,
        }
        for path_text in top_files
    ]
    edges = [
        {
            "id": f"edge:{rule_id}:{path_text}",
            "source": f"rule:{rule_id}",
            "target": f"file:{path_text}",
            "count": count,
        }
        for (rule_id, path_text), count in sorted(pair_counter.items(), key=lambda item: (-item[1], item[0][0], item[0][1]))
        if rule_id in top_rule_set and path_text in top_file_set
    ]

    return {
        "nodes": rule_nodes + file_nodes,
        "edges": edges,
        "topRuleIds": top_rules,
        "topFilePaths": top_files,
    }



def build_dataset(
    input_path: Path,
    *,
    include_function_text: bool,
    details: bool,
    max_graph_nodes: int,
) -> tuple[dict[str, Any], dict[str, str]]:
    findings = parse_findings(input_path)
    if not findings:
        raise SystemExit(f"no findings matched the expected format in {input_path}")

    rule_metadata_by_id = _read_rule_metadata()
    cached_files: dict[Path, list[str]] = {}
    records: list[dict[str, Any]] = []
    detail_map: dict[str, str] = {}
    family_counter: Counter[str] = Counter()
    severity_counter: Counter[str] = Counter()
    status_counter: Counter[str] = Counter()
    language_counter: Counter[str] = Counter()
    repo_counter: Counter[str] = Counter()
    rule_counter: Counter[str] = Counter()
    file_counter: Counter[str] = Counter()
    auto_triage_counter: Counter[str] = Counter()
    missing_files = 0
    missing_functions = 0

    for index, finding in enumerate(findings, start=1):
        file_exists = finding.file_path.exists()
        lines: list[str] = []
        if file_exists:
            if finding.file_path not in cached_files:
                cached_files[finding.file_path] = load_lines(finding.file_path)
            lines = cached_files[finding.file_path]
        else:
            missing_files += 1

        function_span = extract_enclosing_function(lines, finding.file_path, finding.line_no) if lines else None
        if function_span is None:
            function_found = False
            function_start = None
            function_end = None
            function_text = ""
            missing_functions += 1
        else:
            function_found = True
            function_start, function_end, function_lines = function_span
            function_text = _normalize_text_block(function_lines)

        rule_metadata = _resolve_rule_metadata(finding, rule_metadata_by_id)
        family, severity, status, languages, description = _summarize_rule_metadata(
            finding.rule_id,
            rule_metadata_by_id,
        )
        auto_triage, auto_triage_note = _triage_finding(
            finding,
            lines,
            function_start or finding.line_no,
            function_end or finding.line_no,
            rule_metadata,
        )
        language = infer_language(finding.file_path) or "unknown"
        source_repo = _derive_source_repo(finding.file_path)
        display_path = _display_path(finding.file_path, source_repo)
        function_preview = _function_preview(function_text)
        record_id = index
        tag_set = {
            f"family:{family}",
            f"severity:{severity}",
            f"status:{status}",
            f"language:{language}",
            f"triage:{auto_triage.lower()}",
            f"repo:{source_repo}",
        }
        search_text = " ".join(
            part.lower()
            for part in [
                display_path,
                finding.file_path.name,
                finding.rule_id,
                finding.message,
                description,
                function_preview,
                source_repo,
                family,
                severity,
                status,
                auto_triage,
            ]
            if part
        )

        record = {
            "id": record_id,
            "sourcePath": str(finding.file_path),
            "sourceDisplayPath": display_path,
            "sourceFile": finding.file_path.name,
            "sourceRepo": source_repo,
            "line": finding.line_no,
            "ruleId": finding.rule_id,
            "ruleFamily": family,
            "ruleSeverity": severity,
            "ruleStatus": status,
            "ruleLanguages": languages,
            "ruleDescription": description,
            "message": finding.message,
            "autoTriage": auto_triage,
            "autoTriageNote": auto_triage_note,
            "functionFound": function_found,
            "functionStart": function_start,
            "functionEnd": function_end,
            "functionPreview": function_preview,
            "language": language,
            "tags": sorted(tag_set),
            "rawFinding": finding.raw_line,
            "fileExists": file_exists,
            "searchText": search_text,
        }

        if details:
            record["detailMode"] = "full"
            record["registryMetadata"] = {
                "family": family,
                "severity": severity,
                "status": status,
                "languages": languages,
            }

        if include_function_text:
            detail_map[str(record_id)] = function_text

        family_counter[family] += 1
        severity_counter[severity] += 1
        status_counter[status] += 1
        language_counter[language] += 1
        repo_counter[source_repo] += 1
        rule_counter[finding.rule_id] += 1
        file_counter[str(finding.file_path)] += 1
        auto_triage_counter[auto_triage] += 1
        records.append(record)

    records.sort(key=lambda item: item["id"])
    graph = _build_graph(records, max_graph_nodes)
    summary = {
        "totals": {
            "findings": len(records),
            "rules": len(rule_counter),
            "files": len(file_counter),
            "repos": len(repo_counter),
            "missingFiles": missing_files,
            "missingFunctions": missing_functions,
        },
        "counts": {
            "families": _counter_entries(family_counter),
            "severities": sorted(
                _counter_entries(severity_counter),
                key=lambda entry: (SEVERITY_ORDER.get(entry["key"], 99), entry["key"]),
            ),
            "statuses": sorted(
                _counter_entries(status_counter),
                key=lambda entry: (STATUS_ORDER.get(entry["key"], 99), entry["key"]),
            ),
            "languages": _counter_entries(language_counter),
            "repos": _counter_entries(repo_counter),
            "rules": _counter_entries(rule_counter),
            "files": _counter_entries(file_counter, limit=300),
            "autoTriage": _counter_entries(auto_triage_counter),
        },
        "topRules": _counter_entries(rule_counter, limit=12),
        "topFiles": _counter_entries(file_counter, limit=12),
        "topRepos": _counter_entries(repo_counter, limit=12),
    }

    dataset = {
        "version": 1,
        "inputFile": str(input_path),
        "records": records,
        "graph": graph,
        "summary": summary,
    }
    return dataset, detail_map



def write_dataset(
    dataset: dict[str, Any],
    detail_map: dict[str, str],
    output_dir: Path,
    *,
    input_path: Path,
    include_function_text: bool,
    shard_size: int,
    summary_only: bool,
) -> dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)

    for file_path in output_dir.glob("*.json"):
        file_path.unlink()

    records = dataset["records"]
    detail_shards: list[DetailShard] = []
    record_to_shard: dict[int, str] = {}

    manifest: dict[str, Any] = {
        "version": dataset["version"],
        "generatedFrom": str(input_path),
        "totalFindings": len(records),
        "summaryOnly": summary_only,
        "includesFunctionText": include_function_text,
        "summary": dataset["summary"],
        "filters": {
            key: [entry["key"] for entry in dataset["summary"]["counts"][key]]
            for key in ["families", "severities", "statuses", "languages", "repos", "rules"]
        },
        "graph": {
            "nodeCount": len(dataset["graph"]["nodes"]),
            "edgeCount": len(dataset["graph"]["edges"]),
            "topRuleIds": dataset["graph"]["topRuleIds"],
            "topFilePaths": dataset["graph"]["topFilePaths"],
        },
        "files": {},
    }

    if summary_only:
        _json_dump(output_dir / "manifest.json", manifest)
        return manifest

    if include_function_text and shard_size > 0:
        detail_items = list(detail_map.items())
        for start in range(0, len(detail_items), shard_size):
            batch = detail_items[start : start + shard_size]
            start_id = int(batch[0][0])
            end_id = int(batch[-1][0])
            shard_key = f"detail-{start_id:05d}-{end_id:05d}"
            shard_name = f"{shard_key}.json"
            shard_payload = {
                "version": dataset["version"],
                "details": [{"id": int(finding_id), "functionText": function_text} for finding_id, function_text in batch],
            }
            _json_dump(output_dir / shard_name, shard_payload)
            detail_shards.append(
                DetailShard(
                    key=shard_key,
                    path=f"findings/{shard_name}",
                    start_id=start_id,
                    end_id=end_id,
                    count=len(batch),
                )
            )
            for finding_id, _ in batch:
                record_to_shard[int(finding_id)] = shard_key

    export_records: list[dict[str, Any]] = []
    for record in records:
        exported = dict(record)
        if include_function_text:
            if shard_size > 0:
                exported["detailShard"] = record_to_shard.get(record["id"])
            else:
                exported["functionText"] = detail_map.get(str(record["id"]), "")
        export_records.append(exported)

    core_payload = {
        "version": dataset["version"],
        "records": export_records,
        "graph": dataset["graph"],
        "summary": dataset["summary"],
    }

    if include_function_text and shard_size <= 0:
        dataset_name = "dataset.json"
    else:
        dataset_name = "findings-core.json"

    _json_dump(output_dir / dataset_name, core_payload)

    manifest["files"] = {
        "dataset": f"findings/{dataset_name}",
        "detailShards": [asdict(shard) for shard in detail_shards],
    }

    _json_dump(output_dir / "manifest.json", manifest)
    return manifest



def main() -> int:
    args = parse_args()
    input_path = Path(args.input_file).expanduser().resolve()
    output_dir = Path(args.output_dir).expanduser().resolve()

    dataset, detail_map = build_dataset(
        input_path,
        include_function_text=args.include_function_text,
        details=args.details,
        max_graph_nodes=max(24, args.max_graph_nodes),
    )
    manifest = write_dataset(
        dataset,
        detail_map,
        output_dir,
        input_path=input_path,
        include_function_text=args.include_function_text,
        shard_size=max(0, args.shard_size),
        summary_only=args.summary_only,
    )

    print(
        f"Wrote manifest for {manifest['totalFindings']} findings to {output_dir / 'manifest.json'}"
    )
    dataset_path = manifest.get("files", {}).get("dataset")
    if dataset_path:
        print(f"Main dataset: {output_dir / Path(dataset_path).name}")
    if manifest.get("files", {}).get("detailShards"):
        print(f"Detail shards: {len(manifest['files']['detailShards'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
