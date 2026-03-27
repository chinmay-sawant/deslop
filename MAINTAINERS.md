# Maintainers

## Ownership

- Primary maintainer: Chinmay Sawant

## Rust Security Triage

- CI and workflow ownership: maintain release and Rust security workflows.
- Heuristic ownership: review Rust rule additions under `src/heuristics/rust` and parser changes under `src/analysis/rust`.
- Runtime hardening ownership: triage findings from `scripts/check-rust-security.sh`, `cargo audit`, and `cargo geiger`.
- Documentation ownership: keep the Rust plan docs, `README.md`, and `SECURITY.md` aligned with implemented behavior.
- Ownership-cycle review: reject new `Rc<RefCell<_>>` parent or owner graphs unless a `Weak` edge breaks the cycle.
- Global-state review: reject `static mut` and `lazy_static!` additions unless the code documents its `Sync` invariants and there is no `once_cell` or atomic alternative.