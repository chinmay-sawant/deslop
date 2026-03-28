#!/usr/bin/env python3
"""compare_findings.py — diff two deslop result files.

Usage:
    python scripts/compare_findings.py <baseline_file> <latest_file> [--output-dir reports]

Matching strategy (v2):
  - Strips absolute repo-root prefixes so './internal/foo.go' and
    '/home/user/repo/internal/foo.go' compare equal.
  - Ignores line-number differences: (norm_path, message, category) is the
    identity of a finding; same finding at a shifted line is still "unchanged".
  - Reports four buckets: Unchanged, Moved (line-number only shifted),
    Removed (gone), Added (new).
  - Writes a Markdown report with table layout to <output-dir>/<stem>_<timestamp>.md
"""
import argparse
import os
import re
from collections import Counter, defaultdict
from datetime import datetime
from pathlib import Path

_FINDING_RE = re.compile(r'^(.*?):(\d+)\s+(.*?)\s+\[([^\]]+)\]$')


def parse_findings(path):
    findings = []
    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            if line.startswith('  - '):
                findings.append(line[4:].rstrip('\n'))
    return findings


def parse_finding(raw):
    """Return (file_path, line_no, message, category)."""
    m = _FINDING_RE.match(raw)
    if m:
        return m.group(1), m.group(2), m.group(3).strip(), m.group(4)
    return raw, '?', raw, 'unknown'


def _common_abs_prefix(findings):
    """Return the longest common directory prefix of all absolute paths."""
    abs_paths = []
    for raw in findings:
        fpath = parse_finding(raw)[0]
        if fpath.startswith('/'):
            abs_paths.append(fpath)
    if not abs_paths:
        return None
    common = abs_paths[0]
    for p in abs_paths[1:]:
        while common and not p.startswith(common):
            common = common.rsplit('/', 1)[0]
    return (common + '/') if common else None


def normalize_path(path, strip_prefixes):
    for prefix in strip_prefixes:
        if path.startswith(prefix):
            path = path[len(prefix):]
            break
    return path.lstrip('./')


def extract_category(raw):
    m = re.search(r'\[([^\]]+)\]$', raw)
    return m.group(1) if m else 'unknown'


def _all_cats(items):
    return Counter(extract_category(i) for i in items).most_common()


def _all_files(items, strip):
    c = Counter()
    for raw in items:
        fpath = parse_finding(raw)[0]
        c[normalize_path(fpath, strip)] += 1
    return c.most_common()


def _md_table(headers, rows):
    """Return a GFM Markdown table string."""
    lines = []
    lines.append('| ' + ' | '.join(headers) + ' |')
    lines.append('| ' + ' | '.join('---' for _ in headers) + ' |')
    for row in rows:
        lines.append('| ' + ' | '.join(str(c) for c in row) + ' |')
    return '\n'.join(lines)


def _build_report(baseline_path, latest_path, results, strip, timestamp):
    """Return a full Markdown report string."""
    unchanged_raws = results['unchanged']
    moved_raws = results['moved']
    removed_raws = results['removed']
    added_raws = results['added']
    a_count = results['baseline_total']
    b_count = results['latest_total']

    lines = []
    lines.append(f"# Deslop Findings Comparison Report")
    lines.append(f"")
    lines.append(f"**Generated:** {timestamp}")
    lines.append(f"**Baseline:** `{baseline_path}`")
    lines.append(f"**Latest:**   `{latest_path}`")
    if strip:
        lines.append(f"**Stripped path prefixes:** {strip}")
    lines.append("")

    # Summary table
    lines.append("## Summary")
    lines.append("")
    lines.append(_md_table(
        ["Metric", "Count"],
        [
            ["Baseline findings", a_count],
            ["Latest findings", b_count],
            ["Net change", f"{b_count - a_count:+d}"],
            ["Unchanged (same finding, same line)", len(unchanged_raws)],
            ["Moved (same finding, line shifted)", len(moved_raws)],
            ["Removed (finding gone in latest)", len(removed_raws)],
            ["Added (new finding in latest)", len(added_raws)],
        ]
    ))
    lines.append("")

    # Moved findings
    if moved_raws:
        lines.append(f"## Moved Findings ({len(moved_raws)} total)")
        lines.append("")
        lines.append("*Same issue detected at a different line number.*")
        lines.append("")
        rows = []
        for raw_a, raw_b in moved_raws:
            fpath, la, msg, cat = parse_finding(raw_a)
            _, lb, _, _ = parse_finding(raw_b)
            norm = normalize_path(fpath, strip)
            rows.append([norm, cat, msg[:80], la, lb])
        lines.append(_md_table(["File", "Category", "Message", "Baseline Line", "Latest Line"], rows))
        lines.append("")

    # Removed findings
    if removed_raws:
        lines.append(f"## Removed Findings ({len(removed_raws)} total)")
        lines.append("")
        lines.append("### By Category")
        lines.append("")
        lines.append(_md_table(
            ["Category", "Count"],
            _all_cats(removed_raws)
        ))
        lines.append("")
        lines.append("### By File")
        lines.append("")
        lines.append(_md_table(
            ["File", "Count"],
            _all_files(removed_raws, strip)
        ))
        lines.append("")
        lines.append("### All Removed Findings")
        lines.append("")
        rows = []
        for raw in removed_raws:
            fpath, line_no, msg, cat = parse_finding(raw)
            rows.append([normalize_path(fpath, strip), line_no, cat, msg])
        lines.append(_md_table(["File", "Line", "Category", "Message"], rows))
        lines.append("")

    # Added findings
    if added_raws:
        lines.append(f"## Added Findings ({len(added_raws)} total)")
        lines.append("")
        lines.append("### By Category")
        lines.append("")
        lines.append(_md_table(
            ["Category", "Count"],
            _all_cats(added_raws)
        ))
        lines.append("")
        lines.append("### By File")
        lines.append("")
        lines.append(_md_table(
            ["File", "Count"],
            _all_files(added_raws, strip)
        ))
        lines.append("")
        lines.append("### All Added Findings")
        lines.append("")
        rows = []
        for raw in added_raws:
            fpath, line_no, msg, cat = parse_finding(raw)
            rows.append([normalize_path(fpath, strip), line_no, cat, msg])
        lines.append(_md_table(["File", "Line", "Category", "Message"], rows))
        lines.append("")

    return '\n'.join(lines)


