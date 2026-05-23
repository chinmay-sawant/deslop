# Chunk Snippet Analysis Workflow (6 Subagents)

## Objective
Analyze snippet chunks from:
`/home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks`

The snippets may contain Go, Rust, or Python code. Perform architecture-grade review and produce a **CSV-formatted output**.

## Hard Constraints
- Work as a **senior solution architect**.
- Perform analysis **without using any tools**.
- Use exactly **6 subagents** with distinct responsibilities.
- Every CSV row must include:
  - `chunk_file_path`
  - `subchunk_number` (from `Finding X/4098`)
- Preserve deterministic, machine-readable output.

## Input Pattern
Each chunk file contains multiple findings like:
- `Finding X/4098`
- `Source: /absolute/path/to/file.ext:line`
- `Rule: [rule_id]`
- `Rule description: ...`
- `Auto triage note: ...`
- `Function: ...`

## 6-Subagent Design

### Subagent 1: Context Extractor
- Parse each finding block.
- Extract: chunk file path, finding number, source path, line, rule id, rule description, auto triage note, function body.

### Subagent 2: Language + Syntax Specialist
- Identify language: Go / Rust / Python.
- Validate whether the snippet is semantically coherent.
- Flag parsing ambiguity when snippet is partial.

### Subagent 3: Rule Intent Validator
- Interpret what the rule is trying to detect.
- Check if observed code truly matches the rule intent.
- Mark likely false-positive conditions.

### Subagent 4: Architecture & Design Reviewer
- Evaluate coupling, abstraction boundaries, reliability, maintainability, and scalability implications.
- Focus on system-level impact, not only line-level style.

### Subagent 5: Risk & Priority Assessor
- Assign severity: `critical|high|medium|low|info`.
- Assign confidence: `high|medium|low`.
- Explain blast radius and production risk.

### Subagent 6: Decision Synthesizer
- Produce final action: `actionable|needs_context|false_positive|defer`.
- Provide concise remediation guidance and rationale.
- Emit final CSV row.

## Decision Policy
- `actionable`: clear defect/risk and enough context to fix now.
- `needs_context`: likely issue but dependent on missing repository/runtime context.
- `false_positive`: rule matched but practical defect not supported by snippet.
- `defer`: valid observation but low-impact / not cost-effective currently.

## CSV Output Contract
Output only CSV rows with this exact header:

```csv
chunk_file_path,subchunk_number,source_file_path,source_line,language,rule_id,severity,confidence,decision,architectural_risk,remediation_summary,rationale
```

## Field Semantics
- `chunk_file_path`: absolute path of the chunk file being analyzed.
- `subchunk_number`: numeric value from `Finding X/4098` => `X`.
- `source_file_path`: path before the colon from `Source:`.
- `source_line`: line number after the colon from `Source:`.
- `language`: `go|rust|python|unknown`.
- `rule_id`: value inside `Rule: [ ... ]`.
- `severity`: `critical|high|medium|low|info`.
- `confidence`: `high|medium|low`.
- `decision`: `actionable|needs_context|false_positive|defer`.
- `architectural_risk`: short tag list (e.g., `coupling;reliability;performance`).
- `remediation_summary`: one-line fix guidance.
- `rationale`: one-line decision justification.

## Example Row
```csv
/home/chinmay/ChinmayPersonalProjects/deslop/scripts/chunks/Chunk_1_25.txt,1,/home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/handlers/handlers.go,34,go,feature_flag_lookup_without_config_abstraction,low,medium,needs_context,coupling;maintainability,Introduce a narrow feature-flag interface injected into handlers to reduce direct flag lookups,Code suggests direct environment/config probing in handler flow but full dependency graph is not visible
```

## Quality Bar
- Be strict about evidence.
- Do not over-escalate informational findings.
- Favor architecture-aware decisions over generic lint-style commentary.
- Keep remediation practical and minimally invasive.
