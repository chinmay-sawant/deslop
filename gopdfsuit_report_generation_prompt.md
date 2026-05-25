# Reusable Multi-Agent Triage Adjudication Prompt (Based on gopdfsuit Workflow)

Use this prompt to reproduce the same end-to-end process for any repository.

## Objective
Run a 6-subagent adjudication pipeline over a triage markdown table and classify findings into `TP`, `FP`, or `REVIEW_REQUIRED` with evidence-backed reasoning.

## Required Agent Roles
Use exactly 6 subagents:
1. Auditor-1
2. Auditor-2
3. Debater-1
4. Debater-2
5. Prover-1
6. Prover-2

## Parallel 6-Lane Loop (For FP Reduction Iterations)

After initial adjudication output is produced, run a 6-lane parallel loop each iteration:

1. `Lane-1 Current Counts`
   - Parse latest scan output and compute current rule-wise counts.
2. `Lane-2 Truth Join`
   - Join current rules with adjudication truth (`TP/FP`) from prior chunk analysis CSV.
3. `Lane-3 Go Matcher Tightening`
   - Patch highest-impact low-precision Go rules.
4. `Lane-4 Python Matcher Tightening`
   - Patch highest-impact low-precision Python rules.
5. `Lane-5 Perf-Layer Tightening`
   - Patch shared perf-layer strict semantic overrides.
6. `Lane-6 Validation + Diff`
   - Run tests, run scan, emit total delta + top rule deltas.

## Inputs (Replace Placeholders)
- `TRIAGE_TABLE_MD`: absolute path to markdown file containing table rows.
- Table columns expected:
  - `File | Classification | Rationale | Source | Rule | Evidence Source Path | Evidence Line`
- `EVIDENCE_DIR`: directory containing numbered evidence files (`<number>.txt`).
- `BATCH_SIZE`: 1000.

Example placeholders:
- `TRIAGE_TABLE_MD=/abs/path/to/triage_report_full.md`
- `EVIDENCE_DIR=/abs/path/to/findings_or_functions`

## Hard Constraints
- Batch processing only (size = 1000).
- Findings must be mapped to numbered evidence files: `<EVIDENCE_DIR>/<finding_number>.txt`.
- Do not do heuristic-only classification.
- Do not use random regex shortcuts to classify.
- Use explicit row evidence + mapped numbered text-file evidence.

## Batch Number Mapping Rule
For batch index `k` (1-based), start finding number is:
- `start_finding = ((k - 1) * 1000) + 1`

So starts are: `1, 1001, 2001, 3001, 4001, 5001, ...`

## Workflow Steps (Keep Exactly)
1. Read `TRIAGE_TABLE_MD` and confirm the table rows are present.
2. Count total data rows (exclude header + separator).
3. Split into batch files of 1000 rows each.
4. Assign auditors across full range:
   - Auditor-1: first half of batches/findings.
   - Auditor-2: second half of batches/findings.
5. For every row audited, consume:
   - row metadata from batch table
   - mapped evidence file: `<EVIDENCE_DIR>/<global_finding_id>.txt`
6. Auditor outputs must include verdict + confidence + short evidence reason.
7. Debaters cross-review auditor outputs and disagreements, again checking mapped evidence files.
8. Provers independently generate final verdicts using auditor+debater outputs and evidence checks.
9. Merge prover outputs:
   - if `prover1 == prover2` => final verdict = agreed value, confidence high.
   - else => `REVIEW_REQUIRED`, confidence medium.
10. Produce final adjudication CSV.
11. Produce companion CSV including absolute evidence file path per finding.
12. Keep original final adjudication unchanged once generated; add enriched file separately.
13. For FP reduction phases, execute the parallel 6-lane loop per iteration:
    - select top low-precision high-volume families
    - patch/tighten
    - validate with tests
    - re-scan and diff
    - continue until target threshold

## Output Files (Template)
- `batchXX_1000.md` files
- `auditor1_*.csv`
- `auditor2_*.csv`
- `debater1_*.csv`
- `debater2_*.csv`
- `prover1_*.csv`
- `prover2_*.csv`
- `final_adjudication_all.csv`
- `final_adjudication_all_with_text_paths.csv`

## Required Final CSV Schema
For `final_adjudication_all.csv`:
- `global_finding_id,prover1_final,prover2_final,final_verdict,confidence,consensus_note`

For `final_adjudication_all_with_text_paths.csv`:
- `global_finding_id,finding_number,text_file_path,prover1_final,prover2_final,final_verdict,confidence,consensus_note`

## Quality Checks
- Row coverage must be complete (1..N with no gaps).
- All batch rows must map to a valid numbered evidence file.
- Report totals for `TP`, `FP`, `REVIEW_REQUIRED`.
- Report line counts for all key CSV outputs.

## Execution Note
Replicate the same operating style as the gopdfsuit run:
- batch-first,
- strict numbered evidence-file mapping,
- 6-agent auditor/debater/prover flow,
- final consensus merge,
- enriched path-aware companion output.
