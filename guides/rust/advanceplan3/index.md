# Rust Advance Plan 3 Coverage Audit

Date: 2026-04-05

## Purpose

This file is the coverage audit for Rust `advanceplan3`.

- [x] Every Rust-facing architecture review in `guides/architecture_enhancement/` was checked.
- [x] Every rule-worthy bad-practice scenario is now mapped to either a shipped Rust rule or an implemented `advanceplan3` rule.
- [x] Remaining architecture-only concerns are called out separately so they are not mistaken for missing scan rules.

## Source Reviews Audited

- [x] `RUST_ARCHITECTURE_REVIEW_1.md`
- [x] `RUST_ARCHITECTURE_REVIEW_2.md`
- [x] `RUST_ARCHITECTURE_REVIEW_3.md`
- [x] `RUST_ARCHITECTURE_REVIEW_4.md`
- [x] `RUST_ARCHITECTURE_REVIEW_5.md`
- [x] `RUST_ARCHITECTURE_REVIEW_6.md`
- [x] `RUST_ARCHITECTURE_REVIEW_7.md`
- [x] `RUST_ARCHITECTURE_REVIEW_8.md`
- [x] `RUST_ARCHITECTURE_REVIEW_9.md`
- [x] `RUST_ARCHITECTURE_REVIEW_10.md`
- [x] `RUST_ARCHITECTURE_REVIEW_11.md`
- [x] `RUST_ARCHITECTURE_REVIEW_12.md`

## Coverage Matrix

### Error, I/O, Config, And Manifest Boundaries

- [x] Public `anyhow`/`Box<dyn Error>` API surfaces are already covered by shipped rules:
  `rust_public_anyhow_result`, `rust_public_box_dyn_error`
- [x] Internal library `anyhow` usage is covered in backlog:
  `rust_internal_anyhow_result` in `plan1.md`
- [x] Unbounded file slurping is covered in backlog:
  `rust_unbounded_read_to_string` in `plan1.md`
- [x] Check-then-open filesystem race patterns are covered in backlog:
  `rust_check_then_open_path` in `plan1.md`
- [x] Path joining with absolute segments is already covered by shipped rule:
  `rust_path_join_absolute`
- [x] Secret equality comparison is covered in backlog:
  `rust_secret_equality_compare` in `plan1.md`
- [x] Suspicious numeric narrowing casts are covered in backlog:
  `rust_narrowing_numeric_cast` in `plan1.md`
- [x] Manual tempdir plus explicit cleanup plumbing is covered in backlog:
  `rust_manual_tempdir_lifecycle` in `plan1.md`
- [x] Unknown-key-tolerant config / request deserialization is already covered by shipped rule:
  `rust_serde_unknown_fields_allowed`
- [x] Missing Cargo workspace resolver is already covered by shipped rule:
  `rust_workspace_missing_resolver`
- [x] Release profile without `overflow-checks = true` is covered in backlog:
  `rust_release_profile_missing_overflow_checks` in `plan4.md`
- [x] Release profile still using unwind where abort is the intended operational default is covered in backlog:
  `rust_release_profile_panic_unwind` in `plan4.md`

### Module Ownership, File Shape, And Public Surface

- [x] Oversized catch-all modules are covered in backlog:
  `rust_oversized_module_file` in `plan2.md`
- [x] `mod.rs` dumping-ground modules are covered in backlog:
  `rust_mod_rs_catchall` in `plan2.md`
- [x] Public glob re-exports are covered in backlog:
  `rust_pub_use_glob_surface` in `plan2.md`
- [x] Flat crate-root re-export walls are covered in backlog:
  `rust_root_reexport_wall` in `plan2.md`
- [x] Repeated startup/bootstrap sequences are covered in backlog:
  `rust_duplicate_bootstrap_sequence` in `plan2.md`
- [x] Redundant sibling `#[path = "..."]` module attributes are covered in backlog:
  `rust_redundant_path_attribute` in `plan2.md`
- [x] Broad `#![allow(dead_code)]` masking is covered in backlog:
  `rust_broad_allow_dead_code` in `plan2.md`

### Runtime Ownership, Async Control, And Request-Path Setup

- [x] Tokio runtime creation per call is already covered by shipped rule:
  `rust_tokio_runtime_built_per_call`
- [x] Axum router construction inside handlers is already covered by shipped rule:
  `rust_axum_router_built_in_handler`
- [x] Tonic channel connect-per-request is already covered by shipped rule:
  `rust_tonic_channel_connect_per_request`
- [x] Heavy cloned state inside loops is already covered by shipped rule:
  `rust_clone_heavy_state_in_loop`
- [x] Env-var reads on request paths are already covered by shipped rule:
  `rust_env_var_read_in_request_path`
- [x] Detached spawn without supervision is covered in backlog:
  `rust_detached_spawn_without_handle` in `plan3.md`
- [x] Per-request channel/state creation is covered in backlog:
  `rust_channel_created_per_request` in `plan3.md`
- [x] Sync runtime bridging on request paths is covered in backlog:
  `rust_block_in_place_request_path` in `plan3.md`
- [x] Repeated runtime builder setup is covered in backlog:
  `rust_runtime_builder_in_loop` in `plan3.md`
