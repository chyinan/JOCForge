# ADM row count through the first Process-row consumer

## One typed count path

The hash-bound Ghidra receipt identifies descriptor `D` with a decoded row-vector pointer at `D+0x00`, a widened U16 count copy at `D+0x08`, sample count at `D+0x10`, and the exact composite descriptor pointer `B` at `D+0x28`. The same `D` owns both the copied value and the `B` pointer; the first row copier follows `D+0x28` to read `U16(B+0x0A)` directly. It does not infer identity by comparing equal counts.

`FUN_180179ab0` validates the frame sample count and prepares the row-major staging matrix. `FUN_180178cc0` is the first decoded-row-vector consumer, with call shape `FUN_180178cc0(*D, row_staging_buffer, 0x600, U16(*(D+0x28)+0x0A))`. It copies 1536 sample positions across N row pointers. N>3 selects a four-wide copy path followed by a scalar tail; this is a vectorization branch, not an N ceiling. At open time, `FUN_1801788a0` allocates N float rows of 1536 samples and an N-entry row-pointer table from the same count.

The first process callback gate is `D+0x10`: 1536 samples or flush count 0. It constrains frame sample count, not physical row count. The previously sealed open-time N gate is 1..128 inclusive; this package reuses that receipt and does not rescan it.

## 36/38 input-only comparison

Existing metadata receipts report N=36 (10 beds + 26 objects) and N=38 (10 beds + 28 objects). Both are above the `N>3` vectorization threshold and follow the same static wrapper/copy/tail path. The current evidence supports a generic count-driven row staging path in this inspected slice and finds no 38-specific branch there. It does not prove runtime execution for either N, arbitrary unbounded N, or the opaque PE/JOC internals.

The native D→Process owner/generation and selected composite mode remain `UNBOUND`/`NOT_CAPTURED`; the 36/38 values are input-only metadata projections. No PCM, payload, or key was read and DEE was not run.

## Provenance

Upstream Wash-only manifest SHA-256: `b9d327c0726e9bc2dee694e3202720f648dbceefa274bdc49bc505088c3586d3`. Its package verifier passed normally and under `python -O`; the verifier also reports the four parent package verifiers passed in both modes. The upstream manifest and sanitized function hashes are carried in this supplement. No Ghidra logs, native source, or decompilation are included.
