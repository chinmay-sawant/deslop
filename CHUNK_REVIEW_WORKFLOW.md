# Chunk Snippet Review Workflow (6 Subagents)

## Objective

Perform an **independent second-pass review** on findings that were already adjudicated in `CHUNK_ANALYSIS_WORKFLOW.md`.

This workflow:

1. **Scans the existing analysis report first** — load, validate, and index prior `confidence` + `rationale` rows.
2. **Re-scans chunk files the same way** as the analysis workflow — read chunk blocks directly, cross-check source files, assign fresh scores.
3. **Updates the existing report in place** — keep all original columns; append the reviewer's own trailing columns (`review_confidence`, `review_rationale`).

The review pass must be **independent**. Prior analysis rows are context only; reviewers must re-read chunk + source evidence and form their own judgment.

## Prerequisites

- Phase 1 complete: `reports/chunk_analysis_all.csv` exists and passes validation from `CHUNK_ANALYSIS_WORKFLOW.md`.
- Chunk export unchanged since analysis: same files under `scripts/chunks/` with matching global finding IDs.
- Recompute dataset counts before every run (do not assume static counts).

## Relationship to Analysis Workflow

| Phase | Workflow doc | Primary artifact |
|---|---|---|
| First pass (analysis) | `CHUNK_ANALYSIS_WORKFLOW.md` | `reports/chunk_analysis_all.csv` |
| Second pass (review) | `CHUNK_REVIEW_WORKFLOW.md` | `reports/chunk_analysis_all.csv` (updated in place) |

Do **not** overwrite analysis columns (`confidence`, `rationale`). The review pass only **appends** trailing review columns.

---

## Hard Constraints

- Read `Chunk_*.txt` files directly during the review pass. Do not use Python triage/classifier scripts for scoring.
- Read `reports/chunk_analysis_all.csv` first during Phase 1. Do not skip report ingestion.
- Use exactly **6 subagents**, each with a disjoint partition (same ranges as analysis).
- Each finding gets exactly one **`review_confidence`** on **0.00–1.00**, formatted as a **two-decimal float** (e.g. `0.69`, `0.05`, `0.92`).
- Each finding gets exactly one **`review_rationale`**: one-line, evidence-based, tied to the review score.
- No string labels (`true_positive`, `false_positive`, `needs_context`, `defer`, `actionable`, `agree`, `disagree`, etc.) in CSV fields.
- Do not auto-copy prior `confidence` / `rationale` into review columns without fresh chunk + source inspection.
- Do not auto-score low review confidence when `Function: [FUNCTION_NOT_FOUND]` appears without source-file cross-check.
- Always analyze full finding context in chunk text first, then verify against `Source: /abs/path/file.ext:line`.
- Source-file validation is mandatory for adjudication when path is available.

---

## Current Dataset Snapshot

At the time of this update:

- Analysis report: `/home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all.csv`
- Chunk directory: `/home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks`
- File pattern: `Chunk_<start>_<end>.txt`
- Chunk files: `264`
- Findings: `6598`
- Last chunk file: `Chunk_6576_6598.txt`

Recompute before every new run.

---

## Phase 1 — Scan Existing Analysis Report (Mandatory First Step)

Before opening any chunk file, ingest the existing analysis report.

### 1.1 Locate and load report

Primary input:

`/home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all.csv`

Expected header (analysis-only, no review columns yet):

```csv
chunk_file_path,subchunk_number,source_file_path,source_line,rule_id,confidence,rationale
```

### 1.2 Validate analysis report integrity

Confirm before review begins:

- Data row count equals `TOTAL` findings from chunk headers (`Findings N-M of TOTAL`).
- `subchunk_number` contains every integer `1..TOTAL` exactly once.
- `confidence` values are numeric, in **0.00–1.00**, with **two decimal places**.
- No empty required fields in analysis columns.
- If review columns already exist from a partial prior run, treat them as stale and regenerate review parts from scratch (do not merge stale review values).

