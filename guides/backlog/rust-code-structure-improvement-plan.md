# Rust Code Structure Improvement Plan

Date: 2026-04-07

## Scope

This plan focuses on the Rust-heavy parts of the repository, especially:

- `src/analysis/rust`
- `src/heuristics/rust`
- `src/scan`
- `src/rules`
- `tests`

It is intentionally a structure plan, not a detection-expansion plan. The goal is to make the existing Rust code easier to extend, safer to refactor, and simpler to review without weakening current rule coverage.

## Current Assessment

Current rating: 8/10

Why this is already strong:

- The crate has a clear top-level layering contract in `src/lib.rs`.
- The scan pipeline is easy to follow in `src/scan/mod.rs`.
- The backend abstraction in `src/analysis/backend.rs` is clean and already supports multiple languages well.
- The Rust analyzer has meaningful parser coverage, repo-aware indexing, and a broad rule surface.
- The test suite is far above average for a static-analysis tool.

Validation snapshot captured while writing this plan:

- `cargo test` passed
- 351 tests passed
- 2 tests ignored

What keeps the score from being higher:

- Several Rust hotspots are too large and carry multiple responsibilities.
- Rust evaluation flow is partly special-cased instead of following one shared rule-execution model.
- Rule metadata ownership is not uniform across Rust families.
- The test harness is good, but repeated assertion patterns still make Rust integration tests more verbose than they need to be.
- The guides directory has accumulated many historical planning files, which makes it harder to identify the current plan of record.

## Highest-Value Structural Problems

### 1. Rust rule execution is split across too many orchestration styles

Current shape:

- Go and Python mostly flow through `src/heuristics/engine.rs` plus `src/heuristics/registry.rs`.
- Rust still has a custom orchestration layer in `src/analysis/rust/evaluate.rs`.
- Some Rust hygiene-style checks live close to the parser/output layer, while other Rust rules live under `src/heuristics/rust`.

Why this matters:

- Adding or changing Rust rules requires remembering multiple entry points.
- Shared runtime policy changes are harder to roll out consistently.
- Review cost goes up because the mental model is different for Rust than for Go and Python.

### 2. Large Rust files are carrying too much logic

Current hotspots worth shrinking first:

- `src/analysis/rust/findings.rs`
- `src/analysis/rust/parser/items.rs`
- `src/heuristics/rust/performance.rs`
- `src/heuristics/rust/security_footguns.rs`
- `tests/integration_scan/rust.rs`
- `tests/integration_scan/rust_advanced.rs`

Why this matters:

- Large files increase review friction.
- Small edits have a larger blast radius.
- It becomes harder to spot coupling between parsing, rule logic, and metadata.

### 3. Rule metadata placement is inconsistent for Rust

Current shape:

- Some Rust metadata lives under `src/rules/catalog/rust/*.rs`.
- Some Rust families expose `RULE_DEFINITIONS` directly from heuristic files.
- The central catalog builder in `src/rules/catalog/mod.rs` manually assembles everything.

Why this matters:

- Ownership boundaries are blurry.
- Moving a rule family means touching more files than necessary.
- The catalog module becomes a high-churn assembly point.

### 4. Rust parser collectors need another layer of internal structure

Current shape:

- `src/analysis/rust/parser/items.rs` collects symbols, module declarations, literals, structs, statics, attributes, and enums in one file.

Why this matters:

- Parser maintenance is slower than it should be.
- It is harder to add a new item summary without risking accidental coupling.
- Tests cannot target item collectors as cleanly as they could with narrower modules.

### 5. Test helpers are centralized, but not yet fully exploited

Current shape:

- `tests/support/mod.rs` already provides a solid shared harness.
- Some tests still create temp dirs and write files manually.
- Rust rule tests repeat long lists of `assert!(...any(...))` checks.

Why this matters:

- Repetition hides intent.
- Refactoring tests is more expensive than it needs to be.
- It is harder to see which tests are behavior locks versus smoke tests versus invariants.

## Target Architecture

The top-level crate layering is good and should stay:

- `analysis` parses and extracts evidence
- `index` resolves repository context
- `heuristics` evaluates rules
- `scan` orchestrates execution
- `cli` presents results

The refactor should improve structure inside those layers instead of replacing the layers.

### Desired end state

Rust-specific code should move toward this shape:

```text
src/analysis/rust/
  mod.rs
  evaluate.rs
  findings/
    mod.rs
    docs.rs
    hygiene.rs
    import_resolution.rs
    local_calls.rs
  parser/
    mod.rs
    imports.rs
    functions.rs
    evidence.rs
    items/
      mod.rs
      attributes.rs
      enums.rs
      modules.rs
      statics.rs
      structs.rs
      symbols.rs

src/heuristics/rust/
  mod.rs
  family.rs
  api_design/
    mod.rs
    state.rs
    surface.rs
  performance/
    mod.rs
    async_costs.rs
    containers.rs
    paths.rs
  security_footguns/
    mod.rs
    manifest.rs
    globals.rs
    boundary.rs
```

