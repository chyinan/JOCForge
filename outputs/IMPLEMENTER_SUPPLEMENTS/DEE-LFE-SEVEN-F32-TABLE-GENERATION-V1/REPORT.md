# DEE LFE seven-value table generation boundary

## Finding

A hash-verified, Ghidra 12.2 static report binds the selected Atmos processing path for three reused layout-7 fixtures: five FBW lanes plus lane 5 as LFE. In the selected per-block channel loop, indices below the FBW count use the ordinary channel path; the remaining lane path sends the current main-table entry and companion-table entry to separate LFE consumers. The consumer chain writes seven 32-bit floating-point values at indices 0 through 6 into each destination table.

This establishes same-owner generation of the two seven-value table entries for the scoped fixtures. It does **not** provide the numerical equations of the helper transforms in this supplement. Implementers must retrieve the hash-bound Wash source report and derive those equations before implementing this audio-affecting transform. Do not replace it with a generic low-pass or another codec's behavior.

## Conditional main-table transform

The main table pointer is also passed to a separate helper. Its static branch predicate is: argument 6 is nonzero and argument 7 equals 1. If true, the caller builds a pointer vector of `count * 8` bytes and calls an in-place floating-point stream matrix transform. The branch values and execution were not captured. The companion table is not passed at this callsite.

Treat this as a distinct, conditional audio transform. Its activation default and numerical transfer function are not established here; do not assume the branch is taken or skipped.

## Non-closure

The generated tables are not statically joined to the selected frame-record copy / bitwriter path. The generic FBW exponent code path covers FBW indices 0..4 in these fixtures and excludes LFE lane 5. LFE `lfeon`, LFE exponent, and LFE mantissa wire writers remain unbound. This supplement does not prove a difference in JOCForge output; it identifies a DEE algorithm path that must be compared against the current LFE implementation.

## Evidence scope

- Input shape: three reused layout-7 captures; five FBW lanes; six total lanes; LFE lane index 5.
- DEE execution, dynamic branch values, PCM, EC3 payload, keys, and licenses: not inspected in this slice.
- Upstream package checker: reported `NOT_RUN`. Wash independently recomputed all 42 manifest-listed artifact byte counts and SHA-256 values and checked manifest/seal linkage.
- This sanitized supplement excludes decompilation, source code, PCM, payloads, and private key/license material.