- [x] Wait/notify without shutdown or cancellation shape is covered in backlog:
  `rust_notify_without_shutdown_contract` in `plan3.md`
- [x] Process-global env mutation as runtime control flow is covered in backlog:
  `rust_process_global_env_toggle` in `plan3.md`

### Ownership, Shared State, Unsafe, And Security Footguns

- [x] Public interior mutability exposure is already covered by shipped rule:
  `rust_pub_interior_mutability_field`
- [x] Global mutable lock state is already covered by shipped rule:
  `rust_global_lock_state`
- [x] `Rc<RefCell<_>>` domain ownership smell is already covered by shipped rule:
  `rust_rc_refcell_domain_model`
- [x] `Arc<Mutex<Option<T>>>` state-bag shape is already covered by shipped rule:
  `rust_arc_mutex_option_state`
- [x] `static mut` global state is covered in backlog:
  `rust_static_mut_global` in `plan4.md`
- [x] `Rc` parent/back-reference cycle shapes are covered in backlog:
  `rust_rc_cycle_parent_link` in `plan4.md`
- [x] Raw thread spawn calling async work without runtime handoff is covered in backlog:
  `rust_thread_spawn_async_without_runtime` in `plan4.md`
- [x] Unchecked slice/range splitting on external offsets is covered in backlog:
  `rust_split_at_unchecked_external_input` in `plan4.md`
- [x] Unchecked UTF-8 conversion at boundaries is covered in backlog:
  `rust_from_utf8_unchecked_boundary` in `plan4.md`
- [x] Existing unsafe primitive coverage is already shipped:
  `rust_unsafe_get_unchecked`, `rust_unsafe_from_raw_parts`, `rust_unsafe_set_len`,
  `rust_unsafe_assume_init`, `rust_unsafe_transmute`, `rust_unsafe_raw_pointer_cast`,
  `rust_unsafe_aliasing_assumption`

### Existing Shipped Rust Families Still Counted As Covered

- [x] Hygiene leftovers:
  `todo_macro_leftover`, `unimplemented_macro_leftover`, `dbg_macro_leftover`,
  `panic_macro_leftover`, `unreachable_macro_leftover`, `unwrap_in_non_test_code`,
  `expect_in_non_test_code`, `unsafe_without_safety_comment`
- [x] API and builder surface:
  `rust_borrowed_string_api`, `rust_borrowed_vec_api`, `rust_borrowed_pathbuf_api`,
  `rust_public_bool_parameter_api`, `rust_option_bag_config`, `rust_builder_without_validate`,
  `rust_constructor_many_flags`, `rust_partial_init_escape`, `rust_boolean_state_machine`
- [x] Serde and wire contracts:
  `rust_serde_untagged_enum_boundary`, `rust_serde_default_on_required_field`,
  `rust_serde_flatten_catchall`, `rust_serde_unknown_fields_allowed`,
  `rust_stringly_typed_enum_boundary`
- [x] Domain modeling:
  `rust_domain_raw_primitive`, `rust_domain_float_for_money`,
  `rust_domain_impossible_combination`, `rust_domain_default_produces_invalid`,
  `rust_domain_optional_secret_default`, `rust_debug_secret`,
  `rust_serde_sensitive_deserialize`, `rust_serde_sensitive_serialize`
- [x] Async/runtime/performance:
  `rust_blocking_io_in_async`, `rust_lock_across_await`, `rust_async_std_mutex_await`,
  `rust_async_hold_permit_across_await`, `rust_async_spawn_cancel_at_await`,
  `rust_async_missing_fuse_pin`, `rust_async_recreate_future_in_select`,
  `rust_async_lock_order_cycle`, `rust_async_monopolize_executor`,
  `rust_async_blocking_drop`, `rust_async_invariant_broken_at_await`,
  `rust_unbuffered_file_writes`, `rust_lines_allocate_per_line`,
  `rust_hashmap_default_hasher`, `rust_tokio_mutex_unnecessary`,
  `rust_blocking_drop`, `rust_pointer_chasing_vec_box`, `rust_aos_hot_path`,
  `rust_large_future_stack`, `rust_utf8_validate_hot_path`

## Explicit Non-Rule Architecture Concerns

These were reviewed and intentionally not turned into Rust repository scan rules because they are scanner-internal architecture/process concerns rather than target-repo code smells.

- [x] Catalog metadata drift guards and `binding_location` verification
- [x] Runtime registry generation and JSON/doc synchronization flow
- [x] Parser panic invariants and fuzz/property-test expectations
- [x] Cross-rule finding deduplication inside the scan engine
- [x] Shared-rule dispatch comments and backend orchestration shape
- [x] `rayon` vs async executor choice for the scanner itself
- [x] `const` rule arrays vs runtime registration for the scanner itself
- [x] Inventory guard counts and update-flow documentation
- [x] Informational `scan-*-info` make targets and benchmark CI policy

## Result

- [x] `advanceplan3` now implements the full rule-worthy Rust bad-practice backlog implied by the architecture review series.
- [x] Anything still outside `advanceplan3` is outside because it is not honestly a repo-scan rule candidate.
