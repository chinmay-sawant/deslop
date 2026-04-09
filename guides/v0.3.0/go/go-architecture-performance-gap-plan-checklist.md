# Go Architecture And Performance Gap Plan Checklist

Date: 2026-04-09

## Audit

- [x] Re-check the repo-review themes against the live `rules/registry.json`.
- [x] Separate already-covered themes from true net-new Go rule work.
- [x] Keep the `v0.3.0` Go guide content repo-agnostic.

## Documentation

- [x] Replace the repo-specific `more-architecture-missed.rules.md` content with a generic gap audit.
- [x] Record the final list of new architecture/context rule IDs and rule enhancements.
- [x] Record the final category breakdown for the 100 new Go performance rules.

## Architecture And Context Implementation

- [x] Extend `context_background_used` so request handlers with implicit request contexts are covered.
- [x] Add `client_input_error_mapped_to_internal_server_error`.
- [x] Add `cache_interface_method_missing_context`.
- [x] Add `cache_method_uses_context_background`.
- [x] Add positive and clean fixtures for the new context and architecture coverage.
- [x] Add or update integration tests for the new context and architecture coverage.

## Performance Expansion

- [x] Add a new Go performance heuristic module for the extra performance pack.
- [x] Register 100 new Go performance rule definitions without duplicating the current registry.
- [x] Add grouped positive and clean Go fixtures for the new performance rules.
- [x] Add grouped integration tests that assert the new performance rules are present and cleanly suppressed.

## Verification

- [x] Regenerate `rules/registry.json` from the Rust-backed catalog.
- [x] Update rule-count invariants that depend on the catalog size.
- [x] Run targeted Rust tests for context, Go architecture, Go performance, and registry invariants.
- [x] Mark this checklist complete only after the implementation and verification steps pass.
