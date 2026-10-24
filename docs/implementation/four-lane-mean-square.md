# Four-lane 256-sample mean-square reducer

This direct F09G.2 slice consumes exactly 256 caller-supplied finite F32 samples at stride 1 and returns one F32 mean-square scalar. It has no persistent state. Four positive-zero accumulators are updated in ascending input order by `lane[i % 4] = R32(lane + R32(sample * sample))`; after 64 updates per lane, the reduction starts at positive zero and adds lane 0, 1, 2, then 3, followed by one F32 division by 256.

The seven input-only recipes provide all 256 U32 sample words and a bound input hash for each case. Tests compare every lane accumulation, all four final lanes, the five-entry ordered reduction trace, and the final mean. The `partition_order` and `reduction_order` cases distinguish the prescribed topology from a serial sample accumulator, balanced lane reduction, and reversed lane order.

The local intermediate trace is computed from input bits and static rules; it is not a native lane-register capture. The seven final means are marked `RUNTIME_CONFIRMED` by the package. Inputs outside the finite normal `abs <= 1` recipe domain, other counts/strides, overflow/underflow and invalid-pointer behavior have no supplied native error mapping and remain outside this helper.

Package manifest SHA-256: `7a5da39f1b20b8ffcebfe0925dd236df36c2d29552362985680c3f3b8505dbd1`.

Independent-check SHA-256: `fac6afc662b532d19e0e7e3d4932236e3e8235c397331fcd0017dfdbc634a7c0`.
