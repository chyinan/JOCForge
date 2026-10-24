# LFE coefficient-pair exponent route

## Scope

This contract covers the selected 5.1 LFE route for two blocks of seven caller-supplied finite real/imaginary coefficient pairs. When LFE is present, the wrapper invokes this coefficient-pair route for LFE regardless of the tested full-band mode/format choices. It does not produce the coefficient rows, select the parent configuration, or define parent reset/fatal/EOF behavior; those F09 stages remain open.

## Input and output shape

- Two blocks, each with `real: F32[7]`, `imaginary: F32[7]`, and `output: I16[7]`.
- `count=7` and `cap=48` are wrapper constants in this slice.
- The caller may also provide full-band mode, format, and bias values. Tested combinations are in `INPUTS.json`; the LFE calculation does not read or add bias.
- Calls reuse the same buffers and overwrite all seven output entries. The surrounding tail canary remains unchanged. The wrapper returns 0 in every supplied case.

The tested coefficients are finite, exactly representable values from the supplied cycles. Non-finite values, wider coefficient domains, invalid pointers/lengths, and native malformed-input behavior are outside the contract. The native route has no bounds validator in the observed leaf.

## Per-bin calculation

Perform separate binary32 operations in source order:

```text
real_square = F32(real * real)
imag_square = F32(imaginary * imaginary)
energy = F32(real_square + imag_square)
energy = clamp(energy, -1.0, 1.0)

if energy == 0.0:
    output = trunc_toward_zero(cap / 2)  // 24 when cap is 48
else:
    bits = U32 binary32 bits of energy
    w = (2 * bits) modulo 2^32
    source_exp = -127 if w == 0 else (w >> 24) - 126
    d = clamp(-source_exp, 0, 60)
    output = arithmetic_right_shift(min_signed_I16(d, cap), 1)
```

The coefficient-pair route has no bias operand/read. For the supplied nonnegative finite square-and-sum values, the lower clamp at -1 is inactive. Do not substitute the separate POWER helper's rule without preserving this lower clamp and the zero-energy shortcut.

## Selection and evidence boundaries

With `LFE-present != 0`, LFE always uses this coefficient-pair algorithm for the tested mode/format combinations; full-band POWER and bias behavior is separate. The standalone POWER bit values in `EXPECTED.json` are produced by a separate native helper call on the same synthetic coefficients. They are explicitly not captured internal temporaries from this LFE function. The actual LFE outputs come from both LFE-only two-block calls and block 0 in a combined full-band-plus-LFE call.

Five input recipes, two consecutive calls per recipe, and two LFE blocks produce 140 LFE outputs; the independent POWER probe has 140 values; combined invocation evidence has 70 block-0 LFE values. Wash recomputed and matched all these values. The input recipes also vary full-band bias across 0, 2, and 4; LFE outputs remain unchanged while full-band POWER-path outputs change.

This does not close the real/imaginary LFE coefficient producer, external job-to-LFE selector, other LFE profiles, arbitrary floating-point environments, or the whole ADM-to-JOC chain. Product integration and audio acceptance remain `NOT_RUN`.