### 1.3 Build lookup index

Index existing rows by `subchunk_number` (global finding ID). For each ID, record:

- `chunk_file_path`
- `source_file_path`
- `source_line`
- `rule_id`
- `confidence` (prior analysis score)
- `rationale` (prior analysis note)

Use this index to:

- Confirm partition assignments align with analysis rows.
- Provide **read-only context** during review (what the first pass concluded).
- Detect drift if chunk metadata no longer matches the report row for the same `subchunk_number`.

### 1.4 Report scan outputs (optional but recommended)

Produce a quick snapshot before review starts:

- Total rows
- Confidence histogram (low / mid / high bands using same thresholds as analysis workflow)
- Top `rule_id` counts
- Rows where prior `confidence >= 0.85` or `<= 0.15` (priority spot-check candidates during review)

Save optional snapshot to:

`/home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_pre_review_snapshot.txt`

This snapshot is informational only; it does **not** substitute for chunk re-review.

---

## Phase 2 — Chunk Re-Review (Same Method as Analysis)

After Phase 1 completes, review chunk files using the same process as `CHUNK_ANALYSIS_WORKFLOW.md`.

### Finding block format

Each chunk contains finding blocks separated by:

`====================================================================================================`

Each block includes:

- `Finding X/TOTAL`
- `Source: /abs/path/file.ext:line`
- `Rule: [rule_id]`
- `Rule description: ...`
- `Auto triage note: ...`
- `Function:` snippet

Extract the same fields as analysis (`subchunk_number`, `source_file_path`, `source_line`, `rule_id`, etc.).

### Mandatory review order per finding

For **each** finding, follow this order:

1. Load the existing analysis row for `subchunk_number` from Phase 1 index (**context only**).
2. Read the full finding block from the chunk file (Rule, description, Message, auto triage note, Function).
3. Open the referenced `Source:` file and inspect around the cited line.
4. Assign **`review_confidence`** and **`review_rationale`** from fresh evidence.
5. Do **not** copy prior `confidence` / `rationale` unless independent inspection reaches the same conclusion.

If chunk metadata for a finding disagrees with the indexed analysis row (different `rule_id`, source path, or line), stop and reconcile before continuing that partition.

### How to score review confidence

Use the same rubric as analysis. **`review_confidence`** is an independent estimate of how likely the snippet exhibits the defect the rule targets.

| Range | Meaning |
|---|---|
| **0.85–1.00** | Very confident genuine defect |
| **0.65–0.84** | Likely genuine defect |
| **0.36–0.64** | Uncertain / borderline |
| **0.15–0.35** | Likely false alarm |
| **0.00–0.14** | Very confident false alarm |

`review_rationale` must cite visible snippet/source evidence. When review differs from prior analysis, say **why** in the rationale (e.g. source line shows comma-ok assertion, not bare type assertion). Do not use label words like "disagree"; describe evidence instead.

### Review part CSV contract (per subagent)

Each subagent writes only its partition file:

| Subagent | Findings | Part output |
|---|---|---|
| SR1 | `1-1000` | `reports/chunk_review_part1.csv` |
| SR2 | `1001-2000` | `reports/chunk_review_part2.csv` |
| SR3 | `2001-3000` | `reports/chunk_review_part3.csv` |
| SR4 | `3001-4000` | `reports/chunk_review_part4.csv` |
| SR5 | `4001-5000` | `reports/chunk_review_part5.csv` |
| SR6 | `5001-6598` | `reports/chunk_review_part6.csv` |

Header must be exactly:

```csv
subchunk_number,review_confidence,review_rationale
```

Field requirements:

- `subchunk_number`: global finding ID (`X` from `Finding X/TOTAL`)
- `review_confidence`: **0.00–1.00** (two decimals)
- `review_rationale`: one-line evidence-based finding from the independent review

Part files contain **review columns only**. Analysis columns are merged in Phase 3.

