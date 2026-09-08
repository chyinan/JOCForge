# OpenJOC, FFmpeg and MediaInfo interoperability census

Verified against JOCForge `8919c8c75beb3a795539fd7ddb00ad4ce888b00a` and current OpenJOC source HEAD `a52c35d21e68e18dbaa8fa18accfbfea44087367`.

This is a compact, version-scoped census of legal JOC representatives. It is not an exhaustive run of the `140400`-cell capability lattice.

## Scope and implementation families

The public legality authority is the current JOCForge composition matrix. Only rows classified `SUPPORTED_PASS` entered the positive corpus. OpenJOC is measured as a JOC inspector/decoder/renderer, FFmpeg as a mainstream E-AC-3 stream/decode stack, and MediaInfo as a metadata recognizer. MediaInfo output is not used as decode evidence.

LAV, mpv and similar frontends may share FFmpeg decoding infrastructure and are not counted as independent decoder families. PotPlayer file-information output may reflect MediaInfo-family detection and is not counted as an independent metadata detector. No frontend smoke test contributes to the family counts.

## Tool versions

Machine-readable paths: `artifacts/interop-census/tool-versions.json` and `artifacts/interop-census/corpus-manifest.json`.

| Tool | Version | SHA-256 | Source / path |
|---|---|---|---|
| OpenJOC | OpenJOC 0.16.0 | `c740767304775648e18e3fdf8e490744e3c9521e27a8e2fca2aad9027b12977c` | current source HEAD `a52c35d21e68e18dbaa8fa18accfbfea44087367` |
| FFmpeg | ffmpeg version 8.1-full_build-www.gyan.dev Copyright (c) 2000-2026 the FFmpeg developers | `865b4a599b589982cbfa875c32eec92abbe0d6e78120ab3d28701b59e95d8265` | `C:\ProgramData\chocolatey\bin\ffmpeg.exe` |
| ffprobe | ffprobe version 8.1-full_build-www.gyan.dev Copyright (c) 2007-2026 the FFmpeg developers | `d1c3740b2c2a23e3b8990afdf08492a9879fe1845ccdbc43e5cb2404ac01b2f1` | `C:\ProgramData\chocolatey\bin\ffprobe.exe` |
| MediaInfo | MediaInfo Command line,  MediaInfoLib - v21.03 | `62bed8fd20d54e862c35f3a220022da8074538e5ead67895c4c5cbb03100a2a8` | `D:\Programs\dee71joc\test_dee\MediaInfo.exe` |

## Capability baseline

| Matrix verdict | Cells |
|---|---:|
| INVALID | 7800 |
| NOT_APPLICABLE | 110825 |
| PUBLIC_AMBIGUOUS | 2990 |
| REJECTED_CAPACITY | 10152 |
| RESERVED | 0 |
| SEMANTIC_CEILING | 6240 |
| SPEC_LEGAL_NOT_IMPLEMENTED | 0 |
| SUPPORTED_PASS | 2393 |

The positive corpus contains 20 probes. The script does not silently substitute another metadata detector if MediaInfo is unavailable; this run found MediaInfo available.

## Corpus results

