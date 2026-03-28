#!/usr/bin/env python3
import re
from collections import Counter, defaultdict


def parse_findings(path):
    findings = []
    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            if line.startswith('  - '):
                findings.append(line[4:].rstrip('\n'))
    return findings


def extract_category(finding):
    m = re.search(r"\[([^\]]+)\]$", finding)
    return m.group(1) if m else 'unknown'


def extract_file(finding):
    # finding format: /abs/path/to/file:LINE rest... [category]
    # split on first ':' to get path
    parts = finding.split(':', 1)
    return parts[0] if parts else 'unknown'


def summarize(a, b):
    set_a = set(a)
    set_b = set(b)
    removed = sorted(set_a - set_b)
    added = sorted(set_b - set_a)
    common = sorted(set_a & set_b)

    def by_category(items):
        c = Counter()
        for it in items:
            c[extract_category(it)] += 1
        return c

    def by_file(items, top_n=8):
        c = Counter()
        for it in items:
            c[extract_file(it)] += 1
        return c.most_common(top_n)

    print(f"Verified findings: {len(a)}")
    print(f"Temp findings:     {len(b)}")
    print(f"Removed: {len(removed)}")
    print(f"Added:   {len(added)}")
    print(f"Unchanged: {len(common)}")
    print()

    if removed:
        print("Top removed categories:")
        for cat, cnt in by_category(removed).most_common(8):
            print(f"  - {cat}: {cnt}")
        print("Top files with removed findings:")
        for path, cnt in by_file(removed):
            print(f"  - {path}: {cnt}")
        print()

    if added:
        print("Top added categories:")
        for cat, cnt in by_category(added).most_common(8):
            print(f"  - {cat}: {cnt}")
        print("Top files with added findings:")
        for path, cnt in by_file(added):
            print(f"  - {path}: {cnt}")
        print()

    # Print concise samples
    if removed:
        print("Sample removed findings (up to 8):")
        for it in removed[:8]:
            print(f"  - {it}")
        print()

    if added:
        print("Sample added findings (up to 8):")
        for it in added[:8]:
            print(f"  - {it}")
        print()


if __name__ == '__main__':
    a = parse_findings('verified_gopdfsuit_results.txt')
    b = parse_findings('temp_gopdfsuit.txt')
    summarize(a, b)
