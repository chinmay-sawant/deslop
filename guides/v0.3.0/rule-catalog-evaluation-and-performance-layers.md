# Rule Catalog Evaluation And Performance Layers

Date: 2026-04-27

## Scope

Evaluated the checked-in rule registry generated from `src/rules/catalog` and extended the Go, Python, and Rust performance catalogs with 100 additional language-specific performance-layer rules each.

## Inventory

Baseline registry before this expansion:

| Language | Rules |
| --- | ---: |
| Common | 11 |
| Go | 653 |
| Python | 591 |
| Rust | 250 |
| Total | 1,505 |

Registry after this expansion:

| Language | Rules |
| --- | ---: |
| Common | 11 |
| Go | 753 |
| Python | 691 |
| Rust | 350 |
| Total | 1,805 |

Performance-family counts after this expansion:

| Language | Performance rules |
| --- | ---: |
| Go | 261 |
| Python | 129 |
| Rust | 112 |

## Duplicate Review

The registry has no duplicate `(language, rule_id)` entries. The existing `registry_is_unique_per_language_and_sorted` guard covers this mechanically, and the evaluation also checked normalized rule descriptions for duplicate-like intent.

Duplicate-like clusters found in the existing catalog were retained because they are intentional sibling rules, not exact duplicates:

| Cluster | Decision |
| --- | --- |
| Go `bytes.*` and `strings.*` variants | Keep separate. The APIs, evidence text, and implementation hooks differ by package. |
| Go per-call setup rules for AWS, gRPC, loggers, metrics, Viper, and environment lookups | Keep separate. Same performance principle, different library evidence and remediation path. |
| Go checksum, hashing, timer, and encoding calls inside loops | Keep separate. Same layer, different concrete APIs. |
| Go/Python `full_dataset_load` | Keep language-scoped variants. The concept is shared, but detectors and examples are language-specific. |
| Go security rules with similar threshold wording | Keep separate. They cover distinct cryptographic or TLS APIs. |

No existing rules were deleted or merged in this pass because the duplicate-like groups encode useful API-specific behavior.

## Expansion Model

Each language received exactly 100 new performance rules, organized as 20 layers with five rules per layer:

1. Algorithmic complexity
2. Data structure choice
3. Memory allocation
4. Garbage collection and cleanup
5. String handling
6. Collection iteration
7. Async and concurrency
8. I/O operations
9. Database access
10. Network calls
11. Caching
12. Serialization and deserialization
13. Logging overhead
14. Error handling cost
15. Build and runtime configuration
16. Hot path optimization
17. Lazy loading
18. Resource pooling
19. Framework-specific performance
20. Profiling and benchmarking

The new IDs use language-specific namespaces:

| Language | New ID prefix | Source |
| --- | --- | --- |
| Go | `go_perf_layer_` | `src/rules/catalog/go/performance_layers.rs` |
| Python | `python_perf_layer_` | `src/rules/catalog/python/performance_layers.rs` |
| Rust | `rust_perf_layer_` | `src/rules/catalog/rust/performance_layers.rs` |

This keeps the new rules easy to audit and prevents collisions with existing detector-backed rule IDs.

## Quality Notes

The new rules are catalogued as stable, contextual performance metadata. They are intentionally language-specific instead of generic copies:

- Go rules emphasize goroutines, channels, `http.Client`/transport reuse, `database/sql`, `bufio`, `sync.Pool`, Gin/GORM/sqlx/gRPC, and Go benchmarking pitfalls.
- Python rules emphasize iterators, pandas/NumPy, async routes, Requests/session reuse, ORM behavior, Pydantic/FastAPI/Django/SQLAlchemy/Celery, and Python benchmarking pitfalls.
- Rust rules emphasize ownership, cloning, allocation, async executor behavior, `reqwest`/Hyper, `sqlx`, `serde`, `OnceLock`, Axum/Actix/Tonic/Askama, and Criterion benchmarking pitfalls.

Follow-up implementation work can promote any of these catalog rules into detector-backed heuristics by adding evidence extraction and fixtures for the specific language/runtime surface.
