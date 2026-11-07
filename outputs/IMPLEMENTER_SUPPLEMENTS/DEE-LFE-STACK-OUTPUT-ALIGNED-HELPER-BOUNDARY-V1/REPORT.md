# LFE stack-output aligned-helper boundary

## Bound static alignment path

The pointer entering the alignment check is the stack output buffer from FUN_14033c0e0 at RSP+0x40; the original input pointer remains in RDI. Using the Windows x64 call-entry alignment contract, entry RSP is 8 modulo 16. The inspected function pushes 0x38 bytes and subtracts 0x450, leaving current RSP 0 modulo 16. Therefore RSP+0x40 is 16-byte aligned, and the probe expression selects the equal branch to FUN_1403174c0. This is a static ABI-based derivation, not a runtime capture.

The previously sealed helper receipt says FUN_1403174c0 reads 256 F32 values and does not write its input. In this selected edge, no direct writeback to RDI or transfer to e350 is established. Runtime aliasing between the stack output and RDI remains UNBOUND; the alternate unaligned branch was not revisited.

## Limits

No parity, root-cause, or runtime branch claim is made. The upstream checker passed normally and under python -O. Raw instruction bytes and listing excerpts are excluded.