def summarize(a, b, baseline_path, latest_path, output_dir):
    strip = [p for p in [_common_abs_prefix(a), _common_abs_prefix(b)] if p]

    def make_key(raw):
        fpath, _line, msg, cat = parse_finding(raw)
        return (normalize_path(fpath, strip), msg, cat)

    # Group raw findings by key for each file
    a_by_key = defaultdict(list)
    for raw in a:
        a_by_key[make_key(raw)].append(raw)

    b_by_key = defaultdict(list)
    for raw in b:
        b_by_key[make_key(raw)].append(raw)

    all_keys = set(a_by_key) | set(b_by_key)

    unchanged_raws, moved_raws, removed_raws, added_raws = [], [], [], []

    for k in sorted(all_keys):
        la, lb = a_by_key[k], b_by_key[k]
        overlap = min(len(la), len(lb))

        # Findings that exist in both — check if line numbers shifted
        for i in range(overlap):
            raw_a, raw_b = la[i], lb[i]
            line_a = parse_finding(raw_a)[1]
            line_b = parse_finding(raw_b)[1]
            if line_a == line_b:
                unchanged_raws.append(raw_a)
            else:
                moved_raws.append((raw_a, raw_b))

        # Surplus in a → truly removed
        removed_raws.extend(la[overlap:])
        # Surplus in b → truly added
        added_raws.extend(lb[overlap:])

    results = {
        'baseline_total': len(a),
        'latest_total': len(b),
        'unchanged': unchanged_raws,
        'moved': moved_raws,
        'removed': removed_raws,
        'added': added_raws,
    }

    # ── terminal summary ──────────────────────────────────────────────────────
    print(f"Baseline findings : {len(a)}")
    print(f"Latest findings   : {len(b)}")
    print(f"Net change        : {len(b) - len(a):+d}")
    print()
    print(f"  Unchanged (same finding, same line)      : {len(unchanged_raws)}")
    print(f"  Moved     (same finding, line shifted)   : {len(moved_raws)}")
    print(f"  Removed   (finding gone in latest scan)  : {len(removed_raws)}")
    print(f"  Added     (new finding in latest scan)   : {len(added_raws)}")
    if strip:
        print(f"\nNote: stripped path prefixes: {strip}")
    print()

    if moved_raws:
        print(f"Moved findings — {len(moved_raws)} total (same issue, line number shifted):")
        for raw_a, raw_b in moved_raws:
            _, la_line, _, cat = parse_finding(raw_a)
            _, lb_line, _, _   = parse_finding(raw_b)
            norm = normalize_path(parse_finding(raw_a)[0], strip)
            print(f"  [{cat}] {norm}  line {la_line} → {lb_line}")
        print()

    if removed_raws:
        print(f"Removed categories ({len(removed_raws)} total):")
        for cat, cnt in _all_cats(removed_raws):
            print(f"  - {cat}: {cnt}")
        print("Files with removed findings:")
        for fp, cnt in _all_files(removed_raws, strip):
            print(f"  - {fp}: {cnt}")
        print()
        print("All removed findings:")
        for it in removed_raws:
            print(f"  - {it}")
        print()

    if added_raws:
        print(f"Added categories ({len(added_raws)} total):")
        for cat, cnt in _all_cats(added_raws):
            print(f"  - {cat}: {cnt}")
        print("Files with added findings:")
        for fp, cnt in _all_files(added_raws, strip):
            print(f"  - {fp}: {cnt}")
        print()
        print("All added findings:")
        for it in added_raws:
            print(f"  - {it}")
        print()

    # ── markdown report ───────────────────────────────────────────────────────
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    baseline_stem = Path(baseline_path).stem
    latest_stem = Path(latest_path).stem
    report_name = f"{baseline_stem}_vs_{latest_stem}_{timestamp}.md"

    out_dir = Path(output_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    report_path = out_dir / report_name

    ts_display = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
    md = _build_report(baseline_path, latest_path, results, strip, ts_display)
    report_path.write_text(md, encoding='utf-8')

    print(f"Markdown report written to: {report_path}")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description='Diff two deslop result files and generate a Markdown report.'
    )
    parser.add_argument('baseline', help='Baseline (verified) results file')
    parser.add_argument('latest', help='Latest (new scan) results file')
    parser.add_argument(
        '--output-dir', default='reports',
        help='Directory to write the Markdown report into (default: reports)'
    )
    args = parser.parse_args()

    a = parse_findings(args.baseline)
    b = parse_findings(args.latest)
    summarize(a, b, args.baseline, args.latest, args.output_dir)
