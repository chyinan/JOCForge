# Selected coefficient normal equations

`build_selected_normal_equations` consumes object-major 15x48x64 and carrier-major 5x48x64 interleaved complex-F32 windows plus the 48 temporal weights. It applies each weight to each real and imaginary component before products, then computes the selected 12-band symmetric 5x5 Gram matrix and five-item cross vector with an independently rounded binary32 operation after every multiply and add.

The implementation uses band boundaries `[0,1,2,3,4,6,8,11,14,18,23,35,64]`. Each diagonal starts from the specified F32 representation of `1e-20`; other cells and every subband accumulator start at positive zero. Row/column, bin and slot iteration order follows the approved F05JC contract.

Tests regenerate the three synthetic target/carrier QMF generations, run the production temporal detector to obtain weights, and compare all 540 equations in SYNTHETIC-QMF-FIT-CHECKPOINTS-V1. Every 5x5 Gram word and five cross words match exactly. EXPECTED.json is assertion-only test material.

This slice stops at normal equations. Geometry eligibility, signed LDL solve, normalization, score/search, full/sparse representation and ID14 production remain separate stages.
