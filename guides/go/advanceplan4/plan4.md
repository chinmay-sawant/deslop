# Plan 4 — Implementation Roadmap, Priority Order, And Backlog Summary (Go)

Date: 2026-04-03

## Status

- [ ] Draft created on 2026-04-03.
- [ ] This file is the execution roadmap for `advanceplan4` and ties the scenario backlog to the current Rust parser and heuristics architecture.

## Backlog Summary

| Plan | Focus Area | Rules | Category Breakdown |
|------|-----------|-------|-------------------|
| `plan1.md` | Low-Level Performance | 50 | Strings (12), Slices/Maps (13), Runtime/Sync (10), I/O/Encoding (8), Errors/Interfaces (7) |
| `plan2.md` | Security Worst Practices | 52 | Crypto (10), Injection (10), Auth/Session (8), Concurrency Security (7), Network/TLS (8), Data Exposure (9) |
| `plan3.md` | Library-Specific | 30 | Redis (6), gRPC (5), Logging (5), Config/CLI (5), Prometheus (4), AWS SDK (5) |
| **Total** | | **132** | |

## Rule Count vs Target

- [ ] Target: 100+ new worst practices
- [ ] Planned: 132 unique rules across 3 plans
- [ ] Buffer: 32 extra rules beyond the 100 target, allowing flexibility to cut low-confidence rules during implementation

## Dependency Analysis

### No New Parser Evidence Required (can ship immediately)

These rules only need `body_lines()`, `import_aliases_for()`, `body_text` matching, and existing `ParsedFunction` fields:

- [ ] All 12 string rules (plan1, Section A)
- [ ] All 13 slice/map rules (plan1, Section B)
- [ ] All 10 runtime/sync rules (plan1, Section C)
- [ ] All 8 I/O/encoding rules (plan1, Section D)
- [ ] All 7 error/interface rules (plan1, Section E)
- [ ] 8 out of 10 crypto rules (plan2, Section A — A1 through A7, A10)
- [ ] All 10 injection rules (plan2, Section B)
- [ ] 6 out of 8 auth/session rules (plan2, Section C — C1 through C6)
- [ ] 4 out of 7 concurrency security rules (plan2, Section D — D1, D2, D5, D6)
- [ ] All 8 network/TLS rules (plan2, Section E)
- [ ] All 9 data exposure rules (plan2, Section F)
- [ ] All 5 logging rules (plan3, Section C)
- [ ] All 5 config/CLI rules (plan3, Section D)
- [ ] All 4 Prometheus rules (plan3, Section E)

**Total immediately shippable: ~109 rules**

### Requires Minor Import Resolution Extension

These rules need `import_aliases_for()` support for libraries not currently in the alias table:

- [ ] Redis rules (plan3, Section A) — need `github.com/go-redis/redis` / `github.com/redis/go-redis` alias resolution
- [ ] gRPC rules (plan3, Section B) — need `google.golang.org/grpc` alias resolution
- [ ] AWS SDK rules (plan3, Section F) — need `github.com/aws/aws-sdk-go` alias resolution
- [ ] JWT rules (plan2, A8, C5) — need `github.com/golang-jwt/jwt` alias resolution

**Total after minor alias extension: ~132 rules (all)**

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

- [ ] `sprintf_for_simple_int_to_string` (A4) — clear pattern, measurable speedup
- [ ] `sprintf_for_simple_string_format` (A5) — clear pattern, measurable speedup
- [ ] `string_format_for_error_wrap` (A9) — both performance and correctness
- [ ] `strings_hasprefix_then_trimprefix` (A10) — Go 1.20+ optimization
- [ ] `strings_hassuffix_then_trimsuffix` (A11) — Go 1.20+ optimization
- [ ] `string_builder_write_string_vs_plus` (A12) — builder misuse
- [ ] `map_lookup_double_access` (B5) — clear waste
- [ ] `slice_grow_without_cap_hint` (B6) — well-known antipattern
- [ ] `range_copy_large_struct` (B12) — measurable for structs > 64 bytes
- [ ] `sort_slice_vs_sort_sort` (B3) — Go 1.21+ optimization
- [ ] `clear_map_go121` (B9) — Go 1.21+ optimization
- [ ] `sync_mutex_for_atomic_counter` (C1) — lock-free optimization
- [ ] `defer_in_tight_loop` (C6) — defer cost accumulation
- [ ] `select_with_single_case` (C7) — unnecessary overhead
- [ ] `waitgroup_add_inside_loop` (C10) — atomic batching
- [ ] `ioutil_readall_still_used` (D1) — deprecated API
- [ ] `json_marshal_then_write` (D2) — streaming optimization
- [ ] `csv_reader_reuse_record` (D6) — allocation reduction
- [ ] `errors_new_for_static_sentinel` (E3) — alloc reduction
- [ ] `fmt_errorf_without_wrap_verb` (E4) — error chain preservation
- [ ] `error_string_comparison` (E5) — performance + correctness
- [ ] `type_assertion_without_comma_ok` (E1) — panic safety
- [ ] `type_switch_vs_repeated_assertions` (E2) — jump table optimization  
- [ ] `os_getenv_in_hot_path` (D2, plan3) — lock avoidance
- [ ] `viper_get_in_hot_path` (D1, plan3) — mutex avoidance

