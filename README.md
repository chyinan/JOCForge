# JOCForge

JOCForge is an independent, clean-room ADM-to-E-AC-3/JOC encoder proof. The
`0.1.0-dev` milestone prioritizes a reproducible standards-derived writer,
finite round-trip output, deterministic N-to-15 scene reduction, and the pinned
external OpenJOC 0.16.0 validator over perceptual encoder quality.

The encoder supports 48 kHz, integer PCM BW64/RF64/RIFF ADM input with a narrow
public subset: one programme/content relationship, one object pack/channel per
track, one position block per object, and `chna` TrackUID bindings. Authored
`Objects` and positioned `DirectSpeakers` are explicitly accounted for and
reduced to at most 15 coded representations. The parser accepts only full-track
static timing and a single position block; extra blocks, unsupported
trajectories, and ambiguous programme membership are rejected before PCM
processing. Other unsupported kinds are rejected; complete non-LFE bed
semantics are not claimed.

## Build

```powershell
cargo check --workspace --locked
cargo test --workspace --locked
```

## Reproduce the controlled proof

The host must provide Rust and FFmpeg. The validation scripts resolve the
immutable OpenJOC artifact declared by `validation/openjoc-validator.json`.

```powershell
.\scripts\run-mvp.ps1
```

The script generates a repository-owned four-second rank-7 BW64 fixture,
encodes all three mandatory profiles, runs OpenJOC strict inspect/render, and
writes local evidence under `artifacts/mvp/`.

The evidence includes `validation.json`, `metrics.json`, `frame-budget.csv`,
`object-binding.csv`, `profile-comparison.txt`, and the three rendered WAVs.

## Reproduce the generalized clean milestone

The source-derived script first censuses the real ADM-BWF export at
`D:\ADM BWF\打上花火.wav`, verifies its 38-track/29-object metadata, then
reduces and encodes the 28-object programme to at most 15 coded objects. The
generalized outputs and per-frame/source accounting are written under
`artifacts/decision-engine/`.

```powershell
.\scripts\run-source-derived.ps1
```

Manual commands:

```powershell
cargo run -p jocforge -- fixture --seconds 4 artifacts/mvp/synthetic-reference.bw64
cargo run -p jocforge -- fixture --seconds 1 --objects 28 artifacts/decision-engine/fixture-28.bw64
cargo run -p jocforge -- inspect-adm artifacts/mvp/synthetic-reference.bw64
cargo run -p jocforge -- encode artifacts/mvp/synthetic-reference.bw64 artifacts/mvp/jocforge-mvp-idx0.ec3 --profile idx0
cargo run -p jocforge -- encode artifacts/mvp/synthetic-reference.bw64 artifacts/mvp/jocforge-mvp-flat7x-idx1.ec3 --profile idx1
cargo run -p jocforge -- encode artifacts/mvp/synthetic-reference.bw64 artifacts/mvp/jocforge-mvp-5x2-idx2.ec3 --profile idx2
```

`idx1` is Flat-7.X with `L,R,C,Ls,Rs,Lrs,Rrs` and `chanmap=0x0200`.
`idx2` is 5.X+2 with `L,R,C,Ls,Rs,Tfl,Tfr` and `chanmap=0x0010`. They are
different JOC payload headers and different dependent channel maps; idx2 is
not a relabeled idx1.

FFmpeg is used only as the external 5-channel E-AC-3 I0 carrier encoder. The
JOCForge-owned writer creates the PCM-derived two-channel dependent D0,
full-matrix JOC payload, OAMD, EMDF container, and frame carriage. Each D0
channel is driven by a bounded feature of its source carrier's 256-sample PCM
window; it is not a fixed structural tone, but it is not yet a perceptual or
transparent E-AC-3 encoder. D0 is 2560 bytes, so 4096-byte I0 + 2560-byte D0
equals 1664 kbps at 48 kHz/six blocks.

The clean bitrate contract is profile-specific (`PROFILE x BITRATE`), not a
single global bitrate flag. The current controlled contract tests
`idx0 / 5.X`, `idx1 / Flat-7.X`, and `idx2 / 5.X+2` at 1024, 1280, 1408,
1536, and 1664 kbps. At 1024 kbps the 384 kbps I0 carrier disables FFmpeg's
automatic channel coupling because the pinned OpenJOC decoder cannot traverse
that low-rate coupled carrier; this preserves the required 1024 target and
does not change the five-carrier topology. Rates without a legal paired
I0/D0 budget are rejected deterministically.

## Boundaries

This is not Dolby encoder parity, Dolby perceptual quality, production
mastering, native Dolby bitstream identity, or lossless ADM recovery. The
generalized reducer is an independent signal/spatial seed-and-aggregate
algorithm. Clean validation also records source/matrix residuals, coefficient
jumps, and a normalized speaker-projection compatibility proxy; these are
structural checks, not decoded proprietary parity or perceptual quality.
Frequency-local optimization and complete non-LFE bed mapping remain open
research. `PRODUCTION_PERCEPTUAL_QUALITY` is not claimed.

OpenJOC is an external validator/decoder/renderer only. Its identity is pinned
in `validation/openjoc-validator.json` and checked by every script that invokes
it. EBU `libadm`,
`libbw64`, and `libear` were audited from their upstream Apache-2.0 license
files. Full C++ interop is deferred because this Windows host has no exposed
CMake/C++/Boost build path; the narrow ingest boundary and migration plan are
recorded in `PROVENANCE.md`. This project does not claim full ADM support.
