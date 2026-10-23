# F03S.5 outer coefficient postprocess

This direct slice applies the selected F03S.5 floor and dominance rules to one nonempty vector of F32 bit words, with caller-supplied finite floor and dominance-ratio words. The floor is strict: only values below the floor become positive zero. The total is accumulated from positive zero in ordinal order with a separate R32 addition at each step. The maximum keeps the earliest ordinal on ties.

If `maximum < R32(total * dominance_ratio)`, the floored vector is retained. Equality enters the one-hot branch; only the earliest maximum position receives `R32(min(total, 1))`, and every other position becomes positive zero.

The local tests bind and exercise the three selected source cases: `U-OUT-POST-FLOOR`, `U-OUT-POST-DOMINANCE-EQUAL`, and `U-OUT-POST-TIE`. Empty vectors and non-finite inputs are rejected locally. This module has no state or clock and does not run F03S.1-.4, NNLS, geometry, energy correction, selector, U04, or an outer source transaction.
