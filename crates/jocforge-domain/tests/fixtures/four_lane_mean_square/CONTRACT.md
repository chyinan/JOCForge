# Four-lane mean-square reducer

## Scope and interface

This slice consumes exactly 256 caller-supplied F32 samples with stride 1 and returns one F32 mean-square value. A Wash-side API may express this as `mean_square_256(&[f32; 256]) -> f32`; that signature is a clean implementation choice, not a recovered native declaration. The function has no persistent state.

The tested input domain is zero and normal finite F32 values with `abs(sample) <= 1`. No square overflow or subnormal square occurs in these vectors. Other counts/strides, NaN/Inf, overflow/underflow, and malformed pointer behavior are outside the contract. The native generic function has no validated error/status contract for those cases; do not invent one as a native result.

## Exact arithmetic order

Initialize four independent F32 accumulators to positive zero. For each input index `i=0..255` in ascending order:

```text
square = R(sample[i] * sample[i])
lane[i mod 4] = R(lane[i mod 4] + square)
```

Each lane receives 64 values. Then reduce in this exact order, starting from positive zero:

```text
total = R(+0 + lane[0])
total = R(total + lane[1])
total = R(total + lane[2])
total = R(total + lane[3])
mean = R(total / 256.0)
```

`R` means one binary32 rounding after that operation. Do not replace the four accumulators with one serial accumulator or use a balanced/reversed final reduction.

## Evidence and acceptance

`INPUTS.json` contains seven recipes, each with all 256 input U32 bit patterns and a bound input hash. `EXPECTED.json` includes the 64 intermediate accumulations for each lane, the final four lane values, the ordered reduction trace, and the final mean. Wash independently reconstructed all 1,792 input words and replayed all lane and reduction steps. The seven final means are also runtime-captured. Native lane-register snapshots were not captured, so intermediate lane values are static-rule calculations rather than native-register observations.

The `partition_order` case distinguishes four lanes from global serial accumulation; the `reduction_order` case distinguishes the ordered lane reduction from balanced and reversed alternatives. The helper does not implement delayed PCM ownership, gain peak/history, block progression, selector history, or the complete F09G/F09 audio chain.
