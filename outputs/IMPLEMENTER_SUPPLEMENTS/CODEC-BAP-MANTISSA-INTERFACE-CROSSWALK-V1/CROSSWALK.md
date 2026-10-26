# E-AC-3 BAP to mantissa interface crosswalk

## Decision

The approved selected specifications fully define the narrow CODEC1 to CODEC2 interface. No additional native or DLL reverse engineering is required for this seam.

This result does not rederive BAP mathematics, quantizer mathematics, frame syntax, CRC, search order, or transaction behavior. It identifies the exact same-trial inputs and outputs that must be joined.

## Same-trial shape and identity

For one source frame, F09 and F10 build the coefficient and exponent plan once. F12 candidate trials reuse that fixed plan. Candidate `q` changes BAP, mantissa codes, and scratch capacity only; it does not rerun POWER, exponent refresh, normalization, or transforms (`specs/09-exponents.md:65`, `specs/10-exponent-normalization.md:59`, `specs/12-selected-source-frame-composition.md:18-21,46`).

The main trial uses one U10 `q` for all 36 active BAP elements:

- 6 blocks;
- per block: 5 FBW elements with `end_exclusive=217`, followed by 1 LFE element with `end_exclusive=7`;
- `coarse=q>>4`, `fine=q&15` for every element;
- tuple `[slow_decay,fast_decay,slow_gain,db_per_bit,floor]=[2,1,1,F09G[b],7]`;
- main fast-gain code `1`, start bin `0`, 48 kHz, coupling/delta/AHT disabled;
- `all_active_fine_zero` is computed once over the same trial's 36 active elements. Main search starts at `q=16`, so it does not take the all-zero bypass.

Authority: `specs/12-selected-codec-wire.md:47-57` and `specs/12-bit-allocation.md:7-11`.

Each F12A call returns a U8 BAP row exactly as long as its decoded-exponent row. The selected rows are fully active `[0,end_exclusive)`: 217 FBW bins or 7 LFE bins. There is no selected-profile bandwidth tail to append, trim, or infer. F12A has no persistent cross-element or cross-block state (`specs/12-bit-allocation.md:28,64`).

## Positional join into quantization

The join key is the tuple `(source_generation, q_trial, block, channel, bin)`.

For each key, the mantissa quantizer consumes:

1. `bap[b][channel][bin]` returned by the F12A call for the same `q`;
2. `clipped_coefficient[b][channel][bin]` from the F09LB staged PRIMARY row for FBW or staged MAIN row for LFE;
3. `decoded_exponent[b][channel][bin]` from the fixed F10 expanded plan used as that F12A call's input.

The coefficient and decoded exponent therefore have the same active row length as the BAP row. The coefficient must come from the same-source validated strict-clip staged bank, never from the unclipped transform row. F09LB validates selected FBW count 217, LFE count 7, finite F32 words, and LFE raw exponent length 7 before staging (`specs/09-lfe-gate-mantissa-boundaries.md:27-30,34-46,56-61`). The quantizer then scales the coefficient by `2^decoded_exponent` (`specs/12-selected-codec-wire.md:61-71`).

## Item order, grouping, and cost

Within each block, flatten complete active rows in `[L,C,R,Ls,Rs,LFE]` order, with bins increasing inside each channel. This produces `5*217+7=1092` items per block and `6552` items per frame.

Create fresh BAP1, BAP2, and BAP4 groupers at each block boundary. Each grouper advances only when its own BAP occurs, so a group may cross channel boundaries but cannot cross a block boundary. At the final occurrence for the block, fill incomplete groups with codes 1, 2, and 5 respectively. Emit the grouped field at its first member's item position; later members emit no field. Other nonzero BAP values emit one field at their own item position; BAP0 emits no mantissa field (`specs/12-selected-codec-wire.md:73-81`).

The grouped cost identity is:

```text
ceil(count_bap1/3)*5
+ ceil(count_bap2/3)*7
+ ceil(count_bap4/2)*7
+ sum(count_bapN * width[N], N in {3,5,6..15})
```

The published width table is `[0,5,7,3,7,4,5,6,7,8,9,10,11,12,14,16]`. `specs/11-converter.md:29` independently uses the same histogram identity, including cross-channel grouping and per-block reset. That converter formula is a consistency check; it does not replace the main trial's actual scratch write.

For the main trial, the cost feedback is the scratch bitwriter cursor after writing real header, block, EMDF, and mantissa fields. A candidate is feasible only when all construction succeeds and the body cursor is at most `32768-34=32734` bits. The outer main search receives candidate success/failure and retains the last successful scratch result (`specs/12-selected-codec-wire.md:139-146`). There is no separate main `mantissa_cost` reservation or F05Q 4344-bit cap.

## Failure boundary

The boundary is complete:

- Before trial construction: invalid shape/type/range, stale generation/source, invalid plan, invalid BSI/EMDF/state, missing provider, or unsupported profile is programme-fatal (`specs/12-selected-codec-wire.md:166-170`).
- Inside a valid main trial: BAP network failure, BAP3/5 sentinel, non-finite/unrepresentable quantizer intermediate, grouping/field failure, or body capacity overflow makes only that `q` candidate infeasible. Its scratch BAP rows, groupers, and bit cursor are discarded (`specs/12-codec-and-wire.md:61,65`; `specs/12-selected-source-frame-composition.md:50-54`).
- If no candidate succeeds, the source becomes fatal and committed codec state remains unchanged. A successful best candidate remains staged; the existing parent source transaction commits once only after its required publication/suppression gate succeeds.
- Converter producer errors retain the separate F11 fatal protocol and must not be swallowed as main-trial infeasibility (`specs/12-codec-and-wire.md:63`).

## Implementation seam

A minimal typed interface can carry one immutable frame plan plus one `q` into a trial builder:

```text
FramePlan {
  clipped_coefficients: FBW[6][5][217] + LFE[6][7],
  decoded_exponents:    FBW[6][5][217] + LFE[6][7],
  db_per_bit_code:      [6],
  fixed syntax/BSI/EMDF inputs and generation identity
}

Trial(q, FramePlan) -> CandidateInfeasible | ScratchFrame
```

Inside `Trial`, generate BAP rows for all 36 elements with the same `q`, join rows positionally to the immutable coefficient/exponent plan, rebuild the three groupers per block, write the actual scratch body, and return only complete scratch success. This is an implementation crosswalk from existing contracts, not a new native evidence requirement.

## Conclusion

No unresolved native field, ownership join, active-bin rule, filler-cost rule, or error boundary remains at this CODEC1 to CODEC2 seam. Rust wiring and full integration testing may still be absent, but that is implementation work rather than missing reverse-engineering evidence.