| Probe | Key feature | OpenJOC | FFmpeg | MediaInfo |
|---|---|---|---|---|
| `p01-baseline-idx0-strict-6` | ordinary 5.X JOC baseline | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p02-baseline-idx0-deployed-6` | strict/deployed causal pair: deployed EMDF | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p03a-idx0-skipfld-strict-6` | strict/deployed causal pair: strict I0-only skip-field | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p03-idx0-skipfld-deployed-6` | I0-only skip-field deployed compatibility carriage | FULL_PASS | FULL_PASS | JOC_RECOGNIZED_CORRECTLY |
| `p04-idx1-flat-7x-strict-6` | true Flat-7.X profile | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p05-idx2-5x2-strict-6` | 5.X+2 profile | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p06-idx3-phase-strict-6` | 5.X plus phase profile | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p07-idx4-phase-5x2-strict-6` | 5.X+2 plus phase profile | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p08-idx2-short-3x3` | normal PCM-derived 3+3 short-block pair | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p09-idx2-short-2x2x2` | normal PCM-derived 2+2+2 short-block pair | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p10-idx2-short-1x6` | normal PCM-derived 1x6 short-block pair | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p11-idx0-short-3x3-independent-lfe` | independent LFE outside the JOC matrix | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p12-idx2-dependent-lfe-supplement` | dependent LFE supplementation | FULL_PASS | PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED | EAC3_ONLY_FALSE_NEGATIVE |
| `p13-idx2-dependent-lfe-replacement` | dependent LFE replacement | FULL_PASS | PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED | EAC3_ONLY_FALSE_NEGATIVE |
| `p14-idx1-d0-d1-strict-6` | two dependent substreams D0+D1 | FULL_PASS | PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED | EAC3_ONLY_FALSE_NEGATIVE |
| `p15-idx2-d0-d2-strict-6` | higher multi-dependent topology D0..D2 | FULL_PASS | PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED | EAC3_ONLY_FALSE_NEGATIVE |
| `p16-idx2-d0-d1-short-deployed` | orthogonal multi-feature short/deployed composition | FULL_PASS | PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED | EAC3_ONLY_FALSE_NEGATIVE |
| `p17-idx1-legacy-ac3-strict` | original-syntax AC-3 core plus E-AC-3 dependent | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p18-idx0-cmaf-ec3-dec3` | CMAF ec-3/dec3 six-block JOC | FULL_PASS | FULL_PASS | EAC3_ONLY_FALSE_NEGATIVE |
| `p19-idx4-d0-d5-short-monster` | semantic-maximum idx4 D0..D5 3+3 strict | FULL_PASS | PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED | EAC3_ONLY_FALSE_NEGATIVE |

The full per-probe records are in `artifacts/interop-census/probes/<probe>`. The normalized detailed matrix is `artifacts/interop-census/detailed-matrix.csv` and `.json`; the corpus manifest is `artifacts/interop-census/corpus-manifest.json`.

## Family-level summary

- OpenJOC: FULL_PASS: 20.
- FFmpeg: FULL_PASS: 14, PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED: 6.
- MediaInfo: EAC3_ONLY_FALSE_NEGATIVE: 19, JOC_RECOGNIZED_CORRECTLY: 1.

These are family-level observations for the exact binaries above. They do not establish universal support limits for other builds or products.

## Controlled boundaries

- `p01` and `p02` hold profile, raw ETSI carriage, six-block grouping, dependent count, LFE state and bitrate constant while changing strict versus deployed EMDF signaling. `p03a` and `p03` provide the same strict/deployed comparison for I0-only skip-field carriage. Their MediaInfo labels are detector observations, not legality judgments.
- `p08`, `p09` and `p10` hold profile and raw strict single-dependent composition constant while changing the normal short-block partition among `3+3`, `2+2+2` and `1x6`.
- `p14` and `p15` add dependent substreams while retaining source-derived raw ETSI syntax; any FFmpeg partial result is reported as additional-substream coverage, not malformed JOC.
- `p19` is the current generator-compatible semantic maximum: idx4, D0..D5, shared `3+3`, strict EMDF, and 4224 kbps. The exact supported row was resolved from the current matrix before generation.

## Limitations

The corpus is deterministic synthetic audio, with a small supplementary real-program path intentionally omitted from the public reproduction requirement. OpenJOC render health checks finite PCM and positive sample count; it does not claim perceptual parity. FFmpeg output is a decode/recognition observation and does not decide ETSI legality. MediaInfo is not a decoder. Negative controls were not included in this compact run.

## Reproduction

From the JOCForge repository root:

```powershell
.\scripts\run-interop-census.ps1
```

The runner cleans only its generated directory under `artifacts/`, regenerates the current capability matrix and project-owned fixtures, runs bounded OpenJOC/FFmpeg/ffprobe/MediaInfo commands, records raw stdout/stderr beside normalized JSON, and writes this document. Pass `-OutputRoot artifacts\interop-census-rerun` to produce an independent clean rerun for reproducibility comparison.

No production Rust decoder or encoder source was changed for this census.
