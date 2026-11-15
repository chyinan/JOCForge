# JOCForge

**5.1 多码率 CLI：可用。** The selected 48 kHz / 5.1 six-carrier E-AC-3 JOC CLI path supports explicit rates of 384, 448, 576, 640, 768, and 1024 kbps. Local verification covers input/profile admission, frame sizing, keyless integrity, and a full-file output at each rate. These outputs are not equivalent to DEE; DEE parity and W04 acceptance are not established.

Example: encode the selected 5.1 profile at 1024 kbps.

```powershell
jocforge encode "D:\input.adm.wav" "D:\output.ec3" --data-rate 1024
```

**详细结构 Flat7.1：已有全曲实验候选并完成 OpenJOC 渲染。** The target follows the official demo structure:

- I0: 640 kbps legacy AC-3; D0: 1024 kbps E-AC-3 with `chanmap=0x1A00` (1,664 kbps aggregate target).
- Flat7x/JOC reconstruction index 1, 15 JOC objects, seven coefficient rows, and complexity 16. Coefficient rows are separate from ADM physical input rows.
- One complete EMDF skip field in D0 block 2; payloads are limited to 1..511 bytes and oversized fields reject.

The selected Flat7x CLI stages seven-carrier raw-QMF windows, a diagnostic C=7 fit, selector-1 ID14 writing, and an I0+D0 AU. A full Love Song run produced 8,417 AUs at 1,664 kbps aggregate; OpenJOC 0.20.1 parsed and admitted all AUs under observed-vendor compatibility and rendered an 8-channel, 48 kHz float WAV. The renderer remains experimental and semantic binding is unresolved. ADM-role mapping remains provisional, and the ID14-row/OAMD-slot identity is not yet bound. The selected 5.1 route remains available and its outputs are not equivalent to DEE.

The E-1100 run uses the official scalar ID11 wrapper flags while keeping the observed `codecdatae=0`; the selected 5.1 wrapper profile remains unchanged. Its D0 candidate carries a block-two placement receipt with the frame bit span, Codec source-generation receipt, and AU identity. The receipt log labels the separate F15 source-generation value explicitly.

E-1100 completed 8,417 AUs and rendered an 8-channel, 48 kHz float WAV through OpenJOC observed-vendor compatibility. ETSI strict reports only the shared ID11/ID14 `codecdatae=0` deviation. The renderer and ADM-role mapping remain experimental; this output is not equivalent to DEE.

Target command for an isolated experimental candidate:

```powershell
jocforge encode "D:\input.adm.wav" "D:\JOCForge-attempts\attempt-id\output.candidate.ec3" --data-rate 1024 --joc-layout flat7x
```

The existing selected 5.1 path remains available.

Rust 2024 workspace: domain, application, io, CLI and independent validation. See [architecture](docs/architecture/ARCHITECTURE.md).

Run `rtk cargo fmt --all -- --check`, `rtk cargo check --workspace --locked`, `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` and `rtk cargo test --workspace --locked`. Run `rtk proxy python scripts/check-boundaries.py` for dependency checks.

`jocforge --help` and `--version` work. The CLI accepts a selected ADM BWF input/output pair, or `encode --job-xml <job.xml>` for the strict selected ADM BWF / elementary EC-3 subset. XML branches `adm` and `atmos_mezz` use the same BWF reader and remain distinct in normalized configuration identity. Add `--preflight-only` to stop after XML/input admission without starting Meter or creating output. MP4/DASH packaging and unclosed XML options reject explicitly; successful parsing does not establish their audio effects. The application `encode()` API still returns `SourcePipelineUnavailable`; this API limitation is separate from the selected 5.1 CLI path. The Flat7x profile remains experimental and not equivalent to DEE; decoder, render, playback and W04 acceptance have not been verified.
