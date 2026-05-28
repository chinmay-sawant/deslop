# Chunk Snippet Analysis Workflow (6 Subagents)

## Objective

Manually triage every finding in chunk files under:

`/home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks`

Read each chunk file directly, review each finding block, and assign a **`confidence`** score for how likely the match is a **genuine defect** (what used to be called a true positive).

Produce a deterministic CSV with one row per finding.

## Hard Constraints

- Read `Chunk_*.txt` files directly. Do not use Python triage/classifier scripts.
- Use exactly 6 subagents, each with a disjoint partition.
- Each finding gets exactly one **`confidence`** value on **0.00–1.00**, formatted as a **two-decimal float** (e.g. `0.69`, `0.05`, `0.92`).
- No string labels (`true_positive`, `false_positive`, `needs_context`, `defer`, `actionable`, etc.).
- Output must be machine-readable CSV with stable header.
- Do not auto-score low confidence (e.g. `0.10`) when `Function: [FUNCTION_NOT_FOUND]` appears without source-file cross-check.
- Always analyze the full finding context in chunk text first, then verify against the referenced source path from `Source: /abs/path/file.ext:line`.
- Source-file validation is mandatory for adjudication when path is available.

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

## How to Score Confidence (Manual Review Only)

For **each** finding, estimate how confident you are that the snippet **actually exhibits** the defect the rule targets.

Think of **`confidence` like an AI temperature**: a single float from **0.00** to **1.00**, always written with **two decimal places**.

| Range | Meaning |
|---|---|
| **0.85–1.00** | Very confident genuine defect — clear evidence; a reasonable engineer would act on it |
| **0.65–0.84** | Likely genuine defect — pattern matches rule intent with minor caveats |
| **0.36–0.64** | Uncertain / borderline — partial snippet, mixed signals, or rule intent only partly applies |
| **0.15–0.35** | Likely false alarm — heuristic match but context makes the issue benign or inapplicable |
| **0.00–0.14** | Very confident false alarm — rule does not apply; pattern is intentional, idiomatic, or absent |

### High confidence (near 1.00)

The snippet **clearly exhibits** the defect/anti-pattern targeted by the rule. Use **0.85+** when you would previously have called it a true positive with little doubt.

### Low confidence (near 0.00)

The match is heuristic/syntactic but not a real issue in visible snippet context. Use **0.15 or below** when you would previously have called it a false positive with little doubt.

Typical low-score cases:

- Rule intent does not match shown code
- Snippet is benign/intentional
- Snippet does not include the claimed risky behavior
- Evidence is insufficient to support defect claim
- Rule text is generic but source-file inspection disproves the claim for the referenced location

### Middle band (e.g. 0.45–0.69)

Use when evidence is genuinely mixed. Do **not** force `0.00` or `1.00` when uncertainty is honest. The rationale must explain why the number landed where it did.

## Mandatory Context-First Rule (Critical)

For every finding:

1. Read the full finding block from chunk file, including:
   - `Rule`
   - `Rule description`
   - `Message`
   - `Auto triage note`
   - `Function` block
2. If function body is missing (`[FUNCTION_NOT_FOUND]`), do **not** stop.
3. Open the referenced file from `Source:` and inspect around the cited line.
4. Assign **`confidence`** only after chunk-context + source-file cross-check.

This rule overrides any earlier shortcut that treated missing function snippets as automatic low confidence.

## CSV Output Contract

Header must be exactly:

```csv
chunk_file_path,subchunk_number,source_file_path,source_line,rule_id,confidence,rationale
```

Field requirements:

- `chunk_file_path`: absolute path to chunk file
- `subchunk_number`: global finding number (`X` from `Finding X/TOTAL`)
- `source_file_path`: source path from `Source:` line
- `source_line`: source line from `Source:` line
- `rule_id`: value inside `Rule: [ ... ]`
- `confidence`: **0.00–1.00** (two decimals) — likelihood the finding is a **genuine defect**; higher = more likely real issue, lower = more likely false alarm
- `rationale`: one-line evidence-based justification tied to the score

## Example Rows

