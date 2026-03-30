# Plan 2 - HTTP Boundary And Handler Robustness (Go)

Date: 2026-03-30

## Objective

Add a Go-specific boundary rule pack for HTTP client and server code that often looks plausible in generated code but leaves timeout, cleanup, and response-handling gaps.

## Existing Coverage Explicitly Excluded

This plan must not duplicate the currently shipped rules below:

- `missing_context`
- `missing_context_propagation`
- `context_background_used`
- `full_dataset_load`
- `sql_string_concat`

This phase is about HTTP lifecycle correctness after the request is made, not about context propagation or generic full-payload loading.

## Candidate Rule Inventory

- [ ] `http_response_body_not_closed`
  - Detect `http.Get`, `client.Do`, or similar client-response patterns where `resp.Body.Close()` is not observed in the owning function.
- [ ] `http_client_without_timeout`
  - Detect local `http.Client{}` construction or long-lived client fields used without `Timeout` or transport-level timeout evidence.
- [ ] `http_server_without_timeouts`
  - Detect explicit `http.Server{}` values that omit `ReadTimeout`, `WriteTimeout`, and `IdleTimeout` style fields in production-looking code.
- [ ] `http_status_ignored_before_decode`
  - Detect response decode or body use without a visible status check when the call site looks like a request/response boundary.
- [ ] `http_writeheader_after_write`
  - Detect handlers that write to `http.ResponseWriter` before calling `WriteHeader(...)`, which makes the later status-setting call misleading.

## Why These Rules Belong In Advance Plan 2

- [ ] They are high-signal review findings in real Go services.
- [ ] They are common in AI-authored handlers and client wrappers because the code path works in happy-path smoke tests.
- [ ] The current Go detector set does not inspect HTTP cleanup or server timeout configuration directly.

## Parser And Evidence Work

- [ ] Extend Go function evidence to track:
  - [ ] local bindings for `resp`, `client`, `server`, and handler parameter names
  - [ ] `defer` and direct `Close()` calls on response bodies
  - [ ] `Write`, `WriteHeader`, and encoder calls that write to response writers
  - [ ] status checks such as `resp.StatusCode != http.StatusOK`
- [ ] Add lightweight struct-literal capture for `http.Client{}` and `http.Server{}` so timeout fields can be observed without full type checking.
- [ ] Preserve alias-import support because HTTP helpers are often imported as package aliases.

## Implementation Checklist

- [ ] Add parser unit coverage for response-body close detection and write-order capture.
- [ ] Implement HTTP boundary heuristics in a dedicated Go heuristic module or sub-module.
- [ ] Use conservative suppressions for:
  - [ ] helpers that return the response object to a caller
  - [ ] wrappers that delegate cleanup to a documented helper in the same function
  - [ ] test-only handlers and fixtures
- [ ] Keep `http_status_ignored_before_decode` at `Info` until fixture tuning demonstrates low noise.

## Fixture Plan

- [ ] Positive fixtures:
  - [ ] `http_response_close_positive.txt`
  - [ ] `http_client_timeout_positive.txt`
  - [ ] `http_server_timeout_positive.txt`
  - [ ] `http_status_check_positive.txt`
  - [ ] `http_writeheader_order_positive.txt`
- [ ] Negative fixtures:
  - [ ] response body closed via `defer`
  - [ ] clients configured with explicit timeouts or transports
  - [ ] servers with bounded timeout values
  - [ ] handlers that check status before decode and write headers before body output

## Acceptance Criteria

- [ ] Findings reference the concrete boundary object (`resp`, `client`, `server`, or response writer) and the missed lifecycle step.
- [ ] The rules stay local, parser-driven, and understandable from code review evidence alone.
- [ ] Normal helper-style wrappers and tests do not create noisy findings.

## Non-Goals

- [ ] Full HTTP semantics or middleware ordering across packages.
- [ ] TLS, auth, or proxy correctness proofs.
- [ ] Replacing integration tests or runtime load testing.