### Mapped chunk file ranges (current dataset)

- SR1: `Chunk_1_25.txt` .. `Chunk_976_1000.txt`
- SR2: `Chunk_1001_1025.txt` .. `Chunk_1976_2000.txt`
- SR3: `Chunk_2001_2025.txt` .. `Chunk_2976_3000.txt`
- SR4: `Chunk_3001_3025.txt` .. `Chunk_3976_4000.txt`
- SR5: `Chunk_4001_4025.txt` .. `Chunk_4976_5000.txt`
- SR6: `Chunk_5001_5025.txt` .. `Chunk_6576_6598.txt`

### Subagent responsibilities

Each review subagent must:

1. Confirm Phase 1 index covers its assigned `subchunk_number` range.
2. Read every assigned chunk file in full.
3. Parse every finding block.
4. Re-read prior analysis row for context (no copying scores).
5. Open referenced source files and inspect cited locations.
6. If `FUNCTION_NOT_FOUND`, continue adjudication using message/rule/source file context.
7. Assign **`review_confidence`** (0.00–1.00, two decimals) with one-line **`review_rationale`**.
8. Emit exactly one CSV row per finding in its part file.
9. Write only its own `chunk_review_partN.csv`.

### Role breakdown

| Role | Task |
|---|---|
| Report Scanner | Phase 1 ingestion + index by `subchunk_number` |
| File Reader | List/read assigned chunk files; confirm headers |
| Finding Parser | Split blocks; extract source, rule, notes, function body |
| Prior-Row Reader | Load analysis context; detect metadata drift |
| Code Reviewer | Independent judgment from chunk + source evidence |
| Review Scorer | Assign `review_confidence` + `review_rationale` |
| CSV Writer | Emit one review row per finding; no skips or duplicates |

---

## Phase 3 — Merge Review Columns Into Existing Report

Merge review parts back onto the existing analysis report **by `subchunk_number`**.

### Updated report target

Update in place:

`/home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all.csv`

Final header must be exactly:

```csv
chunk_file_path,subchunk_number,source_file_path,source_line,rule_id,confidence,rationale,review_confidence,review_rationale
```

Merge rules:

1. Preserve all original analysis columns and values unchanged.
2. Append `review_confidence` and `review_rationale` as the **last two columns**.
3. Join review parts on `subchunk_number` (inner join — every analysis row must receive review values).
4. Sort output by ascending `subchunk_number`.
5. Write atomically (write temp file, validate, then replace target).

Optional backup before replace:

`/home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all_pre_review_backup.csv`

### Example merged row

```csv
chunk_file_path,subchunk_number,source_file_path,source_line,rule_id,confidence,rationale,review_confidence,review_rationale
/home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks/Chunk_1_25.txt,1,/home/chinmay/ChinmayPersonalProjects/deslop/real-repos/clawvisor/.github/workflows/publish-skill.yml,1,ci_missing_go_test_race,0.10,Workflow file has no go test steps; rule is informational and often inapplicable to non-test automation,0.08,YAML publish workflow only; no Go test invocation and rule is informational
```

---

## End-to-End Execution Order

Run phases strictly in this order:

```text
Phase 1: Scan + validate reports/chunk_analysis_all.csv
    ↓
Phase 2: 6 subagents → reports/chunk_review_part1..6.csv
    ↓
Phase 3: Merge review columns → update reports/chunk_analysis_all.csv
    ↓
Validation runbook (below)
```

Do not start Phase 2 until Phase 1 passes. Do not merge until all six review part files pass schema checks.

---

## Validation Runbook

### 1) Analysis report pre-check (Phase 1)

- Row count equals `TOTAL`.
- `subchunk_number` is a perfect `1..TOTAL` permutation.
- Analysis `confidence` format valid (two decimals, 0.00–1.00).

### 2) Partition integrity (Phase 2)

