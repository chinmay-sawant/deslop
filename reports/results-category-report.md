# Results Category Report

Source: `results.txt`

Current scan summary:

- Source files discovered: 80
- Source files analyzed: 79
- Functions fingerprinted: 766
- Total findings: 40
- Source findings: 0
- Fixture findings: 40
- Parse failures: 0
- Hallucination findings: 0

All remaining findings are intentional positives under `tests/fixtures/`.

Count by rule ID:

| Rule ID | Count |
| --- | ---: |
| rust_blocking_io_in_async | 8 |
| unsafe_without_safety_comment | 6 |
| rust_async_lock_order_cycle | 2 |
| rust_domain_raw_primitive | 2 |
| rust_hashmap_default_hasher | 2 |
| rust_async_hold_permit_across_await | 1 |
| rust_async_missing_fuse_pin | 1 |
| rust_async_recreate_future_in_select | 1 |
| rust_async_spawn_cancel_at_await | 1 |
| rust_async_std_mutex_await | 1 |
| rust_debug_secret | 1 |
| rust_domain_default_produces_invalid | 1 |
| rust_domain_float_for_money | 1 |
| rust_domain_impossible_combination | 1 |
| rust_lines_allocate_per_line | 1 |
| rust_lock_across_await | 1 |
| rust_serde_sensitive_deserialize | 1 |
| rust_serde_sensitive_serialize | 1 |
| rust_unbuffered_file_writes | 1 |
| rust_unsafe_assume_init | 1 |
| rust_unsafe_from_raw_parts | 1 |
| rust_unsafe_get_unchecked | 1 |
| rust_unsafe_raw_pointer_cast | 1 |
| rust_unsafe_set_len | 1 |
| rust_unsafe_transmute | 1 |