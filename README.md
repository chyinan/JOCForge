# JOCForge

**5.1 多码率 CLI：可用。** The selected 48 kHz / 5.1 six-carrier E-AC-3 JOC CLI path supports explicit rates of 384, 448, 576, 640, 768, and 1024 kbps. Local verification covers input/profile admission, frame sizing, keyless integrity, and a full-file output at each rate. These outputs are not equivalent to DEE; DEE parity and W04 acceptance are not established.

Example: encode the selected 5.1 profile at 1024 kbps.

```powershell
jocforge encode "D:\input.adm.wav" "D:\output.ec3" --data-rate 1024
```

**详细结构 Flat7.1：开发中，当前保持 fail-closed。** The target follows the official demo structure:

- I0: 640 kbps legacy AC-3; D0: 1024 kbps E-AC-3 with `chanmap=0x1A00` (1,664 kbps aggregate target).
- Flat7x/JOC reconstruction index 1, 15 JOC objects, seven coefficient rows, and complexity 16. Coefficient rows are separate from ADM physical input rows.
- One complete EMDF skip field in D0 block 2; payloads are limited to 1..511 bytes and oversized fields reject.

The selected Flat7x CLI route currently stops at `Flat7xCoefficientSourceUnavailable`. OpenJOC defines the 15-object/seven-row payload shape. A borrowed PCM view maps ADM bed roles into JOC order `[L,R,C,Ls,Rs,Lrs,Rrs]`; the selected graph fixture resolves to physical rows `[0,1,2,4,5,6,7]`, while I0 and D0 keep their own coded-row maps. A separate producer now builds seven 48-slot raw-QMF windows with independent per-role histories. This remains a source-analysis candidate; the 15x7 coefficient fit and production provider are still unavailable. The role selection remains provisional, and the selected 5.1 example above remains usable; its outputs are not equivalent to DEE.

Target command (currently fails closed before frame assembly and writes no output file):

```powershell
jocforge encode "D:\input.adm.wav" "D:\JOCForge-attempts\attempt-id\output.candidate.ec3" --data-rate 1024 --joc-layout flat7x
```

The existing selected 5.1 path remains available.

Rust 2024 workspace: domain, application, io, CLI and independent validation. See [architecture](docs/architecture/ARCHITECTURE.md).

Run `rtk cargo fmt --all -- --check`, `rtk cargo check --workspace --locked`, `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` and `rtk cargo test --workspace --locked`. Run `rtk proxy python scripts/check-boundaries.py` for dependency checks.

`jocforge --help` and `--version` work. The CLI accepts a selected ADM BWF input/output pair, or `encode --job-xml <job.xml>` for the strict selected ADM BWF / elementary EC-3 subset. XML branches `adm` and `atmos_mezz` use the same BWF reader and remain distinct in normalized configuration identity. Add `--preflight-only` to stop after XML/input admission without starting Meter or creating output. MP4/DASH packaging and unclosed XML options reject explicitly; successful parsing does not establish their audio effects. The application `encode()` API still returns `SourcePipelineUnavailable`; this API limitation is separate from the selected 5.1 CLI path. The Flat7x profile remains experimental and not equivalent to DEE; parser/render and W04 acceptance remain incomplete.
