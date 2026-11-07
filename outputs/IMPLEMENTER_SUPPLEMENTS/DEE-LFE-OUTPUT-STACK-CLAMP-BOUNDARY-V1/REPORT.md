# LFE post-filter output stack clamp boundary

## Bound stack-buffer use

The upstream receipt binds FUN_14033c0e0 stack argument 7 to the caller-local 256-F32 output buffer at RSP+0x40. After that call returns, FUN_140340640 first reads the stack buffer at 0x3408d0 and first writes it at 0x3408ee. A 32-iteration loop processes eight F32 values per iteration, covering the complete 256-F32 buffer, and applies scalar MAXSS/MINSS clamp operations in place.

This excerpt does not directly write the RDI original input pointer and does not transfer the stack output to e350. The runtime alias relation between the stack buffer and RDI remains UNBOUND. The subsequent transfer reaches the alignment-selected helpers, whose bodies are covered only by separate receipts.

## Limits

No numeric parity, root-cause, or end-to-end content-continuity claim is made. The upstream package is Ghidra 12.2 static only; its normal and python -O checks passed. No DEE run or media/PCM/payload/key/license read occurred. Raw Listing material is excluded.
