# DEE raw-PCM DDP downmixer channel-count boundary

Recipient: implementation evidence supplement  
Disposition: conditional raw-PCM branch only; not an ADM-BWF admission result.

## Hash-bound static scope

Wash independently verified the upstream R1 manifest and both payloads. Its checker passed in normal Python and with `python -O`. Ghidra 12.2 traced the hash-bound `dee_ddp_encoder.exe` image (SHA-256 `0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532`) through the raw-PCM CLI configuration, PCM reader/resampler sizing, and DDP downmixer count gate.

The raw-PCM CLI `channels` field is parsed into a 32-bit configuration field; the checked parser has no numeric maximum. Reader and resampler allocation paths scale with channel count and have no fixed ceiling in the checked wrappers. The first explicit cap in this raw-PCM-to-DDP-downmixer branch rejects a frame count greater than 8; downstream dispatch accepts counts 1, 2, 6, and 8.

## Scope boundary

For the raw-PCM branch, a frame count of 36 or 38 is rejected if it reaches the `>8` downmixer gate. The package did not establish that an ADM-BWF 36/38-track input produces this raw-PCM frame, or that this branch is the selected `atmos_mezz` ADM route. Therefore the first native gate for the approved ADM profile remains `UNBOUND`; do not report 36-track ADM as rejected based on this raw-PCM branch.

This is a conditional symbolic boundary, not a runtime observation. No DEE execution or PCM/ADM payload/key read occurred. No native source or decompilation text is included.
