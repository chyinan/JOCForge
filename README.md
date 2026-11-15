# JOCForge

**5.1 多码率 CLI：可用。** The selected 48 kHz / 5.1 six-carrier E-AC-3 JOC CLI path supports explicit rates of 384, 448, 576, 640, 768, and 1024 kbps. Local verification covers input/profile admission, frame sizing, keyless integrity, and a full-file output at each rate. These outputs are not equivalent to DEE; DEE parity and W04 acceptance are not established.

Example: encode the selected 5.1 profile at 1024 kbps.

```powershell
jocforge encode "D:\input.adm.wav" "D:\output.ec3" --data-rate 1024
```

**详细结构 Flat7.1：E-1105 是最新全曲实验候选，已完成 OpenJOC 检查与渲染。** The target follows the official demo structure:

- I0: 640 kbps legacy AC-3; D0: 1024 kbps E-AC-3 with `chanmap=0x1A00` (1,664 kbps aggregate target).
- Flat7x/JOC reconstruction index 1, 15 JOC objects, seven coefficient rows, and complexity 16. Coefficient rows are separate from ADM physical input rows.
- One complete EMDF skip field in D0 block 2; payloads are limited to 1..511 bytes and oversized fields reject.

The selected Flat7x CLI stages seven-carrier raw-QMF windows, a diagnostic C=7 fit, selector-1 ID14 writing, and I0+D0 AUs. E-1105 completed the full Love Song run with 8,417 AUs / 56,023,552 bytes and the experimental EMDF sequence `[11,14]`. OpenJOC 0.20.1 admitted 8,417/8,417 AUs under observed-vendor compatibility, reported zero malformed AUs and continuous timing, and rendered an 8-channel, 48 kHz IEEE float32 WAV with 12,928,544 frames. D0 block-2 placement receipts passed 8,417/8,417, maximum `C=461`, overflow=0.

ETSI strict reports `codecdatae=0` on ID11/ID14; the Wash-sanitized AU0 metadata for the designated official demo reports the same values. E-1104 remains the four-payload `[11,14,2,1]` baseline. Flat7x remains experimental with `production_allowed=false`; ADM-role mapping, ID14-row/OAMD-slot identity and user listening remain open. The selected 5.1 route remains available across all supported rates, and this branch does not claim DEE equivalence.

Wash-sanitized AU0 metadata for the user-designated MKA reports ordered EMDF payloads `[11,14]` across 23,033 AUs; the preserved E-1104 baseline has `[11,14,2,1]` across 8,417 AUs. The local ID2 body carries DRC-profile/LFE metadata and ID1 carries measurement/timer data; they are not empty placeholders. The Flat7x diagnostic variant serializes `[11,14]` while retaining all four F13 queue operations and typed pop provenance for ID1/ID2. E-1104 remains available for A/B comparison; the new variant is `production_allowed=false`. Wash's OpenJOC source audit reports that IDs 1/2 remain in generic EMDF envelope/order/config/length summaries, while only IDs 11/14 enter specialized OAMD/JOC parsing and the JOC frame input. Effects in other decoders remain unresolved.

The application `encode()` API still returns `SourcePipelineUnavailable`; the selected CLI path is usable for experimental candidates. OpenJOC parse and render are verified for E-1104 and E-1105. User listening, spatial identity, production exposure and W04 acceptance remain open. This output is not equivalent to DEE.

Target command for an isolated experimental candidate:

```powershell
jocforge encode "D:\input.adm.wav" "D:\JOCForge-attempts\attempt-id\output.candidate.ec3" --data-rate 1024 --joc-layout flat7x
```

The existing selected 5.1 path remains available.

Rust 2024 workspace: domain, application, io, CLI and independent validation. See [architecture](docs/architecture/ARCHITECTURE.md).

Run `rtk cargo fmt --all -- --check`, `rtk cargo check --workspace --locked`, `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` and `rtk cargo test --workspace --locked`. Run `rtk proxy python scripts/check-boundaries.py` for dependency checks.

`jocforge --help` and `--version` work. The CLI accepts a selected ADM BWF input/output pair, or `encode --job-xml <job.xml>` for the strict selected ADM BWF / elementary EC-3 subset. XML branches `adm` and `atmos_mezz` use the same BWF reader and remain distinct in normalized configuration identity. Add `--preflight-only` to stop after XML/input admission without starting Meter or creating output. MP4/DASH packaging and unclosed XML options reject explicitly; successful parsing does not establish their audio effects. The application `encode()` API still returns `SourcePipelineUnavailable`; this API limitation is separate from the selected 5.1 CLI path. The Flat7x profile remains experimental and not equivalent to DEE; E-1104 and E-1105 decoder admission and OpenJOC rendering are verified, while user listening, spatial identity and W04 acceptance remain open.

The XML parser recognizes and preserves all six Line/RF DRC profile tokens. Current encode admission accepts only `none/none` and `film_light/film_light`; other pairs reject, and the typed source bridge reports `UnsupportedDrcEffect` if reached. This describes the current admission boundary, not the profiles' acoustic effects or a full support matrix.
