# Coefficient rows to POWER rows

## Scope

This slice computes a fixed 253-bin POWER row from caller-supplied finite real and imaginary coefficient rows, then verifies the first 217 values through the already specified raw-exponent kernel. It does not generate the coefficients from PCM/ADM, choose the parent mode/configuration, or define other modes, short blocks, LFE, or parent reset/fatal/EOF behavior. Those F09 parts remain open.

## Inputs, outputs, and preconditions

- `blocks` and `channels` are signed I16 loop counts. The tested wrapper uses either 1×1 or 2×2.
- For each block/channel pair, the caller supplies exactly 253 F32 real values and 253 F32 imaginary values.
- The producer writes 253 U32 words holding the binary32 POWER bits for each row.
- The caller supplies an I16 bias row indexed by channel and block. Before processing each row, the producer overwrites that bias slot with zero.
- The tested downstream consumer reads the first 217 POWER words and uses mode 1, format 7, cap 48, and the generated bias 0. Its output is signed I16 and has at least 217 entries.

The tested finite profile accepts zero and finite coefficients with magnitude in `[2^-20, 2]`. It excludes nonzero magnitudes below `2^-20` and non-finite values. The caller must provide valid disjoint rows of the stated lengths. The native helper does not validate bounds or malformed row shapes; behavior outside these preconditions is not specified here.

## Per-bin formula

For each bin in ascending order, with a separate F32 rounding after every operation:

```text
real_square = F32(real * real)
imag_square = F32(imaginary * imaginary)
energy = F32(real_square + imag_square)
power = min_F32(1.0, energy)
POWER[bin] = the U32 bits of power
```

The operations are separate; do not fuse either multiply with the add. The selected runtime profile had MXCSR 8096 before and after the call, round-to-nearest-even, FTZ disabled, and DAZ disabled. The helper does not set the floating-point environment; no wider arithmetic environment is claimed. Within this finite nonnegative-energy profile, the cap is 1.0.

The producer processes 31 groups of 8 bins and then 5 tail bins. Output extent is always 253, independent of the downstream 217-bin consumer extent.

## Row mapping and state effects

The controlled 2×2 vectors bind input coefficient rows by `[block][channel][bin]`, POWER rows by `[channel][block][bin]`, and bias slots by `[channel][block]`. Loop order is block ascending, then channel ascending, then bin ascending. Repeated calls reuse the same buffers with changed inputs; all 253 POWER values are overwritten each call and each processed bias slot becomes zero. No hidden persistent numeric state is established.

Controlled producer calls return 0 and preserve the surrounding output canaries. Floating-point status flags may change. The first 217 POWER words are then passed to the raw-exponent kernel; complete parent selector and owner lifecycle contracts remain separate.

## Acceptance and limits

`INPUTS.json` contains six input-only recipes. `EXPECTED.json` binds 3,036 POWER words and 2,387 subsequent raw exponents. Wash recomputed all of them from the stated F32 operation order and the raw-exponent contract. The native run also joined 150 naturally observed consumer calls to producer rows by generation, pointer, count, and content hash; 30 first-generation all-zero rows had no matched producer event and are excluded from that join count.

This package does not close the transform that creates real/imaginary coefficients, the external configuration writer, the full bias lifecycle, wider modes, LFE, non-finite/underflow behavior, or whole ADM-to-E-AC-3 JOC output. Independent product integration and audio acceptance are `NOT_RUN`.
