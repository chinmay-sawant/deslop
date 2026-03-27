# Security Policy

## Scope

This repository uses static checks, bounded I/O, and CI linting to reduce unsafe runtime behavior in the analyzer itself.

## Rust Safety Defaults

- Release builds enable `overflow-checks = true` in `Cargo.toml`.
- Production library code should avoid `.unwrap()`, `.expect()`, and `panic!()`.
- Repository scans use bounded reads through `read_to_string_limited` instead of unbounded `fs::read_to_string`.
- Rust async findings can be staged per repository with `.deslop.toml` using `rust_async_experimental = false`.

## Narrowing Casts

- Prefer `TryFrom` / `try_into()` over narrowing `as` casts in production code.
- The Rust security script reports new numeric narrowing casts for review.

## Secret Handling

- Avoid direct equality checks on secret-like values such as tokens, passwords, or API keys.
- Avoid deriving `Debug`, `Serialize`, or `Deserialize` on secret-bearing types without explicit review.
- Prefer redaction or wrapper types for secret-bearing fields.

## Reporting

- CI runs `cargo clippy`, `cargo test`, `scripts/check_rust_hygiene.sh`, and `scripts/check-rust-security.sh`.
- `cargo audit` and `cargo geiger` run in the Rust security workflow and remain report-oriented until stricter gating is adopted.

## Disclosure

Open a private security report through the repository hosting provider if a vulnerability affects released binaries or scan results.