- Six disjoint contiguous review ranges cover full `1..TOTAL`.
- Each `chunk_review_partN.csv` row count matches its assigned range size.

### 3) Review part schema checks

For each `chunk_review_part*.csv`:

- Header exactly: `subchunk_number,review_confidence,review_rationale`
- `review_confidence` numeric, **0.00–1.00**, two decimal places
- No empty review fields
- No duplicate `subchunk_number` within a part file

### 4) Merge checks (Phase 3)

After updating `chunk_analysis_all.csv`:

- Header has **9 columns** in the order defined above.
- Data row count still equals `TOTAL`.
- Every row has non-empty `review_confidence` and `review_rationale`.
- Original analysis columns unchanged vs pre-merge backup (except appended review columns).
- `subchunk_number` still `1..TOTAL` exactly once.

### 5) Agreement diagnostics (optional)

Compute review vs analysis deltas for QA:

- `delta = review_confidence - confidence` (numeric)
- Count rows where `abs(delta) >= 0.30` for spot audit
- Count rows where analysis and review fall on opposite sides of `0.50`

Save optional summary to:

`/home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_review_delta_summary.txt`

### 6) Spot audit

Randomly sample rows from each review partition. For each sample:

- Open chunk block + source line
- Verify `review_rationale` matches visible evidence
- Verify `review_confidence` band matches rationale strength

---

## Quick Validation Commands (Shell)

```bash
TOTAL=6598
REPORT=/home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_analysis_all.csv

# 1) Chunk file count
ls /home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks/Chunk_*.txt | sort -V | wc -l

# 2) Merged row count (minus header)
wc -l "$REPORT"

# 3) Header check (must include review columns after merge)
head -1 "$REPORT"

# 4) Analysis confidence sanity (field 6)
awk -F',' 'NR>1{
  if ($6+0 < 0 || $6+0 > 1) bad++;
  if ($6 !~ /^[0-9]+\.[0-9]{2}$/) fmt++;
} END{
  print "analysis_out_of_range=" bad+0;
  print "analysis_bad_format=" fmt+0;
}' "$REPORT"

# 5) Review confidence sanity (field 8)
awk -F',' 'NR>1{
  if ($8+0 < 0 || $8+0 > 1) bad++;
  if ($8 !~ /^[0-9]+\.[0-9]{2}$/) fmt++;
  if ($8 == "" || $9 == "") empty++;
} END{
  print "review_out_of_range=" bad+0;
  print "review_bad_format=" fmt+0;
  print "review_empty_fields=" empty+0;
}' "$REPORT"

# 6) Missing/duplicate subchunk IDs
awk -F',' -v total="$TOTAL" 'NR>1{seen[$2]++} END{
  miss=0; dup=0;
  for(i=1;i<=total;i++){ if(!(i in seen)) miss++ }
  for(k in seen){ if(seen[k]>1) dup += seen[k]-1 }
  print "missing=" miss;
  print "duplicate=" dup;
}' "$REPORT"

# 7) Review part row counts
for i in 1 2 3 4 5 6; do
  f="/home/chinmay/ChinmayPersonalProjects/deslop/reports/chunk_review_part${i}.csv"
  [ -f "$f" ] && echo "part$i: $(($(wc -l < "$f") - 1)) rows"
done

# 8) Large review-vs-analysis deltas (>= 0.30)
awk -F',' 'NR>1{
  d = ($8+0) - ($6+0);
  if (d < 0) d = -d;
  if (d >= 0.30) n++;
} END{ print "large_delta_rows=" n+0 }' "$REPORT"
```

---

## Quality Bar

- Phase 1 report scan is mandatory before any chunk re-read.
- Review scoring is independent; prior analysis is context, not authority.
- One review row per finding.
- Always emit two decimal places for `review_confidence` (`0.70`, not `0.7`).
- Append only review columns during merge; never mutate original analysis values.
- Deterministic, validated final CSV with both analysis and review findings side by side.
