# Rust Module Ownership

This note documents the refactor seams introduced in the Rust codebase so future changes keep responsibilities narrow.

## Current Ownership

- `src/analysis/mod.rs` is the facade for analysis concerns only. Backend registration now lives in `src/analysis/backend.rs` and analysis configuration in `src/analysis/config.rs`.
- `src/analysis/types/` owns the internal analysis IR. `core.rs` contains `ParsedFile` and `ParsedFunction`, while `common.rs`, `go.rs`, `python.rs`, and `rust.rs` own supporting language and shared types.
- `src/analysis/python/parser/` now separates non-function AST collection in `general.rs` from function assembly in `functions.rs`. Function assembly is intentionally split into shape collection and evidence collection.
- `src/analysis/rust/mod.rs` is a thin backend facade. Rust-specific evaluation now lives in `src/analysis/rust/evaluate.rs`.
- `src/scan/mod.rs` is the scan facade. Stage details live in `src/scan/file_analysis.rs`, `src/scan/evaluate.rs`, and `src/scan/suppression.rs`.
- `src/index/mod.rs` owns repository-index types. Build logic lives in `src/index/build.rs`, and resolution logic lives in `src/index/resolve.rs`.
- `src/heuristics/mod.rs` is the heuristics facade. Dispatch logic lives in `src/heuristics/engine.rs`, and grouped rule registration lives in `src/heuristics/registry.rs`.
- `src/heuristics/go_advanceplan3/mod.rs` is now a family facade. Gin request rules live in `gin.rs`, data-access rules live in `data_access.rs`, and hot-path or repeated-work rules live in `hot_path.rs`.
- `ParsedFunction` stores language-specific evidence in nested owned structs — `go: Option<GoFunctionEvidence>`, `python: Option<PythonFunctionEvidence>`, `rust: Option<RustFunctionEvidence>`. Each parser populates exactly one. The borrow-view accessors `go_evidence()`, `python_evidence()`, `rust_evidence()` delegate to the owned struct and return empty views for the `None` case.

## Rules For Future Changes

- Add parser tree walking and syntax extraction under `analysis/<language>/parser/`.
- Add rule-support evidence in the language parser assembly layer, not in scan or CLI code.
- Add evaluation orchestration in `heuristics/engine.rs` or `analysis/<language>/evaluate.rs`, not in backend registration modules.
- Add repository symbol construction in `index/build.rs` and symbol resolution rules in `index/resolve.rs`.
- Keep `scan/mod.rs`, `analysis/mod.rs`, `index/mod.rs`, and `heuristics/mod.rs` as facades rather than implementation dumps.
- All language-specific evidence must go in `GoFunctionEvidence`, `PythonFunctionEvidence`, or `RustFunctionEvidence`. Adding flat fields to `ParsedFunction` directly is not allowed.
- All heuristic/evaluator/index code must read language-specific evidence through `go_evidence()`, `python_evidence()`, or `rust_evidence()`.

## Near-Term Follow-Up

- The second-stage IR storage split (flat → nested owned structs) is complete.
- Continue shrinking other oversized Go-specific rule files using the facade plus family-module split used in `heuristics/go_advanceplan3/`.