This does not need to happen in one rewrite. The important outcome is narrower ownership, not directory depth for its own sake.

## Principles For The Refactor

1. Keep behavior stable first.
2. Prefer extraction over rewriting.
3. Co-locate things that change together.
4. Keep module facades thin and leaf modules focused.
5. Do not move top-level concepts unless the current boundary is actively hurting maintainability.
6. Improve test readability while refactoring production structure.

## Phase Plan

### Phase 0: Lock The Current Behavior

Goal: make structure refactors low-risk.

Tasks:

- Add a short architecture note in the new Rust family modules that explains current ownership.
- Add characterization tests for the current Rust evaluator path before splitting it.
- Add targeted coverage for rule ordering, deduplication, and config gating on the Rust path.
- Add tests that prove metadata and execution remain aligned for each Rust family.

Suggested additions:

- A Rust-family regression test that scans a curated set of Rust fixtures and asserts the exact set of emitted rule ids for each fixture.
- A test that verifies every Rust rule id in the public registry is reachable from one and only one owning family.
- A test that verifies each Rust family exports both metadata and evaluator bindings.

Acceptance criteria:

- `cargo test` remains green before any large moves.
- The suite gains at least one Rust-specific characterization layer that will catch refactor drift.

### Phase 1: Split The Rust Parser Item Collectors

Goal: reduce the maintenance burden of `src/analysis/rust/parser/items.rs`.

Tasks:

- Create `src/analysis/rust/parser/items/mod.rs` as the facade.
- Extract symbol collection into `items/symbols.rs`.
- Extract struct and field summaries into `items/structs.rs`.
- Extract enum summaries into `items/enums.rs`.
- Extract static summaries into `items/statics.rs`.
- Extract attribute parsing into `items/attributes.rs`.
- Extract module declaration parsing into `items/modules.rs`.
- Keep shared parser helpers only in `items/mod.rs` if more than one leaf module uses them.

Rules for the split:

- Do not change exported types during the first split.
- Avoid renaming public helper functions unless the rename materially clarifies ownership.
- Move tests only after the modules compile and behavior matches.

Acceptance criteria:

- `items.rs` disappears or becomes a thin facade under 100 lines.
- Each extracted module has one clear concern.
- No parser behavior changes.

### Phase 2: Split Rust Hygiene And Resolution Findings

Goal: reduce the structural density of `src/analysis/rust/findings.rs`.

Tasks:

- Create `src/analysis/rust/findings/mod.rs` as the facade.
- Extract doc-comment marker findings into `findings/docs.rs`.
- Extract unsafe-without-comment hygiene checks into `findings/hygiene.rs`.
- Extract import-resolution logic into `findings/import_resolution.rs`.
- Extract local/direct call hallucination logic into `findings/local_calls.rs`.
- Keep shared helpers like alias lookup only where they are actually shared.

Design rule:

- The module split should follow problem domains, not arbitrary line count.

Acceptance criteria:

- The old file is replaced by a facade plus focused leaf modules.
- Import-resolution tests remain green.
- Hygiene findings are clearly separated from resolution findings.

### Phase 3: Normalize Rust Rule Execution

Goal: make Rust rule wiring feel like one system rather than a special case.

Tasks:

- Introduce a Rust family registry that groups file rules, function rules, indexed rules, and repo/file-context rules in one place.
- Move the hard-coded rule arrays inside `src/analysis/rust/evaluate.rs` into declarative family descriptors.
- Decide whether Rust should fully reuse `src/heuristics/engine.rs` or whether the crate should introduce a richer shared execution abstraction that all languages can use.
- Keep only one orchestration layer responsible for ordering, config gating, and dedup-preconditions.

Recommended direction:

- Prefer upgrading the shared engine instead of keeping Rust permanently bespoke.
- If full unification is too disruptive, create a single intermediate Rust family registry first, then converge the engine later.

Acceptance criteria:

- Adding a new Rust rule family should require touching one family module and one registry surface, not several unrelated files.
- The evaluator entry point becomes mostly orchestration, not business logic.

### Phase 4: Make Rule Metadata Ownership Uniform

Goal: reduce drift and simplify rule-family ownership.

Tasks:

- Choose one metadata ownership pattern for Rust and apply it consistently.
- Either:
  - keep metadata under `src/rules/catalog/rust/*` and remove Rust-family-local metadata definitions, or
  - keep metadata co-located with each Rust family and generate/assemble the catalog from those family-owned definitions.
- Reduce manual assembly churn inside `src/rules/catalog/mod.rs`.
- Add a test that fails if a Rust family exports runtime rules without catalog metadata or catalog metadata without runtime ownership.

