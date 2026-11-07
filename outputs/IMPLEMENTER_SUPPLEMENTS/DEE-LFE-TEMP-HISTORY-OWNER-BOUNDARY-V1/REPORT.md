# LFE temporary to history-owner boundary

## Bound argument roles

In the inspected calls to `FUN_14036e350`, `RCX` is the temporary input, `RDX` is an in-place per-row state argument bounded by the parent receipt to 128 F32 / 0x200 bytes, and `R8` is the 256-F32 / 0x400-byte output buffer. The two dispatcher callsites pass distinct expressions in RCX and RDX; pointer aliasing is not established by these callsites.

The first direct caller of both dispatchers is `FUN_140317600`, which passes `RDI` as the dispatcher context argument. The static Listing does not identify the `RDI` owner/type or frame generation.

## First break

The evidence does not prove that the `FUN_14036e350` in-place state is owned by the same state generation and reused for the next frame. The history-owner and cross-frame feedback relationship remain `UNBOUND` / `NOT_PROVEN`; runtime remains `NOT_RUN`. This package does not revisit recurrence selector, coefficients, or FMA/MXCSR.

## Provenance

Source package `dee_lfe_temp_history_owner_boundary_wash_v1` manifest SHA-256 `ac35e0147171e1facd546baec9b928b8bc0dc855ee75766ff82d0e40f03b7425`, package seal SHA-256 `019e12e569adcc1fc5652870e991d62a89953c6273fa1da6f31b5553018190e7`; normal and `python -O` checks passed. Ghidra 12.2 Listing instruction encodings for the 28 inspected instructions matched the pinned EXE. No DEE run or PCM/payload/key/license read occurred.
