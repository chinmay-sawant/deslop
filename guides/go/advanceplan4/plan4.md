# Plan 4 — Implementation Roadmap, Priority Order, And Backlog Summary (Go)

Date: 2026-04-03

## Status

- [x] Completed on 2026-04-03.
- [x] The `advanceplan4` Go rollout now ships all 132 planned rules across plans 1 through 3.
- [x] Grouped fixture coverage ships in `tests/fixtures/go/advanceplan4_*_{positive,clean}.txt`.
- [x] Grouped integration coverage ships in `tests/integration_scan/go_advanceplan4.rs`.
- [x] The detailed draft bullets below are retained as the original roadmap inventory; the completion summary here is the source of truth.

## Backlog Summary

| Plan | Focus Area | Rules | Category Breakdown |
|------|-----------|-------|-------------------|
| `plan1.md` | Low-Level Performance | 50 | Strings (12), Slices/Maps (13), Runtime/Sync (10), I/O/Encoding (8), Errors/Interfaces (7) |
| `plan2.md` | Security Worst Practices | 52 | Crypto (10), Injection (10), Auth/Session (8), Concurrency Security (7), Network/TLS (8), Data Exposure (9) |
| `plan3.md` | Library-Specific | 30 | Redis (6), gRPC (5), Logging (5), Config/CLI (5), Prometheus (4), AWS SDK (5) |
| **Total** | | **132** | |

## Rule Count vs Target

- [x] Target: 100+ new worst practices
- [x] Shipped: 132 unique rules across 3 plans
- [x] Buffer preserved: 32 extra rules beyond the 100 target, with no cuts required in the final implementation slice

## Dependency Analysis

### No New Parser Evidence Required (can ship immediately)

These rules only need `body_lines()`, `import_aliases_for()`, `body_text` matching, and existing `ParsedFunction` fields:

- [x] All 12 string rules (plan1, Section A)
- [x] All 13 slice/map rules (plan1, Section B)
- [x] All 10 runtime/sync rules (plan1, Section C)
- [x] All 8 I/O/encoding rules (plan1, Section D)
- [x] All 7 error/interface rules (plan1, Section E)
- [x] 8 out of 10 crypto rules (plan2, Section A — A1 through A7, A10)
- [x] All 10 injection rules (plan2, Section B)
- [x] 6 out of 8 auth/session rules (plan2, Section C — C1 through C6)
- [x] 4 out of 7 concurrency security rules (plan2, Section D — D1, D2, D5, D6)
- [x] All 8 network/TLS rules (plan2, Section E)
- [x] All 9 data exposure rules (plan2, Section F)
- [x] All 5 logging rules (plan3, Section C)
- [x] All 5 config/CLI rules (plan3, Section D)
- [x] All 4 Prometheus rules (plan3, Section E)

**Shipped immediately with existing evidence: ~109 rules**

### Requires Minor Import Resolution Extension

These rules need `import_aliases_for()` support for libraries not currently in the alias table:

- [x] Redis rules (plan3, Section A) — need `github.com/go-redis/redis` / `github.com/redis/go-redis` alias resolution
- [x] gRPC rules (plan3, Section B) — need `google.golang.org/grpc` alias resolution
- [x] AWS SDK rules (plan3, Section F) — need `github.com/aws/aws-sdk-go` alias resolution
- [x] JWT rules (plan2, A8, C5) — need `github.com/golang-jwt/jwt` alias resolution

**Shipped after minor alias handling and explicit import-path checks: 132 rules (all)**

### No SSA, Type Checking, Or Cross-Package Analysis Required

All 132 rules can be implemented with:
- [x] Function-local body-text pattern matching via `body_lines()`
- [x] Import resolution via `import_aliases_for()`
- [x] Composite literal block extraction
- [x] Binding pattern matching (existing `binding_matches()` helper)
- [x] Handler path detection (existing `*gin.Context` / `http.Handler` heuristics)

## Suggested Implementation Waves

### Wave 1 — Highest Confidence Performance Rules (25 rules)

Priority: highest confidence, broadest applicability, minimal false-positive risk.

- [x] Wave 1 is complete.

