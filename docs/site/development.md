# Development and verification

The Rust workspace separates domain logic, application orchestration, file I/O, the CLI and independent validation. Use the CLI to encode. See the [architecture guide](https://github.com/chyinan/JOCForge/blob/main/docs/architecture/ARCHITECTURE.md).

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked
cargo test --workspace --locked
python scripts/check-boundaries.py
```

## Preview the documentation

```sh
python -m pip install -r requirements-docs.txt
python -m mkdocs serve
python -m mkdocs build --strict
```

The Pages workflow builds both languages from `main`.
