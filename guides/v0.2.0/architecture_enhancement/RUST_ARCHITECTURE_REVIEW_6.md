# Rust Architecture Review 6

**Rating:** 9.5 / 10

## Overview
Following the successful implementation of the 4-phase refactor from `RUST_ARCHITECTURE_REVIEW_5`, the codebase's architectural integrity is fantastic. We eliminated static heap allocations, decentralized our metadata directly into the language catalogs, untangled the CLI router interface, and flattened rule slice aggregation. The application leverages zero-cost `&'static` abstractions, leverages `rayon` for fast concurrent file parsing, and uses robust `tree-sitter` backend processing.

However, the architecture does not achieve a perfect 10/10 yet due to a remaining hardcoded metadata leak and sequential algorithmic bottlenecks during the assessment phase. 

## Areas Preventing a 10/10 Rating

### 1. Leaked Metadata Policy Check (`src/scan/evaluate.rs`)
Despite removing hardcoded rule ID checks in `src/rules.rs`, a policy mapping leak persists in `apply_repository_config`. The helper function `is_async_rollout_rule` actively uses string slice matching (`rule_id.starts_with("rust_async_")` and `matches!(rule_id, "rust_lock_across_await"...)`) instead of referencing the canonical `RuleConfigurability::RustAsyncExperimental` variant we successfully embedded into the catalog bindings during Phase 1. 

### 2. Sequential Heuristic Traversals
While the I/O and tree-sitter parsing phase (`analyze_discovered_files`) runs concurrently via `rayon`, the `evaluate_findings` methodology inherently iterates *sequentially* across the `files` array when deploying language heuristics:
```rust
for file in files {
    if let Some(backend) = backend_for_language(file.language) {
        findings.extend(backend.evaluate_file(file, index, analysis_config));
    }
}
```
For deep monorepo analysis, traversing the parsed file ASTs sequentially blocks available CPU threads and creates a noticeable bottleneck.

### 3. Abstract AST Caching Missing
Currently, rerunning scans forces the entire directory graph to be ingested mapping tree-sitter ASTs repeatedly natively in memory. Implementing file-level structural hash caching (e.g. `.deslop/cache`) would save thousands of CPU cycles across identical unchanged developer environments.

## Next Steps Checklist

### Phase 5: Final Policy Cleanup
- [x] Replace `is_async_rollout_rule` in `src/scan/evaluate.rs` to fetch the rule cleanly via `rule_registry()` and natively check for `RuleConfigurability::RustAsyncExperimental`, subsequently deleting the hardcoded string matching.

### Phase 6: Thread Pool Enhancements
- [x] Parallelize the `evaluate_file` loops in `evaluate_findings` using `rayon`'s parallel iterators (`.par_iter().flat_map(...)`).
