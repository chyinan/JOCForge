# DEE JOC selector initializer boundary

## Static result

In the hash-bound Atmos DDP JOC image, `FUN_1800729a0` calls `FUN_1800a19e0` with destination `RBX+0x140e8`. The initializer wires internal pointers and clears six consecutive 16-bit fields at destination offsets `+0xd8,+0xda,+0xdc,+0xde,+0xe0,+0xe2`. The selector consumer's per-block record stride is 10 bytes, so this directly initializes selector rows 0 and 1 (`+0xd8` and `+0xe2`) to zero. The initializer does not copy selector values from its second argument.

The selector table lookup remains `[0,1,2,4]` (0 disables; 1/2/3 select strides 1/2/4). The constructor's separate `0x5fee0`-byte arena clear does not prove that the inline selector rows 2–5 are zero. A full direct-scalar store scan found no other selector-row writer in its checked scope, but bulk-copy initialization and later update timing remain unbound.

## Implementable rule

Treat rows 0 and 1 as zero-initialized by this constructor. For later rows, preserve explicit per-block selector inputs; their initial values and producer remain `UNBOUND`. Do not set rows 2–5 to zero based on the arena clear, and do not derive selector values from bitrate or channel layout. This narrows the earlier V2 rule and is ready for a focused trial.

## Exclusions

This is a JOC payload selector boundary, not the E-AC-3 core exponent syntax. No DEE run, media/PCM/key access, source code, binary, or decompilation is included.
