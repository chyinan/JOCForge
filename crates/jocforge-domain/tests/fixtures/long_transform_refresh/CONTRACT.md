# Long transform to exponent refresh composition

## Scope

This package defines the bounded long-block path from one caller-supplied 256-word F32 block through fold, pre-rotation, a 128-complex FFT, post-rotation, 253-bin POWER generation, the first 217 raw exponents, and a 128-bin refresh transition. The inputs are synthetic finite vectors. The transform is not connected here to an ADM sample clock, a native cold-state constructor, external configuration selection, short transforms, or LFE input analysis. This does not close the complete F09 producer or the whole audio chain.

## State and interfaces

- Each controlled call consumes 256 F32 values.
- Primary and companion branches each own a caller-provided 128-word I16 history row; the vectors initialize each row to zero and reuse it across two calls.
- The fold produces a 256-word F32 temporary row. Both branches use the same pre-rotation parameters and inner FFT parameters, then produce distinct 256-word primary/companion spectra.
- Composition uses primary as real and companion as imaginary, emits 253 POWER U32 words, passes the first 217 words through the bias-zero raw-exponent mapping with mode 1/format 7/cap 48, then refreshes `[0,128)` with mode 1/special 0 and marker schedule `[1,0]`. `work` is a retained 217-word I16 row initially set to zero by each recipe.
- The two calls are consecutive state transitions in a synthetic test, not ADM sample-position identities. State ownership and reset behavior in the full product remain open.

All numeric formulas below round each stated operation to binary32. The supplied finite input/table profile uses the recorded round-to-nearest-even environment. Do not fuse operations or replace the ordered seven half-scales by one final division.

## Fold

Let `R(x)` mean a separately rounded F32 result. For each `i = 0..63`, process the following two tuples in this order:

```text
(a,b,o,s,h) = (2i,   255-2i, 128+2i, 129+2i, 64+i)
(a,b,o,s,h) = (2i+1, 254-2i, 127-2i, 126-2i, 63-i)
```

Before updating `history[h]`, read `old = history[h]` and compute:

```text
A = R(input[a] * window[b])
B = R(input[b] * window[a])
C = R(input[b] * window[b])
D = R(window[a] * input[a])
```

For primary, write `temp[o]=R(A+B)`, `temp[s]=old`, and `history[h]=R(C-D)`. For companion, write `temp[o]=old`, `temp[s]=R(A-B)`, and `history[h]=R(C+D)`. The window table is the exact 256-value F32 bit array in `PARAMETERS.json`.

## Pre-rotation, FFT, and post-rotation

For `i=0..127`, with `x=temp[2i]`, `y=temp[2i+1]`, `c=cosine[i]`, `s=sine[i]`:

```text
pre[2i]   = R(-R(c*x) + R(s*y))
pre[2i+1] = R( R(c*y) + R(s*x))
```

The FFT operates on 128 interleaved complex F32 values. Run seven stages with `half=64,32,...,1` and `groups=1,2,...,64`. For each stage process `group g` ascending and `pair p=0..half-1`; set `ia=2*(2*half*g+p)`, `ib=ia+2*half`, and read both complex operands before writing either result. Groups 0 and 1 do not consume butterfly parameters:

```text
g=0: nr=R(R(ar+br)*0.5); ni=R(R(ai+bi)*0.5)
g=1: nr=R(R(ar+bi)*0.5); ni=R(R(ai-br)*0.5)
```

For each `g>=2`, select one coefficient pair once per group and hold it fixed for all pairs in that group:

```text
c = B[g-1]
s = B[g-2] if g is odd, otherwise -B[g]
```

`B` is the 64-element butterfly F32 array in `PARAMETERS.json`. Then:

```text
nr = R(R(R(ar - R(c*br)) - R(s*bi)) * 0.5)
ni = R(R(R(ai - R(c*bi)) - R(-R(s*br))) * 0.5)
values[ia:ia+2] = (nr,ni)
values[ib:ib+2] = (R(ar-nr),R(ai-ni))
```

The coefficient index resets for each stage; groups 0/1 do not advance it. `FFT-CANARY-INPUT.json` is a 16-complex discriminator for this group-scoped selection and rejects the pre-increment, per-pair, and no-stage-reset alternatives. The selected transform itself remains 128-complex.

For post-rotation, `j=permutation[i]`, `x=fft[2j]`, `y=fft[2j+1]`:

```text
primary[2i]   = R(R(c*y) - R(s*x))
companion[2i] = R(R(s*x) - R(c*y))
primary[255-2i] = companion[255-2i] = R(R(c*x) + R(s*y))
```

`cosine`, `sine`, `permutation`, and `butterfly` are bound by exact little-endian bit arrays and hashes in `PARAMETERS.json`.

## Composition and evidence

The downstream composition uses the separately specified `COEFFICIENT-TO-POWER-ROW`, `RAW-EXPONENT-FROM-POWER-BITS`, and `EXPONENT-REFRESH-STATE-MAP` slices. `INPUTS.json` and `EXPECTED.json` bind three two-call recipes. Wash independently replayed:

- 3,072 values each at folded, pre-rotated, FFT, post-rotated, and wrapper-spectrum boundaries;
- 1,536 history values and 32 FFT index-canary outputs;
- 3,072 primary/companion composition spectrum values, 1,518 POWER words, 1,302 raw exponents, 2,604 refresh current/work words, and 6 markers.

Every replay matched the bound expected values. The producer module identity is tied to the file hash read from the actual loaded module path and a matching host rehash. Native helper output boundaries were captured; internal per-butterfly FFT stage buffers were not. Inner butterfly/order rules are `STATIC_CONFIRMED` and independently replayed; the full standalone/composed transform also has prototype postcompare. No independent product audio integration has run.

The natural 180-row transform-to-POWER pointer/content joins establish same-generation association only; they do not identify ADM sample positions or prove the full startup/reset clock. Invalid buffers, aliases, non-finite/overflow/underflow domains, other floating-point environments, short transform, and complete parent fatal/EOF/reset behavior are outside this contract. Keep the full F09/ADM-to-JOC implementation and audio acceptance `NOT_RUN`.
