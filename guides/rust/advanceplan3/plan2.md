# Plan 2 - Module Ownership And Surface Architecture (Rust)

Date: 2026-04-05

## Status

- [x] Implemented in the Rust scan pipeline, heuristics tree, and catalog registry.
- [x] Backlog derived from the current code layout, test support structure, and the Rust architecture review series.
- [x] File-metric thresholds, fixture conventions, and catalog entries are now wired up.

## Objective

Add a Rust architecture-oriented rule family that catches catch-all module growth, noisy export surfaces, and duplicate bootstrap shapes that make real Rust projects harder to review and evolve.

## Source Inputs

- [x] `src/rules/catalog/`
- [x] `src/analysis/mod.rs`
- [x] `tests/support/mod.rs`
- [x] `tests/integration_scan/mod.rs`
- [x] `guides/rust/module-ownership.md`
- [x] `guides/architecture_enhancement/RUST_ARCHITECTURE_REVIEW_1.md`
- [x] `guides/architecture_enhancement/RUST_ARCHITECTURE_REVIEW_2.md`
- [x] `guides/architecture_enhancement/RUST_ARCHITECTURE_REVIEW_3.md`
- [x] `guides/architecture_enhancement/RUST_ARCHITECTURE_REVIEW_10.md`
- [x] `guides/architecture_enhancement/RUST_ARCHITECTURE_REVIEW_11.md`

## Existing Coverage Explicitly Excluded

- [x] `rust_public_bool_parameter_api`
- [x] `rust_pub_interior_mutability_field`
- [x] `rust_global_lock_state`
- [x] `rust_option_bag_config`

## Proposed Rules

- [x] `rust_oversized_module_file`
  - Flag `lib.rs`, `main.rs`, `mod.rs`, or module files that mix too many top-level items and exceed a conservative architecture threshold.
- [x] `rust_pub_use_glob_surface`
  - Flag `pub use foo::*;` style public glob re-exports on crate or subsystem boundaries.
- [x] `rust_root_reexport_wall`
  - Flag crate roots that flatten too many unrelated exports into one public surface block.
- [x] `rust_mod_rs_catchall`
  - Flag `mod.rs` files that behave like subsystem dumping grounds instead of narrow composition points.
- [x] `rust_duplicate_bootstrap_sequence`
  - Flag repeated startup or orchestration sequences across multiple entry-style functions or files.
- [x] `rust_redundant_path_attribute`
  - Flag `#[path = "..."]` module attributes that point to same-directory siblings where standard Rust module resolution would suffice.
- [x] `rust_broad_allow_dead_code`
  - Flag broad file- or module-level `#![allow(dead_code)]` suppression that masks unfinished wiring or oversized facades.

## Parser And Evidence Work

- [x] Reused existing line counts, item counts, and import summaries before inventing deeper architecture models.
- [x] Treated file-size and top-level-item thresholds as contextual heuristics, not hard correctness claims.
- [x] Kept duplicate-bootstrap detection text-driven and scoped to obvious setup patterns such as repeated router/runtime/client wiring.

## Fixtures And Tests

- [x] Added dedicated architecture fixtures under `tests/fixtures/rust/advanceplan3/`.
- [x] Added clean coverage for small facade modules, intentional grouped exports, and narrow `mod.rs` coordinators.
- [x] Added one multi-file fixture where the crate root stays clean but a deep subsystem module is intentionally oversized.

## Acceptance

- [x] Findings explain the structure smell in filesystem terms a reviewer can validate quickly.
- [x] Threshold-based rules stay conservative enough to avoid punishing normal facade modules.
- [x] The family reinforces the project's documented split-by-concern design without forcing one crate layout on every repo.
- [x] `#[path]` and dead-code findings stay focused on obviously broad or redundant usage rather than legitimate special-case module wiring.

## Non-Goals

- [ ] Formal architectural layering proofs.
- [ ] Full clone detection across bootstrap code.
- [ ] Enforcing a single preferred module tree for every Rust project.
