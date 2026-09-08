# OpenJOC, FFmpeg and MediaInfo interoperability census investigation

Verified 2026-09-08 against JOCForge HEAD `8919c8c75beb3a795539fd7ddb00ad4ce888b00a` and current OpenJOC source HEAD `a52c35d21e68e18dbaa8fa18accfbfea44087367`.

## Current boundary

- The authoritative matrix regenerated from the central resolver contains 140,400 rows: 2,393 `SUPPORTED_PASS`, 0 `SPEC_LEGAL_NOT_IMPLEMENTED`.
- The census uses the current OpenJOC release binary at `D:\Programs\OpenJOC\target\release\openjoc.exe`, not the frozen JOCForge validator.
- FFmpeg and ffprobe are the same 8.1-full_build-www.gyan.dev build. MediaInfo CLI is MediaInfoLib 21.03 from `D:\Programs\dee71joc\test_dee\MediaInfo.exe`.
- The runner selects 20 deterministic project-owned positive probes, including all five profiles, strict/deployed pairs, I0 skip-field, three normal short partitions, independent and dependent LFE, D1+, legacy AC-3, CMAF, an orthogonal short/deployed composition, and the current generator-compatible semantic maximum.

## Evidence model

- `scripts/run-interop-census.ps1` is an imperative shell. It regenerates the matrix and fixtures, runs bounded external processes, records stdout/stderr, preserves current OpenJOC JSON, checks rendered PCM finiteness, and writes normalized per-probe JSON plus CSV/JSON summaries.
- OpenJOC is scored by inspect, malformed count, render health, and CMAF container decode. FFmpeg is scored by base decode and additional-substream behavior. MediaInfo is scored only by recognition fields; it is never treated as a decoder.
- LAV/mpv are not separate FFmpeg-family votes. PotPlayer file information is not a separate MediaInfo-family vote.

## Observed result

- OpenJOC: 20/20 `FULL_PASS`, 0 malformed positive AUs.
- FFmpeg: 14/20 `FULL_PASS`, 6/20 `PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED`. The partial cases are D1+ or dependent-LFE streams with exit code 69 and sample-rate/decode diagnostics.
- MediaInfo: 1/20 `JOC_RECOGNIZED_CORRECTLY` (deployed I0 skip-field), 19/20 `EAC3_ONLY_FALSE_NEGATIVE`. This is exact-version detector behavior, not a legality verdict.

## Reproduction

Run `scripts/run-interop-census.ps1` for the default clean output, or pass `-OutputRoot artifacts\interop-census-rerun` for a second independent run. Compare `corpus-manifest.json`, `detailed-matrix.csv`, and `summary.json`.
