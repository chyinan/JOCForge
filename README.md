# JOCForge

Backend migration baseline. Selected independent audio and metadata processing components are implemented and undergoing isolated validation. End-to-end E-AC-3 JOC encoding remains unavailable in this experimental branch.

Rust 2024 workspace: domain, application, io, CLI and independent validation. See [architecture](docs/architecture/ARCHITECTURE.md).

Run `rtk cargo fmt --all -- --check`, `rtk cargo check --workspace --locked`, `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` and `rtk cargo test --workspace --locked`. Run `rtk proxy python scripts/check-boundaries.py` for dependency checks.

`jocforge --help` and `--version` work. `encode` returns exit code 2 before reading input or touching output. Algorithm implementation requires architecture approval and an independently approved specification package.
