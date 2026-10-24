# Coefficient pairs to POWER rows

This local slice implements only the washed `COEFFICIENT-TO-POWER-ROW` contract. It consumes caller-supplied real and imaginary binary32 coefficient rows and emits a fixed 253-bin POWER row. It does not produce coefficients from audio, select mode or configuration, provide the coefficient/bias owner, or close F09.

For each ascending bin it performs separately rounded F32 operations: square the real value, square the imaginary value, add those results, then cap the result at F32 `1.0`. The POWER output stores the resulting binary32 bits. The two-by-two wrapper consumes input rows indexed `[block][channel][bin]`, visits block then channel in ascending order, and returns rows indexed `[channel][block][bin]`. It clears each processed I16 bias slot to zero. A single-row wrapper has the same bias effect.

The admitted input profile is finite values whose magnitude is in `[2^-20, 2]`, including either zero sign. The helper does not set MXCSR; the washed package records a runtime with MXCSR `8096` before and after, round-to-nearest-even, FTZ disabled, and DAZ disabled. No environment guarantee is made beyond that recorded profile. Invalid counts, pointers, row lengths, underflow, non-finite values, and other out-of-profile inputs have no defined behavior in this slice and are not assigned invented errors.

The six input-only recipes replay 3,036 POWER U32 values. Five of those calls also exercise the selected raw-exponent consumer, covering 2,387 I16 values. The sixth covers the rounding-edge POWER row only. Expected results are compared after computation and are never used as inputs. The package-bound canary fields are checked as fixture facts; fixed-size Rust row types prevent this helper from addressing an adjacent output element.

Package manifest SHA-256: `4af64c87e7a40c3990075a3aa07d052ea0178c99ef1f1264847dfeaa7265c2d8`.

Independent-check SHA-256: `ddcf281cb1edc3d805bd65aec282403fe962412a49eaf0311c1eac19d06ecd66`.

Whole F09, coefficient generation, bias lifecycle, LFE, other row shapes, product integration, and audio acceptance remain open or `NOT_RUN`.
