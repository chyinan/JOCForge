# Development and verification

The Rust 2024 workspace is split into domain logic, application orchestration, file I/O, the CLI and independent validation. Encode with the CLI. The application `encode()` entry still returns `SourcePipelineUnavailable`. `jocforge --help` lists the flags, and `jocforge --version` prints the version. See the [architecture guide](https://github.com/chyinan/JOCForge/blob/main/docs/architecture/ARCHITECTURE.md).

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked
cargo test --workspace --locked
python scripts/check-boundaries.py
```

## Measured export

One Windows x64 Release build was timed on a Ryzen 7 9800X3D. The input was a 191.69-second, 26-channel, 48 kHz ADM with moving objects.

| Export | Time |
|---|---|
| First 5.1 / 1024 kbps export | 41.4 s, including 16.2 s of loudness metering |
| Previous build, same first export | 52.0 s, including 20.0 s of metering |
| 5.1 / 1024 kbps with a loudness cache hit | 25.2 s |
| Flat 7.1 / 1664 kbps with a loudness cache hit | 37.8 s |

Both full-length EC-3 files matched the previous build bit for bit. The first measurement's loudness, true peak and gating result matched as well.

All six 5.1 rates, from 384 kbps to 1024 kbps, have completed a local full-file encode. Those encodes passed the input-configuration, frame-length and keyless integrity checks.

## Preview the documentation

```sh
python -m pip install -r requirements-docs.txt
python -m mkdocs serve
python -m mkdocs build --strict
```

The Pages workflow builds both languages from `main`.
