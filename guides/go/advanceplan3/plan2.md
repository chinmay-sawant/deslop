# Plan 2 - SQL, GORM, And Database Access Performance Backlog (Go)

Date: 2026-03-31

## Status

- [x] Initial slice implemented on 2026-03-31.
- [x] Ordered GORM chain follow-up implemented on 2026-03-31 with real-repo validation.
- [x] This plan extends the current coarse DB-query heuristics with import-aware SQL, `database/sql`, `sqlx`, `pgx`, and `gorm` performance analysis.
- [x] The target set focuses on round-trip amplification, query-shape cost, ORM chain misuse, and request-path connection churn that common linter bundles usually do not model.

## Already Covered And Excluded From This Plan

- [x] `n_plus_one_query`
- [x] `wide_select_query`
- [x] `likely_unindexed_query`
- [x] `rows_without_close`
- [x] `stmt_without_close`
- [x] `tx_without_rollback_guard`
- [x] Context propagation between `Query` and `QueryContext` style APIs is already covered elsewhere.

## Objective

Move from method-name-only query spotting to framework-aware DB performance heuristics that understand connection churn, batching gaps, ORM chain shape, result width, and handler-driven query misuse. This phase should explicitly support `database/sql`, `jmoiron/sqlx`, `jackc/pgx`, and `gorm.io/gorm` without pretending to prove schema-level truth.

## Shipped Rules

- [x] `sql_open_per_request`
- [x] `gorm_open_per_request`
- [x] `db_ping_per_request`
- [x] `connection_pool_reconfigured_per_request`
- [x] `prepare_inside_loop`
- [x] `prepare_on_every_request_same_sql`
- [x] `tx_begin_per_item_loop`
- [x] `exec_inside_loop_without_batch`
- [x] `queryrow_inside_loop_existence_check`
- [x] `count_inside_loop`
- [x] `gorm_session_allocated_per_item`
- [x] `raw_scan_inside_loop`
- [x] `association_find_inside_loop`
- [x] `preload_inside_loop`
- [x] `first_or_create_in_loop`
- [x] `save_in_loop_full_model`
- [x] `update_single_row_in_loop_without_batch`
- [x] `delete_single_row_in_loop_without_batch`
- [x] `gorm_debug_enabled_in_request_path`
- [x] `create_single_in_loop_instead_of_batches`
- [x] `gorm_find_without_limit_on_handler_path`
- [x] `offset_pagination_on_large_table`
- [x] `gorm_preload_clause_associations_on_wide_graph`
- [x] `count_then_find_same_filter`

## Fixtures And Verification

- [x] Added `tests/fixtures/go/advanceplan3_data_positive.txt`.
- [x] Added `tests/fixtures/go/advanceplan3_data_clean.txt`.
- [x] Added `tests/integration_scan/go_advanceplan3.rs` coverage for the first SQL/GORM family.
- [x] Expanded the data fixtures with ordered `Limit`, `Offset`, `Preload`, `Count`, and `Find` chain cases.
- [x] Expanded the data fixtures with request-path `Ping` and pool-reconfiguration cases plus looped `Exec`, `QueryRow`, and GORM `Count` coverage.
- [x] Expanded the data fixtures with repeated request-path prepare calls, looped transactions, looped GORM sessions and preloads, raw scans, association loads, `FirstOrCreate`, `Save`, `Update*`, and `Delete` coverage.
- [x] Added parser regression coverage for ordered GORM chain summaries.
- [x] Verified with `cargo test --test integration_scan go_advanceplan3` and the full `cargo test --test integration_scan` suite.
- [x] Validated the second GORM wave against `go-admin-team/go-admin`; only two batch-insert findings surfaced in application code, so no extra suppression was needed before expanding the rule set.

## Candidate Scenario Backlog (50 scenarios)

### Connection, Pool, Session, And Statement Churn