- [x] `sprintf_for_simple_int_to_string` (A4) — clear pattern, measurable speedup
- [x] `sprintf_for_simple_string_format` (A5) — clear pattern, measurable speedup
- [x] `string_format_for_error_wrap` (A9) — both performance and correctness
- [x] `strings_hasprefix_then_trimprefix` (A10) — Go 1.20+ optimization
- [x] `strings_hassuffix_then_trimsuffix` (A11) — Go 1.20+ optimization
- [x] `string_builder_write_string_vs_plus` (A12) — builder misuse
- [x] `map_lookup_double_access` (B5) — clear waste
- [x] `slice_grow_without_cap_hint` (B6) — well-known antipattern
- [x] `range_copy_large_struct` (B12) — measurable for structs > 64 bytes
- [x] `sort_slice_vs_sort_sort` (B3) — Go 1.21+ optimization
- [x] `clear_map_go121` (B9) — Go 1.21+ optimization
- [x] `sync_mutex_for_atomic_counter` (C1) — lock-free optimization
- [x] `defer_in_tight_loop` (C6) — defer cost accumulation
- [x] `select_with_single_case` (C7) — unnecessary overhead
- [x] `waitgroup_add_inside_loop` (C10) — atomic batching
- [x] `ioutil_readall_still_used` (D1) — deprecated API
- [x] `json_marshal_then_write` (D2) — streaming optimization
- [x] `csv_reader_reuse_record` (D6) — allocation reduction
- [x] `errors_new_for_static_sentinel` (E3) — alloc reduction
- [x] `fmt_errorf_without_wrap_verb` (E4) — error chain preservation
- [x] `error_string_comparison` (E5) — performance + correctness
- [x] `type_assertion_without_comma_ok` (E1) — panic safety
- [x] `type_switch_vs_repeated_assertions` (E2) — jump table optimization  
- [x] `os_getenv_in_hot_path` (D2, plan3) — lock avoidance
- [x] `viper_get_in_hot_path` (D1, plan3) — mutex avoidance

### Wave 2 — Critical Security Rules (25 rules)

Priority: highest-impact vulnerabilities, clear detection patterns.

- [x] Wave 2 is complete.

- [x] `insecure_random_for_security` (A1) — crypto vs math/rand
- [x] `hardcoded_tls_skip_verify` (A2) — MITM vulnerability
- [x] `hardcoded_tls_min_version_too_low` (A3) — protocol downgrade
- [x] `constant_encryption_key` (A5) — key in binary
- [x] `constant_iv_or_nonce` (A6) — nonce reuse
- [x] `bcrypt_cost_too_low` (A9) — brute-force weakness
- [x] `rsa_key_size_too_small` (A10) — key length
- [x] `os_exec_command_with_user_input` (B1) — command injection
- [x] `template_html_unescaped` (B2) — XSS
- [x] `text_template_for_html` (B3) — XSS
- [x] `filepath_join_with_user_path` (B4) — path traversal
- [x] `url_redirect_without_validation` (B5) — open redirect
- [x] `ssrf_via_user_controlled_url` (B6) — SSRF
- [x] `xml_decoder_without_entity_limit` (B9) — XXE
- [x] `cookie_without_secure_flag` (C1) — cookie theft
- [x] `cookie_without_httponly` (C2) — XSS cookie theft
- [x] `cors_allow_all_origins` (C4) — CORS misconfiguration
- [x] `timing_attack_on_token_comparison` (C6) — timing side-channel
- [x] `http_body_readall_without_limitreader` (D8, plan1) — DoS
- [x] `sensitive_data_in_log` (F1) — data exposure
- [x] `error_detail_leaked_to_client` (F2) — info disclosure
- [x] `debug_endpoint_in_production` (F3) — pprof exposure
- [x] `struct_field_exposed_in_json` (F4) — data leak
- [x] `temp_file_predictable_name` (F5) — symlink attack
- [x] `world_readable_file_permissions` (F6) — permission flaw

### Wave 3 — Library-Specific Rules (20 rules)

Priority: popular libraries, common mistakes, measurable impact.

- [x] Wave 3 is complete.

- [x] `redis_ping_per_request` (A1, plan3)
- [x] `redis_get_set_without_pipeline` (A2, plan3)
- [x] `redis_keys_command_in_handler` (A3, plan3)
- [x] `redis_connection_per_request` (A4, plan3)
- [x] `redis_no_ttl_on_cache_keys` (A6, plan3)
- [x] `grpc_dial_per_request` (B1, plan3)
- [x] `grpc_context_not_propagated` (B3, plan3)
- [x] `grpc_without_tls_credentials` (E6, plan2)
- [x] `log_level_check_after_format` (C1, plan3)
- [x] `logger_created_per_request` (C2, plan3)
- [x] `string_format_in_structured_logger` (C3, plan3)
- [x] `log_printf_for_production` (C4, plan3)
- [x] `error_logged_and_returned` (C5, plan3)
- [x] `prometheus_counter_created_per_request` (E1, plan3)
- [x] `prometheus_high_cardinality_labels` (E2, plan3)
- [x] `aws_session_per_request` (F1, plan3)
- [x] `aws_credential_hardcoded` (F3, plan3)
- [x] `config_file_read_per_request` (D3, plan3)
- [x] `dynamodb_scan_in_handler` (F5, plan3)
- [x] `ssh_host_key_callback_insecure` (E7, plan2)

