# Sample 6 AU-aligned DEE reference vs JOCForge core PCM

## Aligned 4-second experiment

R1 used the same authorized 38-channel ADM SHA-256
`7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1` for a
bounded DEE run at `file_position` 2112..2207, inclusive. The native log maps
this to source samples `[4,224,000,4,416,000)`, exactly 192,000 samples per
channel. The DEE M4A has 125 E-AC-3 AUs, 4.000 seconds, `media_time=0`, no
reported skip-sample side data, and decodes to 192,000 sample frames. The
temporary M4A was removed after its hashes and metadata were captured.

The matching Rust full-run interval is AU `[2750,2875)`: 125 AUs and the same
192,000 samples. Wash independently decoded the retained Rust full EC-3
file with FFmpeg 8.1 to `pcm_f32le` and recomputed its window hash
`6d9a5de71c530e65d87aa7ef203b3900ff50beec336018c7b269ed061747f2f7`, matching
the implementation-provided digest. R1's sealed source package records the
DEE window PCM hash `fcd37d2218c98229a48cff1e25644315ecf62267acdd27da82d6d8524daf8025`.
The DEE M4A is no longer available for Wash to re-decode, so this half of the
comparison is source-package-attested rather than independently recomputed.

## Result

The two same-input, same-range candidate core PCM hashes differ. R1 reports
RMSE `0.00639425`, maximum absolute difference `0.3048092`, and only 17
exactly equal scalar samples out of 1,152,000. The exact decoded sample count
and AU boundaries now match, so the earlier 90-second provisional alignment
uncertainty is removed for this 4-second run.

This is a concrete mismatch in the decoded 5.1 E-AC-3 core/bed candidate.
The job-configuration equivalence and cause remain open; no specific audio
algorithm is yet blamed. The comparison does not cover Atmos object
rendering or spatial playback and does not establish a full-song verdict.

## Method and limits

Both streams were decoded at 48 kHz, six-channel `pcm_f32le` using FFmpeg
8.1. Wash retained only the Rust window's hash; PCM was not saved. The DEE
PCM digest and error statistics are taken from the hash-bound R1 package.
No media, PCM, encoded payload, key, or license material is included here.
FFmpeg was used only as a comparison decoder, not as an encoder or algorithm
source.
