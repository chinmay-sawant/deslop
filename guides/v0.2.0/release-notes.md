# deslop v0.2.0 Release Notes

deslop `v0.2.0` turns the project from a Go-focused analyzer into a multi-language scanner for Go, Python, and Rust.

## Highlights

### Multi-language support

- deslop now scans Go, Python, and Rust repositories.
- The current inventory documents 976 language-scoped rules across common, Go, Python, and Rust rule families, including 653 Go rules.
- Mixed-language repository support is now part of the normal rollout and verification workflow.

### Rule registry and CLI improvements

- `rules/registry.json` is now the source of truth for rule metadata, rollout status, severity, and synced documentation output.
- deslop now exposes a `rules` CLI surface with filtering by language, status, and JSON output.
- `.deslop.toml` now provides practical rollout controls including disabled rules, severity overrides, suppressed paths, and experimental rule toggles.

### Go, Python, and Rust growth

- Go coverage has grown well beyond the original baseline with deeper request-path, hot-path, data-access, security, and framework-aware checks.
- Python support has become a substantial shipped backend with framework, async, duplication, packaging, maintainability, quality, and AI or MLOps-oriented rule families.
- Rust support has matured into a documented backend with hygiene, API-shape, runtime-boundary, async, domain-modeling, and unsafe-review signals.

### Validation and release discipline

- Real-repository validation is now part of the normal release story.
- The project now documents a corpus-based evaluation harness, promotion policy, release checklist, architecture guardrails, and inventory regression guards.
- Documentation is now organized as a versioned guide set, with archived `v0.1.0` material separated from the current `v0.2.0` docs.

### GitHub Action and tooling updates

- The GitHub Action now better reflects the broader product surface, with improved scan controls and release-install behavior.
- Documentation sync and corpus tooling are now part of the product’s day-to-day maintenance flow, not one-off release prep.

## By the numbers

- 214 commits landed on `master` since `v0.1.0`
- 58 commits touched markdown under `guides/`
- 85 guide markdown files changed in the `v0.2.0` documentation set
- 976 language-scoped rules are documented in the current inventory

## From v0.1.0 to v0.2.0

At `v0.1.0`, deslop was primarily a Go analyzer with an early heuristic set and benchmark support.

`v0.2.0` changes that in a few major ways:

- from Go-only scanning to Go, Python, and Rust
- from handwritten inventory descriptions to a registry-driven rule catalog
- from scattered phase notes to a clearer roadmap, promotion policy, and release discipline
- from fixture-only confidence to corpus-backed validation and rollout guidance

## Language summary

### Go

- The Go backend remains the broadest detector surface.
- `v0.2.0` adds much deeper request-path, framework, data-access, hot-path, and security coverage.
- Opt-in semantic Go checks now support deeper analysis without forcing those signals on every repository.

### Python

- Python has moved from roadmap work to a real shipped backend.
- The Python rule surface now spans framework-aware checks, async correctness, duplication, maintainability, quality, packaging, and AI or MLOps-oriented patterns.
- The parser, fixture, and rollout documentation around Python support is now substantially more mature.

### Rust

- Rust support has moved beyond an initial backend scaffold into a documented, test-backed rule surface.
- The current Rust rules cover hygiene, API design, runtime ownership, async patterns, domain modeling, and unsafe-review signals.
- Rust remains intentionally narrower and more selective than Go and Python.

## Documentation

See [README.md](README.md) for the `v0.2.0` guide index and the full roadmap, rollout, and backlog material for this release.
