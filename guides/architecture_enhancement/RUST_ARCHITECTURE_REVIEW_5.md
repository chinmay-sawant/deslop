---
title: "Rust Architecture Enhancement Plan (Part 5)"
description: "Implementation plan for architectural improvements resolving metadata coupling, string allocation overhead, and interface consistency"
---

## Phase 1: Decoupling Metadata Configuration
- [x] Migrate experimental `status` overrides for Rust Async rules to their specific actual `RuleDefinition` in the `rust` catalog.
- [x] Migrate experimental `status` overrides for Go Semantic rules to their specific `RuleDefinition` in the `go` catalog.
- [x] Migrate `configurability` options (like `RustAsyncExperimental`, `GoSemanticExperimental`) directly into the catalog definitions.
- [x] Delete `apply_runtime_policy`, `status_for_rule` and `configurability_for_rule` from `src/rules.rs`.

## Phase 2: Eliminating Static String Heap Allocations
- [x] Refactor `RuleMetadata` struct in `src/rules.rs` to use `&'static str` instead of `String` for `id`, `family`, `description`.
- [x] Refactor `RuleMetadata` struct `configurability` field to use `&'static [RuleConfigurability]` instead of `Vec<RuleConfigurability>`.
- [x] Update `rule_metadata_from_definition` mapping logic to prevent cloning operations.
- [x] Resolve any potential compiler errors in tests or usages caused by the lifetime changes.

## Phase 3: CLI Handler Refactoring
- [x] Extract the `Scan` command execution logic into a dedicated function `execute_scan` in `src/cli/mod.rs` (or a similar suitable module).
- [x] Extract the `Bench` command execution logic into `execute_bench`.
- [x] Extract the `Rules` command execution logic into `execute_rules`.
- [x] Simplify `src/main.rs` to strictly handle routing logic.

## Phase 4: Interface Inconsistencies Fixes
- [x] Convert `go::rule_definitions()` in `src/rules/catalog/go/mod.rs` to a static constant slice `go::RULE_DEFINITIONS` if possible.
- [x] Convert `python::rule_definitions()` in `src/rules/catalog/python/mod.rs` to a static constant slice `python::RULE_DEFINITIONS` if possible.
- [x] Update `rule_catalog()` in `src/rules/catalog/mod.rs` to use these constants.
