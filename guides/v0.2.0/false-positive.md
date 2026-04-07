# Enhancement Backlog

This note captures the confirmed-real findings from `results.txt` after filtering the hallucination classes we verified.

## Backlog Summary

| Area | Real findings | What to tackle next |
| --- | ---: | --- |
| `scripts/` | 30 | Tighten input validation, simplify loop-heavy string building, and reduce orchestration complexity. |
| `src/analysis/` | 30 | Trim parser/module surfaces and keep the language frontends focused. |
| `src/heuristics/go/` | 45 | Shorten overlong helper names and keep the Go heuristics easier to navigate. |
| `src/heuristics/python/` | 32 | Simplify hot-path and quality helpers, especially long parser/heuristic functions. |
| `src/heuristics/rust/` | 14 | Continue refining module-surface, runtime-ownership, and security-footgun checks. |
| `src/index/`, `src/scan/`, `src/cli/`, `src/model/`, `src/rules/catalog/`, `src/lib.rs` | 23 | Finish the shared facade cleanup and keep the public surface small. |

## Suggested Work Order

1. Finish the `scripts/` cleanup first, because those fixes are narrow and will make the maintenance scripts easier to trust.
2. Work through `src/heuristics/rust/` next, since it is the area most likely to affect future false positives and rule quality.
3. Then continue with the larger parser and heuristic surfaces in `src/analysis/`, `src/heuristics/go/`, and `src/heuristics/python/`.
4. Leave the facade and export cleanup in `src/index/`, `src/scan/`, `src/cli/`, `src/model/`, `src/rules/catalog/`, and `src/lib.rs` for a follow-up pass.

## Notes

- The row-level source of truth remains `results.txt`.
- The scanner logic has already been tightened for the confirmed Rust false positives around oversized module detection and `#[path]` handling.
- If you want to turn this into a working checklist later, the quickest next step is to split the backlog into one file per area and mark items off as they land.
