# Chunk Snippet Analysis Workflow (6 Subagents)

## Objective

Manually triage every finding in chunk files under:

`/home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks`

Read each chunk file directly, review each finding block, and classify each finding as exactly one of:

- `true_positive`
- `false_positive`

Produce a deterministic CSV with one row per finding.

## Hard Constraints

- Read `Chunk_*.txt` files directly. Do not use Python triage/classifier scripts.
- Use exactly 6 subagents, each with a disjoint partition.
- Each finding gets exactly one decision.
- Allowed decision labels are only `true_positive` and `false_positive`.
- Output must be machine-readable CSV with stable header.

## Current Dataset Snapshot

At the time of this update:

- Directory: `/home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks`
- File pattern: `Chunk_<start>_<end>.txt`
- Chunk files: `239`
- Findings: `5952`
- Last file: `Chunk_5951_5952.txt`

Recompute before every new run (do not assume static counts).

## Finding Block Format

Each chunk contains finding blocks separated by:

`====================================================================================================`

Each block includes:

- `Finding X/TOTAL`
- `Source: /abs/path/file.ext:line`
- `Rule: [rule_id]`
- `Rule description: ...`
- `Auto triage note: ...`
- `Function:` snippet

Extract fields:

- `subchunk_number` = global `X` from `Finding X/TOTAL`
- `source_file_path`
- `source_line`
- `rule_id`
- `rule_description`
- `auto_triage_note`
- `function_body`

## Classification Rules

### `true_positive`

The snippet actually shows the defect/anti-pattern targeted by the rule.

### `false_positive`

The match is heuristic/syntactic but not a real issue in visible snippet context.

Typical false-positive cases:

- Rule intent does not match shown code
- Snippet is benign/intentional
- Snippet does not include the claimed risky behavior
- Evidence is insufficient to support defect claim

No third category is allowed.

## CSV Output Contract

Header must be exactly:

```csv
chunk_file_path,subchunk_number,source_file_path,source_line,rule_id,decision,rationale
```

Field requirements:

- `chunk_file_path`: absolute path to chunk file
- `subchunk_number`: global finding number (`X` from `Finding X/TOTAL`)
- `source_file_path`: source path from `Source:` line
- `source_line`: source line from `Source:` line
- `rule_id`: value inside `Rule: [ ... ]`
- `decision`: `true_positive` or `false_positive`
- `rationale`: one-line evidence-based justification

## 6-Subagent Partition (5952 Findings)

Use contiguous finding ranges:

- SA1: findings `1-1000` -> `reports/chunk_analysis_part1.csv`
- SA2: findings `1001-2000` -> `reports/chunk_analysis_part2.csv`
- SA3: findings `2001-3000` -> `reports/chunk_analysis_part3.csv`
- SA4: findings `3001-4000` -> `reports/chunk_analysis_part4.csv`
- SA5: findings `4001-5000` -> `reports/chunk_analysis_part5.csv`
- SA6: findings `5001-5952` -> `reports/chunk_analysis_part6.csv`

Mapped file ranges for current dataset:

- SA1: `Chunk_1_25.txt` .. `Chunk_976_1000.txt`
- SA2: `Chunk_1001_1025.txt` .. `Chunk_1976_2000.txt`
- SA3: `Chunk_2001_2025.txt` .. `Chunk_2976_3000.txt`
- SA4: `Chunk_3001_3025.txt` .. `Chunk_3976_4000.txt`
- SA5: `Chunk_4001_4025.txt` .. `Chunk_4976_5000.txt`
- SA6: `Chunk_5001_5025.txt` .. `Chunk_5951_5952.txt`

## Subagent Responsibilities

Each subagent must:

1. Read every assigned chunk file in full.
2. Parse every finding block.
3. Review rule intent + snippet manually.
4. Emit exactly one CSV row per finding.
5. Use global `subchunk_number` from `Finding X/TOTAL`.
6. Write only its own part CSV file.

## Merge Target

Merge parts into:

`/home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all.csv`

Include header once.

## From-Scratch Validation Runbook (Using Subagents)

Use this checklist after every run.

### 1) Dataset discovery

- Count chunk files.
- Confirm first and last chunk file names.
- Confirm total findings from chunk headers (`Findings N-M of TOTAL`).

### 2) Partition integrity

- Verify each subagent received disjoint contiguous range.
- Verify union of ranges covers full `1..TOTAL` with no gaps.

### 3) Part-file schema checks

For each `chunk_analysis_part*.csv`:

- Header exactly matches contract.
- `decision` column contains only `true_positive|false_positive`.
- No empty required fields.

### 4) Global-number checks

- Confirm `subchunk_number` values are global finding IDs, not local per-file counters.
- For each row, `subchunk_number` should align with `Finding X/TOTAL` in the source chunk block.

### 5) Merge checks

After creating `chunk_analysis_all.csv`:

- Data row count must equal `TOTAL`.
- `subchunk_number` must contain every integer `1..TOTAL` exactly once.
- No duplicates, no missing IDs.
- Decision domain still only TP/FP.

### 6) Spot audit

- Randomly sample rows from each partition.
- Open corresponding chunk blocks and verify decision rationale matches visible snippet evidence.

## 6-Lane Reduction Loop (Subagent-Style, Parallel)

Use this loop after initial TP/FP adjudication when reducing false positives toward the validated TP target.

### Lanes

1. `Lane-1 (Current Counts)`: parse latest scan output and produce current `rule_id -> count`.
2. `Lane-2 (Truth Mapping)`: map each active `rule_id` to historical TP/FP counts from `reports/chunk_analysis_all.csv`.
3. `Lane-3 (Go Tightening)`: tighten high-FP Go-specific rules first.
4. `Lane-4 (Python Tightening)`: tighten high-FP Python-specific rules first.
5. `Lane-5 (Perf-Layer Tightening)`: tighten cross-language performance-layer matchers/overrides.
6. `Lane-6 (Validation)`: run tests + scan, diff rule deltas, persist iteration artifacts.

### Execution Cycle

For each iteration:

1. Rank candidate rules by:
   - high current count
   - low historical precision (`TP / (TP + FP)`)
   - high absolute FP count in adjudication truth
2. Tighten only the top families for that iteration.
3. Run:
   - `cargo test --lib --tests`
   - `make scan-gopdfsuit-info`
4. Record:
   - total findings delta
   - top dropping rules
   - top remaining rules
5. Repeat until convergence target.

### Iteration Artifacts

- Save each scan output as `reports/fp-iterations/temp_gopdfsuit_iterN.txt`.
- Save each rule count snapshot as `reports/fp-iterations/iterN_rule_counts.csv`.
- Keep baseline snapshot for diffing.

## Quick Validation Commands (Shell)

```bash
# 1) File count
ls /home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks/Chunk_*.txt | sort -V | wc -l

# 2) Merged row count (minus header)
wc -l /home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all.csv

# 3) Decision domain
cut -d, -f6 /home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all.csv | tail -n +2 | sort | uniq -c

# 4) Missing/duplicate subchunk IDs (adjust TOTAL as needed)
awk -F',' 'NR>1{seen[$2]++} END{
  total=5952;
  miss=0; dup=0;
  for(i=1;i<=total;i++){ if(!(i in seen)) miss++ }
  for(k in seen){ if(seen[k]>1) dup += seen[k]-1 }
  print "missing=" miss;
  print "duplicate=" dup;
}' /home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all.csv
```

## Quality Bar

- Manual evidence-based decisions only.
- No auto-classifier substitution.
- One row per finding.
- Deterministic, validated final CSV.