### Wave 4 — Remaining Rules (62 rules)

Priority: lower confidence or niche patterns. Ship after waves 1-3 settle.

- [x] Wave 4 is complete.

- [x] All remaining plan1 rules (string operations A1-A3, A6-A8; slice operations B1-B2, B4, B7-B8, B10-B11, B13; runtime C2-C5, C8-C9; I/O D3-D5, D7; interface E6-E7)
- [x] All remaining plan2 rules (A4, A7-A8; B7-B8, B10; C3, C7-C8; D1, D3-D4, D7; E1-E2, E4-E5, E8; F7-F9)
- [x] All remaining plan3 rules (A5; B2, B4-B5; F2, F4)

## Parser And Evidence Work

- [x] Extend `import_aliases_for()` usage and explicit import-path checks to cover Redis, gRPC, JWT, Prometheus, Viper, Cobra, and AWS SDK call families.
- [x] Add composite-literal and nearby-line detection for fields such as `InsecureSkipVerify`, cookie flags, keepalive options, and TLS/version settings.
- [x] Reuse the existing request-handler detection helpers (`http.Handler`, Gin, Echo, Fiber) for handler-scoped findings.
- [x] Add name-based classifiers for auth/security-sensitive and trusted-feed functions where the heuristics need extra context.

## Fixtures, Integration Tests, And Benchmarks

- [x] Organize fixtures under `tests/fixtures/go/advanceplan4_perf_positive.txt` and `advanceplan4_perf_clean.txt` for performance rules.
- [x] Organize fixtures under `tests/fixtures/go/advanceplan4_security_positive.txt` and `advanceplan4_security_clean.txt` for security rules.
- [x] Organize fixtures under `tests/fixtures/go/advanceplan4_library_positive.txt` and `advanceplan4_library_clean.txt` for library rules.
- [x] Add grouped integration coverage in `tests/integration_scan/go_advanceplan4.rs`.
- [x] Re-verify the full slice with `cargo test go_advanceplan4 -- --nocapture`.

## False-Positive Controls

- [x] Skip test files via `is_test_file` / `is_test_function` suppression in the pack entrypoint.
- [x] Default to `Info` for micro-optimization rules, `Warning` for correctness/security risk, and `Error` for the direct exploit families.
- [x] Keep version-specific suggestions phrased conservatively without hard-gating on `go.mod` version parsing.
- [x] Gate handler-only findings on handler detection (Gin, stdlib `http.Handler`, Echo, Fiber).
- [x] Suppress library-specific findings when the relevant library import is not present in the file.

## Severity Distribution

| Severity | Performance (plan1) | Security (plan2) | Library (plan3) | Total |
|----------|-------------------|-----------------|----------------|-------|
| Error    | 0                 | 10              | 0              | 10    |
| Warning  | 15                | 35              | 12             | 62    |
| Info     | 35                | 7               | 18             | 60    |
| **Total**| **50**            | **52**          | **30**         | **132**|

## Acceptance Criteria

- [x] Every shipped rule includes a concrete "use this / instead of this" contrast with approximate cost numbers.
- [x] Security rules reference the specific vulnerability class (XSS, SSRF, injection, etc.).
- [x] Clean fixtures demonstrating correct patterns stay quiet.
- [x] The backlog reached zero for the 132-rule target set; nothing in this phase was deferred.
- [x] The shipped implementation stays function-local and lightweight; focused integration verification stays green.
- [x] All 132 rules are statically detectable without SSA, type checking, or cross-package analysis.

## Non-Goals

- [x] Do not claim precise benchmark numbers — all costs are approximate and documented as such.
- [x] Do not require Go version detection for the first wave — version-specific suggestions note the minimum version instead.
- [x] Do not attempt to replace `go vet`, `staticcheck`, or `golangci-lint` — the shipped rules stay focused on library misuse, micro-optimization, and generated-code antipatterns.
- [x] Do not couple implementation to runtime profiling data — all rules remain heuristic-based.
