# Flat7x D0 frame-size field

## Wash-approved scope

For the selected 48 kHz, six-block Flat7x target, D0 is 1,024 kbps and 4,096 bytes. The target contract therefore supplies `frmsiz = 4096 / 2 - 1 = 2047` (`0x7ff`) as the 11-bit frame-size code.

Ghidra 12.2 `READ_ONLY_NO_ANALYSIS` on the hash-bound user-designated DEE 6.7 `dee_ddpjoc_encoder.exe` found a generic field path: read `U16(state + 0x205e6)`, subtract one, and pass the resulting word to `FUN_140733fa0`; its direct writer `FUN_1406b9370` emits 11 bits by masking to `0x7ff`. The inspected writer has no range-rejection path.

This supports an implementation-local 11-bit D0 `frmsiz` writer and the target contract value. The generic state owner is not bound to the target D0 frame, so the static state field is not evidence that a native target run supplied 2047. Keep the caller value explicit and keep this supplement production-disabled.
