# Selected coefficient solve and normalization

`solve_selected_support` extracts active channels in ascending order, adds the approved trace regularizer and tiny diagonal, performs the selected independently rounded binary32 LDL solve, and restores signed coefficients to their original five-channel positions. Empty support rejects.

`normalize_selected_solution` follows slot-major then band-bin order for target and model energies. It weights target and carrier complex components, accumulates real and imaginary energies with the specified binary32 grouping, applies the selected square-root scale and writes supported coefficients into the 1/16 storage domain once. Unsupported channels remain positive zero. The shared quantizer validates `R32(value*32768)` against the signed-I32 range before rounding, and trial restoration uses the generated D192 formula followed by one multiplication by 16. It never overwrites the stored row.

SYNTHETIC-FIT-CELL-BOUNDARIES-V1 has manifest SHA-256 `56aa5f744191002feb44645eb64d58d29c08a3222eeb363c9872b0a51b15c8ba`; all four distributed payload bindings match. Tests independently construct each synthetic carrier/target cell, compute its normal equation, then compare every eligible initial single-channel solved, normalized, stored and trial-restored coefficient. The empty-eligibility case returns the local typed rejection. EXPECTED is assertion-only.

Candidate enumeration, trial quantization, generated score, penalty stages and final search selection remain separate. Therefore this slice does not yet produce full/sparse/index representation or an ID14 body.