### Wave 2 — Critical Security Rules (25 rules)

Priority: highest-impact vulnerabilities, clear detection patterns.

- [ ] `insecure_random_for_security` (A1) — crypto vs math/rand
- [ ] `hardcoded_tls_skip_verify` (A2) — MITM vulnerability
- [ ] `hardcoded_tls_min_version_too_low` (A3) — protocol downgrade
- [ ] `constant_encryption_key` (A5) — key in binary
- [ ] `constant_iv_or_nonce` (A6) — nonce reuse
- [ ] `bcrypt_cost_too_low` (A9) — brute-force weakness
- [ ] `rsa_key_size_too_small` (A10) — key length
- [ ] `os_exec_command_with_user_input` (B1) — command injection
- [ ] `template_html_unescaped` (B2) — XSS
- [ ] `text_template_for_html` (B3) — XSS
- [ ] `filepath_join_with_user_path` (B4) — path traversal
- [ ] `url_redirect_without_validation` (B5) — open redirect
- [ ] `ssrf_via_user_controlled_url` (B6) — SSRF
- [ ] `xml_decoder_without_entity_limit` (B9) — XXE
- [ ] `cookie_without_secure_flag` (C1) — cookie theft
- [ ] `cookie_without_httponly` (C2) — XSS cookie theft
- [ ] `cors_allow_all_origins` (C4) — CORS misconfiguration
- [ ] `timing_attack_on_token_comparison` (C6) — timing side-channel
- [ ] `http_body_readall_without_limitreader` (D8, plan1) — DoS
- [ ] `sensitive_data_in_log` (F1) — data exposure
- [ ] `error_detail_leaked_to_client` (F2) — info disclosure
- [ ] `debug_endpoint_in_production` (F3) — pprof exposure
- [ ] `struct_field_exposed_in_json` (F4) — data leak
- [ ] `temp_file_predictable_name` (F5) — symlink attack
- [ ] `world_readable_file_permissions` (F6) — permission flaw

### Wave 3 — Library-Specific Rules (20 rules)

Priority: popular libraries, common mistakes, measurable impact.

- [ ] `redis_ping_per_request` (A1, plan3)
- [ ] `redis_get_set_without_pipeline` (A2, plan3)
- [ ] `redis_keys_command_in_handler` (A3, plan3)
- [ ] `redis_connection_per_request` (A4, plan3)
- [ ] `redis_no_ttl_on_cache_keys` (A6, plan3)
- [ ] `grpc_dial_per_request` (B1, plan3)
- [ ] `grpc_context_not_propagated` (B3, plan3)
- [ ] `grpc_without_tls_credentials` (E6, plan2)
- [ ] `log_level_check_after_format` (C1, plan3)
- [ ] `logger_created_per_request` (C2, plan3)
- [ ] `string_format_in_structured_logger` (C3, plan3)
- [ ] `log_printf_for_production` (C4, plan3)
- [ ] `error_logged_and_returned` (C5, plan3)
- [ ] `prometheus_counter_created_per_request` (E1, plan3)
- [ ] `prometheus_high_cardinality_labels` (E2, plan3)
- [ ] `aws_session_per_request` (F1, plan3)
- [ ] `aws_credential_hardcoded` (F3, plan3)
- [ ] `config_file_read_per_request` (D3, plan3)
- [ ] `dynamodb_scan_in_handler` (F5, plan3)
- [ ] `ssh_host_key_callback_insecure` (E7, plan2)