Recommended direction:

- Co-locate Rust family metadata with Rust family implementation, then have `src/rules/catalog` assemble from those exports.
- This aligns ownership and reduces context switching during rule work.

Acceptance criteria:

- Every Rust family follows the same metadata pattern.
- The catalog builder becomes mechanical, not hand-curated logic.
- Binding-location tests still pass.

### Phase 5: Break Up The Largest Rust Heuristic Families

Goal: reduce the cognitive weight of Rust rule-family files without changing behavior.

Priority order:

1. `src/heuristics/rust/performance.rs`
2. `src/heuristics/rust/security_footguns.rs`
3. `src/heuristics/rust/api_design.rs`

Tasks for `performance`:

- Split async-cost checks from container/layout checks.
- Split path/string/text heuristics from runtime-sensitive checks.
- Keep common line scanners in a local private helper module.

Tasks for `security_footguns`:

- Separate manifest-level checks from source-level checks.
- Separate shared-state/global-state checks from boundary/API misuse checks.
- Move the metadata block out of the same file as large evaluator logic if metadata stays co-located.

Tasks for `api_design`:

- Keep `state` and `surface` logic as submodules, but turn the top-level file into a thin facade.
- Move purely string-parsing helpers into a local helper module if they are not used outside API-design rules.

Acceptance criteria:

- The top three Rust heuristic hotspots shrink materially.
- Each family facade reads like an index, not a mixed implementation dump.

### Phase 6: Refactor The Rust Test Surface

Goal: keep the current coverage while making tests easier to extend.

Tasks:

- Expand `tests/support/mod.rs` with helpers such as:
  - `scan_rust_fixture(path)`
  - `assert_report_has_all(report, &[...])`
  - `assert_report_has_none(report, &[...])`
  - `assert_report_has_exact_subset(report, &[...])`
- Convert repeated Rust integration assertions in `tests/integration_scan/rust.rs` and `tests/integration_scan/rust_advanced.rs` to table-driven checks.
- Move ad hoc temp-dir setup in `tests/parser_invariants.rs` onto the shared support layer.
- Add fixture manifest helpers so tests express intent as "positive", "negative", "config-gated", or "invariant" instead of inline boilerplate.
- Keep CLI helpers small, but consider an `assert_cmd`-style wrapper if CLI cases continue to grow.

Acceptance criteria:

- Rust integration tests become shorter without losing clarity.
- Shared test helpers own most fixture setup.
- The difference between unit tests, invariants, and end-to-end scan tests is clearer.

### Phase 7: Add Structural Guardrails

Goal: stop the same structural debt from reaccumulating.

Tasks:

- Add a lightweight architecture guide for Rust module ownership.
- Add a maintenance test or script that reports the largest Rust source files.
- Add soft budgets for new modules, for example:
  - preferred: under 250 lines
  - review threshold: 350 lines
  - explicit justification: above 450 lines
- Add `cargo fmt --check` and `cargo clippy --all-targets --all-features` to routine validation if they are not already part of the workflow.
- Add a targeted Rust-only benchmark smoke check if Rust rule families become noticeably slower after refactors.

Important note:

- Line-count budgets should be advisory guardrails, not blind rules. Some parser or registry files may remain intentionally larger.

Acceptance criteria:

- The repo has at least one automated signal for structural regression.
- New large Rust files require intentional review, not accidental growth.

## Suggested Pull Request Sequence

Recommended order:

1. Add characterization tests and metadata ownership guards.
2. Split `src/analysis/rust/parser/items.rs`.
3. Split `src/analysis/rust/findings.rs`.
4. Introduce a Rust family registry and slim `src/analysis/rust/evaluate.rs`.
5. Normalize Rust metadata ownership.
6. Break up the three largest Rust heuristic families.
7. Refactor Rust integration tests onto richer shared helpers.
8. Add structural guardrails and cleanup docs.

This order keeps behavior-risking refactors behind stronger tests and avoids combining too many moving pieces in one PR.

## Non-Goals

This plan should not:

- redesign the product direction
- remove multi-language support
- replace tree-sitter
- change the public CLI contract unless a separate UX decision is made
- rewrite working rule logic just to make it "look cleaner"

## Success Metrics

The refactor should be considered successful when most of the following are true:

- `cargo test` remains green throughout the sequence.
- Rust-specific hotspot files are materially smaller and more focused.
- A new Rust rule family can be added with fewer touch points than today.
- Rust metadata ownership is uniform.
- Test additions require less boilerplate.
- The guides directory has one obvious plan of record for this effort.

## Expected Outcome

If the plan is executed well, the repo should move from roughly 8/10 to 9/10 territory for Rust structure and maintainability.

The key gain is not cosmetic neatness. It is that future work on Rust parsing, Rust rules, and Rust tests becomes cheaper, safer, and easier for contributors who did not author the original modules.
