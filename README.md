# JOCForge

JOCForge is an independent, clean-room ADM-to-E-AC-3/JOC encoder proof. The
`0.1.0-dev` MVP prioritizes a reproducible standards-derived writer, finite
round-trip output, and external OpenJOC v0.15.0 validation over perceptual
encoder quality. The source-derived milestone proves the controlled
15-object path, including seven rank carriers and a PCM-derived dependent D0.

The MVP supports 48 kHz, integer PCM BW64/RF64/RIFF ADM input with a narrow
public subset: one programme/content relationship, up to 15 dynamic object
tracks, one object pack/channel per track, one position block per object, and
`chna` TrackUID bindings. It intentionally does not claim full ADM support.

## Build

```powershell
cargo check --workspace --locked
cargo test --workspace --locked
```

## Reproduce the controlled proof

The host must provide Rust, FFmpeg, and an independently built OpenJOC CLI.
Set `$OpenJoc` in `scripts/run-mvp.ps1` if the OpenJOC executable is elsewhere.

```powershell
.\scripts\run-mvp.ps1
```

The script generates a repository-owned four-second rank-7 BW64 fixture,
encodes all three mandatory profiles, runs OpenJOC strict inspect/render, and
writes local evidence under `artifacts/mvp/`.

The evidence includes `validation.json`, `metrics.json`, `frame-budget.csv`,
`object-binding.csv`, `profile-comparison.txt`, and the three rendered WAVs.

## Reproduce the source-derived milestone

The source-derived script first censuses the real ADM-BWF export at
`D:\ADM BWF\打上花火.wav`, verifies its 38-track/29-object metadata, and checks
that encoding is rejected explicitly because its 28 authored Objects exceed
the current `<=15` scene limit. It then generates the controlled fixture,
encodes all profiles, and writes evidence under `artifacts/source-derived/`.

```powershell
.\scripts\run-source-derived.ps1
```

Manual commands:

```powershell
cargo run -p jocforge -- fixture --seconds 4 artifacts/mvp/synthetic-reference.bw64
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

## Boundaries

This is not Dolby encoder parity, Dolby perceptual quality, production
mastering, native Dolby bitstream identity, or lossless ADM recovery. The
rank-controlled fixture proves an explainable linear pipeline; compatibility
presentation and arbitrary ADM factorization remain experimental.

OpenJOC is an external validator/decoder/renderer only. EBU `libadm`,
`libbw64`, and `libear` were audited from their upstream Apache-2.0 license
files. Full C++ interop is deferred because this Windows host has no exposed
CMake/C++/Boost build path; the narrow ingest boundary and migration plan are
recorded in `PROVENANCE.md`. This project does not claim full ADM support.
