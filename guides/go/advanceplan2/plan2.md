# Plan 2 - HTTP Boundary And Handler Robustness (Go)

Date: 2026-03-30

## Status

- [x] Implemented on 2026-03-30.
- [x] Verified by the Go advanceplan2 integration suite.

## Objective

Add a Go-specific boundary rule pack for HTTP client and server code that often looks plausible in generated code but leaves timeout, cleanup, and response-handling gaps.

## Shipped Rules

- [x] `http_response_body_not_closed`
- [x] `http_client_without_timeout`
- [x] `http_server_without_timeouts`
- [x] `http_status_ignored_before_decode`
- [x] `http_writeheader_after_write`

## Implementation Notes

- [x] Kept this scope separate from `missing_context`, `missing_context_propagation`, `context_background_used`, and `full_dataset_load`.
- [x] Preserved alias-import support for `net/http`.
- [x] Added conservative local response-binding, literal-block, and write-order detection rather than pretending to perform full HTTP semantic analysis.
- [x] Kept `http_status_ignored_before_decode` at `Info` and the lifecycle/timeout findings at `Warning`.
- [x] Suppressed obvious ownership-transfer cases where the response binding is returned.

## Fixtures And Tests

- [x] Added positive fixtures:
  - [x] `http_response_close_positive.txt`
  - [x] `http_client_timeout_positive.txt`
  - [x] `http_server_timeout_positive.txt`
  - [x] `http_status_check_positive.txt`
  - [x] `http_writeheader_order_positive.txt`
- [x] Added clean coverage in `http_boundary_clean.txt`.
- [x] Added grouped integration coverage in `tests/integration_scan/go_advanceplan2.rs`.

## Acceptance

- [x] Findings reference the concrete boundary object or response writer.
- [x] Representative clean handlers and client/server literals stay quiet.
- [x] The rules remain local and explainable.

## Non-Goals

- [x] Full HTTP semantics or middleware ordering across packages.
- [x] TLS, auth, or proxy correctness proofs.
- [x] Replacing integration tests or runtime load testing.