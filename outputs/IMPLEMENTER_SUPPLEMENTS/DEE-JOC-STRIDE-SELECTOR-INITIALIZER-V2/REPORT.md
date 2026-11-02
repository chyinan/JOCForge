# DEE JOC selector initializer boundary — row-specific defaults

## Corrected best-supported rule

The constructor `FUN_1800729a0` calls `FUN_1800a19e0` with destination `RBX+0x140e8` and second argument `*(RBX+0x298c8)+0xf5b8`. The callee wires pointer-table targets at offsets `+0x7698`, `+0x76d4`, `+0xb220`, and `+0xb274` and clears arrays reached through them. Its body has no `memcpy`/bulk-copy call and no selector-value store from the second argument to `param_1+0xd8+block*10`.

The six-short clear writes destination offsets `+0xd8,+0xda,+0xdc,+0xde,+0xe0,+0xe2`. With the consumer's 10-byte block-record stride, this establishes selector row 0 (`+0xd8`) and row 1 (`+0xe2`) as zero. It does **not** establish rows 2–5 as zero. The separate `0x5fee0`-byte arena clear is not evidence about the inline selector rows.

Use this row-specific rule: row 0 and row 1 default to selector 0; rows 2–5 retain explicit per-block inputs when available, otherwise stay `UNBOUND`. Do not default every missing selector row to zero, and do not derive values from bitrate/channel layout. Lookup remains `0→disabled, 1→stride 1, 2→stride 2, 3→stride 4`.

## Boundaries

The active 1024 kbps / 5.1 profile's nonzero selector values, rows 2–5 initialization, later writer, and update clock remain unobserved. This evidence refines a JOC payload configuration input; it does not explain the E-AC-3 core exponent parser error.

## Exclusions

Ghidra 12.2 static only. No DEE run, media/PCM/key access, source code, binary, or decompilation is included.