- [x] `sql_open_per_request`: detect `sql.Open` or equivalent pool creation inside handlers, middleware, or looped work instead of process-level initialization.
- [x] `gorm_open_per_request`: detect `gorm.Open` in request or loop paths where the handle should be shared.
- [x] `db_ping_per_request`: detect `Ping` or `PingContext` on hot request paths rather than startup or health-check boundaries.
- [x] `prepare_inside_loop`: detect `Prepare` or `PrepareContext` inside loops where one prepared statement could serve the batch.
- [x] `prepare_on_every_request_same_sql`: detect the same literal SQL prepared repeatedly in a single request path.
- [x] `tx_begin_per_item_loop`: detect transactions started once per element instead of around the full batch.
- [x] `nested_transaction_in_request_path`: detect nested or repeated transactional scopes in request code without clear batching intent.
- [x] `gorm_session_allocated_per_item`: detect `db.Session(...)` or similar GORM session construction inside inner loops.
- [x] `gorm_debug_enabled_in_request_path`: detect `Debug()` or verbose query logger enabling inside hot request paths.
- [x] `connection_pool_reconfigured_per_request`: detect `SetMaxOpenConns`, `SetMaxIdleConns`, or `SetConnMaxLifetime` being changed at request time.

### Round-Trip Amplification And Missing Batch Paths

- [x] `exec_inside_loop_without_batch`: detect `Exec`, `ExecContext`, or ORM write terminals inside loops when the operation shape is batchable.
- [x] `queryrow_inside_loop_existence_check`: detect `QueryRow` / `First` / `Take` style existence checks inside loops when a bulk prefetch or `IN` query is plausible.
- [x] `select_or_get_inside_loop_lookup`: detect `sqlx.Get`, `sqlx.Select`, `Find`, or `First` style lookups inside loops over IDs or foreign keys.
- [x] `raw_scan_inside_loop`: detect repeated `Raw(...).Scan(...)` or raw-row scans inside loops.
- [x] `count_inside_loop`: detect `Count` calls inside loops or repeated paged handlers where a bulk count or cached total is more appropriate.
- [x] `association_find_inside_loop`: detect `Association(...).Find(...)` or association loaders inside per-row loops.
- [x] `preload_inside_loop`: detect GORM `Preload` setup or execution inside loops rather than once on the broader query.
- [x] `first_or_create_in_loop`: detect `FirstOrCreate` style ORM calls inside loops because each iteration can hide multiple queries.
- [x] `save_in_loop_full_model`: detect `Save` on full models per iteration when only a subset of columns actually changes.
- [x] `create_single_in_loop_instead_of_batches`: detect repeated `Create` of single rows when `CreateInBatches` or driver batching is a better fit.
- [x] `update_single_row_in_loop_without_batch`: detect repeated `Updates`, `UpdateColumn`, or raw `UPDATE` statements in loops.
- [x] `delete_single_row_in_loop_without_batch`: detect repeated row deletes in loops when a set-based delete is available.
- [x] `row_by_row_upsert_loop`: detect repeated upsert-style writes rather than bulk conflict handling.
- [x] `repeated_same_query_template_same_function`: detect the same query template executed multiple times with only scalar parameter changes and no batching.
- [x] `count_then_find_same_filter`: detect a `Count` followed by `Find` or `Select` with effectively the same filter chain in one request path.
- [x] `exists_via_count_star`: detect `COUNT(*)` usage where the call site only needs a boolean existence answer.
- [x] `find_all_then_manual_paginate_in_go`: detect unbounded fetches followed by in-memory page slicing in Go.
- [x] `duplicate_find_then_preload_followup`: detect an initial ORM fetch followed by separate follow-up fetches for associations that could be folded into one query plan.

### Query Shape, Pagination, And Result Width

