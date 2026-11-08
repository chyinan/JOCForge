# 7.1 per-substream rate and frame-size boundary

## Scope

This supplement records a static conditional mapping from selected rate tokens to typed setup candidates and a separate per-record frame-writer boundary. It does not establish that an I0 or D0 job value reaches a particular record or that a native header field is written.

## Static rate mapping

The reused Ghidra 12.2 receipt maps rate token 384 to selector code 0 and typed setup U32 value 6. It maps token 1024 to selector code 5 and typed setup U32 value 8 only if the lookup is reached. The candidate setup value at piVar17+0x206 is not joined to the writer-state owner at param_1+0x810.

## Per-record writer boundary

The per-record path selects one of two encoders and aggregates 16-bit word counts into an output byte count. A separate writer candidate serializes two 16-bit header input words. The producer of the second word's 11-bit frmsiz remains UNBOUND, as do the selected dynamic writer target and BSI bitrate field.

## Target arithmetic

For 48 kHz and six blocks, each frame covers 1,536 samples or 32 ms. The user-approved target is I0=640 kbps / 2,560 bytes and D0=1,024 kbps / 4,096 bytes: 6,656 bytes and 1,664 kbps per AU. This is an aggregate of two frames, not one 1,664 kbps E-AC-3 frame. The mode21 structural summary is used only for this rate allocation; no mode21 payload or failure hash is included.

## Handoff

IMPLEMENTATION_READY=NO for audio production. The existing 5.1 single-I0 rate row remains unchanged. Keep the host-rate-to-writer/BSI join, D0 audio algorithm, and actual output rate UNBOUND/NOT_RUN.
