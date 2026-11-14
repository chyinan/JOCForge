# JOCForge

**5.1 多码率 CLI：可用。** The selected 48 kHz / 5.1 six-carrier E-AC-3 JOC CLI path supports explicit rates of 384, 448, 576, 640, 768, and 1024 kbps. Local verification covers input/profile admission, frame sizing, keyless integrity, and a full-file output at each rate. These outputs are not equivalent to DEE; DEE parity and W04 acceptance are not established.

Example: encode the selected 5.1 profile at 1024 kbps.

```powershell
jocforge encode "D:\input.adm.wav" "D:\output.ec3" --data-rate 1024
```

**详细结构 Flat7.1：开发中。** The target follows the official demo structure:

- I0: 640 kbps legacy AC-3; D0: 1024 kbps E-AC-3 with `chanmap=0x1A00` (1,664 kbps aggregate target).
- Flat7x/JOC reconstruction index 1, 15 JOC objects/reconstruction rows, and complexity 16. These reconstruction rows are separate from ADM physical input rows.
- One complete EMDF skip field in D0 block 2; payloads are limited to 1..511 bytes and oversized fields reject.

Flat7x currently writes only an experimental candidate in a new empty attempt directory; name the output `*.candidate.ec3`. The candidate keeps `production_allowed=false`, does not complete the programme DONE root, and is not equivalent to DEE. Independent parser/render and W04 acceptance remain open.

Example: encode the user-designated ADM BWF at the 1,664 kbps aggregate target (I0 640 + D0 1024 kbps).

```powershell
# Create a fresh, empty attempt directory for each run.
New-Item -ItemType Directory "D:\JOCForge-attempts\love-song-flat7x-try-01"
jocforge encode "D:\Love Song 方大同.wav" "D:\JOCForge-attempts\love-song-flat7x-try-01\love-song-1664.candidate.ec3" --data-rate 1024 --joc-layout flat7x
```

The existing selected 5.1 path remains available.

Rust 2024 workspace: domain, application, io, CLI and independent validation. See [architecture](docs/architecture/ARCHITECTURE.md).

Run `rtk cargo fmt --all -- --check`, `rtk cargo check --workspace --locked`, `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` and `rtk cargo test --workspace --locked`. Run `rtk proxy python scripts/check-boundaries.py` for dependency checks.

`jocforge --help` and `--version` work. The CLI accepts a selected ADM BWF input/output pair, or `encode --job-xml <job.xml>` for the strict selected ADM BWF / elementary EC-3 subset. XML branches `adm` and `atmos_mezz` use the same BWF reader and remain distinct in normalized configuration identity. Add `--preflight-only` to stop after XML/input admission without starting Meter or creating output. MP4/DASH packaging and unclosed XML options reject explicitly; successful parsing does not establish their audio effects. The application `encode()` API still returns `SourcePipelineUnavailable`; this API limitation is separate from the selected 5.1 CLI path. The Flat7.1 CLI candidate path is experimental, and independent parser/render/W04 acceptance remains incomplete.
