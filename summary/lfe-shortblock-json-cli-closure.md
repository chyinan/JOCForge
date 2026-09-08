# JOCForge LFE short-block and OpenJOC Inspector JSON closure

Verified 2026-09-08 from JOCForge start HEAD `f2eec69678d9993aedc93490a839e6075afdcd09`, JOCForge end work before commit, OpenJOC source HEAD `a52c35d21e68e18dbaa8fa18accfbfea44087367`, and frozen OpenJOC validator `0.16.0`.

## LFE short-block gap map

- `crates/jocforge-cli/src/main.rs:1722` rejected every non-default partition whenever ADM census found an LFE, before normal source-derived package work.
- `crates/jocforge-cli/src/main.rs:1750-1915` already produced a 1,536-sample LFE carrier and used the same programme interval/range ledger, but the second reduction/factorization pass did not pass LFE into the short writer.
- `crates/jocforge-core/src/writer.rs:471` serialized short I0/D0 frames with `lfeon=0`; the source-derived short package accepted five I0 carriers only, and appended no LFE audio-block syntax.
- `crates/jocforge-core/src/conformance.rs:688` independently rejected `independent.lfe_on`, so removing only the CLI guard would still leave an invalid core contract.
- `crates/jocforge-core/src/writer.rs:720` already contained the six-block source-dependent LFE exponent/mantissa pattern, which supplied the public syntax/CRC reference but did not solve short block prefix ordering or per-short-frame slicing.

## Closed contract

Normal strict raw I0+D0 short output now accepts an optional independent 1,536-sample LFE carrier. Each I0 short frame sets `lfeon=1`, uses the selected `numblkscod`, writes its corresponding LFE source feature in the same short audio-block interval, and keeps LFE outside the profile JOC matrix. Block prefixes and mantissas are interleaved per short syncframe; CRC2 is regenerated from the final frame bytes. D0 remains two full-band channels with no dependent LFE. Dependent-LFE, deployed, legacy-core, mixed partitions, and CMAF short paths remain closed.

The independent validator now reports `lfe_present`, requires consistent I0 LFE ownership across all short frames, and continues rejecting dependent LFE in the canonical short AU.

## Evidence

- Core tests cover all idx0..idx4 × `3+3`, `2+2+2`, and `1x6` at 1280 kbps, including contiguous 1,536-sample ranges and LFE causal serialization.
- `scripts/run-short-block-closure.ps1` produced 15/15 independent-LFE rows passing frozen OpenJOC inspect, full render with `LFE count: 1`, FFmpeg decode, accounting, and deterministic repeat.
- Real Hanabi `D:\ADM BWF\打上花火.wav` encode succeeded at idx2 / 1280 kbps / 2+2+2: 9,042 AUs, 54,252 frames, 46,295,040 bytes, frozen strict inspect PASS, zero malformed EMDF, full render PASS, and FFmpeg decode PASS.
- Current OpenJOC source-built `openjoc inspect FILE --json` reported direct fields: schema 1, JOC present, profile index 2 / `5.X+2`, carriers `L,R,C,Ls,Rs,Tfl,Tfr`, partition `[2,2,2]`, 1,536 samples/AU, `independent_owned:I0`, malformed 0, strict `pass`.
- Rendered Hanabi LFE PCM had 13,888,544 samples, zero NaNs/Infs, and nonzero RMS. A project-owned baseline/changed-only-LFE render produced different LFE PCM hashes, proving decoded/rendered LFE causality.

## OpenJOC Inspector JSON audit

The current OpenJOC source already implements JSON in `openjoc-inspect`, exposes it through `crates/openjoc-cli/src/stream_inspect.rs`, documents `openjoc inspect <FILE> --json`, and passes the targeted CLI/core JSON tests. The frozen validator artifact predates that source commit and its real help exposes no inspect JSON option. No OpenJOC code change was necessary for this task; strict/render authority remains the pinned frozen artifact, while the current source-built CLI supplied the JSON reachability evidence.
