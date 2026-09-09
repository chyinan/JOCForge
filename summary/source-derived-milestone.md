# JOCForge source-derived milestone

Verified 2026-09-02.

The separately supplied Logic Pro ADM-BWF file is scanned through EOF even
though metadata follows the large `data` chunk. The chunk sequence is `JUNK`,
`fmt `, `data`, `axml`, `chna`, `dbmd`. The file has 38 tracks, 38 TrackUIDs,
29 authored objects, 28 `Objects`, and 1 `DirectSpeakers`; the source path,
filename, size, hash, and media are intentionally omitted from public docs.

JOCForge does not reduce that scene. Encoding is rejected before PCM loading:

`Unsupported ADM scene: 28 authored dynamic objects exceed current <=15 source-derived MVP profile; scene reduction is not implemented.`

The controlled fixture produces three ignored source-derived streams under
`artifacts/source-derived/`. Each has 125 access units, a 4096-byte FFmpeg I0,
and a 2560-byte JOCForge-owned dependent D0. Both D0 channels are driven by
source carrier PCM features from each 256-sample window; this is not a fixed
structural tone, but it is also not yet a transparent perceptual E-AC-3
encoder.

OpenJOC v0.16.0 strict inspect, render, and internal-base decode, plus generic
FFmpeg decode, passed for idx0, idx1, and idx2. Regression tests cover metadata
after `data`, a 128 MiB seek-over, odd padding, and clean EOF.
