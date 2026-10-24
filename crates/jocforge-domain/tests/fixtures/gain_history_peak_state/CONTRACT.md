# F09 gain peak, history, and reuse state

## Scope

This package covers the selected pass-0, summary-gate-1, format-7 gain path for six native-order channels over six blocks. It includes current/previous peak state, Q15 maximum state, per-block summary, five full-band gain outputs, and reuse-group minimum projection. It is separate from primary/companion FFT history, filter-state arithmetic, the external analysis input writer, and the complete F09/ADM lifecycle.

## Input and owner state

- Each block has six 256-sample finite F32 rows in native channel order.
- The tested domain has `abs(sample) <= 1.5`, previous peak finite and numerically in `[0,3]` (including negative zero), and Q15 maximum in `[0,32767]`.
- Owner state contains `previous_peak: F32[6]`, `summary: F32[6 blocks][6 channels]`, `q15_max: I16[6]`, and `gain_output: I16[6 blocks][5 full-band channels]`.
- The vectors cold-initialize those fields to +0/0. Previous peak and Q15 max carry across blocks and across the two-frame recipe; the six-by-six summary is overwritten per block. Gain rows are overwritten per block. The owner layout is semantic, not a native ABI mandate.
- The selected native order is six channels: five full-band plus LFE. LFE contributes to statistics but has no gain output.
- Pass 0, summary gate 1, format 7, full-band count 5, sample-rate code 0, and masks 0/1/3/4/5/7 are covered. A reduction profile is explicit in each recipe; do not infer it from Rust allocation alignment.

The test generator supplies explicit deterministic 256-word U32 inputs for zero, carry across frames, unaligned carry, LFE-only, weighted masks, positive/negative ties, signed-zero permutations, and Q15 half ties. Input hashes and exact generator rules are in `INPUTS.json`.

## Peak reduction and state update

For finite values, `MAX(a,b)` selects `a` only if `a>b`, otherwise it selects the second operand; `MIN(a,b)` selects `a` only if `a<b`, otherwise the second operand. This preserves the native signed-zero tie behavior.

For the aligned four-lane profile, initialize four minima and maxima from samples 0..3. For each later four-sample group, update lane `j` as `MIN(low[j], sample[j])` and `MAX(high[j], sample[j])`. Horizontally reduce `MIN(low0,low2)`, then `MIN(low1,low3)`, then those two; do the corresponding ordered MAX reduction. Flip the sign bit of the minimum and set `peak=MAX(signbit_flipped_min, maximum)`.

For the unaligned scalar profile, initialize low/high from sample 0. For each sample 1..255, update `low=MIN(low,sample)` and `high=MAX(sample,high)` in that operand order. Then use the same sign-bit flip and final MAX.

Per channel and block:

```text
current = R(2.0 * peak)
summary[block][channel] = MAX(previous_peak[channel], current)
previous_peak[channel] = current
q = clamp(lrint_RNE(R(summary * 32768.0)), -32768, 32767)
q15_max[channel] = signed_max(q15_max[channel], q)
```

`lrint_RNE` describes the effective tested half-tie result; MXCSR control bits were not read by the successful capture. The exact SSE min/max operand order and sign-bit results are in `DERIVED-TRACES.json`.

## Gain calculation

Let `p[0..5]` be the current block's six summary values, `CENTER` and `SIDE` the exact F32 values in `PARAMETERS.json`, and apply operations in this sequence with a separate F32 rounding after each indicated operation:

1. Start `v=p[channel]`.
2. If channel is 0 or 2 and `(mask & 1)==0`, set `v=R(R(p[1]*CENTER)+v)`.
3. Then: if channel is 2 and `(mask & 4)==0`, set `v=R(R(p[4]*CENTER)+v)`; else if channel is 1 and `mask==1`, sequentially add `R(p[3]*SIDE)`, `R(p[4]*SIDE)`, `R(p[0]*CENTER)`, and `R(p[2]*CENTER)` with a separate R after each add; else if channel is 0 and `(mask & 4)==0`, set `v=R(R(p[3]*CENTER)+v)`.
4. Clip as `MIN(MAX(-1.0,v),1.0)` using the same second-operand tie rule.
5. Let `word=(2*U32(bits(clipped))) mod 2^32`; `binade=(word>>24)-126` if nonzero, otherwise `-127`; `headroom=clamp(-binade,0,60)`; `gain=max(0,min(4,headroom)-1)`.

Write gain only for full-band channels 0..4. Channel 5 still updates previous/current/summary/Q15 statistics but does not run the gain emitter. Mask-specific recipes cover 0/1/3/4/5/7.

## Reuse minimum

For each full-band channel, every nonzero strategy value starts a new anchor. Subsequent zero-strategy blocks merge their signed gain into that anchor using strict `<`; after the scan, propagate the segment minimum to each block in that segment. The input recipes cover all-new anchors, two runs, and mixed runs. Reuse rows are not a substitute for exponent or normalization behavior.

## Evidence and open boundaries

Wash independently regenerated 165,888 sample words and replayed 648 peak transitions, 540 gain outputs, and 90 reuse values with zero mismatches. Native receipts cover ordinary and signed-zero/tie cases; 0-capture R45 is excluded. The derived lane-level traces are computed from explicit inputs and static instruction rules, not captured native registers. Native LFE/filter auxiliary effects are separately documented and are not a gain-helper acceptance claim.

Reset, fatal recovery, EOF/partial-block behavior, absolute ADM sample mapping, generic non-finite/overflow inputs, and the actual external `delayed_six` producer remain open. The slice is not a whole F09 or audio-product pass.
