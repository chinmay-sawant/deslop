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
- Prefer constant-time comparison helpers such as `subtle::ConstantTimeEq` when secret material must be compared.
- Avoid deriving `Debug`, `Serialize`, or `Deserialize` on secret-bearing types without explicit review.
- Prefer redaction or wrapper types for secret-bearing fields.

## Filesystem Safety

- Repository scan roots are canonicalized before discovery so symlinked files cannot escape the requested root.
- Bounded file reads reject symlink targets and use `O_NOFOLLOW` on Unix when opening files.
- New path-accepting code should canonicalize resolved paths and assert they stay under the intended root before use.

## Shared State

- Prefer `once_cell::sync::Lazy`, atomics, or explicit locks over `static mut` and `lazy_static!`.
- Prefer `Weak` parent edges over `Rc<RefCell<_>>` ownership cycles.

## Reporting

- CI runs `cargo clippy`, `cargo test`, `scripts/check_rust_hygiene.sh`, and `scripts/check-rust-security.sh`.
- CI compares the generated Rust security baseline against `reports/rust-security-baseline/latest.txt` so new findings require an explicit baseline update.
- `cargo audit` runs as a blocking Rust security check, while `cargo geiger` remains report-oriented until stricter unsafe-count gating is adopted.

## Disclosure

Open a private security report through the repository hosting provider if a vulnerability affects released binaries or scan results.