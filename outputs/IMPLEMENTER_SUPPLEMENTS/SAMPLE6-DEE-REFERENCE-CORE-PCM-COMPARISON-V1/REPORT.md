# Sample 6 bounded DEE reference vs JOCForge core PCM

## Experiment

The authorized ADM SHA-256 is
`7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1` in both
the bounded DEE reference receipt and the keyless JOCForge full-run receipt.
The DEE window receipt maps `file_position 2160..2208` to source sample
frames `[4,320,000,4,418,000)`, 98,000 samples per channel. The M4A contains
64 E-AC-3 AUs, 98,304 coded samples. Its MP4 edit list has `media_time=0`,
but the location/meaning of the extra 304 coded samples is not established.

FFmpeg 8.1 decoded both files with the same decode-only settings to
48 kHz/6-channel `pcm_f32le`, streamed through memory only. The primary
candidate compares the first 98,000 decoded sample frames of the M4A with
the JOCForge full-file decoded interval `[4,320,000,4,418,000)`. No PCM file
was written. This is an E-AC-3 5.1 core/bed comparison; it does not compare
Atmos object rendering or spatial playback.

## Candidate result

The candidate PCM hashes differ:

- DEE M4A first 98,000 frames: `6c11dac2b2edfa0b5f5fe653b5d5df7f780b4243ba66423d78b7dbefd5148dfa`
- JOCForge Rust candidate interval: `77f5fc12a515484ece90534104ac0cf17dd88ac80c31394ca889dd5e087884c9`

Across 588,000 interleaved F32 sample values, only 2 were exactly equal;
correlation was `0.987903`, RMS difference `0.00656319` (about 15.5% of the
Rust interval RMS). A lag search over ±1,536 samples found its best result at
M4A start `+304` and Rust start `+304` relative to the candidate source
window; it still produced different hashes, correlation `0.988386`, and
15.2% relative RMS difference. The `+304` search is sensitivity evidence,
not a proven trim or source-origin rule.

Per-channel least-squares gains for the primary candidate are not a single
common gain (`0.9594`, `0.9572`, `1.0489`, `0.9844`, `0.9883`, `0.9880` in
the six decoded channels). Per-channel gain normalization still leaves
residual RMS differences. This does not look like a single global-level
offset, but it does not identify the responsible processing stage.

FFmpeg's E-AC-3 decoder produced identical hashes with its default DRC scale
and `-drc_scale 0` for each file independently. That decoder option therefore
does not account for this measured difference; it says nothing about DEE's
encoding decisions.

## Interpretation and limits

This is a useful same-source, bounded core-audio comparison and indicates a
candidate mismatch, but exact alignment is not proven: the Rust receipt has
no mapping from ADM sample zero to decoded sample zero, and the reference and
full-run job configurations are not established as equivalent. The result
does not prove a full-song mismatch and does not cover Atmos objects/spatial
rendering. The native reference is a 2.048-second M4A from a separate
file-position job; do not treat its 64 coded AUs as the exact Rust AU range.

The next comparison needs a bound Rust source-sample-to-decoded-PCM origin
and matching job controls, then the exact 98,000-sample interval can be
rechecked. If the aligned hashes still differ, investigate gain/measurement,
selected RenderBus mode, spatial row transforms, and codec input as separate
audio-changing dependencies.

## Method and exclusions

The command used the installed FFmpeg 8.1 decoder, selected the first audio
stream, output 48 kHz/6-channel `pcm_f32le` to a pipe, and retained only
window digests and numeric comparison statistics in process memory. No PCM
or encoded payload was saved. FFmpeg was used only as a comparison decoder,
not as an encoder or algorithm source. No private key or license material
was accessed.