### Wave 4 — Remaining Rules (62 rules)

Priority: lower confidence or niche patterns. Ship after waves 1-3 settle.

- [ ] All remaining plan1 rules (string operations A1-A3, A6-A8; slice operations B1-B2, B4, B7-B8, B10-B11, B13; runtime C2-C5, C8-C9; I/O D3-D5, D7; interface E6-E7)
- [ ] All remaining plan2 rules (A4, A7-A8; B7-B8, B10; C3, C7-C8; D1, D3-D4, D7; E1-E2, E4-E5, E8; F7-F9)
- [ ] All remaining plan3 rules (A5; B2, B4-B5; F2, F4)

## Parser And Evidence Work

- [ ] Extend `import_aliases_for()` to resolve third-party library import paths for Redis, gRPC, Zap, Logrus, Viper, Cobra, Prometheus, and AWS SDK.
- [ ] Add composite-literal-field checker that can detect specific fields in struct literals (e.g., `InsecureSkipVerify: true`, `Secure: true`).
- [ ] Add a handler detection helper that recognizes common HTTP handler signatures beyond Gin (stdlib `http.Handler`, Echo, Chi, Fiber).
- [ ] Add a "function name suggests" classifier for security-sensitive function detection (names containing `Login`, `Auth`, `Token`, `Password`, `Key`, `Encrypt`).

## Fixtures, Integration Tests, And Benchmarks

- [ ] Organize fixtures under `tests/fixtures/go/advanceplan4_perf_positive.txt` and `advanceplan4_perf_clean.txt` for performance rules.
- [ ] Organize fixtures under `tests/fixtures/go/advanceplan4_security_positive.txt` and `advanceplan4_security_clean.txt` for security rules.
- [ ] Organize fixtures under `tests/fixtures/go/advanceplan4_library_positive.txt` and `advanceplan4_library_clean.txt` for library rules.
- [ ] Add grouped integration coverage in `tests/integration_scan/go_advanceplan4.rs`.
- [ ] Benchmark scan time with all 132 new rules enabled against at least one large real-world Go repository.

## False-Positive Controls

- [ ] Skip all test files and generated files.
- [ ] Default to `Info` for micro-optimization rules; `Warning` for correctness + performance rules; `Error` only for critical security vulnerabilities.
- [ ] Require Go version hints (from `go.mod` or parser context) before enabling Go 1.20+, 1.21+, or 1.22+ specific suggestions.
- [ ] Gate handler-only findings on handler detection (Gin `*gin.Context`, stdlib handler signatures, or common framework patterns).
- [ ] Suppress library-specific findings when the library import is not present in the file.

## Severity Distribution

| Severity | Performance (plan1) | Security (plan2) | Library (plan3) | Total |
|----------|-------------------|-----------------|----------------|-------|
| Error    | 0                 | 10              | 0              | 10    |
| Warning  | 15                | 35              | 12             | 62    |
| Info     | 35                | 7               | 18             | 60    |
| **Total**| **50**            | **52**          | **30**         | **132**|

## Acceptance Criteria

- [ ] Every shipped rule includes a concrete "use this / instead of this" contrast with approximate cost numbers.
- [ ] Security rules reference the specific vulnerability class (XSS, SSRF, injection, etc.).
- [ ] Clean fixtures demonstrating correct patterns stay quiet.
- [ ] The backlog reduces monotonically — rules are either shipped or explicitly deferred with rationale.
- [ ] Scan time impact stays within 15% of the pre-advanceplan4 baseline on representative repositories.
- [ ] All 132 rules are statically detectable without SSA, type checking, or cross-package analysis.

## Non-Goals

- [ ] Do not claim precise benchmark numbers — all costs are approximate and documented as such.
- [ ] Do not require Go version detection for the first wave — version-specific suggestions should note the minimum version.
- [ ] Do not attempt to replace `go vet`, `staticcheck`, or `golangci-lint` — focus on patterns those tools don't cover well (library misuse, micro-optimization, generated-code antipatterns).
- [ ] Do not couple implementation to runtime profiling data — all rules are heuristic-based.
