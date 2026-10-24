# Long-transform refresh composition

This direct slice implements the washed synthetic 256-word long-transform path for one selected primary or companion branch, then verifies the same-call primary/companion outputs through POWER, raw exponent, and refresh. It is not connected to an ADM sample clock, an external job/config selector, or a parent cold/reset owner.

The transform returns folded, pre-rotated, FFT, post-rotated, wrapper-spectrum, and next-history bit arrays. History is an explicit caller-owned value; the helper does not mutate caller state. It consumes the package's exact fixed window, cosine, sine, permutation, and butterfly tables. Every arithmetic operation uses the existing independently rounded F32 primitives.

For each FFT stage, the group loop resets. Groups 0 and 1 use their specified special butterflies. For each group `g >= 2`, coefficients are selected once and held for every pair in that group: `c=B[g-1]`, `s=B[g-2]` when `g` is odd, otherwise `s=-B[g]`. This is not per-pair coefficient indexing. The 16-complex canary catches pre-increment, per-pair, and missing-stage-reset variants; the selected transform remains 128-complex.

Six standalone recipes, each with two calls and retained branch history, compare 3,072 values at every fold/pre/FFT/post/wrapper boundary and 1,536 history words. Three composition recipes, each with two calls, compare 3,072 primary/companion spectrum words, 1,518 POWER words, 1,302 raw exponents, 2,604 refresh current/work words, and six markers. Expected outputs are assertions after computation and are never seeds.

The `native_inner_fft_stage_buffers` boundary remains `NOT_RUN`; FFT intermediate stages are matched by washed closed-form replay and the supplied canary, not by native internal-stage capture. Native output-boundary capture, prototype replay, and static formulas remain distinct evidence classes. Product audio integration and whole F09 readiness remain `NOT_RUN`/`NO`.

Package manifest SHA-256: `4eaa2690e30ed0705404d7a6cd835e8637c2f933319d60be8812d900cb6521c1`.

Independent-check SHA-256: `0574d3f4aff6460340709216fd1d0c141c47baa68367f2c43abbc446ba2157b5`.
