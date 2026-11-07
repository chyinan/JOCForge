# LFE recurrence instruction-order evidence

## Machine-instruction facts

The hash-bound Ghidra 12.2 Listing for `FUN_14036d9e0` shows the inline recurrence using three distinct `MULPS` instructions followed by `ADDPS` for the grouped F32 operations. The nonzero `FUN_14036d7c0` path similarly uses separate multiplies and adds for both low and high banks. The six inspected functions (two recurrence routines and their four direct callers) contain no FMA instruction and no explicit MXCSR read/write or ROUND/CVT/RCP/RSQRT instruction.

These are function-scoped instruction observations. Runtime MXCSR state, upstream caller behavior beyond the direct callers, and any broader pipeline instruction sequence are not established. Separate `MULPS`/`ADDPS` are rounding-sensitive under the active MXCSR mode; that mode was not captured.

## Corrected LFE branch context

In the two previously reviewed core dispatchers, the LFE-lane selector is explicitly zero, so those lanes take the inline recurrence. Descriptor U16 stride is used for gather and is overwritten before the recurrence call. The nonzero table branch is a separate FBW-lane path in those dispatchers. This does not establish a selected layout/generation for any sample or explain the reported LFE residual.

Numeric parity is `NOT_RUN`, coefficient raw words were not included, and the ~0.819 correlation has no proven cause. Do not use this receipt alone to claim LFE parity or change the Rust algorithm.

## Provenance

Upstream evidence-manifest SHA-256: `64b337cedbea20e23e871541e9473a0f5509e41a2cf9027b6326f293967b8087`; upstream package-seal SHA-256: `2a1d93772ac38bb4c8096567fbb701b0d3dfb91c8c745ad96e8b9d9f1194b1cb`. The upstream verifier passed normally and with `python -O`; the Wash rehashed the sealed payload/source identities. Raw Listing bytes, Ghidra scripts, binaries, PCM, and encoded payload are not copied here.
