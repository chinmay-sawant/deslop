# Reduce False Positives (Phased Implementation Plan)

## Context
Current FP concentration is high and clustered around broad matcher families and low-context scans. The highest-impact reduction path is to add stronger admission guards, tighten over-broad textual matchers, and enforce regression gates in CI.

## Phase 0 (P0): Precision Guardrails First

### 1) Non-code and syntax gate before heuristics
- Add a scan admission gate after parse in `src/scan/file_analysis.rs`.
- Add defense-in-depth guard in `src/heuristics/engine.rs`.
- Behavior:
  - If `syntax_error == true` and no strong code anchors, skip heuristic emission.
  - Record skip/failure reason as `likely_non_code_text`.

### 2) Rust syntax guard for noisy Rust families
- Add Rust-specific guard in `src/heuristics/rust/evaluate.rs`.
- Apply to hallucination/hygiene style checks to avoid firing on malformed/partial Rust parses unless evidence is high-confidence.

### 3) Tighten performance-layer textual matching
- Harden matcher logic in `src/heuristics/performance_layers.rs`.
- Match on comment-stripped and string-literal-stripped executable text.
- Require stronger evidence groups and at least one syntax anchor where applicable.

### 4) Tighten Rust unresolved import handling
- Update unresolved-path behavior in `src/analysis/rust/findings/import_resolution.rs`.
- Only emit unresolved import findings when corroborated by local module/import structure.
- Otherwise downgrade/suppress noisy unresolveds.

## Phase 1 (P1): Rule Semantics Tightening

### 5) Rust bad-practices marker quality
- Refactor broad marker rules in `src/heuristics/rust/bad_practices.rs`.
- Introduce `trigger/context/anti-marker` semantics.
- Require proximity and same-receiver / same-variable coherence.

### 6) Request-path qualification hardening
- Tighten request-path detection in `src/heuristics/rust/runtime_ownership.rs`.
- Require framework import + handler-shape confirmation before per-request findings.

### 7) Token semantics for secret/domain matching
- Replace broad substring matching in `src/heuristics/rust/mod.rs` and related modules.
- Use identifier tokenization and whole-token matching.
- Add denylist for common false compounds.

### 8) Comment/prose-driven rule gating
- Tighten comment-only findings in:
  - `src/heuristics/python/ai_smells.rs`
  - `src/heuristics/python/maintainability/file_rules.rs`
- Require minimum code anchors before firing on prose-heavy files.

### 9) Context extraction fidelity improvements
- Improve context provenance + stability in:
  - `src/export/function.rs`
  - `src/export/triage.rs`
  - `src/export/block.rs`
  - `src/export/chunk.rs`
- Preserve span/evidence metadata; avoid over-flattening context.

## Phase 2 (P2): Output Quality + Regression Protection

### 10) Dedupe and burst control
- Add finding dedupe by `(rule_id, file, function, line)`.
- Add line/function burst cap to avoid floods of weak co-triggers.

### 11) Dynamic Rust fixture coverage completeness
- Remove fixed Rust rule range assumptions in:
  - `tests/integration_scan/rust/rule_fixture_coverage.rs`
- Enforce invariant: all registered Rust rules execute in fixture coverage.

### 12) FP regression gates in CI
- Expand Rust/text heuristic tests and add hard-negative corpus checks.
- Add fail thresholds on FP drift:
  - no new FP on clean corpus in PR
  - per-rule FP regression cap
  - release gate against baseline FP budget

## Execution Order
1. Implement Phase 0 completely first.
2. Implement Phase 1 next with targeted rule-family rollouts.
3. Implement Phase 2 to lock stability and prevent regressions.

## Success Criteria
- Immediate drop in top noisy FP families.
- No significant true-positive regression in existing fixtures.
- Deterministic CI guardrails blocking FP drift.
