# DEE LFE terminal seven-F32 projection rules

## Exact terminal transforms

For an input window `x[0..15]` of F32 values and a seven-element F32 output `y[0..6]`, the static main projection is:

```text
y[0] = A[0]   * x[1]  - B[0]   * x[0]
y[1] = A[127] * x[14] + B[127] * x[15]
y[2] = A[1]   * x[3]  - B[1]   * x[2]
y[3] = A[126] * x[12] + B[126] * x[13]
y[4] = A[2]   * x[5]  - B[2]   * x[4]
y[5] = A[125] * x[10] + B[125] * x[11]
y[6] = A[3]   * x[7]  - B[3]   * x[6]
```

The companion projection is:

```text
y[0] = B[0]   * x[0]  - A[0]   * x[1]
y[1] = A[127] * x[14] + B[127] * x[15]
y[2] = B[1]   * x[2]  - A[1]   * x[3]
y[3] = A[126] * x[12] + B[126] * x[13]
y[4] = B[2]   * x[4]  - A[2]   * x[5]
y[5] = A[125] * x[10] + B[125] * x[11]
y[6] = B[3]   * x[6]  - A[3]   * x[7]
```

`A` is the 128-element F32 table at DLL RVA `0x6bd860`; `B` is the 128-element F32 table at RVA `0x6bda60`. Each table occupies 512 bytes. Exact hexadecimal F32 values used by these equations are included in `EVIDENCE.json`.

Both functions read source indices 0..7 and 10..15; indices 8 and 9 are not read. Each output y[0..6] is written once, in order `[0,2,4,6,5,3,1]`; source/destination strides are four bytes. The terminal bodies have no loop or recurrence. They use F32 operands and coefficients; rounding mode and FMA contraction are not identified.

## Gating and limits

The main and companion wrappers call two status gates in sequence; the terminal projection runs only if both return less than 1. The three reused layout-7 captures use the selector-zero pre-transform path. That earlier path processes 64 iterations over four F32 state words per iteration; its full recurrence is not specified by this supplement and is a separate audio transform.

A separate conditional main-table matrix path is not the seven-F32 projection: it runs only when arg6 is nonzero and arg7 equals 1, copies `count*8` bytes of pointers, then applies an in-place 6×6 F32 matrix transform. Runtime condition values and execution were not captured, and the matrix coefficients are not part of this package.

The formulas close only the terminal projection. They do not join the generated LFE tables to the selected frame-record/bitwriter path, establish LFE exponent/mantissa/`lfeon` wire fields, or prove this is the current JOCForge-vs-DEE output mismatch.

## Provenance and status

- Upstream package: `DEE-LFE-SEVEN-F32-FORMULA-WASH-V1`, manifest SHA-256 `c40d0acab237e4661c6b156b156e813dc0c4cc8c9bfdcc4c26e64e19229be6ef`.
- Wash recomputed all five manifest payload hashes and ten listed source receipt hashes; all matched.
- Ghidra 12.2 read-only reports reused; DEE/runtime and dynamic observation are `NOT_RUN`.
- No PCM, EC3 payload, key, or license content was read. The sanitized supplement excludes decompilation and native source.