```csv
chunk_file_path,subchunk_number,source_file_path,source_line,rule_id,confidence,rationale
/home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks/Chunk_1_25.txt,1,/home/chinmay/ChinmayPersonalProjects/deslop/real-repos/sqlx/Makefile,1,ci_missing_go_test_race,0.08,Makefile defines test-race target; rule claim not supported by visible snippet
/home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks/Chunk_1_25.txt,4,/home/chinmay/ChinmayPersonalProjects/deslop/real-repos/sqlx/bind.go,106,buffer_write_rune_ascii_literal,0.91,rebindBuff calls WriteRune for ASCII literal instead of WriteByte
```

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
3. Review rule intent + chunk context manually.
4. Open referenced source files and inspect cited locations.
5. If `FUNCTION_NOT_FOUND`, continue adjudication using message/rule/source file context.
6. Assign **`confidence`** (0.00–1.00, two decimals) with a one-line rationale.
7. Emit exactly one CSV row per finding.
8. Use global `subchunk_number` from `Finding X/TOTAL`.
9. Write only its own part CSV file.

### Role breakdown

| Role | Task |
|---|---|
| File Reader | List and read assigned chunk files; confirm headers |
| Finding Parser | Split blocks; extract source, rule, notes, function body |
| Rule Intent Reviewer | Determine what defect the rule targets |
| Code Reviewer | Judge whether snippet matches rule intent |
| Confidence Scorer | Assign 0.00–1.00 score with evidence-based rationale |
| CSV Writer | Emit one row per finding; no skips or duplicates |

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
- `confidence` column values are numeric, in **0.00–1.00**, with **two decimal places**.
- No empty required fields.

### 4) Global-number checks

- Confirm `subchunk_number` values are global finding IDs, not local per-file counters.
- For each row, `subchunk_number` should align with `Finding X/TOTAL` in the source chunk block.

### 5) Merge checks

After creating `chunk_analysis_all.csv`:

- Data row count must equal `TOTAL`.
- `subchunk_number` must contain every integer `1..TOTAL` exactly once.
- No duplicates, no missing IDs.
- All `confidence` values remain in **0.00–1.00** with two decimals.

### 6) Spot audit

- Randomly sample rows from each partition.
- Open corresponding chunk blocks and verify rationale matches visible snippet evidence and the confidence band.

## 6-Lane Reduction Loop (Subagent-Style, Parallel)

Use this loop after initial adjudication when reducing false alarms toward validated high-confidence findings.

### Lanes

1. `Lane-1 (Current Counts)`: parse latest scan output and produce current `rule_id -> count`.
2. `Lane-2 (Truth Mapping)`: map each active `rule_id` to historical confidence distribution from `reports/chunk_analysis_all.csv` (treat **≥ 0.85** as high-confidence genuine, **≤ 0.15** as high-confidence false alarm; middle band is uncertain).
3. `Lane-3 (Go Tightening)`: tighten high–false-alarm Go-specific rules first.
4. `Lane-4 (Python Tightening)`: tighten high–false-alarm Python-specific rules first.
5. `Lane-5 (Perf-Layer Tightening)`: tighten cross-language performance-layer matchers/overrides.
6. `Lane-6 (Validation)`: run tests + scan, diff rule deltas, persist iteration artifacts.

### Execution Cycle

For each iteration:

1. Rank candidate rules by:
   - high current count
   - low historical precision (share of rows with confidence **≥ 0.85** among adjudicated rows for that rule)
   - high count of rows with confidence **≤ 0.15**
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

# 3) Confidence range sanity (field 6 = confidence)
awk -F',' 'NR>1{
  if ($6+0 < 0 || $6+0 > 1) bad++;
  if ($6 !~ /^[0-9]+\.[0-9]{2}$/) fmt++;
} END{
  print "out_of_range=" bad+0;
  print "bad_format=" fmt+0;
}' /home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all.csv

# 4) Confidence band histogram
awk -F',' 'NR>1{
  c=$6+0;
  if (c<=0.15) a++;
  else if (c>=0.85) b++;
  else m++;
} END{
  print "low_0.00-0.15=" a+0;
  print "mid_0.16-0.84=" m+0;
  print "high_0.85-1.00=" b+0;
}' /home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all.csv

# 5) Missing/duplicate subchunk IDs (adjust TOTAL as needed)
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

- Manual evidence-based scoring only.
- No auto-classifier substitution.
- One row per finding.
- Always emit two decimal places (`0.70`, not `0.7`).
- Do not inflate confidence on informational or mismatched rules — use the low band (≤ 0.35) when the rule clearly does not apply.
- Deterministic, validated final CSV.
