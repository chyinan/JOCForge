# F06 BSI owner to final AU boundary

## What is now joined

For one hash-bound full DEE run on the authorized ADM, a same-generation path follows the typed BSI owner into its serializer, the serializer result into the composition routine, and the resulting 4096-byte frame buffer into the task's FileWriter. At frame index 3481 the file grew by exactly one 4096-byte frame. A bounded seven-byte header probe at that offset identified an independent E-AC-3 syncframe with U5 DialNorm 17. No frame bytes were retained.

Separately, Ghidra traces the active BSI owner allocation and its explicit zero initialization at `+0x34`. A short measurement run observed U16 zero in one conditional branch and the same owner pointer at that run's serializer, but its first FileWriter call was zero length. It is not the same-generation nonempty-frame observation above. The full run's U16 was not reread.

## What remains unbound

The owner U16 `+0x34` is treated by the custom serializer as a raw custom-BSI code; this evidence does not connect it numerically to the standard frame-header U5 DialNorm. The relationship between the opaque source-vector qwords and the BSI U16 is also not established. Alternate writers across all modules are not exhaustively ruled out.

An earlier short-run diagnostic exceeded the later seven-byte prefix cap by reading 16 bytes; a subsequent local hash operation streamed the complete 262144-byte temporary output. Those bytes were not retained, but the scope deviation is preserved here. The successful full-run AU observation itself read only seven header bytes and no payload.

## Disposition

This is a reference-only boundary supplement, `implementation_ready=false`. It strengthens the path from one BSI owner generation to one actual EC3 file frame, but supplies no owner-U16-to-header-DialNorm formula, clamp, or rounding rule. Keep that edge `UNBOUND/NOT_PROVEN`; do not merge the separately observed U16 zero with the full-run U5 17.

DEE processed the authorized ADM internally. No PCM was manually read or decoded. No key/license/HMAC content was read. No algorithm rule is newly authorized by this package.
