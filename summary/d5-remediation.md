# D5 finding remediation

Verified 2026-09-03.

## F-D5-001

`Measure-FinalRenderEvidence` now measures `final_render_leakage` from the
actual PCM rendered by the pinned OpenJOC executable. The metric is unintended
rendered output-channel energy divided by intended rendered output-channel
energy, with threshold `1e-12`. Source-stage
`unexplained_cluster_leakage` is recorded separately and cannot satisfy the
final-render predicate. The closure mutates temporary rendered PCM, rejects
the mutation, and verifies that source-stage metadata cannot override a bad
final-render value.

## F-D5-002

The reproduced failure was shared by all three profiles at the 1024 total-rate
cell: the 384 kbps five-channel FFmpeg I0 used automatic channel coupling,
which pinned OpenJOC could not traverse (`invalid E-AC-3 grouped exponent
125`). The minimal correctness fix disables automatic coupling only for the
384 kbps I0 path with `-channel_coupling 0`.

The authoritative contract is profile × bitrate. Fresh evidence supports
`idx0`, `idx1`, and `idx2` at 1024, 1280, 1408, 1536, and 1664 kbps; 384, 448,
640, 768, and 896 kbps are rejected deterministically for insufficient paired
I0/D0 capacity. No supported cell has a mandatory render failure.

## Validator pin migration

The original `b4fb45…` executable was not recoverable from permitted locations
after the active OpenJOC development target was overwritten. OpenJOC source
revision `ad6556…` was checked out cleanly and built once with
`cargo build --release --locked`. The resulting OpenJOC 0.16.0 executable is
frozen read-only under `D:\Programs\JOCForge-validator` and is resolved only
through `validation/openjoc-validator.json` and
`scripts/openjoc-validator-pin.ps1`.

## Fresh closure

`artifacts/validation-closure/clean-validation-closure.json` records 12/12
clean tests, underasserted=0, 15 supported cells, 15 capacity rejections,
zero correctness failures, zero supported-with-render-failure cells, passing
negative mutations, and final-render leakage measurements from frozen OpenJOC
PCM. C7 does not adjudicate the final provenance gate.