- [x] `gorm_find_without_limit_on_handler_path`: detect request-path `Find` chains with no visible `Limit`, `First`, `Take`, `Rows`, or batching marker.
- [x] `gorm_preload_clause_associations_on_wide_graph`: detect `Preload(clause.Associations)` or very broad preload graphs on handler-backed queries.
- [x] `gorm_select_missing_projection_on_wide_model`: detect wide model fetches with no `Select` or `Omit` when the handler only reads a narrow field subset.
- [x] `gorm_joins_plus_preload_plus_find_without_limit`: detect expensive query chains that combine `Joins`, `Preload`, and broad `Find` terminals without bounding the result set.
- [x] `order_by_without_limit_orm_chain`: detect `.Order(...)` chains with no bounding clause on latency-sensitive reads.
- [x] `offset_pagination_on_large_table`: detect `.Offset(...)` pagination without keyset cues on obviously hot list endpoints.
- [x] `order_by_random_request_path`: detect `ORDER BY RAND()`, `ORDER BY RANDOM()`, or equivalent random ordering in request paths.
- [x] `distinct_wide_row_request_path`: detect `Distinct` on wide row projections in request handlers where a key-only subquery would be cheaper.
- [x] `lower_or_func_wrapped_indexed_column`: detect GORM or query-builder filter expressions that wrap columns in `LOWER`, `UPPER`, `COALESCE`, or similar functions in ways that often defeat indexes.
- [x] `date_or_cast_wrapped_indexed_column`: detect `DATE(column)`, `CAST(column AS ...)`, or similar wrapped-column filters on hot paths.
- [x] `leading_wildcard_builder_chain`: detect ORM-builder or raw query chains that contain leading-wildcard search patterns and no evident narrowing clause.
- [x] `unbounded_in_clause_expansion`: detect extremely large `IN (?)` or placeholder-expanded clauses built directly from request collections.
- [x] `many_column_or_filter_chain`: detect large `OR` chains across many columns in handler-driven list/search paths.

### Scan Shape, Materialization, And ORM Tuning

- [x] `scan_into_map_string_any_hot_path`: detect scanning rows into `map[string]any`, `[]map[string]any`, or similarly dynamic shapes in hot code.
- [x] `rows_to_struct_allocation_per_row_without_reuse`: detect per-row struct allocation helpers when a reusable destination or callback path would be clearer and cheaper.
- [x] `sqlx_select_large_slice_without_limit`: detect `sqlx.Select` into slices with no visible limit, cursor, or batching shape.
- [x] `pgx_collectrows_unbounded_materialization`: detect `pgx.CollectRows` or similar helpers that materialize full result sets in request code.
- [x] `default_transaction_enabled_for_bulk_create`: detect high-volume GORM create paths that do not opt out of the default transaction when safe bulk insertion patterns already exist.
- [x] `save_for_single_column_change`: detect `Save` on full models when the call site only mutates one or two fields.
- [x] `updates_map_allocated_per_row`: detect `Updates(map[string]any{...})` style row-by-row update maps allocated inside loops.
- [x] `findinbatches_candidate_for_large_scan`: detect broad table scans or wide list handlers that would be better served by `FindInBatches`, cursor iteration, or driver streaming.
- [x] `automigrate_or_schema_probe_in_request_path`: detect `AutoMigrate`, schema introspection, or migration-like probes in request or loop paths.

## Shared Implementation Checklist

- [ ] Replace the current method-name-only DB classifier with an import-aware classifier keyed by `database/sql`, `sqlx`, `pgx`, and `gorm` symbols.
- [x] Add a `GoQueryChainStep` style summary so GORM call sequences can be analyzed as ordered chains instead of isolated calls.
- [x] Capture terminal operations, pagination clauses, preload breadth, projection clauses, and write-batch cues for ORM chains.
- [x] Add request-path detection so `missing limit` and broad materialization findings can stay quiet for CLI tools, migrations, and offline jobs.
- [ ] Add representative fixtures for `database/sql`, `sqlx`, `pgx`, and `gorm` separately so false positives can be isolated by framework.
- [ ] Validate on at least one real GORM-heavy repo and one lower-level SQL repo before enabling any new rule set by default.
- [x] Validate the ordered GORM chain rules on at least one real GORM-heavy repository before expanding their scope.

## Acceptance Criteria

- [x] Each new rule explains whether the cost comes from connection churn, query multiplicity, result width, pagination choice, or ORM chain breadth.
- [x] Clean fixtures that already use batching, keyset pagination, explicit projection, or process-level pool reuse stay quiet.
- [x] No rule claims to prove missing indexes or wrong schema design; all messages remain heuristic and explainable.